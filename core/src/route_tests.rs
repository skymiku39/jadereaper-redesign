//! Tests for the routes (套路): taking one, the move order, and the moves
//! landing. Numbers are read from the content tables, so settling them in
//! `content/game.toml` does not break these. Included from `game.rs` under
//! `#[cfg(test)]` so the tests can reach private fields.

use super::routes::BASIC;
use super::*;

/// A running game with every art parked, so only what a test does happens.
fn base() -> Game {
    let mut g = Game::new(1);
    g.running = true;
    for art in g.arts.iter_mut() {
        art.timer = 1000.0;
    }
    g
}

/// Place a tanky enemy (hp 100000) so damage is the hp difference.
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

/// The first art that has a route to take, and its routes.
fn art_with_routes() -> (usize, Vec<usize>) {
    let g = Game::new(1);
    let a = (0..N_ARTS).find(|&a| g.routes_of(a).next().is_some()).expect("an art with routes");
    (a, g.routes_of(a).collect())
}

/// Offer art `a`'s upgrade as the only card and pick it.
fn pick_art(g: &mut Game, a: usize) {
    g.choosing = true;
    g.pending = 1;
    g.choices = [ARTS[a].upgrade as i32, -1, -1];
    g.choose(0);
}

#[test]
fn every_route_belongs_to_an_art_and_its_moves_follow_each_other() {
    for (r, def) in ROUTES.iter().enumerate() {
        assert!(def.art < N_ARTS);
        assert!(def.n >= 1 && def.first + def.n <= N_MOVES);
        for (m, mv) in MOVES.iter().enumerate().skip(def.first).take(def.n) {
            assert_eq!(mv.route, r, "move {m} of route {r}");
        }
        assert!(def.beat > 0.0, "route {r} needs a beat");
    }
}

#[test]
fn an_art_at_level_two_asks_for_its_route_first() {
    let (a, routes) = art_with_routes();
    let mut g = base();
    g.up[ARTS[a].upgrade] = 2;
    pick_art(&mut g, a);
    assert!(g.choosing, "the choice stays open for the route");
    assert_eq!(g.route_pick as usize, a + 1);
    for (slot, &r) in routes.iter().enumerate() {
        assert_eq!(g.choices[slot], r as i32);
    }
    assert_eq!(g.up[ARTS[a].upgrade], 2, "no level until a route is taken");
    let last = routes.len() - 1;
    g.choose(last);
    assert!(!g.choosing);
    assert_eq!(g.route_pick, 0);
    assert_eq!(g.up[ARTS[a].upgrade], 3);
    assert_eq!(g.route_of(a), Some(routes[last]));
    assert_eq!(g.upgrade_top(ARTS[a].upgrade) as usize, 2 + ROUTES[routes[last]].n);
}

#[test]
fn an_empty_route_slot_cannot_be_picked() {
    let (a, routes) = art_with_routes();
    let mut g = base();
    g.up[ARTS[a].upgrade] = 2;
    pick_art(&mut g, a);
    if routes.len() < 3 {
        g.choose(2);
        assert!(g.choosing && g.route_pick > 0, "an empty slot changes nothing");
    }
}

#[test]
fn with_routes_off_an_art_grows_as_before() {
    let (a, _) = art_with_routes();
    let mut g = base();
    g.tune[T_ROUTES] = 0.0;
    g.up[ARTS[a].upgrade] = 2;
    pick_art(&mut g, a);
    assert!(!g.choosing);
    assert_eq!(g.up[ARTS[a].upgrade], 3);
    assert_eq!(g.route_of(a), None);
    assert_eq!(g.upgrade_top(ARTS[a].upgrade), UPGRADE_MAX[ARTS[a].upgrade]);
}

