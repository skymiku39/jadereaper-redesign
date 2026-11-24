//! The `escape` mission: reach the exit while surrounded.
//!
//! The exit sits `MISSION_GOAL` world units from the start, in a direction
//! drawn from the run's seed. The run starts inside a ring of enemies, and
//! extra enemies keep appearing in the direction of the exit, so the way there
//! has to be fought open. Enemies are slower than the player and never block
//! him, so without that pressure the mission would be a plain walk.

use crate::defs::*;
use crate::fmath;
use crate::game::Game;
use core::f32::consts::TAU;

/// The player wins when he is this close to the exit.
const EXIT_R: f32 = 48.0;
/// The band of distances the enemies of the starting ring stand at. How many
/// there are is `T_ESCAPE_RING_COUNT`.
const RING_NEAR: f32 = 150.0;
const RING_FAR: f32 = 260.0;
/// Extra enemies appear ahead of the player, `T_ESCAPE_AHEAD_RATE` per second,
/// in a cone this many radians to each side of the direction to the exit. They
/// stand beyond the normal spawn ring, spread over `AHEAD_DEPTH` units of distance.
const AHEAD_CONE: f32 = 0.8;
const AHEAD_DEPTH: f32 = 300.0;
/// Chances of a brute (`T_ESCAPE_BRUTE_SHARE`) and of a wisp among the enemies
/// of the ring and the stream ahead; the rest are blots.
const WISP_SHARE: f32 = 0.25;

/// What the mission keeps between steps. Zeroed for every other mission.
#[derive(Default)]
pub struct Escape {
    /// Where the exit is, in world units.
    pub x: f32,
    pub y: f32,
    /// Enemies owed by the stream ahead; a whole one is spawned per unit.
    acc: f32,
}

impl Escape {
    /// Distance from a point to the exit.
    pub fn dist(&self, x: f32, y: f32) -> f32 {
        let (dx, dy) = (self.x - x, self.y - y);
        (dx * dx + dy * dy).sqrt()
    }

    /// Reached the exit.
    pub fn won(&self, x: f32, y: f32) -> bool {
        self.dist(x, y) <= EXIT_R
    }

    /// Words for the state hash.
    pub fn hash_words(&self) -> [u32; 3] {
        [self.x.to_bits(), self.y.to_bits(), self.acc.to_bits()]
    }
}

impl Game {
    /// Mission progress: the distance closed since the start, as it is now. It
    /// drops again when the player turns back.
    pub(crate) fn escape_progress(&self) -> f32 {
        (self.goal() - self.escape.dist(self.player.x, self.player.y)).max(0.0)
    }

    /// Place the exit and the starting ring. Called once, from `Game::start`.
    pub(crate) fn escape_begin(&mut self) {
        let a = self.rng.range(0.0, TAU);
        self.escape.x = self.player.x + fmath::cos(a) * self.goal();
        self.escape.y = self.player.y + fmath::sin(a) * self.goal();
        let ring_count = self.tune[T_ESCAPE_RING_COUNT] as u32;
        for i in 0..ring_count {
            let a = (i as f32 + self.rng.f()) * TAU / ring_count as f32;
            let r = self.rng.range(RING_NEAR, RING_FAR);
            let kind = self.escape_kind();
            let (x, y) = (self.player.x + fmath::cos(a) * r, self.player.y + fmath::sin(a) * r);
            self.spawn_enemy_at(kind, x, y);
        }
    }

    fn escape_kind(&mut self) -> u8 {
        let roll = self.rng.f();
        let brute = self.tune[T_ESCAPE_BRUTE_SHARE];
        if roll < brute {
            K_BRUTE
        } else if roll < brute + WISP_SHARE {
            K_WISP
        } else {
            K_BLOT
        }
    }

    /// The stream of enemies ahead of the player, on top of the normal spawning.
    pub(crate) fn escape_spawn(&mut self, dt: f32) {
        self.escape.acc += self.tune[T_ESCAPE_AHEAD_RATE] * dt;
        while self.escape.acc >= 1.0 {
            self.escape.acc -= 1.0;
            let (dx, dy) = (self.escape.x - self.player.x, self.escape.y - self.player.y);
            let a = fmath::atan2(dy, dx) + self.rng.range(-AHEAD_CONE, AHEAD_CONE);
            let r = self.spawn_ring() + self.rng.range(0.0, AHEAD_DEPTH);
            let kind = self.escape_kind();
            let (x, y) = (self.player.x + fmath::cos(a) * r, self.player.y + fmath::sin(a) * r);
            self.spawn_enemy_at(kind, x, y);
        }
    }

