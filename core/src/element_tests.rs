//! Tests for the elements, their reactions and the cards that hand them out.
//! Numbers are read from the content tables, so settling them in
//! `content/game.toml` does not break these. Included from `game.rs` under
//! `#[cfg(test)]` so the tests can reach private fields.

use super::*;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-2
}

/// A running game with every art parked, so only what a test does happens.
fn base() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g
}

/// Place a tanky enemy (hp 1000) so damage is the hp difference.
fn put(g: &mut Game, kind: u8, x: f32, y: f32) -> usize {
    let id = g.enemies.len() as u32 + 1;
    g.enemies.push(Enemy {
        x,
        y,
        kx: 0.0,
        ky: 0.0,
        hp: 1000.0,
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

fn dealt(g: &Game, i: usize) -> f32 {
    1000.0 - g.enemies[i].hp
}

/// Let the tags run for `seconds`.
fn wait(g: &mut Game, seconds: f32) {
    for _ in 0..(seconds / TICK).round() as u32 {
        g.tick_tags(TICK);
    }
}

fn el(id: u8) -> &'static ElementDef {
    &ELEMENTS[id as usize]
}

fn tag(g: &Game, i: usize, id: u8) -> f32 {
    g.enemies[i].tags.left[id as usize]
}

fn reacted(g: &Game, reaction: usize) -> bool {
    g.events.iter().any(|e| e[0] == EV_REACT && e[3] == reaction as f32)
}

// ------------------------------------------------------------------ tables

#[test]
fn every_art_has_numbers_for_every_level() {
    for art in &ARTS {
        assert_eq!(art.levels.len(), UPGRADE_MAX[art.upgrade] as usize);
        assert!(art.start_level <= UPGRADE_MAX[art.upgrade]);
    }
}

#[test]
fn the_elements_are_the_seven_of_the_design() {
    assert_eq!(N_ELEMENTS, 7);
    assert_eq!([E_METAL, E_WOOD, E_WATER, E_FIRE, E_EARTH, E_LIGHTNING, E_HOLY], [1, 2, 3, 4, 5, 6, 7]);
    // Metal and wood act on the player and leave no tag; the other five do.
    let tags: Vec<bool> = ELEMENTS[1..].iter().map(|e| e.tags).collect();
    assert_eq!(tags, [false, false, true, true, true, true, true]);
    assert_eq!(ELEMENTS[0].effect, EF_NONE);
}

#[test]
fn a_hit_without_an_element_is_a_plain_hit() {
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 30.0, 0.0, 0);
    assert!(near(dealt(&g, e), 20.0));
    assert!(near(g.enemies[e].kx, 30.0));
    assert!((1..=N_ELEMENTS).all(|k| g.enemies[e].tags.left[k] == 0.0));
    wait(&mut g, 5.0);
    assert!(near(dealt(&g, e), 20.0));
    // An id past the table is treated as none, not as a crash.
    g.strike(e, 20.0, 0.0, 0.0, 200);
    assert!(near(dealt(&g, e), 40.0));
}

// ----------------------------------------------------------------- effects

#[test]
fn fire_burns_for_its_time_and_a_new_hit_only_restarts_it() {
    let fire = el(E_FIRE);
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    assert!(near(dealt(&g, e), 20.0));
    assert!(near(tag(&g, e, E_FIRE), fire.time));
    wait(&mut g, fire.time + 0.5);
    let burn = 20.0 * fire.power * fire.time;
    assert!(near(dealt(&g, e), 20.0 + burn), "burned {}", dealt(&g, e) - 20.0);
    assert_eq!(tag(&g, e, E_FIRE), 0.0);

    // Two hits close together burn once, from the second hit on.
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    wait(&mut g, fire.time + 0.5);
    assert!(near(dealt(&g, e), 40.0 + burn));
}

#[test]
fn water_lands_a_second_blow_after_its_delay() {
    let water = el(E_WATER);
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_WATER);
    wait(&mut g, water.time - 0.1);
    assert!(near(dealt(&g, e), 20.0), "the surge came early");
    wait(&mut g, 0.2);
    assert!(near(dealt(&g, e), 20.0 + 20.0 * water.power));
    wait(&mut g, 3.0);
    assert!(near(dealt(&g, e), 20.0 + 20.0 * water.power), "it surged twice");

    // A second hit while one waits neither delays it nor adds a second one;
    // the larger of the two is what goes off.
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_WATER);
    wait(&mut g, water.time * 0.5);
    g.strike(e, 30.0, 0.0, 0.0, E_WATER);
    wait(&mut g, water.time * 0.5 + 0.05);
    assert!(near(dealt(&g, e), 50.0 + 30.0 * water.power));
    wait(&mut g, 3.0);
    assert!(near(dealt(&g, e), 50.0 + 30.0 * water.power));
}

