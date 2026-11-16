// Take pictures of the game being played by a bot, game view included.
//
//     npm run shot -- [options]
//
//   --mission <name>         the mission to play (default endless)
//   --seconds <n>            how long to play before the last picture (default 20)
//   --every <n>              also take a picture every n seconds
//   --out <path>             where the picture goes (default shot.png; with
//                            --every, shot-005.png and so on beside it)
//   --size <w>x<h>           the window (default 1280x720)
//   --lang <code>            en or zh-Hant (default zh-Hant)
//   --tune <name>=<value>    a tune of the setup screen; may be repeated
//   --give <art>:<level>[:<element>]   a martial art to hold from the start,
//                            with an element on it; may be repeated
//   --still                  stand still instead of walking in a square
//
// The accept phase is skipped unless --tune accept_time=... says otherwise.
// Example, to look at fire meeting lightning:
//     npm run shot -- --give scythe:3:fire --give pulse:2:lightning --seconds 15

import { dirname, extname, join, basename } from 'node:path';
import { openGame } from './play.mjs';

const args = process.argv.slice(2);
const opt = { mission: 'endless', seconds: 20, every: 0, out: 'shot.png', size: '1280x720', lang: 'zh-Hant', still: false };
const tuning = { accept_time: 0 };
const gives = [];
for (let i = 0; i < args.length; i++) {
  const [flag, value] = [args[i], args[i + 1]];
  if (flag === '--still') {
    opt.still = true;
  } else if (flag === '--tune') {
    const [name, v] = value.split('=');
    tuning[name] = Number(v);
    i++;
  } else if (flag === '--give') {
    const [art, level, element] = value.split(':');
    gives.push([art, Number(level), element]);
    i++;
  } else if (flag.startsWith('--') && flag.slice(2) in opt && value !== undefined) {
    opt[flag.slice(2)] = typeof opt[flag.slice(2)] === 'number' ? Number(value) : value;
    i++;
  } else {
    console.error(`shot: unknown option ${flag}`);
    process.exit(2);
  }
}
const [width, height] = opt.size.split('x').map(Number);

const game = await openGame({ width, height, lang: opt.lang, tuning });
try {
  if (!game.webgpu) throw new Error('this browser has no WebGPU, so there is nothing to draw');
  await game.start(opt.mission);
  for (const [art, level, element] of gives) await game.give(art, level, element);
  const numbered = (t) => join(dirname(opt.out), `${basename(opt.out, extname(opt.out))}-${String(t).padStart(3, '0')}${extname(opt.out)}`);
  let next = opt.every;
  await game.play(opt.seconds, {
    still: opt.still,
    each: async (t) => {
      if (opt.every && t >= next) {
        await game.shot(numbered(next));
        next += opt.every;
      }
    },
  });
  await game.shot(opt.out);
  const s = await game.state();
  const S = game.data.state;
  console.log(`shot: ${opt.out} at ${s[S.TIME].toFixed(1)} s, level ${s[S.LEVEL]}, ${s[S.KILLS]} kills, ${s[S.ENEMIES]} enemies, health ${Math.ceil(s[S.HP])}`);
} catch (err) {
  game.problems.push(`shot failed: ${err.message}`);
} finally {
  await game.close();
}
if (game.problems.length) {
  for (const problem of game.problems) console.error(`  ${problem}`);
  process.exitCode = 1;
}
