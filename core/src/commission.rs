//! Commissions (委託) in endless (docs/design.md, section 4.3).
//!
//! Now and then a commission turns up somewhere on the map, pointed at by an
//! edge marker. It waits a while and then goes, with no penalty for leaving it.
//! Standing at its spot for a moment accepts it. The one kind so far is
//! defending a village: for a while, enemies come from all round and go for
//! the village instead of the player. If the village stands to the end, the
//! player gets one more level-up and some health back; if it falls, nothing
//! is lost but the reward. Then the next one comes after a pause. Only one
//! commission is on the map at a time.
//!
//! The whole thing is off when the `commissions` tune is 0, and in every
//! mission but endless.

use super::Game;
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// Seconds into the run before the first commission.
const FIRST: f32 = 30.0;
/// Seconds between one commission ending (or going) and the next.
const PAUSE: f32 = 40.0;
/// Seconds an offered commission waits to be taken.
const OFFER: f32 = 45.0;
/// Seconds to stand at the spot to take it.
const STAND: f32 = 2.0;
/// Seconds the village has to be held.
const DEFEND: f32 = 35.0;
/// The player takes the commission within this distance of the spot.
pub const RADIUS: f32 = 80.0;
/// How far from the player a commission turns up.
const NEAR: f32 = 650.0;
const FAR: f32 = 950.0;
/// The village's health, and how close an enemy must be to harm it.
const VILLAGE_HP: f32 = 150.0;
const BODY: f32 = 34.0;
/// Damage per second an enemy at the village deals, as a share of its contact damage.
const GNAW: f32 = 0.3;
/// Where the attackers come from: a ring round the village.
const RING: f32 = 430.0;
/// Attackers per second at the start of a run, growing with the run's time.
const RATE: f32 = 1.0;
/// Health back on success, besides the level-up.
const HEAL: f32 = 30.0;

const GOLD: [f32; 4] = [0.85, 0.72, 0.30, 1.0];

/// No commission on the map; the next is due when `left` runs out.
pub const WAITING: u8 = 0;
/// One is on the map, waiting to be taken.
pub const OFFERED: u8 = 1;
/// The village is being defended.
pub const DEFENDING: u8 = 2;

pub struct Commission {
    pub phase: u8,
    /// Where the village is.
    pub x: f32,
    pub y: f32,
    /// Seconds left of the current phase.
    pub left: f32,
    /// Seconds stood at the spot without leaving it, while offered.
    pub held: f32,
    pub hp: f32,
    /// Enemies sent after the village, by id.
    attackers: Vec<u32>,
    spawn_acc: f32,
    /// Damage dealt to the village this step, applied in `commission_step`.
    dmg: f32,
    /// 1 right after the village was hit, fading out. Drawing only.
    flash: f32,
    /// Commissions done and failed this run.
    pub done: u32,
    pub failed: u32,
}

impl Default for Commission {
    fn default() -> Commission {
        Commission {
            phase: WAITING,
            x: 0.0,
            y: 0.0,
            left: FIRST,
            held: 0.0,
            hp: VILLAGE_HP,
            attackers: Vec::new(),
            spawn_acc: 0.0,
            dmg: 0.0,
            flash: 0.0,
            done: 0,
            failed: 0,
        }
    }
}

impl Commission {
    /// The village, for an enemy sent after it.
    pub fn target(&self, enemy_id: u32) -> Option<(f32, f32)> {
        if self.phase == DEFENDING && self.attackers.contains(&enemy_id) {
            Some((self.x, self.y))
        } else {
            None
        }
    }

    /// An enemy at distance `d` from the village (its own radius `r` included) deals `dmg` this step.
    pub fn gnaw(&mut self, d: f32, r: f32, dmg: f32) {
        if d < r + BODY {
            self.dmg += dmg * GNAW;
        }
    }

    pub fn max_hp(&self) -> f32 {
        VILLAGE_HP
    }

    /// How long the current phase lasts in all, for the HUD.
    pub fn need(&self) -> f32 {
        match self.phase {
            OFFERED => STAND,
            DEFENDING => DEFEND,
            _ => 0.0,
        }
    }

