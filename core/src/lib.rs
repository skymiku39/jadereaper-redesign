//! Jade Reaper simulation core.
//!
//! Compiled to `wasm32-unknown-unknown` with no dependencies and no bindgen.
//! The host (JavaScript) calls the `jr_*` functions below and reads three
//! buffers straight out of linear memory each frame:
//!
//! - instances: `jr_instance_count()` x 12 floats, uploaded to the GPU as is
//! - state: `STATE_FLOATS` floats for the HUD (layout in `content/game.toml`)
//! - events: `jr_event_count()` x 4 floats `[type, x, y, value]` for sound

pub mod accept;
pub mod defs;
pub mod fmath;
pub mod game;
pub mod generated;
pub mod mission_arena;
pub mod mission_escape;
pub mod mission_formation;
pub mod mission_hold;
pub mod mission_retrieval;
pub mod schema;

use game::Game;

static mut GAME: Option<Game> = None;

fn g() -> &'static mut Game {
    // Single-threaded wasm: there is only ever one caller.
    unsafe { (*core::ptr::addr_of_mut!(GAME)).get_or_insert_with(|| Game::new(1)) }
}

/// Reset to the idle scene shown behind the title screen.
#[no_mangle]
pub extern "C" fn jr_reset(seed: u32) {
    let (w, h) = (g().view_w, g().view_h);
    *g() = Game::new(seed);
    g().set_view(w, h);
}

/// The tunes the next `jr_start` gives its run, indexed like `[[tune]]` in the content file.
static mut TUNE: [f32; defs::N_TUNES] = defs::TUNE_DEFAULT;

fn tune() -> &'static mut [f32; defs::N_TUNES] {
    // Single-threaded wasm, like `g`.
    unsafe { &mut *core::ptr::addr_of_mut!(TUNE) }
}

/// Start a new run of a mission (an id from `[[mission]]` in the content file),
/// with the tunes set by `jr_tune` since the last `jr_tune_reset`. The run
/// opens with the accept phase (see `accept.rs`) unless its time is tuned to 0.
#[no_mangle]
pub extern "C" fn jr_start(seed: u32, mission: u32) {
    jr_reset(seed);
    for (id, value) in tune().iter().enumerate() {
        g().set_tune(id, *value);
    }
    g().brief(mission as usize);
}

/// Change one tune (an id from `[[tune]]`) for the runs started from now on.
/// The value is kept inside the tune's range; an unknown id is ignored.
#[no_mangle]
pub extern "C" fn jr_tune(id: u32, value: f32) {
    if let Some(slot) = tune().get_mut(id as usize) {
        if value.is_finite() {
            *slot = value;
        }
    }
}

/// Put every tune back to its default.
#[no_mangle]
pub extern "C" fn jr_tune_reset() {
    *tune() = defs::TUNE_DEFAULT;
}

/// Visible area in world units (full width and height).
#[no_mangle]
pub extern "C" fn jr_set_view(w: f32, h: f32) {
    g().set_view(w * 0.5, h * 0.5);
}

/// Movement input, each axis in -1..1.
#[no_mangle]
pub extern "C" fn jr_input(mx: f32, my: f32) {
    g().input = (mx, my);
}

/// Advance by `dt` seconds of real time. The simulation moves in whole steps
/// of 1/60 s; see `Game::update`.
#[no_mangle]
pub extern "C" fn jr_update(dt: f32) {
    g().update(dt);
}

/// Steps taken since the last reset.
#[no_mangle]
pub extern "C" fn jr_ticks() -> u32 {
    g().ticks
}

/// Checksum of the simulation state, for comparing two runs.
#[no_mangle]
pub extern "C" fn jr_state_hash() -> u32 {
    g().hash()
}

/// Play a scripted run on a separate game and return its final state hash.
/// The determinism check compares this with the native build.
#[no_mangle]
pub extern "C" fn jr_scripted_run(seed: u32, ticks: u32) -> u32 {
    game::scripted_run(seed, ticks)
}

