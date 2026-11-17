//! Routes (套路): what a martial art turns into from level 3 (docs/design.md 4.8.3).
//!
//! Which routes exist, their moves and every number come from `[[route]]` and
//! `[[move]]` in `content/game.toml`, through `ROUTES` and `MOVES`. With the
//! `routes` tune on, an art that reaches level 3 takes a route, and from then
//! on uses one move per beat, in turn: its basic move and the moves learned so
//! far, then, once every move is learned, the moves alone and the ultimate
//! last. Off, nothing here runs and the arts grow as they did before.
//!
//! Moves are measured against the art's own numbers at level 2: a move's
//! `power` is a share of that damage and its `reach` a share of that reach.

use super::{Fx, Game, Shot, MAX_SHOTS};
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// The colour of a move while its art carries no element.
const JADE: [f32; 3] = [0.25, 0.62, 0.48];
/// A move order entry for the art's basic move.
pub const BASIC: u16 = u16::MAX;
/// The longest a move order can be: the basic move and eight moves.
pub const MOST_STEPS: usize = 9;
/// How many delayed hits can wait at once.
const MAX_DELAYED: usize = 64;

/// A hit that lands a moment after its move: a later sweep of an arc, or a mark.
#[derive(Clone, Copy)]
pub struct Delayed {
    /// Seconds until it lands.
    pub left: f32,
    pub art: u8,
    pub mv: u16,
    /// Which sweep or mark of the move it is, from 0.
    pub k: u8,
    /// Where a mark lands; unused by a sweep, which lands where the player is then.
    pub x: f32,
    pub y: f32,
    /// The aim when the move began.
    pub ax: f32,
    pub ay: f32,
}

impl Game {
    /// Whether routes are on for this run.
    pub(crate) fn routes_on(&self) -> bool {
        self.tune[T_ROUTES] > 0.5
    }

    /// The routes an art can take now, in id order, at most three.
    pub(crate) fn routes_of(a: usize) -> impl Iterator<Item = usize> {
        (0..N_ROUTES).filter(move |&r| ROUTES[r].art == a && ROUTES[r].start).take(3)
    }

    /// The art an upgrade levels, if it is one.
    pub(crate) fn art_of_upgrade(id: usize) -> Option<usize> {
        ARTS.iter().position(|art| art.upgrade == id)
    }

    /// The route an art follows, if any.
    pub(crate) fn route_of(&self, a: usize) -> Option<usize> {
        match self.arts[a].route {
            0 => None,
            r => Some(r as usize - 1),
        }
    }

    /// The highest level an upgrade can reach in this run. An art on a route
    /// reaches 2 + the route's moves; anything else its own top.
    pub(crate) fn upgrade_top(&self, id: usize) -> u8 {
        if let Some(r) = Self::art_of_upgrade(id).and_then(|a| self.route_of(a)) {
            return (2 + ROUTES[r].n) as u8;
        }
        // An inner art has three layers.
        if self.inner_on() && Self::is_passive(id) {
            return UPGRADE_MAX[id].min(3);
        }
        UPGRADE_MAX[id]
    }

    /// The art whose route must be chosen before this upgrade can take its
    /// next level: one at level 2 with routes on and a route to take.
    pub(crate) fn route_needed(&self, id: usize) -> Option<usize> {
        let a = Self::art_of_upgrade(id)?;
        let waiting = self.routes_on() && self.up[id] == 2 && self.arts[a].route == 0;
        (waiting && Self::routes_of(a).next().is_some()).then_some(a)
    }

    /// Put art `a` on route `r` (an index into `ROUTES`), starting its move order over.
    pub(crate) fn take_route(&mut self, a: usize, r: usize) {
        self.arts[a].route = r as u8 + 1;
        self.arts[a].step = 0;
        self.arts[a].timer = self.arts[a].timer.min(ROUTES[r].beat);
    }

    /// The art's move order now: `BASIC` or move ids, and how many there are.
    /// Empty for an art without a route or below level 3.
    pub(crate) fn move_order(&self, a: usize) -> ([u16; MOST_STEPS], usize) {
        let mut order = [BASIC; MOST_STEPS];
        let (Some(r), level) = (self.route_of(a), self.art_level(a)) else { return (order, 0) };
        if level < 3 {
            return (order, 0);
        }
        let def = &ROUTES[r];
        let ultimate = MOVES[def.first + def.n - 1].ultimate as usize;
        let regular = def.n - ultimate;
        let learned = (level - 2).min(def.n);
        let mut len = 0;
        if learned < regular {
            // Still learning: the basic move opens the order.
            len = 1;
            for m in 0..learned {
                order[len] = (def.first + m) as u16;
                len += 1;
            }
        } else {
            for m in 0..learned {
                order[len] = (def.first + m) as u16;
                len += 1;
            }
        }
        (order, len)
    }

