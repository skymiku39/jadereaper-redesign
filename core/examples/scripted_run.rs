//! Prints the state hash of scripted runs: every mission, without and with
//! every martial art and an element on each, for each seed. One line per run,
//! `<mission id> <armed 0 or 1> <seed> <hash>`:
//!
//!     cargo run --release --manifest-path core/Cargo.toml --example scripted_run -- <ticks> <seed>...
//!
//! `tools/determinism.mjs` compares the output with the wasm build.

use jadereaper_core::defs::N_MISSIONS;
use jadereaper_core::game::scripted_mission;

fn main() {
    let mut args = std::env::args().skip(1);
    let ticks: u32 = args.next().and_then(|a| a.parse().ok()).expect("usage: scripted_run <ticks> <seed>...");
    let seeds: Vec<u32> = args.map(|a| a.parse().expect("seed must be a number")).collect();
    for mission in 0..N_MISSIONS {
        for armed in [false, true] {
            for &seed in &seeds {
                println!("{mission} {} {seed} {}", armed as u8, scripted_mission(seed, ticks, mission, armed));
            }
        }
    }
}
