//! The height of a route (極致, docs/design.md 4.8.5): an ultimate that its
//! element turns into something more.
//!
//! An art reaches its height when its route is whole (every move learned and
//! open), it carries an element, and the player holds the inner art the route
//! names (`zenith` on `[[route]]`). From then on its ultimate lands half as
//! hard again (a rule of this code, not of docs/martial-arts.md), and every
//! hit of it does what the element does at its height:
//!   metal      the guard fills and blows are halved for 2 s, when it is used
//!   wood       each hit heals 1
//!   water      each enemy hit surges together, half a second later
//!   fire       each burn spreads to the nearest enemy once
//!   earth      each enemy hit is held for 2 s, however recently it was held
//!   lightning  each hit jumps to two more enemies
//!   holy       a sigil (符陣) over the ultimate's reach, for the element's
//!              time: corpses and spirits that do not come alone fall in it at
//!              once, and every blow on the others inside lands harder by the
//!              element's `minor`
//! Without routes nothing here runs.

use super::Game;
use crate::defs::*;

/// How much harder an ultimate at its height lands.
const HEIGHT: f32 = 1.5;
/// Seconds blows are halved after a metal height.
const BULWARK: f32 = 2.0;

/// The sigil (符陣) of a holy ultimate at its height, on the ground where it was used.
#[derive(Clone, Copy)]
pub struct Sigil {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    /// Seconds before it fades.
    pub life: f32,
    pub element: u8,
}

impl Game {
    /// The element art `a`'s ultimate reaches its height with, if it does now.
    pub(crate) fn zenith_of(&self, a: usize) -> Option<u8> {
        let r = self.route_of(a)?;
        let def = &ROUTES[r];
        let whole = self.moves_open(r) == def.n && self.art_level(a) >= 2 + def.n;
        let ultimate = MOVES[def.first + def.n - 1].ultimate;
        let element = self.arts[a].element;
        let inner = def.zenith < N_UPGRADES && self.layer(def.zenith) > 0;
        (whole && ultimate && element > 0 && inner).then_some(element)
    }

    /// The damage factor of a hit while an ultimate at its height lands.
    pub(crate) fn height_power(&self) -> f32 {
        if self.zenith > 0 {
            HEIGHT
        } else {
            1.0
        }
    }

    /// Ultimate `m` of art `a` at its height is used: what its element does once, and the call-out.
    pub(crate) fn reach_height(&mut self, a: usize, m: usize, element: u8) {
        let el = &ELEMENTS[element as usize];
        match el.effect {
            EF_GUARD => {
                let (cap, time) = (el.cap, el.time);
                let p = &mut self.player;
                p.guard = cap;
                p.guard_element = element;
                p.guard_idle = time;
                self.bulwark = BULWARK;
            }
            EF_BANE => {
                let (x, y, r) = self.move_span(a, m);
                self.inner.sigil = Some(Sigil { x, y, r, life: el.time.max(0.5), element });
                // Whatever corpse stands in it falls before the ultimate lands.
                self.sigil_fells();
            }
            _ => {}
        }
        // The value packs the move and the element: move + 256 × element.
        self.event(EV_ZENITH, self.player.x, self.player.y, (m + 256 * element as usize) as f32);
    }

    /// A hit of an ultimate at its height landed on enemy `i` with `dmg`.
    pub(crate) fn height_hit(&mut self, i: usize, dmg: f32) {
        let element = self.zenith as usize;
        if element == 0 || element > N_ELEMENTS || i >= self.enemies.len() {
            return;
        }
        let el = &ELEMENTS[element];
        match el.effect {
            EF_REGROW => {
                let p = &mut self.player;
                p.hp = (p.hp + 1.0).min(p.max_hp);
            }
            EF_SURGE => {
                let t = &mut self.enemies[i].tags;
                if t.left[element] > 0.5 {
                    t.left[element] = 0.5;
                }
            }
            EF_BURN => {
                let (amount, time) = (self.enemies[i].tags.amount[element], el.time);
                let (near, n) = self.nearest(i, 120.0, 1);
                for &j in near.iter().take(n) {
                    let t = &mut self.enemies[j].tags;
                    t.left[element] = time;
                    t.amount[element] = amount;
                }
            }
            EF_ROOT => {
                let t = &mut self.enemies[i].tags;
                t.rest[element] = 0.0;
                t.left[element] = t.left[element].max(2.0);
            }
            EF_CHAIN => {
                let (near, n) = self.nearest(i, el.reach, 2);
                for &j in near.iter().take(n) {
                    self.hurt_enemy_as(j, dmg * el.power, 0.0, 0.0, el.color);
                }
            }
            _ => {}
        }
    }

