//! Tests for the deck-first level-up. Included from `game.rs` under
//! `#[cfg(test)]` so the tests can reach private fields.

use super::decks::*;
use super::*;

/// A running game with routes, inner arts and decks on, and one level-up waiting.
fn levelled(seed: u32) -> Game {
    let mut g = Game::new(seed);
    g.running = true;
    assert!(g.decks_on());
    g.pending = 1;
    g.offer();
    g
}

#[test]
fn a_level_up_asks_for_a_deck_first() {
    let g = levelled(1);
    assert!(g.choosing && g.deck_pick);
    assert_eq!(g.choices, [-1; 3]);
    // At the start only the scythe is held: every deck has something.
    for d in 0..N_DECKS {
        assert!(g.deck_open(d), "deck {d}");
    }
}

#[test]
fn each_deck_draws_only_its_own_kind() {
    for seed in 1..30 {
        for d in [DECK_WEAPONS, DECK_ARTS, DECK_INNER, DECK_MAGIC] {
            let mut g = levelled(seed);
            g.choose(d);
            assert!(!g.deck_pick && g.choosing);
            let drawn: Vec<i32> = g.choices.iter().copied().filter(|&c| c >= 0).collect();
            assert!(!drawn.is_empty(), "seed {seed} deck {d}");
            for &c in &drawn {
                let id = c as usize;
                match d {
                    DECK_WEAPONS => assert!((kinds::CARD_WEAPON0..kinds::CARD_WEAPON0 + N_WEAPONS).contains(&id)),
                    DECK_ARTS => assert!(Game::art_of_upgrade(id).is_some() && g.up[id] > 0),
                    DECK_INNER => assert!(Game::is_passive(id)),
                    _ => assert!(id >= N_UPGRADES),
                }
            }
            let mut sorted = drawn.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), drawn.len(), "the cards are different");
            assert_eq!(g.last_deck as usize, d);
        }
    }
}

#[test]
fn a_card_from_a_deck_is_taken_like_any_other() {
    let mut g = levelled(3);
    g.choose(DECK_INNER);
    let id = g.choices[0] as usize;
    let before = g.up[id];
    g.choose(0);
    assert!(!g.choosing);
    assert!(g.up[id] > before);
    assert_eq!(g.pending, 0);
}

#[test]
fn a_weapon_card_brings_a_new_art_or_changes_a_held_one() {
    for seed in 1..40 {
        let mut g = levelled(seed);
        g.choose(DECK_WEAPONS);
        let w = g.choices[0] as usize - kinds::CARD_WEAPON0;
        let a = WEAPONS[w].art;
        let level = g.art_level(a);
        g.choose(0);
        assert_eq!(g.kind_of(a), Some(w), "seed {seed}");
        assert_eq!(g.art_level(a), level.max(1), "a change keeps the level");
    }
}

#[test]
fn weapon_quality_has_a_pity_and_never_goes_down() {
    let mut g = levelled(7);
    let w = Game::start_kind(A_TALISMAN).unwrap();
    // Three plain cards in a row make the next one fine steel at least.
    for seed in 1..400u32 {
        g.rng = Rng(seed | 1);
        g.dry = 3;
        assert!(g.roll_quality(w) >= kinds::STEEL);
        assert_eq!(g.dry, 0);
    }
    // A card of the kind an art already is brings a better one.
    let a = WEAPONS[w].art;
    g.take_weapon(w, kinds::PLAIN);
    for _ in 0..20 {
        let q = g.roll_quality(w);
        assert!(q > g.arts[a].quality || g.arts[a].quality == kinds::FAMOUS);
        g.take_weapon(w, q);
    }
    assert_eq!(g.arts[a].quality, kinds::FAMOUS);
    assert!(!g.weapon_offerable(w), "a famous blade of the same kind is not offered again");
    // Changing to another kind keeps the better quality.
    let other = (0..N_WEAPONS).find(|&o| o != w && WEAPONS[o].art == a).unwrap();
    g.take_weapon(other, kinds::PLAIN);
    assert_eq!(g.arts[a].quality, kinds::FAMOUS);
    assert!(g.kind_power(a) > 1.0);
}

