//! The `retrieval` mission: fetch relics from far-off spots and bring them
//! back to the start. Every relic carried slows the player down, so the
//! player picks between many short, fast trips and one long, slow one.
//!
//! This file holds the mission's own state and rules. `Game` keeps one
//! `Retrieval` and calls into it from `start`, `step`, `build`, `hash`,
//! `progress` and `mission_won`.

use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// The most relics a run can have: the top of the goal tune's range.
const MAX_ITEMS: usize = TUNE_MAX[T_RETRIEVAL_GOAL] as usize;
/// The drop-off is the start point.
pub const HOME: (f32, f32) = (0.0, 0.0);
/// A relic is picked up when the player is this close to it.
pub const PICKUP_R: f32 = 40.0;
/// Carried relics are delivered when the player is this close to the drop-off.
pub const DROP_R: f32 = 80.0;
/// How far inside the screen edge the pointing markers sit.
const EDGE: f32 = 44.0;

const GOLD: [f32; 4] = [0.92, 0.74, 0.25, 1.0];
const JADE: [f32; 4] = [0.25, 0.62, 0.48, 1.0];

#[derive(Clone, Copy, Default)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    /// Picked up and not yet delivered.
    pub carried: bool,
    pub delivered: bool,
}

/// What one step of walking did.
#[derive(Default)]
pub struct Moved {
    pub picked: u32,
    pub delivered: u32,
}

pub struct Retrieval {
    /// True once a retrieval run has started. Other missions leave this off.
    active: bool,
    /// Room for the most relics there can be; only the first `count` are in play.
    items: [Item; MAX_ITEMS],
    /// How many relics there are: the goal of the mission, `T_RETRIEVAL_GOAL`.
    count: usize,
    /// Fraction of walking speed lost per carried relic, `T_RETRIEVAL_SLOW`.
    slow: f32,
}

impl Default for Retrieval {
    /// Nothing started, holding the default number of relics, so the checksum
    /// of the other missions is the same as it always was.
    fn default() -> Self {
        Retrieval {
            active: false,
            items: [Item::default(); MAX_ITEMS],
            count: TUNE_DEFAULT[T_RETRIEVAL_GOAL] as usize,
            slow: TUNE_DEFAULT[T_RETRIEVAL_SLOW],
        }
    }
}

impl Retrieval {
    /// Lay the relics out for a new run, using the run's own generator and the
    /// run's tunes. Relics lie between the two distances from the drop-off (in
    /// either order), each in its own share of the circle.
    pub fn reset(&mut self, rng: &mut Rng, tune: &[f32; N_TUNES]) {
        self.active = true;
        self.count = (tune[T_RETRIEVAL_GOAL] as usize).clamp(1, MAX_ITEMS);
        self.slow = tune[T_RETRIEVAL_SLOW];
        let (d_min, d_max) = (
            tune[T_RETRIEVAL_DIST_MIN].min(tune[T_RETRIEVAL_DIST_MAX]),
            tune[T_RETRIEVAL_DIST_MIN].max(tune[T_RETRIEVAL_DIST_MAX]),
        );
        let turn = TAU / self.count as f32;
        for (k, item) in self.items.iter_mut().take(self.count).enumerate() {
            let a = (k as f32 + rng.range(0.1, 0.9)) * turn;
            let d = rng.range(d_min, d_max);
            *item =
                Item { x: HOME.0 + fmath::cos(a) * d, y: HOME.1 + fmath::sin(a) * d, carried: false, delivered: false };
        }
    }

    /// The relics of the run.
    pub fn items(&self) -> &[Item] {
        &self.items[..self.count]
    }

    pub fn carried(&self) -> u32 {
        self.items().iter().filter(|i| i.carried).count() as u32
    }

    pub fn delivered(&self) -> u32 {
        self.items().iter().filter(|i| i.delivered).count() as u32
    }

    /// Multiplier on walking speed. 1 when nothing is carried.
    pub fn speed_factor(&self) -> f32 {
        1.0 - self.slow * self.carried() as f32
    }

    pub fn won(&self) -> bool {
        self.active && self.delivered() as usize >= self.count
    }

