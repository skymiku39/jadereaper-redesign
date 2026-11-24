//! The `escort` mission: walk a villager from where the run starts to a far
//! shrine.
//!
//! The villager follows a route drawn from the run's seed. It only walks while
//! the player is close, and waits otherwise. Enemies chase whichever of the two
//! is nearer and hurt the villager on touch. The mission is won when the route
//! is walked to its end, and lost when the player dies or the villager does.
//!
//! This file is a child module of `game`, so it can use the private parts of
//! `Game` (events, effects, the camera). `game.rs` only calls the hooks below.

use super::*;

/// Legs of the route. Each is a straight walk of `T_ESCORT_LEG_LEN`, turning by
/// a random angle at its start, so the route is `LEGS` times that long. That
/// product is the goal of the mission; `Game::goal` computes it.
pub const LEGS: usize = 7;
/// Largest turn between two legs, in radians.
const TURN: f32 = 0.9;
/// Share of an enemy's damage the villager takes. It is hurt less than the player,
/// who can fight back and dodge.
const HURT_SCALE: f32 = 0.2;
/// Contact radius, added to an enemy's own, like the player's 10.
const BODY: f32 = 10.0;
/// Seconds of grace after a hit, like the player's.
const HURT_GRACE: f32 = 0.8;
/// Where the villager stands next to the player at the start.
const START_OFFSET: (f32, f32) = (48.0, 6.0);
/// The shrine's marker shows at the screen edge from this leg on.
const SHRINE_MARK_LEG: usize = LEGS - 1;

pub struct Escort {
    /// True during a run of the escort mission.
    pub active: bool,
    pub x: f32,
    pub y: f32,
    pub hp: f32,
    /// Health the villager started with, `T_ESCORT_HP`.
    pub max_hp: f32,
    /// Length of one leg of the route, `T_ESCORT_LEG_LEN`.
    leg_len: f32,
    /// Route length covered so far.
    pub walked: f32,
    /// True when the villager's death ended the run.
    pub fallen: bool,
    /// True while the player is too far and the villager stands still.
    pub waiting: bool,
    face: f32,
    walk: f32,
    iframe: f32,
    /// Corners of the route: the start, then the end of each leg. The last is the shrine.
    pub route: [(f32, f32); LEGS + 1],
    dir: [(f32, f32); LEGS],
}

impl Escort {
    pub fn new() -> Escort {
        Escort {
            active: false,
            x: 0.0,
            y: 0.0,
            hp: 0.0,
            max_hp: TUNE_DEFAULT[T_ESCORT_HP],
            leg_len: TUNE_DEFAULT[T_ESCORT_LEG_LEN],
            walked: 0.0,
            fallen: false,
            waiting: false,
            face: 1.0,
            walk: 0.0,
            iframe: 0.0,
            route: [(0.0, 0.0); LEGS + 1],
            dir: [(1.0, 0.0); LEGS],
        }
    }

    /// Put the villager beside the player and draw the route from the run's generator.
    fn begin(&mut self, rng: &mut Rng, px: f32, py: f32, tune: &[f32; N_TUNES]) {
        *self = Escort::new();
        self.active = true;
        self.max_hp = tune[T_ESCORT_HP];
        self.leg_len = tune[T_ESCORT_LEG_LEN];
        self.hp = self.max_hp;
        self.x = px + START_OFFSET.0;
        self.y = py + START_OFFSET.1;
        self.route[0] = (self.x, self.y);
        let mut heading = rng.range(0.0, TAU);
        for leg in 0..LEGS {
            heading += rng.range(-TURN, TURN);
            let d = (fmath::cos(heading), fmath::sin(heading));
            self.dir[leg] = d;
            let (x, y) = self.route[leg];
            self.route[leg + 1] = (x + d.0 * self.leg_len, y + d.1 * self.leg_len);
        }
    }

    /// The shrine.
    pub fn shrine(&self) -> (f32, f32) {
        self.route[LEGS]
    }

    /// The leg the villager is on.
    fn leg(&self) -> usize {
        ((self.walked / self.leg_len) as usize).min(LEGS - 1)
    }

