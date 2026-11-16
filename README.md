# Jade Reaper

A survivor game backbone that runs natively in the browser. The simulation is Rust compiled to WebAssembly, the picture is drawn with WebGPU, and everything on screen is styled as cut paper and ink.

## Play it online

https://skymiku39.github.io/jadereaper-redesign/ (needs a browser with WebGPU). Every push to `main` republishes it through the `Pages` workflow in `.github/workflows/pages.yml`, which checks the fonts and then deploys `index.html`, `style.css`, `src/`, `wasm/` and `assets/` as they are committed. This repository is the redesign; the version it was copied from, [skymiku39/jadereaper](https://github.com/skymiku39/jadereaper), stays playable as it was at https://skymiku39.github.io/jadereaper/.

## Run it

Serve the folder with any static file server and open it in a browser with WebGPU (current Chrome, Edge or Safari). WebGPU needs `http://localhost` or `https`, so opening `index.html` from disk will not work. The repository has a small one with no dependencies (Node 26):

```bash
npm start
```

Then open http://localhost:5173.

The built `wasm/jadereaper_core.wasm` is checked in, so Rust is only needed if you change the core.

To check that the page loads cleanly, run the smoke test. It loads the game in headless Chromium and fails on any console error or failed request; without WebGPU it checks the "Cannot start" screen instead. CI runs it on every pull request.

```bash
npm ci
npx playwright install chromium
npm run smoke
```

## Controls

| Action | Keyboard | Gamepad |
| --- | --- | --- |
| Move | WASD or arrow keys | Left stick or d-pad |
| Pause | Esc or P | Start |
| Menu move | Arrow keys or WASD | Stick or d-pad |
| Menu confirm | Enter or Space | A |
| Back | Esc | B |
| Pick a card | 1, 2, 3 (1 to 6 when choosing a martial art for an element) | Stick, then A |
| Mute | M | |

Attacks are automatic. The game pauses itself when the window loses focus.

## How the paper style works

Characters are puppets made of rigid paper parts, not sprite sheets. The reaper is a robe, a hood and a scythe. A walk cycle is two poses: tilt one way, then tilt the other with a small hop. Enemies do the same, each on its own offset. This needs one still image per part and no animation frames.

The renderer adds the rest to every part: an ink outline, a white cutout margin, a drop shadow, paper grain, and an outline that "boils" at 8 frames per second like stop-motion. Parts are drawn in code as signed distance shapes, and any part can be swapped for your own PNG. See [assets/parts/README.md](assets/parts/README.md).

The UI is HTML and CSS over the canvas: torn-edge paper sheets, tags and cards, a hand-written font and a vermilion seal, using two SVG filters defined in `index.html`.

## Layout

