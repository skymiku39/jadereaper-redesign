//! Shared definitions: the instance format sent to the renderer, the ids and
//! content tables, and a tiny RNG.
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
