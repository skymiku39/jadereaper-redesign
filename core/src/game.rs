//! Game state, simulation step and instance building.
//!
//! All simulation happens on the ground plane (x right, y down). Characters
//! are drawn standing up from their ground point, so draw code offsets parts
//! upwards. Animation is done by posing rigid paper parts (tilt, hop, flip)
//! on a low, stepped frame rate instead of playing sprite frames.

use crate::defs::*;
use crate::fmath;
use crate::mission_arena::Arena;
use crate::mission_escape::Escape;
use crate::mission_formation::Formation;
use crate::mission_hold::{Hold, POINT};
use crate::mission_retrieval::{Canvas, Retrieval};
use arts::ArtState;
use core::f32::consts::TAU;
use elements::Tags;
use mission_escort::Escort;

const MAX_ENEMIES: usize = 500;
const MAX_GEMS: usize = 700;
const MAX_FX: usize = 500;
const MAX_SHOTS: usize = 200;
const CELL: f32 = 96.0;
const BUCKETS: usize = 4096;

#[derive(Clone, Copy)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    /// Knockback velocity.
    pub kx: f32,
    pub ky: f32,
    pub hp: f32,
    pub kind: u8,
    pub flash: f32,
    pub seed: f32,
    pub bead_cd: f32,
    /// The element tags it holds.
    pub tags: Tags,
    pub id: u32,
    pub face: f32,
}

pub struct Shot {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    dmg: f32,
    pierce: i32,
    last: u32,
    /// The martial art that fired it; a hit carries that art's element.
    art: u8,
    /// The share of its speed a hit pushes the enemy at.
    push: f32,
}

pub struct Gem {
    x: f32,
    y: f32,
    value: f32,
    seed: f32,
    pull: bool,
    heart: bool,
}

pub struct Fx {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max: f32,
    part: f32,
    hw: f32,
    hh: f32,
    rot: f32,
    vr: f32,
    grow: f32,
    color: [f32; 4],
    seed: f32,
    /// Drawn above characters instead of on the ground.
    top: bool,
}

/// A damage number floating up from where a hit landed. Drawing only: it
/// takes no part in the run, its hash or its random numbers.
pub struct Num {
    x: f32,
    y: f32,
    value: f32,
    life: f32,
    /// The enemy it belongs to; more damage to it while the number is fresh adds up.
    id: u32,
    color: [f32; 3],
}

const MAX_NUMS: usize = 40;
const NUM_LIFE: f32 = 0.7;
/// The colour of plain damage: ink, which reads on the paper's white cutout edge.
const NUM_PLAIN: [f32; 3] = [0.22, 0.2, 0.18];

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub xp: f32,
    pub xp_next: f32,
    pub level: u32,
    pub aim: (f32, f32),
    pub face: f32,
    pub walk: f32,
    pub moving: bool,
    pub iframe: f32,
    pub attack: f32,
    /// Guard held (an element with the guard effect): it takes damage before health does.
    pub guard: f32,
    /// Seconds since the last hit before the guard starts to fade.
    pub guard_idle: f32,
    /// The element the guard came from; its numbers say how the guard behaves.
    pub guard_element: u8,
    /// Seconds of slow healing left (an element with the regrow effect).
    pub regrow: f32,
    pub regrow_element: u8,
}

pub struct Game {
    pub rng: Rng,
    pub running: bool,
    pub choosing: bool,
    pub over: bool,
    /// The mission being played, an `M_*` id.
    pub mission: usize,
    /// The run's tunes, indexed by `T_*`: the defaults, unless the setup screen
    /// changed some for a test run. Set before `start` and fixed after it.
    pub tune: [f32; N_TUNES],
    /// The wait before the mission, while the player has yet to accept it.
    pub accept: crate::accept::Accept,
    /// True once the mission's goal is met. The run stops there.
    pub won: bool,
    /// Run time in seconds. Stops while choosing an upgrade.
    pub time: f32,
    /// Animation clock. Keeps going on the title and upgrade screens.
    pub anim: f32,
    /// Steps taken since the game was created.
    pub ticks: u32,
    /// Real time not yet turned into a step.
    acc: f32,
    /// Half extents of the visible area in world units.
    pub view_w: f32,
    pub view_h: f32,
    pub input: (f32, f32),
    pub player: Player,
    /// State of the `escort` mission; idle in every other mission.
    pub escort: Escort,
    pub enemies: Vec<Enemy>,
    pub shots: Vec<Shot>,
    pub gems: Vec<Gem>,
    pub fx: Vec<Fx>,
    pub nums: Vec<Num>,
    /// Draw damage numbers (a setting of the host).
    pub show_numbers: bool,
    /// How much the camera shakes, 0..1 (a setting of the host).
    pub motion: f32,
    pub up: [u8; N_UPGRADES],
    /// The timer and the element of each martial art, by art id.
    pub arts: [ArtState; N_ARTS],
    /// The cards on offer: an upgrade id, `N_UPGRADES + element id - 1` for
    /// an element card, or -1 for an empty slot.
    pub choices: [i32; 3],
    /// The element whose card was picked and now waits for the player to say
    /// which martial art gets it; 0 when none does.
    pub attach: u8,
    pub pending: u32,
    /// The log of what the player does, and a log being played back.
    pub replay: replay::Replay,
    pub kills: u32,
    /// State of the retrieval mission; idle in other missions.
    pub retrieval: Retrieval,
    /// State of the `hold` mission; idle in every other mission.
    pub hold: Hold,
    /// The commission in endless, if any.
    pub commission: commission::Commission,
    /// State of the `escape` mission.
    pub escape: Escape,
    /// State of the boss mission; idle in the others.
    boss: mission_boss::Boss,
    pub shake: f32,
    pub cam: (f32, f32),
    /// The nodes of the `formation` mission. Empty in every other mission.
    pub formation: Formation,
    /// State of the arena mission.
    pub arena: Arena,
    /// State of the vendetta mission; empty in every other mission.
    pub vendetta: mission_vendetta::Vendetta,
    spawn_acc: f32,
    /// When the next enemy of each kind that comes alone is due.
    next_lone: [f32; N_KINDS],
    next_id: u32,
    /// State of the pursuit mission; idle in every other mission.
    pursuit: mission_pursuit::Pursuit,
    pub inst: Vec<Inst>,
    pub events: Vec<[f32; 4]>,
    /// What the edge-of-screen markers of this frame point at, for the host to
    /// label: `[x, y, distance from the player, 0]` in world units. Rebuilt by `build`.
    pub pointers: Vec<[f32; 4]>,
    pub state: [f32; STATE_FLOATS],
    // Scratch buffers for the spatial grid and draw sorting.
    cell_start: Vec<u32>,
    cell_cursor: Vec<u32>,
    cell_items: Vec<u16>,
    push: Vec<(f32, f32)>,
    order: Vec<(f32, u16)>,
}

/// Length of one simulation step in seconds.
pub const TICK: f32 = 1.0 / 60.0;

/// Play `ticks` steps of a run with inputs drawn from a second generator and
/// return the final state hash. Native and wasm builds must agree on it; the
/// determinism check compares them.
pub fn scripted_run(seed: u32, ticks: u32) -> u32 {
    scripted_mission(seed, ticks, M_ENDLESS, false)
}