    /// The exit: a gold ring on the ground that swells in two poses.
    pub(crate) fn escape_draw(&mut self) {
        let (x, y) = (self.escape.x, self.escape.y);
        // The host draws the arrow to the exit while it is off screen.
        if (x - self.cam.0).abs() > self.view_w || (y - self.cam.1).abs() > self.view_h {
            self.point(x, y);
        }
        let pose = (self.anim * 3.0).floor() as i32 & 1;
        let swell = if pose == 0 { 1.0 } else { 1.12 };
        let gold = [0.85, 0.72, 0.30];
        self.put(x, y, EXIT_R * 1.1, EXIT_R * 0.8, 0.0, P_ELLIPSE, 0.0, 0.1, [gold[0], gold[1], gold[2], 0.25]);
        self.put(
            x,
            y,
            EXIT_R * swell * 1.2,
            EXIT_R * swell * 0.9,
            0.0,
            P_RING,
            0.0,
            0.4,
            [gold[0], gold[1], gold[2], 0.95],
        );
        self.put(x, y, EXIT_R * 0.6, EXIT_R * 0.45, 0.0, P_RING, 0.0, 0.8, [0.25, 0.62, 0.48, 0.8]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RING_COUNT: u32 = TUNE_DEFAULT[T_ESCAPE_RING_COUNT] as u32;
    const AHEAD_RATE: f32 = TUNE_DEFAULT[T_ESCAPE_AHEAD_RATE];

    fn start(seed: u32) -> Game {
        let mut game = Game::new(seed);
        game.start(M_ESCAPE);
        game
    }

    /// Play a bot until the run ends or `seconds` pass. `dodge` makes it steer
    /// away from enemies close by. Upgrades are taken as they come (the run
    /// stops while a card is open) and the player is not healed. Returns the
    /// end state: (won, over, seconds, level).
    fn run_bot(seed: u32, dodge: bool, seconds: u32) -> (bool, bool, f32, u32) {
        let mut game = start(seed);
        for _ in 0..(60 * seconds) {
            let (px, py) = (game.player.x, game.player.y);
            let (mut mx, mut my) = (game.escape.x - px, game.escape.y - py);
            let d = (mx * mx + my * my).sqrt().max(0.001);
            mx /= d;
            my /= d;
            if dodge {
                for e in &game.enemies {
                    let (ex, ey) = (px - e.x, py - e.y);
                    let ed = (ex * ex + ey * ey).sqrt().max(0.001);
                    if ed < 160.0 {
                        let w = (1.0 - ed / 160.0) * 2.5;
                        mx += ex / ed * w;
                        my += ey / ed * w;
                    }
                }
            }
            game.input = (mx, my);
            game.events.clear();
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            if game.won || game.over {
                break;
            }
        }
        (game.won, game.over, game.time, game.player.level)
    }

    #[test]
    #[ignore = "prints bot results for tuning; run with --ignored --nocapture"]
    fn bot_report() {
        for dodge in [false, true] {
            let mut wins = 0;
            for seed in 1..=12 {
                let (won, over, t, level) = run_bot(seed * 7919, dodge, 600);
                wins += won as u32;
                println!("dodge={dodge} seed={seed} won={won} died={over} time={t:.0}s level={level}");
            }
            println!("dodge={dodge}: {wins}/12 won");
        }
    }

    #[test]
    fn walking_straight_at_the_exit_is_not_enough() {
        for seed in [1, 2, 3, 4] {
            let (won, over, t, _) = run_bot(seed, false, 300);
            assert!(!won && over, "seed {seed} walked in: won={won} over={over} at {t}s");
        }
    }

    #[test]
    fn the_run_starts_surrounded() {
        let game = start(4);
        assert_eq!(game.enemies.len() as u32, RING_COUNT);
        for e in &game.enemies {
            let d = ((e.x - game.player.x).powi(2) + (e.y - game.player.y).powi(2)).sqrt();
            assert!((RING_NEAR - 1.0..=RING_FAR + 1.0).contains(&d), "enemy at {d}");
        }
        // They stand all the way round, not in one clump.
        let mut octants = [0; 8];
        for e in &game.enemies {
            let a = fmath::atan2(e.y - game.player.y, e.x - game.player.x);
            octants[(((a / TAU + 0.5) * 8.0) as usize).min(7)] += 1;
        }
        assert!(octants.iter().all(|&n| n > 0), "{octants:?}");
    }

    #[test]
    fn the_exit_is_goal_units_away_and_the_same_for_a_seed() {
        let a = start(9);
        let b = start(9);
        let c = start(10);
        assert!(a.goal() > 0.0);
        assert!((a.escape.dist(0.0, 0.0) - a.goal()).abs() < 1.0);
        assert_eq!((a.escape.x, a.escape.y), (b.escape.x, b.escape.y));
        assert_ne!((a.escape.x, a.escape.y), (c.escape.x, c.escape.y));
        assert_eq!(a.hash(), b.hash());
    }

    #[test]
    fn other_missions_have_no_exit_and_no_ring() {
        let mut game = Game::new(9);
        game.start(M_SURVIVE);
        assert!(game.enemies.is_empty());
        assert_eq!((game.escape.x, game.escape.y), (0.0, 0.0));
    }

    #[test]
    fn progress_follows_the_distance_to_the_exit() {
        let mut game = start(5);
        let goal = game.goal();
        assert!(game.progress() < 0.01);
        // Standing on the start, or further from the exit than that, is zero.
        game.player.x -= (game.escape.x - game.player.x) * 0.5;
        game.player.y -= (game.escape.y - game.player.y) * 0.5;
        assert_eq!(game.progress(), 0.0);

        let mut game = start(5);
        let (ex, ey) = (game.escape.x, game.escape.y);
        game.player.x += (ex - game.player.x) * 0.25;
        game.player.y += (ey - game.player.y) * 0.25;
        assert!((game.progress() - goal * 0.25).abs() < 1.0);
        game.player.x += (ex - game.player.x) * 0.5;
        game.player.y += (ey - game.player.y) * 0.5;
        assert!((game.progress() - goal * 0.625).abs() < 1.0);
        game.update(crate::game::TICK);
        assert_eq!(game.state[S_GOAL], goal);
        assert_eq!(game.state[S_PROGRESS], game.progress());
        assert_eq!(game.state[S_MISSION], M_ESCAPE as f32);
        assert_eq!((game.state[S_EXIT_X], game.state[S_EXIT_Y]), (ex, ey));
        assert!((game.state[S_EXIT_DIST] - game.escape.dist(game.player.x, game.player.y)).abs() < 0.01);
    }

    #[test]
    fn the_mission_is_won_at_the_exit_and_not_before() {
        let mut game = start(6);
        let (ex, ey) = (game.escape.x, game.escape.y);
        // Just outside the exit: the run goes on.
        game.player.x = ex - EXIT_R - 5.0;
        game.player.y = ey;
        game.player.hp = 1e6;
        game.player.max_hp = 1e6;
        game.tick();
        assert!(!game.won && !game.over);
        // Inside it: the win is on the very next step, and the run freezes.
        game.player.x = ex - EXIT_R + 5.0;
        game.tick();
        assert!(game.won && !game.over);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));
        let ticks = game.ticks;
        game.update(crate::game::TICK);
        assert_eq!(game.ticks, ticks + 1);
        assert_eq!(game.state[S_WON], 1.0);
    }

