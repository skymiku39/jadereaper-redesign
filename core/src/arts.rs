//! Martial arts (武學): how the player attacks.
//!
//! Which arts exist and every number of theirs come from `[[art]]` in
//! `content/game.toml`, through `ARTS`. This file holds one function per
//! behaviour. An art's level is the level of the upgrade named in its
//! definition, and the element it carries (see `elements.rs`) rides along on
//! every hit it lands.

use super::{Fx, Game, Shot, MAX_SHOTS};
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// The colour of an art's swing, shot, bead or ring while it carries no element.
const JADE: [f32; 3] = [0.25, 0.62, 0.48];
const PAPER_GOLD: [f32; 3] = [0.89, 0.77, 0.38];
const BEAD_GREEN: [f32; 3] = [0.30, 0.66, 0.50];

/// What a run keeps per martial art, held or not.
#[derive(Clone, Copy)]
pub struct ArtState {
    /// Seconds until the art is used next.
    pub timer: f32,
    /// The element it carries, an `E_*` id; 0 for none.
    pub element: u8,
    /// The route it follows, by route id plus 1; 0 for none (see `routes.rs`).
    pub route: u8,
    /// Where it is in its route's move order.
    pub step: u8,
}

impl ArtState {
    /// Every art as a run starts: waiting out its first delay, with no element.
    pub fn fresh() -> [ArtState; N_ARTS] {
        core::array::from_fn(|a| ArtState { timer: ARTS[a].first_delay, element: 0, route: 0, step: 0 })
    }
}

impl Game {
    /// The level the art is held at; 0 when it is not held.
    pub(crate) fn art_level(&self, a: usize) -> usize {
        self.up[ARTS[a].upgrade] as usize
    }

