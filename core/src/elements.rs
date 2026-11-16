//! Magic elements (魔法) and the reactions between them.
//!
//! A martial art that carries an element adds that element's effect to every
//! hit it lands (`Game::strike`). Most effects leave a tag on the enemy; when
//! a hit leaves a tag on an enemy that already holds the other tag of a
//! reaction, the reaction fires and both tags end (`Game::react`). The
//! elements, the reactions and all their numbers come from `[[element]]` and
//! `[[reaction]]` in `content/game.toml`; this file holds one piece of code
//! per effect and per reaction kind.
//!
//! Damage dealt by an effect or a reaction is plain: it leaves no tag, so a
//! reaction never sets off another one by itself.

use super::{Fx, Game, TICK};
use crate::defs::*;

/// The element tags on one enemy. Every array is indexed by element id.
#[derive(Clone, Copy, Default)]
pub struct Tags {
    /// Seconds the tag has left; 0 for no tag.
    pub left: [f32; N_ELEMENTS + 1],
    /// What the tag will still do: a burn's damage per second, or the damage
    /// of the surge that is waiting.
    pub amount: [f32; N_ELEMENTS + 1],
    /// Seconds before the element's effect can take hold of this enemy again.
    pub rest: [f32; N_ELEMENTS + 1],
}

impl Tags {
    /// Whether an element with the root effect is holding the enemy in place.
    pub fn rooted(&self) -> bool {
        (1..=N_ELEMENTS).any(|k| ELEMENTS[k].effect == EF_ROOT && self.left[k] > 0.0)
    }

    /// End the tag of element `k` and whatever it was still going to do.
    fn end(&mut self, k: usize) {
        self.left[k] = 0.0;
        self.amount[k] = 0.0;
    }
}

/// The most enemies one chain or relay can reach.
const MOST_JUMPS: usize = 8;

impl Game {
    /// Land a hit of a martial art on enemy `i`, with the effect of the
    /// element the art carries (an `E_*` id; 0 for none).
    pub(crate) fn strike(&mut self, i: usize, dmg: f32, kx: f32, ky: f32, element: u8) {
        if element == 0 || element as usize > N_ELEMENTS {
            self.hurt_enemy(i, dmg, kx, ky);
            return;
        }
        let k = element as usize;
        let el = &ELEMENTS[k];
        let (mut dmg, mut kx, mut ky) = (dmg, kx, ky);
        let mut rooting = false;
        match el.effect {
            EF_BANE => {
                let corpse = KINDS[self.enemies[i].kind as usize].corpse;
                dmg *= 1.0 + if corpse { el.power } else { el.minor };
            }
            EF_ROOT if self.enemies[i].tags.rest[k] <= 0.0 => {
                // Thrown straight back from the player, then held where it lands.
                let e = &self.enemies[i];
                let (dx, dy) = (e.x - self.player.x, e.y - self.player.y);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                kx += dx / d * el.power;
                ky += dy / d * el.power;
                rooting = true;
            }
            _ => {}
        }
        self.hurt_enemy(i, dmg, kx, ky);
        match el.effect {
            EF_GUARD => {
                let p = &mut self.player;
                p.guard = (p.guard + el.power).min(el.cap);
                p.guard_idle = el.time;
                p.guard_element = element;
            }
            EF_REGROW => {
                self.player.regrow = el.time;
                self.player.regrow_element = element;
            }
            EF_SURGE => {
                let t = &mut self.enemies[i].tags;
                // A surge already waiting keeps its time, so hitting again cannot put it off.
                if t.left[k] <= 0.0 {
                    t.left[k] = el.time;
                }
                t.amount[k] = t.amount[k].max(dmg * el.power);
            }
            EF_BURN => {
                let t = &mut self.enemies[i].tags;
                t.left[k] = el.time;
                t.amount[k] = dmg * el.power;
            }
            EF_ROOT => {
                if rooting {
                    let t = &mut self.enemies[i].tags;
                    t.left[k] = el.time;
                    t.rest[k] = el.time + el.rest;
                }
            }
            EF_CHAIN => {
                self.enemies[i].tags.left[k] = el.time;
                let (targets, n) = self.nearest(i, el.reach, el.count);
                for &j in targets.iter().take(n) {
                    self.hurt_enemy_as(j, dmg * el.power, 0.0, 0.0, el.color);
                    self.enemies[j].tags.left[k] = el.time;
                    let (x, y) = (self.enemies[j].x, self.enemies[j].y);
                    self.pop(x, y, 26.0, el.color);
                }
                // The enemies it jumped to hold the tag now, so they can react too.
                for &j in targets.iter().take(n) {
                    self.react(j, element, dmg * el.power);
                }
            }
            EF_BANE => self.enemies[i].tags.left[k] = el.time,
            _ => {}
        }
        if el.tags {
            self.react(i, element, dmg);
        }
    }

