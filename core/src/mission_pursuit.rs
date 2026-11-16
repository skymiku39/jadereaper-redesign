//! The pursuit mission: catch and kill a target before it escapes.
//!
//! The target is an ordinary entry of `Game::enemies` (kind elite, a reserved
//! id, `goal` health), so every weapon already hurts it and the normal reaping
//! removes it. What differs is that it walks a fixed route away from the
//! player instead of towards them, never leaves that route, and does no
//! contact damage. It runs at about 90% of the player's speed and every few
//! seconds stops to call a pack of enemies behind it, between itself and the
//! player. The run is lost, through the same `over` flag as dying, when the
//! target reaches the exit at the end of its route.
//!
//! Included from `game.rs` so it can reach the private fields of `Game`.

use super::*;

/// Id of the target in `Game::enemies`. Normal enemies count up from 1.
pub const PURSUIT_ID: u32 = u32::MAX;

/// The route is this many straight legs of `T_PURSUIT_LEG_LEN`; the last waypoint is the exit.
const LEGS: usize = 6;
/// Each leg heads within this many radians of the direction the target first runs.
const WANDER: f32 = 0.9;
/// The target runs for this long, then stops for `PAUSE_TIME` and calls a pack.
const RUN_TIME: f32 = 8.0;
const PAUSE_TIME: f32 = 3.0;
/// A waypoint counts as reached this close to it.
const REACH: f32 = 40.0;
/// Margin of the off-screen marker from the screen edge.
const MARKER_MARGIN: f32 = 44.0;

const GOLD: [f32; 4] = [0.92, 0.70, 0.16, 1.0];

#[derive(Default)]
pub struct Pursuit {
    /// A pursuit run is set up.
    active: bool,
    /// The target reached the exit.
    escaped: bool,
    start: (f32, f32),
    route: [(f32, f32); LEGS],
    /// Index of the waypoint the target is heading to.
    leg: usize,
    /// Time in the current phase.
    clock: f32,
    pausing: bool,
    /// Unit vector of the way the target is heading.
    dir: (f32, f32),
    /// How far along the route the target is, 0..1.
    frac: f32,
}

impl Game {
    /// Lay out the route from the run's own generator and put the target on it.
    pub(super) fn pursuit_start(&mut self) {
        let a0 = self.rng.range(0.0, TAU);
        let (px, py) = (self.player.x, self.player.y);
        // The target starts `T_PURSUIT_START_DIST` away, just past the edge of the screen at the default.
        let start_dist = self.tune[T_PURSUIT_START_DIST];
        let leg_len = self.tune[T_PURSUIT_LEG_LEN];
        let start = (px + fmath::cos(a0) * start_dist, py + fmath::sin(a0) * start_dist);
        let mut route = [(0.0, 0.0); LEGS];
        let mut at = start;
        for w in route.iter_mut() {
            let h = a0 + self.rng.range(-WANDER, WANDER);
            at = (at.0 + fmath::cos(h) * leg_len, at.1 + fmath::sin(h) * leg_len);
            *w = at;
        }
        let seed = self.rng.f();
        self.enemies.push(Enemy {
            x: start.0,
            y: start.1,
            kx: 0.0,
            ky: 0.0,
            hp: self.goal(),
            kind: K_ELITE,
            flash: 0.0,
            seed,
            bead_cd: 0.0,
            tags: Default::default(),
            id: PURSUIT_ID,
            face: 1.0,
        });
        self.pursuit = Pursuit {
            active: true,
            escaped: false,
            start,
            route,
            leg: 0,
            clock: 0.0,
            pausing: false,
            dir: (fmath::cos(a0), fmath::sin(a0)),
            frac: 0.0,
        };
    }

    fn pursuit_index(&self) -> Option<usize> {
        self.enemies.iter().position(|e| e.id == PURSUIT_ID)
    }

