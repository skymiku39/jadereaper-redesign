//! The shapes of the content tables in `crate::generated`. The tables come
//! from `content/game.toml`; what each number means is written there.

/// What a martial art does. One function in `arts.rs` per behaviour.
pub const B_ARC: u8 = 0;
pub const B_PROJECTILE: u8 = 1;
pub const B_ORBIT: u8 = 2;
pub const B_NOVA: u8 = 3;

/// The numbers of a martial art at one level.
pub struct ArtLevel {
    pub damage: f32,
    pub cooldown: f32,
    pub count: usize,
    pub reach: f32,
    pub width: f32,
    pub pierce: i32,
    pub knockback: f32,
}

/// A martial art (武學). Its level is the level of `upgrade`.
pub struct ArtDef {
    pub upgrade: usize,
    pub behaviour: u8,
    pub part: f32,
    pub start_level: u8,
    pub first_delay: f32,
    pub speed: f32,
    pub life: f32,
    pub range: f32,
    /// One entry per level, level 1 first.
    pub levels: &'static [ArtLevel],
}

/// What an element does on a hit. One piece of code in `elements.rs` per effect.
pub const EF_NONE: u8 = 0;
pub const EF_GUARD: u8 = 1;
pub const EF_REGROW: u8 = 2;
pub const EF_SURGE: u8 = 3;
pub const EF_BURN: u8 = 4;
pub const EF_ROOT: u8 = 5;
pub const EF_CHAIN: u8 = 6;
pub const EF_BANE: u8 = 7;

/// A magic element (魔法) a martial art can carry.
pub struct ElementDef {
    pub effect: u8,
    /// Whether a hit leaves this element's tag on the enemy.
    pub tags: bool,
    pub color: [f32; 3],
    pub power: f32,
    pub minor: f32,
    pub time: f32,
    pub cap: f32,
    pub rate: f32,
    pub reach: f32,
    pub rest: f32,
    pub count: usize,
}

impl ElementDef {
    /// The entry for "no element".
    pub const NONE: ElementDef = ElementDef {
        effect: EF_NONE,
        tags: false,
        color: [0.0; 3],
        power: 0.0,
        minor: 0.0,
        time: 0.0,
        cap: 0.0,
        rate: 0.0,
        reach: 0.0,
        rest: 0.0,
        count: 0,
    };
}

/// How a reaction plays out. One piece of code in `elements.rs` per kind.
pub const RK_BURST: u8 = 0;
pub const RK_RELAY: u8 = 1;

/// What happens when an enemy holds the tags of elements `a` and `b` at once.
pub struct ReactionDef {
    pub a: u8,
    pub b: u8,
    pub kind: u8,
    pub power: f32,
    pub reach: f32,
    pub count: usize,
    pub color: [f32; 3],
}

/// An enemy kind. Kind 0 fills whatever share of the horde the others leave.
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
    /// Hard to push, and it bursts into more pieces.
    pub heavy: bool,
    /// Drawn floating above the ground instead of standing on it.
    pub hover: bool,
    /// The horde sends it once the run is this many seconds old.
    pub from: f32,
    /// Its share of the horde's spawns.
    pub share: f32,
    /// Above 0, the share grows from nothing to `share` over this many seconds after `from`.
    pub ramp: f32,
    /// Above 0, one comes every this many seconds, announced, instead of with the horde.
    pub every: f32,
    /// The tune that holds its share instead of `share`; `N_TUNES` for none.
    pub share_tune: usize,
}

/// The stats a passive upgrade can raise. `Game::stat` says how each one adds up.
pub const ST_MAX_HP: u8 = 0;
pub const ST_MOVE_SPEED: u8 = 1;
pub const ST_PICKUP_RANGE: u8 = 2;
pub const ST_DAMAGE: u8 = 3;
pub const ST_COOLDOWN: u8 = 4;
pub const ST_REGEN: u8 = 5;

/// A passive upgrade: `stat` changes by `amount` per level of `upgrade`.
pub struct PassiveDef {
    pub upgrade: usize,
    pub stat: u8,
    pub amount: f32,
}