#[test]
fn the_move_order_grows_drops_the_basic_move_and_ends_with_the_ultimate() {
    for (r, def) in ROUTES.iter().enumerate() {
        let a = def.art;
        let mut g = base();
        g.take_route(a, r);
        let ultimate = MOVES[def.first + def.n - 1].ultimate;
        let regular = def.n - ultimate as usize;
        for level in 3..=2 + def.n {
            g.up[ARTS[a].upgrade] = level as u8;
            let (order, len) = g.move_order(a);
            let learned = level - 2;
            if learned < regular {
                assert_eq!(len, learned + 1, "route {r} level {level}");
                assert_eq!(order[0], BASIC, "the basic move opens the order while learning");
            } else {
                assert_eq!(len, learned, "route {r} level {level}");
                assert!(order[..len].iter().all(|&m| m != BASIC), "no basic move once every move is learned");
            }
            assert!(order[1..len].iter().all(|&m| m != BASIC));
        }
        if ultimate {
            g.up[ARTS[a].upgrade] = (2 + def.n) as u8;
            let (order, len) = g.move_order(a);
            assert_eq!(order[len - 1] as usize, def.first + def.n - 1, "the ultimate comes last");
        }
    }
}

#[test]
fn a_route_uses_its_moves_in_turn_and_they_land() {
    for (r, def) in ROUTES.iter().enumerate() {
        let a = def.art;
        let mut g = base();
        g.take_route(a, r);
        g.up[ARTS[a].upgrade] = (2 + def.n) as u8;
        let near = put(&mut g, 60.0, 0.0);
        // A move that lands ahead (a ring with an offset) is checked on an enemy there.
        let ahead =
            ARTS[a].levels[1].reach * MOVES[def.first..def.first + def.n].iter().map(|m| m.offset).fold(0.0, f32::max);
        let far = put(&mut g, ahead.max(60.0), 0.0);
        g.player.aim = (1.0, 0.0);
        let (order, len) = g.move_order(a);
        g.arts[a].timer = 0.0;
        for (step, &m) in order.iter().enumerate().take(len) {
            g.events.clear();
            let before = g.enemies[near].hp + g.enemies[far].hp;
            g.route_beat(a, TICK, 1.0, 1.0);
            // Let waves fly and delayed hits land.
            for _ in 0..120 {
                g.move_shots(TICK);
                g.tick_delayed(TICK);
                g.tick_turrets(TICK);
                g.tick_stretch(TICK);
                for e in g.enemies.iter_mut() {
                    e.bead_cd = (e.bead_cd - TICK).max(0.0);
                }
            }
            if m != BASIC {
                assert!(
                    g.events.iter().any(|e| e[0] == EV_MOVE && e[3] == m as f32),
                    "route {r} step {step} did not report move {m}"
                );
            }
            assert!(
                g.enemies[near].hp + g.enemies[far].hp < before,
                "route {r} step {step} (move {m}) missed the enemies ahead"
            );
            g.enemies[near].x = 60.0;
            g.enemies[near].y = 0.0;
            g.enemies[far].x = ahead.max(60.0);
            g.enemies[far].y = 0.0;
            g.arts[a].timer = 0.0;
        }
        assert_eq!(g.arts[a].step, 0, "the order wraps around");
        assert!(g.delayed.is_empty());
    }
}

#[test]
fn a_stolen_route_is_offered_once_a_disciple_of_its_school_falls() {
    let r = (0..N_ROUTES).find(|&r| !ROUTES[r].start).expect("a stolen route");
    let (a, s) = (ROUTES[r].art, ROUTES[r].school);
    let mut g = base();
    assert!(!g.routes_of(a).any(|x| x == r), "not known at first");
    // A disciple of that school falls in play.
    let k = (0..N_KINDS).find(|&k| KINDS[k].school == s).expect("a disciple of the school");
    let i = put(&mut g, 60.0, 0.0);
    g.enemies[i].kind = k as u8;
    g.enemies[i].hp = 0.0;
    g.reap();
    assert!(g.routes_of(a).any(|x| x == r));
    assert!(g.events.iter().any(|e| e[0] == EV_STEAL && e[3] == r as f32));
    // A second one teaches nothing new.
    g.events.clear();
    g.steal(s);
    assert!(!g.events.iter().any(|e| e[0] == EV_STEAL));
    // A later run knows it from the start through the `stolen` tune.
    let mut later = Game::new(2);
    later.tune[T_STOLEN] = g.stolen as f32;
    later.start(M_ENDLESS);
    assert!(later.routes_of(a).any(|x| x == r));
}

