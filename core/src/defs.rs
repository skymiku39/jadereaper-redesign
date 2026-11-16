//! Shared definitions: the instance format sent to the renderer, part ids,
//! tuning tables and a tiny RNG.
//!
//! Part ids, upgrade ids and max levels, event ids, state indices and the ink
//! colour come from `content/game.toml` through `crate::generated` and are
//! re-exported here. Edit the content file, not the generated code.

pub use crate::generated::*;
pub use crate::schema::*;

/// One paper cutout drawn by the renderer. 12 floats, 48 bytes.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Inst {
    pub x: f32,
    pub y: f32,
    /// Half width. A negative value mirrors the part horizontally.
    pub hw: f32,
    pub hh: f32,
    pub rot: f32,
    pub part: f32,
    /// 0..1, flashes the part white when hit.
    pub flash: f32,
    /// Per-instance random value, used for outline wobble and shape variety.
    pub seed: f32,
    pub color: [f32; 4],
}

pub const INST_FLOATS: usize = 12;
pub const MAX_INST: usize = 4096;

/// Enemy archetype.
pub struct Kind {
    pub hp: f32,
    pub speed: f32,
    /// Collision radius.
    pub r: f32,
    pub dmg: f32,
    pub xp: f32,
    pub part: f32,
    pub color: [f32; 3],
    pub hw: f32,
    pub hh: f32,
    /// Puppet pose changes per second.
    pub rate: f32,
    /// A corpse or a spirit: what an element with the bane effect hits hardest.
    pub corpse: bool,
}

pub const K_BLOT: u8 = 0;
pub const K_WISP: u8 = 1;
pub const K_BRUTE: u8 = 2;
pub const K_ELITE: u8 = 3;

pub const KINDS: [Kind; 4] = [
    Kind {
        hp: 14.0,
        speed: 62.0,
        r: 16.0,
        dmg: 5.0,
        xp: 1.0,
        part: P_BLOT,
        color: [0.33, 0.31, 0.30],
        hw: 18.0,
        hh: 18.0,
        rate: 5.0,
        corpse: false,
    },
    Kind {
        hp: 7.0,
        speed: 118.0,
        r: 12.0,
        dmg: 4.0,
        xp: 1.0,
        part: P_WISP,
        color: [0.36, 0.40, 0.56],
        hw: 22.0,
        hh: 13.0,
        rate: 9.0,
        corpse: true,
    },
    Kind {
        hp: 70.0,
        speed: 44.0,
        r: 26.0,
        dmg: 12.0,
        xp: 5.0,
        part: P_BRUTE,
        color: [0.58, 0.31, 0.24],
        hw: 30.0,
        hh: 30.0,
        rate: 3.0,
        corpse: false,
    },
    Kind {
        hp: 600.0,
        speed: 56.0,
        r: 44.0,
        dmg: 20.0,
        xp: 40.0,
        part: P_BRUTE,
        color: [0.74, 0.25, 0.18],
        hw: 52.0,
        hh: 52.0,
        rate: 2.5,
        corpse: false,
    },
];

pub struct Rng(pub u32);

impl Rng {
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// Uniform in 0..1.
    pub fn f(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.f()
    }
}

pub fn hash2(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}
