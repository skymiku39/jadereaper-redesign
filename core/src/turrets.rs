//! Mechanisms (機關, docs/martial-arts.md, the Tang clan's nine-mechanism
//! box): set down by a route move of the `turret` shape, they shoot by
//! themselves for a while.
//!
//! A turret move sets down `count` mechanisms: at the player, in a ring
//! `offset × 100` round him when there are several, or circling him at a
//! radius of `width` when `width` is above 0 (a wooden bird). Each one lives
//! `life` seconds, shoots at the nearest enemy within reach every `delay`
//! seconds (`speed`, `pierce` and `power` as a wave's), follows the player
//! with `seek = 1`, and bursts into `split` shots all round when it ends. At
//! most `MAX_TURRETS` stand at once; a new one takes the place of the oldest.

use super::{Game, Shot, MAX_SHOTS};
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// The most mechanisms at once.
pub const MAX_TURRETS: usize = 8;
/// How near an enemy must be for a mechanism to shoot at it.
const TURRET_RANGE: f32 = 420.0;
/// How fast a following mechanism walks, as a share of the player's base speed.
const FOLLOW: f32 = 0.6 * 190.0;
/// How fast a wooden bird circles, in radians a second.
const CIRCLE: f32 = 3.0;

/// How far past the ring of mechanisms a net reaches.
const NET_MARGIN: f32 = 20.0;

/// The net a ring of mechanisms stretches between them (天羅地網): enemies
/// inside walk slower while it stands.
#[derive(Clone, Copy)]
pub struct Net {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub life: f32,
    /// The share taken off an enemy's speed inside.
    pub slow: f32,
    pub art: u8,
}

/// A mechanism set down by a route move.
#[derive(Clone, Copy)]
pub struct Turret {
    pub x: f32,
    pub y: f32,
    /// Seconds before it ends.
    pub life: f32,
    /// Seconds before it shoots again.
    pub cd: f32,
    pub art: u8,
    pub mv: u16,
    /// Above 0, the radius it circles the player at, and where on the circle it is.
    pub orbit: f32,
    pub angle: f32,
    /// The damage of each of its shots.
    pub dmg: f32,
}

impl Game {
    /// Set down the mechanisms of turret move `m` of art `a`, with damage factor `might`.
    pub(crate) fn set_turrets(&mut self, a: usize, m: usize, might: f32) {
        let def = &MOVES[m];
        let n = &ARTS[a].levels[1.min(ARTS[a].levels.len() - 1)];
        let count = def.count.clamp(1, MAX_TURRETS);
        let dmg = n.damage * def.power * might;
        let (px, py) = (self.player.x, self.player.y);
        for k in 0..count {
            let angle = k as f32 / count as f32 * TAU;
            let (x, y) = if def.width > 0.0 || count == 1 {
                (px, py)
            } else {
                let r = def.offset * 100.0;
                (px + fmath::cos(angle) * r, py + fmath::sin(angle) * r * 0.8)
            };
            if self.turrets.len() >= MAX_TURRETS {
                self.turrets.remove(0);
            }
            self.turrets.push(Turret {
                x,
                y,
                life: def.life.max(0.5),
                cd: 0.2,
                art: a as u8,
                mv: m as u16,
                orbit: def.width.max(0.0),
                angle,
                dmg,
            });
        }
        if def.slow > 0.0 && count > 1 && def.width <= 0.0 {
            // The ring of mechanisms is a net.
            self.net = Some(Net {
                x: px,
                y: py,
                r: def.offset * 100.0 + NET_MARGIN,
                life: def.life.max(0.5),
                slow: def.slow.min(0.9),
                art: a as u8,
            });
        }
    }

    /// How much of its speed an enemy at (x, y) keeps: less inside a net.
    pub(crate) fn net_pace(net: &Option<Net>, x: f32, y: f32) -> f32 {
        match net {
            Some(n) => {
                let (dx, dy) = (x - n.x, (y - n.y) / 0.8);
                if dx * dx + dy * dy < n.r * n.r {
                    1.0 - n.slow
                } else {
                    1.0
                }
            }
            None => 1.0,
        }
    }

    /// One step of the net: it comes down when its time is up.
    pub(crate) fn tick_net(&mut self, dt: f32) {
        if let Some(n) = &mut self.net {
            n.life -= dt;
            if n.life <= 0.0 {
                self.net = None;
            }
        }
    }