| Path | What it is |
| --- | --- |
| `content/game.toml` | Part, upgrade, event and mission ids, upgrade max levels and glyphs, the enemies, the martial arts, magic elements and reactions with all their numbers, mission goals, tunes (the numbers a mission's setup screen can change), state indices, ink colour |
| `core/` | Rust simulation. No dependencies, no bindgen. |
| `core/src/defs.rs` | Instance format, the random generator, re-exports the generated ids and tables |
| `core/src/schema.rs` | The shapes of the generated content tables (enemies, martial arts, elements, reactions) |
| `core/src/generated.rs` | Generated from `content/game.toml` |
| `core/src/game.rs` | State, simulation step, level-up cards, puppet posing, instance building |
| `core/src/arts.rs` | Martial arts: one function per behaviour (`arc`, `projectile`, `orbit`, `nova`) |
| `core/src/elements.rs` | Magic elements: what each effect does on a hit, the tags on enemies and the reactions between them |
| `core/src/fmath.rs` | Float functions that give the same result natively and in wasm |
| `core/src/lib.rs` | The `jr_*` functions exported to JavaScript, and tests |
| `core/src/bin/sim.rs` | The balance tool: many native runs with bots, as a table |
| `wasm/` | Built core |
| `src/main.js` | Loads the core, frame loop, game modes |
| `src/renderer.js` | WebGPU setup, part image atlas, drawing |
| `src/shader.wgsl` | Paper shapes, cutout shading, paper background |
| `src/input.js` | Keyboard and gamepad |
| `src/audio.js` | Synthesized sound effects (no audio files) |
| `src/ui.js` | Screens, HUD, menu focus |
| `src/save.js` | Records and settings in localStorage |
| `src/replay.js` | Packs a run's replay into a link and back |
| `src/data.js` | Re-exports the generated tables from `src/generated.js` |
| `assets/parts.json` | Optional image overrides for parts |
| `assets/fonts/` | Bundled UI fonts, cut down to the game's characters, with their licenses |
| `tools/` | Font subset and check scripts, the local server, the smoke test, and `shot.mjs`, which takes pictures of a bot playing (`play.mjs` drives the browser for it) |
| `tools/gen/` | Generator for the id constants (a separate Rust crate) |

## Fonts

The Traditional Chinese UI uses bundled fonts: [Bakudai](https://github.com/max32002/bakudaifont) (莫大毛筆) for titles, buttons and brush marks, and [LXGW WenKai TC](https://github.com/lxgw/LxgwWenkaiTC) for body text. [Caveat Brush](https://fonts.google.com/specimen/Caveat+Brush) draws numbers in every language. The English UI still uses system fonts (issue #3). All three fonts are under the SIL Open Font License 1.1; the license texts are in `assets/fonts/`.

The full fonts are 12 to 16 MB each, so `assets/fonts/` holds copies cut down to only the characters the game shows, about 270 KB in total. After changing any Chinese text, rebuild them and commit `assets/fonts/`:

```bash
npm ci
tools/subset-fonts.sh
```

The scripts need Node 26 (see `.node-version`) and the dev dependencies that `npm ci` installs into `node_modules/`; the game itself needs neither. `tools/subset-fonts.sh` collects every Chinese character in `src/*.js` and `index.html`, downloads the pinned source fonts into `.font-cache/`, and writes the woff2 files. `tools/check-fonts.sh` checks the committed fonts against the text without downloading anything, and CI runs it on every pull request.

## How core and host talk

Each frame the host calls `jr_input`, then `jr_update(dt)`. The core turns that real time into whole simulation steps of 1/60 s, so the same seed and inputs always play the same run. The host then reads three buffers out of wasm memory:

- Instances: 12 floats per paper part (`x, y, half width, half height, rotation, part id, flash, seed, r, g, b, a`). These bytes go to the GPU unchanged.
- State: a fixed run of floats for the HUD (health, experience, time, offered cards and so on; `state_floats` in the content file says how many). Indices are the `[[state]]` entries in `content/game.toml`, generated as `STATE` in JavaScript and `S_*` in Rust.
- Events: 4 floats each (`type, x, y, value`), used for sound.

When the core sets the "choosing" flag the host shows the offered cards and answers with `jr_choose(slot)`. A card is an upgrade id, or `number of upgrades + element id - 1` for an element card. After an element card the choice stays open and the "attach" state holds that element's id: the host then shows the held martial arts and answers with `jr_choose(art id)`, which puts the element on that art in place of the one it carried. A run starts with `jr_start(seed, mission)`; the core sets "won" when the mission's goal is met and "over" when the run is lost, and stops the run either way. A run is lost when the player dies; three missions can also be lost while the player lives (the pursuit target escapes, the escorted villager falls, the shrine falls), and each sets its own state flag so the game-over screen can say why.

### Replays

The core logs what the player does in a run (`core/src/replay.rs`): the movement input and the size of the visible area whenever they change, and every pick of a card, each with the step it happened at. The game-over and mission-complete screens have a "Copy replay link" button; the link holds the seed, the mission, the tunes and that log, packed by `src/replay.js`, and opening it plays the run back exactly, with a "Replay" tag on the HUD. A replay is never recorded in the save. `window.jadereaper.replayLink()` gives the link of the current run from the console. A link only plays the same run on the same build of the game: after the content or the core changes, an old link plays something else.

## Build the core

`rust-toolchain.toml` pins the Rust version and lists the wasm target, rustfmt and clippy; rustup installs them on the first `cargo` command.

```bash
./build.sh
```

The script first regenerates the id constants from `content/game.toml`, then builds the wasm.

Run the simulation tests natively (they also check that the generated files are current):

```bash
cargo test --release --manifest-path core/Cargo.toml
```

## Looking at the game without playing it

`npm run shot` plays a mission with a bot in headless Chromium and saves pictures of it, game view and UI together (after `npm ci`). The options are listed at the top of `tools/shot.mjs`. For example, to see fire meet lightning on a thick crowd:

```bash
npm run shot -- --give scythe:3:fire --give pulse:2:lightning --tune spawn_rate=4 --tune enemy_hp=4 --seconds 20 --every 4 --out /tmp/shot.png
```

`--give` uses `jr_give(art, level, element)`, which the core exports for tools and the browser console (`jadereaper.core.jr_give(3, 2, 6)` gives the pulse at level 2 with lightning). The game itself never calls it.

## Balance by numbers

`core/src/bin/sim.rs` plays many runs natively with simple bots, with no drawing and no browser, and prints how they went:

```bash
cargo run --release --manifest-path core/Cargo.toml --bin sim
```

Without options it prints a row for each of three bots (one stands still, one walks a square, one backs away from whatever is near) and four ways of picking cards, 24 endless runs of up to five minutes each, in about a quarter of a minute. The columns are runs won and runs that ended in death, and the mean time, level, kills, kills per minute and health lost per minute. `--element each` prints a row for no element and one for each element carried by every art, which is how the numbers in `[[element]]` are compared; `--mission`, `--tune name=value`, `--seeds`, `--seconds` and `--csv` do what they say. The options are listed at the top of the file.

The bots are crude: they show which of two numbers is larger, not how the game feels.

## Extending

Ids live in one place, `content/game.toml`. After editing it, run `./build.sh` (or just the generator, `cargo run --release --manifest-path tools/gen/Cargo.toml`) and commit the content file together with `core/src/generated.rs`, `src/generated.js` and `src/shader.wgsl`. A part named `player_body` becomes `P_PLAYER_BODY` in Rust, `PART_PLAYER_BODY` in the shader and `PART_IDS.player_body` in JavaScript.

- New enemy: add an `[[enemy]]` entry to `content/game.toml` (its numbers, part, colour and how it joins the horde; the comment above the entries says what each key means). No code changes, unless it needs a new part shape or a mission is to place it by name.
- New mission: add a `[[mission]]` entry (next id, name, goal) to `content/game.toml`, its win rule in `Game::mission_won` and its progress in `Game::progress` in `core/src/game.rs`, and its text as `mission.<name>.name` and `.desc` in every language in `src/i18n.js` (a timed mission also has `.hud`, the label beside its countdown). The title screen builds its button from the generated list, and the button opens the mission's setup screen; how the HUD shows the goal is in `hud` in `src/ui.js`. A mission with its own state (things on the map, a target, waves) keeps its rules, numbers, drawing and tests in `core/src/mission_<name>.rs`, held as one field on `Game` and called from a few hooks in `game.rs`. Some of these files are declared in `core/src/lib.rs` and some from `game.rs` with `#[path]` (those reach private parts of `Game`); the two ways are not unified yet.
- New tune (a number of a mission that the setup screen can change for a test run): add a `[[tune]]` entry (next id, name, mission or `"any"`, default, min, max, step) to `content/game.toml`, read it in the core as `self.tune[T_<NAME>]` instead of a constant, and add its label as `tune.<name>` in every language in `src/i18n.js`. A tune named `<mission>_goal` is that mission's goal. The setup screen builds its steppers from the generated list. A run with any tune off its default is a test run and is not recorded; the checks and the recorded runs only ever use the defaults.
- New passive upgrade: add an `[[upgrade]]` entry (next id, name, max level, glyph) to `content/game.toml`, its effect where the stat is read in the core, and its card text as `upgrade.<id>.name` and `upgrade.<id>.desc` in every language in `src/i18n.js`.
- New martial art of an existing behaviour: add an `[[upgrade]]` entry for its card and an `[[art]]` entry of the same name (behaviour, part, and its numbers per level; the comment above `[[art]]` in the content file says what each means), plus the card text. No Rust or JavaScript changes. A new behaviour is a new function in `core/src/arts.rs` and a name in `BEHAVIOURS` in `tools/gen/src/content.rs`. The player holds at most `max_arts` arts.
- New element: add an `[[element]]` entry (next id, name, glyph, colour, effect and the effect's numbers) and its text as `element.<name>.name` and `.desc`, then run `tools/subset-fonts.sh` for the new glyph. An element with an existing effect needs no code; a new effect is a new arm in `Game::strike` (and `Game::tick_tags` if it does something over time) in `core/src/elements.rs`, and a name in `EFFECTS` in the generator. The `start_element` tune's `max` and its label list the element ids, so raise them too.
- New reaction: add a `[[reaction]]` entry naming two elements that leave a tag, its kind and its numbers. A new kind is a new arm in `Game::react`.
- Changing a number of an art, an element or a reaction is an edit of the content file and `./build.sh`.
- New text: add a key to every language in `src/i18n.js`, then use `t(key)` in code or a `data-i18n` attribute in `index.html`. Missing keys fall back to English.
- New language: add its code to `LANGS` and a table to `STRINGS` in `src/i18n.js`, and to start new players in it, set `DEFAULT_LANG`.
- New part shape: add a `[[part]]` entry to `content/game.toml`, then a `case PART_<NAME>` in `shape()` in `src/shader.wgsl` (and in `isFlat` if it is a flat mark).

## What is in the backbone

- Four enemy kinds (blot, wisp, brute, elite every 90 seconds) with time-scaled health and spawn rate, capped at 500 on screen
- Four martial arts (scythe, talisman, jade beads, spirit pulse), defined as data, and six passive upgrades, with level-up choices
- Seven magic elements (metal, wood, water, fire, earth, lightning, holy). An element card turns up among the level-up cards now and then; the player puts it on one martial art, which then adds the element's effect to every hit: a guard, a slow heal, a delayed second blow, a burn, a throw and hold, a chain to nearby enemies, or extra damage to spirits. Three pairs react when both tags meet on one enemy (fire and lightning, water and lightning, water and fire). Every number is a first pick that nobody has played yet. Two tunes on every setup screen help to try them: the chance of an element card and an element to start with
- Three tunes, off by default, to try ways of putting pressure on a player who only runs away: `chasers` (a share of the horde are hounds, which outrun the player), `ambush` (enemies come from where the player is heading) and `arena_radius` (a walled field). With the balance tool's bot that only backs away, which otherwise never dies, 15% hounds end 12 runs of 32 within five minutes and a wall of radius 600 ends 7; ambushes alone end none
- Experience gems, health drops, pickup magnet, knockback, invulnerability frames, camera shake
- Every mission opens with an accept phase (`core/src/accept.rs`): the player starts away from a circle, walks to it and stands in it for a few seconds, and only then does the mission begin. No enemies come before that. The distance and the time are tunes; a time of 0 skips the phase. Each edge-of-screen marker carries a tag with the distance to what it points at.
- Missions picked on the title screen: "Hold Out" (stay alive for five minutes), "Cull" (cut down 1,500 spirits), endless, and nine prototypes tuned against bots only: break the formation (destroy five nodes that spawn spirits), arena (ten waves, one at a time), retrieval (carry three relics home, slower while carrying), pursuit (kill a fleeing target before it reaches the exit), escort (walk a villager to a shrine), hold the shrine (stand in its circle for five minutes), break out (reach a far exit), boss fight (a boss with a charge, a ring of ink shots and a summoned pack) and vendetta (kill a marked group at a far place)
- Title, settings, level-up, pause, game-over and mission-complete screens, playable by keyboard or gamepad
- Saved best time, kills and level for endless, tries and wins per mission, sound and language settings
- Traditional Chinese (the default) and English text, switchable in settings

Not included yet: touch controls, damage numbers, music, meta progression between runs, and a WebGL fallback for browsers without WebGPU.

The ranked plan for what comes next is in [ROADMAP.md](ROADMAP.md).