    /// Put the villager where `walked` says along the route.
    fn place(&mut self) {
        let leg = self.leg();
        let t = (self.walked - leg as f32 * self.leg_len).min(self.leg_len);
        let (x, y) = self.route[leg];
        self.x = x + self.dir[leg].0 * t;
        self.y = y + self.dir[leg].1 * t;
        if self.dir[leg].0.abs() > 0.1 {
            self.face = self.dir[leg].0.signum();
        }
    }

    /// The point an enemy at `(ex, ey)` walks towards: the villager if it is
    /// alive and nearer than the player at `(px, py)`, else the player.
    pub fn target(&self, ex: f32, ey: f32, px: f32, py: f32) -> (f32, f32) {
        if self.active && !self.fallen {
            let to_escort = (self.x - ex) * (self.x - ex) + (self.y - ey) * (self.y - ey);
            let to_player = (px - ex) * (px - ex) + (py - ey) * (py - ey);
            if to_escort < to_player {
                return (self.x, self.y);
            }
        }
        (px, py)
    }

    /// Mix what decides how the run continues into the state hash.
    pub fn mix_into(&self, mix: &mut dyn FnMut(u32)) {
        for v in [self.x, self.y, self.hp, self.walked, self.iframe] {
            mix(v.to_bits());
        }
        mix(self.fallen as u32);
    }
}

impl Default for Escort {
    fn default() -> Self {
        Escort::new()
    }
}

