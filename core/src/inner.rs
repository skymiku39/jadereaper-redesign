//! Inner arts (心法, docs/design.md 4.8.4): the passive upgrades, with rules
//! of their own from the second layer up.
//!
//! With the `routes` tune on, every passive upgrade is an inner art of three
//! layers (levels). Its first layer does what the passive always did, once;
//! the second and third change a rule. The three newest (borrowed force,
//! piercing qi and the art of all methods) are only offered then. Off, none of
//! this runs and the passives stack as before, up to their own top level.

use super::{Game, Shot, MAX_SHOTS};
use crate::defs::*;

/// The most splashes of borrowed force that wait for the end of a step.
const MAX_SPLASH: usize = 32;
/// How far a splash of borrowed force reaches from the enemy that was struck.
const SPLASH_REACH: f32 = 55.0;

/// What the inner arts keep track of during a run.
#[derive(Default)]
pub struct Inner {
    /// Seconds since the player was last hurt.
    pub since_hurt: f32,
    /// Seconds the player has kept walking without stopping.
    pub moving_for: f32,
    /// Seconds until the third layer of breathing can save the player again.
    pub last_stand: f32,
    /// Seconds until the third layer of the sleeve gathers every jade again.
    pub gather: f32,
    /// Seconds since a martial art last struck an enemy.
    pub since_strike: f32,
    /// Seconds left of the haste after an ultimate (third layer of the fleeting colt).
    pub rush: f32,
    /// Blows of borrowed force to land at the end of the step: where, how hard, and the enemy struck.
    pub splash: Vec<(f32, f32, f32, u32)>,
}

impl Inner {
    pub fn mix(&self, mix: &mut impl FnMut(u32)) {
        for v in [self.since_hurt, self.moving_for, self.last_stand, self.gather, self.since_strike, self.rush] {
            mix(v.to_bits());
        }
    }
}

impl Game {
    /// Whether the passives are inner arts in this run.
    pub(crate) fn inner_on(&self) -> bool {
        self.routes_on()
    }

    /// The layer an inner art is held at; 0 when it is not, or inner arts are off.
    pub(crate) fn layer(&self, id: usize) -> u8 {
        if self.inner_on() {
            self.up[id]
        } else {
            0
        }
    }

    /// The amount of the passive levelled by upgrade `id`.
    fn amount(id: usize) -> f32 {
        PASSIVES.iter().find(|p| p.upgrade == id).map_or(0.0, |p| p.amount)
    }

    /// Whether an upgrade is only offered while inner arts are on.
    pub(crate) fn inner_only(id: usize) -> bool {
        PASSIVES.iter().any(|p| p.upgrade == id && matches!(p.stat, ST_SPLASH | ST_REACH | ST_ELEMENT))
    }

    /// Whether an upgrade is kept out of the offers in this run: the newest
    /// inner arts and the second batch of weapons while routes are off.
    pub(crate) fn upgrade_hidden(&self, id: usize) -> bool {
        let routes_only = ARTS.iter().any(|art| art.upgrade == id && art.routes_only);
        if !self.inner_on() {
            return Self::inner_only(id) || routes_only;
        }
        // An inner art not yet learned with silver (the `inner_owned` tune).
        Self::is_passive(id) && (self.tune[T_INNER_OWNED] as u32 >> id.min(31)) & 1 == 0
    }

    /// Whether an upgrade is a passive, so an inner art while they are on.
    pub(crate) fn is_passive(id: usize) -> bool {
        PASSIVES.iter().any(|p| p.upgrade == id)
    }

    /// Count the inner arts' clocks for one step, and gather the jade when it is time.
    pub(crate) fn inner_tick(&mut self, dt: f32) {
        if !self.inner_on() {
            return;
        }
        let moving = self.player.moving;
        let gathering = self.layer(U_LURE) >= 3;
        self.bulwark = (self.bulwark - dt).max(0.0);
        let n = &mut self.inner;
        n.since_hurt += dt;
        n.moving_for = if moving { n.moving_for + dt } else { 0.0 };
        n.last_stand = (n.last_stand - dt).max(0.0);
        n.since_strike += dt;
        n.rush = (n.rush - dt).max(0.0);
        if gathering {
            n.gather -= dt;
            if n.gather <= 0.0 {
                n.gather = 8.0;
                self.gather_jade();
            }
        }
    }

    /// Every jade on the field flies to the player (the sleeve, 袖裡乾坤).
    pub(crate) fn gather_jade(&mut self) {
        for g in self.gems.iter_mut() {
            g.pull = true;
        }
    }

    /// How much faster the player walks after walking a while (second layer of crossing on a reed).
    pub(crate) fn stride(&self) -> f32 {
        if self.layer(U_SWIFT) >= 2 && self.inner.moving_for > 2.0 {
            1.12
        } else {
            1.0
        }
    }

    /// How much faster the player heals while unhurt (second layer of breathing).
    pub(crate) fn mending(&self) -> f32 {
        if self.layer(U_MENDING) >= 2 && self.inner.since_hurt > 3.0 {
            2.0
        } else {
            1.0
        }
    }

