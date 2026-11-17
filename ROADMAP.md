# Roadmap

This is the plan for growing the Jade Reaper backbone into a full game. Steps are ranked by impact for the cost, highest first. Each step lists why it matters, what it involves, what "done" looks like, and the risks.

What the game should be is recorded in [docs/design.md](docs/design.md) (in Chinese). Steps 1 and 4 follow it; where the two disagree, the design document wins and this file needs updating.

## Current state

The backbone is about 3,000 lines: a Rust simulation compiled to WebAssembly (`core/`), a WebGPU renderer that draws everything as paper cutouts (`src/renderer.js`, `src/shader.wgsl`), and a paper-style HTML UI over the canvas. It plays end to end: title, combat, level-up, pause, game over, saved records, keyboard and gamepad, synthesized sound.

Known weaknesses, which the steps below address:

- **Some of the content is still code.** Ids, the enemies with how they join the horde, the passive upgrades, the martial arts, the magic elements and the reactions are data in `content/game.toml` (most of step 1, done): a new enemy, a passive of an existing stat, a new art of an existing behaviour, a new element of an existing effect or a new reaction is a content edit. The spawn rate and health curves, the enemies a mission places itself, schools, and the puppet rigs are still code.
- **The elements are unplayed.** Seven elements and three reactions work and are covered by tests, but the numbers were only levelled against two bots with the balance tool (each element now adds between a twentieth and a third to how long those bots last), their effects are only shown by a colour (a tinted swing, a dot above the enemy, a ring), and holy's extra damage has only the wisp to land on, since no corpse enemy exists yet.
- **Puppets are posed in code.** `draw_player` and `draw_enemy` hardcode part offsets, tilts and sizes. A custom image can replace a part's picture but not how a character is assembled or how it moves.
- **Step 3 is done but for smarter bots.** The core steps at a fixed 60 Hz and plays the same run natively and in wasm; a run's replay link plays it back exactly (the smoke test plays one back in Node and compares the state); `cargo run --bin sim` plays runs in bulk with three bots and prints a balance table. The bots are generic: none of them knows how to win a mission, so missions cannot be balanced with the tool yet.
- **Weapon hits are brute force.** The spatial grid is only used for enemy separation; every weapon loops over every enemy. Fine at the 500-enemy cap, not beyond.
- **A run has little shape.** There is a mission framework with eleven goal missions next to endless: stay alive for five minutes, cut down 1,500 enemies, and a prototype of each other type in the design document (section 4.2.1). The nine prototypes were tuned against bots only and nobody has played them. It is still one open arena, only the boss mission's boss has attacks, and nothing carries over between runs.
- **Enemies cannot hold the player.** They never block and are slower than the player, and at most 500 exist. A bot that only runs away wins the survive mission every time at level 1, and a bot that only dodges wins the break-out mission 9 times in 12 and the retrieval mission 7 times in 8. Those missions need pressure that running cannot escape. Three candidates exist as tunes that are off by default (`chasers`, `ambush`, `arena_radius`); which of them becomes part of the game, and at what strength, is the maintainer's call after playing them.
- **Checks reach the missions, but only just.** The determinism check plays every mission natively and in wasm, with and without elements, and the smoke test starts each one in a browser for a few seconds. Both use a bot that walks at random, so neither gets far into a mission: no check wins one, loses one, or reaches its later stages.
- **Balance is untested by people.** Numbers were tuned only against a bot that walks in circles, which survived about 2 minutes 20 seconds.

## Summary

| Rank | Step | Impact | Cost | Depends on |
| --- | --- | --- | --- | --- |
| 1 | Data-driven content and puppet rigs | Very high | Medium to high | Nothing |
| 2 | Game feel pass | High | Low to medium | Nothing (easier after 1) |
| 3 | Fixed time step, replays and a headless balance tool | High | Low to medium | Nothing |
| 4 | Run structure and meta progression | High | Medium | 1, ideally 3 |
| 5 | CI and one-command deploy | Medium | Low | A GitHub remote |
| 6 | Touch controls and a WebGL2 fallback | Medium to high for reach | Medium to high | Nothing |
| 7 | Scale to thousands of enemies | Low to medium for now | Medium | 1 |

The ranks above are by impact for the cost. The working order, agreed with the maintainer on 2026-11-16, puts playable mission prototypes early:

1. The rest of step 5: core tests, the wasm drift check, format and lint checks in CI.
2. Step 3: fixed time step, replays and the balance tool, so later rewrites can be checked against recorded runs.
3. The mission framework from step 4: a mission has an objective, a win condition and a result screen, and can be picked from the title screen. "Survive a time" is the first mission.
4. Mission prototypes from the list in the design document, section 4.2.1. All of them are built (2026-11-16); what is left is playing them and settling their numbers.
5. Step 1: the data-driven rewrite, then martial arts × magic elements and reactions.
6. Step 2 (game feel) and the rest of step 4.