    #[test]
    fn the_mission_is_lost_on_death() {
        let mut game = start(3);
        // A player who stands still and takes no cards falls in the end.
        game.tune[T_ROUTES] = 0.0;
        for _ in 0..(60 * 600) {
            game.tick();
            if game.choosing {
                game.pending = 0;
                game.choosing = false;
            }
            if game.over {
                break;
            }
        }
        assert!(game.over && !game.won);
    }

    #[test]
    fn enemies_keep_appearing_ahead_of_the_player() {
        let mut game = start(8);
        game.enemies.clear();
        game.player.hp = 1e6;
        game.player.max_hp = 1e6;
        // No weapons fire in a few seconds, so what spawns stays where it was put.
        let (dx, dy) = (game.escape.x - game.player.x, game.escape.y - game.player.y);
        let d = (dx * dx + dy * dy).sqrt();
        let (fx, fy) = (dx / d, dy / d);
        let before = game.rng.0;
        for _ in 0..(60 * 5) {
            game.escape_spawn(crate::game::TICK);
        }
        assert_ne!(game.rng.0, before);
        assert!(game.enemies.len() >= (AHEAD_RATE * 5.0) as usize - 1);
        for e in &game.enemies {
            let (ex, ey) = (e.x - game.player.x, e.y - game.player.y);
            let ed = (ex * ex + ey * ey).sqrt();
            assert!(ed >= game.spawn_ring() - 1.0);
            // Within the cone (a little slack for the rounding of the angle).
            assert!((ex * fx + ey * fy) / ed > fmath::cos(AHEAD_CONE) - 0.01, "enemy off to the side");
        }
    }

