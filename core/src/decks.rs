//! The deck-first level-up (docs/design.md 4.9): pick a deck, then one of
//! three cards from it.
//!
//! With the `decks` tune on, a level-up first asks which deck to draw from:
//! weapons (an art not yet held), martial arts (the next level of a held art,
//! or its route), inner arts (the next layer of a passive, now and then two
//! at once), magic (three different elements) or fate (any of those, with
//! better odds of two layers at once and of fine steel and famous blades on
//! weapon cards: 50/35/15 against 70/25/5). The kind is certain; what is drawn is
//! not. A deck with nothing to draw cannot be picked, and when only one can,
//! its cards come at once. Off, the old mixed offer runs as before.

use super::kinds::CARD_WEAPON0;
use super::Game;
use crate::defs::*;

/// The decks, in the order they are offered.
pub const DECK_WEAPONS: usize = 0;
pub const DECK_ARTS: usize = 1;
pub const DECK_INNER: usize = 2;
pub const DECK_MAGIC: usize = 3;
pub const DECK_FATE: usize = 4;
pub const N_DECKS: usize = 5;

/// The chance an inner-art card from the inner deck is a fine one (two layers at once).
const FINE_INNER: f32 = 0.2;
/// The same from the fate deck.
const FINE_FATE: f32 = 0.4;

impl Game {
    /// Whether level-ups go through the decks in this run.
    pub(crate) fn decks_on(&self) -> bool {
        self.tune[T_DECKS] > 0.5
    }

    /// Whether art `a` could be offered now as an upgrade.
    pub(crate) fn offerable_art(&self, a: usize) -> bool {
        self.offerable(ARTS[a].upgrade)
    }

    /// Whether upgrade `id` can be offered now.
    fn offerable(&self, id: usize) -> bool {
        let new_art = self.up[id] == 0 && Self::art_of_upgrade(id).is_some();
        let full = self.arts_held() >= MAX_ARTS;
        let hidden = self.upgrade_hidden(id);
        self.up[id] < self.upgrade_top(id) && !(full && new_art) && !hidden
    }

    /// The cards a deck holds now (not magic, which holds elements): upgrade
    /// ids, or with weapon kinds on, weapon kind cards for the weapons deck,
    /// and for fate, weapon kind cards in place of new arts beside the rest.
    pub(crate) fn deck_pool(&self, deck: usize) -> ([usize; 64], usize) {
        let mut pool = [0usize; 64];
        let mut n = 0;
        let kinds = self.kinds_on() && N_WEAPONS > 0;
        if kinds && (deck == DECK_WEAPONS || deck == DECK_FATE) {
            for w in 0..N_WEAPONS {
                if self.weapon_offerable(w) && n < pool.len() {
                    pool[n] = CARD_WEAPON0 + w;
                    n += 1;
                }
            }
            if deck == DECK_WEAPONS {
                return (pool, n);
            }
        }
        for id in 0..N_UPGRADES {
            if !self.offerable(id) || n >= pool.len() {
                continue;
            }
            let art = Self::art_of_upgrade(id).is_some();
            let fits = match deck {
                DECK_WEAPONS => art && self.up[id] == 0,
                DECK_ARTS => art && self.up[id] > 0,
                DECK_INNER => !art,
                // With kinds on, a new art comes into fate as a weapon kind card instead.
                _ => !(kinds && art && self.up[id] == 0),
            };
            if fits {
                pool[n] = id;
                n += 1;
            }
        }
        (pool, n)
    }

    /// Whether a deck has anything to draw now.
    pub fn deck_open(&self, deck: usize) -> bool {
        match deck {
            DECK_MAGIC => self.arts_held() > 0,
            DECK_FATE => self.deck_pool(DECK_FATE).1 > 0 || self.arts_held() > 0,
            d if d < N_DECKS => self.deck_pool(d).1 > 0,
            _ => false,
        }
    }

    /// Start a level-up through the decks. Returns false when no deck has
    /// anything, which the caller treats as everything being at its top.
    pub(crate) fn offer_decks(&mut self) -> bool {
        let open: Vec<usize> = (0..N_DECKS).filter(|&d| self.deck_open(d)).collect();
        // Fate only opens when another deck does too, so one open deck is a real one.
        let real = open.iter().filter(|&&d| d != DECK_FATE).count();
        if real == 0 {
            return false;
        }
        self.choosing = true;
        if real == 1 {
            let d = open[0];
            self.draw(d);
        } else {
            self.deck_pick = true;
            self.choices = [-1; 3];
            self.ranks = [0; 3];
        }
        self.event(EV_LEVELUP, self.player.x, self.player.y, self.player.level as f32);
        true
    }

    /// Pick deck `d` while the decks are offered. The host only offers open
    /// decks; a closed or unknown one (from a bot or a tool) means the first
    /// open deck, so a pick never leaves the level-up stuck.
    pub(crate) fn pick_deck(&mut self, d: usize) {
        let d = if d < N_DECKS && self.deck_open(d) {
            d
        } else {
            match (0..N_DECKS).find(|&k| self.deck_open(k)) {
                Some(k) => k,
                None => return,
            }
        };
        self.deck_pick = false;
        self.last_deck = d as u8;
        self.draw(d);
    }

    /// Draw three cards from deck `d` into the choices.
    fn draw(&mut self, d: usize) {
        self.choices = [-1; 3];
        self.ranks = [0; 3];
        if d == DECK_MAGIC {
            // Three different elements.
            let mut els: [usize; N_ELEMENTS] = core::array::from_fn(|k| k + 1);
            for c in 0..3.min(N_ELEMENTS) {
                let j = c + (self.rng.next_u32() as usize) % (N_ELEMENTS - c);
                els.swap(c, j);
                self.choices[c] = (N_UPGRADES + els[c] - 1) as i32;
            }
            return;
        }
        let (mut pool, n) = self.deck_pool(d);
        let fine = if d == DECK_FATE { FINE_FATE } else { FINE_INNER };
        for c in 0..3 {
            if c >= n {
                break;
            }
            let j = c + (self.rng.next_u32() as usize) % (n - c);
            pool.swap(c, j);
            let id = pool[c];
            self.choices[c] = id as i32;
            // A weapon card's rank is its quality, with better odds from fate.
            if id >= CARD_WEAPON0 {
                self.ranks[c] = self.roll_quality(id - CARD_WEAPON0, d == DECK_FATE);
                continue;
            }
            // An inner art with two layers left is now and then a fine card: both at once.
            let two_left = Self::is_passive(id) && self.up[id] + 2 <= self.upgrade_top(id);
            if two_left && (d == DECK_INNER || d == DECK_FATE) && self.rng.f() < fine {
                self.ranks[c] = 1;
            }
        }
        // Fate may turn its last card into an element, when an art can take one.
        if d == DECK_FATE && self.arts_held() > 0 && (n < 3 || self.rng.f() < 0.25) {
            let element = 1 + (self.rng.next_u32() as usize) % N_ELEMENTS;
            let slot = n.min(2);
            self.choices[slot] = (N_UPGRADES + element - 1) as i32;
            self.ranks[slot] = 0;
        }
    }
}
