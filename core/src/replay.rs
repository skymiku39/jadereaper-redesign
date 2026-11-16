//! Recording what the player does in a run, and playing it back.
//!
//! A run depends only on its seed, its tunes, its mission and the player's
//! inputs, and on the size of the visible area (enemies spawn just outside
//! it), so those are all a replay needs. The core logs them as they happen: a
//! move whenever the movement input changes, the visible area whenever it
//! changes, and every pick of a card. Each entry is `[step, kind, a, b]`; a
//! move holds the bits of the two axes, a view the bits of the half width and
//! half height, a pick the slot or art id. To play a run back, the host starts the
//! same mission with the same seed and tunes, then hands the log over; from
//! then on the core takes its inputs and picks from the log and ignores the
//! host's.

use super::Game;

/// A move: `a` and `b` are the bits of the two axes of the movement input.
pub const MOVE: u32 = 0;
/// A pick of a card: `a` is what `Game::choose` was given.
pub const PICK: u32 = 1;

/// A change of the visible area: `a` and `b` are the bits of its half width and half height.
pub const VIEW: u32 = 2;

/// The longest log kept. A run that goes past it can no longer be replayed exactly.
pub const MAX_LOG: usize = 250_000;

#[derive(Default)]
pub struct Replay {
    /// What the player did this run, in order.
    pub log: Vec<[u32; 4]>,
    /// The movement input as last logged, so only changes are kept.
    last: (u32, u32),
    /// True while playing a log back instead of taking the host's inputs.
    pub playing: bool,
    /// The log being played back, and how far along it is.
    pub tape: Vec<[u32; 4]>,
    cursor: usize,
    /// The movement input the tape holds now. It is put back before every
    /// step, so nothing the host does can change a replay.
    held: (f32, f32),
    /// The visible area as last logged.
    last_view: (u32, u32),
}

impl Replay {
    /// Play a log back from the start of the run. The entries come with `push`.
    pub fn begin(&mut self) {
        self.playing = true;
        self.tape.clear();
        self.cursor = 0;
        self.held = (0.0, 0.0);
    }

    pub fn push(&mut self, entry: [u32; 4]) {
        if self.tape.len() < MAX_LOG {
            self.tape.push(entry);
        }
    }

    fn note(&mut self, entry: [u32; 4]) {
        if self.log.len() < MAX_LOG {
            self.log.push(entry);
        }
    }
}

impl Game {
    /// At the start of a step: take the movement input from the tape when
    /// playing back, and log it when it changed.
    pub(crate) fn replay_before_step(&mut self) {
        if !self.running || self.over || self.won {
            return;
        }
        let now = self.ticks;
        if self.replay.playing {
            while let Some(&e) = self.replay.tape.get(self.replay.cursor) {
                if e[0] > now || (e[1] == PICK && e[0] == now) {
                    break;
                }
                // A pick from an earlier step could not be made; it is skipped.
                if e[1] == MOVE {
                    self.replay.held = (f32::from_bits(e[2]), f32::from_bits(e[3]));
                } else if e[1] == VIEW {
                    // Not through `set_view`, which ignores the host while a replay plays.
                    self.view_w = f32::from_bits(e[2]);
                    self.view_h = f32::from_bits(e[3]);
                }
                self.replay.cursor += 1;
            }
            self.input = self.replay.held;
        }
        let bits = (self.input.0.to_bits(), self.input.1.to_bits());
        if bits != self.replay.last {
            self.replay.last = bits;
            self.replay.note([now, MOVE, bits.0, bits.1]);
        }
        let view = (self.view_w.to_bits(), self.view_h.to_bits());
        if view != self.replay.last_view {
            self.replay.last_view = view;
            self.replay.note([now, VIEW, view.0, view.1]);
        }
    }

    /// At the end of a step: when playing back, make the picks the tape holds for this step.
    pub(crate) fn replay_after_step(&mut self) {
        if !self.replay.playing {
            return;
        }
        let now = self.ticks;
        while let Some(&e) = self.replay.tape.get(self.replay.cursor) {
            if e[0] != now || e[1] != PICK {
                break;
            }
            self.replay.cursor += 1;
            if self.choosing {
                self.choose(e[2] as usize);
            }
        }
    }

