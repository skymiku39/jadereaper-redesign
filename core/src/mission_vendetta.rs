//! The vendetta mission: travel to a marked place and kill the marked group
//! that waits there.
//!
//! The place is picked from the run's seed, far from the start. The marked
//! enemies are ordinary entries of `Game::enemies` (brutes with a lot more
//! health), told apart by id. They stand at home slots around the place until
//! the player comes within `AGGRO_R` of it, then chase the player. If the
//! player gets more than `LEASH_R` away they walk home again. The horde keeps
//! spawning around the player the whole way, so the player levels up on the
//! journey. The mission is won when every marked enemy is dead.
//!
//! This is a child module of `game` so it can reach the private fields of `Game`.

use super::*;

/// Radius of the circle the marked enemies stand on.
const SLOT_R: f32 = 100.0;
/// The group wakes when the player is this close to the place.
const AGGRO_R: f32 = 600.0;
/// A woken group gives up and walks home when the player is this far away.
const LEASH_R: f32 = 1200.0;
/// Walking home is faster than chasing.
const HOME_SPEED: f32 = 1.5;
/// The pointer at the screen edge stays this far inside it.
const EDGE_PAD: f32 = 36.0;

const RED: [f32; 4] = [0.74, 0.25, 0.18, 1.0];

#[derive(Default)]
pub struct Vendetta {
    /// Ids of the marked enemies, one per home slot.
    ids: Vec<u32>,
    homes: Vec<(f32, f32)>,
    pub place: (f32, f32),
    pub aggro: bool,
}

impl Vendetta {
    /// Wake or calm the group by how close the player is to the place.
    pub fn step(&mut self, px: f32, py: f32) {
        if self.ids.is_empty() {
            return;
        }
        let (dx, dy) = (px - self.place.0, py - self.place.1);
        let d2 = dx * dx + dy * dy;
        if !self.aggro && d2 < AGGRO_R * AGGRO_R {
            self.aggro = true;
        } else if self.aggro && d2 > LEASH_R * LEASH_R {
            self.aggro = false;
        }
    }

    pub fn is_marked(&self, id: u32) -> bool {
        self.ids.contains(&id)
    }

    /// The velocity of a marked enemy, or None for any other enemy.
    pub fn steer(&self, id: u32, x: f32, y: f32, px: f32, py: f32, speed: f32) -> Option<(f32, f32)> {
        let slot = self.ids.iter().position(|&m| m == id)?;
        let (tx, ty, v) = if self.aggro {
            (px, py, speed)
        } else {
            let h = self.homes[slot];
            (h.0, h.1, speed * HOME_SPEED)
        };
        let (dx, dy) = (tx - x, ty - y);
        let d = (dx * dx + dy * dy).sqrt();
        // Stand still once at home, instead of jittering around it.
        if d < 4.0 {
            return Some((0.0, 0.0));
        }
        Some((dx / d * v, dy / d * v))
    }

    /// The words the state hash mixes in.
    pub fn hash_words(&self) -> [u32; 3] {
        [self.place.0.to_bits(), self.place.1.to_bits(), self.aggro as u32]
    }

    /// The screen position of the pointer to the place, if the place is off screen.
    fn pointer(&self, cam: (f32, f32), view: (f32, f32)) -> Option<(f32, f32, f32, f32)> {
        let (dx, dy) = (self.place.0 - cam.0, self.place.1 - cam.1);
        let (hw, hh) = (view.0 - EDGE_PAD, view.1 - EDGE_PAD);
        if dx.abs() <= hw && dy.abs() <= hh {
            return None;
        }
        let t = (hw / dx.abs().max(0.001)).min(hh / dy.abs().max(0.001));
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        Some((cam.0 + dx * t, cam.1 + dy * t, dx / d, dy / d))
    }
}

