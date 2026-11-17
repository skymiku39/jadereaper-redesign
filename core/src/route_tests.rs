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
    for m in 1..=n.saturating_sub(2) {
        let i = put(&mut g, 60.0, 0.0);
        g.enemies[i].kind = master as u8;
        g.enemies[i].hp = 0.0;
        g.events.clear();
        g.reap();
        assert!(g.events.iter().any(|e| e[0] == EV_MANUAL && e[3] == r as f32));
        assert_eq!(g.upgrade_top(ARTS[a].upgrade) as usize, 2 + (2 + m).min(n));
    }
    // Every move open: another manual adds nothing.
    g.events.clear();
    g.gain_manual(s);
    assert!(!g.events.iter().any(|e| e[0] == EV_MANUAL && e[3] == r as f32));
    assert_eq!(g.upgrade_top(ARTS[a].upgrade) as usize, 2 + n);
    // A later run keeps them through the `manuals` tune.
    let mut later = Game::new(2);
    later.tune[T_MANUALS] = g.manuals as f32;
    later.start(M_ENDLESS);
    assert_eq!(later.moves_open(r), n);
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
    // No more than the most at once.
    for _ in 0..20 {
        g.set_turrets(a, m, 1.0);
    }
    assert_eq!(g.turrets.len(), turrets::MAX_TURRETS);
}

#[test]
fn a_net_slows_the_enemies_inside_until_it_comes_down() {
    let m = (0..N_MOVES).find(|&m| MOVES[m].slow > 0.0).expect("a net move");
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
