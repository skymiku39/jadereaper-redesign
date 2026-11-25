//! The boss mission: a short warm-up of the normal horde, then one boss with a
//! large health pool that cycles through three patterns, each announced by a
//! telegraph: a charge (a lane marks where it will dash), a ring of ink shots
//! (with gaps to weave between) and a summoned pack. The mission is won when
//! the boss dies. Its health is the mission goal and the damage the player's
//! weapons have dealt is the progress.
//!
//! The boss is an ordinary elite in `Game::enemies`, so every weapon already
//! hurts it. Two hooks in `Game::move_enemies` and `Game::draw_enemy` let this
//! file steer and draw it; everything else lives here. All numbers are below.

use super::*;

// ------------------------------------------------------------------ numbers

// Three numbers come from the run's tunes: the seconds of normal horde before
// the boss arrives (`T_BOSS_WARMUP`), the fraction of the normal spawn rate
// once the boss is here (`T_BOSS_SPAWN_SCALE`) and its walking speed toward the
// player (`T_BOSS_WALK_SPEED`).

/// Farther than `NEAR_DIST` from the player, the boss speeds up by `CATCH_GAIN`
/// per unit of distance, up to `CATCH_SPEED` (faster than the player at any
/// speed level), so it cannot be left behind. Patterns start within
/// `PATTERN_RANGE`.
const NEAR_DIST: f32 = 250.0;
const CATCH_GAIN: f32 = 1.5;
const CATCH_SPEED: f32 = 480.0;
const PATTERN_RANGE: f32 = 650.0;
/// Seconds of walking between two patterns, and of rest after one.
const WALK_TIME: f32 = 2.5;
const REST_TIME: f32 = 0.7;
/// The patterns run in this order, over and over.
const ORDER: [Pattern; 3] = [Pattern::Charge, Pattern::Ring, Pattern::Summon];

/// Charge: telegraph, then a straight dash. The aim freezes `CHARGE_LOCK`
/// seconds before the dash so it can be dodged sideways.
const CHARGE_TELE: f32 = 1.0;
const CHARGE_LOCK: f32 = 0.3;
const CHARGE_SPEED: f32 = 560.0;
const CHARGE_TIME: f32 = 0.7;
/// Half width of the lane shown by the telegraph.
const CHARGE_WIDTH: f32 = 54.0;

/// Ring of ink shots: `RING_WAVES` rings, the later ones turned half a step
/// so the gaps of one ring are filled by the next.
const RING_TELE: f32 = 0.9;
const RING_COUNT: u32 = 14;
const RING_WAVES: u32 = 2;
const RING_GAP: f32 = 0.8;
const RING_AFTER: f32 = 0.4;
const SHOT_SPEED: f32 = 190.0;
const SHOT_LIFE: f32 = 4.5;
const SHOT_RADIUS: f32 = 9.0;
const SHOT_DMG: f32 = 10.0;
const MAX_SHOTS: usize = 96;

/// Summon: a pack of normal enemies in a circle around the boss.
const SUMMON_TELE: f32 = 1.0;
const SUMMON_AFTER: f32 = 0.3;
const PACK_BLOTS: u32 = 6;
const PACK_WISPS: u32 = 2;
const PACK_RADIUS: f32 = 110.0;

/// The boss is drawn this much larger than an elite.
const BOSS_SCALE: f32 = 1.3;

// -------------------------------------------------------------------- state

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pattern {
    Charge,
    Ring,
    Summon,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Walking toward the player until the next pattern.
    Walk,
    /// Standing still, showing what comes next.
    Tele(Pattern),
    /// Doing it.
    Act(Pattern),
    /// Standing still after a pattern, open to attack.
    Rest,
}

impl Phase {
    fn code(self) -> u32 {
        match self {
            Phase::Walk => 0,
            Phase::Tele(p) => 1 + p as u32,
            Phase::Act(p) => 4 + p as u32,
            Phase::Rest => 7,
        }
    }
}

/// An ink shot. Enemy shots do not exist elsewhere in the core.
pub struct InkShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
}

