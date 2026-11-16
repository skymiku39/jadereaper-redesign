// Determinism check: the same scripted run must end in the same state hash in
// the native build and in the committed wasm. It plays every mission, once
// with the starting build and once with every martial art and an element on
// each, so the mission rules, the elements and their reactions are all covered.
//
//     npm run determinism
//
// Needs Rust (for the native side). A mismatch means some float operation
// gives different results on the two targets; see docs/README.md.

import { execFileSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';

const TICKS = 60 * 150; // two and a half minutes of play, past the first elite
const SEEDS = [1, 987654321];

const root = new URL('..', import.meta.url);
const bytes = await readFile(new URL('wasm/jadereaper_core.wasm', root));
const core = (await WebAssembly.instantiate(bytes, {})).instance.exports;
const { MISSIONS } = await import(new URL('src/generated.js', root));

const native = execFileSync(
  'cargo',
  ['run', '--quiet', '--release', '--manifest-path', 'core/Cargo.toml', '--example', 'scripted_run', '--', String(TICKS), ...SEEDS.map(String)],
  { cwd: root, encoding: 'utf8' },
);
// Keyed by "mission armed seed".
const nativeHash = new Map(native.trim().split('\n').map((line) => {
  const [mission, armed, seed, hash] = line.split(' ');
  return [`${mission} ${armed} ${seed}`, Number(hash)];
}));

let failed = 0;
let runs = 0;
for (const mission of MISSIONS) {
  const differ = [];
  for (const armed of [0, 1]) {
    for (const seed of SEEDS) {
      runs++;
      const wasm = core.jr_scripted_mission(seed, TICKS, mission.id, armed) >>> 0;
      const want = nativeHash.get(`${mission.id} ${armed} ${seed}`);
      if (wasm !== want) differ.push(`seed ${seed}${armed ? ' armed' : ''}: wasm ${wasm}, native ${want}`);
    }
  }
  failed += differ.length;
  console.log(`determinism: ${mission.name}: ${differ.length ? 'MISMATCH' : 'same'}`);
  for (const line of differ) console.log(`  ${line}`);
}
if (failed) {
  console.error(`determinism: failed, ${failed} of ${runs} runs differ`);
  process.exitCode = 1;
} else {
  console.log(`determinism: ok, ${runs} runs`);
}