#[test]
fn a_stolen_route_opens_one_more_move_per_manual() {
    // The longest stolen route whose school has a master to drop its manuals.
    let has_master = |s: usize| (0..N_KINDS).any(|k| KINDS[k].school == s && KINDS[k].manual);
    let r = (0..N_ROUTES)
        .filter(|&r| !ROUTES[r].start && has_master(ROUTES[r].school))
        .max_by_key(|&r| ROUTES[r].n)
        .unwrap();
    let (a, s, n) = (ROUTES[r].art, ROUTES[r].school, ROUTES[r].n);
    let mut g = base();
    g.stolen |= 1 << r;
    g.take_route(a, r);
    assert_eq!(g.upgrade_top(ARTS[a].upgrade) as usize, 2 + 2.min(n), "two moves without manuals");
    let master = (0..N_KINDS).find(|&k| KINDS[k].school == s && KINDS[k].manual).expect("a master");
    // A master drops a manual of its school, kept for after the run: nothing opens now.
    let i = put(&mut g, 60.0, 0.0);
    g.enemies[i].kind = master as u8;
    g.enemies[i].hp = 0.0;
    g.events.clear();
    g.reap();
    assert!(g.events.iter().any(|e| e[0] == EV_MANUAL && e[3] == s as f32));
    assert_eq!(g.upgrade_top(ARTS[a].upgrade) as usize, 2 + 2.min(n), "it opens nothing in this run");
    // Given to this route after the run (by the host, through the `manuals`
    // tune), each manual opens one more move.
    let slot = Game::manual_slot(r).unwrap();
    for m in 1..=n.saturating_sub(2) {
        let mut later = base();
        later.tune[T_MANUALS] = (m << (2 * slot)) as f32;
        later.start(M_ENDLESS);
        assert_eq!(later.moves_open(r), (2 + m).min(n));
    }
}

#[test]
fn a_route_limited_to_mechanisms_is_only_for_mechanisms() {
    let r = (0..N_ROUTES).find(|&r| ROUTES[r].only != 0).expect("a limited route");
    let a = ROUTES[r].art;
    let mechanism = (0..N_WEAPONS).find(|&w| ROUTES[r].only & (1 << w) != 0).unwrap();
    let plain = (0..N_WEAPONS).find(|&w| WEAPONS[w].art == a && WEAPONS[w].forms & FORM_MECHANISM == 0).unwrap();
    let mut g = base();
    g.stolen = u32::MAX;
    g.up[ARTS[a].upgrade] = 2;
    g.arts[a].kind = plain as u8 + 1;
    assert!(!g.routes_of(a).any(|x| x == r), "a plain kind does not learn it");
    g.arts[a].kind = mechanism as u8 + 1;
    let open: Vec<usize> = g.routes_of(a).collect();
    assert_eq!(open, vec![r], "a mechanism learns only it");
    // A mechanism is only drawn once the route is known.
    g.stolen = 0;
    g.arts[a].kind = plain as u8 + 1;
    assert!(!g.weapon_offerable(mechanism));
    g.stolen = 1 << r;
    assert!(g.weapon_offerable(mechanism));
}

#[test]
fn mechanisms_shoot_by_themselves_and_end() {
    let r = (0..N_ROUTES).find(|&r| ROUTES[r].only != 0).unwrap();
    let a = ROUTES[r].art;
    let m = ROUTES[r].first + 1;
    let mut g = base();
    let e = put(&mut g, 200.0, 0.0);
    g.set_turrets(a, m, 1.0);
    assert_eq!(g.turrets.len(), MOVES[m].count.max(1));
    for _ in 0..(1.0 / TICK) as u32 {
        g.tick_turrets(TICK);
        g.move_shots(TICK);
    }
    assert!(g.enemies[e].hp < 100_000.0, "the mechanism shot the enemy");
    for _ in 0..((MOVES[m].life + 1.0) / TICK) as u32 {
        g.tick_turrets(TICK);
    }
    assert!(g.turrets.is_empty(), "it ends after its life");
    // No more of its own than it may keep, and no more than the most at once.
    for _ in 0..20 {
        g.set_turrets(a, m, 1.0);
    }
    let own = if MOVES[m].most > 0 { MOVES[m].most as usize } else { turrets::MAX_TURRETS };
    assert_eq!(g.turrets.len(), own.min(turrets::MAX_TURRETS));
    let other = (ROUTES[r].first..ROUTES[r].first + ROUTES[r].n).find(|&o| MOVES[o].most == 0).unwrap();
    for _ in 0..20 {
        g.set_turrets(a, other, 1.0);
    }
    assert_eq!(g.turrets.len(), turrets::MAX_TURRETS);
}