    /// One step of the mechanisms: move, shoot, and burst at the end.
    pub(crate) fn tick_turrets(&mut self, dt: f32) {
        if self.turrets.is_empty() {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let mut i = 0;
        while i < self.turrets.len() {
            let mut t = self.turrets[i];
            let def = &MOVES[t.mv as usize];
            t.life -= dt;
            if t.orbit > 0.0 {
                t.angle += CIRCLE * dt;
                t.x = px + fmath::cos(t.angle) * t.orbit;
                t.y = py + fmath::sin(t.angle) * t.orbit * 0.8;
            } else if def.seek > 0 {
                let (dx, dy) = (px - t.x, py - t.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 60.0 {
                    t.x += dx / d * FOLLOW * dt;
                    t.y += dy / d * FOLLOW * dt;
                }
            }
            t.cd -= dt;
            if t.cd <= 0.0 {
                if let Some((dx, dy)) = self.nearest_to(t.x, t.y, TURRET_RANGE) {
                    t.cd = def.delay.max(0.15);
                    self.turret_shot(&t, dx, dy, t.dmg, def.pierce.max(1), def.speed.max(300.0));
                } else {
                    t.cd = 0.15;
                }
            }
            if t.life <= 0.0 {
                // It ends: a burst all round, if it is made to.
                for k in 0..def.split.min(24) {
                    let a = k as f32 / def.split as f32 * TAU;
                    self.turret_shot(&t, fmath::cos(a), fmath::sin(a), t.dmg, 1, 520.0);
                }
                self.turrets.remove(i);
                continue;
            }
            self.turrets[i] = t;
            i += 1;
        }
    }

    /// The way from (x, y) to the nearest enemy within `reach`, if any.
    fn nearest_to(&self, x: f32, y: f32, reach: f32) -> Option<(f32, f32)> {
        let mut best: Option<(f32, f32, f32)> = None;
        for e in &self.enemies {
            let (dx, dy) = (e.x - x, e.y - y);
            let d2 = dx * dx + dy * dy;
            if d2 < reach * reach && best.is_none_or(|(b, _, _)| d2 < b) {
                best = Some((d2, dx, dy));
            }
        }
        best.map(|(d2, dx, dy)| {
            let d = d2.sqrt().max(0.001);
            (dx / d, dy / d)
        })
    }

    /// A shot of mechanism `t` toward (dx, dy).
    fn turret_shot(&mut self, t: &Turret, dx: f32, dy: f32, dmg: f32, pierce: i32, speed: f32) {
        if self.shots.len() >= MAX_SHOTS {
            return;
        }
        self.shots.push(Shot {
            x: t.x,
            y: t.y,
            vx: dx * speed,
            vy: dy * speed,
            life: 0.8,
            dmg,
            pierce,
            last: 0,
            art: t.art,
            push: 0.15,
            split: 0,
            grow: 0.0,
            size: 0.7,
            zen: 0,
        });
    }

    /// Draw the mechanisms: a small paper box on the ground, or a bird in the air.
    pub(crate) fn draw_turrets(&mut self) {
        if let Some(n) = self.net {
            // The net: a ring of thread on the ground, fading as it ends.
            let color = self.art_color(n.art as usize, [0.55, 0.42, 0.26], 0.55 * n.life.min(1.0));
            self.put(n.x, n.y, n.r, n.r * 0.8, 0.0, P_RING, 0.0, 0.3, color);
            self.put(n.x, n.y, n.r * 0.6, n.r * 0.48, 0.0, P_RING, 0.0, 0.31, color);
        }
        for i in 0..self.turrets.len() {
            let t = self.turrets[i];
            let a = t.art as usize;
            let color = self.art_color(a, [0.55, 0.42, 0.26], 1.0);
            let lift = if t.orbit > 0.0 { 46.0 } else { 16.0 };
            let size = if t.orbit > 0.0 { 9.0 } else { 13.0 };
            self.put(t.x, t.y - lift, size, size * 1.3, t.angle * 0.2, ARTS[a].part, 0.0, 0.2, color);
        }
    }

    /// The words the hash takes from the mechanisms, when there are any.
    pub(crate) fn turrets_hash(&self, mix: &mut impl FnMut(u32)) {
        if let Some(n) = &self.net {
            mix(n.life.to_bits());
            mix(n.r.to_bits());
        }
        if self.turrets.is_empty() {
            return;
        }
        mix(self.turrets.len() as u32);
        for t in &self.turrets {
            mix(t.x.to_bits());
            mix(t.y.to_bits());
            mix(t.life.to_bits());
            mix(t.cd.to_bits());
        }
    }
}