    /// Pick up relics the player stands on, and deliver the carried ones when
    /// the player is at the drop-off.
    pub fn step(&mut self, px: f32, py: f32) -> Moved {
        let mut moved = Moved::default();
        if !self.active {
            return moved;
        }
        let (hx, hy) = (px - HOME.0, py - HOME.1);
        let home = hx * hx + hy * hy < DROP_R * DROP_R;
        for item in self.items[..self.count].iter_mut() {
            if item.delivered {
                continue;
            }
            if item.carried {
                if home {
                    item.carried = false;
                    item.delivered = true;
                    moved.delivered += 1;
                }
                continue;
            }
            let (dx, dy) = (px - item.x, py - item.y);
            if dx * dx + dy * dy < PICKUP_R * PICKUP_R {
                item.carried = true;
                moved.picked += 1;
            }
        }
        moved
    }

    /// Feed the mission state into the run's checksum.
    pub fn mix(&self, mix: &mut impl FnMut(u32)) {
        mix(self.active as u32);
        for i in self.items() {
            mix(i.x.to_bits());
            mix(i.y.to_bits());
            mix(i.carried as u32 | (i.delivered as u32) << 1);
        }
    }

    /// The drop-off, relics lying about and delivered relics. Drawn on the ground, below the characters.
    pub fn draw_ground(&self, c: &mut Canvas, anim: f32) {
        if !self.active {
            return;
        }
        c.put(HOME, (DROP_R, DROP_R), 0.0, P_RING, [JADE[0], JADE[1], JADE[2], 0.7], 0.2);
        c.put(HOME, (DROP_R * 0.5, DROP_R * 0.5), 0.0, P_RING, [JADE[0], JADE[1], JADE[2], 0.45], 0.6);
        let shadow = [INK[0], INK[1], INK[2], 0.2];
        for (k, item) in self.items().iter().enumerate() {
            if item.delivered {
                // Delivered relics stack up beside the drop-off marker.
                let x = HOME.0 + (k as f32 - (self.count as f32 - 1.0) * 0.5) * 22.0;
                c.put((x, HOME.1 - 8.0), (7.0, 9.0), 0.0, P_GEM, GOLD, k as f32 * 0.31);
            } else if !item.carried {
                let pose = (((anim * 3.0 + k as f32 * 1.7).floor() as i32) & 1) as f32;
                c.put((item.x, item.y), (16.0, 6.0), 0.0, P_GROUND_SHADOW, shadow, 0.0);
                c.put((item.x, item.y), (PICKUP_R, PICKUP_R), 0.0, P_RING, [GOLD[0], GOLD[1], GOLD[2], 0.5], 0.4);
                c.put(
                    (item.x, item.y - 20.0 - pose * 2.0),
                    (13.0, 17.0),
                    pose * 0.2 - 0.1,
                    P_GEM,
                    GOLD,
                    k as f32 * 0.31,
                );
            }
        }
    }

    /// Carried relics above the player and the edge markers. Drawn on top.
    pub fn draw_top(&self, c: &mut Canvas, player: (f32, f32), anim: f32) {
        if !self.active {
            return;
        }
        let carried = self.carried();
        for k in 0..carried {
            let x = player.0 + (k as f32 - (carried as f32 - 1.0) * 0.5) * 18.0;
            let bob = fmath::sin(anim * 5.0 + k as f32 * 2.0) * 2.0;
            c.put((x, player.1 - 96.0 + bob), (8.0, 10.0), 0.0, P_GEM, GOLD, k as f32 * 0.31);
        }
        for item in self.items().iter().filter(|i| !i.carried && !i.delivered) {
            c.marker((item.x, item.y), GOLD);
        }
        if carried > 0 {
            c.marker(HOME, JADE);
        }
    }
}

/// Where the mission draws to: the game's instance list and the camera view.
pub struct Canvas<'a> {
    pub inst: &'a mut Vec<Inst>,
    /// The game's list of what the edge markers point at.
    pub points: &'a mut Vec<[f32; 4]>,
    pub cam: (f32, f32),
    /// Half extents of the visible area in world units.
    pub view: (f32, f32),
}