#[test]
fn earth_throws_the_enemy_back_and_holds_it() {
    let earth = el(E_EARTH);
    let mut g = base();
    g.spawn_acc = -1000.0;
    let e = put(&mut g, K_BLOT, 100.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_EARTH);
    assert!(near(g.enemies[e].kx, earth.power), "pushed straight away from the player");
    assert!(g.enemies[e].tags.rooted());
    // Once the push has died down it stands still instead of walking in.
    g.enemies[e].kx = 0.0;
    let x = g.enemies[e].x;
    g.move_enemies(TICK);
    assert_eq!(g.enemies[e].x, x);
    wait(&mut g, earth.time + 0.05);
    assert!(!g.enemies[e].tags.rooted());
    g.move_enemies(TICK);
    assert!(g.enemies[e].x < x, "it walks at the player again");

    // It cannot be rooted again straight away.
    g.strike(e, 20.0, 0.0, 0.0, E_EARTH);
    assert!(!g.enemies[e].tags.rooted());
    assert_eq!(g.enemies[e].kx, 0.0);
    wait(&mut g, earth.rest + 0.05);
    g.strike(e, 20.0, 0.0, 0.0, E_EARTH);
    assert!(g.enemies[e].tags.rooted());
}

#[test]
fn lightning_jumps_to_the_nearest_enemies_in_reach() {
    let bolt = el(E_LIGHTNING);
    let mut g = base();
    let hit = put(&mut g, K_BLOT, 300.0, 0.0);
    let mut near_ones = Vec::new();
    for k in 0..bolt.count {
        near_ones.push(put(&mut g, K_BLOT, 300.0 + 20.0 * (k + 1) as f32, 0.0));
    }
    // One more in reach than it can jump to, and one out of reach.
    let spare = put(&mut g, K_BLOT, 300.0, bolt.reach - 5.0);
    let far = put(&mut g, K_BLOT, 300.0 + bolt.reach + 60.0, 0.0);
    g.strike(hit, 20.0, 0.0, 0.0, E_LIGHTNING);
    assert!(near(dealt(&g, hit), 20.0));
    for &j in &near_ones {
        assert!(near(dealt(&g, j), 20.0 * bolt.power));
        assert!(tag(&g, j, E_LIGHTNING) > 0.0);
    }
    assert_eq!(dealt(&g, spare), 0.0);
    assert_eq!(dealt(&g, far), 0.0);
    assert!(near(tag(&g, hit, E_LIGHTNING), bolt.time));
}

#[test]
fn holy_hits_corpses_hardest() {
    let holy = el(E_HOLY);
    let mut g = base();
    let corpse = (0..KINDS.len() as u8).find(|&k| KINDS[k as usize].corpse).expect("no corpse kind");
    let other = (0..KINDS.len() as u8).find(|&k| !KINDS[k as usize].corpse).expect("no other kind");
    let a = put(&mut g, corpse, 50.0, 0.0);
    let b = put(&mut g, other, 0.0, 50.0);
    g.strike(a, 20.0, 0.0, 0.0, E_HOLY);
    g.strike(b, 20.0, 0.0, 0.0, E_HOLY);
    assert!(near(dealt(&g, a), 20.0 * (1.0 + holy.power)));
    assert!(near(dealt(&g, b), 20.0 * (1.0 + holy.minor)));
    assert!(holy.power > holy.minor);
    assert!(tag(&g, a, E_HOLY) > 0.0);
}