    /// A blow to the player before his guard: dodged (third layer of crossing on a
    /// reed), lessened (second layer of copper skin) or halved while he is near full
    /// health (its third layer). Returns what is left of it.
    pub(crate) fn soften(&mut self, dmg: f32) -> f32 {
        if !self.inner_on() {
            return dmg;
        }
        self.inner.since_hurt = 0.0;
        if self.layer(U_SWIFT) >= 3 && self.rng.f() < 0.15 {
            return 0.0;
        }
        let mut dmg = dmg;
        let skin = self.layer(U_VITALITY);
        if skin >= 2 {
            dmg = (dmg - 3.0).max(0.0);
        }
        if skin >= 3 && self.player.hp > self.player.max_hp * 0.8 {
            dmg *= 0.5;
        }
        // Just after a metal ultimate at its height, blows are halved (`zenith.rs`).
        if self.bulwark > 0.0 {
            dmg *= 0.5;
        }
        dmg
    }

    /// Whether a blow that would have ended the run leaves the player at 1
    /// instead (third layer of breathing, once a minute).
    pub(crate) fn last_stand(&mut self) -> bool {
        if self.layer(U_MENDING) >= 3 && self.inner.last_stand <= 0.0 {
            self.inner.last_stand = 60.0;
            self.player.hp = 1.0;
            self.player.iframe = 1.5;
            self.event(EV_HEAL, self.player.x, self.player.y, 0.0);
            return true;
        }
        false
    }

    /// The share of a route's beat it waits now: shorter after a recent hit
    /// (second layer of the fleeting colt) and after an ultimate (its third).
    pub(crate) fn beat_factor(&self) -> f32 {
        let colt = self.layer(U_HASTE);
        let mut f = 1.0;
        if colt >= 2 && self.inner.since_strike < 1.0 {
            f *= 0.9;
        }
        if colt >= 3 && self.inner.rush > 0.0 {
            f *= 0.5;
        }
        f
    }

    /// How much harder the last move of a route's order lands (second and third layers of the mountain lifter).
    pub(crate) fn finisher(&self) -> f32 {
        match self.layer(U_MIGHT) {
            0 | 1 => 1.0,
            2 => 1.3,
            _ => 1.6,
        }
    }

    /// An ultimate was used.
    pub(crate) fn note_ultimate(&mut self) {
        if self.layer(U_HASTE) >= 3 {
            self.inner.rush = 2.0;
        }
    }

    /// How much further melee moves reach (piercing qi, 氣貫長虹).
    pub(crate) fn melee_reach(&self) -> f32 {
        match self.layer(U_QIGUAN) {
            0 => 1.0,
            _ => 1.0 + Self::amount(U_QIGUAN),
        }
    }

    /// The last move of a melee route's order sends a blade of qi along the
    /// aim (second layer of piercing qi), which passes through everything at
    /// the third.
    pub(crate) fn qi_blade(&mut self, a: usize, dmg: f32) {
        let layer = self.layer(U_QIGUAN);
        if layer < 2 || ARTS[a].behaviour == B_PROJECTILE || self.shots.len() >= MAX_SHOTS {
            return;
        }
        let (ax, ay) = self.player.aim;
        self.shots.push(Shot {
            x: self.player.x,
            y: self.player.y,
            vx: ax * 620.0,
            vy: ay * 620.0,
            life: 0.6,
            dmg,
            pierce: if layer >= 3 { 999 } else { 3 },
            last: 0,
            art: a as u8,
            push: 0.2,
            split: 0,
            grow: 0.0,
            size: 1.0,
            zen: 0,
        });
    }

    /// A struck enemy was thrown back: part of the blow passes to the
    /// enemies it is thrown into (borrowed force, 四兩撥千斤), landed at the
    /// end of the step so that it cannot set off more of itself.
    pub(crate) fn borrow_force(&mut self, i: usize, dmg: f32, kx: f32, ky: f32) {
        let layer = self.layer(U_JIELI);
        if layer == 0 || kx * kx + ky * ky < 60.0 * 60.0 || self.inner.splash.len() >= MAX_SPLASH {
            return;
        }
        let e = &self.enemies[i];
        let share = Self::amount(U_JIELI) * layer as f32;
        self.inner.splash.push((e.x, e.y, dmg * share, e.id));
    }

    /// Land the blows of borrowed force gathered this step.
    pub(crate) fn land_splash(&mut self) {
        if self.inner.splash.is_empty() {
            return;
        }
        let splash = core::mem::take(&mut self.inner.splash);
        for &(x, y, dmg, id) in &splash {
            for j in 0..self.enemies.len() {
                let e = &self.enemies[j];
                if e.id == id || e.hp <= 0.0 {
                    continue;
                }
                let (dx, dy) = (e.x - x, e.y - y);
                let reach = SPLASH_REACH + KINDS[e.kind as usize].r;
                if dx * dx + dy * dy < reach * reach {
                    self.hurt_enemy(j, dmg, 0.0, 0.0);
                }
            }
        }
        self.inner.splash = splash;
        self.inner.splash.clear();
    }

    /// How much harder a hit lands for carrying an element (the art of all methods, 萬法歸一功).
    pub(crate) fn element_boost(&self) -> f32 {
        match self.layer(U_WANFA) {
            0 => 1.0,
            _ => 1.0 + Self::amount(U_WANFA),
        }
    }

    /// How much harder reactions land (second and third layers of the art of all methods).
    pub(crate) fn reaction_boost(&self) -> f32 {
        match self.layer(U_WANFA) {
            0 | 1 => 1.0,
            2 => 1.25,
            _ => 1.5,
        }
    }
}
