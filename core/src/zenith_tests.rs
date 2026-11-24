//! Tests for the height of a route (極致). Included from `game.rs` under
//! `#[cfg(test)]` so the tests can reach private fields.

use super::*;

/// A running game with the scythe whole on its first route, the element
/// given, and the route's inner art held when `inner` is set.
fn whole(element: u8, inner: bool) -> (Game, usize) {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g.give(A_SCYTHE, 99, element as u32);
    let r = g.route_of(A_SCYTHE).unwrap();
    if inner {
        g.up[ROUTES[r].zenith] = 1;
    }
    (g, r)
}

fn put(g: &mut Game, kind: u8, x: f32) -> usize {
    let id = g.enemies.len() as u32 + 1;
    g.enemies.push(Enemy {
        x,
        y: 0.0,
        kx: 0.0,
        ky: 0.0,
        hp: 100_000.0,
        kind,
        flash: 0.0,
        seed: 0.5,
        bead_cd: 0.0,
        tags: Default::default(),
        id,
        face: 1.0,
    });
    g.enemies.len() - 1
}

/// Use the scythe's ultimate once and let its later hits land.
fn ultimate(g: &mut Game) {
    let (order, len) = g.move_order(A_SCYTHE);
    let last = order[..len].iter().position(|&m| m != routes::BASIC && MOVES[m as usize].ultimate).unwrap();
    g.arts[A_SCYTHE].step = last as u8;
    g.arts[A_SCYTHE].timer = 0.0;
    g.player.aim = (1.0, 0.0);
    g.route_beat(A_SCYTHE, TICK, 1.0, 1.0);
    for _ in 0..60 {
        g.move_shots(TICK);
        g.tick_delayed(TICK);
    }
}

#[test]
fn every_route_names_an_inner_art_for_its_height() {
    for (r, def) in ROUTES.iter().enumerate() {
        assert!(def.zenith < N_UPGRADES && Game::is_passive(def.zenith), "route {r}");
    }
}

#[test]
fn the_height_needs_the_whole_route_an_element_and_the_inner_art() {
    let (g, _) = whole(E_FIRE, true);
    assert_eq!(g.zenith_of(A_SCYTHE), Some(E_FIRE));
    let (g, _) = whole(E_FIRE, false);
    assert_eq!(g.zenith_of(A_SCYTHE), None, "without the inner art");
    let (g, _) = whole(0, true);
    assert_eq!(g.zenith_of(A_SCYTHE), None, "without an element");
    let (mut g, _) = whole(E_FIRE, true);
    g.up[U_SCYTHE] -= 1;
    assert_eq!(g.zenith_of(A_SCYTHE), None, "without the ultimate");
}

#[test]
fn an_ultimate_at_its_height_is_called_and_lands_harder() {
    let dealt = |inner: bool| {
        let (mut g, _) = whole(E_HOLY, inner);
        let e = put(&mut g, K_BRUTE, 60.0);
        ultimate(&mut g);
        (100_000.0 - g.enemies[e].hp, g.events.iter().any(|ev| ev[0] == EV_ZENITH))
    };
    let (plain, called_plain) = dealt(false);
    let (height, called) = dealt(true);
    assert!(!called_plain && called);
    assert!(height > plain * 1.3, "{height} against {plain}");
}

#[test]
fn metal_fills_the_guard_and_halves_blows() {
    let (mut g, _) = whole(E_METAL, true);
    put(&mut g, 0, 60.0);
    ultimate(&mut g);
    assert_eq!(g.player.guard, ELEMENTS[E_METAL as usize].cap);
    assert!(g.bulwark > 0.0);
    assert_eq!(g.soften(10.0), 5.0);
}

#[test]
fn wood_heals_on_every_hit() {
    let (mut g, _) = whole(E_WOOD, true);
    for k in 0..5 {
        put(&mut g, 0, 50.0 + k as f32 * 4.0);
    }
    g.player.hp = 50.0;
    g.player.regrow = 0.0;
    ultimate(&mut g);
    assert!(g.player.hp >= 55.0, "hp {}", g.player.hp);
}

#[test]
fn earth_holds_and_holy_fells_corpses() {
    let (mut g, _) = whole(E_EARTH, true);
    let e = put(&mut g, 0, 60.0);
    ultimate(&mut g);
    assert!(g.enemies[e].tags.left[E_EARTH as usize] >= 1.9);

    let (mut g, _) = whole(E_HOLY, true);
    let corpse = (0..N_KINDS).find(|&k| KINDS[k].corpse && KINDS[k].every <= 0.0).unwrap();
    let e = put(&mut g, corpse as u8, 60.0);
    ultimate(&mut g);
    assert!(g.enemies[e].hp <= 0.0);
}

#[test]
fn holy_leaves_a_sigil_that_fells_corpses_and_hurts_the_rest_more() {
    let (mut g, _) = whole(E_HOLY, true);
    let corpse = (0..N_KINDS).find(|&k| KINDS[k].corpse && KINDS[k].every <= 0.0).unwrap() as u8;
    let alone = (0..N_KINDS).find(|&k| KINDS[k].corpse && KINDS[k].every > 0.0).unwrap() as u8;
    let far = put(&mut g, corpse, 5000.0);
    let elite = put(&mut g, alone, 60.0);
    let brute = put(&mut g, K_BRUTE, 70.0);
    let away = put(&mut g, K_BRUTE, 5200.0);
    ultimate(&mut g);
    let s = g.inner.sigil.expect("a sigil stands");
    assert!(s.r >= 60.0);
    assert!(g.enemies[far].hp > 0.0, "outside the sigil nothing falls");
    assert!(g.enemies[elite].hp > 0.0, "one that comes alone does not fall");
    assert!(g.sigil_power(brute) > 1.0);
    assert_eq!(g.sigil_power(away), 1.0);
    // A corpse that walks in falls too, while the sigil stands.
    let late = put(&mut g, corpse, 40.0);
    g.tick_sigil(TICK);
    assert!(g.enemies[late].hp <= 0.0);
    for _ in 0..(ELEMENTS[E_HOLY as usize].time / TICK) as u32 + 2 {
        g.tick_sigil(TICK);
    }
    assert!(g.inner.sigil.is_none(), "it fades");
    assert_eq!(g.sigil_power(brute), 1.0);
    // Another element leaves no sigil.
    let (mut g, _) = whole(E_FIRE, true);
    put(&mut g, K_BRUTE, 60.0);
    ultimate(&mut g);
    assert!(g.inner.sigil.is_none());
}
