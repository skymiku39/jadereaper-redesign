# Jade Reaper

A survivor game backbone that runs natively in the browser. The simulation is Rust compiled to WebAssembly, the picture is drawn with WebGPU, and everything on screen is styled as cut paper and ink.

## Play it online

This repository, the redesign, has no site of its own yet. The version it was copied from, [skymiku39/jadereaper](https://github.com/skymiku39/jadereaper), stays playable at https://skymiku39.github.io/jadereaper/ (needs a browser with WebGPU). Once GitHub Pages is enabled here, every push to `main` publishes through the `Pages` workflow in `.github/workflows/pages.yml`, which checks the fonts and then deploys `index.html`, `style.css`, `src/`, `wasm/` and `assets/` as they are committed; until then that workflow fails.

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
| Pick upgrade | 1, 2, 3 | Stick, then A |
| Mute | M | |

Attacks are automatic. The game pauses itself when the window loses focus.

## How the paper style works

Characters are puppets made of rigid paper parts, not sprite sheets. The reaper is a robe, a hood and a scythe. A walk cycle is two poses: tilt one way, then tilt the other with a small hop. Enemies do the same, each on its own offset. This needs one still image per part and no animation frames.

The renderer adds the rest to every part: an ink outline, a white cutout margin, a drop shadow, paper grain, and an outline that "boils" at 8 frames per second like stop-motion. Parts are drawn in code as signed distance shapes, and any part can be swapped for your own PNG. See [assets/parts/README.md](assets/parts/README.md).

The UI is HTML and CSS over the canvas: torn-edge paper sheets, tags and cards, a hand-written font and a vermilion seal, using two SVG filters defined in `index.html`.

## Layout

| Path | What it is |
| --- | --- |
| `content/game.toml` | Part, upgrade, event and mission ids, upgrade max levels and glyphs, mission goals, tunes (the numbers a mission's setup screen can change), state indices, ink colour |
| `core/` | Rust simulation. No dependencies, no bindgen. |
| `core/src/defs.rs` | Instance format, enemy table, re-exports the generated ids |
| `core/src/generated.rs` | Generated from `content/game.toml` |
| `core/src/game.rs` | State, simulation step, puppet posing, instance building |
| `core/src/fmath.rs` | Float functions that give the same result natively and in wasm |
| `core/src/lib.rs` | The `jr_*` functions exported to JavaScript, and tests |
| `wasm/` | Built core |
| `src/main.js` | Loads the core, frame loop, game modes |
| `src/renderer.js` | WebGPU setup, part image atlas, drawing |
| `src/shader.wgsl` | Paper shapes, cutout shading, paper background |
| `src/input.js` | Keyboard and gamepad |
| `src/audio.js` | Synthesized sound effects (no audio files) |
| `src/ui.js` | Screens, HUD, menu focus |
| `src/save.js` | Records and settings in localStorage |
| `src/data.js` | Re-exports the generated tables from `src/generated.js` |
| `assets/parts.json` | Optional image overrides for parts |
| `assets/fonts/` | Bundled UI fonts, cut down to the game's characters, with their licenses |
| `tools/` | Font subset and check scripts, the local server and the smoke test |
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
- State: 32 floats for the HUD (health, experience, time, offered upgrades and so on). Indices are the `[[state]]` entries in `content/game.toml`, generated as `STATE` in JavaScript and `S_*` in Rust.
- Events: 4 floats each (`type, x, y, value`), used for sound.

When the core sets the "choosing" flag the host shows the upgrade cards and answers with `jr_choose(slot)`. A run starts with `jr_start(seed, mission)`; the core sets "won" when the mission's goal is met and "over" when the run is lost, and stops the run either way. A run is lost when the player dies; three missions can also be lost while the player lives (the pursuit target escapes, the escorted villager falls, the shrine falls), and each sets its own state flag so the game-over screen can say why.

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

## Extending

Ids live in one place, `content/game.toml`. After editing it, run `./build.sh` (or just the generator, `cargo run --release --manifest-path tools/gen/Cargo.toml`) and commit the content file together with `core/src/generated.rs`, `src/generated.js` and `src/shader.wgsl`. A part named `player_body` becomes `P_PLAYER_BODY` in Rust, `PART_PLAYER_BODY` in the shader and `PART_IDS.player_body` in JavaScript.

- New enemy: add a row to `KINDS` in `core/src/defs.rs` and a spawn rule in `Game::spawn`.
- New mission: add a `[[mission]]` entry (next id, name, goal) to `content/game.toml`, its win rule in `Game::mission_won` and its progress in `Game::progress` in `core/src/game.rs`, and its text as `mission.<name>.name` and `.desc` in every language in `src/i18n.js` (a timed mission also has `.hud`, the label beside its countdown). The title screen builds its button from the generated list, and the button opens the mission's setup screen; how the HUD shows the goal is in `hud` in `src/ui.js`. A mission with its own state (things on the map, a target, waves) keeps its rules, numbers, drawing and tests in `core/src/mission_<name>.rs`, held as one field on `Game` and called from a few hooks in `game.rs`. Some of these files are declared in `core/src/lib.rs` and some from `game.rs` with `#[path]` (those reach private parts of `Game`); the two ways are not unified yet.
- New tune (a number of a mission that the setup screen can change for a test run): add a `[[tune]]` entry (next id, name, mission or `"any"`, default, min, max, step) to `content/game.toml`, read it in the core as `self.tune[T_<NAME>]` instead of a constant, and add its label as `tune.<name>` in every language in `src/i18n.js`. A tune named `<mission>_goal` is that mission's goal. The setup screen builds its steppers from the generated list. A run with any tune off its default is a test run and is not recorded; the checks and the recorded runs only ever use the defaults.
- New upgrade: add an `[[upgrade]]` entry (next id, name, max level, glyph) to `content/game.toml`, its effect in `Game::weapons` or where the stat is read, and its card text as `upgrade.<id>.name` and `upgrade.<id>.desc` in every language in `src/i18n.js`.
- New text: add a key to every language in `src/i18n.js`, then use `t(key)` in code or a `data-i18n` attribute in `index.html`. Missing keys fall back to English.
- New language: add its code to `LANGS` and a table to `STRINGS` in `src/i18n.js`, and to start new players in it, set `DEFAULT_LANG`.
- New part shape: add a `[[part]]` entry to `content/game.toml`, then a `case PART_<NAME>` in `shape()` in `src/shader.wgsl` (and in `isFlat` if it is a flat mark).

## What is in the backbone

- Four enemy kinds (blot, wisp, brute, elite every 90 seconds) with time-scaled health and spawn rate, capped at 500 on screen
- Four weapons (scythe, talisman, jade beads, spirit pulse) and six passive upgrades, with level-up choices
- Experience gems, health drops, pickup magnet, knockback, invulnerability frames, camera shake
- Every mission opens with an accept phase (`core/src/accept.rs`): the player starts away from a circle, walks to it and stands in it for a few seconds, and only then does the mission begin. No enemies come before that. The distance and the time are tunes; a time of 0 skips the phase. Each edge-of-screen marker carries a tag with the distance to what it points at.
- Missions picked on the title screen: "Hold Out" (stay alive for five minutes), "Cull" (cut down 1,500 spirits), endless, and nine prototypes tuned against bots only: break the formation (destroy five nodes that spawn spirits), arena (ten waves, one at a time), retrieval (carry three relics home, slower while carrying), pursuit (kill a fleeing target before it reaches the exit), escort (walk a villager to a shrine), hold the shrine (stand in its circle for five minutes), break out (reach a far exit), boss fight (a boss with a charge, a ring of ink shots and a summoned pack) and vendetta (kill a marked group at a far place)
- Title, settings, level-up, pause, game-over and mission-complete screens, playable by keyboard or gamepad
- Saved best time, kills and level for endless, tries and wins per mission, sound and language settings
- Traditional Chinese (the default) and English text, switchable in settings

Not included yet: touch controls, damage numbers, music, meta progression between runs, and a WebGL fallback for browsers without WebGPU.

The ranked plan for what comes next is in [ROADMAP.md](ROADMAP.md).
