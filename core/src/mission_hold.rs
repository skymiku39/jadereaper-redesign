//! The `hold` mission: stand inside the circle round a shrine until the hold
//! meter is full, while a share of the horde goes for the shrine instead of
//! the player. The run is lost on death or when the shrine's health is gone.
//!
//! The state lives in one `Hold` field on `Game`. `Game` calls into it from
//! `start`, `move_enemies`, `step`, `build` and `write_state`; nothing here
//! runs unless the mission is `M_HOLD`.

use crate::defs::*;
use crate::fmath;
use crate::game::Game;
use core::f32::consts::TAU;

/// Where the shrine stands: the player's start position.
pub const POINT: (f32, f32) = (0.0, 0.0);
/// Radius of the shrine itself; enemies this close to its centre gnaw at it.
pub const BODY: f32 = 30.0;
/// Damage per second an enemy at the shrine deals, as a share of its contact damage.
pub const GNAW: f32 = 0.3;

pub struct Hold {
    pub active: bool,
    /// Seconds spent inside the circle.
    pub held: f32,
    pub hp: f32,
    /// Starting (and maximum) health of the shrine, `T_HOLD_POINT_HP`.
    pub max_hp: f32,
    /// Radius of the circle the player has to stand in, `T_HOLD_RADIUS`.
    pub radius: f32,
    /// Every `share`-th enemy (by id) goes for the shrine instead of the
    /// player, `T_HOLD_SHARE`.
    share: u32,
    /// The run ended because the shrine fell, not because the player died.
    pub fell: bool,
    /// Damage dealt by enemies this step, applied in `step`.
    dmg: f32,
    /// 1 right after the shrine was hit, fading out. Draw only.
    flash: f32,
}

impl Hold {
    pub fn new() -> Hold {
        Hold {
            active: false,
            held: 0.0,
            hp: TUNE_DEFAULT[T_HOLD_POINT_HP],
            max_hp: TUNE_DEFAULT[T_HOLD_POINT_HP],
            radius: TUNE_DEFAULT[T_HOLD_RADIUS],
            share: TUNE_DEFAULT[T_HOLD_SHARE] as u32,
            fell: false,
            dmg: 0.0,
            flash: 0.0,
        }
    }

    /// Reset for a new run. `on` is true when the mission is `hold`; only then
    /// do the run's tunes apply.
    pub fn begin(&mut self, on: bool, tune: &[f32; N_TUNES]) {
        *self = Hold::new();
        self.active = on;
        if on {
            self.max_hp = tune[T_HOLD_POINT_HP];
            self.hp = self.max_hp;
            self.radius = tune[T_HOLD_RADIUS];
            self.share = (tune[T_HOLD_SHARE] as u32).max(1);
        }
    }

    /// The point an enemy walks to instead of the player, if it was sent
    /// after the shrine.
    pub fn target(&self, enemy_id: u32) -> Option<(f32, f32)> {
        if self.active && enemy_id.is_multiple_of(self.share) {
            Some(POINT)
        } else {
            None
        }
    }

    /// An enemy at distance `d` from the shrine's centre (its own radius `r`
    /// included) deals `dmg` this step.
    pub fn gnaw(&mut self, d: f32, r: f32, dmg: f32) {
        if d < r + BODY {
            self.dmg += dmg * GNAW;
        }
    }

    pub fn inside(&self, x: f32, y: f32) -> bool {
        let (dx, dy) = (x - POINT.0, y - POINT.1);
        dx * dx + dy * dy < self.radius * self.radius
    }

    /// Advance one step with the player at (`px`, `py`). True once the
    /// shrine's health is gone.
    pub fn step(&mut self, px: f32, py: f32, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        if self.inside(px, py) {
            self.held += dt;
        }
        if self.dmg > 0.0 {
            self.hp = (self.hp - self.dmg).max(0.0);
            self.flash = 1.0;
            self.dmg = 0.0;
        }
        self.flash = (self.flash - dt * 4.0).max(0.0);
        self.hp <= 0.0
    }

    pub fn progress(&self, goal: f32) -> f32 {
        self.held.min(goal)
    }

    pub fn won(&self, goal: f32) -> bool {
        self.active && self.held >= goal
    }

    /// Fold into the run's state hash. Nothing for other missions, so their
    /// hashes stay as they were.
    pub fn mix(&self, mix: &mut impl FnMut(u32)) {
        if self.active {
            mix(self.held.to_bits());
            mix(self.hp.to_bits());
            mix(self.fell as u32);
        }
    }
}

impl Default for Hold {
    fn default() -> Hold {
        Hold::new()
    }
}

