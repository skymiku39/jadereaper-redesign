//! The staff stretching (伸縮, docs/martial-arts.md, 翻江棍): a route move of
//! the `stretch` shape makes the circling beads of the staff reach out to
//! `reach` times as far and come back over `life` seconds. Meanwhile every
//! bead is the tip of a staff held from the player, which hits along its
//! whole length, each enemy once a stretch, instead of the beads' own hits.
//!
//! On top of that a stretch may turn the staff `speed` times faster (大江東去),
//! land a ring at the tip nearest the aim at its height (`offset`, 驚濤拍岸),
//! and push along its turn instead of outward (`side`) and slow the enemies it
//! hits (`slow`, 逆水行舟). Without a stretch nothing here runs, the beads are
//! where they always were (a factor of exactly 1, a turn of exactly 0), and the
//! hash takes nothing from here.

use super::Game;
use crate::defs::*;
use core::f32::consts::TAU;

/// How much of a stretch goes reaching out, and as much coming back; it holds in between.
const RAMP: f32 = 0.2;
/// The part of the staff near the hand that does not hit.
const GRIP: f32 = 30.0;

/// A stretch of the staff under way.
#[derive(Clone, Copy)]
pub struct Stretch {
    pub art: u8,
    pub mv: u16,
    /// Seconds since it began, and how long it lasts.
    pub time: f32,
    pub life: f32,
    /// How many times its length the staff reaches at its height.
    pub peak: f32,
    /// The damage and the push of each of its hits.
    pub dmg: f32,
    pub knock: f32,
    /// Whether its tip has landed its ring.
    pub landed: bool,
    /// The element of the ultimate at its height it belongs to; 0 for none.
    pub zen: u8,
}

impl Game {
    /// Art `a` uses stretch move `m` with damage factor `might`.
    pub(crate) fn start_stretch(&mut self, a: usize, m: usize, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        self.stretch = Some(Stretch {
            art: a as u8,
            mv: m as u16,
            time: 0.0,
            life: def.life.max(0.1),
            peak: def.reach.max(1.0),
            dmg: n.damage * def.power * might,
            knock: n.knockback * def.knockback * self.kind_knock(a),
            landed: false,
            zen: self.zenith,
        });
    }

    /// Whether art `a`'s staff is stretching now.
    pub(crate) fn stretching(&self, a: usize) -> bool {
        self.stretch.is_some_and(|s| s.art as usize == a)
    }

    /// How many times its length art `a`'s staff reaches now: exactly 1 unless it is stretching.
    pub(crate) fn stretch_factor(&self, a: usize) -> f32 {
        match self.stretch {
            Some(s) if s.art as usize == a => {
                let u = (s.time / s.life).clamp(0.0, 1.0);
                let out = if u < RAMP {
                    u / RAMP
                } else if u > 1.0 - RAMP {
                    (1.0 - u) / RAMP
                } else {
                    1.0
                };
                1.0 + (s.peak - 1.0) * out
            }
            _ => 1.0,
        }
    }

    /// One step of the stretching staff: turn it, hit along it, land its tip.
    pub(crate) fn tick_stretch(&mut self, dt: f32) {
        let Some(mut s) = self.stretch else { return };
        let a = s.art as usize;
        let Some(n) = self.art_numbers(a) else {
            self.stretch = None;
            return;
        };
        let def = &MOVES[s.mv as usize];
        s.time += dt;
        if def.speed > 1.0 {
            // Spinning faster: the staff turns on beyond its clock.
            self.staff_turn += (def.speed - 1.0) * ARTS[a].speed * dt;
            if self.staff_turn > TAU {
                self.staff_turn -= TAU;
            }
        }
        self.stretch = Some(s);
        self.striking = s.art + 1;
        self.hit_move = s.mv;
        self.zenith = s.zen;
        let element = self.arts[a].element;
        let (px, py) = (self.player.x, self.player.y);
        // An enemy is hit once a stretch.
        let rest = (s.life - s.time).max(0.0) + 0.05;
        for k in 0..n.count {
            let (bx, by) = self.orbit_pos(a, n, k);
            let (sx, sy) = (bx - px, by - py);
            let len = (sx * sx + sy * sy).sqrt().max(0.001);
            let (ux, uy) = (sx / len, sy / len);
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if e.bead_cd > 0.0 {
                    continue;
                }
                let (ox, oy) = (e.x - px, e.y - py);
                let along = ox * ux + oy * uy;
                let off = (ox * uy - oy * ux).abs();
                let r = KINDS[e.kind as usize].r + 12.0;
                if along < GRIP || along > len + r || off > r {
                    continue;
                }
                let d = (ox * ox + oy * oy).sqrt().max(0.001);
                // Outward from the player, or along the turn (the way the angle grows).
                let (rx, ry) = (ox / d, oy / d);
                let side = def.side.clamp(0.0, 1.0);
                let (kx, ky) = if side > 0.0 {
                    ((rx * (1.0 - side) - ry * side) * s.knock, (ry * (1.0 - side) + rx * side) * s.knock)
                } else {
                    (rx * s.knock, ry * s.knock)
                };
                self.enemies[i].bead_cd = rest.max(n.cooldown);
                self.strike(i, s.dmg, kx, ky, element);
            }
            self.formation_hit_bead(bx, by, s.dmg);
        }
        // At its height the tip nearest the aim strikes the ground: a ring there.
        if def.offset > 0.0 && !s.landed && s.time >= s.life * 0.5 {
            s.landed = true;
            self.stretch = Some(s);
            let (ax, ay) = self.player.aim;
            let mut tip = (px + ax, py + ay);
            let mut best = f32::MIN;
            for k in 0..n.count {
                let (bx, by) = self.orbit_pos(a, n, k);
                let (dx, dy) = (bx - px, by - py);
                let facing = (dx * ax + dy * ay) / (dx * dx + dy * dy).sqrt().max(0.001);
                if facing > best {
                    best = facing;
                    tip = (bx, by);
                }
            }
            let radius = Self::basic_numbers(a).reach * def.offset * self.kind_reach(a);
            self.ring(a, tip.0, tip.1, radius, s.dmg, s.knock);
            self.event(EV_PULSE, tip.0, tip.1, 0.0);
        }
        self.striking = 0;
        self.hit_move = super::routes::NO_MOVE;
        self.zenith = 0;
        if s.time >= s.life {
            self.stretch = None;
        }
    }

    /// The words the hash takes from the staff, while it stretches or has spun.
    pub(crate) fn staff_hash(&self, mix: &mut impl FnMut(u32)) {
        if let Some(s) = &self.stretch {
            mix(s.time.to_bits());
            mix(s.mv as u32);
        }
        if self.staff_turn != 0.0 {
            mix(self.staff_turn.to_bits());
        }
    }
}