/// The same for any mission, without its accept phase. With `armed`, the
/// player holds every martial art from the start, each with an element (which
/// ones depends on the seed), so the elements and their reactions are played
/// too. A run that is won stands still from then on.
pub fn scripted_mission(seed: u32, ticks: u32, mission: usize, armed: bool) -> u32 {
    let mut game = Game::new(seed);
    game.start(mission);
    if armed {
        for a in 0..N_ARTS {
            game.give(a, 3, (seed as usize + a) as u32 % N_ELEMENTS as u32 + 1);
        }
    }
    let mut bot = Rng((seed ^ 0x9e37_79b9) | 1);
    for t in 0..ticks {
        if t % 30 == 0 {
            game.input = (bot.range(-1.0, 1.0), bot.range(-1.0, 1.0));
        }
        game.tick();
        if game.choosing {
            game.choose((bot.next_u32() % 3) as usize);
            // Fewer than three cards were offered and the pick was empty.
            if game.choosing {
                game.choose(0);
            }
        }
        // Keep going after death so late-game spawning is covered too.
        if game.over {
            game.over = false;
            game.player.hp = game.player.max_hp;
        }
    }
    game.hash()
}

fn xp_for(level: u32) -> f32 {
    let l = level as f32;
    4.0 + l * 3.5 + l * l * 0.4
}

fn cell_key(cx: i32, cy: i32) -> usize {
    (((cy & 63) as usize) << 6) | ((cx & 63) as usize)
}

impl Game {
    pub fn new(seed: u32) -> Game {
        let mut up = [0u8; N_UPGRADES];
        for art in &ARTS {
            up[art.upgrade] = art.start_level;
        }
        Game {
            rng: Rng(seed | 1),
            running: false,
            choosing: false,
            over: false,
            mission: M_ENDLESS,
            tune: TUNE_DEFAULT,
            accept: crate::accept::Accept::default(),
            won: false,
            time: 0.0,
            anim: 0.0,
            ticks: 0,
            acc: 0.0,
            view_w: 640.0,
            view_h: 360.0,
            input: (0.0, 0.0),
            player: Player {
                x: 0.0,
                y: 0.0,
                hp: 100.0,
                max_hp: 100.0,
                xp: 0.0,
                xp_next: xp_for(1),
                level: 1,
                aim: (1.0, 0.0),
                face: 1.0,
                walk: 0.0,
                moving: false,
                iframe: 0.0,
                attack: 0.0,
                guard: 0.0,
                guard_idle: 0.0,
                guard_element: 0,
                regrow: 0.0,
                regrow_element: 0,
            },
            escort: Escort::new(),
            enemies: Vec::with_capacity(MAX_ENEMIES),
            shots: Vec::with_capacity(MAX_SHOTS),
            gems: Vec::with_capacity(MAX_GEMS),
            fx: Vec::with_capacity(MAX_FX),
            nums: Vec::with_capacity(MAX_NUMS),
            show_numbers: true,
            motion: 1.0,
            up,
            arts: ArtState::fresh(),
            choices: [-1; 3],
            attach: 0,
            pending: 0,
            replay: replay::Replay::default(),
            kills: 0,
            retrieval: Retrieval::default(),
            hold: Hold::new(),
            escape: Escape::default(),
            commission: commission::Commission::default(),
            boss: mission_boss::Boss::default(),
            shake: 0.0,
            cam: (0.0, -30.0),
            formation: Formation::default(),
            arena: Arena::new(),
            vendetta: mission_vendetta::Vendetta::default(),
            spawn_acc: 0.0,
            next_lone: core::array::from_fn(|k| KINDS[k].every),
            next_id: 1,
            pursuit: mission_pursuit::Pursuit::default(),
            inst: Vec::with_capacity(MAX_INST),
            events: Vec::with_capacity(256),
            pointers: Vec::with_capacity(16),
            state: [0.0; STATE_FLOATS],
            cell_start: vec![0; BUCKETS + 1],
            cell_cursor: vec![0; BUCKETS + 1],
            cell_items: Vec::with_capacity(MAX_ENEMIES),
            push: Vec::with_capacity(MAX_ENEMIES),
            order: Vec::with_capacity(MAX_ENEMIES + 1),
        }
    }

    /// Start a run of the given mission. An unknown id plays endless.
    pub fn start(&mut self, mission: usize) {
        self.mission = if mission < N_MISSIONS { mission } else { M_ENDLESS };
        self.hold.begin(self.mission == M_HOLD, &self.tune);
        self.running = true;
        // A test run can open with an element already on the first held art.
        let element = self.tune[T_START_ELEMENT] as usize;
        if (1..=N_ELEMENTS).contains(&element) {
            if let Some(a) = (0..N_ARTS).find(|&a| self.art_level(a) > 0) {
                self.arts[a].element = element as u8;
            }
        }
        if self.mission == M_FORMATION {
            self.formation_start();
        }
        self.arena = Arena::from_tunes(&self.tune);
        if self.mission == M_RETRIEVAL {
            self.retrieval.reset(&mut self.rng, &self.tune);
        }
        if self.mission == M_PURSUIT {
            self.pursuit_start();
        }
        self.escort_begin();
        if self.mission == M_ESCAPE {
            self.escape_begin();
        }
        if self.mission == M_VENDETTA {
            self.vendetta_start();
        }
    }

    /// Change one tune for the run about to start, kept inside its range.
    /// An unknown id or a value that is not a number changes nothing.
    pub fn set_tune(&mut self, id: usize, value: f32) {
        if id < N_TUNES && value.is_finite() {
            self.tune[id] = value.clamp(TUNE_MIN[id], TUNE_MAX[id]);
        }
    }

    /// What the mission asks for, in its own unit. 0 for none.
    pub fn goal(&self) -> f32 {
        // The escort route is its legs laid end to end, so its length follows the leg tune.
        if self.mission == M_ESCORT {
            return mission_escort::LEGS as f32 * self.tune[T_ESCORT_LEG_LEN];
        }
        match MISSION_GOAL_TUNE[self.mission] {
            t if t < N_TUNES => self.tune[t],
            _ => MISSION_GOAL[self.mission],
        }
    }

    /// How far along the mission is, in the same unit as the goal.
    pub fn progress(&self) -> f32 {
        match self.mission {
            M_SURVIVE => self.time.min(self.goal()),
            M_KILLS => (self.kills as f32).min(self.goal()),
            M_FORMATION => self.formation.destroyed() as f32,
            M_ARENA => (self.arena.cleared as f32).min(self.goal()),
            M_RETRIEVAL => self.retrieval.delivered() as f32,
            M_PURSUIT => self.pursuit_progress(),
            M_ESCORT => self.escort.walked.min(self.goal()),
            M_HOLD => self.hold.progress(self.goal()),
            M_ESCAPE => self.escape_progress(),
            M_BOSS => self.boss_progress(),
            M_VENDETTA => self.vendetta_progress(),
            _ => 0.0,
        }
    }

    /// The win rule of each mission. Endless has none.
    fn mission_won(&self) -> bool {
        match self.mission {
            M_SURVIVE => self.time >= self.goal(),
            M_KILLS => self.kills as f32 >= self.goal(),
            M_FORMATION => self.formation.cleared(),
            M_ARENA => self.arena.won(),
            M_RETRIEVAL => self.retrieval.won(),
            M_PURSUIT => self.pursuit_won(),
            M_ESCORT => self.escort.walked >= self.goal(),
            M_HOLD => self.hold.won(self.goal()),
            M_ESCAPE => self.escape.won(self.player.x, self.player.y),
            M_BOSS => self.boss_dead(),
            M_VENDETTA => self.vendetta_won(),
            _ => false,
        }
    }

    pub fn set_view(&mut self, half_w: f32, half_h: f32) {
        // While a replay plays, the visible area comes from its log.
        if self.replay.playing {
            return;
        }
        self.view_w = half_w.max(100.0);
        self.view_h = half_h.max(100.0);
    }