    /// One beat of an art on a route: when its timer runs out, use the next move.
    pub(crate) fn route_beat(&mut self, a: usize, dt: f32, might: f32, haste: f32) {
        let Some(r) = self.route_of(a) else { return };
        self.arts[a].timer -= dt;
        if self.arts[a].timer > 0.0 {
            return;
        }
        self.arts[a].timer = ROUTES[r].beat * haste * self.beat_factor();
        let (order, len) = self.move_order(a);
        if len == 0 {
            return;
        }
        let step = self.arts[a].step as usize % len;
        self.arts[a].step = ((step + 1) % len) as u8;
        // The last move of the order closes it: the mountain lifter makes it
        // land harder, and piercing qi sends a blade after it.
        let last = step + 1 == len && len > 1;
        let might = if last { might * self.finisher() } else { might };
        match order[step] {
            BASIC => self.basic_move(a, might),
            m => {
                self.use_move(a, m as usize, might);
                if MOVES[m as usize].ultimate {
                    self.note_ultimate();
                }
            }
        }
        if last {
            self.qi_blade(a, Self::basic_numbers(a).damage * might);
        }
    }

    /// The art's numbers at level 2, which its moves are measured against.
    fn basic_numbers(a: usize) -> &'static ArtLevel {
        let levels = ARTS[a].levels;
        &levels[1.min(levels.len() - 1)]
    }

    /// The art's own move at level 2, as one step of its order. Circling
    /// beads circle all the time, so for them this step is a rest. A route
    /// that beats faster than the art's own cooldown uses it more often, so
    /// it hits and pushes that much more lightly.
    fn basic_move(&mut self, a: usize, might: f32) {
        let n = Self::basic_numbers(a);
        let (px, py) = (self.player.x, self.player.y);
        let light = self.route_of(a).map_or(1.0, |r| (ROUTES[r].beat / n.cooldown).min(1.0));
        match ARTS[a].behaviour {
            B_ARC => {
                let (ax, ay) = self.player.aim;
                self.sweep(a, n.damage * might * light, n.reach, n.width, n.knockback * light, ax, ay);
                self.event(EV_SLASH, px, py, 0.0);
            }
            B_PROJECTILE => {
                self.shoot_nearest(a, n, might * light);
            }
            B_NOVA => {
                self.ring(a, px, py, n.reach, n.damage * might * light, n.knockback * light);
                self.event(EV_PULSE, px, py, 0.0);
            }
            _ => {}
        }
    }

    /// Use move `m` of art `a`.
    fn use_move(&mut self, a: usize, m: usize, might: f32) {
        let def = &MOVES[m];
        let (px, py) = (self.player.x, self.player.y);
        let (ax, ay) = self.player.aim;
        match def.shape {
            MS_ARC => {
                self.arc_sweep(a, m, 0, ax, ay, might);
                for k in 1..def.count.min(8) {
                    if def.delay > 0.0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.arc_sweep(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_SLASH, px, py, 0.0);
            }
            MS_WAVE => {
                // A seeking move aims at the nearest enemy, or along the aim when none is near.
                let (ax, ay) = if def.seek > 0 { self.toward_nearest().unwrap_or((ax, ay)) } else { (ax, ay) };
                for k in 0..def.count.clamp(1, 32) {
                    if def.delay > 0.0 && k > 0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.wave(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_SHOOT, px, py, 0.0);
            }
            MS_RING => {
                self.ring_of(a, m, 0, ax, ay, might);
                for k in 1..def.count.min(8) {
                    if def.delay > 0.0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.ring_of(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_PULSE, px, py, 0.0);
            }
            _ => {
                // Marks: a line of them ahead, each striking a moment later.
                let gap = Self::basic_numbers(a).reach * def.spread;
                let color = self.art_color(a, JADE, 0.6);
                for k in 0..def.count.min(8) {
                    let d = gap * (k + 1) as f32;
                    let (x, y) = (px + ax * d, py + ay * d);
                    self.delay_hit(a, m, k, def.delay, x, y, ax, ay);
                    self.mark_fx(a, x, y, def.delay, color);
                }
                self.event(EV_SLASH, px, py, 0.0);
            }
        }
        self.event(EV_MOVE, px, py, m as f32);
    }

    /// Sweep `k` of an arc move, where the player stands now.
    fn arc_sweep(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        // Each later sweep turns by `spread` and reaches `grow` further.
        let turn = def.spread * TAU * k as f32;
        let (c, s) = (fmath::cos(turn), fmath::sin(turn));
        let (dx, dy) = (ax * c - ay * s, ax * s + ay * c);
        let reach = n.reach * def.reach * (1.0 + def.grow * k as f32) * self.melee_reach();
        let dmg = n.damage * def.power * might;
        self.sweep(a, dmg, reach, def.width, n.knockback * def.knockback, dx, dy);
    }

    /// One sweep around the player towards (ax, ay): everything within `reach`
    /// and inside the `width` bound is hit and pushed away.
    #[allow(clippy::too_many_arguments)]
    fn sweep(&mut self, a: usize, dmg: f32, reach: f32, width: f32, knockback: f32, ax: f32, ay: f32) {
        let element = self.arts[a].element;
        let (px, py) = (self.player.x, self.player.y);
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - px, e.y - py);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let r = KINDS[e.kind as usize].r;
            if d < reach + r && (d < 30.0 || (dx * ax + dy * ay) / d > width) {
                self.strike(i, dmg, dx / d * knockback, dy / d * knockback, element);
            }
        }
        self.formation_hit_arc(px, py, ax, ay, reach, width, dmg);
        self.player.attack = 0.25;
        let rot = fmath::atan2(ay, ax);
        let color = self.art_color(a, JADE, 0.9);
        // A sweep all round draws as two half swings back to back.
        let marks = if width < -1.0 { 2 } else { 1 };
        for k in 0..marks {
            self.add_fx(Fx {
                x: px,
                y: py - 24.0,
                vx: 0.0,
                vy: 0.0,
                life: 0.22,
                max: 0.22,
                part: P_SLASH,
                hw: reach,
                hh: reach,
                rot: rot + k as f32 * TAU * 0.5,
                vr: 5.0,
                grow: 60.0,
                color,
                seed: 0.3,
                top: true,
            });
        }
    }

    /// The way to the nearest enemy, if there is one.
    fn toward_nearest(&self) -> Option<(f32, f32)> {
        let (px, py) = (self.player.x, self.player.y);
        let mut best: Option<(f32, f32, f32)> = None;
        for e in &self.enemies {
            let (dx, dy) = (e.x - px, e.y - py);
            let d2 = dx * dx + dy * dy;
            if best.is_none_or(|(b, _, _)| d2 < b) {
                best = Some((d2, dx, dy));
            }
        }
        best.map(|(d2, dx, dy)| {
            let d = d2.sqrt().max(0.001);
            (dx / d, dy / d)
        })
    }

    /// Wave `k` of a wave move, its place in the fan around (ax, ay).
    fn wave(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        if self.shots.len() >= MAX_SHOTS {
            return;
        }
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let (px, py) = (self.player.x, self.player.y);
        let count = def.count.clamp(1, 32);
        // A fan of a whole turn has no ends, so its waves sit evenly round.
        let off = if count == 1 || def.delay > 0.0 {
            0.0
        } else if def.spread >= 1.0 {
            k as f32 / count as f32 * TAU
        } else {
            (k as f32 / (count - 1) as f32 - 0.5) * def.spread * TAU
        };
        let angle = fmath::atan2(ay, ax) + off;
        let (dx, dy) = (fmath::cos(angle), fmath::sin(angle));
        self.shots.push(Shot {
            x: px,
            y: py,
            vx: dx * def.speed,
            vy: dy * def.speed,
            life: def.life,
            dmg: n.damage * def.power * might,
            pierce: def.pierce.max(1),
            last: 0,
            art: a as u8,
            push: def.knockback,
            split: def.split.min(8) as u8,
            grow: def.grow,
            size: def.power.sqrt().clamp(0.7, 2.5),
        });
    }

    /// Shot `s` reached the end of its flight: it bursts into smaller shots
    /// fanned around the way it flew, each doing half its damage.
    pub(crate) fn burst_shot(&mut self, s: usize) {
        let sh = &self.shots[s];
        let (x, y, vx, vy, n, dmg, art) = (sh.x, sh.y, sh.vx, sh.vy, sh.split as usize, sh.dmg * 0.5, sh.art);
        let speed = (vx * vx + vy * vy).sqrt() * 0.8;
        let base = fmath::atan2(vy, vx);
        for k in 0..n {
            if self.shots.len() >= MAX_SHOTS {
                break;
            }
            let off = if n == 1 { 0.0 } else { (k as f32 / (n - 1) as f32 - 0.5) * TAU * 0.25 };
            let angle = base + off;
            self.shots.push(Shot {
                x,
                y,
                vx: fmath::cos(angle) * speed,
                vy: fmath::sin(angle) * speed,
                life: 0.35,
                dmg,
                pierce: 1,
                last: 0,
                art,
                push: 0.2,
                split: 0,
                grow: 0.0,
                size: 0.6,
            });
        }
    }

    /// Ring `k` of a ring move: centred `offset` ahead along (ax, ay) from where
    /// the player stands now, each later ring reaching `grow` further.
    fn ring_of(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let ahead = n.reach * def.offset;
        let (cx, cy) = (self.player.x + ax * ahead, self.player.y + ay * ahead);
        let radius = n.reach * def.reach * (1.0 + def.grow * k as f32) * self.melee_reach();
        self.ring(a, cx, cy, radius, n.damage * def.power * might, n.knockback * def.knockback);
    }

    /// Queue a hit of move `m` to land `left` seconds from now.
    #[allow(clippy::too_many_arguments)]
    fn delay_hit(&mut self, a: usize, m: usize, k: usize, left: f32, x: f32, y: f32, ax: f32, ay: f32) {
        if self.delayed.len() < MAX_DELAYED {
            self.delayed.push(Delayed { left, art: a as u8, mv: m as u16, k: k as u8, x, y, ax, ay });
        }
    }

    /// Count down the delayed hits and land the ones that are due, oldest first.
    pub(crate) fn tick_delayed(&mut self, dt: f32) {
        if self.delayed.is_empty() {
            return;
        }
        let might = self.stat(ST_DAMAGE);
        let mut i = 0;
        while i < self.delayed.len() {
            self.delayed[i].left -= dt;
            if self.delayed[i].left > 0.0 {
                i += 1;
                continue;
            }
            let d = self.delayed.remove(i);
            let (a, m, k) = (d.art as usize, d.mv as usize, d.k as usize);
            match MOVES[m].shape {
                MS_ARC => self.arc_sweep(a, m, k, d.ax, d.ay, might),
                MS_WAVE => self.wave(a, m, k, d.ax, d.ay, might),
                MS_RING => self.ring_of(a, m, k, d.ax, d.ay, might),
                _ => self.mark_strike(a, m, d.x, d.y, might),
            }
        }
    }

    /// A mark of a marks move landing at (x, y).
    fn mark_strike(&mut self, a: usize, m: usize, x: f32, y: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let radius = n.reach * def.reach;
        let dmg = n.damage * def.power * might;
        let element = self.arts[a].element;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - x, e.y - y);
            let r = radius + KINDS[e.kind as usize].r;
            if dx * dx + dy * dy < r * r {
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                self.strike(i, dmg, dx / d * 80.0, dy / d * 80.0, element);
            }
        }
        self.formation_hit_circle(x, y, radius, dmg);
        let color = self.art_color(a, JADE, 0.9);
        self.add_fx(Fx {
            x,
            y: y - 10.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.2,
            max: 0.2,
            part: P_SLASH,
            hw: radius,
            hh: radius,
            rot: fmath::atan2(-1.0, 0.3),
            vr: 9.0,
            grow: 40.0,
            color,
            seed: 0.5,
            top: true,
        });
    }

    /// The scratch left by a mark until it strikes.
    fn mark_fx(&mut self, a: usize, x: f32, y: f32, life: f32, color: [f32; 4]) {
        let _ = a;
        self.add_fx(Fx {
            x,
            y: y - 4.0,
            vx: 0.0,
            vy: 0.0,
            life,
            max: life,
            part: P_SLASH,
            hw: 22.0,
            hh: 22.0,
            rot: 1.2,
            vr: 0.0,
            grow: 0.0,
            color,
            seed: 0.9,
            top: false,
        });
    }

    /// The words the hash takes from the routes, when any art follows one.
    pub(crate) fn routes_hash(&self, mix: &mut impl FnMut(u32)) {
        if self.arts.iter().all(|a| a.route == 0) && self.delayed.is_empty() && self.route_pick == 0 {
            return;
        }
        for art in &self.arts {
            mix(art.route as u32);
            mix(art.step as u32);
            mix(art.timer.to_bits());
        }
        mix(self.route_pick as u32);
        mix(self.delayed.len() as u32);
        for d in &self.delayed {
            mix(d.left.to_bits());
            mix(d.mv as u32);
        }
    }
}