    /// Fold into the run's state hash.
    pub fn mix(&self, mix: &mut impl FnMut(u32)) {
        for v in [self.x, self.y, self.left, self.held, self.hp] {
            mix(v.to_bits());
        }
        mix(self.phase as u32);
        mix(self.attackers.len() as u32);
        mix(self.done);
        mix(self.failed);
    }
}

impl Game {
    /// Whether commissions happen in this run.
    pub(crate) fn commissions_on(&self) -> bool {
        self.mission == M_ENDLESS && self.tune[T_COMMISSIONS] > 0.5
    }

    /// One step of the commission on the map, if any.
    pub(crate) fn commission_step(&mut self, dt: f32) {
        if !self.commissions_on() {
            return;
        }
        self.commission.left -= dt;
        self.commission.flash = (self.commission.flash - dt * 4.0).max(0.0);
        match self.commission.phase {
            WAITING => {
                if self.commission.left <= 0.0 {
                    let a = self.rng.range(0.0, TAU);
                    let d = self.rng.range(NEAR, FAR);
                    let c = &mut self.commission;
                    c.x = self.player.x + fmath::cos(a) * d;
                    c.y = self.player.y + fmath::sin(a) * d;
                    c.phase = OFFERED;
                    c.left = OFFER;
                    c.held = 0.0;
                    let (x, y) = (c.x, c.y);
                    self.event(EV_COMMISSION, x, y, 0.0);
                }
            }
            OFFERED => {
                let c = &mut self.commission;
                let (dx, dy) = (self.player.x - c.x, self.player.y - c.y);
                c.held = if dx * dx + dy * dy <= RADIUS * RADIUS { c.held + dt } else { 0.0 };
                if c.held >= STAND {
                    c.phase = DEFENDING;
                    c.left = DEFEND;
                    c.hp = VILLAGE_HP;
                    c.attackers.clear();
                    c.spawn_acc = 0.0;
                    let (x, y) = (c.x, c.y);
                    self.event(EV_LEVELUP, x, y, 0.0);
                } else if c.left <= 0.0 {
                    // Not taken in time: it goes, and another comes later.
                    c.phase = WAITING;
                    c.left = PAUSE;
                }
            }
            _ => self.commission_defend(dt),
        }
    }

    /// The village under attack: send enemies at it, let them harm it, and
    /// end the commission when time is up or the village falls.
    fn commission_defend(&mut self, dt: f32) {
        // The attackers are the commission's own, apart from the horde and its tune.
        self.commission.spawn_acc += RATE * (1.0 + self.time / 120.0) * dt;
        while self.commission.spawn_acc >= 1.0 {
            self.commission.spawn_acc -= 1.0;
            let a = self.rng.range(0.0, TAU);
            let kind = if self.rng.f() < 0.2 { K_BRUTE } else { K_BLOT };
            let (x, y) = (self.commission.x + fmath::cos(a) * RING, self.commission.y + fmath::sin(a) * RING);
            let (before, id) = (self.enemies.len(), self.next_id);
            self.spawn_enemy_at(kind, x, y);
            if self.enemies.len() > before {
                self.commission.attackers.push(id);
            }
        }
        let c = &mut self.commission;
        if c.dmg > 0.0 {
            c.hp = (c.hp - c.dmg).max(0.0);
            c.dmg = 0.0;
            c.flash = 1.0;
        }
        let (x, y) = (c.x, c.y);
        if c.hp <= 0.0 {
            c.phase = WAITING;
            c.left = PAUSE;
            c.failed += 1;
            // The attackers that are left turn on the player.
            c.attackers.clear();
            self.event(EV_COMMISSION_FAILED, x, y, 0.0);
        } else if c.left <= 0.0 {
            c.phase = WAITING;
            c.left = PAUSE;
            c.done += 1;
            c.attackers.clear();
            self.pending += 1;
            self.player.hp = (self.player.hp + HEAL).min(self.player.max_hp);
            self.event(EV_COMMISSION_DONE, x, y, 0.0);
        }
    }

