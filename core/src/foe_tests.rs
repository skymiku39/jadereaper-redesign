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
        let reach = foes::foe_reach(foes::foe_move(k, 0));
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
        // Shots and stakes take a moment more to reach him.
        run(&mut g, 2.0);
        assert!(g.player.hp < 1000.0, "kind {k} missed a player standing in its move");
    }
}

#[test]
fn stepping_out_of_the_warning_dodges_the_move() {
    for k in foe_kinds() {
        let reach = foes::foe_reach(foes::foe_move(k, 0));
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
    let Some(k) = foe_kinds().into_iter().find(|&k| MOVES[foes::foe_move(k, 0)].knockback < 0.0) else { return };
    let reach = foes::foe_reach(foes::foe_move(k, 0));
    let mut g = with_foe(k, reach * 0.8);
    g.player.hp = 1000.0;
    g.player.max_hp = 1000.0;
    let i = 0;
    run(&mut g, 2.6);
    let (dx, dy) = (g.player.x - g.enemies[i].x, g.player.y - g.enemies[i].y);
    assert!((dx * dx + dy * dy).sqrt() < reach * 0.8, "the player was not pulled in");
}

#[test]
fn every_master_goes_through_its_whole_route_ultimate_last() {
    let masters: Vec<usize> = (0..N_KINDS).filter(|&k| KINDS[k].manual && KINDS[k].strike < N_MOVES).collect();
    assert!(masters.len() >= 5, "a master for each school that has one");
    for k in masters {
        master_goes_through_its_route(k);
    }
    // An elite that is not a master and has one move always uses it.
    let e = foe_kinds().into_iter().find(|&k| !KINDS[k].manual && KINDS[k].strike2 >= N_MOVES).expect("an elite");
    assert_eq!(foes::foe_move(e, 3), KINDS[e].strike);
}

fn master_goes_through_its_route(k: usize) {
    let r = &ROUTES[MOVES[KINDS[k].strike].route];
    let used: Vec<usize> = (0..r.n as u8 * 2).map(|s| foes::foe_move(k, s)).collect();
    assert_eq!(used[0], r.first, "it begins with the route's first move");
    assert_eq!(used[r.n - 1], r.first + r.n - 1, "and ends with its last");
    assert!(MOVES[used[r.n - 1]].ultimate, "the ultimate");
    assert_eq!(used[r.n], r.first, "then round again");
    // Playing it: the moves it calls out follow the route.
    let reach = foes::foe_reach(r.first);
    let mut g = with_foe(k, reach * 0.5);
    g.player.hp = 1e6;
    g.player.max_hp = 1e6;
    let mut called = Vec::new();
    for _ in 0..(30.0 / TICK) as u32 {
        g.events.clear();
        g.foes_step(TICK);
        g.player.iframe = 0.0;
        // Keep the player in reach of every move.
        let (ex, ey) = (g.enemies[0].x, g.enemies[0].y);
        g.player.x = ex + 20.0;
        g.player.y = ey;
        called.extend(g.events.iter().filter(|e| e[0] == EV_FOE).map(|e| e[3] as usize));
    }
    assert!(called.len() > r.n, "kind {k} used more than the whole route: {called:?}");
    for (s, &m) in called.iter().enumerate() {
        assert_eq!(m, r.first + s % r.n, "kind {k} move {s}");
    }
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

#[test]
fn warnings_are_short_for_elites_and_long_for_a_masters_ultimate() {
    // design 4.8.6: about 0.6 s before a move lands, 1.2 s before an ultimate.
    for k in foe_kinds() {
        let r = &ROUTES[MOVES[KINDS[k].strike].route];
        let steps: Vec<u8> = if KINDS[k].manual { vec![0, r.n as u8 - 1] } else { vec![0] };
        for step in steps {
            let m = foes::foe_move(k, step);
            let mut g = with_foe(k, foes::foe_reach(m) * 0.5);
            g.player.hp = 1e6;
            g.player.max_hp = 1e6;
            g.foes_step(TICK);
            g.foes[0].step = step;
            g.foes[0].rest = 0.0;
            g.foes_step(TICK);
            let want = if MOVES[m].ultimate && KINDS[k].manual { 1.2 } else { 0.6 };
            assert!((g.foes[0].wind - want).abs() < 0.01, "kind {k} step {step}: {}", g.foes[0].wind);
        }
    }
}

#[test]
fn an_elite_with_two_moves_takes_turns() {
    let k =
        foe_kinds().into_iter().find(|&k| !KINDS[k].manual && KINDS[k].strike2 < N_MOVES).expect("a two-move elite");
    let (a, b) = (KINDS[k].strike, KINDS[k].strike2);
    assert_eq!((foes::foe_move(k, 0), foes::foe_move(k, 1), foes::foe_move(k, 2)), (a, b, a));
    let mut g = with_foe(k, 60.0);
    g.player.hp = 1e6;
    g.player.max_hp = 1e6;
    let mut called = Vec::new();
    for _ in 0..(16.0 / TICK) as u32 {
        g.events.clear();
        g.foes_step(TICK);
        g.player.iframe = 0.0;
        let (ex, ey) = (g.enemies[0].x, g.enemies[0].y);
        g.player.x = ex + 20.0;
        g.player.y = ey;
        called.extend(g.events.iter().filter(|e| e[0] == EV_FOE).map(|e| e[3] as usize));
    }
    assert!(called.len() >= 3, "{called:?}");
    for (s, &m) in called.iter().enumerate() {
        assert_eq!(m, if s % 2 == 0 { a } else { b }, "move {s}");
    }
}

/// The first kind whose `strike` (or first move, for a master) has shape `shape`.
fn kind_using(shape: u8, master: bool) -> usize {
    foe_kinds()
        .into_iter()
        .find(|&k| KINDS[k].manual == master && MOVES[foes::foe_move(k, 0)].shape == shape)
        .expect("a kind using the shape")
}

#[test]
fn a_thrown_volley_flies_and_hurts_the_player() {
    let k = kind_using(MS_WAVE, false);
    let m = foes::foe_move(k, 0);
    let mut g = with_foe(k, foes::foe_reach(m) * 0.5);
    g.player.hp = 1000.0;
    g.player.max_hp = 1000.0;
    // Into the warning, then the shots go.
    run(&mut g, 2.2);
    assert!(!g.foe_shots.is_empty(), "the volley is in the air");
    assert!(g.foe_shots.len() <= MOVES[m].count.max(1));
    run(&mut g, 1.5);
    assert!(g.player.hp < 1000.0, "a shot hit");
}

#[test]
fn a_mechanist_sets_down_a_stake_that_shoots_then_ends() {
    let k = kind_using(MS_TURRET, false);
    let m = foes::foe_move(k, 0);
    let mut g = with_foe(k, 200.0);
    g.player.hp = 1000.0;
    g.player.max_hp = 1000.0;
    run(&mut g, 2.2);
    assert_eq!(g.stakes.len(), 1, "one stake at its feet");
    let at = (g.stakes[0].x, g.stakes[0].y);
    run(&mut g, 0.6);
    assert!(!g.foe_shots.is_empty() || g.player.hp < 1000.0, "the stake shoots");
    assert_eq!((g.stakes[0].x, g.stakes[0].y), at, "a plain stake stays put");
    run(&mut g, MOVES[m].life + 0.2);
    // The first stake ended; at most a newer one stands.
    assert!(g.stakes.len() <= 1 && g.stakes.iter().all(|s| s.life < MOVES[m].life - 1.0), "the first stake ended");
}

#[test]
fn a_masters_ring_of_stakes_nets_the_player_and_slows_him() {
    let k = kind_using(MS_TURRET, true);
    let r = &ROUTES[MOVES[KINDS[k].strike].route];
    let last = (r.n - 1) as u8;
    let m = foes::foe_move(k, last);
    assert!(MOVES[m].ultimate && MOVES[m].count > 1 && MOVES[m].slow > 0.0);
    let mut g = with_foe(k, 150.0);
    g.player.hp = 1e6;
    g.player.max_hp = 1e6;
    g.foes_step(TICK);
    g.foes[0].step = last;
    g.foes[0].rest = 0.0;
    run(&mut g, 1.3);
    assert_eq!(g.stakes.len(), MOVES[m].count.min(foes::MAX_STAKES), "a ring of stakes");
    assert!(g.foe_net.is_some(), "and a net");
    // Walking inside the net is slower than outside it.
    let walked = |g: &mut Game| {
        let x0 = g.player.x;
        g.input = (1.0, 0.0);
        g.walk(TICK);
        g.input = (0.0, 0.0);
        g.player.x - x0
    };
    let n = g.foe_net.unwrap();
    g.player.x = n.x;
    g.player.y = n.y;
    let slow = walked(&mut g);
    g.player.x = n.x + n.r * 3.0;
    let fast = walked(&mut g);
    assert!(slow < fast * 0.9, "{slow} against {fast}");
    // The net comes down with the stakes.
    run(&mut g, MOVES[m].life + 0.2);
    assert!(g.foe_net.is_none());
}

#[test]
fn a_tang_disciple_throws_from_afar() {
    let k = (0..N_KINDS).find(|&k| KINDS[k].throw > 0.0).expect("a thrower");
    assert!(KINDS[k].strike >= N_MOVES, "a disciple, not an elite");
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g.spawn_enemy_at(k as u8, 300.0, 0.0);
    g.player.hp = 1000.0;
    g.player.max_hp = 1000.0;
    let at = g.enemies[0].x;
    for _ in 0..((KINDS[k].throw * 2.0 + 1.5) / TICK) as u32 {
        g.time += TICK;
        g.foes_step(TICK);
    }
    // Held where it is, so only a dart can reach the player.
    assert_eq!(g.enemies[0].x, at);
    assert!(g.player.hp < 1000.0, "a dart hit");
    assert!(!g.events.iter().any(|e| e[0] == EV_FOE), "a disciple calls out nothing");
}

#[test]
fn an_elite_now_and_then_drops_a_manual() {
    let e = foe_kinds().into_iter().find(|&k| !KINDS[k].manual).expect("an elite");
    let mut g = Game::new(3);
    g.running = true;
    let (mut drops, n) = (0, 200);
    for _ in 0..n {
        g.spawn_enemy_at(e as u8, 50.0, 0.0);
        let i = g.enemies.len() - 1;
        g.enemies[i].hp = 0.0;
        g.events.clear();
        g.reap();
        drops += g.events.iter().filter(|v| v[0] == EV_MANUAL).count();
    }
    let share = drops as f32 / n as f32;
    assert!((share - foes::ELITE_MANUAL).abs() < 0.1, "dropped {drops} of {n}");
    // A disciple never does.
    let d = (0..N_KINDS).find(|&k| KINDS[k].school == KINDS[e].school && KINDS[k].strike >= N_MOVES).unwrap();
    for _ in 0..50 {
        g.spawn_enemy_at(d as u8, 50.0, 0.0);
        let i = g.enemies.len() - 1;
        g.enemies[i].hp = 0.0;
        g.events.clear();
        g.reap();
        assert!(!g.events.iter().any(|v| v[0] == EV_MANUAL));
    }
}