    /// Move the target along its route and call packs. Does nothing outside
    /// the mission.
    pub(super) fn pursuit_step(&mut self, dt: f32) {
        if !self.pursuit.active || self.pursuit.escaped {
            return;
        }
        let Some(i) = self.pursuit_index() else { return };

        // Phases: run, then pause and call a pack.
        let mut summon = false;
        {
            let p = &mut self.pursuit;
            p.clock += dt;
            if p.pausing {
                if p.clock >= PAUSE_TIME {
                    p.pausing = false;
                    p.clock = 0.0;
                }
            } else if p.clock >= RUN_TIME {
                p.pausing = true;
                p.clock = 0.0;
                summon = true;
            }
        }

        let mut reached_exit = false;
        let leg_len_set = self.tune[T_PURSUIT_LEG_LEN];
        // The player walks at 190.
        let speed = self.tune[T_PURSUIT_SPEED];
        {
            let p = &mut self.pursuit;
            let e = &mut self.enemies[i];
            // Weapons shove it; it holds its line anyway.
            e.kx = 0.0;
            e.ky = 0.0;
            e.flash = (e.flash - dt * 6.0).max(0.0);
            let from = if p.leg == 0 { p.start } else { p.route[p.leg - 1] };
            let to = p.route[p.leg];
            let (dx, dy) = (to.0 - e.x, to.1 - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            // How far along the route it is, never going back.
            let (fx, fy) = (to.0 - from.0, to.1 - from.1);
            let leg_len = (fx * fx + fy * fy).sqrt().max(0.001);
            let done = p.leg as f32 * leg_len_set + (leg_len - d).clamp(0.0, leg_len);
            p.frac = p.frac.max((done / (LEGS as f32 * leg_len_set)).clamp(0.0, 1.0));
            if d < REACH {
                if p.leg + 1 == LEGS {
                    reached_exit = true;
                } else {
                    p.leg += 1;
                }
            } else if !p.pausing {
                p.dir = (dx / d, dy / d);
                e.x += p.dir.0 * speed * dt;
                e.y += p.dir.1 * speed * dt;
                if p.dir.0.abs() > 0.1 {
                    e.face = p.dir.0.signum();
                }
            }
        }

        if summon {
            self.pursuit_summon(i);
        }
        if reached_exit {
            self.pursuit.escaped = true;
            self.pursuit.frac = 1.0;
            // Lost without dying: the run ends through the same flag.
            self.over = true;
            self.event(EV_DEATH, self.player.x, self.player.y, 0.0);
        }
    }

    /// A pack of enemies appears behind the target, towards the player.
    fn pursuit_summon(&mut self, i: usize) {
        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
        let (dx, dy) = self.pursuit.dir;
        let behind = fmath::atan2(-dy, -dx);
        for _ in 0..self.tune[T_PURSUIT_PACK] as usize {
            let a = behind + self.rng.range(-1.2, 1.2);
            let r = self.rng.range(110.0, 200.0);
            let roll = self.rng.f();
            let kind = if roll < 0.55 {
                K_BLOT
            } else if roll < 0.85 || self.time < 60.0 {
                K_WISP
            } else {
                K_BRUTE
            };
            self.spawn_enemy_at(kind, x + fmath::cos(a) * r, y + fmath::sin(a) * r);
        }
        self.add_fx(Fx {
            x,
            y: y - 10.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.5,
            max: 0.5,
            part: P_RING,
            hw: 30.0,
            hh: 30.0,
            rot: 0.0,
            vr: 0.0,
            grow: 260.0,
            color: GOLD,
            seed: 0.4,
            top: true,
        });
        self.event(EV_ELITE, x, y, 0.0);
    }

    /// Damage dealt to the target; the goal once it is dead.
    pub(super) fn pursuit_progress(&self) -> f32 {
        let goal = self.goal();
        match self.pursuit_index() {
            Some(i) => (goal - self.enemies[i].hp).clamp(0.0, goal),
            None if self.pursuit.active && !self.pursuit.escaped => goal,
            None => 0.0,
        }
    }

    /// The target is dead and did not get away first.
    pub(super) fn pursuit_won(&self) -> bool {
        self.pursuit.active && !self.pursuit.escaped && self.pursuit_index().is_none()
    }

    /// Fields that decide how the run continues, for the state hash.
    pub(super) fn pursuit_hash(&self) -> [u32; 5] {
        let p = &self.pursuit;
        [p.leg as u32, p.clock.to_bits(), p.pausing as u32, p.escaped as u32, p.frac.to_bits()]
    }

    pub(super) fn pursuit_write_state(&mut self) {
        if self.mission == M_PURSUIT {
            self.state[S_ESCAPE] = self.pursuit.frac;
            self.state[S_ESCAPED] = self.pursuit.escaped as u8 as f32;
        }
    }

    /// The target: a gold brute on a gold ring, so it stands out in a crowd.
    pub(super) fn pursuit_draw_target(&mut self, e: &Enemy) {
        let k = &KINDS[K_ELITE as usize];
        let pose = ((self.anim * k.rate + e.seed * 7.0).floor() as i32) & 1;
        let (tilt, hop) = if pose == 0 { (0.13, 0.0) } else { (-0.13, -3.0) };
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        let pulse = 66.0 + 5.0 * fmath::sin(self.anim * 5.0);
        self.put(e.x, e.y, k.r * 0.95, k.r * 0.36, 0.0, P_GROUND_SHADOW, 0.0, 0.0, shadow);
        self.put(e.x, e.y - 8.0, pulse, pulse, 0.0, P_RING, 0.0, e.seed, [GOLD[0], GOLD[1], GOLD[2], 0.75]);
        self.put(e.x, e.y - k.hh + hop, k.hw * e.face, k.hh, tilt, P_BRUTE, e.flash, e.seed, GOLD);
    }

    /// The exit when it is on screen, and an arrow at the screen edge pointing
    /// at the target when it is not.
    pub(super) fn pursuit_draw_markers(&mut self) {
        if !self.pursuit.active || self.pursuit.escaped {
            return;
        }
        let exit = self.pursuit.route[LEGS - 1];
        let beat = 0.5 + 0.5 * fmath::sin(self.anim * 3.0);
        self.put(exit.0, exit.1, 130.0, 130.0, 0.0, P_RING, 0.0, 0.2, [GOLD[0], GOLD[1], GOLD[2], 0.35 + 0.3 * beat]);
        let Some(i) = self.pursuit_index() else { return };
        let (dx, dy) = (self.enemies[i].x - self.cam.0, self.enemies[i].y - self.cam.1);
        let (mx, my) = (self.view_w - MARKER_MARGIN, self.view_h - MARKER_MARGIN);
        if dx.abs() <= mx && dy.abs() <= my {
            return;
        }
        self.point(self.cam.0 + dx, self.cam.1 + dy);
        // Pull the point back along the line from the screen centre until it
        // sits inside the margin.
        let t = (mx / dx.abs().max(0.001)).min(my / dy.abs().max(0.001));
        let (x, y) = (self.cam.0 + dx * t, self.cam.1 + dy * t);
        let rot = fmath::atan2(dy, dx) + TAU * 0.25;
        self.put(x, y, 12.0, 20.0, rot, P_TALISMAN, 0.0, 0.5, GOLD);
        self.put(x, y, 30.0, 30.0, 0.0, P_RING, 0.0, 0.5, [GOLD[0], GOLD[1], GOLD[2], 0.6 + 0.3 * beat]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEG_LEN: f32 = TUNE_DEFAULT[T_PURSUIT_LEG_LEN];
    const START_DIST: f32 = TUNE_DEFAULT[T_PURSUIT_START_DIST];
    const PACK: usize = TUNE_DEFAULT[T_PURSUIT_PACK] as usize;

    /// Play on with the player kept healthy until the run ends or `seconds`
    /// pass. `bot` sets the movement input before each step.
    fn play(game: &mut Game, seconds: u32, mut bot: impl FnMut(&mut Game)) {
        for _ in 0..(60 * seconds) {
            game.player.hp = game.player.max_hp;
            bot(game);
            game.events.clear();
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            if game.won || game.over {
                break;
            }
        }
    }

    fn target(game: &Game) -> Option<Enemy> {
        game.enemies.iter().find(|e| e.id == PURSUIT_ID).copied()
    }

    /// Walk at the target, and stand still once close, so the scythe keeps
    /// facing it.
    fn chase(game: &mut Game) {
        if let Some(t) = target(game) {
            let (dx, dy) = (t.x - game.player.x, t.y - game.player.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            game.input = if d > 90.0 { (dx / d, dy / d) } else { (0.0, 0.0) };
        }
    }

    /// Walk away from enemies that are close and ignore the target.
    fn dodge(game: &mut Game) {
        let (px, py) = (game.player.x, game.player.y);
        let (mut ax, mut ay) = (0.0, 0.0);
        for e in game.enemies.iter().filter(|e| e.id != PURSUIT_ID) {
            let (dx, dy) = (px - e.x, py - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < 250.0 {
                ax += dx / d * (250.0 - d);
                ay += dy / d * (250.0 - d);
            }
        }
        let len = (ax * ax + ay * ay).sqrt();
        game.input = if len > 0.001 { (ax / len, ay / len) } else { (0.0, 0.0) };
    }

    fn started(seed: u32) -> Game {
        let mut game = Game::new(seed);
        game.start(M_PURSUIT);
        game
    }

    #[test]
    fn the_target_starts_off_screen_with_goal_health() {
        let game = started(4);
        let t = target(&game).expect("a target");
        assert_eq!(t.hp, game.goal());
        assert!(game.goal() > 0.0);
        let d = (t.x * t.x + t.y * t.y).sqrt();
        assert!((d - START_DIST).abs() < 1.0, "{d}");
        assert_eq!(game.progress(), 0.0);
        assert!(!game.won && !game.over);
    }

    #[test]
    fn the_same_seed_gives_the_same_route() {
        let (a, b, c) = (started(9), started(9), started(10));
        assert_eq!(a.pursuit.route, b.pursuit.route);
        assert_ne!(a.pursuit.route, c.pursuit.route);
        // Every leg is as long as set, so the route length does not depend on the seed.
        let mut at = a.pursuit.start;
        for w in a.pursuit.route {
            let (dx, dy) = (w.0 - at.0, w.1 - at.1);
            assert!(((dx * dx + dy * dy).sqrt() - LEG_LEN).abs() < 5.0);
            at = w;
        }
        // And the run itself repeats.
        let (mut x, mut y) = (started(9), started(9));
        for _ in 0..600 {
            x.tick();
            y.tick();
        }
        assert_eq!(x.hash(), y.hash());
    }

    #[test]
    fn progress_follows_the_damage_dealt() {
        let mut game = started(4);
        let goal = game.goal();
        let i = game.pursuit_index().unwrap();
        game.enemies[i].hp = goal - 250.0;
        assert_eq!(game.progress(), 250.0);
        game.update(TICK);
        assert_eq!(game.state[S_GOAL], goal);
        assert_eq!(game.state[S_PROGRESS], 250.0);
        assert_eq!(game.state[S_MISSION], M_PURSUIT as f32);
        assert!(!game.won);
        // Not a count of kills: killing other enemies changes nothing.
        game.kills += 40;
        assert_eq!(game.progress(), 250.0);
    }

    #[test]
    fn only_killing_the_target_wins() {
        let mut game = started(4);
        let i = game.pursuit_index().unwrap();
        // Almost dead is not dead.
        game.enemies[i].hp = 0.5;
        game.tick();
        assert!(!game.won);
        game.enemies[0].hp = 0.0;
        game.tick();
        assert!(game.won && !game.over);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));
        assert_eq!(game.progress(), game.goal());
        game.update(TICK);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_PROGRESS], game.goal());
        assert_eq!(game.state[S_ESCAPED], 0.0);
    }

    #[test]
    fn the_target_cannot_die_of_anything_but_damage() {
        // Nothing in the step touches its health, and it is exempt from the
        // rule that brings far-away enemies back to the player.
        let mut game = started(4);
        play(&mut game, 20, |g| g.input = (0.0, 0.0));
        let t = target(&game).expect("still there");
        assert_eq!(t.hp, game.goal());
        assert!(game.progress() == 0.0 && !game.won);
    }

    #[test]
    fn the_target_walks_its_route_and_escapes_when_nobody_stops_it() {
        let mut game = started(4);
        play(&mut game, 400, |g| g.input = (0.0, 0.0));
        assert!(game.over && !game.won, "no escape in 400 s");
        assert!(game.pursuit.escaped);
        assert!(game.time > 120.0 && game.time < 300.0, "escaped after {} s", game.time);
        assert!(game.events.iter().any(|e| e[0] == EV_DEATH));
        // The player was never hurt to death.
        assert!(game.player.hp > 0.0);
        game.update(TICK);
        assert_eq!(game.state[S_OVER], 1.0);
        assert_eq!(game.state[S_ESCAPED], 1.0);
        assert_eq!(game.state[S_ESCAPE], 1.0);
        assert_eq!(game.state[S_WON], 0.0);
    }

    #[test]
    fn the_escape_meter_climbs_along_the_route() {
        let mut game = started(4);
        let mut last = 0.0;
        for _ in 0..(60 * 60) {
            game.player.hp = game.player.max_hp;
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            assert!(game.pursuit.frac >= last - 1e-4);
            last = game.pursuit.frac;
        }
        assert!(last > 0.15 && last < 0.6, "{last} after a minute");
    }

    #[test]
    fn dying_loses_the_mission_without_an_escape() {
        let mut game = started(4);
        game.player.hp = 1.0;
        game.enemies.push(Enemy {
            x: 0.0,
            y: 0.0,
            kx: 0.0,
            ky: 0.0,
            hp: 100.0,
            kind: K_BRUTE,
            flash: 0.0,
            seed: 0.5,
            bead_cd: 0.0,
            tags: Default::default(),
            id: 1,
            face: 1.0,
        });
        game.tick();
        assert!(game.over && !game.won);
        assert!(!game.pursuit.escaped);
        game.update(TICK);
        assert_eq!(game.state[S_ESCAPED], 0.0);
    }

    #[test]
    fn packs_appear_behind_the_target() {
        let mut game = started(4);
        let before = game.enemies.len();
        game.pursuit.clock = RUN_TIME;
        game.tick();
        assert!(game.pursuit.pausing);
        assert!(game.enemies.len() >= before + PACK);
        assert!(game.events.iter().any(|e| e[0] == EV_ELITE));
    }

    #[test]
    fn a_marker_points_at_an_off_screen_target() {
        let mut game = started(4);
        game.set_view(640.0, 360.0);
        game.present();
        let t = target(&game).unwrap();
        // The target is 800 away, past the 640 by 360 view: the arrow is drawn
        // at the edge. Find the talisman part among the instances.
        let arrow = game.inst.iter().find(|i| i.part == P_TALISMAN).expect("arrow");
        assert!((arrow.x - game.cam.0).abs() <= 640.0 && (arrow.y - game.cam.1).abs() <= 360.0);
        // It sits on the same side of the screen centre as the target.
        let (tx, ty) = (t.x - game.cam.0, t.y - game.cam.1);
        assert!((arrow.x - game.cam.0) * tx >= 0.0 && (arrow.y - game.cam.1) * ty >= 0.0);
        // Close up, there is no arrow.
        game.player.x = t.x - 100.0;
        game.player.y = t.y;
        game.present();
        assert!(game.inst.iter().all(|i| i.part != P_TALISMAN));
    }

    #[test]
    fn chasing_through_the_crowd_wins() {
        let mut won = 0;
        for seed in [1, 3, 5] {
            let mut game = started(seed);
            play(&mut game, 400, chase);
            if game.won {
                won += 1;
            }
            eprintln!(
                "chase seed {seed}: won {} at {:.0} s, escape {:.2}, kills {}",
                game.won, game.time, game.pursuit.frac, game.kills
            );
        }
        assert!(won >= 2, "the chase won only {won} of 3 runs");
    }

    #[test]
    fn running_from_enemies_does_not_win() {
        for seed in [1, 3, 5] {
            let mut game = started(seed);
            play(&mut game, 400, dodge);
            assert!(game.over && !game.won, "seed {seed}: won {} over {}", game.won, game.over);
            assert!(game.pursuit.escaped, "seed {seed} did not see the target escape");
            assert_eq!(game.progress(), 0.0);
        }
    }

    #[test]
    fn tunes_change_the_route_and_the_target() {
        let mut game = Game::new(4);
        game.set_tune(T_PURSUIT_GOAL, 500.0);
        game.set_tune(T_PURSUIT_LEG_LEN, 1200.0);
        game.set_tune(T_PURSUIT_START_DIST, 300.0);
        game.set_tune(T_PURSUIT_PACK, 24.0);
        game.start(M_PURSUIT);
        let t = target(&game).unwrap();
        assert_eq!(t.hp, 500.0);
        assert_eq!(game.goal(), 500.0);
        assert!((t.x.hypot(t.y) - 300.0).abs() < 1.0);
        let mut at = game.pursuit.start;
        for w in game.pursuit.route {
            assert!(((w.0 - at.0).hypot(w.1 - at.1) - 1200.0).abs() < 5.0);
            at = w;
        }
        let before = game.enemies.len();
        game.pursuit.clock = RUN_TIME;
        game.tick();
        assert!(game.enemies.len() >= before + 24);
    }

    #[test]
    fn a_target_without_a_pack_still_runs() {
        let mut game = Game::new(4);
        game.set_tune(T_PURSUIT_PACK, 0.0);
        game.set_tune(T_PURSUIT_SPEED, 220.0);
        game.set_tune(T_PURSUIT_LEG_LEN, 1200.0);
        game.start(M_PURSUIT);
        let before = game.enemies.len();
        game.pursuit.clock = RUN_TIME;
        game.tick();
        assert!(game.pursuit.pausing);
        assert_eq!(game.enemies.len(), before);
        play(&mut game, 120, |g| g.input = (0.0, 0.0));
        assert!(game.pursuit.frac > 0.0);
    }
}