impl Game {
    /// Called by `start`. Sets up the villager and its route for this mission only.
    pub(super) fn escort_begin(&mut self) {
        if self.mission != M_ESCORT {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        self.escort.begin(&mut self.rng, px, py, &self.tune);
    }

    /// One step of the villager: walk, take hits from touching enemies, fall.
    pub(super) fn escort_step(&mut self, dt: f32) {
        if !self.escort.active || self.escort.fallen {
            return;
        }
        let goal = self.goal();
        // Walking speed in world units per second; the player moves at 190 or more.
        let speed = self.tune[T_ESCORT_SPEED];
        // The villager only walks while the player is closer than this.
        let follow = self.tune[T_ESCORT_FOLLOW];
        let (px, py) = (self.player.x, self.player.y);
        let e = &mut self.escort;
        let (dx, dy) = (px - e.x, py - e.y);
        e.waiting = dx * dx + dy * dy > follow * follow;
        e.iframe = (e.iframe - dt).max(0.0);
        if !e.waiting {
            e.walked = (e.walked + speed * dt).min(goal);
            e.walk += dt;
            e.place();
        }
        let (x, y) = (e.x, e.y);

        let mut hit: Option<f32> = None;
        for en in &self.enemies {
            let k = &KINDS[en.kind as usize];
            let r = k.r + BODY;
            let (ox, oy) = (en.x - x, en.y - y);
            if ox * ox + oy * oy < r * r {
                hit = Some(hit.map_or(k.dmg, |h: f32| h.max(k.dmg)));
            }
        }
        let Some(dmg) = hit else { return };
        if self.escort.iframe > 0.0 {
            return;
        }
        self.escort.iframe = HURT_GRACE;
        self.escort.hp -= dmg * HURT_SCALE;
        self.event(EV_HURT, x, y, dmg);
        for _ in 0..6 {
            let a = self.rng.range(0.0, TAU);
            let s = self.rng.range(80.0, 220.0);
            let seed = self.rng.f();
            self.add_fx(Fx {
                x,
                y: y - 30.0,
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
                color: [0.86, 0.66, 0.30, 1.0],
                seed,
                top: true,
            });
        }
        if self.escort.hp <= 0.0 {
            self.escort.hp = 0.0;
            // The player dying on the same step counts as the player dying.
            if !self.over {
                self.escort.fallen = true;
                self.over = true;
                self.event(EV_DEATH, x, y, 0.0);
            }
        }
    }

    /// The villager's numbers for the HUD.
    pub(super) fn escort_state(&mut self) {
        let e = &self.escort;
        self.state[S_ESCORT_HP] = e.hp;
        self.state[S_ESCORT_MAX_HP] = if e.active { e.max_hp } else { 0.0 };
        self.state[S_ESCORT_FELL] = e.fallen as u8 as f32;
    }

    /// The villager: a smaller paper figure in warm colours, with a heart over
    /// its head so it never reads as an enemy.
    pub(super) fn draw_escort(&mut self) {
        let e = &self.escort;
        let (x, y, f, waiting) = (e.x, e.y, e.face, e.waiting);
        let (tilt, bob) = if !waiting {
            if ((e.walk * 8.0).floor() as i32) & 1 == 0 {
                (0.09, 0.0)
            } else {
                (-0.09, -2.5)
            }
        } else if ((self.anim * 2.0).floor() as i32) & 1 == 0 {
            (0.0, 0.0)
        } else {
            (0.0, -1.5)
        };
        let a = if e.iframe > 0.0 && ((e.iframe * 20.0) as i32) & 1 == 1 { 0.4 } else { 1.0 };
        let flash = if e.iframe > HURT_GRACE - 0.15 { 1.0 } else { 0.0 };

        self.put(x, y, 16.0, 6.0, 0.0, P_GROUND_SHADOW, 0.0, 0.0, [INK[0], INK[1], INK[2], 0.22]);
        self.put(x, y - 21.0 + bob, 15.0 * f, 21.0, tilt, P_PLAYER_BODY, flash, 0.61, [0.86, 0.66, 0.30, a]);
        let top_x = x + 21.0 * fmath::sin(tilt);
        self.put(
            top_x + f * 2.0,
            y - 48.0 + bob * 1.5,
            12.0 * f,
            12.0,
            -tilt * 0.6,
            P_PLAYER_HEAD,
            flash,
            0.17,
            [0.95, 0.82, 0.58, a],
        );
        self.put(x, y - 72.0 + bob, 7.0, 7.0, 0.0, P_HEART, 0.0, 0.3, [0.78, 0.27, 0.18, 1.0]);
        if waiting {
            // A slow pulse on the ground: the villager is waiting for you.
            let pulse = ((self.anim * 3.0).floor() as i32 & 1) as f32;
            self.put(x, y, 26.0 + pulse * 5.0, 9.0 + pulse * 2.0, 0.0, P_RING, 0.0, 0.2, [INK[0], INK[1], INK[2], 0.5]);
        }
    }

    /// The shrine: a gold ring on the ground and a tall banner.
    pub(super) fn draw_escort_goal(&mut self) {
        if !self.escort.active {
            return;
        }
        let (x, y) = self.escort.shrine();
        let pulse = ((self.anim * 2.0).floor() as i32 & 1) as f32;
        let gold = [0.89, 0.72, 0.30, 1.0];
        self.put(x, y, 96.0, 36.0, 0.0, P_GROUND_SHADOW, 0.0, 0.0, [INK[0], INK[1], INK[2], 0.2]);
        self.put(x, y, 92.0 + pulse * 6.0, 40.0 + pulse * 3.0, 0.0, P_RING, 0.0, 0.4, [gold[0], gold[1], gold[2], 0.9]);
        self.put(x, y, 56.0, 24.0, 0.0, P_RING, 0.0, 0.8, [gold[0], gold[1], gold[2], 0.6]);
        self.put(x, y - 70.0, 20.0, 66.0, 0.0, P_TALISMAN, 0.0, 0.5, gold);
        self.put(x, y - 150.0, 11.0, 11.0, 0.0, P_HEART, 0.0, 0.5, [0.78, 0.27, 0.18, 1.0]);
    }

    /// Edge-of-screen markers: one for the villager while it is off screen, and
    /// one for the shrine on the last leg of the route.
    pub(super) fn draw_escort_markers(&mut self) {
        if !self.escort.active || self.escort.fallen {
            return;
        }
        let (x, y) = (self.escort.x, self.escort.y - 30.0);
        self.edge_marker(x, y, [0.86, 0.66, 0.30, 1.0], true);
        if self.escort.leg() >= SHRINE_MARK_LEG {
            let (sx, sy) = self.escort.shrine();
            self.edge_marker(sx, sy - 60.0, [0.89, 0.72, 0.30, 0.8], false);
        }
    }

    /// A ring with a pointer, kept on the screen edge, for a point that is off screen.
    fn edge_marker(&mut self, tx: f32, ty: f32, color: [f32; 4], heart: bool) {
        // The top is kept clear of the HUD.
        let (vw, vh) = (self.view_w, self.view_h);
        let (rx, ry) = (tx - self.cam.0, ty - self.cam.1);
        if rx.abs() < vw - 20.0 && ry.abs() < vh - 20.0 {
            return;
        }
        self.point(tx, ty);
        let ky = if ry < 0.0 { vh - 96.0 } else { vh - 44.0 };
        let k = ((vw - 44.0) / rx.abs().max(0.001)).min(ky / ry.abs().max(0.001)).min(1.0);
        let (mx, my) = (self.cam.0 + rx * k, self.cam.1 + ry * k);
        let len = (rx * rx + ry * ry).sqrt().max(0.001);
        let (ux, uy) = (rx / len, ry / len);
        self.put(mx, my, 17.0, 17.0, 0.0, P_RING, 0.0, 0.3, color);
        if heart {
            self.put(mx, my, 8.0, 8.0, 0.0, P_HEART, 0.0, 0.3, [0.78, 0.27, 0.18, 1.0]);
        }
        let rot = fmath::atan2(uy, ux) + TAU * 0.25;
        self.put(mx + ux * 28.0, my + uy * 28.0, 6.0, 11.0, rot, P_TALISMAN, 0.0, 0.5, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECONDS: u32 = 60;
    const LEG_LEN: f32 = TUNE_DEFAULT[T_ESCORT_LEG_LEN];
    const SPEED: f32 = TUNE_DEFAULT[T_ESCORT_SPEED];
    const MAX_HP: f32 = TUNE_DEFAULT[T_ESCORT_HP];

    fn escort_game(seed: u32) -> Game {
        let mut game = Game::new(seed);
        // The bots pick from the mixed offer, not the decks.
        game.tune[T_DECKS] = 0.0;
        game.start(M_ESCORT);
        game
    }

    fn enemy_at(x: f32, y: f32) -> Enemy {
        Enemy {
            x,
            y,
            kx: 0.0,
            ky: 0.0,
            hp: 14.0,
            kind: K_BLOT,
            flash: 0.0,
            seed: 0.5,
            bead_cd: 0.0,
            tags: Default::default(),
            id: 9999,
            face: 1.0,
        }
    }

    /// Move the player onto the villager's place on the route.
    fn join(game: &mut Game) {
        game.escort.place();
        game.player.x = game.escort.x - START_OFFSET.0;
        game.player.y = game.escort.y - START_OFFSET.1;
    }

    fn unhurt_tick(game: &mut Game) {
        game.player.hp = game.player.max_hp;
        game.escort.hp = MAX_HP;
        game.events.clear();
        game.tick();
        if game.choosing {
            game.choose(0);
        }
    }

    #[test]
    fn the_route_adds_up_to_the_mission_goal() {
        assert_eq!(LEGS as f32 * LEG_LEN, MISSION_GOAL[M_ESCORT]);
        let game = escort_game(1);
        assert_eq!(game.goal(), MISSION_GOAL[M_ESCORT]);
        for leg in 0..LEGS {
            let (a, b) = (game.escort.route[leg], game.escort.route[leg + 1]);
            let len = ((b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1)).sqrt();
            assert!((len - LEG_LEN).abs() < 1.0, "leg {leg} is {len} long");
        }
    }

    #[test]
    fn the_same_seed_gives_the_same_route() {
        let (a, b, c) = (escort_game(7), escort_game(7), escort_game(8));
        assert_eq!(a.escort.route, b.escort.route);
        assert_ne!(a.escort.route, c.escort.route);
        // It starts beside the player.
        assert_eq!((a.escort.x, a.escort.y), (START_OFFSET.0, START_OFFSET.1));
        assert_eq!(a.escort.route[0], (a.escort.x, a.escort.y));
    }

    #[test]
    fn other_missions_have_no_escort() {
        for mission in [M_ENDLESS, M_SURVIVE, M_KILLS] {
            let mut game = Game::new(3);
            game.start(mission);
            assert!(!game.escort.active);
            game.tick();
            assert_eq!(game.escort.target(10.0, 10.0, 500.0, 500.0), (500.0, 500.0));
            assert_eq!(game.escort_state_after_update(), (0.0, 0.0, 0.0));
        }
    }

    impl Game {
        fn escort_state_after_update(&mut self) -> (f32, f32, f32) {
            self.update(TICK);
            (self.state[S_ESCORT_HP], self.state[S_ESCORT_MAX_HP], self.state[S_ESCORT_FELL])
        }
    }

    #[test]
    fn it_only_walks_while_the_player_is_close() {
        let mut near = escort_game(2);
        for _ in 0..SECONDS {
            unhurt_tick(&mut near);
        }
        assert!(!near.escort.waiting);
        assert!((near.escort.walked - SPEED).abs() < 1.0, "walked {}", near.escort.walked);
        assert_eq!(near.progress(), near.escort.walked);

        let mut far = escort_game(2);
        far.player.x = 1000.0;
        for _ in 0..2 * SECONDS {
            unhurt_tick(&mut far);
        }
        assert!(far.escort.waiting);
        assert_eq!(far.escort.walked, 0.0);
        assert_eq!((far.escort.x, far.escort.y), (START_OFFSET.0, START_OFFSET.1));

        // Coming back sets it walking again.
        far.player.x = 0.0;
        unhurt_tick(&mut far);
        assert!(!far.escort.waiting && far.escort.walked > 0.0);
    }

    #[test]
    fn enemies_chase_the_nearer_of_the_two() {
        let game = escort_game(4);
        let (ex, ey) = (game.escort.x, game.escort.y);
        // Near the villager, far from the player.
        assert_eq!(game.escort.target(ex + 200.0, ey, -300.0, 0.0), (ex, ey));
        // Near the player.
        assert_eq!(game.escort.target(-300.0, 20.0, 0.0, 0.0), (0.0, 0.0));
        let mut fallen = escort_game(4);
        fallen.escort.fallen = true;
        assert_eq!(fallen.escort.target(ex + 200.0, ey, -300.0, 0.0), (-300.0, 0.0));
    }

    #[test]
    fn an_enemy_walks_towards_the_villager_when_it_is_nearer() {
        let mut game = escort_game(4);
        game.player.x = -400.0;
        game.escort.waiting = true;
        let (ex, ey) = (game.escort.x, game.escort.y);
        game.enemies.push(enemy_at(ex + 200.0, ey));
        let before = game.enemies[0].x;
        game.tick();
        assert!(game.enemies[0].x < before, "the enemy did not move towards the villager");
    }

    #[test]
    fn touching_enemies_hurt_the_villager() {
        let mut game = escort_game(5);
        game.enemies.push(enemy_at(game.escort.x, game.escort.y));
        game.tick();
        assert_eq!(game.escort.hp, MAX_HP - KINDS[K_BLOT as usize].dmg * HURT_SCALE);
        assert!(game.events.iter().any(|e| e[0] == EV_HURT));
        // The grace time stops a second hit on the next step.
        game.tick();
        assert_eq!(game.escort.hp, MAX_HP - KINDS[K_BLOT as usize].dmg * HURT_SCALE);
        assert!(!game.over && !game.escort.fallen);
    }

    #[test]
    fn it_is_won_at_the_shrine_and_not_before() {
        let mut game = escort_game(6);
        let goal = game.goal();
        game.escort.walked = goal - 30.0;
        join(&mut game);
        for _ in 0..10 {
            unhurt_tick(&mut game);
        }
        assert!(!game.won, "won {} short of the shrine", goal - game.escort.walked);
        assert!(game.progress() < goal);
        for _ in 0..SECONDS {
            unhurt_tick(&mut game);
            if game.won {
                break;
            }
        }
        assert!(game.won && !game.over && !game.escort.fallen);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));
        let (sx, sy) = game.escort.shrine();
        assert!((game.escort.x - sx).abs() < 1.0 && (game.escort.y - sy).abs() < 1.0);

        // The run is frozen once won.
        game.update(TICK);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_MISSION], M_ESCORT as f32);
        assert_eq!(game.state[S_GOAL], goal);
        assert_eq!(game.state[S_PROGRESS], goal);
        assert_eq!(game.state[S_ESCORT_FELL], 0.0);
    }

    #[test]
    fn the_run_is_lost_when_the_villager_dies() {
        let mut game = escort_game(7);
        game.escort.hp = 1.0;
        game.enemies.push(enemy_at(game.escort.x, game.escort.y));
        game.tick();
        assert!(game.over && !game.won && game.escort.fallen);
        assert_eq!(game.escort.hp, 0.0);
        assert!(game.player.hp > 0.0, "the player did not die");
        assert!(game.events.iter().any(|e| e[0] == EV_DEATH));
        game.update(TICK);
        assert_eq!(game.state[S_OVER], 1.0);
        assert_eq!(game.state[S_WON], 0.0);
        assert_eq!(game.state[S_ESCORT_FELL], 1.0);
        assert_eq!(game.state[S_ESCORT_HP], 0.0);
        // Nothing moves any more.
        let ticks = game.escort.walked;
        game.tick();
        assert_eq!(game.escort.walked, ticks);
    }

    #[test]
    fn the_run_is_lost_when_the_player_dies() {
        let mut game = escort_game(8);
        game.player.hp = 1.0;
        game.enemies.push(enemy_at(0.0, 0.0));
        game.tick();
        assert!(game.over && !game.won);
        assert!(!game.escort.fallen, "a player's death is not the villager's fall");
        game.update(TICK);
        assert_eq!(game.state[S_OVER], 1.0);
        assert_eq!(game.state[S_ESCORT_FELL], 0.0);
        assert_eq!(game.state[S_ESCORT_HP], MAX_HP);
        assert_eq!(game.state[S_ESCORT_MAX_HP], MAX_HP);
    }

    #[test]
    fn the_hash_sees_the_villager() {
        let (mut a, mut b) = (escort_game(9), escort_game(9));
        assert_eq!(a.hash(), b.hash());
        b.escort.hp -= 1.0;
        assert_ne!(a.hash(), b.hash());
        a.tick();
        b.tick();
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn markers_and_figure_are_drawn() {
        let mut game = escort_game(10);
        game.update(TICK);
        let near = game.inst.len();
        assert!(near > 0 && game.inst.iter().any(|i| i.part == P_HEART));
        // Far away: the villager and the shrine are off screen, so only the markers show.
        game.player.x = 5000.0;
        game.update(TICK);
        let rings = game.inst.iter().filter(|i| i.part == P_RING).count();
        assert!(rings >= 1, "no marker for the villager");
        assert!(game.inst.iter().all(|i| i.x.is_finite() && i.y.is_finite() && i.rot.is_finite()));
    }

    #[derive(Clone, Copy)]
    enum Bot {
        /// Only steps away from enemies near the player.
        Flee,
        /// Circles the villager and lets the weapons work.
        Guard,
    }

    struct Outcome {
        won: bool,
        fell: bool,
        died: bool,
        seconds: f32,
        walked: f32,
    }

    fn play(seed: u32, bot: Bot, minutes: u32) -> Outcome {
        let mut game = escort_game(seed);
        game.set_view(640.0, 360.0);
        for t in 0..(60 * 60 * minutes) {
            if t % 6 == 0 {
                game.input = match bot {
                    Bot::Flee => kite(&game, None),
                    Bot::Guard => kite(&game, Some((game.escort.x, game.escort.y))),
                };
            }
            game.tick();
            if game.choosing {
                smart_choose(&mut game);
            }
            if game.won || game.over {
                break;
            }
        }
        Outcome {
            won: game.won,
            fell: game.escort.fallen,
            died: game.over && !game.escort.fallen,
            seconds: game.time,
            walked: game.escort.walked,
        }
    }

    #[test]
    fn a_bot_that_only_flees_does_not_win() {
        for seed in (1..=7).step_by(2) {
            let o = play(seed, Bot::Flee, 10);
            eprintln!(
                "flee seed {seed}: won {} fell {} died {} at {:.0}s, walked {:.0}",
                o.won, o.fell, o.died, o.seconds, o.walked
            );
            assert!(!o.won, "the fleeing bot won with seed {seed}");
        }
    }

    #[test]
    fn a_bot_that_guards_the_villager_can_win() {
        let mut wins = 0;
        for seed in (1..=11).step_by(2) {
            let o = play(seed, Bot::Guard, 12);
            eprintln!(
                "guard seed {seed}: won {} fell {} died {} at {:.0}s, walked {:.0}",
                o.won, o.fell, o.died, o.seconds, o.walked
            );
            if o.won {
                wins += 1;
                assert!(
                    o.seconds >= LEGS as f32 * LEG_LEN / SPEED - 1.0,
                    "won in {}s, faster than the walk allows",
                    o.seconds
                );
            }
        }
        assert!(wins >= 1, "the guarding bot never won");
    }

    /// Choose the offered upgrade that comes first in a fixed order of liking.
    fn smart_choose(game: &mut Game) {
        const LIKED: [usize; 10] =
            [U_VITALITY, U_SCYTHE, U_BEADS, U_PULSE, U_MIGHT, U_HASTE, U_TALISMAN, U_MENDING, U_SWIFT, U_LURE];
        for id in LIKED {
            if let Some(slot) = game.choices.iter().position(|&c| c == id as i32) {
                game.choose(slot);
                return;
            }
        }
        game.choose(0);
    }

    /// Step away from enemies that are close, and otherwise drift to `anchor`.
    fn kite(game: &Game, anchor: Option<(f32, f32)>) -> (f32, f32) {
        const RADIUS: f32 = 110.0;
        let (px, py) = (game.player.x, game.player.y);
        let (mut sx, mut sy) = (0.0, 0.0);
        for e in &game.enemies {
            let (dx, dy) = (px - e.x, py - e.y);
            let d2 = dx * dx + dy * dy;
            if d2 < RADIUS * RADIUS {
                let d = d2.sqrt().max(1.0);
                sx += dx / d * (RADIUS - d) / RADIUS * 3.0;
                sy += dy / d * (RADIUS - d) / RADIUS * 3.0;
            }
        }
        if let Some((ax, ay)) = anchor {
            let (dx, dy) = (ax - px, ay - py);
            let d = (dx * dx + dy * dy).sqrt().max(1.0);
            if d > 70.0 {
                sx += dx / d * 2.0;
                sy += dy / d * 2.0;
            }
        }
        (sx, sy)
    }

    #[test]
    fn tunes_change_the_route_and_the_villager() {
        let mut game = Game::new(3);
        game.set_tune(T_ESCORT_LEG_LEN, 500.0);
        game.set_tune(T_ESCORT_HP, 1200.0);
        game.set_tune(T_ESCORT_SPEED, 160.0);
        game.set_tune(T_ESCORT_FOLLOW, 500.0);
        game.start(M_ESCORT);
        assert_eq!(game.goal(), 7.0 * 500.0);
        assert_eq!(game.escort.hp, 1200.0);
        game.update(TICK);
        assert_eq!(game.state[S_ESCORT_MAX_HP], 1200.0);
        assert_eq!(game.state[S_GOAL], 3500.0);
        for leg in 0..LEGS {
            let (a, b) = (game.escort.route[leg], game.escort.route[leg + 1]);
            assert!(((b.0 - a.0).hypot(b.1 - a.1) - 500.0).abs() < 1.0);
        }
        // 400 away is inside the wider follow radius, so it walks, at the set speed.
        game.player.x = 400.0;
        let before = game.escort.walked;
        game.tick();
        assert!(!game.escort.waiting);
        assert!((game.escort.walked - before - 160.0 * TICK).abs() < 1.0e-3);
    }

    #[test]
    fn extreme_tunes_do_not_break_a_run() {
        let mut game = Game::new(3);
        game.set_tune(T_ESCORT_LEG_LEN, 500.0);
        game.set_tune(T_ESCORT_HP, 60.0);
        game.set_tune(T_ESCORT_SPEED, 160.0);
        game.set_tune(T_ESCORT_FOLLOW, 100.0);
        game.start(M_ESCORT);
        for _ in 0..400 {
            game.player.hp = game.player.max_hp;
            game.tick();
            if game.choosing {
                game.choose(0);
            }
        }
        assert!(game.escort.walked <= game.goal());
        assert!(game.escort.x.is_finite() && game.escort.y.is_finite());
    }
}