Leave 6 and 7 until the target devices and horde size are decided.

## 1. Data-driven content and puppet rigs

**Why.** Every later content step (new weapons, enemies, characters, bosses) is cheaper once content is data instead of code. It also removes the three-way id mirroring, which is the most likely source of silent bugs, and it lets art replace how a character is built and animated, not only what one part looks like.

**What it involves.**

- One content file, for example `content/game.toml` or `content/game.json`, that defines:
  - Parts: name, id, whether it is a flat mark or a cutout.
  - Enemies: health, speed, radius, damage, experience, rig, colour, spawn rules (start time, weight, cap).
  - Martial arts (the design's 武學 axis): each is one of a small set of behaviours with parameters: `arc` (scythe), `projectile` (talisman), `orbit` (beads), `nova` (pulse). Each has a per-level table for damage, cooldown, count, reach, pierce and knockback, and belongs to a school (流派). Players and enemies use the same definitions with different numbers.
  - Magic elements (the 魔法 axis): seven (metal, wood, water, fire, earth, lightning, holy), each with one effect it applies on hit. Metal and wood act on the player (a damage-absorbing shield, a slow heal) and leave no tag; the other five act on the enemy and leave a tag. A martial art carries at most one element, and works with none. The effects are listed in the design document, section 4.5.
  - Reactions: three element pairs in the first version (fire and lightning, water and lightning, water and fire). When an enemy holds both tags, the reaction fires and consumes them, so effects combine instead of stacking without limit.
  - At most six martial arts are held at once.
  - Passive upgrades as stat modifiers (`move_speed +8%`, `damage +12%`, `cooldown -8%`, `regen +0.4/s`, `pickup_range +30%`, `max_hp +20`).
  - Puppet rigs: a list of parts with pivot, offset, size, draw order and which pose channels move them; and named poses (`idle`, `walk_a`, `walk_b`, `attack`, `hurt`) as small per-part deltas of offset, tilt and squash.
- A generator script (Node, no dependencies) run by `build.sh` that writes `core/src/generated.rs`, `src/generated.js` and the part id constants at the top of `src/shader.wgsl`. Generated files are committed so the game still runs without the build step.
- Core changes: replace the per-weapon timer fields with a `Vec<WeaponState>` (behaviour, level, timer, element); add element tags and the reaction check to enemies; add element cards to the level-up choices, with a second step to pick which martial art receives the card (a new card replaces the old element); replace `weapons()` with one function per behaviour that reads its parameters from the generated table; replace `draw_player` and `draw_enemy` with one rig-posing function that blends between poses on a stepped clock.
- Let `assets/parts.json` map a part to several images (for example `blot` and `blot_hurt`) so a pose can swap a part's picture, still without frame-by-frame animation.
- Update `assets/parts/README.md` and the README's "Extending" section.

**Done when.** Adding a new enemy, a new martial art of an existing behaviour, a new element or a new reaction is a content-file edit plus `./build.sh`, with no Rust or JavaScript changes. With no element attached, the existing four weapons, four enemies and six passives play the same as today (checked with step 3's tool if it exists, or by side-by-side play). Each of the seven elements can be attached to each martial art, and the three reactions work. A test fails the build if any id is duplicated or referenced but undefined.

**Done so far (2026-11-17).** The martial arts are `[[art]]` entries read by one function per behaviour in `core/src/arts.rs`, with the art state in one array instead of a timer field each; a recorded run without element cards plays exactly as before the rewrite. The seven elements and three reactions are `[[element]]` and `[[reaction]]` entries, with one piece of code per effect and per reaction kind in `core/src/elements.rs`. Element cards appear among the level-up cards, with the second step that picks the art; at most `max_arts` (6) arts are held. Still to do from the list above: enemies and passives as data, schools, puppet rigs and poses, several images per part, and the README's assets section. The slot limit cannot be reached while only four arts exist.

**Built (2026-11-17, branch `feat/martial-routes`, not yet played by the user).** Routes and inner arts behind the `routes` tune (on by default): every one of the four arts has two routes (eight, from six schools), each move one of four shapes (`arc`, `wave`, `marks`, `ring`), taken at level 3 through a route card; the move order, the basic move dropping out, the ultimate last; move names brushed over the game (`EV_MOVE`), dots for the move order on the HUD chips, hit-stop and a drum on ultimates. The six passives are inner arts of three layers with rules from the second up, and three new ones (borrowed force, piercing qi, all methods). Then on `feat/level-decks`: the deck-first level-up (five decks, fine inner-art cards with two layers, the magic deck replacing the chance of an element card), and weapon kinds (twelve, with form tags that scale reach, beat, damage and knockback, drawn from the weapons deck as a new art or a change of kind, and routes that suit kinds hitting 15% harder). Then on `feat/weapon-quality`: weapon quality with a pity, and stealing (four stolen routes, the disciples of three schools, routes kept in the save). Then on `feat/school-elites`: three school elites whose moves are warned of on the ground and called out in red. Then on `feat/masters-manuals`: school masters that drop manuals, and stolen routes opened a move per manual. On `feat/martial-record`: the martial record on the title screen. On `feat/zenith`: the height of every route (極致), seven element heights, and the hero's memories. Still to do: masters that use whole routes, choosing where a manual goes between runs, the special traits of some kinds (the sickle's hook, needles in pairs), mechanisms, the meta progression of design 4.12, the player holding the weapon.

**Proposed next (2026-11-17, not yet agreed).** The user found levelling up messy and flat ("升級很亂，玩起來沒勁") and asked for weapons and martial arts to be linked and to feel more wuxia. `docs/design.md` 4.8 to 4.10 and `docs/martial-arts.md` propose weapons (刀, 棍, 暗器, 拳掌 first, built on `arc`, `orbit`, `projectile` and `nova`; 劍 and 槍 later with a new `thrust`), each with routes (套路) from schools, nine inner arts (心法) replacing the six passives, a deck-first level-up with rarity, and named moves on screen. In order, once the user agrees:

1. Routes on the sabre only, as a prototype the user can play: a weapon carries its type, kind and form tags (`[[weapon_kind]]` entries); a route carries its type, school, the kinds it suits or is limited to, and a list of moves of any length, each a behaviour with its own numbers, under one beat; an art holds a move order (`step` index) and its route, and its top level is 2 + moves + 1; the level-3 card opens the route choice, like the element step. The recorded run stays unchanged while the prototype is behind a tune that is off by default.
2. The other three weapons and their two home routes each, then the stolen routes behind the school enemies they need (bandit, constable, evil sect needle thrower and corpse puppet).
3. The nine inner arts as `[[passive]]` entries with three tiers, the rule changes in code; the ultimate condition reads the route's named inner art.
4. The deck-first level-up and weapon quality (design 4.9), then the presentation of design 4.10 (move names in brush strokes, the move-order dots on the HUD chips, the pitch per beat).

**Risks.** Over-designing the behaviour system. Keep it to the four existing behaviours plus whatever step 4 needs; add more only when a real weapon needs it. Rig blending adds per-instance work in the core; measure at the 500-enemy cap. This step is now large: the data-driven rewrite and the element axis can be separate PRs, rewrite first. Effects for 28 art-and-element pairs (four behaviours, seven elements) are a lot of art; start with a colour tint per element.

## 2. Game feel pass

**Why.** The biggest visible improvement per hour. Right now hits, kills and pickups have sound but little visual weight, enemies pop in at the screen edge, and there is no music.

**What it involves.**

- Damage numbers as small paper tags that pop up, drift and fade. Draw them in the instance stream with a digit part (an atlas of brush-style digits, or SDF digits in the shader) so they stay in the paper style and cost no DOM work.
- Hit-stop: freeze the simulation for 30 to 60 ms on big hits and kills of brutes and elites.
- Enemy death: the cutout tears into two or three paper pieces that fall with spin, on top of the existing ink splat.
- Spawn warning: an ink dot that grows for half a second where an enemy is about to enter, so spawns at the edge do not feel unfair.
- Pickup feel: gems arc towards the player instead of moving in a straight line, with a small squash when collected; the experience bar flashes on level-up.
- Low-health state: the vignette tints vermilion and the heartbeat sound plays below 25% health.
- Music: a generative pentatonic loop with WebAudio (guqin-like plucks and a low drone), with its own volume setting, ducked during level-up cards.
- A screen-shake setting (off, low, normal) for players who dislike it.

**Done so far (2026-11-17).** Damage numbers (brush digits drawn by the shader, coloured by element and reaction), hit-stop on heavy kills and reactions, and settings that switch off the numbers, and shake with hit-stop together. Then death tearing, the low-health state (a reddening edge and a heartbeat), a flash of the experience bar on level-up, and generative music with its own switch. Then pulled gems drawn on a curve (drawing only), gamepad rumble on blows, and a soft music level. The wasm core takes about 0.08 ms a frame at 489 enemies (measured in Node); the drawing side at the cap is unmeasured, since headless Chromium is capped at 60 frames a second. Not yet: spawn warnings, the squash of a collected gem.

**Done when.** Each item above is in, a "reduce motion" preference turns off shake and hit-stop, and frame time at the enemy cap is no worse than today (about 8.3 ms median measured in the browser pane).

**Risks.** Hit-stop and shake can make the game feel sluggish if overused; keep values small and tunable in one place.

## 3. Fixed time step, replays and a headless balance tool

**Why.** Balance is the main thing that makes a survivor game good, and today it has only been checked by a bot walking in circles. A deterministic core lets thousands of runs be simulated natively in seconds and turns tuning into reading numbers. It also makes bug reports reproducible.

**What it involves.**

- Change the core to step at a fixed 60 Hz with an accumulator in `jr_update`; render the last state (interpolation is optional, the paper style already steps visually).
- Make the run depend only on the seed and the input stream. Use the core's own RNG for everything (it already does) and avoid any float operation whose result differs between native and wasm builds (`powf`, `sin`, `cos` and `exp` are the ones to check; replace with fixed approximations if native and wasm results diverge).
- Record inputs as a compact list of (tick, move x, move y, choice) and export them with the seed as a replay string. Add "copy replay" to the game-over screen and a `?replay=` URL parameter to play one back.
- A native binary in `core/` (`cargo run --release --bin sim`) that runs N seeds with scripted bots (circle, kite away from the nearest enemy, stand still) and upgrade-pick strategies (random, weapon-first, a fixed build), and prints survival time, level reached, kills and damage taken per minute as a table or CSV.
- A determinism test: the same replay run natively and in wasm (via Node) ends in the same state hash.

**Done when.** The same seed and inputs always produce the same run on native and wasm, replays play back exactly, and the sim tool prints a balance table for at least three bots and three pick strategies in under a minute.

**Risks.** Float differences between native and wasm. Catch them early with the hash test rather than by debugging replays.

## 4. Run structure and meta progression

**Why.** Today a run is an endless arena with no goal and nothing to come back for. A win condition, bosses and unlocks are what make people play a second and tenth run.

**What it involves.**

The first release is one story chapter plus endless mode (see the design document, section 4).

- Stage types, each built first as a stage with a single win condition so it can be tested alone: survive a time, kill a count, boss fight; escort, hold a point and escape later. A stage alone runs 5 to 10 minutes; stages chained into a chapter run 1 to 3 minutes each. Enemy mix and spawn curve per stage come from the step 1 content file.
- Story chapter: a chain of stages. Level, martial arts and elements carry across the chain; dying restarts the chapter. The stage progress bar and what happens between stages are not decided yet.
- Endless mode: ends only on death. One commission (委託) on the map at a time, pointed to by a marker at the screen edge. Standing on its spot for a while accepts it; it has a time limit, and it vanishes and later respawns if ignored. Three kinds: defend a village, escort from A to B, kill a target at a place. Some need a return to where they were accepted. Failure only loses the reward. Only commissions from civilians in the first release.
- Enemies that attack with martial arts from step 1, so each school is recognisable. Boss behaviours stay a few reusable patterns: charge (telegraph then dash), ring of ink shots, summon a pack.
- Learning from enemies (偷師): defeating any enemy of a school unlocks its martial art for good, and adds it to the card pool at once with a notice. A tutorial outside the story teaches this by unlocking the basic art of each style; skipping it unlocks them all. A run starts with one melee art.
- Ultimate forms (極致): a martial art at maximum level, carrying an element, with a specific passive held, takes its final form; the element decides the form's effect. Aim for one or two per story run for a practised player.
- Meta progression in the existing save in `src/save.js`, with a save version bump and migration: manuals (武學秘笈) dropped by elites and bosses raise the level cap of a martial art the player chooses; a separate currency comes from commissions. What the currency buys, and all names, are still to be designed.
- More stages with different paper backgrounds (rice paper, indigo dyed paper, burnt edges) driven by the background shader's colours.

**Done so far (2026-11-17).** The mission framework and every stage type as a single mission (earlier), and the three commissions in endless, saving a village, a hunt of a marked band (the design's vendetta commission) and an escort, with the commission cycle around them (offer, accept by standing, time limit, pause, one at a time). The escort commission has its own small villager rather than the escort mission's code. The reward is a level-up and some health; the out-of-run currency and achievements the design names are not designed yet.
**Done when.** A story chapter of at least three stage types can be won, endless mode offers all three commissions, defeating a new school unlocks its art in the same run and in later ones, at least one ultimate form can be reached, and manuals are spent and saved.

**Risks.** Scope; this step is several PRs. Ship single stages and learning from enemies first, then the chapter chain, then commissions, then ultimate forms and manuals. Enemies that use martial arts need attack telegraphs to be fair, which is new work in the core and the renderer.

## 5. CI and one-command deploy

**Why.** Cheap insurance. It catches a stale `wasm/` build or a broken test before it lands, and gives a shareable link for playtesting.

**What it involves.**

- Create a GitHub repository and push (needs your go-ahead; there is no remote today).
- A GitHub Actions workflow that installs Rust with the wasm target, runs `cargo test`, runs `./build.sh`, and fails if the built wasm differs from the committed one (or switches to building the wasm only in CI and stops committing it).
- Lint and format checks: `cargo fmt --check`, `cargo clippy`, and a small JavaScript check (for example `node --check` on each module, or Biome with no config).
- Deploy `index.html`, `style.css`, `src/`, `wasm/` and `assets/` to GitHub Pages on each push to `main`.
- A smoke test with a headless browser that loads the page and checks there are no console errors. Headless Chrome in CI may not have WebGPU; if not, the test checks that the "Cannot start" screen appears cleanly instead.

**Done when.** Every push runs the checks, `main` is always deployed to a public URL, and a stale wasm build fails CI.

**Risks.** Few. WebGPU availability in CI browsers limits how much rendering can be tested automatically.

## 6. Touch controls and a WebGL2 fallback

**Why.** Opens the game to phones, tablets and browsers without WebGPU (notably older Safari and Firefox on some platforms). Survivor games suit phones well.

**What it involves.**

- Touch: a floating virtual joystick (appears where the thumb lands, drawn as a paper disc), tap to choose cards, a pause button on the HUD. Layout already adapts to narrow screens; the HUD would need a pass for notches and safe areas.
- Performance on phones: cap the device pixel ratio lower on small screens and add a "low quality" setting that turns off line boil and the per-pixel shadow sample.
- WebGL2 fallback: port `src/shader.wgsl` to GLSL ES 3.0 and add a second renderer behind the same `resize` and `draw` interface. The instance format and atlas stay the same. Detect WebGPU first and fall back automatically.

**Done so far (2026-11-17).** The floating stick, tap for cards and menus, a pause button shown after the first touch, and the HUD kept clear of notches. Checked only with synthetic touch events in a phone-sized headless browser; not on a phone. Then a pixel ratio of at most 1.5 on screens under 600 pixels on their short side, and a "Quality: low" setting that drops line boil, paper grain and the drop shadow's second shape evaluation and draws one pixel per CSS pixel. Headless Chromium runs both at its 60 frames a second cap, so the saving is unmeasured. Then the WebGL2 fallback: `src/renderer-gl.js`, the shader ported by hand to GLSL behind the same interface, picked when WebGPU fails (and by `?renderer=webgl2`); the smoke test plays a run with it. A picture of it with WebGPU switched off looked the same as the WebGPU one. Not yet: a phone, and a way to keep the two shaders from drifting apart.

**Done when.** The game is playable start to finish on a phone with touch only, and in a browser with WebGPU turned off, with the same look.

**Risks.** Two shaders to maintain. Keeping the shape functions small and generating both from one source would help but is extra work; decide when you start.

## 7. Scale to thousands of enemies

**Why.** Only matters if the design wants much bigger hordes than the current cap of 500. Worth doing once step 4's stages show what the late game needs.

**What it involves.**

- Reuse the spatial grid built in `Game::separate` for weapon hit queries and player contact, instead of looping over every enemy.
- Raise `MAX_ENEMIES` and `MAX_INST` (currently 500 and 4096), and grow the GPU instance buffer to match.
- Shader cost: the cutout shader evaluates each shape twice (shape plus shadow) and samples noise several times per pixel. Options are a cheaper path for small or distant instances, or pre-rendering each part to a texture once and drawing the texture with the margin and shadow baked in.
- Move enemy movement and separation to a compute shader only if the CPU becomes the bottleneck after the grid change; that is a large step and probably unnecessary.

**Done when.** 2,000 enemies on screen hold 60 frames per second on a mid-range laptop, measured with a fixed replay from step 3.

**Risks.** Overdraw from large overlapping cutouts dominates before the CPU does; measure before optimizing.

## Smaller items

These are too small to be steps but worth doing along the way:

- Add a version field and a migration path to the save in `src/save.js` before step 4 changes its shape.
- Pause when the gamepad disconnects mid-run.
- Show which gamepad is connected on the settings screen (`src/input.js` already tracks its name).
- Add a favicon and a social preview image once there is a deploy URL.