#[test]
fn metal_builds_a_guard_that_takes_the_blow_and_fades() {
    let metal = el(E_METAL);
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_METAL);
    assert!(near(g.player.guard, metal.power));
    assert_eq!(tag(&g, e, E_METAL), 0.0, "metal leaves no tag");
    for _ in 0..100 {
        g.strike(e, 1.0, 0.0, 0.0, E_METAL);
    }
    assert!(near(g.player.guard, metal.cap));
    assert!(near(g.guard_cap(), metal.cap));

    // A blow smaller than the guard costs no health; a larger one costs the rest.
    assert_eq!(g.guard_against(metal.cap - 1.0), 0.0);
    assert!(near(g.player.guard, 1.0));
    assert!(near(g.guard_against(10.0), 9.0));
    assert_eq!(g.player.guard, 0.0);
    assert_eq!(g.guard_against(10.0), 10.0);
    assert_eq!(g.guard_cap(), 0.0);

    // Left alone, the guard holds for a moment and then wears off.
    g.strike(e, 20.0, 0.0, 0.0, E_METAL);
    for _ in 0..((metal.time - 0.1) / TICK) as u32 {
        g.tick_boons(TICK);
    }
    assert!(near(g.player.guard, metal.power));
    for _ in 0..(60.0 * (0.2 + metal.power / metal.rate)) as u32 {
        g.tick_boons(TICK);
    }
    assert_eq!(g.player.guard, 0.0);
}

#[test]
fn an_enemy_touching_the_player_wears_the_guard_first() {
    let mut g = base();
    g.spawn_acc = -1000.0;
    g.player.guard = 8.0;
    g.player.guard_element = E_METAL;
    put(&mut g, K_BLOT, 5.0, 0.0);
    g.move_enemies(TICK);
    // A blot hits for 5: all of it lands on the guard.
    assert_eq!(g.player.hp, 100.0);
    assert!(near(g.player.guard, 3.0));
    g.player.iframe = 0.0;
    g.move_enemies(TICK);
    assert!(near(g.player.hp, 98.0));
    assert_eq!(g.player.guard, 0.0);
}

#[test]
fn wood_heals_slowly_for_its_time() {
    let wood = el(E_WOOD);
    let mut g = base();
    g.player.hp = 40.0;
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_WOOD);
    assert_eq!(tag(&g, e, E_WOOD), 0.0, "wood leaves no tag");
    // Hitting again in the meantime only restarts the time.
    g.strike(e, 20.0, 0.0, 0.0, E_WOOD);
    for _ in 0..((wood.time + 1.0) / TICK) as u32 {
        g.tick_boons(TICK);
    }
    // One step more or less of healing, depending on how the time rounds.
    assert!((g.player.hp - (40.0 + wood.power * wood.time)).abs() < 0.05, "hp {}", g.player.hp);
    assert_eq!(g.player.regrow, 0.0);

    // It never heals past full health.
    g.player.hp = g.player.max_hp - 0.5;
    g.strike(e, 20.0, 0.0, 0.0, E_WOOD);
    for _ in 0..120 {
        g.tick_boons(TICK);
    }
    assert_eq!(g.player.hp, g.player.max_hp);
}

// --------------------------------------------------------------- reactions

