//! Tests for what weapon kinds do of their own and the routes' own rules.
//! Included from `game.rs` under `#[cfg(test)]` so the tests can reach private fields.

use super::*;

fn base() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g
}

fn put(g: &mut Game, x: f32, y: f32) -> usize {
    let id = g.enemies.len() as u32 + 1;
    g.enemies.push(Enemy {
        x,
        y,
        kx: 0.0,
        ky: 0.0,
        hp: 100_000.0,
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

fn weapon_with(t: u8) -> usize {
    (0..N_WEAPONS).find(|&w| WEAPONS[w].trait_ == t).expect("a kind with the trait")
}

fn route_with(rule: u8) -> usize {
    (0..N_ROUTES).find(|&r| ROUTES[r].rule == rule).expect("a route with the rule")
}

#[test]
fn a_hook_pulls_the_farthest_enemy_in() {
    let w = weapon_with(WT_HOOK);
    let a = WEAPONS[w].art;
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    g.arts[a].kind = w as u8 + 1;
    let far = put(&mut g, 100.0, 0.0);
    g.hook(a, Some(g.enemies[far].id));
    assert!(g.enemies[far].kx < 0.0, "pulled toward the player at the origin");
}

#[test]
fn a_soft_sword_bends_on_to_an_enemy_beside_its_tip() {
    let w = weapon_with(WT_BEND);
    let a = WEAPONS[w].art;
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    // One enemy off to the side of the thrust's tip, out of the line itself.
    let beside = put(&mut g, 100.0, 50.0);
    g.line(a, 0.0, 0.0, 1.0, 0.0, 100.0, 10.0, 10.0, 0.0);
    assert_eq!(g.enemies[beside].hp, 100_000.0, "a stiff sword misses it");
    g.arts[a].kind = w as u8 + 1;
    g.line(a, 0.0, 0.0, 1.0, 0.0, 100.0, 10.0, 10.0, 0.0);
    assert!(g.enemies[beside].hp < 100_000.0, "the soft one bends to it");
}

#[test]
fn a_paired_kind_lands_two_blows_of_half() {
    let w = (0..N_WEAPONS).find(|&w| WEAPONS[w].forms & FORM_PAIRED != 0).expect("a paired kind");
    let a = WEAPONS[w].art;
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    let first = g.second_hand(a, 10.0, 80.0, -2.0, 0.0, 1.0, 0.0);
    assert_eq!(first, 10.0, "an art without a paired kind hits once, whole");
    assert!(g.echoes.is_empty());
    g.arts[a].kind = w as u8 + 1;
    let e = put(&mut g, 40.0, 0.0);
    let first = g.second_hand(a, 10.0, 80.0, -2.0, 0.0, 1.0, 0.0);
    assert_eq!(first, 5.0);
    assert_eq!(g.echoes.len(), 1);
    let hp = g.enemies[e].hp;
    for _ in 0..(0.3 / TICK) as u32 {
        g.tick_echoes(TICK);
    }
    assert!(g.echoes.is_empty(), "the second blow landed");
    assert!(g.enemies[e].hp < hp, "and hit the enemy");
}

#[test]
fn pierce_and_swarm_change_the_shots() {
    let w = weapon_with(WT_PIERCE);
    let a = WEAPONS[w].art;
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    put(&mut g, 100.0, 0.0);
    let n = &ARTS[a].levels[0];
    g.shoot_nearest(a, n, 1.0);
    let plain = g.shots[0].pierce;
    g.shots.clear();
    g.arts[a].kind = w as u8 + 1;
    g.shoot_nearest(a, n, 1.0);
    assert_eq!(g.shots[0].pierce, plain + 1);

    let w = weapon_with(WT_SWARM);
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    for k in 0..8 {
        put(&mut g, 100.0 + k as f32 * 10.0, 0.0);
    }
    g.shoot_nearest(a, n, 1.0);
    let (count, dmg) = (g.shots.len(), g.shots[0].dmg);
    g.shots.clear();
    g.arts[a].kind = w as u8 + 1;
    g.shoot_nearest(a, n, 1.0);
    assert_eq!(g.shots.len(), (count * 2).min(8));
    assert!((g.shots[0].dmg - dmg * 0.5).abs() < 1e-4);
}

#[test]
fn a_halberd_sweeps_round_its_tip() {
    let w = weapon_with(WT_TIP);
    let a = WEAPONS[w].art;
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    let beside_tip = put(&mut g, 300.0, 45.0);
    g.line(a, 0.0, 0.0, 1.0, 0.0, 300.0, 10.0, 10.0, 0.0);
    assert_eq!(g.enemies[beside_tip].hp, 100_000.0, "a plain spear misses beside its tip");
    g.arts[a].kind = w as u8 + 1;
    g.line(a, 0.0, 0.0, 1.0, 0.0, 300.0, 10.0, 10.0, 0.0);
    assert!(g.enemies[beside_tip].hp < 100_000.0);
}

/// The art of the kind with famous blade `f`, held in that kind at `quality`.
fn famous_with(g: &mut Game, f: u8, quality: u8) -> (usize, usize) {
    let w = (0..N_WEAPONS).find(|&w| WEAPONS[w].famous == f).expect("a kind with the famous blade");
    let a = WEAPONS[w].art;
    g.up[ARTS[a].upgrade] = 1;
    g.arts[a].kind = w as u8 + 1;
    g.arts[a].quality = quality;
    (w, a)
}

#[test]
fn a_famous_blade_does_nothing_more_below_famous() {
    for f in [FM_REACH, FM_STOP, FM_KNOCK, FM_SPEED, FM_TURRET, FM_GUARD] {
        let mut g = base();
        let (_, a) = famous_with(&mut g, f, kinds::STEEL);
        assert_eq!(g.famous(a), FM_NONE);
        assert_eq!((g.kind_speed(a), g.kind_turrets(a)), (1.0, 0));
        g.famous_move(a, true);
        assert_eq!(g.player.guard, 0.0);
        assert!(g.events.is_empty());
        g.arts[a].quality = kinds::FAMOUS;
        assert_eq!(g.famous(a), f);
    }
}

#[test]
fn jade_reaches_further_and_black_gold_throws_harder() {
    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_REACH, kinds::STEEL);
    let steel = g.kind_reach(a);
    g.arts[a].quality = kinds::FAMOUS;
    let famous = g.kind_reach(a);
    // Quality alone goes 1.05 to 1.1; the jade blade adds 10% on top.
    assert!((famous / steel - 1.1 / 1.05 * 1.1).abs() < 1e-4, "{steel} {famous}");
    let fx = g.fx.len();
    g.jade_light(a, 0.0, 0.0, 100.0, 0.0);
    assert_eq!(g.fx.len(), fx + 1, "the jade blade light");

    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_KNOCK, kinds::STEEL);
    let steel = g.kind_knock(a);
    g.arts[a].quality = kinds::FAMOUS;
    assert!((g.kind_knock(a) / steel - 1.3).abs() < 1e-5);
}

#[test]
fn willow_leaf_knives_fly_faster() {
    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_SPEED, kinds::STEEL);
    put(&mut g, 100.0, 0.0);
    let n = &ARTS[a].levels[0];
    g.shoot_nearest(a, n, 1.0);
    let slow = g.shots[0].vx;
    g.shots.clear();
    g.arts[a].quality = kinds::FAMOUS;
    g.shoot_nearest(a, n, 1.0);
    assert!((g.shots[0].vx / slow - 1.25).abs() < 1e-4);
}