/// Pick one of the offered upgrades (0..2).
#[no_mangle]
pub extern "C" fn jr_choose(i: u32) {
    g().choose(i as usize);
}

/// Give the player a martial art (an id from `[[art]]`) at a level, carrying
/// an element (an id from `[[element]]`, 0 for none). For the tools and the
/// browser console, to look at a build without playing up to it; the game
/// itself never calls it.
#[no_mangle]
pub extern "C" fn jr_give(art: u32, level: u32, element: u32) {
    g().give(art as usize, level, element);
}

#[no_mangle]
pub extern "C" fn jr_instances_ptr() -> *const f32 {
    g().inst.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_instance_count() -> u32 {
    g().inst.len() as u32
}

#[no_mangle]
pub extern "C" fn jr_state_ptr() -> *const f32 {
    g().state.as_ptr()
}

#[no_mangle]
pub extern "C" fn jr_events_ptr() -> *const f32 {
    g().events.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_event_count() -> u32 {
    g().events.len() as u32
}

/// What this frame's edge-of-screen markers point at: `jr_pointer_count()` x 4
/// floats `[x, y, distance from the player, 0]` in world units.
#[no_mangle]
pub extern "C" fn jr_pointers_ptr() -> *const f32 {
    g().pointers.as_ptr() as *const f32
}

#[no_mangle]
pub extern "C" fn jr_pointer_count() -> u32 {
    g().pointers.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use defs::*;

    #[test]
    fn long_run_does_not_panic() {
        let mut game = Game::new(7);
        game.set_view(640.0, 360.0);
        game.running = true;
        let mut rng = Rng(99);
        let mut picks = 0;
        for frame in 0..(60 * 600) {
            if frame % 30 == 0 {
                game.input = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
            }
            game.update(1.0 / 60.0);
            assert!(game.inst.len() <= MAX_INST);
            if game.choosing {
                game.choose((rng.next_u32() % 3) as usize);
                picks += 1;
            }
            // Keep the run going so late-game spawning is exercised too.
            if game.over {
                game.over = false;
                game.player.hp = game.player.max_hp;
            }
            for i in &game.inst {
                assert!(i.x.is_finite() && i.y.is_finite() && i.hw.is_finite() && i.rot.is_finite());
            }
        }
        assert!(picks > 5, "expected level ups, got {picks}");
        assert!(game.kills > 50, "expected kills, got {}", game.kills);
    }

    #[test]
    fn same_seed_and_inputs_give_the_same_run() {
        let a = game::scripted_run(42, 60 * 120);
        assert_eq!(a, game::scripted_run(42, 60 * 120));
        assert_ne!(a, game::scripted_run(44, 60 * 120));
    }

    /// Drive two games with the same inputs per step, one at 60 frames per
    /// second and one at 30 with a pause in the middle.
    #[test]
    fn the_run_does_not_depend_on_the_frame_rate() {
        let mut fast = Game::new(5);
        let mut slow = Game::new(5);
        fast.running = true;
        slow.running = true;
        let mut bot = Rng(77);
        for frame in 0..(30 * 60) {
            let input = (bot.range(-1.0, 1.0), bot.range(-1.0, 1.0));
            fast.input = input;
            slow.input = input;
            fast.update(game::TICK);
            fast.update(game::TICK);
            slow.update(game::TICK * 2.0);
            if frame == 400 {
                slow.update(0.0);
            }
            assert_eq!(fast.choosing, slow.choosing);
            if fast.choosing {
                fast.choose(0);
                slow.choose(0);
            }
        }
        assert_eq!(fast.ticks, 2 * 30 * 60);
        assert_eq!(fast.ticks, slow.ticks);
        assert_eq!(fast.hash(), slow.hash());
    }

    #[test]
    fn time_shorter_than_a_step_is_carried_over() {
        let mut game = Game::new(1);
        game.running = true;
        for _ in 0..4 {
            game.update(game::TICK / 4.0);
        }
        assert_eq!(game.ticks, 1);
        game.update(0.0);
        assert_eq!(game.ticks, 1);
    }

    /// Play until the run ends or `seconds` pass, keeping the player alive.
    fn play_unhurt(game: &mut Game, seconds: u32) {
        for _ in 0..(60 * seconds) {
            game.player.hp = game.player.max_hp;
            // `tick` does not clear the event list the way `update` does.
            game.events.clear();
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            if game.won || game.over {
                break;
            }
        }
    }

    #[test]
    fn survive_mission_is_won_at_its_goal() {
        let mut game = Game::new(11);
        game.start(M_SURVIVE);
        let goal = game.goal();
        assert!(goal > 0.0);
        play_unhurt(&mut game, goal as u32 - 1);
        assert!(!game.won, "won early at {}", game.time);
        play_unhurt(&mut game, 2);
        assert!(game.won && !game.over);
        assert!(game.time >= goal && game.time < goal + 0.1, "won at {}", game.time);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));

        // The run is frozen once won.
        let (time, ticks) = (game.time, game.ticks);
        game.update(game::TICK);
        assert_eq!(game.time, time);
        assert_eq!(game.ticks, ticks + 1);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_MISSION], M_SURVIVE as f32);
        assert_eq!(game.state[S_GOAL], goal);
        assert_eq!(game.state[S_PROGRESS], goal);
    }

    #[test]
    fn survive_mission_is_lost_on_death() {
        let mut game = Game::new(3);
        game.start(M_SURVIVE);
        for _ in 0..(60 * 300) {
            game.tick();
            if game.choosing {
                game.choose(0);
            }
            if game.over {
                break;
            }
        }
        assert!(game.over && !game.won);
    }

    #[test]
    fn kills_mission_is_won_at_its_goal() {
        let mut game = Game::new(11);
        game.start(M_KILLS);
        let goal = game.goal();
        assert!(goal > 0.0);
        // Leave the goal one kill away, then let the weapons make it.
        game.kills = goal as u32 - 1;
        game.tick();
        assert!(!game.won);
        assert_eq!(game.progress(), goal - 1.0);
        play_unhurt(&mut game, 120);
        assert!(game.won && !game.over, "no kill in two minutes");
        assert!(game.kills as f32 >= goal);
        assert!(game.events.iter().any(|e| e[0] == EV_WIN));

        game.update(game::TICK);
        assert_eq!(game.state[S_WON], 1.0);
        assert_eq!(game.state[S_MISSION], M_KILLS as f32);
        assert_eq!(game.state[S_GOAL], goal);
        // Several enemies can die in the winning step; progress stops at the goal.
        assert_eq!(game.state[S_PROGRESS], goal);
    }

    #[test]
    fn kills_mission_counts_kills_not_time() {
        let mut game = Game::new(5);
        game.start(M_KILLS);
        play_unhurt(&mut game, 30);
        assert!(game.kills > 0 && (game.kills as f32) < game.goal(), "{} kills", game.kills);
        assert!(!game.won && !game.over);
        assert_eq!(game.progress(), game.kills as f32);
    }

    #[test]
    fn endless_mission_is_never_won() {
        let mut game = Game::new(11);
        game.start(M_ENDLESS);
        play_unhurt(&mut game, MISSION_GOAL[M_SURVIVE] as u32 + 30);
        assert!(!game.won);
        assert_eq!(game.state[S_GOAL], 0.0);
    }

    #[test]
    fn unknown_mission_plays_endless() {
        let mut game = Game::new(1);
        game.start(N_MISSIONS + 3);
        assert_eq!(game.mission, M_ENDLESS);
        assert!(game.running);
    }

    #[test]
    fn player_dies_when_idle() {
        let mut game = Game::new(3);
        game.set_view(640.0, 360.0);
        game.running = true;
        for _ in 0..(60 * 900) {
            game.update(1.0 / 60.0);
            if game.choosing {
                game.choose(0);
            }
            if game.over {
                break;
            }
        }
        assert!(game.over);
        assert_eq!(game.state[S_OVER], 1.0);
    }
}