    /// The art's numbers at its current level, or none when it is not held.
    /// An art on a route keeps its level-2 numbers, since its route's moves
    /// grow instead; circling beads (a staff) still grow with the level, since
    /// they keep circling under the moves.
    fn art_numbers(&self, a: usize) -> Option<&'static ArtLevel> {
        let levels = ARTS[a].levels;
        match self.art_level(a) {
            0 => None,
            l if self.arts[a].route > 0 && ARTS[a].behaviour != B_ORBIT => levels.get(l.min(2).min(levels.len()) - 1),
            l => levels.get(l.min(levels.len()) - 1),
        }
    }

    /// Set a martial art's level and element, each kept inside what exists.
    /// An unknown art changes nothing. Not part of play: see `jr_give`.
    pub fn give(&mut self, a: usize, level: u32, element: u32) {
        if a >= N_ARTS {
            return;
        }
        // With routes on, an art given level 3 or more takes its first route.
        if level >= 3 && self.routes_on() && self.arts[a].route == 0 {
            if let Some(r) = Self::routes_of(a).next() {
                self.take_route(a, r);
            }
        }
        let top = self.upgrade_top(ARTS[a].upgrade) as u32;
        self.up[ARTS[a].upgrade] = level.min(top) as u8;
        self.arts[a].element = if element as usize <= N_ELEMENTS { element as u8 } else { 0 };
    }

    /// How many martial arts are held.
    pub(crate) fn arts_held(&self) -> usize {
        (0..N_ARTS).filter(|&a| self.art_level(a) > 0).count()
    }

    /// The art's colour: its element's when it carries one, else `plain`.
    pub(crate) fn art_color(&self, a: usize, plain: [f32; 3], alpha: f32) -> [f32; 4] {
        let c = match self.arts[a].element {
            0 => plain,
            e => ELEMENTS[e as usize].color,
        };
        [c[0], c[1], c[2], alpha]
    }

    /// Use every held art that is ready, in id order.
    pub(crate) fn weapons(&mut self, dt: f32) {
        let might = self.stat(ST_DAMAGE);
        let haste = self.stat(ST_COOLDOWN);
        for (a, def) in ARTS.iter().enumerate() {
            let Some(n) = self.art_numbers(a) else { continue };
            if self.arts[a].route > 0 && self.art_level(a) >= 3 {
                // Circling beads (a staff) keep circling; the moves come on top.
                if def.behaviour == B_ORBIT {
                    self.art_orbit(a, n, might);
                }
                self.route_beat(a, dt, might, haste);
                continue;
            }
            match def.behaviour {
                B_ARC => self.art_arc(a, n, dt, might, haste),
                B_PROJECTILE => self.art_projectile(a, n, dt, might, haste),
                B_ORBIT => self.art_orbit(a, n, might),
                _ => self.art_nova(a, n, dt, might, haste),
            }
        }
    }

    /// A sweeping arc in the aim direction.
    fn art_arc(&mut self, a: usize, n: &ArtLevel, dt: f32, might: f32, haste: f32) {
        self.arts[a].timer -= dt;
        if self.arts[a].timer > 0.0 {
            return;
        }
        self.arts[a].timer = n.cooldown * haste;
        let element = self.arts[a].element;
        let (px, py) = (self.player.x, self.player.y);
        let (ax, ay) = self.player.aim;
        let reach = n.reach * self.melee_reach();
        let dmg = n.damage * might;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - px, e.y - py);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let r = KINDS[e.kind as usize].r;
            if d < reach + r && (d < 30.0 || (dx * ax + dy * ay) / d > n.width) {
                self.strike(i, dmg, dx / d * n.knockback, dy / d * n.knockback, element);
            }
        }
        self.formation_hit_arc(px, py, ax, ay, reach, n.width, dmg);
        self.player.attack = 0.25;
        let rot = fmath::atan2(ay, ax);
        let color = self.art_color(a, JADE, 0.9);
        for k in 0..n.count {
            self.add_fx(Fx {
                x: px,
                y: py - 24.0,
                vx: 0.0,
                vy: 0.0,
                life: 0.22,
                max: 0.22,
                part: ARTS[a].part,
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
        self.event(EV_SLASH, px, py, 0.0);
    }

    /// Shots that fly at the nearest enemies.
    fn art_projectile(&mut self, a: usize, n: &ArtLevel, dt: f32, might: f32, haste: f32) {
        self.arts[a].timer -= dt;
        if self.arts[a].timer > 0.0 {
            return;
        }
        self.arts[a].timer = n.cooldown * haste;
        if !self.shoot_nearest(a, n, might) {
            // Nothing in range: look again soon instead of waiting out the cooldown.
            self.arts[a].timer = 0.15;
        }
    }

    /// One volley of art `a` with numbers `n`: a shot at each of the nearest
    /// enemies in range. Returns whether anything was shot at.
    pub(crate) fn shoot_nearest(&mut self, a: usize, n: &ArtLevel, might: f32) -> bool {
        let def = &ARTS[a];
        let (px, py) = (self.player.x, self.player.y);
        // The nearest `count` enemies, nearest first.
        const MOST: usize = 8;
        let count = n.count.min(MOST);
        let mut best = [(f32::MAX, 0usize); MOST];
        for (i, e) in self.enemies.iter().enumerate() {
            let d = (e.x - px) * (e.x - px) + (e.y - py) * (e.y - py);
            for s in 0..count {
                if d < best[s].0 {
                    for t in (s + 1..count).rev() {
                        best[t] = best[t - 1];
                    }
                    best[s] = (d, i);
                    break;
                }
            }
        }
        let mut fired = false;
        for &(d2, i) in best.iter().take(count) {
            if d2 > def.range * def.range || self.shots.len() >= MAX_SHOTS {
                continue;
            }
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - px, e.y - py);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            self.shots.push(Shot {
                x: px,
                y: py,
                vx: dx / d * def.speed,
                vy: dy / d * def.speed,
                life: def.life,
                dmg: n.damage * might,
                pierce: n.pierce,
                last: 0,
                art: a as u8,
                push: n.knockback,
                split: 0,
                grow: 0.0,
                size: 1.0,
            });
            fired = true;
        }
        if fired {
            self.event(EV_SHOOT, px, py, 0.0);
        }
        fired
    }

    /// Beads that circle the player. Their places come from the clock, so
    /// there is nothing to store.
    fn art_orbit(&mut self, a: usize, n: &ArtLevel, might: f32) {
        let element = self.arts[a].element;
        let (px, py) = (self.player.x, self.player.y);
        let dmg = n.damage * might;
        for k in 0..n.count {
            let (bx, by) = self.orbit_pos(a, n, k);
            for i in 0..self.enemies.len() {
                let e = &self.enemies[i];
                if e.bead_cd > 0.0 {
                    continue;
                }
                let r = KINDS[e.kind as usize].r + 12.0;
                let (dx, dy) = (e.x - bx, e.y - by);
                if dx * dx + dy * dy < r * r {
                    let (ox, oy) = (e.x - px, e.y - py);
                    let d = (ox * ox + oy * oy).sqrt().max(0.001);
                    self.enemies[i].bead_cd = n.cooldown;
                    self.strike(i, dmg, ox / d * n.knockback, oy / d * n.knockback, element);
                }
            }
            self.formation_hit_bead(bx, by, dmg);
        }
    }

    /// How many beads orbit art `a` has circling; 0 when it is not held.
    #[cfg(test)]
    pub(crate) fn orbit_count(&self, a: usize) -> usize {
        self.art_numbers(a).map_or(0, |n| n.count)
    }

    /// Where bead `k` of orbit art `a` is now. The art must be held.
    #[cfg(test)]
    pub(crate) fn orbit_at(&self, a: usize, k: usize) -> (f32, f32) {
        match self.art_numbers(a) {
            Some(n) => self.orbit_pos(a, n, k),
            None => (self.player.x, self.player.y),
        }
    }

    /// Where bead `k` of an orbit art is now.
    fn orbit_pos(&self, a: usize, n: &ArtLevel, k: usize) -> (f32, f32) {
        let radius = n.reach;
        let angle = self.time * ARTS[a].speed + k as f32 * TAU / n.count as f32;
        (self.player.x + fmath::cos(angle) * radius, self.player.y + fmath::sin(angle) * radius * 0.8)
    }

    /// A ring that damages and shoves everything nearby.
    fn art_nova(&mut self, a: usize, n: &ArtLevel, dt: f32, might: f32, haste: f32) {
        self.arts[a].timer -= dt;
        if self.arts[a].timer > 0.0 {
            return;
        }
        self.arts[a].timer = n.cooldown * haste;
        let (px, py) = (self.player.x, self.player.y);
        self.ring(a, px, py, n.reach, n.damage * might, n.knockback);
        self.event(EV_PULSE, px, py, 0.0);
    }

    /// A ring of art `a` centred on (cx, cy): everything within `radius` is
    /// hit and pushed away from the centre.
    pub(crate) fn ring(&mut self, a: usize, cx: f32, cy: f32, radius: f32, dmg: f32, knockback: f32) {
        let element = self.arts[a].element;
        let (px, py) = (cx, cy);
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - px, e.y - py);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < radius {
                self.strike(i, dmg, dx / d * knockback, dy / d * knockback, element);
            }
        }
        self.formation_hit_circle(px, py, radius, dmg);
        let color = self.art_color(a, JADE, 0.8);
        self.add_fx(Fx {
            x: px,
            y: py - 10.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.35,
            max: 0.35,
            part: P_RING,
            hw: 20.0,
            hh: 20.0,
            rot: 0.0,
            vr: 0.0,
            grow: (radius - 20.0) / 0.35,
            color,
            seed: 0.7,
            top: true,
        });
    }

    /// Draw what the arts keep in the air: shots in flight and circling beads.
    pub(crate) fn draw_arts(&mut self) {
        for i in 0..self.shots.len() {
            let s = &self.shots[i];
            let (x, y, rot, a, size) = (s.x, s.y, fmath::atan2(s.vy, s.vx) + TAU * 0.25, s.art as usize, s.size);
            let color = self.art_color(a, PAPER_GOLD, 1.0);
            // A blade wave of a sweeping art is drawn as a slash across its flight.
            if ARTS[a].behaviour == B_ARC {
                self.put(x, y - 24.0, 26.0 * size, 26.0 * size, rot - TAU * 0.25, ARTS[a].part, 0.0, 0.5, color);
            } else {
                self.put(x, y - 24.0, 6.0 * size, 11.0 * size, rot, ARTS[a].part, 0.0, 0.5, color);
            }
        }
        for (a, def) in ARTS.iter().enumerate() {
            if def.behaviour != B_ORBIT {
                continue;
            }
            let Some(n) = self.art_numbers(a) else { continue };
            let color = self.art_color(a, BEAD_GREEN, 1.0);
            for k in 0..n.count {
                let (x, y) = self.orbit_pos(a, n, k);
                self.put(x, y - 24.0, 9.0, 9.0, 0.0, def.part, 0.0, k as f32 * 0.13, color);
            }
        }
    }
}