#[test]
fn the_famous_tang_box_sets_one_more_mechanism() {
    let m = (0..N_MOVES)
        .find(|&m| MOVES[m].shape == MS_TURRET && MOVES[m].count >= turrets::MAX_TURRETS)
        .expect("a ring of mechanisms");
    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_TURRET, kinds::STEEL);
    g.set_turrets(a, m, 1.0);
    assert_eq!(g.turrets.len(), turrets::MAX_TURRETS);
    g.turrets.clear();
    g.arts[a].quality = kinds::FAMOUS;
    g.set_turrets(a, m, 1.0);
    assert_eq!(g.turrets.len(), turrets::MAX_TURRETS + 1);
}

#[test]
fn black_iron_gauntlets_add_guard_and_duanyun_stops() {
    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_GUARD, kinds::FAMOUS);
    assert_eq!(g.arts[a].element, 0, "no metal on the art");
    g.famous_move(a, false);
    g.famous_move(a, false);
    assert_eq!(g.player.guard, 2.0);
    assert!(g.guard_cap() > 0.0);
    assert!(g.events.is_empty());

    let mut g = base();
    let (_, a) = famous_with(&mut g, FM_STOP, kinds::FAMOUS);
    g.famous_move(a, false);
    assert!(g.events.is_empty(), "only the first move of a round");
    g.famous_move(a, true);
    assert_eq!(g.events.len(), 1);
    assert_eq!((g.events[0][0], g.events[0][3]), (EV_STOP, a as f32));
}

#[test]
fn the_crowd_rule_grows_with_the_enemies_near() {
    let r = route_with(RL_CROWD);
    let mut g = base();
    assert_eq!(g.rule_power(r), 1.0);
    for k in 0..30 {
        put(&mut g, 20.0 + k as f32, 0.0);
    }
    assert!((g.rule_power(r) - 1.6).abs() < 1e-4, "at most 60%");
}

#[test]
fn an_opening_makes_the_next_blow_land_harder() {
    let r = route_with(RL_OPENING);
    let a = ROUTES[r].art;
    let mut g = base();
    g.take_route(a, r);
    g.up[ARTS[a].upgrade] = 3;
    let e = put(&mut g, 60.0, 0.0);
    g.striking = a as u8 + 1;
    g.strike(e, 10.0, 0.0, 0.0, 0);
    g.striking = 0;
    assert!(g.enemies[e].tags.opening);
    let before = g.enemies[e].hp;
    g.strike(e, 10.0, 0.0, 0.0, 0);
    assert!((before - g.enemies[e].hp - 13.0).abs() < 1e-3);
    assert!(!g.enemies[e].tags.opening, "the opening is spent");
}

#[test]
fn momentum_stacks_with_kills_and_fades() {
    let r = route_with(RL_MOMENTUM);
    let a = ROUTES[r].art;
    let mut g = base();
    g.take_route(a, r);
    g.up[ARTS[a].upgrade] = 3;
    for _ in 0..15 {
        g.note_kill();
    }
    assert_eq!(g.momentum, 10);
    assert!((g.rule_power(r) - 1.5).abs() < 1e-4);
    for _ in 0..(3.1 / TICK) as u32 {
        g.tick_momentum(TICK);
    }
    assert_eq!(g.momentum, 0);
}