    /// Fire the first reaction between the tag just left on enemy `i` and a
    /// tag it already held. `dmg` is the damage of the hit that left the tag.
    fn react(&mut self, i: usize, applied: u8, dmg: f32) {
        for (id, r) in REACTIONS.iter().enumerate() {
            let other = if r.a == applied {
                r.b
            } else if r.b == applied {
                r.a
            } else {
                continue;
            };
            let (a, b) = (applied as usize, other as usize);
            if self.enemies[i].tags.left[a] <= 0.0 || self.enemies[i].tags.left[b] <= 0.0 {
                continue;
            }
            let (x, y) = (self.enemies[i].x, self.enemies[i].y);
            match r.kind {
                RK_RELAY => {
                    // The surge that was waiting goes off now, here and on the enemies nearby.
                    let surge = if ELEMENTS[a].effect == EF_SURGE { a } else { b };
                    let amount = self.enemies[i].tags.amount[surge] * r.power;
                    self.enemies[i].tags.end(a);
                    self.enemies[i].tags.end(b);
                    let (targets, n) = self.nearest(i, r.reach, r.count);
                    self.hurt_enemy_as(i, amount, 0.0, 0.0, r.color);
                    self.pop(x, y, 40.0, r.color);
                    for &j in targets.iter().take(n) {
                        self.hurt_enemy_as(j, amount, 0.0, 0.0, r.color);
                        let (jx, jy) = (self.enemies[j].x, self.enemies[j].y);
                        self.pop(jx, jy, 40.0, r.color);
                    }
                }
                _ => {
                    self.enemies[i].tags.end(a);
                    self.enemies[i].tags.end(b);
                    let amount = dmg * r.power;
                    for j in 0..self.enemies.len() {
                        let e = &self.enemies[j];
                        let (dx, dy) = (e.x - x, e.y - y);
                        let reach = r.reach + KINDS[e.kind as usize].r;
                        if e.hp > 0.0 && dx * dx + dy * dy < reach * reach {
                            let d = (dx * dx + dy * dy).sqrt().max(0.001);
                            self.hurt_enemy_as(j, amount, dx / d * 120.0, dy / d * 120.0, r.color);
                        }
                    }
                    self.pop(x, y, r.reach, r.color);
                    self.shake = self.shake.max(0.25);
                }
            }
            self.event(EV_REACT, x, y, id as f32);
            return;
        }
    }

    /// The living enemies nearest to enemy `i` within `reach`, nearest first,
    /// `count` at most. Returns their indices and how many there are.
    fn nearest(&self, i: usize, reach: f32, count: usize) -> ([usize; MOST_JUMPS], usize) {
        let count = count.min(MOST_JUMPS);
        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
        let mut best = [(f32::MAX, 0usize); MOST_JUMPS];
        let mut found = 0;
        for (j, e) in self.enemies.iter().enumerate() {
            if j == i || e.hp <= 0.0 {
                continue;
            }
            let d = (e.x - x) * (e.x - x) + (e.y - y) * (e.y - y);
            if d > reach * reach {
                continue;
            }
            for s in 0..count {
                if d < best[s].0 {
                    for t in (s + 1..count).rev() {
                        best[t] = best[t - 1];
                    }
                    best[s] = (d, j);
                    found = (found + 1).min(count);
                    break;
                }
            }
        }
        let mut out = [0usize; MOST_JUMPS];
        for s in 0..found {
            out[s] = best[s].1;
        }
        (out, found)
    }