pub struct Boss {
    /// Id of the boss in `Game::enemies`, 0 until it arrives.
    id: u32,
    pub phase: Phase,
    /// Seconds in the current phase.
    t: f32,
    /// How many patterns have started; picks the next one.
    next: usize,
    /// Charge direction.
    dir: (f32, f32),
    /// Velocity the boss wants this step.
    vel: (f32, f32),
    /// Rings fired or packs summoned in this pattern.
    wave: u32,
    pub shots: Vec<InkShot>,
}

impl Default for Boss {
    fn default() -> Boss {
        Boss {
            id: 0,
            phase: Phase::Walk,
            t: 0.0,
            next: 0,
            dir: (1.0, 0.0),
            vel: (0.0, 0.0),
            wave: 0,
            shots: Vec::new(),
        }
    }
}

impl Boss {
    /// True if the enemy with this id is the boss.
    pub fn is(&self, id: u32) -> bool {
        self.id != 0 && self.id == id
    }

    /// The velocity the boss steers itself with, for the boss's own id.
    pub fn drive(&self, id: u32) -> Option<(f32, f32)> {
        if self.is(id) {
            Some(self.vel)
        } else {
            None
        }
    }

    fn enter(&mut self, phase: Phase) {
        self.phase = phase;
        self.t = 0.0;
        self.wave = 0;
    }

    /// What the run hash mixes in.
    pub fn hash_words(&self) -> Vec<u32> {
        let mut w = vec![self.id, self.phase.code(), self.t.to_bits(), self.next as u32, self.wave];
        w.push(self.shots.len() as u32);
        for s in &self.shots {
            w.push(s.x.to_bits());
            w.push(s.y.to_bits());
        }
        w
    }
}

fn tele_time(p: Pattern) -> f32 {
    match p {
        Pattern::Charge => CHARGE_TELE,
        Pattern::Ring => RING_TELE,
        Pattern::Summon => SUMMON_TELE,
    }
}

// --------------------------------------------------------------------- game

impl Game {
    fn boss_index(&self) -> Option<usize> {
        if self.boss.id == 0 {
            return None;
        }
        self.enemies.iter().position(|e| e.id == self.boss.id)
    }

    /// Damage dealt to the boss, in health. All of it once the boss is dead.
    pub(super) fn boss_progress(&self) -> f32 {
        let goal = self.goal();
        if self.boss.id == 0 {
            return 0.0;
        }
        match self.boss_index() {
            Some(i) => (goal - self.enemies[i].hp).clamp(0.0, goal),
            None => goal,
        }
    }

    /// The boss has arrived and is gone from the field: it died.
    pub(super) fn boss_dead(&self) -> bool {
        self.boss.id != 0 && self.boss_index().is_none()
    }

    /// Seconds until the boss arrives, 0 once it is here or in other missions.
    pub(super) fn boss_wait(&self) -> f32 {
        if self.mission == M_BOSS && self.boss.id == 0 {
            (self.tune[T_BOSS_WARMUP] - self.time).max(0.0)
        } else {
            0.0
        }
    }

    /// The horde thins out while the boss is alive.
    pub(super) fn boss_spawn_scale(&self) -> f32 {
        if self.mission == M_BOSS && self.boss.id != 0 {
            self.tune[T_BOSS_SPAWN_SCALE]
        } else {
            1.0
        }
    }