#[test]
fn fire_and_lightning_blast_everything_nearby_and_end_the_burn() {
    let r = &REACTIONS[R_BLAST];
    for (first, second) in [(E_FIRE, E_LIGHTNING), (E_LIGHTNING, E_FIRE)] {
        let mut g = base();
        let e = put(&mut g, K_BLOT, 400.0, 0.0);
        // Out of the lightning's reach, so only the blast can touch it.
        let far = put(&mut g, K_BLOT, 400.0, 2000.0);
        g.strike(e, 20.0, 0.0, 0.0, first);
        assert!(!reacted(&g, R_BLAST));
        g.strike(e, 20.0, 0.0, 0.0, second);
        assert!(reacted(&g, R_BLAST), "{first} then {second}");
        assert_eq!(tag(&g, e, E_FIRE), 0.0);
        assert_eq!(tag(&g, e, E_LIGHTNING), 0.0);
        assert_eq!(dealt(&g, far), 0.0);
        let before = dealt(&g, e);
        assert!(before >= 40.0 + 20.0 * r.power - 0.01, "the struck enemy is in its own blast");
        // The burn is over: nothing more happens to it.
        wait(&mut g, 5.0);
        assert!(near(dealt(&g, e), before));
    }
}

#[test]
fn a_blast_hits_enemies_within_its_reach_only() {
    let r = &REACTIONS[R_BLAST];
    let mut g = base();
    let e = put(&mut g, K_BLOT, 400.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    // Both stand outside the lightning's reach, so only the blast can reach them.
    let reach = el(E_LIGHTNING).reach.max(r.reach);
    let inside = put(&mut g, K_BLOT, 400.0, r.reach - 5.0);
    let outside = put(&mut g, K_BLOT, 400.0, -(reach + 80.0));
    g.strike(e, 20.0, 0.0, 0.0, E_LIGHTNING);
    if r.reach - 5.0 > el(E_LIGHTNING).reach {
        assert!(near(dealt(&g, inside), 20.0 * r.power));
    } else {
        assert!(dealt(&g, inside) >= 20.0 * r.power - 0.01);
    }
    assert_eq!(dealt(&g, outside), 0.0);
}

#[test]
fn water_and_lightning_set_the_surge_off_at_once_and_pass_it_on() {
    let r = &REACTIONS[R_CONDUCT];
    let (water, bolt) = (el(E_WATER), el(E_LIGHTNING));
    let mut g = base();
    let e = put(&mut g, K_BLOT, 400.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_WATER);
    let near_one = put(&mut g, K_BLOT, 420.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_LIGHTNING);
    assert!(reacted(&g, R_CONDUCT));
    let surge = 20.0 * water.power * r.power;
    assert!(near(dealt(&g, e), 40.0 + surge), "dealt {}", dealt(&g, e));
    // The neighbour takes the lightning's jump and then the passed-on surge.
    assert!(near(dealt(&g, near_one), 20.0 * bolt.power + surge));
    assert_eq!(tag(&g, e, E_WATER), 0.0);
    assert_eq!(tag(&g, e, E_LIGHTNING), 0.0);
    // The surge is spent: it does not go off a second time.
    wait(&mut g, 3.0);
    assert!(near(dealt(&g, e), 40.0 + surge));
}

#[test]
fn water_on_a_shocked_enemy_conducts_too() {
    let r = &REACTIONS[R_CONDUCT];
    let mut g = base();
    let e = put(&mut g, K_BLOT, 400.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_LIGHTNING);
    g.strike(e, 20.0, 0.0, 0.0, E_WATER);
    assert!(reacted(&g, R_CONDUCT));
    assert!(near(dealt(&g, e), 40.0 + 20.0 * el(E_WATER).power * r.power));
}

#[test]
fn water_and_fire_make_steam_and_end_both() {
    let r = &REACTIONS[R_STEAM];
    for (first, second) in [(E_WATER, E_FIRE), (E_FIRE, E_WATER)] {
        let mut g = base();
        let e = put(&mut g, K_BLOT, 400.0, 0.0);
        let by = put(&mut g, K_BLOT, 400.0, r.reach - 5.0);
        g.strike(e, 20.0, 0.0, 0.0, first);
        g.strike(e, 20.0, 0.0, 0.0, second);
        assert!(reacted(&g, R_STEAM), "{first} then {second}");
        assert!(near(dealt(&g, by), 20.0 * r.power));
        assert_eq!(tag(&g, e, E_WATER), 0.0);
        assert_eq!(tag(&g, e, E_FIRE), 0.0);
        let before = dealt(&g, e);
        wait(&mut g, 5.0);
        assert!(near(dealt(&g, e), before), "neither the surge nor the burn goes on");
    }
}

#[test]
fn earth_and_holy_have_no_reaction() {
    for other in [E_WATER, E_FIRE, E_LIGHTNING, E_EARTH, E_HOLY] {
        for quiet in [E_EARTH, E_HOLY] {
            let mut g = base();
            let e = put(&mut g, K_BLOT, 400.0, 0.0);
            g.strike(e, 20.0, 0.0, 0.0, other);
            g.strike(e, 20.0, 0.0, 0.0, quiet);
            assert!(!g.events.iter().any(|ev| ev[0] == EV_REACT), "{other} with {quiet}");
        }
    }
}

#[test]
fn a_reaction_needs_both_tags_again_before_it_fires_again() {
    let mut g = base();
    let e = put(&mut g, K_BLOT, 400.0, 0.0);
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    g.strike(e, 20.0, 0.0, 0.0, E_LIGHTNING);
    g.events.clear();
    g.strike(e, 20.0, 0.0, 0.0, E_LIGHTNING);
    assert!(!reacted(&g, R_BLAST), "the fire tag was spent");
    g.strike(e, 20.0, 0.0, 0.0, E_FIRE);
    assert!(reacted(&g, R_BLAST));
}

// ------------------------------------------------------------ arts carrying

#[test]
fn an_art_passes_its_element_on_to_what_it_hits() {
    // The scythe (arc).
    let mut g = base();
    let e = put(&mut g, K_BLOT, 50.0, 0.0);
    g.arts[A_SCYTHE].element = E_FIRE;
    g.arts[A_SCYTHE].timer = 0.0;
    g.weapons(TICK);
    assert!(tag(&g, e, E_FIRE) > 0.0);
    // Its slash is drawn in the element's colour.
    assert_eq!(&g.fx[0].color[..3], &el(E_FIRE).color[..]);

    // A talisman (projectile) carries it in flight.
    let mut g = base();
    g.up[U_TALISMAN] = 1;
    g.arts[A_TALISMAN].element = E_WATER;
    g.arts[A_TALISMAN].timer = 0.0;
    let e = put(&mut g, K_BLOT, 100.0, 0.0);
    g.weapons(TICK);
    for _ in 0..30 {
        g.move_shots(TICK);
    }
    assert!(tag(&g, e, E_WATER) > 0.0);

    // The beads (orbit) and the pulse (nova).
    let mut g = base();
    g.up[U_BEADS] = 1;
    g.arts[A_BEADS].element = E_HOLY;
    let (bx, by) = g.orbit_at(A_BEADS, 0);
    let e = put(&mut g, K_BLOT, bx, by);
    g.weapons(TICK);
    assert!(tag(&g, e, E_HOLY) > 0.0);

    let mut g = base();
    g.up[U_PULSE] = 1;
    g.arts[A_PULSE].element = E_EARTH;
    g.arts[A_PULSE].timer = 0.0;
    let e = put(&mut g, K_BLOT, 100.0, 0.0);
    g.weapons(TICK);
    assert!(g.enemies[e].tags.rooted());
}

#[test]
fn two_arts_with_a_pair_of_elements_react_in_play() {
    let mut g = base();
    g.spawn_acc = -1000.0;
    g.up[U_PULSE] = 1;
    g.arts[A_SCYTHE].element = E_FIRE;
    g.arts[A_PULSE].element = E_LIGHTNING;
    g.arts[A_SCYTHE].timer = 0.0;
    g.arts[A_PULSE].timer = 0.0;
    put(&mut g, K_BRUTE, 80.0, 0.0);
    g.weapons(TICK);
    assert!(reacted(&g, R_BLAST));
}

// ------------------------------------------------------------------- cards

#[test]
fn an_element_card_is_offered_as_the_last_card_at_its_chance() {
    let mut seen = 0;
    for seed in 1..400u32 {
        let mut g = Game::new(seed);
        g.pending = 1;
        g.offer();
        assert!(g.choices[0] < N_UPGRADES as i32 && g.choices[1] < N_UPGRADES as i32);
        if g.choices[2] >= N_UPGRADES as i32 {
            assert!((g.choices[2] as usize) < N_UPGRADES + N_ELEMENTS);
            seen += 1;
        }
    }
    let share = seen as f32 / 399.0;
    let want = TUNE_DEFAULT[T_ELEMENT_CARDS];
    assert!((share - want).abs() < 0.1, "element cards in {share} of offers, wanted about {want}");

    // Always, and never.
    for (chance, always) in [(1.0, true), (0.0, false)] {
        for seed in 1..50u32 {
            let mut g = Game::new(seed);
            g.tune[T_ELEMENT_CARDS] = chance;
            g.pending = 1;
            g.offer();
            assert_eq!(g.choices[2] >= N_UPGRADES as i32, always, "chance {chance}, seed {seed}");
        }
    }
}

#[test]
fn an_element_card_takes_the_first_empty_slot_when_upgrades_run_short() {
    let mut g = Game::new(3);
    g.tune[T_ELEMENT_CARDS] = 1.0;
    g.up = UPGRADE_MAX;
    g.up[U_HASTE] -= 1;
    g.pending = 1;
    g.offer();
    assert_eq!(g.choices[0], U_HASTE as i32);
    assert!(g.choices[1] >= N_UPGRADES as i32);
    assert_eq!(g.choices[2], -1);
}

#[test]
fn picking_an_element_card_asks_which_art_gets_it() {
    let mut g = Game::new(3);
    g.choosing = true;
    g.pending = 1;
    g.choices = [U_MIGHT as i32, -1, (N_UPGRADES + E_FIRE as usize - 1) as i32];
    g.choose(2);
    assert!(g.choosing, "the choice stays open");
    assert_eq!(g.attach, E_FIRE);
    assert_eq!(g.choices, [-1; 3]);
    assert_eq!(g.pending, 1, "the level-up is not spent yet");

    // An art that is not held cannot take it, nor can an id past the table.
    assert_eq!(g.up[U_PULSE], 0);
    g.choose(A_PULSE);
    g.choose(N_ARTS + 2);
    assert!(g.choosing && g.attach == E_FIRE);

    g.choose(A_SCYTHE);
    assert_eq!(g.arts[A_SCYTHE].element, E_FIRE);
    assert_eq!(g.attach, 0);
    assert!(!g.choosing);
    assert_eq!(g.pending, 0);
    assert!(g.events.iter().any(|e| e[0] == EV_ATTACH && e[3] == E_FIRE as f32));
    assert_eq!(g.up[U_MIGHT], 0, "no upgrade was taken along the way");
}

#[test]
fn a_new_element_replaces_the_one_an_art_carried() {
    let mut g = Game::new(3);
    g.arts[A_SCYTHE].element = E_FIRE;
    g.choosing = true;
    g.pending = 1;
    g.choices = [(N_UPGRADES + E_WATER as usize - 1) as i32, -1, -1];
    g.choose(0);
    g.choose(A_SCYTHE);
    assert_eq!(g.arts[A_SCYTHE].element, E_WATER);
}

#[test]
fn the_state_shows_the_attach_step_and_what_each_art_carries() {
    let mut g = Game::new(3);
    g.running = true;
    g.arts[A_SCYTHE].element = E_HOLY;
    g.choosing = true;
    g.pending = 1;
    g.choices = [(N_UPGRADES + E_EARTH as usize - 1) as i32, -1, -1];
    g.present();
    assert_eq!(g.state[S_CHOICE0], (N_UPGRADES + E_EARTH as usize - 1) as f32);
    assert_eq!(g.state[S_ATTACH], 0.0);
    g.choose(0);
    g.player.guard = 7.0;
    g.player.guard_element = E_METAL;
    g.player.regrow = 1.5;
    g.present();
    assert_eq!(g.state[S_CHOOSING], 1.0);
    assert_eq!(g.state[S_ATTACH], E_EARTH as f32);
    assert_eq!(g.state[S_ART_ELEMENT0 + A_SCYTHE], E_HOLY as f32);
    assert_eq!(g.state[S_ART_ELEMENT0 + A_PULSE], 0.0);
    assert_eq!(g.state[S_GUARD], 7.0);
    assert_eq!(g.state[S_GUARD_MAX], el(E_METAL).cap);
    assert_eq!(g.state[S_REGROW], 1.5);
}

#[test]
fn the_start_element_tune_puts_an_element_on_the_first_art() {
    let mut g = Game::new(3);
    g.start(M_ENDLESS);
    assert!(g.arts.iter().all(|a| a.element == 0));

    let mut g = Game::new(3);
    g.set_tune(T_START_ELEMENT, E_LIGHTNING as f32);
    g.start(M_ENDLESS);
    assert_eq!(g.arts[A_SCYTHE].element, E_LIGHTNING);
    assert!(g.arts[1..].iter().all(|a| a.element == 0));
}

#[test]
fn the_held_arts_are_counted_for_the_slot_limit() {
    // The content holds fewer arts than there are slots, so the limit cannot
    // be reached in a real run yet; this checks the count the limit uses.
    let mut g = Game::new(3);
    assert_eq!(g.arts_held(), ARTS.iter().filter(|a| a.start_level > 0).count());
    g.up[U_BEADS] = 2;
    g.up[U_PULSE] = 1;
    assert_eq!(g.arts_held(), 3);
}

// ------------------------------------------------------------------- runs

#[test]
fn a_run_with_an_element_on_every_art_holds_up() {
    let mut reactions = 0;
    for element in 1..=N_ELEMENTS as u8 {
        let mut g = Game::new(20 + element as u32);
        g.set_view(640.0, 360.0);
        g.tune[T_ELEMENT_CARDS] = 1.0;
        g.start(M_ENDLESS);
        g.up[U_TALISMAN] = 3;
        g.up[U_BEADS] = 3;
        g.up[U_PULSE] = 3;
        for (a, art) in g.arts.iter_mut().enumerate() {
            // Mix the elements so reactions happen too.
            art.element = ((element as usize + a - 1) % N_ELEMENTS + 1) as u8;
        }
        let mut bot = Rng(77 + element as u32);
        for frame in 0..(60 * 90) {
            if frame % 30 == 0 {
                g.input = (bot.range(-1.0, 1.0), bot.range(-1.0, 1.0));
            }
            g.update(TICK);
            reactions += g.events.iter().filter(|e| e[0] == EV_REACT).count();
            assert!(g.inst.len() <= MAX_INST);
            if g.choosing {
                g.choose((bot.next_u32() % 3) as usize);
                if g.choosing {
                    g.choose(0);
                }
            }
            if g.over {
                g.over = false;
                g.player.hp = g.player.max_hp;
            }
            assert!(g.player.hp.is_finite() && g.player.guard.is_finite());
            for e in &g.enemies {
                assert!(e.hp.is_finite() && e.x.is_finite() && e.kx.is_finite());
            }
        }
        assert!(g.kills > 20, "element {element}: {} kills", g.kills);
    }
    assert!(reactions > 0, "no reaction in any of the runs");
}
