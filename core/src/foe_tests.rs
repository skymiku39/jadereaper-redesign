//! Tests for the school elites and their warned moves. Included from
//! `game.rs` under `#[cfg(test)]` so the tests can reach private fields.

use super::*;

/// A running game with one elite of kind `k` at (x, 0) and the player at the origin.
fn with_foe(k: usize, x: f32) -> Game {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g.spawn_enemy_at(k as u8, x, 0.0);
    g
}

fn foe_kinds() -> Vec<usize> {
    (0..N_KINDS).filter(|&k| KINDS[k].strike < N_MOVES).collect()
}

/// Step the foes and the enemies' walk for `seconds`.
fn run(g: &mut Game, seconds: f32) {
    for _ in 0..(seconds / TICK) as u32 {
        g.move_enemies(TICK);
        g.foes_step(TICK);
    }
}

#[test]
fn every_elite_belongs_to_a_school_and_comes_alone() {
    let kinds = foe_kinds();
    assert!(!kinds.is_empty());
    for k in kinds {
        assert!(KINDS[k].school < N_SCHOOLS, "kind {k}");
        assert!(KINDS[k].every > 0.0, "kind {k}");
    }
}

#[test]
fn an_elite_warns_stands_still_and_then_lands_its_move() {
    for k in foe_kinds() {
        let reach = MOVES[KINDS[k].strike].reach.abs() * 100.0;
        let mut g = with_foe(k, reach * 0.6);
        g.player.hp = 1000.0;
        g.player.max_hp = 1000.0;
        run(&mut g, 1.6);
        assert!(g.events.iter().any(|e| e[0] == EV_FOE), "kind {k} never warned");
        let i = g.enemies.iter().position(|e| KINDS[e.kind as usize].strike < N_MOVES).unwrap();
        assert!(g.winding(g.enemies[i].id), "kind {k} is winding up");
        let at = (g.enemies[i].x, g.enemies[i].y);
        g.events.clear();
        run(&mut g, 0.3);
        assert_eq!((g.enemies[i].x, g.enemies[i].y), at, "kind {k} stood still while winding up");
        run(&mut g, 1.0);
        assert!(g.player.hp < 1000.0, "kind {k} missed a player standing in its move");
    }
}

#[test]
fn stepping_out_of_the_warning_dodges_the_move() {
    for k in foe_kinds() {
        let reach = MOVES[KINDS[k].strike].reach.abs() * 100.0;
        let mut g = with_foe(k, reach * 0.6);
        g.player.hp = 1000.0;
        g.player.max_hp = 1000.0;
        run(&mut g, 1.6);
        // The warning is up: walk far out of it before it lands.
        g.player.x = -reach * 3.0;
        let before = g.player.hp;
        run(&mut g, 0.8);
        assert_eq!(g.player.hp, before, "kind {k} hit a player who stepped away");
    }
}

#[test]
fn a_pulling_move_drags_the_player_in() {
    let Some(k) = foe_kinds().into_iter().find(|&k| MOVES[KINDS[k].strike].knockback < 0.0) else { return };
    let reach = MOVES[KINDS[k].strike].reach.abs() * 100.0;
    let mut g = with_foe(k, reach * 0.8);
    g.player.hp = 1000.0;
    g.player.max_hp = 1000.0;
    let i = 0;
    run(&mut g, 2.6);
    let (dx, dy) = (g.player.x - g.enemies[i].x, g.player.y - g.enemies[i].y);
    assert!((dx * dx + dy * dy).sqrt() < reach * 0.8, "the player was not pulled in");
}

#[test]
fn elites_only_come_while_disciples_do() {
    let mut g = Game::new(1);
    g.tune[T_SCHOOL_SHARE] = 0.0;
    g.start(M_ENDLESS);
    g.running = true;
    for _ in 0..(200.0 / TICK) as u32 {
        g.spawn(TICK);
        g.time += TICK;
    }
    assert!(!g.enemies.iter().any(|e| KINDS[e.kind as usize].strike < N_MOVES));
}
