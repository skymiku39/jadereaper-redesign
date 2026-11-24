// Replays: the core's log of a run packed into a link, and back.
//
// A replay holds the seed, the mission, the tunes and the log of what the
// player did (see core/src/replay.rs). It only plays the same run on the same
// build of the game: once the content or the core changes, an old link plays
// something else.

const VERSION = 1;

async function squeeze(bytes, stream) {
  const piped = new Blob([bytes]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(piped).arrayBuffer());
}

/** Bytes as URL-safe base64 without padding. */
function toText(bytes) {
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromText(text) {
  const s = atob(text.replace(/-/g, '+').replace(/_/g, '/'));
  return Uint8Array.from(s, (c) => c.charCodeAt(0));
}

/**
 * Pack a run into text for a link. `log` is a Uint32Array of 4 numbers per
 * entry; `seed` a number, `mission` a mission name, `tunes` values by tune name.
 */
export async function packReplay({ seed, mission, tunes, log }) {
  const head = new TextEncoder().encode(JSON.stringify({ v: VERSION, seed, mission, tunes }));
  const body = new Uint8Array(log.buffer, log.byteOffset, log.byteLength);
  const all = new Uint8Array(4 + head.length + body.length);
  new DataView(all.buffer).setUint32(0, head.length, true);
  all.set(head, 4);
  all.set(body, 4 + head.length);
  return toText(await squeeze(all, new CompressionStream('deflate-raw')));
}

/** The reverse of `packReplay`. Throws on text that is not a replay of this version. */
export async function unpackReplay(text) {
  const all = await squeeze(fromText(text), new DecompressionStream('deflate-raw'));
  const view = new DataView(all.buffer, all.byteOffset, all.byteLength);
  const n = view.getUint32(0, true);
  const head = JSON.parse(new TextDecoder().decode(all.subarray(4, 4 + n)));
  if (head.v !== VERSION) throw new Error(`replay version ${head.v}, expected ${VERSION}`);
  const rest = all.slice(4 + n);
  if (rest.length % 16) throw new Error('replay log is cut short');
  if (typeof head.seed !== 'number' || typeof head.mission !== 'string') throw new Error('replay header is incomplete');
  return { seed: head.seed >>> 0, mission: head.mission, tunes: head.tunes || {}, log: new Uint32Array(rest.buffer) };
}
