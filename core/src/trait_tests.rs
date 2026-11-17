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