impl Game {
    /// Pick the place from the seed and put the marked group on it.
    pub(super) fn vendetta_start(&mut self) {
        let n = self.goal() as usize;
        // The place is this far from the start, in world units, between the two
        // tunes in either order. At the default, straight walking at the base
        // speed takes about a minute and a half; the crowd makes the real trip longer.
        let (d_a, d_b) = (self.tune[T_VENDETTA_DIST_MIN], self.tune[T_VENDETTA_DIST_MAX]);
        // Health of each marked enemy. Unlike the horde it does not grow with time.
        let group_hp = self.tune[T_VENDETTA_GROUP_HP];
        let a = self.rng.range(0.0, TAU);
        let dist = self.rng.range(d_a.min(d_b), d_a.max(d_b));
        let place = (self.player.x + fmath::cos(a) * dist, self.player.y + fmath::sin(a) * dist);
        let a0 = self.rng.range(0.0, TAU);
        let mut v = Vendetta { place, ..Vendetta::default() };
        for i in 0..n {
            let b = a0 + i as f32 * TAU / n as f32;
            let home = (place.0 + fmath::cos(b) * SLOT_R, place.1 + fmath::sin(b) * SLOT_R);
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            let seed = self.rng.f();
            self.enemies.push(Enemy {
                x: home.0,
                y: home.1,
                kx: 0.0,
                ky: 0.0,
                hp: group_hp,
                kind: K_BRUTE,
                flash: 0.0,
                seed,
                bead_cd: 0.0,
                tags: Default::default(),
                id,
                face: 1.0,
            });
            v.ids.push(id);
            v.homes.push(home);
        }
        self.vendetta = v;
    }

    /// Marked enemies still standing.
    fn vendetta_alive(&self) -> usize {
        self.enemies.iter().filter(|e| self.vendetta.is_marked(e.id)).count()
    }

    /// Marked enemies killed.
    pub(super) fn vendetta_progress(&self) -> f32 {
        (self.vendetta.ids.len() - self.vendetta_alive()) as f32
    }

    pub(super) fn vendetta_won(&self) -> bool {
        !self.vendetta.ids.is_empty() && self.vendetta_alive() == 0
    }