    /// Advance by `dt` seconds of real time and rebuild what the host reads.
    ///
    /// The simulation only ever moves in whole steps of `TICK`, so a run
    /// depends on the seed and the inputs, not on the frame rate. Time that
    /// does not fill a step is carried over to the next call.
    pub fn update(&mut self, dt: f32) {
        let mut dt = if dt.is_finite() { dt.clamp(0.0, 0.05) } else { 0.0 };
        // A 60 Hz display reports frame times that wobble around one step.
        // Treat those as exactly one, or the wobble shows as skipped and
        // doubled steps.
        if (dt - TICK).abs() < TICK * 0.12 {
            dt = TICK;
        }
        self.events.clear();
        self.acc += dt;
        while self.acc >= TICK {
            self.acc -= TICK;
            self.tick();
        }
        self.present();
    }

    /// Advance the simulation by exactly one step, without drawing.
    pub fn tick(&mut self) {
        self.ticks = self.ticks.wrapping_add(1);
        self.anim += TICK;
        self.replay_before_step();
        if self.running && !self.choosing && !self.over && !self.won {
            if self.accept.active {
                self.accept_step(TICK);
            } else {
                self.step(TICK);
            }
        }
        self.replay_after_step();
        self.shake = (self.shake - TICK * 3.0).max(0.0);
    }

    /// Rebuild the camera, the instance list and the HUD state.
    pub fn present(&mut self) {
        // Camera follows the player, with a decaying shake.
        let s = self.shake * self.shake * 9.0 * self.motion;
        let q = (self.anim * 30.0).floor();
        self.cam = (self.player.x + s * fmath::sin(q * 12.9898), self.player.y - 30.0 + s * fmath::cos(q * 78.233));
        self.build();
        self.write_state();
    }

    /// A checksum of everything that decides how the run continues. Two runs
    /// with the same seed and inputs have the same hash after the same number
    /// of steps, on every platform.
    pub fn hash(&self) -> u32 {
        let mut h = 0x811c_9dc5u32;
        let mut mix = |v: u32| h = (h ^ v).wrapping_mul(0x0100_0193);
        mix(self.ticks);
        mix(self.rng.0);
        mix(self.time.to_bits());
        mix(self.kills);
        mix(self.pending);
        if self.mission == M_PURSUIT {
            for v in self.pursuit_hash() {
                mix(v);
            }
        }
        mix(self.mission as u32);
        mix(self.won as u32);
        mix(self.arena.spawned);
        mix(self.arena.cleared);
        mix(self.arena.pause.to_bits());
        self.retrieval.mix(&mut mix);
        if self.escort.active {
            self.escort.mix_into(&mut mix);
        }
        self.hold.mix(&mut mix);
        if self.commissions_on() {
            self.commission.mix(&mut mix);
        }
        for v in self.escape.hash_words() {
            mix(v);
        }
        if self.mission == M_VENDETTA {
            for v in self.vendetta.hash_words() {
                mix(v);
            }
        }
        let p = &self.player;
        for v in [p.x, p.y, p.hp, p.max_hp, p.xp, p.xp_next] {
            mix(v.to_bits());
        }
        mix(p.level);
        for &level in &self.up {
            mix(level as u32);
        }
        for art in &self.arts {
            mix(art.element as u32);
        }
        mix(self.attach as u32);
        mix(p.guard.to_bits());
        mix(p.regrow.to_bits());
        mix(self.enemies.len() as u32);
        for e in &self.enemies {
            mix(e.x.to_bits());
            mix(e.y.to_bits());
            mix(e.hp.to_bits());
            for left in &e.tags.left[1..] {
                mix(left.to_bits());
            }
        }
        mix(self.shots.len() as u32);
        for s in &self.shots {
            mix(s.x.to_bits());
            mix(s.y.to_bits());
        }
        mix(self.gems.len() as u32);
        for g in &self.gems {
            mix(g.x.to_bits());
            mix(g.y.to_bits());
        }
        self.formation.mix_into(&mut mix);
        if self.mission == M_BOSS {
            for v in self.boss.hash_words() {
                mix(v);
            }
        }
        h
    }

    pub(crate) fn event(&mut self, kind: f32, x: f32, y: f32, v: f32) {
        if self.events.len() < 256 {
            self.events.push([kind, x, y, v]);
        }
    }

    /// What the passive upgrades make of a stat (an `ST_*` id), from their levels:
    /// a factor for speed, range, damage and cooldown, health per second for regen.
    /// The most health is raised when the upgrade is picked, not here.
    pub(crate) fn stat(&self, stat: u8) -> f32 {
        let held = PASSIVES.iter().filter(|p| p.stat == stat);
        match stat {
            ST_COOLDOWN => held.fold(1.0, |f, p| f * fmath::powu(p.amount, self.up[p.upgrade] as u32)),
            ST_REGEN => held.fold(0.0, |sum, p| sum + p.amount * self.up[p.upgrade] as f32),
            _ => 1.0 + held.fold(0.0, |sum, p| sum + p.amount * self.up[p.upgrade] as f32),
        }
    }

    // ---------------------------------------------------------------- step

    /// Move the player by the input, and count down his timers.
    pub(crate) fn walk(&mut self, dt: f32) {
        let (mut mx, mut my) = self.input;
        if !mx.is_finite() || !my.is_finite() {
            mx = 0.0;
            my = 0.0;
        }
        let mut len = (mx * mx + my * my).sqrt();
        if len > 1.0 {
            mx /= len;
            my /= len;
            len = 1.0;
        }
        let speed = 190.0 * self.stat(ST_MOVE_SPEED) * self.retrieval.speed_factor();
        let regen = self.stat(ST_REGEN);
        let p = &mut self.player;
        p.moving = len > 0.15;
        if p.moving {
            p.x += mx * speed * dt;
            p.y += my * speed * dt;
            p.aim = (mx / len, my / len);
            p.walk += dt;
            if mx.abs() > 0.1 {
                p.face = mx.signum();
            }
        }
        p.iframe = (p.iframe - dt).max(0.0);
        p.attack = (p.attack - dt).max(0.0);
        p.hp = (p.hp + regen * dt).min(p.max_hp);
    }

    fn step(&mut self, dt: f32) {
        self.time += dt;
        self.walk(dt);
        // A walled field: the player cannot leave the circle around where the mission began.
        let wall = self.tune[T_ARENA_RADIUS];
        if wall > 0.0 {
            let p = &mut self.player;
            let d = (p.x * p.x + p.y * p.y).sqrt();
            if d > wall {
                p.x *= wall / d;
                p.y *= wall / d;
            }
        }
        self.tick_boons(dt);

        let moved = self.retrieval.step(self.player.x, self.player.y);
        if moved.picked > 0 {
            self.event(EV_PICKUP, self.player.x, self.player.y, 0.0);
        }
        if moved.delivered > 0 {
            self.event(EV_HEAL, self.player.x, self.player.y, 0.0);
        }

        self.weapons(dt);
        self.spawn(dt * self.boss_spawn_scale());
        self.formation_step(dt);
        self.pursuit_step(dt);
        if self.mission == M_ESCAPE {
            self.escape_spawn(dt);
        }
        self.boss_think(dt);
        self.vendetta.step(self.player.x, self.player.y);
        self.move_enemies(dt);
        self.escort_step(dt);
        self.commission_step(dt);
        // The shrine falling on the step the player dies counts as dying.
        if self.hold.step(self.player.x, self.player.y, dt) && !self.over {
            self.over = true;
            self.hold.fell = true;
            self.event(EV_DEATH, POINT.0, POINT.1, 0.0);
        }
        self.separate();
        self.move_shots(dt);
        self.tick_tags(dt);
        self.reap();
        self.move_gems(dt);
        self.move_fx(dt);
        self.move_nums(dt);

        // Level ups.
        while self.player.xp >= self.player.xp_next {
            self.player.xp -= self.player.xp_next;
            self.player.level += 1;
            self.player.xp_next = xp_for(self.player.level);
            self.pending += 1;
        }
        // Dying on the same step as meeting the goal counts as dying.
        if !self.over && self.mission_won() {
            self.won = true;
            self.event(EV_WIN, self.player.x, self.player.y, 0.0);
        }
        if self.pending > 0 && !self.choosing && !self.over && !self.won {
            self.offer();
        }
    }