#[test]
fn the_start_art_comes_in_its_start_kind_and_routes_suit_kinds() {
    let mut g = Game::new(1);
    g.start(M_ENDLESS);
    assert_eq!(g.kind_of(A_SCYTHE), Game::start_kind(A_SCYTHE));
    // A route that suits the kind the art is in hits harder.
    let r = g.routes_of(A_SCYTHE).next().unwrap();
    g.give(A_SCYTHE, 3, 0);
    g.take_route(A_SCYTHE, r);
    let suited = (0..N_WEAPONS).find(|&w| ROUTES[r].suits & (1 << w) != 0).unwrap();
    let plain = (0..N_WEAPONS).find(|&w| WEAPONS[w].art == A_SCYTHE && ROUTES[r].suits & (1 << w) == 0).unwrap();
    g.arts[A_SCYTHE].kind = plain as u8 + 1;
    assert!(!g.oneness(A_SCYTHE));
    g.arts[A_SCYTHE].kind = suited as u8 + 1;
    assert!(g.oneness(A_SCYTHE));
}

#[test]
fn with_routes_off_no_art_has_a_kind_and_every_factor_is_one() {
    let mut g = Game::new(1);
    g.tune[T_ROUTES] = 0.0;
    g.start(M_ENDLESS);
    g.give(A_PULSE, 3, 0);
    for a in 0..N_ARTS {
        assert_eq!(g.kind_of(a), None);
        assert_eq!((g.kind_reach(a), g.kind_beat(a), g.kind_power(a), g.kind_knock(a)), (1.0, 1.0, 1.0, 1.0));
    }
}

#[test]
fn a_fine_inner_card_brings_two_layers() {
    let mut found = false;
    for seed in 1..200 {
        let mut g = levelled(seed);
        g.choose(DECK_INNER);
        if let Some(slot) = g.ranks.iter().position(|&r| r == 1) {
            let id = g.choices[slot] as usize;
            g.choose(slot);
            assert_eq!(g.up[id], 2, "seed {seed}");
            found = true;
            break;
        }
    }
    assert!(found, "no fine card in 200 draws");
}

#[test]
fn with_one_deck_left_its_cards_come_at_once() {
    let mut g = Game::new(5);
    // Without routes there are no weapon kinds to change to.
    g.tune[T_ROUTES] = 0.0;
    g.running = true;
    // Every art held and at its top, every passive at its top: only magic is left.
    g.up = UPGRADE_MAX;
    g.pending = 1;
    g.offer();
    assert!(g.choosing && !g.deck_pick, "only magic and fate are open");
    assert!(g.choices.iter().all(|&c| c < 0 || c as usize >= N_UPGRADES));
}

#[test]
fn a_closed_deck_falls_back_to_the_first_open_one() {
    let mut g = Game::new(5);
    // Without routes there are no weapon kinds to change to.
    g.tune[T_ROUTES] = 0.0;
    g.running = true;
    for a in 0..N_ARTS {
        g.give(a, 1, 0);
    }
    g.pending = 1;
    g.offer();
    assert!(g.deck_pick);
    assert!(!g.deck_open(DECK_WEAPONS), "every art is held");
    g.choose(DECK_WEAPONS);
    assert!(!g.deck_pick);
    assert_eq!(g.last_deck as usize, DECK_ARTS);
}

#[test]
fn with_decks_off_the_old_offer_runs() {
    let mut g = Game::new(1);
    g.tune[T_DECKS] = 0.0;
    g.pending = 1;
    g.offer();
    assert!(!g.deck_pick);
    assert!(g.choices.iter().any(|&c| c >= 0));
}