    /// Where move `m` of art `a` reaches from where the player stands: the
    /// centre and the radius of the ground it covers.
    fn move_span(&self, a: usize, m: usize) -> (f32, f32, f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let (px, py) = (self.player.x, self.player.y);
        let (ax, ay) = self.player.aim;
        let last = def.count.max(1) as f32 - 1.0;
        let melee = n.reach * def.reach * (1.0 + def.grow.max(0.0) * last) * self.melee_reach() * self.kind_reach(a);
        let (x, y, r) = match def.shape {
            MS_WAVE => (px, py, def.speed * def.life),
            MS_TURRET => (px, py, (def.offset * 100.0).max(def.width) + 60.0),
            MS_RING => {
                let ahead = n.reach * def.offset;
                (px + ax * ahead, py + ay * ahead, melee)
            }
            MS_ARC | MS_THRUST => (px, py, melee),
            // Marks: a line of them ahead, the farthest `count` gaps away.
            _ => (px, py, n.reach * def.spread * (last + 1.0) + n.reach * def.reach),
        };
        // Never so small that it holds no enemy, nor so large that it covers the whole field.
        (x, y, r.clamp(60.0, 900.0))
    }

    /// How much harder a blow on enemy `i` lands for standing in a sigil; exactly 1 outside one.
    pub(crate) fn sigil_power(&self, i: usize) -> f32 {
        match &self.inner.sigil {
            Some(s) if self.in_sigil(s, i) => 1.0 + ELEMENTS[s.element as usize].minor,
            _ => 1.0,
        }
    }

    fn in_sigil(&self, s: &Sigil, i: usize) -> bool {
        let e = &self.enemies[i];
        let (dx, dy) = (e.x - s.x, e.y - s.y);
        let r = s.r + KINDS[e.kind as usize].r;
        dx * dx + dy * dy < r * r
    }

    /// Every corpse or spirit in the sigil that is not a boss, an elite or a
    /// master (those come alone, on their own clock) falls at once.
    fn sigil_fells(&mut self) {
        let Some(s) = self.inner.sigil else { return };
        let color = ELEMENTS[s.element as usize].color;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let k = &KINDS[e.kind as usize];
            if e.hp <= 0.0 || !k.corpse || k.every > 0.0 || self.boss.is(e.id) || !self.in_sigil(&s, i) {
                continue;
            }
            let hp = e.hp;
            self.hurt_enemy_as(i, hp + 1.0, 0.0, 0.0, color);
        }
    }

    /// One step of the sigil: what walks into it falls too, until it fades.
    pub(crate) fn tick_sigil(&mut self, dt: f32) {
        let Some(s) = &mut self.inner.sigil else { return };
        s.life -= dt;
        if s.life <= 0.0 {
            self.inner.sigil = None;
            return;
        }
        self.sigil_fells();
    }

    /// Draw the sigil: two rings of the element's colour on the ground, fading as it ends.
    pub(crate) fn draw_sigil(&mut self) {
        let Some(s) = self.inner.sigil else { return };
        let c = ELEMENTS[s.element as usize].color;
        let fade = s.life.min(1.0);
        let color = [c[0], c[1], c[2], 0.7 * fade];
        self.put(s.x, s.y, s.r, s.r, 0.0, P_RING, 0.0, 0.3, color);
        self.put(s.x, s.y, s.r * 0.82, s.r * 0.82, self.time, P_RING, 0.0, 0.31, color);
    }
}
