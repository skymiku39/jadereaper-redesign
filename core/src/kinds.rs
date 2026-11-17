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
/// The most second blows waiting at once.
pub const MAX_ECHOES: usize = 8;
/// How long after the first blow of a paired kind the second lands.
const SECOND_HAND: f32 = 0.12;

/// The second blow of a paired kind (雙刀, 雙劍): the art's basic blow is two
/// of half the damage each, the other hand's a moment after the first.
#[derive(Clone, Copy)]
pub struct Echo {
    pub left: f32,
    pub art: u8,
    pub dmg: f32,
    pub reach: f32,
    pub width: f32,
    pub knock: f32,
    pub ax: f32,
    pub ay: f32,
}

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

    /// What art `a`'s kind does of its own (a `WT_*`); nothing without a kind.
    pub(crate) fn kind_trait(&self, a: usize) -> u8 {
        self.kind_of(a).map_or(WT_NONE, |w| WEAPONS[w].trait_)
    }

    /// How many more enemies art `a`'s shots pass through for its kind.
    pub(crate) fn kind_pierce(&self, a: usize) -> i32 {
        (self.kind_trait(a) == WT_PIERCE) as i32
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

    /// The basic blow of art `a` lands for `dmg`: with a paired kind it is
    /// two blows of half that, the second a moment later along the same aim.
    /// Returns the damage of the first.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn second_hand(
        &mut self,
        a: usize,
        dmg: f32,
        reach: f32,
        width: f32,
        knock: f32,
        ax: f32,
        ay: f32,
    ) -> f32 {
        if self.forms(a) & FORM_PAIRED == 0 || self.echoes.len() >= MAX_ECHOES {
            return dmg;
        }
        let half = dmg * 0.5;
        self.echoes.push(Echo { left: SECOND_HAND, art: a as u8, dmg: half, reach, width, knock, ax, ay });
        half
    }

    /// Count down the second blows and land the ones that are due.
    pub(crate) fn tick_echoes(&mut self, dt: f32) {
        if self.echoes.is_empty() {
            return;
        }
        let mut i = 0;
        while i < self.echoes.len() {
            self.echoes[i].left -= dt;
            if self.echoes[i].left > 0.0 {
                i += 1;
                continue;
            }
            let e = self.echoes.remove(i);
            let a = e.art as usize;
            let (px, py) = (self.player.x, self.player.y);
            self.striking = e.art + 1;
            if ARTS[a].behaviour == B_THRUST {
                self.line(a, px, py, e.ax, e.ay, e.reach, e.width, e.dmg, e.knock);
            } else {
                self.sweep(a, e.dmg, e.reach, e.width, e.knock, e.ax, e.ay);
            }
            self.striking = 0;
            self.event(EV_SLASH, px, py, 0.0);
        }
    }

    /// The words the hash takes from the second blows, when there are any.
    pub(crate) fn echoes_hash(&self, mix: &mut impl FnMut(u32)) {
        for e in &self.echoes {
            mix(e.left.to_bits());
            mix(e.dmg.to_bits());
        }
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
        // A kind not bought at the weapon shop does not come; a start kind
        // and a mechanism (which comes with its stolen route) always may.
        let free = WEAPONS[w].start || WEAPONS[w].forms & FORM_MECHANISM != 0;
        if !free && (self.tune[T_KINDS_OWNED] as u32 >> w.min(31)) & 1 == 0 {
            return false;
        }
        // A mechanism only comes once a route it can learn is known.
        if WEAPONS[w].forms & FORM_MECHANISM != 0 {
            let known = (0..N_ROUTES).any(|r| {
                let def = &ROUTES[r];
                def.art == a && (def.start || self.stolen & (1 << r) != 0) && Self::kind_fits(Some(w), r)
            });
            if !known {
                return false;
            }
        }
        let level = self.art_level(a);
        if level == 0 {
            self.arts_held() < MAX_ARTS && self.offerable_art(a)
        } else if let Some(r) = self.route_of(a) {
            // An art on a route only changes to a kind that can learn that route.
            Self::kind_fits(Some(w), r) && (self.kind_of(a) != Some(w) || self.arts[a].quality < FAMOUS)
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