    /// One step of the boss mission, before enemies move.
    pub(super) fn boss_think(&mut self, dt: f32) {
        if self.mission != M_BOSS {
            return;
        }
        if self.boss.id == 0 {
            if self.time >= self.tune[T_BOSS_WARMUP] {
                self.boss_arrive();
            }
            return;
        }
        self.boss_move_shots(dt);
        let Some(i) = self.boss_index() else {
            self.boss.vel = (0.0, 0.0);
            return;
        };
        let (bx, by) = (self.enemies[i].x, self.enemies[i].y);
        let (dx, dy) = (self.player.x - bx, self.player.y - by);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        let to = (dx / d, dy / d);

        self.boss.t += dt;
        let t = self.boss.t;
        let mut vel = (0.0, 0.0);
        match self.boss.phase {
            Phase::Walk => {
                let speed = (self.tune[T_BOSS_WALK_SPEED] + CATCH_GAIN * (d - NEAR_DIST).max(0.0)).min(CATCH_SPEED);
                vel = (to.0 * speed, to.1 * speed);
                // Far away, keep running; patterns only start once it is near.
                if t >= WALK_TIME && d <= PATTERN_RANGE {
                    let pattern = ORDER[self.boss.next % ORDER.len()];
                    self.boss.next += 1;
                    self.boss.enter(Phase::Tele(pattern));
                    self.boss.dir = to;
                    self.event(EV_ELITE, bx, by, 0.0);
                }
            }
            Phase::Tele(p) => {
                let dur = tele_time(p);
                if p == Pattern::Charge && t < dur - CHARGE_LOCK {
                    self.boss.dir = to;
                }
                if t >= dur {
                    self.boss.enter(Phase::Act(p));
                }
            }
            Phase::Act(Pattern::Charge) => {
                vel = (self.boss.dir.0 * CHARGE_SPEED, self.boss.dir.1 * CHARGE_SPEED);
                if t >= CHARGE_TIME {
                    self.boss.enter(Phase::Rest);
                }
            }
            Phase::Act(Pattern::Ring) => {
                while self.boss.wave < RING_WAVES && t >= self.boss.wave as f32 * RING_GAP {
                    self.boss_fire_ring(bx, by, self.boss.wave);
                    self.boss.wave += 1;
                }
                if self.boss.wave >= RING_WAVES && t >= (RING_WAVES - 1) as f32 * RING_GAP + RING_AFTER {
                    self.boss.enter(Phase::Rest);
                }
            }
            Phase::Act(Pattern::Summon) => {
                if self.boss.wave == 0 {
                    self.boss.wave = 1;
                    self.boss_summon(bx, by);
                }
                if t >= SUMMON_AFTER {
                    self.boss.enter(Phase::Rest);
                }
            }
            Phase::Rest => {
                if t >= REST_TIME {
                    self.boss.enter(Phase::Walk);
                }
            }
        }
        self.boss.vel = vel;
    }