impl Game {
    /// The circle and the shrine, drawn under the characters.
    pub(crate) fn draw_hold_ground(&mut self) {
        if !self.hold.active {
            return;
        }
        let inside = self.hold.inside(self.player.x, self.player.y);
        let (x, y) = POINT;
        let radius = self.hold.radius;
        let wash = if inside { 0.16 } else { 0.08 };
        self.put(x, y, radius / 0.95, radius / 0.95, 0.0, P_ELLIPSE, 0.0, 0.2, [0.25, 0.62, 0.48, wash]);
        let line = if inside { [0.85, 0.72, 0.30, 0.8] } else { [INK[0], INK[1], INK[2], 0.45] };
        self.put(x, y, radius / 0.9, radius / 0.9, 0.0, P_RING, 0.0, 0.6, line);

        // The shrine: a stone base and an upright talisman slip that turns
        // from gold to red as it is worn down.
        let life = (self.hold.hp / self.hold.max_hp.max(1.0)).clamp(0.0, 1.0);
        let c = [0.85 - 0.11 * (1.0 - life), 0.72 * life + 0.25 * (1.0 - life), 0.30 * life + 0.18 * (1.0 - life), 1.0];
        self.put(x, y, BODY * 1.3, BODY * 0.55, 0.0, P_STONE, 0.0, 0.4, [INK[0], INK[1], INK[2], 0.35]);
        self.put(x, y - 34.0, 15.0, 34.0, 0.0, P_TALISMAN, self.hold.flash, 0.8, c);
    }