    fn hurt_enemy(&mut self, i: usize, dmg: f32, kx: f32, ky: f32) {
        self.hurt_enemy_as(i, dmg, kx, ky, NUM_PLAIN);
    }

    /// Hurt an enemy, with its damage number in `color`.
    pub(crate) fn hurt_enemy_as(&mut self, i: usize, dmg: f32, kx: f32, ky: f32, color: [f32; 3]) {
        let (id, x, y, r) = {
            let e = &self.enemies[i];
            (e.id, e.x, e.y, KINDS[e.kind as usize].hh)
        };
        self.add_num(id, x, y - r * 2.0 - 6.0, dmg, color);
        let e = &mut self.enemies[i];
        e.hp -= dmg;
        e.flash = 1.0;
        let heavy = if KINDS[e.kind as usize].heavy { 0.35 } else { 1.0 };
        e.kx += kx * heavy;
        e.ky += ky * heavy;
        let (x, y) = (e.x, e.y);
        self.event(EV_HIT, x, y, dmg);
    }

    pub(crate) fn spawn_ring(&self) -> f32 {
        (self.view_w * self.view_w + self.view_h * self.view_h).sqrt() + 70.0
    }

    pub(crate) fn spawn_enemy(&mut self, kind: u8) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let mut a = self.rng.range(0.0, TAU);
        // An ambush: now and then an enemy comes from where the player is heading,
        // so running in a straight line runs into something.
        let ambush = self.tune[T_AMBUSH];
        if ambush > 0.0 && self.player.moving && self.rng.f() < ambush {
            let (ax, ay) = self.player.aim;
            a = fmath::atan2(ay, ax) + self.rng.range(-0.6, 0.6);
        }
        let ring = self.spawn_ring();
        let (x, y) = (self.player.x + fmath::cos(a) * ring, self.player.y + fmath::sin(a) * ring);
        self.spawn_enemy_at(kind, x, y);
    }

    /// Add an enemy of `kind` at a world position, with the health it has at this time.
    pub(crate) fn spawn_enemy_at(&mut self, kind: u8, x: f32, y: f32) {
        if self.enemies.len() >= MAX_ENEMIES {
            return;
        }
        let k = &KINDS[kind as usize];
        let scale = if self.mission == M_ARENA { self.arena.hp_scale() } else { 1.0 + self.time / 120.0 };
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let seed = self.rng.f();
        self.enemies.push(Enemy {
            x,
            y,
            kx: 0.0,
            ky: 0.0,
            hp: k.hp * scale * self.tune[T_ENEMY_HP],
            kind,
            flash: 0.0,
            seed,
            bead_cd: 0.0,
            tags: Tags::default(),
            id,
            face: 1.0,
        });
    }

    /// The share of the horde's spawns that are of kind `k`, `t` seconds into the run.
    fn horde_share(&self, k: usize, t: f32) -> f32 {
        let def = &KINDS[k];
        if def.every > 0.0 {
            0.0
        } else if def.share_tune < N_TUNES {
            self.tune[def.share_tune]
        } else if t > def.from {
            if def.ramp > 0.0 {
                ((t - def.from) / def.ramp).min(def.share)
            } else {
                def.share
            }
        } else {
            0.0
        }
    }

    fn spawn(&mut self, dt: f32) {
        if self.mission == M_ARENA {
            self.arena_step(dt);
            return;
        }
        let t = self.time;
        let rate = (1.0 + t / 24.0) * self.formation_edge_scale() * self.tune[T_SPAWN_RATE];
        self.spawn_acc += rate * dt;
        while self.spawn_acc >= 1.0 {
            self.spawn_acc -= 1.0;
            let roll = self.rng.f();
            // Each kind takes its share of the rolls, the last kind first; kind 0 gets the rest.
            let mut kind = 0;
            let mut taken = 0.0;
            for k in (1..N_KINDS).rev() {
                taken += self.horde_share(k, t);
                if roll < taken {
                    kind = k as u8;
                    break;
                }
            }
            self.spawn_enemy(kind);
        }
        // The kinds that come alone, on a clock.
        for k in 0..N_KINDS {
            if KINDS[k].every > 0.0 && t >= self.next_lone[k] {
                self.next_lone[k] += KINDS[k].every;
                self.spawn_enemy(k as u8);
                self.event(EV_ELITE, self.player.x, self.player.y, 0.0);
            }
        }
    }

    fn move_enemies(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let ring = self.spawn_ring();
        let far = ring * 1.8;
        let decay = fmath::exp(-8.0 * dt);
        let pace = self.tune[T_ENEMY_SPEED];
        let escort = &self.escort;
        let mut hit: Option<f32> = None;
        let hold = &mut self.hold;
        let siege = &mut self.commission;
        for e in self.enemies.iter_mut() {
            // The pursuit target moves itself and never leaves its route.
            if e.id == mission_pursuit::PURSUIT_ID {
                continue;
            }
            let k = &KINDS[e.kind as usize];
            let speed = k.speed * pace;
            let (dx, dy) = (px - e.x, py - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            // The boss of the boss mission steers itself.
            let drive = self.boss.drive(e.id);
            // A marked enemy of the vendetta mission follows the mission, not the horde.
            let marked = self.vendetta.steer(e.id, e.x, e.y, px, py, speed);
            // An enemy sent after the village keeps going there, however far the player is.
            let sieging = siege.owns(e.id);
            if d > far && drive.is_none() && marked.is_none() && !sieging {
                // Left far behind: bring it back to the spawn ring on the
                // other side so the crowd keeps closing in.
                e.x = px + dx / d * ring;
                e.y = py + dy / d * ring;
                continue;
            }
            // An enemy chases the player, or the escorted character when
            // that one is nearer.
            let (tx, ty) = escort.target(e.x, e.y, px, py);
            let (cx, cy) = (tx - e.x, ty - e.y);
            let cd = (cx * cx + cy * cy).sqrt().max(0.001);
            // In the hold mission, the enemies sent after the shrine walk to
            // it instead and face it.
            let (hx, hy, hd, face) = match hold.target(e.id) {
                Some((qx, qy)) => {
                    let (ax, ay) = (qx - e.x, qy - e.y);
                    let hd = (ax * ax + ay * ay).sqrt();
                    hold.gnaw(hd, k.r, k.dmg * dt);
                    (ax, ay, hd.max(0.001), ax)
                }
                // Likewise for the village of a commission in endless.
                None => match siege.target(e.id) {
                    Some((qx, qy)) => {
                        let (ax, ay) = (qx - e.x, qy - e.y);
                        let hd = (ax * ax + ay * ay).sqrt();
                        siege.gnaw(hd, k.r, k.dmg * dt);
                        (ax, ay, hd.max(0.001), ax)
                    }
                    None => (cx, cy, cd, dx),
                },
            };
            // The boss and the marked enemies bring their own velocity.
            let (vx, vy) = match (drive, marked) {
                (Some(v), _) => {
                    e.kx = 0.0;
                    e.ky = 0.0;
                    v
                }
                (None, Some(v)) => v,
                // Rooted by an element: it stands where the blow threw it.
                (None, None) if e.tags.rooted() => (0.0, 0.0),
                (None, None) => (hx / hd * speed, hy / hd * speed),
            };
            e.x += (vx + e.kx) * dt;
            e.y += (vy + e.ky) * dt;
            e.kx *= decay;
            e.ky *= decay;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            e.bead_cd = (e.bead_cd - dt).max(0.0);
            if face.abs() > 4.0 {
                e.face = face.signum();
            }
            if d < k.r + 10.0 {
                hit = Some(hit.map_or(k.dmg, |h: f32| h.max(k.dmg)));
            }
        }
        if let Some(dmg) = hit {
            if self.player.iframe <= 0.0 {
                self.player.hp -= self.guard_against(dmg);
                self.player.iframe = 0.8;
                self.shake = 1.0;
                self.event(EV_HURT, px, py, dmg);
                for _ in 0..6 {
                    let a = self.rng.range(0.0, TAU);
                    let s = self.rng.range(80.0, 220.0);
                    let seed = self.rng.f();
                    self.add_fx(Fx {
                        x: px,
                        y: py - 30.0,
                        vx: fmath::cos(a) * s,
                        vy: fmath::sin(a) * s - 60.0,
                        life: 0.5,
                        max: 0.5,
                        part: P_SCRAP,
                        hw: 4.0,
                        hh: 4.0,
                        rot: a,
                        vr: 9.0,
                        grow: 0.0,
                        color: [0.20, 0.42, 0.36, 1.0],
                        seed,
                        top: true,
                    });
                }
                if self.player.hp <= 0.0 {
                    self.player.hp = 0.0;
                    self.over = true;
                    self.event(EV_DEATH, px, py, 0.0);
                }
            }
        }
    }

    /// Push overlapping enemies apart using a hashed uniform grid.
    fn separate(&mut self) {
        let n = self.enemies.len();
        if n < 2 {
            return;
        }
        for c in self.cell_start.iter_mut() {
            *c = 0;
        }
        for e in &self.enemies {
            let k = cell_key((e.x / CELL).floor() as i32, (e.y / CELL).floor() as i32);
            self.cell_start[k + 1] += 1;
        }
        for i in 0..BUCKETS {
            self.cell_start[i + 1] += self.cell_start[i];
        }
        self.cell_cursor.copy_from_slice(&self.cell_start);
        self.cell_items.clear();
        self.cell_items.resize(n, 0);
        for (i, e) in self.enemies.iter().enumerate() {
            let k = cell_key((e.x / CELL).floor() as i32, (e.y / CELL).floor() as i32);
            self.cell_items[self.cell_cursor[k] as usize] = i as u16;
            self.cell_cursor[k] += 1;
        }
        self.push.clear();
        self.push.resize(n, (0.0, 0.0));
        for i in 0..n {
            let e = self.enemies[i];
            let er = KINDS[e.kind as usize].r;
            let cx = (e.x / CELL).floor() as i32;
            let cy = (e.y / CELL).floor() as i32;
            for oy in -1..=1 {
                for ox in -1..=1 {
                    let k = cell_key(cx + ox, cy + oy);
                    for s in self.cell_start[k]..self.cell_start[k + 1] {
                        let j = self.cell_items[s as usize] as usize;
                        if j <= i {
                            continue;
                        }
                        let o = &self.enemies[j];
                        let rr = er + KINDS[o.kind as usize].r;
                        let (dx, dy) = (o.x - e.x, o.y - e.y);
                        let d2 = dx * dx + dy * dy;
                        if d2 < rr * rr && d2 > 0.0001 {
                            let d = d2.sqrt();
                            let f = (rr - d) * 0.5 / d;
                            self.push[i].0 -= dx * f;
                            self.push[i].1 -= dy * f;
                            self.push[j].0 += dx * f;
                            self.push[j].1 += dy * f;
                        }
                    }
                }
            }
        }
        for (e, p) in self.enemies.iter_mut().zip(self.push.iter()) {
            // The boss is not shoved by the crowd.
            if self.boss.is(e.id) {
                continue;
            }
            e.x += p.0 * 0.6;
            e.y += p.1 * 0.6;
        }
    }

    fn move_shots(&mut self, dt: f32) {
        let mut s = 0;
        while s < self.shots.len() {
            let (x, y, dmg, last) = {
                let sh = &mut self.shots[s];
                sh.x += sh.vx * dt;
                sh.y += sh.vy * dt;
                sh.life -= dt;
                (sh.x, sh.y, sh.dmg, sh.last)
            };
            let mut hit = None;
            for (i, e) in self.enemies.iter().enumerate() {
                if e.id == last || e.hp <= 0.0 {
                    continue;
                }
                let r = KINDS[e.kind as usize].r + 8.0;
                let (dx, dy) = (e.x - x, e.y - y);
                if dx * dx + dy * dy < r * r {
                    hit = Some(i);
                    break;
                }
            }
            if let Some(i) = hit {
                let (vx, vy, push) = (self.shots[s].vx, self.shots[s].vy, self.shots[s].push);
                let element = self.arts[self.shots[s].art as usize].element;
                self.shots[s].last = self.enemies[i].id;
                self.shots[s].pierce -= 1;
                self.strike(i, dmg, vx * push, vy * push, element);
            } else if self.formation_hit_shot(x, y, dmg) {
                self.shots[s].pierce = 0;
            }
            if self.shots[s].life <= 0.0 || self.shots[s].pierce <= 0 {
                self.shots.swap_remove(s);
            } else {
                s += 1;
            }
        }
    }

    /// Remove dead enemies and drop their rewards.
    fn reap(&mut self) {
        let mut i = 0;
        while i < self.enemies.len() {
            if self.enemies[i].hp > 0.0 {
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            let k = &KINDS[e.kind as usize];
            self.kills += 1;
            self.event(EV_KILL, e.x, e.y, e.kind as f32);
            let heart = self.rng.f() < if e.kind == K_ELITE { 1.0 } else { 0.012 };
            self.drop_gem(e.x, e.y, k.xp, false);
            if heart {
                self.drop_gem(e.x + 14.0, e.y, 0.0, true);
            }
            let n = if k.heavy { 9 } else { 4 };
            for _ in 0..n {
                let a = self.rng.range(0.0, TAU);
                let s = self.rng.range(20.0, 150.0);
                let size = self.rng.range(0.25, 0.6) * k.r;
                let seed = self.rng.f();
                self.add_fx(Fx {
                    x: e.x,
                    y: e.y - k.r * 0.5,
                    vx: fmath::cos(a) * s,
                    vy: fmath::sin(a) * s * 0.6,
                    life: 0.55,
                    max: 0.55,
                    part: P_SPLAT,
                    hw: size,
                    hh: size * 0.8,
                    rot: a,
                    vr: 0.0,
                    grow: 8.0,
                    color: [INK[0], INK[1], INK[2], 0.7],
                    seed,
                    top: false,
                });
            }
            self.tear(&e);
        }
    }

    /// The paper of a fallen enemy tears into a few pieces that spin away.
    /// Drawing only: the pieces come from the enemy's id, not the run's random
    /// numbers, so they change nothing about how the run goes.
    fn tear(&mut self, e: &Enemy) {
        let k = &KINDS[e.kind as usize];
        let pieces = if k.heavy { 3 } else { 2 };
        for p in 0..pieces {
            let h = hash2(e.id as i32, p);
            let r = |shift: u32| ((h >> shift) & 255) as f32 / 255.0;
            // Each piece flies off to its own side, up and out.
            let side = if p % 2 == 0 { -1.0 } else { 1.0 };
            let (vx, vy) = (side * (60.0 + 90.0 * r(0)), -90.0 - 80.0 * r(8));
            self.add_fx(Fx {
                x: e.x + side * k.hw * 0.3,
                y: e.y - k.hh,
                vx,
                vy,
                life: 0.45,
                max: 0.45,
                part: P_SCRAP,
                hw: k.hw * (0.35 + 0.15 * r(16)),
                hh: k.hh * (0.3 + 0.15 * r(24)),
                rot: r(4) * TAU,
                vr: side * (6.0 + 6.0 * r(12)),
                grow: -6.0,
                color: [k.color[0], k.color[1], k.color[2], 0.95],
                seed: r(20),
                top: true,
            });
        }
    }

    fn drop_gem(&mut self, x: f32, y: f32, value: f32, heart: bool) {
        if !heart && self.gems.len() >= MAX_GEMS {
            // Too many on the ground: fold the value into an existing gem.
            let i = (self.rng.next_u32() as usize) % self.gems.len();
            self.gems[i].value += value;
            return;
        }
        let seed = self.rng.f();
        let jx = self.rng.range(-6.0, 6.0);
        let jy = self.rng.range(-6.0, 6.0);
        self.gems.push(Gem { x: x + jx, y: y + jy, value, seed, pull: false, heart });
    }

    fn move_gems(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let lure = 95.0 * self.stat(ST_PICKUP_RANGE);
        let mut i = 0;
        while i < self.gems.len() {
            let g = &mut self.gems[i];
            let (dx, dy) = (px - g.x, py - g.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < lure {
                g.pull = true;
            }
            if g.pull {
                let v = 460.0 * dt;
                g.x += dx / d * v;
                g.y += dy / d * v;
            }
            if d < 20.0 {
                let g = self.gems.swap_remove(i);
                if g.heart {
                    self.player.hp = (self.player.hp + 25.0).min(self.player.max_hp);
                    self.event(EV_HEAL, px, py, 25.0);
                } else {
                    self.player.xp += g.value;
                    self.event(EV_PICKUP, px, py, g.value);
                }
            } else {
                i += 1;
            }
        }
    }

    /// Show damage over an enemy: added to its fresh number of the same colour, or a new one.
    fn add_num(&mut self, id: u32, x: f32, y: f32, value: f32, color: [f32; 3]) {
        // Also leaves out a value that is not a number.
        if !self.show_numbers || value.partial_cmp(&0.0) != Some(core::cmp::Ordering::Greater) {
            return;
        }
        if let Some(n) = self.nums.iter_mut().find(|n| n.id == id && n.color == color && n.life > NUM_LIFE * 0.25) {
            n.value += value;
            n.life = NUM_LIFE;
            return;
        }
        if self.nums.len() >= MAX_NUMS {
            return;
        }
        // A little sideways spread, from the enemy's id rather than the run's random numbers.
        let jitter = ((hash2(id as i32, self.nums.len() as i32) & 255) as f32 / 255.0 - 0.5) * 18.0;
        self.nums.push(Num { x: x + jitter, y, value, life: NUM_LIFE, id, color });
    }

    fn move_nums(&mut self, dt: f32) {
        self.nums.retain_mut(|n| {
            n.life -= dt;
            n.y -= 38.0 * dt;
            n.life > 0.0
        });
    }

    /// Damage numbers: one brush digit per figure, popping in and fading out.
    fn draw_nums(&mut self) {
        for i in 0..self.nums.len() {
            let n = &self.nums[i];
            let (x, y, life, color) = (n.x, n.y, n.life, n.color);
            let value = n.value.round().clamp(1.0, 99_999.0) as u32;
            let t = life / NUM_LIFE;
            let alpha = (t / 0.35).min(1.0);
            let digits = if value >= 10_000 {
                5
            } else if value >= 1000 {
                4
            } else if value >= 100 {
                3
            } else if value >= 10 {
                2
            } else {
                1
            };
            // Big for a moment when it appears, and bigger for larger hits.
            let size = (1.0 + 0.35 * ((t - 0.85) / 0.15).max(0.0)) * (1.0 + 0.12 * (digits as f32 - 1.0).min(3.0));
            let (hw, hh) = (6.0 * size, 9.5 * size);
            let step = hw * 2.15;
            let mut v = value;
            for k in 0..digits {
                let d = v % 10;
                v /= 10;
                let dx = (digits as f32 - 1.0) * 0.5 * step - k as f32 * step;
                let c = [color[0], color[1], color[2], alpha];
                self.put(x + dx, y, hw, hh, 0.0, P_DIGIT, 0.0, d as f32 * 0.1, c);
            }
        }
    }

    fn add_fx(&mut self, f: Fx) {
        if self.fx.len() < MAX_FX {
            self.fx.push(f);
        }
    }

    fn move_fx(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.fx.len() {
            let f = &mut self.fx[i];
            f.life -= dt;
            if f.life <= 0.0 {
                self.fx.swap_remove(i);
                continue;
            }
            f.x += f.vx * dt;
            f.y += f.vy * dt;
            let drag = fmath::exp(-5.0 * dt);
            f.vx *= drag;
            f.vy *= drag;
            f.rot += f.vr * dt;
            f.hw += f.grow * dt;
            f.hh += f.grow * dt;
            i += 1;
        }
    }

    // ------------------------------------------------------------ upgrades

    fn offer(&mut self) {
        let mut pool = [0usize; N_UPGRADES];
        let mut n = 0;
        // With every slot for a martial art taken, the ones not held are not offered.
        let full = self.arts_held() >= MAX_ARTS;
        for id in 0..N_UPGRADES {
            let new_art = self.up[id] == 0 && ARTS.iter().any(|a| a.upgrade == id);
            if self.up[id] < UPGRADE_MAX[id] && !(full && new_art) {
                pool[n] = id;
                n += 1;
            }
        }
        if n == 0 {
            // Everything maxed: heal instead.
            self.player.hp = (self.player.hp + 30.0 * self.pending as f32).min(self.player.max_hp);
            self.pending = 0;
            return;
        }
        for c in 0..3 {
            if c < n {
                let j = c + (self.rng.next_u32() as usize) % (n - c);
                pool.swap(c, j);
                self.choices[c] = pool[c] as i32;
            } else {
                self.choices[c] = -1;
            }
        }
        // Now and then the last card is an element instead: it takes the
        // third slot, or the first empty one when fewer upgrades are left.
        let chance = self.tune[T_ELEMENT_CARDS];
        if chance > 0.0 && self.rng.f() < chance {
            let element = 1 + (self.rng.next_u32() as usize) % N_ELEMENTS;
            self.choices[n.min(2)] = (N_UPGRADES + element - 1) as i32;
        }
        self.choosing = true;
        self.event(EV_LEVELUP, self.player.x, self.player.y, self.player.level as f32);
    }

    /// Pick one of the cards on offer (0..2). After an element card, the
    /// choice stays open and the next pick is the id of the held martial art
    /// that gets the element; whatever element it carried is lost.
    pub fn choose(&mut self, i: usize) {
        if !self.choosing {
            return;
        }
        self.replay_pick(i);
        if self.attach > 0 {
            if i >= N_ARTS || self.art_level(i) == 0 {
                return;
            }
            self.arts[i].element = self.attach;
            self.event(EV_ATTACH, self.player.x, self.player.y, self.attach as f32);
            self.attach = 0;
        } else {
            if i >= 3 || self.choices[i] < 0 {
                return;
            }
            let id = self.choices[i] as usize;
            if id >= N_UPGRADES {
                self.attach = (id - N_UPGRADES + 1).min(N_ELEMENTS) as u8;
                self.choices = [-1; 3];
                return;
            }
            if self.up[id] < UPGRADE_MAX[id] {
                self.up[id] += 1;
                // A passive that raises the most health does it once, when picked.
                for p in PASSIVES.iter().filter(|p| p.upgrade == id && p.stat == ST_MAX_HP) {
                    self.player.max_hp += p.amount;
                    self.player.hp += p.amount;
                }
            }
        }
        self.choices = [-1; 3];
        self.choosing = false;
        self.pending = self.pending.saturating_sub(1);
        if self.pending > 0 {
            self.offer();
        }
    }

    // ---------------------------------------------------------------- draw

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn put(
        &mut self,
        x: f32,
        y: f32,
        hw: f32,
        hh: f32,
        rot: f32,
        part: f32,
        flash: f32,
        seed: f32,
        color: [f32; 4],
    ) {
        if self.inst.len() >= MAX_INST {
            return;
        }
        // Cull what the camera cannot see.
        let m = hw.abs().max(hh) + 24.0;
        if (x - self.cam.0).abs() > self.view_w + m || (y - self.cam.1).abs() > self.view_h + m {
            return;
        }
        self.inst.push(Inst { x, y, hw, hh, rot, part, flash, seed, color });
    }

    fn build(&mut self) {
        self.inst.clear();
        self.pointers.clear();
        self.draw_ground();
        self.draw_wall();
        self.draw_hold_ground();
        self.accept_draw_ground();
        self.commission_draw_ground();
        if self.mission == M_ESCAPE {
            self.escape_draw();
        }
        self.vendetta_draw_ground();

        // Ground-level effects and pickups.
        for i in 0..self.fx.len() {
            if !self.fx[i].top {
                self.draw_fx(i);
            }
        }
        for i in 0..self.gems.len() {
            let g = &self.gems[i];
            let (x, y, seed, heart) = (g.x, g.y, g.seed, g.heart);
            // Two-pose rock, like a paper tag on a string.
            let pose = (((self.anim * 4.0 + seed * 9.0).floor() as i32) & 1) as f32;
            let rot = pose * 0.3 - 0.15;
            if heart {
                self.put(x, y - 12.0, 11.0, 11.0, rot, P_HEART, 0.0, seed, [0.78, 0.27, 0.18, 1.0]);
            } else {
                let big = self.gems[i].value >= 5.0;
                let s = if big { 10.0 } else { 7.0 };
                let c = if big { [0.85, 0.72, 0.30, 1.0] } else { [0.36, 0.72, 0.56, 1.0] };
                self.put(x, y - 8.0, s * 0.8, s, rot, P_GEM, 0.0, seed, c);
            }
        }
        self.boss_draw_ground();

        self.draw_formation();
        let mut canvas = Canvas {
            inst: &mut self.inst,
            points: &mut self.pointers,
            cam: self.cam,
            view: (self.view_w, self.view_h),
        };
        self.retrieval.draw_ground(&mut canvas, self.anim);
        self.draw_escort_goal();

        // Characters, sorted by depth.
        self.order.clear();
        for (i, e) in self.enemies.iter().enumerate() {
            self.order.push((e.y, i as u16));
        }
        self.order.push((self.player.y, u16::MAX));
        if self.escort.active && !self.escort.fallen {
            self.order.push((self.escort.y, u16::MAX - 1));
        }
        self.order.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));
        for n in 0..self.order.len() {
            let idx = self.order[n].1;
            if idx == u16::MAX {
                self.draw_player();
            } else if idx == u16::MAX - 1 {
                self.draw_escort();
            } else {
                self.draw_enemy(idx as usize);
            }
        }

        self.vendetta_draw_marks();

        // Things in the air.
        self.draw_arts();
        self.boss_draw_air();
        for i in 0..self.fx.len() {
            if self.fx[i].top {
                self.draw_fx(i);
            }
        }
        self.draw_nums();
        self.draw_formation_marker();
        let mut canvas = Canvas {
            inst: &mut self.inst,
            points: &mut self.pointers,
            cam: self.cam,
            view: (self.view_w, self.view_h),
        };
        self.retrieval.draw_top(&mut canvas, (self.player.x, self.player.y), self.anim);
        self.pursuit_draw_markers();
        self.draw_escort_markers();
        self.draw_hold_marker();
        self.accept_draw_marker();
        self.commission_draw_marker();
        // The host shows how far away each marked thing is.
        let (px, py) = (self.player.x, self.player.y);
        for p in self.pointers.iter_mut() {
            p[2] = ((p[0] - px) * (p[0] - px) + (p[1] - py) * (p[1] - py)).sqrt();
        }
    }

    /// Note that an edge-of-screen marker drawn this frame points at this world position.
    pub(crate) fn point(&mut self, x: f32, y: f32) {
        self.pointers.push([x, y, 0.0, 0.0]);
    }

    fn draw_fx(&mut self, i: usize) {
        let f = &self.fx[i];
        let t = (f.life / f.max).clamp(0.0, 1.0);
        let mut c = f.color;
        c[3] *= t.sqrt();
        let (x, y, hw, hh, rot, part, seed) = (f.x, f.y, f.hw, f.hh, f.rot, f.part, f.seed);
        self.put(x, y, hw, hh, rot, part, 0.0, seed, c);
    }

    /// Sparse ink strokes and stones so movement reads against the paper.
    fn draw_ground(&mut self) {
        const G: f32 = 170.0;
        let x0 = ((self.cam.0 - self.view_w) / G).floor() as i32;
        let x1 = ((self.cam.0 + self.view_w) / G).floor() as i32;
        let y0 = ((self.cam.1 - self.view_h) / G).floor() as i32;
        let y1 = ((self.cam.1 + self.view_h) / G).floor() as i32;
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                let h = hash2(cx, cy);
                if h % 100 >= 55 {
                    continue;
                }
                let fx = ((h >> 8) & 255) as f32 / 255.0;
                let fy = ((h >> 16) & 255) as f32 / 255.0;
                let x = (cx as f32 + fx) * G;
                let y = (cy as f32 + fy) * G;
                let seed = (h & 255) as f32 / 255.0;
                if (h >> 24).is_multiple_of(4) {
                    self.put(
                        x,
                        y,
                        16.0 + seed * 12.0,
                        8.0 + seed * 5.0,
                        0.0,
                        P_STONE,
                        0.0,
                        seed,
                        [INK[0], INK[1], INK[2], 0.16],
                    );
                } else {
                    let flip = if h & 1 == 0 { 1.0 } else { -1.0 };
                    self.put(
                        x,
                        y,
                        (10.0 + seed * 6.0) * flip,
                        10.0 + seed * 8.0,
                        0.0,
                        P_GRASS,
                        0.0,
                        seed,
                        [0.22, 0.36, 0.30, 0.45],
                    );
                }
            }
        }
    }

    /// The edge of a walled field (the `arena_radius` tune): a ring of stones around the origin.
    fn draw_wall(&mut self) {
        let wall = self.tune[T_ARENA_RADIUS];
        if wall <= 0.0 {
            return;
        }
        // One stone about every 70 units; `put` drops the ones out of view.
        let n = ((TAU * wall / 70.0) as usize).clamp(12, 400);
        for k in 0..n {
            let a = k as f32 * TAU / n as f32;
            let (x, y) = (fmath::cos(a) * (wall + 34.0), fmath::sin(a) * (wall + 34.0));
            let seed = (hash2(k as i32, 77) & 255) as f32 / 255.0;
            self.put(
                x,
                y,
                22.0 + seed * 8.0,
                12.0 + seed * 5.0,
                0.0,
                P_STONE,
                0.0,
                seed,
                [INK[0], INK[1], INK[2], 0.6],
            );
        }
    }

    fn draw_enemy(&mut self, i: usize) {
        let e = self.enemies[i];
        let k = &KINDS[e.kind as usize];
        if e.id == mission_pursuit::PURSUIT_ID {
            self.pursuit_draw_target(&e);
            return self.draw_tags(i, e.y - k.hh * 2.0);
        }
        if self.boss.is(e.id) {
            self.boss_draw_body(i);
            return self.draw_tags(i, e.y - k.hh * 2.0);
        }
        // Two poses, flipped back and forth. That is the whole walk cycle.
        let pose = ((self.anim * k.rate + e.seed * 7.0).floor() as i32) & 1;
        let (tilt, hop) = if pose == 0 { (0.13, 0.0) } else { (-0.13, -3.0) };
        let c = [k.color[0], k.color[1], k.color[2], 1.0];
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        self.put(e.x, e.y, k.r * 0.95, k.r * 0.36, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);
        if k.hover {
            // Wisps hover and flap by squeezing their width.
            let hw = if pose == 0 { k.hw } else { k.hw * 0.7 };
            let bob = fmath::sin(self.anim * 5.0 + e.seed * 20.0) * 3.0;
            self.put(e.x, e.y - 26.0 + bob, hw * e.face, k.hh, tilt * 0.5, k.part, e.flash, e.seed, c);
            self.draw_tags(i, e.y - 26.0 - k.hh);
        } else {
            self.put(e.x, e.y - k.hh + hop, k.hw * e.face, k.hh, tilt, k.part, e.flash, e.seed, c);
            self.draw_tags(i, e.y - k.hh * 2.0);
        }
    }

    fn draw_player(&mut self) {
        let p = &self.player;
        let (px, py, f) = (p.x, p.y, p.face);
        let (tilt, bob) = if p.moving {
            if ((p.walk * 9.0).floor() as i32) & 1 == 0 {
                (0.10, 0.0)
            } else {
                (-0.10, -3.0)
            }
        } else if ((self.anim * 2.0).floor() as i32) & 1 == 0 {
            (0.0, 0.0)
        } else {
            (0.0, -1.5)
        };
        // Blink while invulnerable.
        let a = if p.iframe > 0.0 && ((p.iframe * 20.0) as i32) & 1 == 1 { 0.4 } else { 1.0 };
        let flash = if p.iframe > 0.65 { 1.0 } else { 0.0 };
        let attack = p.attack;

        let shadow = [INK[0], INK[1], INK[2], 0.22];
        self.put(px, py, 20.0, 7.0, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);

        let scythe = [0.36, 0.66, 0.52, a];
        let swinging = attack > 0.0;
        if !swinging {
            // Resting over the shoulder, behind the body.
            self.put(px - f * 15.0, py - 46.0 + bob, 20.0 * f, 34.0, -f * 0.45 + tilt, P_SCYTHE, flash, 0.21, scythe);
        }
        let top_x = px + 26.0 * fmath::sin(tilt);
        self.put(px, py - 26.0 + bob, 20.0 * f, 26.0, tilt, P_PLAYER_BODY, flash, 0.37, [0.20, 0.42, 0.36, a]);
        self.put(
            top_x + f * 2.0,
            py - 60.0 + bob * 1.5,
            15.0 * f,
            15.0,
            -tilt * 0.6,
            P_PLAYER_HEAD,
            flash,
            0.53,
            [0.15, 0.33, 0.29, a],
        );
        if swinging {
            let t = 1.0 - attack / 0.25;
            let ease = t * t * (3.0 - 2.0 * t);
            let ang = (-1.3 + 2.8 * ease) * f;
            self.put(px + f * 12.0, py - 42.0 + bob, 20.0 * f, 34.0, ang, P_SCYTHE, flash, 0.21, scythe);
        }
        self.draw_guard();
    }

    // --------------------------------------------------------------- state

    /// HUD state read by the host. The layout is the `[[state]]` list in
    /// `content/game.toml`.
    fn write_state(&mut self) {
        let (goal, progress) = (self.goal(), self.progress());
        let boss_wait = self.boss_wait();
        let guard_cap = self.guard_cap();
        let band = self.commission_band_left();
        let p = &self.player;
        let s = &mut self.state;
        s[S_HP] = p.hp;
        s[S_MAX_HP] = p.max_hp;
        s[S_XP] = p.xp;
        s[S_XP_NEXT] = p.xp_next;
        s[S_LEVEL] = p.level as f32;
        s[S_TIME] = self.time;
        s[S_KILLS] = self.kills as f32;
        s[S_CHOOSING] = self.choosing as u8 as f32;
        for c in 0..3 {
            s[S_CHOICE0 + c] = self.choices[c] as f32;
        }
        s[S_OVER] = self.over as u8 as f32;
        s[S_CAM_X] = self.cam.0;
        s[S_CAM_Y] = self.cam.1;
        s[S_ENEMIES] = self.enemies.len() as f32;
        s[S_RUNNING] = self.running as u8 as f32;
        s[S_MISSION] = self.mission as f32;
        s[S_GOAL] = goal;
        s[S_PROGRESS] = progress;
        s[S_WON] = self.won as u8 as f32;
        s[S_CARRIED] = self.retrieval.carried() as f32;
        s[S_POINT_HP] = self.hold.hp;
        s[S_POINT_MAX] = self.hold.max_hp;
        s[S_POINT_FELL] = self.hold.fell as u8 as f32;
        s[S_EXIT_X] = self.escape.x;
        s[S_EXIT_Y] = self.escape.y;
        s[S_EXIT_DIST] = self.escape.dist(p.x, p.y);
        s[S_BOSS_WAIT] = boss_wait;
        s[S_ACCEPTING] = self.accept.active as u8 as f32;
        s[S_ACCEPT_HELD] = if self.accept.active { self.accept.held } else { 0.0 };
        s[S_ACCEPT_NEED] = if self.accept.active { self.accept.need } else { 0.0 };
        for id in 0..N_UPGRADES {
            s[S_UPGRADE0 + id] = self.up[id] as f32;
        }
        s[S_ATTACH] = self.attach as f32;
        let c = &self.commission;
        let shown = self.mission == M_ENDLESS && self.tune[T_COMMISSIONS] > 0.5;
        s[S_COMMISSION] = if shown { c.phase as f32 } else { 0.0 };
        s[S_COMMISSION_LEFT] = c.left.max(0.0);
        s[S_COMMISSION_NEED] = c.need();
        s[S_COMMISSION_HELD] = c.held;
        s[S_COMMISSION_HP] = c.hp;
        s[S_COMMISSION_MAX] = c.max_hp();
        s[S_COMMISSION_DONE] = c.done as f32;
        s[S_COMMISSION_KIND] = c.kind as f32;
        s[S_COMMISSION_BAND] = band as f32;
        s[S_COMMISSION_WALKED] = c.walked(c.start);
        s[S_GUARD] = p.guard;
        s[S_GUARD_MAX] = guard_cap;
        s[S_REGROW] = p.regrow;
        for (a, art) in self.arts.iter().enumerate() {
            s[S_ART_ELEMENT0 + a] = art.element as f32;
        }
        self.pursuit_write_state();
        self.escort_state();
    }
}

#[path = "arts.rs"]
pub mod arts;
#[path = "commission.rs"]
pub mod commission;
#[path = "elements.rs"]
pub mod elements;
#[path = "mission_boss.rs"]
mod mission_boss;
#[path = "mission_escort.rs"]
pub mod mission_escort;
#[path = "mission_pursuit.rs"]
mod mission_pursuit;
#[path = "mission_vendetta.rs"]
mod mission_vendetta;
#[path = "replay.rs"]
pub mod replay;

#[cfg(test)]
#[path = "game_tests.rs"]
mod game_tests;

#[cfg(test)]
#[path = "element_tests.rs"]
mod element_tests;