    /// The boss steps in from the edge of the screen.
    fn boss_arrive(&mut self) {
        let a = self.rng.range(0.0, TAU);
        let ring = self.spawn_ring();
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1).max(1);
        let seed = self.rng.f();
        self.enemies.push(Enemy {
            x: self.player.x + fmath::cos(a) * ring,
            y: self.player.y + fmath::sin(a) * ring,
            kx: 0.0,
            ky: 0.0,
            hp: self.goal(),
            kind: K_ELITE,
            flash: 0.0,
            seed,
            bead_cd: 0.0,
            tags: Default::default(),
            id,
            face: 1.0,
        });
        self.boss.id = id;
        self.boss.enter(Phase::Walk);
        self.event(EV_ELITE, self.player.x, self.player.y, 0.0);
    }

    fn boss_fire_ring(&mut self, x: f32, y: f32, wave: u32) {
        let step = TAU / RING_COUNT as f32;
        let turn = if wave % 2 == 1 { step * 0.5 } else { 0.0 };
        for k in 0..RING_COUNT {
            if self.boss.shots.len() >= MAX_SHOTS {
                break;
            }
            let a = turn + k as f32 * step;
            self.boss.shots.push(InkShot {
                x,
                y,
                vx: fmath::cos(a) * SHOT_SPEED,
                vy: fmath::sin(a) * SHOT_SPEED,
                life: SHOT_LIFE,
            });
        }
        self.event(EV_SHOOT, x, y, 0.0);
    }

    fn boss_summon(&mut self, x: f32, y: f32) {
        let n = PACK_BLOTS + PACK_WISPS;
        let scale = self.hp_growth();
        for k in 0..n {
            if self.enemies.len() >= MAX_ENEMIES {
                break;
            }
            let kind = if k < PACK_BLOTS { K_BLOT } else { K_WISP };
            let a = (k as f32 + self.rng.range(-0.3, 0.3)) * TAU / n as f32;
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            let seed = self.rng.f();
            self.enemies.push(Enemy {
                x: x + fmath::cos(a) * PACK_RADIUS,
                y: y + fmath::sin(a) * PACK_RADIUS * 0.8,
                kx: 0.0,
                ky: 0.0,
                hp: KINDS[kind as usize].hp * scale,
                kind,
                flash: 0.0,
                seed,
                bead_cd: 0.0,
                tags: Default::default(),
                id,
                face: 1.0,
            });
        }
        self.event(EV_ELITE, x, y, 0.0);
    }

    /// Move the ink shots and let them hurt the player.
    fn boss_move_shots(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let reach = SHOT_RADIUS + 10.0;
        let mut i = 0;
        while i < self.boss.shots.len() {
            let s = &mut self.boss.shots[i];
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.life -= dt;
            let (dx, dy) = (px - s.x, py - s.y);
            let hit = self.player.iframe <= 0.0 && dx * dx + dy * dy < reach * reach;
            if s.life <= 0.0 || hit {
                self.boss.shots.swap_remove(i);
                if hit {
                    self.boss_hurt_player(SHOT_DMG);
                }
            } else {
                i += 1;
            }
        }
    }

    fn boss_hurt_player(&mut self, dmg: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let dmg = self.soften(dmg);
        self.player.hp -= self.guard_against(dmg);
        self.player.iframe = 0.8;
        self.shake = 1.0;
        self.event(EV_HURT, px, py, dmg);
        if self.player.hp <= 0.0 && !self.last_stand() {
            self.player.hp = 0.0;
            self.over = true;
            self.event(EV_DEATH, px, py, 0.0);
        }
    }

    // ---------------------------------------------------------------- draw

    /// The boss itself: a big dark-red brute with a ring under its feet.
    pub(super) fn boss_draw_body(&mut self, i: usize) {
        let e = self.enemies[i];
        let k = &KINDS[e.kind as usize];
        let pose = ((self.anim * k.rate).floor() as i32) & 1;
        let (tilt, hop) = if pose == 0 { (0.10, 0.0) } else { (-0.10, -4.0) };
        let s = BOSS_SCALE;
        // Blink while it shows what comes next.
        let blink = matches!(self.boss.phase, Phase::Tele(_)) && ((self.boss.t * 8.0) as i32) & 1 == 0;
        let flash = if blink { 0.7 } else { e.flash };
        let shadow = [INK[0], INK[1], INK[2], 0.25];
        self.put(e.x, e.y, k.r * 1.2 * s, k.r * 0.45 * s, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);
        let pulse = 1.0 + 0.06 * fmath::sin(self.anim * 4.0);
        self.put(e.x, e.y - 6.0, k.r * 1.5 * pulse, k.r * 0.7 * pulse, 0.0, P_RING, 0.0, 0.4, [0.62, 0.10, 0.14, 0.7]);
        self.put(
            e.x,
            e.y - k.hh * s + hop,
            k.hw * s * e.face,
            k.hh * s,
            tilt,
            P_BRUTE,
            flash,
            e.seed,
            [0.42, 0.08, 0.17, 1.0],
        );
    }

    /// Telegraphs, drawn on the ground under the characters.
    pub(super) fn boss_draw_ground(&mut self) {
        if self.mission != M_BOSS {
            return;
        }
        let Some(i) = self.boss_index() else {
            return;
        };
        let (bx, by) = (self.enemies[i].x, self.enemies[i].y);
        let Phase::Tele(p) = self.boss.phase else {
            return;
        };
        let f = (self.boss.t / tele_time(p)).clamp(0.0, 1.0);
        match p {
            Pattern::Charge => {
                let (dx, dy) = self.boss.dir;
                let len = CHARGE_SPEED * CHARGE_TIME;
                let (cx, cy) = (bx + dx * len * 0.5, by + dy * len * 0.5);
                let rot = fmath::atan2(dy, dx);
                self.put(
                    cx,
                    cy,
                    len * 0.5,
                    CHARGE_WIDTH,
                    rot,
                    P_ELLIPSE,
                    0.0,
                    0.1,
                    [0.80, 0.18, 0.12, 0.15 + 0.35 * f],
                );
            }
            Pattern::Ring => {
                let r = 40.0 + 90.0 * f;
                self.put(bx, by - 6.0, r, r * 0.8, 0.0, P_RING, 0.0, 0.2, [0.10, 0.09, 0.09, 0.3 + 0.5 * f]);
            }
            Pattern::Summon => {
                let r = PACK_RADIUS + 20.0 - 20.0 * f;
                self.put(bx, by - 6.0, r, r * 0.8, 0.0, P_RING, 0.0, 0.3, [0.36, 0.40, 0.70, 0.3 + 0.5 * f]);
            }
        }
    }

    /// Ink shots, and an arrow at the edge of the screen when the boss is off it.
    pub(super) fn boss_draw_air(&mut self) {
        if self.mission != M_BOSS || self.boss.id == 0 {
            return;
        }
        let ink = [0.07, 0.06, 0.06, 1.0];
        for i in 0..self.boss.shots.len() {
            let s = &self.boss.shots[i];
            let (x, y) = (s.x, s.y);
            self.put(
                x,
                y,
                SHOT_RADIUS * 1.1,
                SHOT_RADIUS * 0.4,
                0.0,
                P_GROUND_SHADOW,
                0.0,
                0.0,
                [INK[0], INK[1], INK[2], 0.25],
            );
            self.put(x, y - 14.0, SHOT_RADIUS * 1.2, SHOT_RADIUS * 1.2, 0.0, P_BLOT, 0.0, i as f32 * 0.17, ink);
        }
        let Some(i) = self.boss_index() else {
            return;
        };
        let (cx, cy) = self.cam;
        let (dx, dy) = (self.enemies[i].x - cx, self.enemies[i].y - 40.0 - cy);
        if dx.abs() < self.view_w - 20.0 && dy.abs() < self.view_h - 20.0 {
            return;
        }
        self.point(cx + dx, cy + dy + 40.0);
        let m = 44.0;
        let kx = (self.view_w - m) / dx.abs().max(0.001);
        let ky = (self.view_h - m) / dy.abs().max(0.001);
        let k = kx.min(ky);
        let (x, y) = (cx + dx * k, cy + dy * k);
        let rot = fmath::atan2(dy, dx) + TAU * 0.25;
        self.put(x, y, 22.0, 22.0, 0.0, P_RING, 0.0, 0.6, [0.62, 0.10, 0.14, 0.8]);
        self.put(x, y, 11.0, 20.0, rot, P_TALISMAN, 0.0, 0.5, [0.80, 0.18, 0.12, 1.0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARMUP: f32 = TUNE_DEFAULT[T_BOSS_WARMUP];

    /// A new run of the boss mission.
    fn fight(seed: u32) -> Game {
        let mut game = Game::new(seed);
        game.start(M_BOSS);
        game
    }

    /// One step with a player nothing can kill, and a boss no weapon can kill.
    fn step_immortal(game: &mut Game) {
        game.player.hp = game.player.max_hp;
        game.player.iframe = 1.0;
        if let Some(i) = game.boss_index() {
            game.enemies[i].hp = game.goal();
        }
        game.tick();
        if game.choosing {
            game.choose(0);
        }
    }

    #[test]
    fn boss_appears_after_the_warm_up() {
        let mut game = fight(3);
        assert!(game.goal() > 0.0);
        game.time = WARMUP - 1.0;
        for _ in 0..50 {
            step_immortal(&mut game);
        }
        assert!(game.boss_index().is_none());
        assert!(game.boss_wait() > 0.0);
        assert_eq!(game.progress(), 0.0);
        assert!(!game.won);
        for _ in 0..20 {
            step_immortal(&mut game);
        }
        let i = game.boss_index().expect("no boss after the warm-up");
        assert_eq!(game.enemies[i].hp, game.goal());
        assert_eq!(game.boss_wait(), 0.0);
        assert!(!game.won && game.progress() == 0.0);
        assert!(game.events.iter().any(|e| e[0] == EV_ELITE));
    }

    #[test]
    fn other_missions_have_no_boss() {
        for m in [M_ENDLESS, M_SURVIVE, M_KILLS] {
            let mut game = Game::new(3);
            game.start(m);
            game.time = WARMUP;
            for _ in 0..30 {
                step_immortal(&mut game);
            }
            assert!(game.boss_index().is_none() && game.boss.id == 0);
            assert_eq!(game.boss_wait(), 0.0);
            assert_eq!(game.boss_spawn_scale(), 1.0);
        }
    }

    #[test]
    fn the_three_patterns_happen_and_the_charge_is_telegraphed() {
        let mut game = fight(9);
        game.time = WARMUP;
        let (mut prev, mut in_phase) = (Phase::Walk, 0u32);
        let (mut charge, mut ring, mut summon) = (false, false, false);
        let mut charge_from = (0.0, 0.0);
        for _ in 0..60 * 45 {
            step_immortal(&mut game);
            let now = game.boss.phase;
            if now != prev {
                if let Phase::Act(p) = now {
                    // Every pattern is announced by its own telegraph.
                    assert_eq!(prev, Phase::Tele(p), "{p:?} started without a telegraph");
                    let secs = in_phase as f32 * TICK;
                    assert!(secs >= tele_time(p) - 0.05, "{p:?} telegraph lasted only {secs}");
                }
                match now {
                    Phase::Act(Pattern::Charge) => {
                        charge = true;
                        let i = game.boss_index().unwrap();
                        charge_from = (game.enemies[i].x, game.enemies[i].y);
                    }
                    Phase::Act(Pattern::Ring) => ring = true,
                    Phase::Act(Pattern::Summon) => summon = true,
                    Phase::Rest if prev == Phase::Act(Pattern::Charge) => {
                        let i = game.boss_index().unwrap();
                        let (dx, dy) = (game.enemies[i].x - charge_from.0, game.enemies[i].y - charge_from.1);
                        let dist = (dx * dx + dy * dy).sqrt();
                        assert!(dist > CHARGE_SPEED * CHARGE_TIME * 0.7, "the dash covered {dist}");
                    }
                    _ => {}
                }
                prev = now;
                in_phase = 0;
            }
            in_phase += 1;
            if game.boss.phase == Phase::Act(Pattern::Ring) && game.boss.wave >= 1 {
                assert!(game.boss.shots.len() >= RING_COUNT as usize);
            }
        }
        assert!(charge && ring && summon, "charge {charge}, ring {ring}, summon {summon}");
    }

    #[test]
    fn a_summon_adds_a_pack() {
        let mut game = fight(4);
        game.time = WARMUP;
        game.boss_arrive();
        game.enemies.truncate(1);
        game.boss.enter(Phase::Tele(Pattern::Summon));
        let mut before = 0;
        for _ in 0..120 {
            before = game.enemies.len();
            step_immortal(&mut game);
            if game.boss.phase == Phase::Act(Pattern::Summon) {
                // The pack arrives on the first step of the pattern.
                step_immortal(&mut game);
                break;
            }
        }
        assert!(game.enemies.len() >= before + (PACK_BLOTS + PACK_WISPS) as usize - 1);
    }

    #[test]
    fn ink_shots_hurt_on_contact_and_expire() {
        let mut game = fight(5);
        game.time = WARMUP;
        game.boss_arrive();
        game.boss.shots.clear();
        let (px, py) = (game.player.x, game.player.y);
        game.player.iframe = 0.0;
        let hp = game.player.hp;
        game.boss.shots.push(InkShot { x: px, y: py, vx: 0.0, vy: 0.0, life: 1.0 });
        game.tick();
        assert_eq!(game.player.hp, hp - SHOT_DMG);
        assert!(game.boss.shots.is_empty(), "a shot that hit stays");
        assert!(game.events.iter().any(|e| e[0] == EV_HURT));

        // A shot that never meets the player runs out of life.
        game.boss.shots.push(InkShot { x: px + 4000.0, y: py, vx: 0.0, vy: 0.0, life: 0.1 });
        for _ in 0..5 {
            game.tick();
        }
        assert_eq!(game.boss.shots.len(), 1);
        for _ in 0..3 {
            game.tick();
        }
        assert!(game.boss.shots.is_empty());

        // A real ring is dodgeable: standing still between two shots is safe.
        game.player.hp = game.player.max_hp;
        game.player.iframe = 0.0;
        game.boss.shots.clear();
        game.boss.shots.push(InkShot { x: px + 40.0, y: py, vx: 0.0, vy: 0.0, life: 1.0 });
        game.tick();
        assert_eq!(game.player.hp, game.player.max_hp);
    }

    #[test]
    fn progress_follows_damage_and_the_boss_decides_the_win() {
        let mut game = fight(6);
        game.time = WARMUP;
        game.tick();
        let goal = game.goal();
        let i = game.boss_index().unwrap();
        game.enemies[i].hp = goal - 1500.0;
        assert_eq!(game.progress(), 1500.0);
        game.enemies[i].hp = 1.0;
        for _ in 0..10 {
            game.player.hp = game.player.max_hp;
            game.tick();
        }
        assert!(!game.won && !game.over, "won with the boss still alive");
        assert!(game.progress() > goal - 2.0 && game.progress() < goal);

        let i = game.boss_index().unwrap();
        game.enemies[i].hp = 0.0;
        game.tick();
        assert!(game.won && !game.over);
        assert_eq!(game.progress(), goal);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));
        game.update(TICK);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_MISSION], M_BOSS as f32);
        assert_eq!(game.state[S_GOAL], goal);
        assert_eq!(game.state[S_PROGRESS], goal);
    }

    #[test]
    fn the_mission_is_lost_on_death() {
        let mut game = fight(7);
        game.time = WARMUP;
        game.boss_arrive();
        game.player.hp = 1.0;
        game.player.iframe = 0.0;
        let (px, py) = (game.player.x, game.player.y);
        game.boss.shots.push(InkShot { x: px, y: py, vx: 0.0, vy: 0.0, life: 1.0 });
        game.tick();
        assert!(game.over && !game.won);
        assert!(game.events.iter().any(|e| e[0] == EV_DEATH));
    }

    #[test]
    fn the_boss_keeps_closing_in() {
        let mut game = fight(8);
        game.time = WARMUP;
        // The top speed of the passive on its own; the inner art's stride is not part of this.
        game.tune[T_ROUTES] = 0.0;
        game.up[U_SWIFT] = 5;
        game.input = (1.0, 0.0);
        // Running flat out in a straight line at the top speed level: the boss
        // may stop to wind up a pattern, but it must be back on the player
        // again and again.
        let (mut nearest, mut farthest) = (f32::MAX, 0.0f32);
        for t in 0..60 * 60 {
            step_immortal(&mut game);
            if t > 60 * 20 {
                let i = game.boss_index().unwrap();
                let (dx, dy) = (game.player.x - game.enemies[i].x, game.player.y - game.enemies[i].y);
                let d = (dx * dx + dy * dy).sqrt();
                nearest = nearest.min(d);
                farthest = farthest.max(d);
            }
        }
        assert!(game.player.x > 10000.0);
        assert!(nearest < 500.0, "the boss never got within {nearest} of a runner");
        assert!(farthest < 1600.0, "the boss fell {farthest} behind");
    }

    #[test]
    fn the_same_seed_gives_the_same_fight() {
        let play = |seed: u32| {
            let mut game = fight(seed);
            game.time = WARMUP - 2.0;
            let mut bot = Rng(seed ^ 0x55aa);
            for t in 0..60 * 50 {
                if t % 30 == 0 {
                    game.input = (bot.range(-1.0, 1.0), bot.range(-1.0, 1.0));
                }
                step_immortal(&mut game);
            }
            game.hash()
        };
        assert_eq!(play(12), play(12));
        assert_ne!(play(12), play(13));
    }

    // -------------------------------------------------------------- bots

    #[derive(Clone, Copy)]
    enum Bot {
        /// Only moves away from nearby enemies.
        Flee,
        /// Walks at the boss, and backs off while hurt.
        Chase,
        /// Walks at the boss and cannot be hurt: how fast the weapons alone
        /// can take the boss down.
        Tank,
    }

    /// A step away from enemies within `range`, or none.
    fn away(game: &Game, range: f32) -> (f32, f32) {
        let (mut ax, mut ay) = (0.0, 0.0);
        for e in &game.enemies {
            let (dx, dy) = (game.player.x - e.x, game.player.y - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < range {
                let w = (range - d) / range;
                ax += dx / d * w;
                ay += dy / d * w;
            }
        }
        let len = (ax * ax + ay * ay).sqrt();
        if len > 0.001 {
            (ax / len, ay / len)
        } else {
            (0.0, 0.0)
        }
    }

    /// Upgrade picks that favour damage.
    fn pick(game: &mut Game) {
        const LIKE: [usize; 10] =
            [U_SCYTHE, U_BEADS, U_TALISMAN, U_PULSE, U_MIGHT, U_HASTE, U_VITALITY, U_MENDING, U_SWIFT, U_LURE];
        let best = (0..3)
            .filter(|&c| game.choices[c] >= 0)
            .min_by_key(|&c| LIKE.iter().position(|&u| u as i32 == game.choices[c]).unwrap_or(99))
            .unwrap_or(0);
        game.choose(best);
    }

    /// Play a run with a bot; returns (won, over, seconds, boss health left).
    fn run_bot(seed: u32, bot: Bot, secs: f32) -> (bool, bool, f32, f32) {
        let mut game = fight(seed);
        let mut retreat = false;
        for t in 0..(secs * 60.0) as u32 {
            if t % 6 == 0 {
                let hurt = game.player.hp / game.player.max_hp;
                game.input = match bot {
                    Bot::Flee => away(&game, 260.0),
                    Bot::Tank => match game.boss_index() {
                        Some(i) => (game.enemies[i].x - game.player.x, game.enemies[i].y - game.player.y),
                        None => (0.0, 0.0),
                    },
                    Bot::Chase => {
                        if retreat && hurt > 0.7 {
                            retreat = false;
                        } else if !retreat && hurt < 0.4 {
                            retreat = true;
                        }
                        match game.boss_index() {
                            Some(i) if !retreat => {
                                (game.enemies[i].x - game.player.x, game.enemies[i].y - game.player.y)
                            }
                            _ => away(&game, 300.0),
                        }
                    }
                };
            }
            if matches!(bot, Bot::Tank) {
                game.player.hp = game.player.max_hp;
            }
            game.tick();
            if game.choosing {
                pick(&mut game);
            }
            if game.won || game.over {
                break;
            }
        }
        let left = game.boss_index().map_or(0.0, |i| game.enemies[i].hp);
        (game.won, game.over, game.time, left)
    }

    #[test]
    fn a_runner_cannot_win() {
        for seed in [1, 2] {
            let (won, over, time, left) = run_bot(seed, Bot::Flee, 600.0);
            assert!(!won, "seed {seed}: a bot that only ran away won at {time}s");
            assert!(over || left > 0.0);
        }
    }

    /// Prints how the bots fare. `cargo test --release -- --ignored --nocapture bot_report`
    #[test]
    #[ignore]
    fn bot_report() {
        for bot in [Bot::Tank, Bot::Flee, Bot::Chase] {
            for seed in (1..=15).step_by(2) {
                let (won, over, time, left) = run_bot(seed, bot, 900.0);
                let name = match bot {
                    Bot::Flee => "flee ",
                    Bot::Chase => "chase",
                    Bot::Tank => "tank ",
                };
                println!("{name} seed {seed}: won {won:5} dead {over:5} at {time:6.1}s, boss left {left:6.0}");
            }
        }
    }

    #[test]
    fn tunes_change_the_boss() {
        let mut game = Game::new(3);
        game.set_tune(T_BOSS_GOAL, 2000.0);
        game.set_tune(T_BOSS_WARMUP, 30.0);
        game.set_tune(T_BOSS_WALK_SPEED, 160.0);
        game.set_tune(T_BOSS_SPAWN_SCALE, 1.0);
        game.start(M_BOSS);
        assert_eq!(game.goal(), 2000.0);
        assert_eq!(game.boss_wait(), 30.0);
        assert_eq!(game.boss_spawn_scale(), 1.0);
        game.time = 29.0;
        for _ in 0..50 {
            step_immortal(&mut game);
        }
        assert!(game.boss_index().is_none());
        for _ in 0..30 {
            step_immortal(&mut game);
        }
        let i = game.boss_index().expect("the boss arrived after 30 s");
        assert_eq!(game.enemies[i].hp, 2000.0);
        game.update(TICK);
        assert_eq!(game.state[S_GOAL], 2000.0);
        assert_eq!(game.boss_wait(), 0.0);
    }

    #[test]
    fn no_warm_up_and_no_horde_is_fine() {
        let mut game = Game::new(3);
        game.set_tune(T_BOSS_WARMUP, 0.0);
        game.set_tune(T_BOSS_SPAWN_SCALE, 0.0);
        game.start(M_BOSS);
        game.tick();
        assert!(game.boss_index().is_some(), "the boss is there on the first step");
        assert_eq!(game.boss_spawn_scale(), 0.0);
        for _ in 0..400 {
            step_immortal(&mut game);
        }
        assert!(game.enemies.iter().all(|e| e.x.is_finite() && e.y.is_finite()));
        assert!(!game.over);
    }
}