    /// Log a pick of a card. Called by `choose` while a choice is open.
    pub(crate) fn replay_pick(&mut self, i: usize) {
        let now = self.ticks;
        self.replay.note([now, PICK, i as u32, 0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::*;

    /// Play a run with a bot that moves at random and picks at random, the
    /// way the host would, and return its hash, its step count and its log.
    fn record(seed: u32, mission: usize, steps: u32) -> (u32, u32, Vec<[u32; 4]>) {
        let mut g = Game::new(seed);
        g.tune[T_ELEMENT_CARDS] = 0.5;
        g.start(mission);
        let mut bot = Rng(seed.wrapping_mul(3) | 1);
        for t in 0..steps {
            if t % 20 == 0 {
                g.input = (bot.range(-1.0, 1.0), bot.range(-1.0, 1.0));
            }
            if t == 600 {
                g.set_view(500.0, 420.0);
            }
            g.tick();
            let mut tries = 0;
            while g.choosing && tries < 8 {
                let pick = if g.attach > 0 { bot.next_u32() as usize % N_ARTS } else { bot.next_u32() as usize % 3 };
                g.choose(pick);
                tries += 1;
            }
            if g.over || g.won {
                break;
            }
        }
        (g.hash(), g.ticks, g.replay.log.clone())
    }

    /// Play a log back on a fresh game of the same seed and mission, for as many steps.
    fn play(seed: u32, mission: usize, steps: u32, tape: &[[u32; 4]]) -> Game {
        let mut g = Game::new(seed);
        g.tune[T_ELEMENT_CARDS] = 0.5;
        g.start(mission);
        g.replay.begin();
        for &e in tape {
            g.replay.push(e);
        }
        while g.ticks < steps {
            // The host's input and window are ignored while playing back.
            g.input = (1.0, 1.0);
            g.set_view(900.0, 200.0);
            g.tick();
        }
        g
    }

    #[test]
    fn a_replay_ends_where_the_run_ended() {
        for (seed, mission) in [(3, M_ENDLESS), (8, M_SURVIVE), (21, M_ARENA), (34, M_ESCORT), (5, M_BOSS)] {
            let (hash, steps, log) = record(seed, mission, 60 * 90);
            assert!(log.iter().any(|e| e[1] == PICK), "seed {seed}: no card was picked, so picks are untested");
            let g = play(seed, mission, steps, &log);
            assert_eq!(g.hash(), hash, "mission {mission}, seed {seed}");
            // Playing back logs the same thing again, so a replay of a replay is the same run.
            assert_eq!(g.replay.log, log);
        }
    }

    #[test]
    fn a_changed_log_plays_a_different_run() {
        let (hash, steps, mut log) = record(3, M_ENDLESS, 60 * 60);
        let first_move = log.iter().position(|e| e[1] == MOVE).unwrap();
        log[first_move][2] = (-1.0f32).to_bits();
        log[first_move][3] = 0.0f32.to_bits();
        assert_ne!(play(3, M_ENDLESS, steps, &log).hash(), hash);
    }

    #[test]
    fn only_changes_of_the_input_are_logged() {
        let mut g = Game::new(1);
        g.start(M_ENDLESS);
        g.input = (1.0, 0.0);
        for _ in 0..100 {
            g.player.hp = g.player.max_hp;
            g.tick();
        }
        g.input = (0.0, 1.0);
        g.tick();
        let moves: Vec<_> = g.replay.log.iter().filter(|e| e[1] == MOVE).collect();
        assert_eq!(moves.len(), 2);
        assert_eq!(moves[1][0], 101);
    }

    #[test]
    fn nothing_is_logged_before_the_run_starts() {
        let mut g = Game::new(1);
        g.input = (1.0, 0.0);
        for _ in 0..30 {
            g.tick();
        }
        assert!(g.replay.log.is_empty());
    }
}