    /// The village, and the circle to stand in while it is offered.
    pub(crate) fn commission_draw_ground(&mut self) {
        if !self.commissions_on() || self.commission.phase == WAITING {
            return;
        }
        let c = &self.commission;
        let (x, y, phase, flash) = (c.x, c.y, c.phase, c.flash);
        if phase == OFFERED {
            let pose = (self.anim * 3.0).floor() as i32 & 1;
            let swell = if pose == 0 { 1.0 } else { 1.08 };
            let done = (c.held / STAND).clamp(0.0, 1.0);
            self.put(x, y, RADIUS * 1.1, RADIUS * 0.85, 0.0, P_ELLIPSE, 0.0, 0.1, [GOLD[0], GOLD[1], GOLD[2], 0.16]);
            if done > 0.0 {
                let r = RADIUS * done;
                self.put(x, y, r * 1.1, r * 0.85, 0.0, P_ELLIPSE, 0.0, 0.3, [GOLD[0], GOLD[1], GOLD[2], 0.5]);
            }
            self.put(x, y, RADIUS * 1.15 * swell, RADIUS * 0.9 * swell, 0.0, P_RING, 0.0, 0.5, GOLD);
        }
        // The village: three paper houses on a stone wash, reddening as it is harmed.
        let life = (self.commission.hp / VILLAGE_HP).clamp(0.0, 1.0);
        let roof = [0.55 + 0.25 * (1.0 - life), 0.30 * life + 0.15, 0.22 * life + 0.12, 1.0];
        self.put(x, y + 6.0, BODY * 2.0, BODY * 0.7, 0.0, P_STONE, 0.0, 0.4, [INK[0], INK[1], INK[2], 0.3]);
        for (k, (ox, s)) in [(-26.0, 0.8), (0.0, 1.0), (26.0, 0.8)].into_iter().enumerate() {
            let seed = 0.2 + k as f32 * 0.3;
            self.put(x + ox, y - 18.0 * s, 15.0 * s, 16.0 * s, 0.0, P_SCRAP, 0.0, seed, [0.93, 0.88, 0.76, 1.0]);
            self.put(x + ox, y - 36.0 * s, 18.0 * s, 9.0 * s, 0.0, P_TALISMAN, flash, seed, roof);
        }
    }