    /// The place on the ground: a red ring with stones, and a ring under each marked enemy.
    pub(super) fn vendetta_draw_ground(&mut self) {
        if self.mission != M_VENDETTA || self.won {
            return;
        }
        let (x, y) = self.vendetta.place;
        let ring = [RED[0], RED[1], RED[2], 0.55];
        self.put(x, y, 190.0, 120.0, 0.0, P_RING, 0.0, 0.2, ring);
        self.put(x, y, 150.0, 94.0, 0.0, P_RING, 0.0, 0.6, [ring[0], ring[1], ring[2], 0.3]);
        for k in 0..6 {
            let a = k as f32 * TAU / 6.0 + 0.4;
            let (sx, sy) = (x + fmath::cos(a) * 190.0, y + fmath::sin(a) * 120.0);
            self.put(sx, sy, 22.0, 12.0, 0.0, P_STONE, 0.0, k as f32 * 0.17, [INK[0], INK[1], INK[2], 0.5]);
        }
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if self.vendetta.is_marked(e.id) {
                let r = KINDS[e.kind as usize].r;
                let (ex, ey) = (e.x, e.y);
                self.put(ex, ey, r * 1.5, r * 0.7, 0.0, P_RING, 0.0, 0.4, [RED[0], RED[1], RED[2], 0.9]);
            }
        }
    }

    /// A tag over each marked enemy, and a pointer at the screen edge while the place is off screen.
    pub(super) fn vendetta_draw_marks(&mut self) {
        if self.mission != M_VENDETTA || self.won {
            return;
        }
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            if self.vendetta.is_marked(e.id) {
                let k = &KINDS[e.kind as usize];
                let bob = fmath::sin(self.anim * 4.0 + e.seed * 20.0) * 3.0;
                let (ex, ey) = (e.x, e.y - k.hh * 2.0 - 16.0 + bob);
                self.put(ex, ey, 7.0, 12.0, 0.0, P_TALISMAN, 0.0, 0.5, RED);
            }
        }
        if let Some((x, y, ux, uy)) = self.vendetta.pointer(self.cam, (self.view_w, self.view_h)) {
            self.put(x, y, 16.0, 16.0, 0.0, P_RING, 0.0, 0.3, RED);
            let place = self.vendetta.place;
            self.point(place.0, place.1);
            // A trail of beads toward the middle of the screen shows the direction.
            for s in 1..4 {
                let off = s as f32 * 20.0 + 6.0;
                let r = 8.0 - s as f32 * 1.6;
                self.put(x - ux * off, y - uy * off, r, r, 0.0, P_BEAD, 0.0, s as f32 * 0.1, RED);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIST_MIN: f32 = TUNE_DEFAULT[T_VENDETTA_DIST_MIN];
    const DIST_MAX: f32 = TUNE_DEFAULT[T_VENDETTA_DIST_MAX];

    fn started(seed: u32) -> Game {
        let mut game = Game::new(seed);
        game.set_view(640.0, 360.0);
        game.start(M_VENDETTA);
        game
    }

    fn marked_positions(game: &Game) -> Vec<(u32, f32, f32)> {
        let mut v: Vec<_> =
            game.enemies.iter().filter(|e| game.vendetta.is_marked(e.id)).map(|e| (e.id, e.x, e.y)).collect();
        v.sort_by_key(|m| m.0);
        v
    }

    /// Advance with the player kept alive, picking the first upgrade card.
    fn run(game: &mut Game, seconds: u32) {
        for _ in 0..(60 * seconds) {
            game.player.hp = game.player.max_hp;
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

    fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
        ((a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)).sqrt()
    }

    #[test]
    fn the_place_comes_from_the_seed_and_is_far_away() {
        let a = started(21);
        let b = started(21);
        let c = started(22);
        assert_eq!(a.vendetta.place, b.vendetta.place);
        assert_ne!(a.vendetta.place, c.vendetta.place);
        for g in [&a, &c] {
            let d = dist(g.vendetta.place, (g.player.x, g.player.y));
            assert!((DIST_MIN - 1.0..=DIST_MAX + 1.0).contains(&d), "{d}");
        }
        assert_eq!(a.vendetta.ids.len(), a.goal() as usize);
        assert_eq!(marked_positions(&a).len(), MISSION_GOAL[M_VENDETTA] as usize);
    }

    #[test]
    fn the_group_waits_and_is_never_despawned() {
        let mut game = started(5);
        let before = marked_positions(&game);
        assert_eq!(before.len(), game.goal() as usize);
        // The player stays at the start, 16000 units away: far enough that the
        // horde's wrap-around rule would move any ordinary enemy.
        run(&mut game, 120);
        assert!(!game.over && !game.won);
        assert!(!game.vendetta.aggro);
        assert_eq!(marked_positions(&game), before, "the group moved or changed");
        assert_eq!(game.vendetta_progress(), 0.0);
    }

    #[test]
    fn the_group_attacks_when_the_player_is_close_and_walks_home_when_he_leaves() {
        let mut game = started(5);
        let (px, py) = game.vendetta.place;
        let homes = game.vendetta.homes.clone();
        game.player.x = px + AGGRO_R - 50.0;
        game.player.y = py;
        run(&mut game, 1);
        assert!(game.vendetta.aggro);
        // Hold the player still; the group closes in.
        run(&mut game, 6);
        let near = marked_positions(&game);
        assert!(near.iter().any(|m| dist((m.1, m.2), (px + 100.0, py)) > 0.0));
        let closest = near.iter().map(|m| dist((m.1, m.2), (game.player.x, game.player.y))).fold(f32::MAX, f32::min);
        assert!(closest < AGGRO_R - 50.0 - 100.0 - 50.0, "no one came, closest {closest}");

        // The player runs off: the group gives up and returns to its slots.
        game.player.x = px + LEASH_R + 400.0;
        for _ in 0..(60 * 60) {
            game.player.hp = game.player.max_hp;
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            game.player.x = px + LEASH_R + 400.0;
            game.player.y = py;
        }
        assert!(!game.vendetta.aggro);
        let back = marked_positions(&game);
        assert_eq!(back.len(), homes.len());
        for (m, h) in back.iter().zip(game.vendetta.homes.iter()) {
            assert!(dist((m.1, m.2), *h) < 40.0, "not home: {m:?} {h:?}");
        }
    }

    #[test]
    fn only_marked_kills_count_and_the_last_one_wins() {
        let mut game = started(8);
        let n = game.goal() as usize;
        game.kills = 100;
        game.tick();
        assert_eq!(game.progress(), 0.0, "normal kills must not count");

        // A normal enemy dying does not move progress either.
        game.enemies.push(Enemy {
            x: 5.0,
            y: 5.0,
            kx: 0.0,
            ky: 0.0,
            hp: 0.0,
            kind: K_BLOT,
            flash: 0.0,
            seed: 0.0,
            bead_cd: 0.0,
            tags: Default::default(),
            id: 9999,
            face: 1.0,
        });
        game.tick();
        assert_eq!(game.progress(), 0.0);

        let ids = game.vendetta.ids.clone();
        for (k, id) in ids.iter().enumerate() {
            assert!(!game.won, "won with {k} of {n} dead");
            for e in game.enemies.iter_mut().filter(|e| e.id == *id) {
                e.hp = 0.0;
            }
            game.tick();
            assert_eq!(game.progress(), (k + 1) as f32);
        }
        assert!(game.won && !game.over);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));
        game.update(TICK);
        assert_eq!(game.state[S_GOAL], n as f32);
        assert_eq!(game.state[S_PROGRESS], n as f32);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_MISSION], M_VENDETTA as f32);
    }

    #[test]
    fn the_horde_keeps_coming_during_the_journey() {
        let mut game = started(3);
        let marked = game.goal() as usize;
        run(&mut game, 60);
        assert!(game.enemies.len() > marked + 20, "{} enemies", game.enemies.len());
        assert!(game.kills > 0);
        assert_eq!(game.vendetta_progress(), 0.0);
    }

    #[test]
    fn the_mission_is_lost_on_death() {
        let mut game = started(4);
        let (px, py) = game.vendetta.place;
        game.player.x = px;
        game.player.y = py;
        for _ in 0..(60 * 120) {
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            if game.over {
                break;
            }
        }
        assert!(game.over && !game.won);
    }

    #[test]
    fn the_pointer_shows_only_while_the_place_is_off_screen() {
        let mut game = started(6);
        game.present();
        let (x, y, ux, uy) = game.vendetta.pointer(game.cam, (game.view_w, game.view_h)).expect("off screen");
        assert!((x - game.cam.0).abs() <= game.view_w && (y - game.cam.1).abs() <= game.view_h);
        assert!((ux * ux + uy * uy - 1.0).abs() < 1e-3);
        let place = game.vendetta.place;
        game.player.x = place.0 + 50.0;
        game.player.y = place.1 + 80.0;
        game.present();
        assert!(game.vendetta.pointer(game.cam, (game.view_w, game.view_h)).is_none());
    }

    #[test]
    fn other_missions_have_no_marked_enemies() {
        for m in [M_ENDLESS, M_SURVIVE, M_KILLS] {
            let mut game = Game::new(2);
            game.start(m);
            run(&mut game, 10);
            assert!(game.vendetta.ids.is_empty());
            assert!(game.enemies.iter().all(|e| !game.vendetta.is_marked(e.id)));
        }
    }

    // ------------------------------------------------------------ bots

    /// Which way an enemy-avoiding player should step: away from the crowd nearby.
    fn away_from_crowd(game: &Game) -> (f32, f32) {
        let (px, py) = (game.player.x, game.player.y);
        let (mut ax, mut ay) = (0.0, 0.0);
        for e in &game.enemies {
            let (dx, dy) = (px - e.x, py - e.y);
            let d2 = dx * dx + dy * dy;
            if d2 < 350.0 * 350.0 {
                let d = d2.sqrt().max(1.0);
                ax += dx / d * (350.0 - d);
                ay += dy / d * (350.0 - d);
            }
        }
        (ax, ay)
    }

    fn unit(v: (f32, f32)) -> (f32, f32) {
        let d = (v.0 * v.0 + v.1 * v.1).sqrt();
        if d < 0.001 {
            (0.0, 0.0)
        } else {
            (v.0 / d, v.1 / d)
        }
    }

    /// Play `minutes` of a run with a bot. `hunt` walks to the place and fights.
    /// Returns (won, seconds, level, alive marked enemies).
    fn play_bot(seed: u32, hunt: bool, minutes: u32) -> (bool, f32, u32, usize, f32) {
        let mut game = started(seed);
        let mut rng = Rng(seed.wrapping_mul(2654435761) | 1);
        for t in 0..(60 * 60 * minutes) {
            if t % 6 == 0 {
                let flee = away_from_crowd(&game);
                let (px, py) = (game.player.x, game.player.y);
                let place = game.vendetta.place;
                let to_place = (place.0 - px, place.1 - py);
                let hurt = game.player.hp < game.player.max_hp * 0.45;
                let target = game
                    .enemies
                    .iter()
                    .filter(|e| game.vendetta.is_marked(e.id))
                    .min_by(|a, b| {
                        let da = dist((a.x, a.y), (px, py));
                        let db = dist((b.x, b.y), (px, py));
                        da.partial_cmp(&db).unwrap()
                    })
                    .map(|e| (e.x - px, e.y - py));
                let v = if !hunt {
                    // Flee any crowd; otherwise drift sideways to nothing in particular.
                    let f = unit(flee);
                    if f == (0.0, 0.0) {
                        (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0))
                    } else {
                        f
                    }
                } else if dist(place, (px, py)) > 450.0 {
                    // Walk to the place, bending around the crowd a little.
                    let w = unit(to_place);
                    let f = unit(flee);
                    unit((w.0 + f.0 * 1.0, w.1 + f.1 * 1.0))
                } else if hurt {
                    unit(flee)
                } else if let Some(d) = target {
                    // Close in until the scythe reaches, then stay on it.
                    let len = (d.0 * d.0 + d.1 * d.1).sqrt();
                    if len > 90.0 {
                        unit(d)
                    } else {
                        (0.0, 0.0)
                    }
                } else {
                    (0.0, 0.0)
                };
                game.input = v;
            }
            game.tick();
            if game.choosing {
                game.choose((rng.next_u32() % 3) as usize);
            }
            game.events.clear();
            if game.won || game.over {
                break;
            }
        }
        (
            game.won,
            game.time,
            game.player.level,
            game.vendetta_alive(),
            dist(game.vendetta.place, (game.player.x, game.player.y)),
        )
    }

    #[test]
    fn a_bot_that_only_runs_away_does_not_win() {
        for seed in 1..=3 {
            let (won, t, level, alive, _) = play_bot(seed, false, 10);
            assert!(!won, "seed {seed}: won at {t}s");
            assert_eq!(alive, MISSION_GOAL[M_VENDETTA] as usize, "level {level}");
        }
    }

    /// Run with `cargo test --release mission_vendetta -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn hunter_bot_report() {
        for seed in 1..=12 {
            let (won, t, level, alive, left) = play_bot(seed, true, 12);
            println!("seed {seed}: won {won} at {t:.0}s, level {level}, marked left {alive}, {left:.0} from the place");
        }
    }

    #[test]
    fn tunes_change_the_group_and_the_place() {
        let mut game = Game::new(7);
        game.set_view(640.0, 360.0);
        game.set_tune(T_VENDETTA_GOAL, 12.0);
        game.set_tune(T_VENDETTA_DIST_MIN, 2000.0);
        game.set_tune(T_VENDETTA_DIST_MAX, 3000.0);
        game.set_tune(T_VENDETTA_GROUP_HP, 1200.0);
        game.start(M_VENDETTA);
        assert_eq!(game.goal(), 12.0);
        assert_eq!(game.vendetta.ids.len(), 12);
        let d = dist(game.vendetta.place, (game.player.x, game.player.y));
        assert!((1999.0..=3001.0).contains(&d), "{d}");
        for e in game.enemies.iter().filter(|e| game.vendetta.is_marked(e.id)) {
            assert_eq!(e.hp, 1200.0);
        }
        game.update(TICK);
        assert_eq!(game.state[S_GOAL], 12.0);
    }

    #[test]
    fn a_minimum_above_the_maximum_is_put_in_order() {
        let mut game = Game::new(7);
        game.set_view(640.0, 360.0);
        game.set_tune(T_VENDETTA_GOAL, 1.0);
        game.set_tune(T_VENDETTA_DIST_MIN, 30000.0);
        game.set_tune(T_VENDETTA_DIST_MAX, 2000.0);
        game.start(M_VENDETTA);
        let d = dist(game.vendetta.place, (game.player.x, game.player.y));
        assert!((1999.0..=30001.0).contains(&d), "{d}");
        run(&mut game, 8);
        assert!(game.vendetta.place.0.is_finite() && !game.over);
    }
}
