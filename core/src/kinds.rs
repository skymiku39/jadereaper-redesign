//! Weapon kinds (兵器種類, docs/design.md 4.8.2): which kind of weapon an art
//! is, and what its form tags do to it.
//!
//! With the `routes` tune on, an art taken for the first time comes in its
//! start kind, and the weapons deck can hand out other kinds: a new art in
//! any of its kinds, or a held art changed to another kind (its level and
//! route stay). A kind's form tags scale reach, beat, damage and knockback,
//! and a route that suits the kind (人器合一) hits harder. Off, no art has a
//! kind and every factor here is exactly 1.

use super::Game;
use crate::defs::*;

/// Cards for weapon kinds come after the upgrades and the elements: card
/// `CARD_WEAPON0 + w` is weapon kind `w`.
pub const CARD_WEAPON0: usize = N_UPGRADES + N_ELEMENTS;
/// How much harder a route lands with a kind it suits.
const ONENESS: f32 = 1.15;
/// Weapon quality (品質, docs/design.md 4.8.2): plain iron, fine steel, a famous blade.
pub const PLAIN: u8 = 0;
pub const STEEL: u8 = 1;
pub const FAMOUS: u8 = 2;
/// Per quality: damage and reach.
const QUALITY_POWER: [f32; 3] = [1.0, 1.15, 1.3];
const QUALITY_REACH: [f32; 3] = [1.0, 1.05, 1.1];
/// The chance of fine steel and of a famous blade on a weapon card.
const STEEL_CHANCE: f32 = 0.25;
const FAMOUS_CHANCE: f32 = 0.05;
/// Weapon cards in a row without fine steel or better before the next is sure to be.
const PITY: u8 = 3;

impl Game {
    /// Whether arts come in weapon kinds in this run.
    pub(crate) fn kinds_on(&self) -> bool {
        self.routes_on()
    }

    /// The weapon kind art `a` is, if any.
    pub(crate) fn kind_of(&self, a: usize) -> Option<usize> {
        match self.arts[a].kind {
            0 => None,
            k => Some(k as usize - 1),
        }
    }

    /// The kind an art comes in when it is first taken.
    pub(crate) fn start_kind(a: usize) -> Option<usize> {
        WEAPONS.iter().position(|w| w.art == a && w.start)
    }

    /// Art `a` was just taken: it comes in its start kind, if kinds are on.
    pub(crate) fn hold_kind(&mut self, a: usize) {
        if self.kinds_on() && self.arts[a].kind == 0 {
            if let Some(w) = Self::start_kind(a) {
                self.arts[a].kind = w as u8 + 1;
            }
        }
    }

    /// The form tags of art `a`'s kind; none without a kind.
    fn forms(&self, a: usize) -> u8 {
        self.kind_of(a).map_or(0, |w| WEAPONS[w].forms)
    }

    /// How much further art `a` reaches for its kind and quality.
    pub(crate) fn kind_reach(&self, a: usize) -> f32 {
        let f = self.forms(a);
        let mut x = QUALITY_REACH[(self.arts[a].quality as usize).min(2)];
        if f & FORM_LONG != 0 {
            x *= 1.15;
        }
        if f & FORM_SHORT != 0 {
            x *= 0.9;
        }
        x
    }

    /// How much longer art `a`'s beat or cooldown is for its kind.
    pub(crate) fn kind_beat(&self, a: usize) -> f32 {
        let f = self.forms(a);
        let mut x = 1.0;
        if f & (FORM_LONG | FORM_HEAVY) != 0 {
            x *= 1.1;
        }
        if f & (FORM_SHORT | FORM_LIGHT) != 0 {
            x *= 0.9;
        }
        x
    }

    /// How much harder art `a` hits for its kind and quality, and for a route that suits it.
    pub(crate) fn kind_power(&self, a: usize) -> f32 {
        let f = self.forms(a);
        let mut x = QUALITY_POWER[(self.arts[a].quality as usize).min(2)];
        if f & FORM_HEAVY != 0 {
            x *= 1.1;
        }
        if f & FORM_PAIRED != 0 {
            x *= 1.2;
        }
        if self.oneness(a) {
            x *= ONENESS;
        }
        x
    }

    /// How much harder art `a` throws enemies back for its kind.
    pub(crate) fn kind_knock(&self, a: usize) -> f32 {
        if self.forms(a) & FORM_HEAVY != 0 {
            1.3
        } else {
            1.0
        }
    }

    /// Whether art `a`'s route suits its kind (人器合一).
    pub(crate) fn oneness(&self, a: usize) -> bool {
        match (self.route_of(a), self.kind_of(a)) {
            (Some(r), Some(w)) => ROUTES[r].suits & (1 << w) != 0,
            _ => false,
        }
    }

    /// Whether weapon kind `w` can be handed out now: a new art while a slot
    /// is free, another kind for a held art, or a better one of the kind it
    /// is while it is not yet a famous blade.
    pub(crate) fn weapon_offerable(&self, w: usize) -> bool {
        let a = WEAPONS[w].art;
        let level = self.art_level(a);
        if level == 0 {
            self.arts_held() < MAX_ARTS && self.offerable_art(a)
        } else {
            self.kind_of(a) != Some(w) || self.arts[a].quality < FAMOUS
        }
    }

    /// The quality of a weapon card for kind `w` just drawn. A card of the
    /// kind an art already is always brings a better one; after `PITY` cards
    /// without fine steel or better, the next is sure to be.
    pub(crate) fn roll_quality(&mut self, w: usize) -> u8 {
        let roll = self.rng.f();
        let mut q = if roll < FAMOUS_CHANCE {
            FAMOUS
        } else if roll < FAMOUS_CHANCE + STEEL_CHANCE {
            STEEL
        } else {
            PLAIN
        };
        if q == PLAIN && self.dry >= PITY {
            q = STEEL;
        }
        self.dry = if q == PLAIN { self.dry.saturating_add(1) } else { 0 };
        let a = WEAPONS[w].art;
        if self.art_level(a) > 0 && self.kind_of(a) == Some(w) {
            q = q.max(self.arts[a].quality + 1).min(FAMOUS);
        }
        q
    }

    /// Take weapon kind card `w` of quality `q`: a new art at level 1 in that
    /// kind, or a held art changed to it. A held art keeps the better of its
    /// old quality and the new one.
    pub(crate) fn take_weapon(&mut self, w: usize, q: u8) {
        let a = WEAPONS[w].art;
        if self.art_level(a) == 0 {
            self.up[ARTS[a].upgrade] = 1;
            self.arts[a].quality = q;
        } else {
            self.arts[a].quality = self.arts[a].quality.max(q);
        }
        self.arts[a].kind = w as u8 + 1;
        self.event(EV_ATTACH, self.player.x, self.player.y, 0.0);
    }
}