#[test]
fn a_net_slows_the_enemies_inside_until_it_comes_down() {
    let m = (0..N_MOVES).find(|&m| MOVES[m].shape == MS_TURRET && MOVES[m].slow > 0.0).expect("a net move");
    let a = ROUTES[MOVES[m].route].art;
    let mut g = base();
    assert_eq!(Game::net_pace(&g.net, 0.0, 0.0), 1.0, "no net, no slowing");
    g.set_turrets(a, m, 1.0);
    let n = g.net.expect("the ring is a net");
    assert_eq!(Game::net_pace(&g.net, 50.0, 0.0), 1.0 - MOVES[m].slow);
    assert_eq!(Game::net_pace(&g.net, n.r + 10.0, 0.0), 1.0, "outside the net");
    for _ in 0..((MOVES[m].life + 0.5) / TICK) as u32 {
        g.tick_net(TICK);
    }
    assert!(g.net.is_none(), "it comes down after its life");
}

#[test]
fn a_thrust_hits_along_its_line_and_not_beside_it() {
    let a = (0..N_ARTS).find(|&a| ARTS[a].behaviour == B_THRUST).expect("a thrust art");
    let mut g = base();
    g.up[ARTS[a].upgrade] = 1;
    let ahead = put(&mut g, 150.0, 0.0);
    let beside = put(&mut g, 60.0, 120.0);
    g.player.aim = (1.0, 0.0);
    let n = &ARTS[a].levels[0];
    g.line(a, 0.0, 0.0, 1.0, 0.0, n.reach, n.width, 10.0, 0.0);
    assert!(g.enemies[ahead].hp < 100_000.0);
    assert_eq!(g.enemies[beside].hp, 100_000.0);
}

#[test]
fn the_second_batch_of_weapons_only_comes_with_routes() {
    let thrusts: Vec<usize> = (0..N_ARTS).filter(|&a| ARTS[a].routes_only).collect();
    assert!(!thrusts.is_empty());
    let mut g = base();
    g.tune[T_ROUTES] = 0.0;
    for &a in &thrusts {
        assert!(g.upgrade_hidden(ARTS[a].upgrade));
    }
    g.tune[T_ROUTES] = 1.0;
    for &a in &thrusts {
        assert!(!g.upgrade_hidden(ARTS[a].upgrade));
    }
}

#[test]
fn giving_a_high_level_takes_the_first_route() {
    let (a, routes) = art_with_routes();
    let mut g = base();
    g.give(a, 99, 0);
    assert_eq!(g.route_of(a), Some(routes[0]));
    assert_eq!(g.up[ARTS[a].upgrade] as usize, 2 + ROUTES[routes[0]].n);
}