    /// An arrow at the screen edge pointing to the commission while it is off screen.
    pub(crate) fn commission_draw_marker(&mut self) {
        if !self.commissions_on() || self.commission.phase == WAITING {
            return;
        }
        let (cx, cy) = (self.commission.x, self.commission.y);
        let (dx, dy) = (cx - self.cam.0, cy - self.cam.1);
        let (hw, hh) = ((self.view_w - 44.0).max(1.0), (self.view_h - 44.0).max(1.0));
        if dx.abs() <= hw && dy.abs() <= hh {
            return;
        }
        self.point(cx, cy);
        let k = (hw / dx.abs().max(0.001)).min(hh / dy.abs().max(0.001));
        let (mx, my) = (self.cam.0 + dx * k, self.cam.1 + dy * k);
        let rot = fmath::atan2(dy, dx) + TAU * 0.25;
        self.put(mx, my, 22.0, 22.0, 0.0, P_RING, 0.0, 0.5, [GOLD[0], GOLD[1], GOLD[2], 0.6]);
        self.put(mx, my, 9.0, 16.0, rot, P_TALISMAN, 0.0, 0.5, GOLD);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::TICK;

    /// An endless run with the horde switched off, so only the commission sends enemies.
    fn quiet() -> Game {
        let mut g = Game::new(4);
        g.tune[T_SPAWN_RATE] = 0.0;
        g.start(M_ENDLESS);
        g
    }

    /// Play `seconds`, keeping the player alive and standing where he is unless `walk_to` says where to go.
    fn play(g: &mut Game, seconds: f32, walk_to: Option<(f32, f32)>) {
        for _ in 0..(seconds / TICK) as u32 {
            g.player.hp = g.player.max_hp;
            g.input = match walk_to {
                Some((x, y)) => (x - g.player.x, y - g.player.y),
                None => (0.0, 0.0),
            };
            g.tick();
            while g.choosing {
                g.choose(0);
                if g.choosing {
                    g.choose(A_SCYTHE);
                }
            }
        }
    }

    #[test]
    fn a_commission_turns_up_waits_and_goes() {
        let mut g = quiet();
        play(&mut g, FIRST - 1.0, None);
        assert_eq!(g.commission.phase, WAITING);
        play(&mut g, 1.5, None);
        assert_eq!(g.commission.phase, OFFERED);
        let (dx, dy) = (g.commission.x - g.player.x, g.commission.y - g.player.y);
        let d = (dx * dx + dy * dy).sqrt();
        assert!((NEAR..=FAR).contains(&d), "{d} away");
        // It is pointed at, and the state says so.
        g.update(TICK);
        assert!(g.pointers.iter().any(|p| p[0] == g.commission.x && p[1] == g.commission.y));
        assert_eq!(g.state[S_COMMISSION], OFFERED as f32);
        // Left alone, it goes, and another comes after the pause.
        play(&mut g, OFFER + 0.5, None);
        assert_eq!(g.commission.phase, WAITING);
        play(&mut g, PAUSE + 0.5, None);
        assert_eq!(g.commission.phase, OFFERED);
    }

    #[test]
    fn a_village_held_to_the_end_gives_a_level_up() {
        let mut g = quiet();
        play(&mut g, FIRST + 0.5, None);
        let spot = (g.commission.x, g.commission.y);
        // Walk there; standing long enough takes it.
        play(&mut g, 8.0, Some(spot));
        assert_eq!(g.commission.phase, DEFENDING);
        assert!(!g.enemies.is_empty(), "attackers should come");
        // Make the village unbreakable, and see the commission through.
        let level = g.player.level;
        for _ in 0..((DEFEND + 1.0) / TICK) as u32 {
            g.commission.hp = VILLAGE_HP;
            g.player.hp = g.player.max_hp;
            g.input = (spot.0 - g.player.x, spot.1 - g.player.y);
            g.tick();
            if g.events.iter().any(|e| e[0] == EV_COMMISSION_DONE) {
                break;
            }
            g.events.clear();
            while g.choosing {
                g.choose(0);
                if g.choosing {
                    g.choose(A_SCYTHE);
                }
            }
        }
        assert_eq!(g.commission.done, 1);
        assert_eq!(g.commission.phase, WAITING);
        assert!(g.choosing || g.player.level > level, "the reward is a level-up");
    }

    #[test]
    fn a_fallen_village_only_costs_the_reward() {
        let mut g = quiet();
        play(&mut g, FIRST + 0.5, None);
        let spot = (g.commission.x, g.commission.y);
        play(&mut g, 8.0, Some(spot));
        assert_eq!(g.commission.phase, DEFENDING);
        let pending = g.pending;
        // Walk far away and let the attackers have the village.
        let away = (spot.0 + 5000.0, spot.1);
        for _ in 0..(DEFEND / TICK) as u32 {
            g.player.hp = g.player.max_hp;
            g.input = (away.0 - g.player.x, away.1 - g.player.y);
            g.tick();
            if g.commission.failed > 0 {
                break;
            }
            while g.choosing {
                g.choose(0);
                if g.choosing {
                    g.choose(A_SCYTHE);
                }
            }
        }
        assert_eq!(g.commission.failed, 1, "the village should fall with nobody there");
        assert_eq!(g.commission.done, 0);
        assert_eq!(g.pending, pending);
        assert!(!g.over);
    }

    #[test]
    fn no_commission_outside_endless_or_with_the_tune_off() {
        let mut g = Game::new(4);
        g.start(M_SURVIVE);
        play(&mut g, FIRST + 5.0, None);
        assert_eq!(g.commission.phase, WAITING);
        let mut g = Game::new(4);
        g.tune[T_COMMISSIONS] = 0.0;
        g.start(M_ENDLESS);
        play(&mut g, FIRST + 5.0, None);
        assert_eq!(g.commission.phase, WAITING);
    }
}
