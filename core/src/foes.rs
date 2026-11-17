//! School elites (docs/design.md 4.8.6): enemies that use a move of their
//! school, warned of on the ground first, so the player can see it and step
//! out of it. The move's name is called out in red; felling one teaches the
//! school's stolen routes like any disciple (`routes.rs`).
//!
//! An enemy kind with a `strike` uses that `[[move]]`: an arc toward the
//! player or a ring around itself, `reach` times `FOE_REACH` across. A
//! negative knockback pulls the player in. Only kinds that come alone with a
//! `school_share` above 0 exist, so with it at 0 nothing here runs.

use super::{Fx, Game};
use crate::defs::*;
use crate::fmath;

/// How far a foe's move reaches per 1 of the move's `reach`.
const FOE_REACH: f32 = 100.0;
/// Seconds of warning before the move lands.
const WIND_UP: f32 = 0.8;
/// Seconds between the end of one move and the start of the next.
const REST: f32 = 3.0;
/// The most foes tracked at once.
const MAX_FOES: usize = 16;
/// The colour of a foe's warning and blow.
const RED: [f32; 3] = [0.72, 0.16, 0.12];

/// A foe that uses its school's move.
#[derive(Clone, Copy)]
pub struct Foe {
    /// The enemy it is, by id.
    pub id: u32,
    /// Seconds left of the warning; 0 while not winding up.
    pub wind: f32,
    /// Seconds before it may start its next move.
    pub rest: f32,
    /// Which way the move goes, set when the warning starts.
    pub ax: f32,
    pub ay: f32,
}

impl Game {
    /// Whether an enemy stands still, winding up a move.
    #[cfg(test)]
    pub(crate) fn winding(&self, id: u32) -> bool {
        self.foes.iter().any(|f| f.id == id && f.wind > 0.0)
    }

    /// One step of the foes: track new ones, forget fallen ones, start warnings
    /// and land the moves whose warning ran out.
    pub(crate) fn foes_step(&mut self, dt: f32) {
        // Track every enemy with a move, and forget the ones that are gone.
        self.foes.retain(|f| self.enemies.iter().any(|e| e.id == f.id));
        for e in &self.enemies {
            if KINDS[e.kind as usize].strike < N_MOVES
                && self.foes.len() < MAX_FOES
                && !self.foes.iter().any(|f| f.id == e.id)
            {
                self.foes.push(Foe { id: e.id, wind: 0.0, rest: 1.5, ax: 1.0, ay: 0.0 });
            }
        }
        let (px, py) = (self.player.x, self.player.y);
        for n in 0..self.foes.len() {
            let f = self.foes[n];
            let Some(i) = self.enemies.iter().position(|e| e.id == f.id) else { continue };
            let (ex, ey, kind) = (self.enemies[i].x, self.enemies[i].y, self.enemies[i].kind as usize);
            let m = KINDS[kind].strike;
            let reach = MOVES[m].reach.abs() * FOE_REACH;
            if f.wind > 0.0 {
                let wind = f.wind - dt;
                self.foes[n].wind = wind.max(0.0);
                if wind <= 0.0 {
                    self.foes[n].rest = REST;
                    self.foe_strike(i, m, f.ax, f.ay);
                }
                continue;
            }
            let rest = f.rest - dt;
            self.foes[n].rest = rest.max(0.0);
            let (dx, dy) = (px - ex, py - ey);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if rest <= 0.0 && d < reach * 1.1 {
                // Start the warning: the shape of the move on the ground, and its name.
                let (ax, ay) = (dx / d, dy / d);
                self.foes[n].wind = WIND_UP;
                self.foes[n].ax = ax;
                self.foes[n].ay = ay;
                self.foe_mark(ex, ey, ax, ay, m, reach, 0.35, WIND_UP);
                self.event(EV_FOE, ex, ey, m as f32);
            }
        }
    }

    /// Draw a foe's move at (x, y): faint while it warns, solid when it lands.
    #[allow(clippy::too_many_arguments)]
    fn foe_mark(&mut self, x: f32, y: f32, ax: f32, ay: f32, m: usize, reach: f32, alpha: f32, life: f32) {
        let ring = MOVES[m].shape == MS_RING || MOVES[m].width < -1.0;
        self.add_fx(Fx {
            x,
            y: y - 6.0,
            vx: 0.0,
            vy: 0.0,
            life,
            max: life,
            part: if ring { P_RING } else { P_SLASH },
            hw: reach,
            hh: reach,
            rot: fmath::atan2(ay, ax),
            vr: 0.0,
            grow: 0.0,
            color: [RED[0], RED[1], RED[2], alpha],
            seed: 0.6,
            top: false,
        });
    }

    /// Foe `i` lands move `m` toward (ax, ay).
    fn foe_strike(&mut self, i: usize, m: usize, ax: f32, ay: f32) {
        let def = &MOVES[m];
        let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
        let reach = def.reach.abs() * FOE_REACH;
        let dmg = KINDS[self.enemies[i].kind as usize].dmg * def.power.max(0.5);
        self.foe_mark(ex, ey, ax, ay, m, reach, 0.85, 0.25);
        let (dx, dy) = (self.player.x - ex, self.player.y - ey);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        let ring = def.shape == MS_RING || def.width < -1.0;
        let inside = d < reach + 14.0 && (ring || d < 30.0 || (dx * ax + dy * ay) / d > def.width);
        if !inside {
            return;
        }
        // A pull drags the player toward the foe; a push throws him back.
        let shove = if def.knockback < 0.0 { -(d - 30.0).clamp(0.0, 90.0) } else { 40.0 * def.knockback };
        self.player.x += dx / d * shove;
        self.player.y += dy / d * shove;
        self.hurt_player(dmg);
    }

    /// A blow from a foe's move, through the player's grace, inner arts and guard.
    fn hurt_player(&mut self, dmg: f32) {
        if self.player.iframe > 0.0 {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let dmg = self.soften(dmg);
        self.player.hp -= self.guard_against(dmg);
        self.player.iframe = 0.8;
        self.shake = 1.0;
        self.event(EV_HURT, px, py, dmg);
        if self.player.hp <= 0.0 && !self.last_stand() {
            self.player.hp = 0.0;
            self.over = true;
            self.event(EV_DEATH, px, py, 0.0);
        }
    }

    /// The words the hash takes from the foes, when there are any.
    pub(crate) fn foes_hash(&self, mix: &mut impl FnMut(u32)) {
        if self.foes.is_empty() {
            return;
        }
        mix(self.foes.len() as u32);
        for f in &self.foes {
            mix(f.id);
            mix(f.wind.to_bits());
            mix(f.rest.to_bits());
        }
    }
}