    /// Count the tags down and do what they do in the meantime: burns tick,
    /// and a surge goes off when its time is up.
    pub(crate) fn tick_tags(&mut self, dt: f32) {
        for i in 0..self.enemies.len() {
            for k in 1..=N_ELEMENTS {
                let t = &mut self.enemies[i].tags;
                t.rest[k] = (t.rest[k] - dt).max(0.0);
                let before = t.left[k];
                if before <= 0.0 {
                    continue;
                }
                let after = before - dt;
                let amount = t.amount[k];
                t.left[k] = after.max(0.0);
                if after <= 0.0 {
                    t.amount[k] = 0.0;
                }
                let el = &ELEMENTS[k];
                match el.effect {
                    EF_BURN => {
                        // One tick each time the time left comes down to a multiple of the
                        // rate, the last one as it runs out.
                        let rate = el.rate.max(TICK);
                        if (before / rate).ceil() != (after / rate).ceil() {
                            self.hurt_enemy_as(i, amount * rate, 0.0, 0.0, el.color);
                        }
                    }
                    EF_SURGE if after <= 0.0 => {
                        self.hurt_enemy_as(i, amount, 0.0, 0.0, el.color);
                        let (x, y) = (self.enemies[i].x, self.enemies[i].y);
                        self.pop(x, y, 34.0, el.color);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Count the player's guard and slow heal down.
    pub(crate) fn tick_boons(&mut self, dt: f32) {
        let p = &mut self.player;
        if p.guard > 0.0 {
            p.guard_idle -= dt;
            if p.guard_idle <= 0.0 {
                p.guard = (p.guard - ELEMENTS[p.guard_element as usize].rate * dt).max(0.0);
            }
        }
        if p.regrow > 0.0 {
            p.regrow = (p.regrow - dt).max(0.0);
            p.hp = (p.hp + ELEMENTS[p.regrow_element as usize].power * dt).min(p.max_hp);
        }
    }

    /// Let the player's guard take its share of a blow. Returns the damage left for his health.
    pub(crate) fn guard_against(&mut self, dmg: f32) -> f32 {
        let p = &mut self.player;
        if p.guard <= 0.0 {
            return dmg;
        }
        // The guard only ever takes part of a blow, so it softens hits without making him untouchable.
        let share = ELEMENTS[p.guard_element as usize].minor.clamp(0.0, 1.0);
        let taken = (dmg * share).min(p.guard);
        p.guard -= taken;
        dmg - taken
    }

    /// The most guard the player can hold now; 0 while he has none.
    pub(crate) fn guard_cap(&self) -> f32 {
        if self.player.guard > 0.0 {
            ELEMENTS[self.player.guard_element as usize].cap
        } else {
            0.0
        }
    }

    /// A ring that opens to `radius` and fades, in an element's or a reaction's colour.
    fn pop(&mut self, x: f32, y: f32, radius: f32, color: [f32; 3]) {
        self.add_fx(Fx {
            x,
            y: y - 12.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.3,
            max: 0.3,
            part: P_RING,
            hw: radius * 0.25,
            hh: radius * 0.25,
            rot: 0.0,
            vr: 0.0,
            grow: radius * 0.75 / 0.3,
            color: [color[0], color[1], color[2], 0.85],
            seed: 0.4,
            top: true,
        });
    }

    /// A dot above an enemy for each tag it holds, in the element's colour.
    pub(crate) fn draw_tags(&mut self, i: usize, top: f32) {
        let e = &self.enemies[i];
        let (x, tags) = (e.x, e.tags);
        let held = (1..=N_ELEMENTS).filter(|&k| tags.left[k] > 0.0).count();
        let mut slot = 0;
        for k in 1..=N_ELEMENTS {
            if tags.left[k] <= 0.0 {
                continue;
            }
            let c = ELEMENTS[k].color;
            let dx = (slot as f32 - (held as f32 - 1.0) * 0.5) * 11.0;
            self.put(x + dx, top - 9.0, 4.5, 4.5, 0.0, P_ELLIPSE, 0.0, 0.0, [c[0], c[1], c[2], 1.0]);
            slot += 1;
        }
    }

    /// A ring around the player while he holds guard, fainter as it wears off.
    pub(crate) fn draw_guard(&mut self) {
        let p = &self.player;
        if p.guard <= 0.0 {
            return;
        }
        let el = &ELEMENTS[p.guard_element as usize];
        let share = if el.cap > 0.0 { (p.guard / el.cap).min(1.0) } else { 1.0 };
        let (x, y) = (p.x, p.y - 30.0);
        self.put(x, y, 38.0, 38.0, 0.0, P_RING, 0.0, 0.6, [el.color[0], el.color[1], el.color[2], 0.25 + 0.55 * share]);
    }
}
