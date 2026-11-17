//! The height of a route (極致, docs/design.md 4.8.5): an ultimate that its
//! element turns into something more.
//!
//! An art reaches its height when its route is whole (every move learned and
//! open), it carries an element, and the player holds the inner art the route
//! names (`zenith` on `[[route]]`). From then on its ultimate lands half as
//! hard again, and every hit of it does what the element does at its height:
//!   metal      the guard fills and blows are halved for 2 s, when it is used
//!   wood       each hit heals 1
//!   water      each enemy hit surges together, half a second later
//!   fire       each burn spreads to the nearest enemy once
//!   earth      each enemy hit is held for 2 s, however recently it was held
//!   lightning  each hit jumps to two more enemies
//!   holy       each corpse or spirit hit that does not come alone falls at once
//! Without routes nothing here runs.

use super::Game;
use crate::defs::*;

/// How much harder an ultimate at its height lands.
const HEIGHT: f32 = 1.5;
/// Seconds blows are halved after a metal height.
const BULWARK: f32 = 2.0;

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

    /// An ultimate at its height is used: what its element does once, and the call-out.
    pub(crate) fn reach_height(&mut self, m: usize, element: u8) {
        if ELEMENTS[element as usize].effect == EF_GUARD {
            let cap = ELEMENTS[element as usize].cap;
            let p = &mut self.player;
            p.guard = cap;
            p.guard_element = element;
            p.guard_idle = ELEMENTS[element as usize].time;
            self.bulwark = BULWARK;
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
            EF_BANE => {
                let k = &KINDS[self.enemies[i].kind as usize];
                if k.corpse && k.every <= 0.0 {
                    let hp = self.enemies[i].hp;
                    self.hurt_enemy_as(i, hp.max(0.0) + 1.0, 0.0, 0.0, el.color);
                }
            }
            _ => {}
        }
    }
}