    /// An arrow at the screen edge pointing to the shrine while it is off screen.
    pub(crate) fn draw_hold_marker(&mut self) {
        if !self.hold.active {
            return;
        }
        let (dx, dy) = (POINT.0 - self.cam.0, POINT.1 - self.cam.1);
        let (hw, hh) = (self.view_w - 36.0, self.view_h - 36.0);
        if dx.abs() < self.view_w + 20.0 && dy.abs() < self.view_h + 20.0 {
            return;
        }
        self.point(POINT.0, POINT.1);
        // Scale the direction so it ends on the inset screen rectangle.
        let k = (hw / dx.abs().max(0.001)).min(hh / dy.abs().max(0.001));
        let (mx, my) = (self.cam.0 + dx * k, self.cam.1 + dy * k);
        let rot = fmath::atan2(dy, dx) + TAU * 0.25;
        let c = [0.85, 0.72, 0.30, 1.0];
        self.put(mx, my, 11.0, 18.0, rot, P_TALISMAN, 0.0, 0.8, c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::TICK;

    const RADIUS: f32 = TUNE_DEFAULT[T_HOLD_RADIUS];
    const POINT_HP: f32 = TUNE_DEFAULT[T_HOLD_POINT_HP];
    const SHARE: u32 = TUNE_DEFAULT[T_HOLD_SHARE] as u32;

    /// A running `hold` game with the weapons idle and no spawning noise
    /// beyond what a test adds, at time 0.
    fn hold_game(seed: u32) -> Game {
        let mut g = Game::new(seed);
        g.start(M_HOLD);
        g
    }

    fn enemy(g: &mut Game, kind: u8, id: u32, x: f32, y: f32) {
        g.enemies.push(crate::game::Enemy {
            x,
            y,
            kx: 0.0,
            ky: 0.0,
            hp: 100000.0,
            kind,
            flash: 0.0,
            seed: 0.5,
            bead_cd: 0.0,
            tags: Default::default(),
            id,
            face: 1.0,
        });
    }

    /// Play with the player kept alive and upgrades picked.
    fn run(g: &mut Game, steps: u32, mut each: impl FnMut(&mut Game)) {
        for _ in 0..steps {
            g.player.hp = g.player.max_hp;
            each(g);
            g.tick();
            if g.choosing {
                g.choose(0);
            }
            if g.won || g.over {
                break;
            }
        }
    }

    #[test]
    fn meter_fills_only_inside_the_circle() {
        let mut g = hold_game(1);
        g.player.x = RADIUS + 50.0;
        run(&mut g, 120, |g| g.player.x = RADIUS + 50.0);
        assert_eq!(g.hold.held, 0.0);
        assert_eq!(g.progress(), 0.0);
        run(&mut g, 120, |g| g.player.x = 10.0);
        assert!((g.hold.held - 2.0).abs() < 0.05, "held {}", g.hold.held);
        // Leaving does not drain the meter.
        let held = g.hold.held;
        run(&mut g, 60, |g| g.player.x = RADIUS + 50.0);
        assert_eq!(g.hold.held, held);
        assert_eq!(g.progress(), held);
    }

    #[test]
    fn enemies_damage_the_shrine() {
        let mut g = hold_game(2);
        let id = SHARE; // an enemy sent after the shrine
        enemy(&mut g, K_BRUTE, id, 40.0, 0.0);
        g.player.x = 500.0; // out of the enemy's reach, so only the shrine is
        let before = g.hold.hp;
        g.tick();
        assert!(g.hold.hp < before, "hp {}", g.hold.hp);
    }

    #[test]
    fn only_a_share_of_the_horde_goes_for_the_shrine() {
        let g = hold_game(1);
        assert!(g.hold.target(SHARE).is_some());
        assert!(g.hold.target(SHARE + 1).is_none());
        // Other missions never redirect anybody.
        let mut other = Game::new(1);
        other.start(M_SURVIVE);
        assert!(other.hold.target(SHARE).is_none());
    }

    #[test]
    fn won_when_the_meter_is_full_and_not_before() {
        let mut g = hold_game(3);
        let goal = g.goal();
        assert!(goal > 0.0);
        g.hold.held = goal - 0.5;
        run(&mut g, 20, |_| {});
        assert!(!g.won);
        assert!((g.progress() - g.hold.held).abs() < 1e-6);
        run(&mut g, 60, |g| g.hold.hp = POINT_HP);
        assert!(g.won && !g.over && !g.hold.fell);
        assert!(g.events.iter().any(|e| e[0] == EV_WIN));
        g.update(TICK);
        assert_eq!(g.state[S_WON], 1.0);
        assert_eq!(g.state[S_PROGRESS], goal);
        assert_eq!(g.state[S_POINT_FELL], 0.0);
    }

    #[test]
    fn lost_without_dying_when_the_shrine_falls() {
        let mut g = hold_game(4);
        g.hold.hp = 0.5;
        enemy(&mut g, K_BRUTE, SHARE, 20.0, 0.0);
        run(&mut g, 600, |_| {});
        assert!(g.over && !g.won);
        assert!(g.hold.fell);
        assert!(g.player.hp > 0.0, "the player is alive");
        assert!(g.events.iter().any(|e| e[0] == EV_DEATH));
        g.update(TICK);
        assert_eq!(g.state[S_OVER], 1.0);
        assert_eq!(g.state[S_POINT_FELL], 1.0);
        assert_eq!(g.state[S_POINT_HP], 0.0);
        assert_eq!(g.state[S_POINT_MAX], POINT_HP);
    }

    #[test]
    fn lost_when_the_player_dies() {
        let mut g = hold_game(5);
        g.player.hp = 1.0;
        enemy(&mut g, K_BLOT, 1, 0.0, 0.0); // id 1: goes for the player
        g.tick();
        assert!(g.over && !g.won);
        assert!(!g.hold.fell);
        g.update(TICK);
        assert_eq!(g.state[S_POINT_FELL], 0.0);
    }

    #[test]
    fn dying_on_the_step_the_shrine_falls_counts_as_dying() {
        let mut g = hold_game(6);
        g.player.hp = 1.0;
        g.hold.hp = 0.1;
        enemy(&mut g, K_BLOT, 1, 0.0, 0.0);
        enemy(&mut g, K_BRUTE, SHARE, 0.0, 0.0);
        g.tick();
        assert!(g.over && !g.hold.fell);
    }

    #[test]
    fn other_missions_leave_the_shrine_alone() {
        let mut g = Game::new(1);
        g.start(M_SURVIVE);
        g.draw_hold_ground();
        g.draw_hold_marker();
        assert!(!g.hold.active);
        assert!(!g.hold.step(0.0, 0.0, TICK));
        assert_eq!(g.hold.held, 0.0);
    }

    #[test]
    fn marker_points_at_the_shrine_only_when_it_is_off_screen() {
        let mut g = hold_game(1);
        g.set_view(640.0, 360.0);
        g.present();
        g.player.x = 3000.0;
        g.present();
        // The marker is the last instance drawn and sits inside the view.
        let m = g.inst.last().unwrap();
        assert_eq!(m.part, P_TALISMAN);
        assert!(m.x < g.cam.0 && (m.x - g.cam.0).abs() <= 640.0 + 1.0);
    }

    // ------------------------------------------------------------ bot runs

    /// Walk a slow loop inside the circle, swinging the scythe all round.
    fn circle_bot(g: &mut Game, t: u32) {
        let a = t as f32 * 0.012;
        let (tx, ty) = (fmath::cos(a) * RADIUS * 0.7, fmath::sin(a) * RADIUS * 0.7);
        let (dx, dy) = (tx - g.player.x, ty - g.player.y);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        g.input = (dx / d, dy / d);
    }
    /// Step away from whatever enemy is closest; never go to the shrine.
    fn runner_bot(g: &mut Game, _t: u32) {
        let (px, py) = (g.player.x, g.player.y);
        let mut best = (f32::MAX, 0.0, 0.0);
        for e in &g.enemies {
            let (dx, dy) = (px - e.x, py - e.y);
            let d = dx * dx + dy * dy;
            if d < best.0 {
                best = (d, dx, dy);
            }
        }
        let d = best.0.sqrt().max(0.001);
        g.input = if best.0 < 400.0 * 400.0 { (best.1 / d, best.2 / d) } else { (0.0, 0.0) };
    }

    fn play(mission: usize, seed: u32, bot: fn(&mut Game, u32), minutes: u32, immortal: bool) -> Game {
        let mut g = Game::new(seed);
        // The bots pick from the mixed offer, not the decks.
        g.tune[T_DECKS] = 0.0;
        g.set_view(640.0, 360.0);
        g.start(mission);
        for t in 0..(60 * 60 * minutes) {
            if immortal {
                g.player.hp = g.player.max_hp;
            }
            bot(&mut g, t);
            g.tick();
            if g.choosing {
                g.choose(0);
            }
            if g.won || g.over {
                break;
            }
        }
        g
    }

    #[test]
    fn running_away_does_not_win() {
        for seed in 1..=3 {
            let g = play(M_HOLD, seed, runner_bot, 10, false);
            assert!(!g.won, "seed {seed}: a runner won at {:.0}s", g.time);
        }
    }

    #[test]
    fn holding_the_circle_wins_in_play() {
        let g = play(M_HOLD, 1, circle_bot, 10, true);
        assert!(g.won && !g.over, "no win by {:.0}s, shrine {:.0}", g.time, g.hold.hp);
        assert!(g.time >= g.goal() && g.time < 600.0);
    }

    #[test]
    #[ignore = "prints how a circle-holding bot fares; run with --ignored --nocapture"]
    fn report_circle_bot() {
        for seed in [1, 2, 4, 6, 8, 10, 12, 14] {
            let s = play(M_SURVIVE, seed, circle_bot, 6, false);
            println!("seed {seed} survive: won={} over={} time={:.0}s", s.won, s.over, s.time);
            let g = play(M_HOLD, seed, circle_bot, 12, false);
            let u = play(M_HOLD, seed, circle_bot, 12, true);
            println!(
                "seed {seed} unhurt: won={} over={} fell={} time={:.0}s shrine={:.0}",
                u.won, u.over, u.hold.fell, u.time, u.hold.hp
            );
            println!(
                "seed {seed}: won={} over={} fell={} time={:.0}s held={:.0}s shrine={:.0} player={:.0} kills={}",
                g.won, g.over, g.hold.fell, g.time, g.hold.held, g.hold.hp, g.player.hp, g.kills
            );
        }
    }

    #[test]
    fn tunes_change_the_shrine_and_the_circle() {
        let mut g = Game::new(1);
        g.set_tune(T_HOLD_GOAL, 60.0);
        g.set_tune(T_HOLD_RADIUS, 400.0);
        g.set_tune(T_HOLD_POINT_HP, 3000.0);
        g.set_tune(T_HOLD_SHARE, 2.0);
        g.start(M_HOLD);
        assert_eq!(g.goal(), 60.0);
        assert_eq!(g.hold.hp, 3000.0);
        assert!(g.hold.inside(350.0, 0.0) && !g.hold.inside(450.0, 0.0));
        assert!(g.hold.target(2).is_some() && g.hold.target(3).is_none());
        g.update(TICK);
        assert_eq!(g.state[S_POINT_HP], 3000.0);
        assert_eq!(g.state[S_POINT_MAX], 3000.0);
        assert_eq!(g.state[S_GOAL], 60.0);
        // Other missions keep the default shrine numbers.
        g.start(M_SURVIVE);
        g.update(TICK);
        assert_eq!(g.state[S_POINT_MAX], POINT_HP);
    }

    #[test]
    fn every_enemy_may_go_for_the_shrine() {
        let mut g = Game::new(1);
        g.set_tune(T_HOLD_SHARE, 1.0);
        g.set_tune(T_HOLD_RADIUS, 60.0);
        g.set_tune(T_HOLD_POINT_HP, 100.0);
        g.start(M_HOLD);
        assert!((1..20).all(|id| g.hold.target(id).is_some()));
        run(&mut g, 400, |_| {});
        assert!(g.hold.hp >= 0.0 && g.hold.hp.is_finite());
    }
}
