//! Tests for the inner arts (心法): their layers and the rules from the
//! second layer up. Included from `game.rs` under `#[cfg(test)]` so the tests
//! can reach private fields.

use super::*;

/// A running game with routes and inner arts on and every art parked.
fn base() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    assert!(g.inner_on());
    g
}

fn put(g: &mut Game, x: f32, y: f32) -> usize {
    let id = g.enemies.len() as u32 + 1;
    g.enemies.push(Enemy {
        x,
        y,
        kx: 0.0,
        ky: 0.0,
        hp: 1000.0,
        kind: 0,
        flash: 0.0,
        seed: 0.5,
        bead_cd: 0.0,
        tags: Default::default(),
        id,
        face: 1.0,
    });
    g.enemies.len() - 1
}

/// Take upgrade `id` once.
fn pick(g: &mut Game, id: usize) {
    g.choosing = true;
    g.pending = 1;
    g.choices = [id as i32, -1, -1];
    g.choose(0);
}

#[test]
fn an_inner_art_has_three_layers_and_the_newest_hide_without_them() {
    let mut g = base();
    for _ in 0..5 {
        pick(&mut g, U_SWIFT);
    }
    assert_eq!(g.up[U_SWIFT], 3);
    assert_eq!(g.upgrade_top(U_SWIFT), 3);
    // Off, the newest three are never offered.
    for seed in 1..40 {
        let mut g = Game::new(seed);
        g.tune[T_ROUTES] = 0.0;
        g.tune[T_ELEMENT_CARDS] = 0.0;
        g.pending = 1;
        g.offer();
        for c in g.choices {
            assert!(c < 0 || !Game::inner_only(c as usize), "seed {seed} offered {c}");
        }
    }
}

#[test]
fn layers_count_as_levels_two_four_and_five() {
    let mut g = base();
    let amount = PASSIVES.iter().find(|p| p.upgrade == U_VITALITY).unwrap().amount;
    pick(&mut g, U_VITALITY);
    assert_eq!(g.player.max_hp, 100.0 + amount * 2.0);
    pick(&mut g, U_VITALITY);
    pick(&mut g, U_VITALITY);
    assert_eq!(g.player.max_hp, 100.0 + amount * 5.0);
}

#[test]
fn copper_skin_lessens_blows_and_halves_them_near_full_health() {
    let mut g = base();
    g.up[U_VITALITY] = 2;
    assert_eq!(g.soften(10.0), 7.0);
    g.up[U_VITALITY] = 3;
    g.player.hp = g.player.max_hp;
    assert_eq!(g.soften(10.0), 3.5);
    g.player.hp = g.player.max_hp * 0.5;
    assert_eq!(g.soften(10.0), 7.0);
}

#[test]
fn crossing_on_a_reed_strides_and_dodges() {
    let mut g = base();
    g.up[U_SWIFT] = 2;
    g.inner.moving_for = 2.5;
    assert!(g.stride() > 1.0);
    g.inner.moving_for = 1.0;
    assert_eq!(g.stride(), 1.0);
    g.up[U_SWIFT] = 3;
    let dodged = (0..400).filter(|_| g.soften(10.0) == 0.0).count();
    assert!((20..120).contains(&dodged), "dodged {dodged} of 400");
}

#[test]
fn breathing_heals_faster_unhurt_and_saves_once_a_minute() {
    let mut g = base();
    g.up[U_MENDING] = 2;
    g.inner.since_hurt = 5.0;
    assert_eq!(g.mending(), 2.0);
    g.soften(1.0);
    assert_eq!(g.mending(), 1.0, "a blow resets the clock");
    g.up[U_MENDING] = 3;
    g.player.hp = -5.0;
    assert!(g.last_stand());
    assert_eq!(g.player.hp, 1.0);
    g.player.hp = -5.0;
    assert!(!g.last_stand(), "only once a minute");
}

#[test]
fn the_sleeve_gathers_every_jade() {
    let mut g = base();
    g.up[U_LURE] = 3;
    g.drop_gem(2000.0, 0.0, 1.0, false);
    assert!(!g.gems[0].pull);
    for _ in 0..(9.0 / TICK) as u32 {
        g.inner_tick(TICK);
    }
    assert!(g.gems.iter().all(|gem| gem.pull));
}

#[test]
fn borrowed_force_passes_to_the_enemies_nearby() {
    let mut g = base();
    let struck = put(&mut g, 100.0, 0.0);
    let near = put(&mut g, 130.0, 0.0);
    let far = put(&mut g, 400.0, 0.0);
    g.strike(struck, 100.0, 300.0, 0.0, 0);
    g.land_splash();
    assert_eq!(g.enemies[near].hp, 1000.0, "nothing without the inner art");
    g.up[U_JIELI] = 2;
    g.strike(struck, 100.0, 300.0, 0.0, 0);
    g.land_splash();
    assert!(g.enemies[near].hp < 1000.0);
    assert_eq!(g.enemies[far].hp, 1000.0);
    // A blow that throws nothing back passes nothing on.
    let before = g.enemies[near].hp;
    g.strike(struck, 100.0, 0.0, 0.0, 0);
    g.land_splash();
    assert_eq!(g.enemies[near].hp, before);
}

#[test]
fn the_art_of_all_methods_strengthens_element_hits() {
    let mut g = base();
    let plain = put(&mut g, 100.0, 0.0);
    g.strike(plain, 100.0, 0.0, 0.0, E_HOLY);
    let without = 1000.0 - g.enemies[plain].hp;
    g.up[U_WANFA] = 1;
    let boosted = put(&mut g, 100.0, 50.0);
    g.strike(boosted, 100.0, 0.0, 0.0, E_HOLY);
    assert!(1000.0 - g.enemies[boosted].hp > without);
}

#[test]
fn piercing_qi_sends_a_blade_after_the_last_move() {
    let mut g = base();
    g.up[U_QIGUAN] = 2;
    g.give(A_SCYTHE, 99, 0);
    let (_, len) = g.move_order(A_SCYTHE);
    for _ in 0..len {
        g.arts[A_SCYTHE].timer = 0.0;
        let before = g.shots.len();
        let last = g.arts[A_SCYTHE].step as usize + 1 == len;
        g.route_beat(A_SCYTHE, TICK, 1.0, 1.0);
        assert_eq!(g.shots.len() > before, last, "a blade only after the last move");
    }
}

#[test]
fn a_run_with_every_inner_art_is_the_same_twice() {
    let run = || {
        let mut g = Game::new(11);
        g.start(M_ENDLESS);
        for (id, level) in g.up.iter_mut().enumerate() {
            if Game::is_passive(id) {
                *level = 3;
            }
        }
        g.give(A_SCYTHE, 99, 1);
        g.give(A_PULSE, 99, 4);
        for t in 0..1800 {
            if t % 40 == 0 {
                g.input = ((t as f32 * 0.37).sin(), (t as f32 * 0.21).cos());
            }
            g.tick();
            if g.choosing {
                g.choose(0);
                if g.choosing {
                    g.choose(0);
                }
            }
        }
        g.hash()
    };
    assert_eq!(run(), run());
}