#[test]
fn a_run_on_routes_is_the_same_twice() {
    let run = || {
        let mut g = Game::new(7);
        g.start(M_ENDLESS);
        g.give(0, 99, 0);
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

/// An art on route `r` at its top level, aiming right, nothing else moving.
fn on_route(r: usize) -> (Game, usize) {
    let a = ROUTES[r].art;
    let mut g = base();
    g.take_route(a, r);
    g.up[ARTS[a].upgrade] = (2 + ROUTES[r].n) as u8;
    g.player.aim = (1.0, 0.0);
    (g, a)
}

#[test]
fn a_stretching_staff_reaches_out_and_back() {
    let m = MV_CHAOSHENG;
    let (mut g, a) = on_route(MOVES[m].route);
    let count = g.art_numbers(a).unwrap().count;
    let (x0, _) = g.orbit_at(a, 0);
    // Beyond the staff's own reach, but within twice it, along bead 0.
    let far = put(&mut g, x0 * 1.7, 0.0);
    g.use_move(a, m, 1.0);
    assert!(g.stretching(a));
    let mut most: f32 = 1.0;
    for _ in 0..((MOVES[m].life + 0.1) / TICK) as u32 {
        g.tick_stretch(TICK);
        most = most.max(g.stretch_factor(a));
    }
    assert!((most - MOVES[m].reach).abs() < 1e-3, "it reaches its full length");
    assert!(!g.stretching(a) && g.stretch_factor(a) == 1.0, "and comes back");
    assert!(g.enemies[far].hp < 100_000.0, "hitting what lies along it");
    assert_eq!(g.orbit_count(a), count);
}

#[test]
fn the_staff_tip_lands_a_ring_ahead_at_the_height() {
    let m = MV_JINGTAO;
    let (mut g, a) = on_route(MOVES[m].route);
    let (x0, _) = g.orbit_at(a, 0);
    // Just past the stretched tip, out of the staff's own line.
    let tip = put(&mut g, x0 * MOVES[m].reach + 40.0, 0.0);
    g.use_move(a, m, 1.0);
    for _ in 0..((MOVES[m].life + 0.1) / TICK) as u32 {
        g.tick_stretch(TICK);
    }
    assert!(g.enemies[tip].hp < 100_000.0, "the ring at the tip hit it");
}

#[test]
fn against_the_water_pushes_sideways_and_slows() {
    let m = MV_NISHUI;
    let (mut g, a) = on_route(MOVES[m].route);
    let (x0, _) = g.orbit_at(a, 0);
    let e = put(&mut g, x0 * 0.8, 0.0);
    g.use_move(a, m, 1.0);
    g.tick_stretch(TICK);
    let en = g.enemies[e];
    assert!(en.hp < 100_000.0);
    assert!(en.ky.abs() > en.kx.abs(), "pushed along the turn, not outward");
    assert!(en.tags.slowed > 0.0 && (en.tags.slow - MOVES[m].slow).abs() < 1e-6);
    for _ in 0..(1.1 / TICK) as u32 {
        g.tick_tags(TICK);
    }
    assert_eq!(g.enemies[e].tags.slowed, 0.0, "the slow wears off");
}

#[test]
fn the_great_river_spins_the_staff_faster() {
    let m = MV_DAJIANG;
    let (mut g, a) = on_route(MOVES[m].route);
    assert_eq!(g.staff_turn, 0.0);
    g.use_move(a, m, 1.0);
    let steps = (MOVES[m].life / TICK) as u32;
    for _ in 0..steps {
        g.tick_stretch(TICK);
    }
    let want = (MOVES[m].speed - 1.0) * ARTS[a].speed * steps as f32 * TICK;
    assert!((g.staff_turn - want).abs() < 1e-3, "turned {} beyond its clock, not {want}", g.staff_turn);
}

#[test]
fn a_shard_flies_from_the_enemy_hit_to_the_next() {
    let m = MV_TASHAN;
    let (mut g, a) = on_route(MOVES[m].route);
    let hit = put(&mut g, 40.0, 0.0);
    let next = put(&mut g, 40.0, 200.0);
    g.use_move(a, m, 1.0);
    let shard = g.shots.iter().find(|s| s.last == g.enemies[hit].id).expect("a shard from the enemy hit");
    assert!((shard.x - 40.0).abs() < 1e-3 && shard.y.abs() < 1e-3, "it starts on the enemy");
    assert!(shard.vy > 0.0 && shard.vx.abs() < 1e-3, "toward the other one");
    assert_eq!(g.enemies[next].hp, 100_000.0);
    for _ in 0..60 {
        g.move_shots(TICK);
    }
    assert!(g.enemies[next].hp < 100_000.0, "and hits it");
}

#[test]
fn a_lock_makes_every_blow_land_harder_for_a_second() {
    let m = MV_YUQIN;
    let (mut g, a) = on_route(MOVES[m].route);
    let e = put(&mut g, 40.0, 0.0);
    g.use_move(a, m, 1.0);
    let tags = g.enemies[e].tags;
    assert!(tags.locked > 0.0 && (tags.lock - MOVES[m].lock).abs() < 1e-6);
    assert!((1..=N_ELEMENTS).all(|k| tags.left[k] == 0.0), "not an element tag");
    let before = g.enemies[e].hp;
    g.strike(e, 10.0, 0.0, 0.0, 0);
    assert!((before - g.enemies[e].hp - 10.0 * (1.0 + MOVES[m].lock)).abs() < 1e-3);
    for _ in 0..(1.1 / TICK) as u32 {
        g.tick_tags(TICK);
    }
    let before = g.enemies[e].hp;
    g.strike(e, 10.0, 0.0, 0.0, 0);
    assert!((before - g.enemies[e].hp - 10.0).abs() < 1e-3, "the lock wears off");
}

#[test]
fn homing_needles_turn_in_the_air() {
    let m = MV_YANWANG;
    let (mut g, a) = on_route(MOVES[m].route);
    let first = put(&mut g, 100.0, 0.0);
    g.use_move(a, m, 1.0);
    assert!(g.shots.iter().all(|s| s.home > 0.0));
    // The enemy aimed at goes; another stands off to the side.
    g.enemies[first].x = 5000.0;
    let side = put(&mut g, 150.0, 150.0);
    for _ in 0..60 {
        g.move_shots(TICK);
    }
    assert!(g.enemies[side].hp < 100_000.0, "a needle found the other enemy");
}

#[test]
fn a_bloody_thrust_bursts_where_its_enemy_falls() {
    let m = MV_XUEJI;
    let (mut g, a) = on_route(MOVES[m].route);
    let weak = put(&mut g, 60.0, 0.0);
    g.enemies[weak].hp = 1.0;
    let beside = put(&mut g, 60.0, 40.0);
    g.use_move(a, m, 1.0);
    assert_eq!(g.blooms.len(), 1, "the enemy it felled bursts");
    let before = g.enemies[beside].hp;
    g.tick_blooms();
    assert!(g.enemies[beside].hp < before, "hitting the one beside it");
    assert!(g.blooms.is_empty());
}

#[test]
fn many_kills_thrust_twice_only_at_five_stacks() {
    let m = MV_SHAREN;
    let (mut g, a) = on_route(MOVES[m].route);
    put(&mut g, 60.0, 0.0);
    g.use_move(a, m, 1.0);
    assert!(g.delayed.is_empty(), "once below five stacks");
    g.momentum = MOVES[m].rage as u8;
    g.use_move(a, m, 1.0);
    assert_eq!(g.delayed.len(), MOVES[m].count - 1, "twice at five");
}

#[test]
fn falling_stars_is_one_dart_that_splits_in_three() {
    let m = MV_XINGLUO;
    let (mut g, a) = on_route(MOVES[m].route);
    put(&mut g, 2000.0, 0.0);
    g.use_move(a, m, 1.0);
    assert_eq!(g.shots.len(), 1);
    for _ in 0..((MOVES[m].life + 0.05) / TICK) as u32 {
        g.move_shots(TICK);
    }
    assert_eq!(g.shots.len(), MOVES[m].split as usize);
}

#[test]
fn the_long_river_stretches_the_staff_fourfold_all_round() {
    let m = MV_CHANGHE;
    assert!(MOVES[m].ultimate);
    let (mut g, a) = on_route(MOVES[m].route);
    let (x0, _) = g.orbit_at(a, 0);
    // Far out on the other side, where only a staff four times as long reaches.
    let far = put(&mut g, -x0 * 3.5, 0.0);
    g.use_move(a, m, 1.0);
    let mut most: f32 = 1.0;
    for _ in 0..((MOVES[m].life + 0.1) / TICK) as u32 {
        g.time += TICK;
        g.tick_stretch(TICK);
        most = most.max(g.stretch_factor(a));
    }
    assert!((most - 4.0).abs() < 1e-3, "four times as long");
    assert!(g.enemies[far].hp < 100_000.0, "and round to the far side");
}

#[test]
fn falling_leaves_slows_the_enemies_by_half() {
    let m = MV_LUOYE;
    let (mut g, a) = on_route(MOVES[m].route);
    let e = put(&mut g, 60.0, 0.0);
    g.use_move(a, m, 1.0);
    let tags = g.enemies[e].tags;
    assert!(tags.slowed > 0.0 && (tags.slow - 0.5).abs() < 1e-6);
}

#[test]
fn the_ten_courts_throw_forty_homing_needles() {
    let m = MV_SHIDIAN;
    let (mut g, a) = on_route(MOVES[m].route);
    put(&mut g, 300.0, 0.0);
    g.use_move(a, m, 1.0);
    assert_eq!(g.shots.len(), 40);
    assert!(g.shots.iter().all(|s| s.home > 0.0));
}