    #[test]
    fn the_exit_is_pointed_at_with_its_distance_while_off_screen() {
        let mut game = start(2);
        game.update(crate::game::TICK);
        assert_eq!(game.pointers.len(), 1);
        let p = game.pointers[0];
        assert_eq!((p[0], p[1]), (game.escape.x, game.escape.y));
        assert!((p[2] - game.escape.dist(game.player.x, game.player.y)).abs() < 0.01);
        // On screen, the arrow and its label are gone.
        game.player.x = game.escape.x - 100.0;
        game.player.y = game.escape.y;
        game.update(crate::game::TICK);
        assert!(game.pointers.is_empty());
        // Other missions point at nothing here.
        let mut other = Game::new(2);
        other.start(M_SURVIVE);
        other.update(crate::game::TICK);
        assert!(other.pointers.is_empty());
    }

    #[test]
    fn the_exit_is_drawn_when_on_screen() {
        let mut game = start(2);
        game.player.x = game.escape.x - 100.0;
        game.player.y = game.escape.y;
        game.update(crate::game::TICK);
        let near = game.inst.iter().filter(|i| i.part == P_RING).count();
        assert!(near >= 2, "{near} rings");
        game.player.x = game.escape.x - 5000.0;
        game.update(crate::game::TICK);
        assert_eq!(game.inst.iter().filter(|i| i.part == P_RING).count(), 0);
    }

    #[test]
    fn tunes_change_the_ring_and_the_stream() {
        let mut game = Game::new(4);
        game.set_tune(T_ESCAPE_GOAL, 2000.0);
        game.set_tune(T_ESCAPE_RING_COUNT, 120.0);
        game.set_tune(T_ESCAPE_BRUTE_SHARE, 0.5);
        game.start(M_ESCAPE);
        assert_eq!(game.goal(), 2000.0);
        assert!((game.escape.dist(0.0, 0.0) - 2000.0).abs() < 1.0);
        assert_eq!(game.enemies.len(), 120);
        let brutes = game.enemies.iter().filter(|e| e.kind == K_BRUTE).count();
        assert!(brutes > 30, "{brutes} brutes of 120 at a 0.5 share");

        let mut game = Game::new(4);
        game.set_tune(T_ESCAPE_RING_COUNT, 0.0);
        game.set_tune(T_ESCAPE_AHEAD_RATE, 80.0);
        game.start(M_ESCAPE);
        assert!(game.enemies.is_empty());
        for _ in 0..60 {
            game.escape_spawn(crate::game::TICK);
        }
        assert!(game.enemies.len() >= 79, "{} ahead", game.enemies.len());
    }

    #[test]
    fn no_ring_and_no_stream_is_a_plain_walk() {
        let mut game = Game::new(4);
        game.set_tune(T_ESCAPE_RING_COUNT, 0.0);
        game.set_tune(T_ESCAPE_AHEAD_RATE, 0.0);
        game.set_tune(T_ESCAPE_BRUTE_SHARE, 0.0);
        game.set_tune(T_ESCAPE_GOAL, 2000.0);
        game.start(M_ESCAPE);
        for _ in 0..300 {
            game.player.hp = game.player.max_hp;
            game.tick();
            if game.choosing {
                game.choose(0);
            }
        }
        assert!(game.progress().is_finite() && !game.over);
        assert_eq!(game.escape.hash_words()[2], 0.0f32.to_bits());
    }
}