impl Canvas<'_> {
    fn put(&mut self, at: (f32, f32), half: (f32, f32), rot: f32, part: f32, color: [f32; 4], seed: f32) {
        let m = half.0.abs().max(half.1) + 24.0;
        if self.inst.len() >= MAX_INST
            || (at.0 - self.cam.0).abs() > self.view.0 + m
            || (at.1 - self.cam.1).abs() > self.view.1 + m
        {
            return;
        }
        self.inst.push(Inst { x: at.0, y: at.1, hw: half.0, hh: half.1, rot, part, flash: 0.0, seed, color });
    }

    /// A paper tag at the screen edge pointing at a spot that is off screen.
    fn marker(&mut self, target: (f32, f32), color: [f32; 4]) {
        let (dx, dy) = (target.0 - self.cam.0, target.1 - self.cam.1);
        let (ex, ey) = ((self.view.0 - EDGE).max(1.0), (self.view.1 - EDGE).max(1.0));
        if dx.abs() <= ex && dy.abs() <= ey {
            return;
        }
        self.points.push([target.0, target.1, 0.0, 0.0]);
        let s = (ex / dx.abs().max(0.001)).min(ey / dy.abs().max(0.001));
        let at = (self.cam.0 + dx * s, self.cam.1 + dy * s);
        let rot = fmath::atan2(dy, dx) + TAU * 0.25;
        self.put(at, (24.0, 24.0), 0.0, P_RING, [color[0], color[1], color[2], 0.55], 0.5);
        self.put(at, (9.0, 16.0), rot, P_TALISMAN, color, 0.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Enemy, Game, TICK};

    const ITEMS: usize = TUNE_DEFAULT[T_RETRIEVAL_GOAL] as usize;
    const SLOW_PER_ITEM: f32 = TUNE_DEFAULT[T_RETRIEVAL_SLOW];
    const MIN_DIST: f32 = TUNE_DEFAULT[T_RETRIEVAL_DIST_MIN];
    const MAX_DIST: f32 = TUNE_DEFAULT[T_RETRIEVAL_DIST_MAX];

    /// A retrieval run with the weapons parked, as far from any trouble as the
    /// first steps are.
    fn run(seed: u32) -> Game {
        let mut g = Game::new(seed);
        g.start(M_RETRIEVAL);
        g
    }

    /// Stand the player at a spot with full health and take one step.
    fn tick_at(g: &mut Game, x: f32, y: f32) {
        g.player.x = x;
        g.player.y = y;
        g.player.hp = g.player.max_hp;
        g.tick();
    }

    fn carry_all(g: &mut Game) {
        for k in 0..ITEMS {
            let (x, y) = (g.retrieval.items[k].x, g.retrieval.items[k].y);
            tick_at(g, x, y);
        }
    }

    #[test]
    fn the_mission_asks_for_every_item() {
        let g = run(3);
        assert_eq!(g.goal(), ITEMS as f32);
        assert_eq!(g.progress(), 0.0);
    }

    #[test]
    fn an_item_is_picked_up_only_by_standing_on_it() {
        let mut g = run(3);
        let (x, y) = (g.retrieval.items[0].x, g.retrieval.items[0].y);
        tick_at(&mut g, x + PICKUP_R + 5.0, y);
        assert_eq!(g.retrieval.carried(), 0);
        // Walking past the home and the other items picks nothing up either.
        tick_at(&mut g, 1000.0, 1000.0);
        assert_eq!(g.retrieval.carried(), 0);
        g.events.clear();
        tick_at(&mut g, x + PICKUP_R - 5.0, y);
        assert_eq!(g.retrieval.carried(), 1);
        assert!(g.retrieval.items[0].carried);
        assert!(g.events.iter().any(|e| e[0] == EV_PICKUP));
        // It is picked up once; standing there again changes nothing.
        tick_at(&mut g, x, y);
        assert_eq!(g.retrieval.carried(), 1);
    }

    #[test]
    fn each_carried_item_slows_the_player() {
        let full = 190.0 * TICK;
        for n in 0..=ITEMS {
            let mut g = run(3);
            for k in 0..n {
                let (x, y) = (g.retrieval.items[k].x, g.retrieval.items[k].y);
                tick_at(&mut g, x, y);
            }
            assert_eq!(g.retrieval.carried() as usize, n);
            // Walk right from a quiet spot, away from home and the relics.
            g.player.x = 20_000.0;
            g.player.y = 20_000.0;
            g.input = (1.0, 0.0);
            g.tick();
            let moved = g.player.x - 20_000.0;
            let want = full * (1.0 - SLOW_PER_ITEM * n as f32);
            assert!((moved - want).abs() < 0.01, "{n} carried: moved {moved} want {want}");
        }
    }

    #[test]
    fn items_are_delivered_only_at_the_drop_off() {
        let mut g = run(3);
        carry_all(&mut g);
        assert_eq!(g.retrieval.carried() as usize, ITEMS);
        // Next to the drop-off but outside its reach, and out in the field.
        tick_at(&mut g, DROP_R + 5.0, 0.0);
        tick_at(&mut g, 2000.0, 0.0);
        assert_eq!(g.retrieval.delivered(), 0);
        assert_eq!(g.progress(), 0.0);
        g.events.clear();
        tick_at(&mut g, DROP_R - 5.0, 0.0);
        assert_eq!(g.retrieval.delivered() as usize, ITEMS);
        assert_eq!(g.retrieval.carried(), 0);
        assert_eq!(g.retrieval.speed_factor(), 1.0);
    }

    #[test]
    fn the_mission_is_won_when_the_last_item_is_delivered() {
        let mut g = run(3);
        for k in 0..ITEMS {
            let (x, y) = (g.retrieval.items[k].x, g.retrieval.items[k].y);
            tick_at(&mut g, x, y);
            assert!(!g.won);
            tick_at(&mut g, 0.0, 0.0);
            assert_eq!(g.progress(), (k + 1) as f32);
            assert_eq!(g.won, k + 1 == ITEMS, "after delivering {}", k + 1);
        }
        assert!(g.won && !g.over);
        assert!(g.events.iter().any(|e| e[0] == EV_WIN));
        g.update(TICK);
        assert_eq!(g.state[S_WON], 1.0);
        assert_eq!(g.state[S_MISSION], M_RETRIEVAL as f32);
        assert_eq!(g.state[S_GOAL], ITEMS as f32);
        assert_eq!(g.state[S_PROGRESS], ITEMS as f32);
        assert_eq!(g.state[S_CARRIED], 0.0);
    }

    #[test]
    fn the_hud_state_counts_what_is_carried() {
        let mut g = run(3);
        let (x, y) = (g.retrieval.items[1].x, g.retrieval.items[1].y);
        tick_at(&mut g, x, y);
        g.update(0.0);
        assert_eq!(g.state[S_CARRIED], 1.0);
        assert_eq!(g.state[S_PROGRESS], 0.0);
    }

    #[test]
    fn the_mission_is_lost_on_death() {
        let mut g = run(3);
        carry_all(&mut g);
        // A brute on top of the player, with one hit point left.
        g.player.hp = 1.0;
        g.enemies.push(Enemy {
            x: g.player.x,
            y: g.player.y,
            kx: 0.0,
            ky: 0.0,
            hp: 1000.0,
            kind: K_BRUTE,
            flash: 0.0,
            seed: 0.0,
            bead_cd: 0.0,
            tags: Default::default(),
            id: 9999,
            face: 1.0,
        });
        g.tick();
        assert!(g.over && !g.won);
        assert_eq!(g.progress(), 0.0);
    }

    #[test]
    fn other_missions_have_no_items() {
        for mission in [M_ENDLESS, M_SURVIVE] {
            let mut g = Game::new(3);
            g.start(mission);
            assert_eq!(g.retrieval.carried(), 0);
            tick_at(&mut g, 0.0, 0.0);
            assert_eq!(g.retrieval.delivered(), 0);
            assert!(!g.retrieval.won());
            assert_eq!(g.retrieval.speed_factor(), 1.0);
            g.update(0.0);
            assert_eq!(g.state[S_CARRIED], 0.0);
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_layout() {
        let layout =
            |seed| run(seed).retrieval.items().iter().map(|i| (i.x.to_bits(), i.y.to_bits())).collect::<Vec<_>>();
        assert_eq!(layout(21), layout(21));
        assert_ne!(layout(21), layout(22));
        for seed in 1..50 {
            let g = run(seed);
            for (k, a) in g.retrieval.items().iter().enumerate() {
                let d = (a.x * a.x + a.y * a.y).sqrt();
                assert!((MIN_DIST - 1.0..=MAX_DIST + 1.0).contains(&d), "seed {seed}: item {k} at {d}");
                for b in &g.retrieval.items()[k + 1..] {
                    let far = ((a.x - b.x).powi(2) + (a.y - b.y).powi(2)).sqrt();
                    assert!(far > 1500.0, "seed {seed}: two items {far} apart");
                }
            }
        }
    }

    #[test]
    fn the_run_is_reproducible() {
        let play = || {
            let mut g = run(8);
            g.set_view(640.0, 360.0);
            for t in 0..600 {
                g.input = if t < 300 { (1.0, 0.3) } else { (-0.2, 1.0) };
                g.update(TICK);
            }
            g.hash()
        };
        assert_eq!(play(), play());
    }

    #[test]
    fn markers_point_at_off_screen_items_and_home_when_carrying() {
        let mut g = run(3);
        g.set_view(640.0, 360.0);
        g.update(TICK);
        let gold = |g: &Game| g.inst.iter().filter(|i| i.part == P_TALISMAN && i.color == GOLD).count();
        let jade = |g: &Game| g.inst.iter().filter(|i| i.part == P_TALISMAN && i.color == JADE).count();
        // Home is on screen at the start and the three items are far away.
        assert_eq!(gold(&g), ITEMS);
        assert_eq!(jade(&g), 0);
        let (x, y) = (g.retrieval.items[0].x, g.retrieval.items[0].y);
        tick_at(&mut g, x, y);
        g.update(0.0);
        assert_eq!(gold(&g), ITEMS - 1);
        assert_eq!(jade(&g), 1);
        for i in g.inst.iter().filter(|i| i.part == P_TALISMAN) {
            assert!((i.x - g.cam.0).abs() <= 640.0 && (i.y - g.cam.1).abs() <= 360.0, "marker off screen");
        }
    }

    // ---------------------------------------------------------------- bots

    /// Steer one step of a bot that ignores the enemies and walks to relics
    /// and home. `chain` collects every relic before going home.
    fn walker_input(g: &Game, chain: bool) -> (f32, f32) {
        let (px, py) = (g.player.x, g.player.y);
        let nearest = g.retrieval.items().iter().filter(|i| !i.carried && !i.delivered).min_by(|a, b| {
            let da = (a.x - px).powi(2) + (a.y - py).powi(2);
            let db = (b.x - px).powi(2) + (b.y - py).powi(2);
            da.partial_cmp(&db).unwrap()
        });
        let target = match nearest {
            Some(item) if chain || g.retrieval.carried() == 0 => (item.x, item.y),
            _ => HOME,
        };
        let (dx, dy) = (target.0 - px, target.1 - py);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        (dx / d, dy / d)
    }

    /// Like `walker_input`, but bends away from enemies close by.
    fn dodger_input(g: &Game, chain: bool) -> (f32, f32) {
        let (wx, wy) = walker_input(g, chain);
        let (px, py) = (g.player.x, g.player.y);
        let (mut fx, mut fy) = (0.0, 0.0);
        for e in &g.enemies {
            let (dx, dy) = (px - e.x, py - e.y);
            let d2 = dx * dx + dy * dy;
            if d2 < 160.0 * 160.0 {
                let d = d2.sqrt().max(1.0);
                let w = (160.0 - d) / 160.0;
                fx += dx / d * w;
                fy += dy / d * w;
            }
        }
        (wx + fx * 1.5, wy + fy * 1.5)
    }

    /// Play up to `minutes`; returns (won, seconds survived).
    fn play_bot(seed: u32, minutes: u32, steer: impl Fn(&Game) -> (f32, f32)) -> (bool, f32) {
        let mut g = run(seed);
        g.set_view(640.0, 360.0);
        for t in 0..(60 * 60 * minutes) {
            if t % 6 == 0 {
                g.input = steer(&g);
            }
            g.tick();
            if g.choosing {
                g.choose(0);
            }
            if g.won || g.over {
                break;
            }
        }
        (g.won, g.time)
    }

    #[test]
    fn a_bot_that_only_flees_does_not_win() {
        for seed in 1..=3 {
            let (won, secs) = play_bot(seed, 10, |g| {
                let (px, py) = (g.player.x, g.player.y);
                let (mut fx, mut fy) = (0.0, 0.0);
                for e in &g.enemies {
                    let (dx, dy) = (px - e.x, py - e.y);
                    let d2 = dx * dx + dy * dy;
                    if d2 < 250.0 * 250.0 {
                        let d = d2.sqrt().max(1.0);
                        fx += dx / d;
                        fy += dy / d;
                    }
                }
                (fx, fy)
            });
            assert!(!won, "seed {seed}: the fleeing bot won at {secs}s");
        }
    }

    /// Exploratory: how a walk-to-the-relic bot fares. Run with
    /// `cargo test --release walker_bot_report -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn walker_bot_report() {
        for chain in [false, true] {
            for seed in 1..=8 {
                let (won, secs) = play_bot(seed, 10, |g| walker_input(g, chain));
                println!("chain={chain} seed={seed}: {} at {secs:.0}s", if won { "won" } else { "lost or timed out" });
                let (won, secs) = play_bot(seed, 10, |g| dodger_input(g, chain));
                println!(
                    "dodger chain={chain} seed={seed}: {} at {secs:.0}s",
                    if won { "won" } else { "lost or timed out" }
                );
            }
        }
    }

    #[test]
    fn tunes_change_the_relics() {
        let mut g = Game::new(3);
        g.set_tune(T_RETRIEVAL_GOAL, 6.0);
        g.set_tune(T_RETRIEVAL_DIST_MIN, 1000.0);
        g.set_tune(T_RETRIEVAL_DIST_MAX, 1500.0);
        g.set_tune(T_RETRIEVAL_SLOW, 0.2);
        g.start(M_RETRIEVAL);
        assert_eq!(g.goal(), 6.0);
        assert_eq!(g.retrieval.items().len(), 6);
        for i in g.retrieval.items() {
            let d = i.x.hypot(i.y);
            assert!((999.0..=1501.0).contains(&d), "relic at {d}");
        }
        carry_six(&mut g);
        assert_eq!(g.retrieval.carried(), 6);
        assert!((g.retrieval.speed_factor() - (1.0 - 0.2 * 6.0)).abs() < 1.0e-6);
    }

    fn carry_six(g: &mut Game) {
        for k in 0..6 {
            let (x, y) = (g.retrieval.items()[k].x, g.retrieval.items()[k].y);
            tick_at(g, x, y);
        }
    }

    #[test]
    fn degenerate_tunes_do_not_break_a_run() {
        let mut g = Game::new(3);
        g.set_tune(T_RETRIEVAL_GOAL, 1.0);
        g.set_tune(T_RETRIEVAL_DIST_MIN, 12000.0);
        g.set_tune(T_RETRIEVAL_DIST_MAX, 500.0);
        g.set_tune(T_RETRIEVAL_SLOW, 0.0);
        g.start(M_RETRIEVAL);
        let i = g.retrieval.items()[0];
        assert!((499.0..=12001.0).contains(&i.x.hypot(i.y)));
        for _ in 0..400 {
            g.player.hp = g.player.max_hp;
            g.tick();
        }
        assert_eq!(g.retrieval.speed_factor(), 1.0);
        let (x, y) = (i.x, i.y);
        tick_at(&mut g, x, y);
        tick_at(&mut g, 0.0, 0.0);
        assert!(g.won);
    }
}
