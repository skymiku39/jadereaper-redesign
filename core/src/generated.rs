//! Generated from `content/game.toml` by `tools/gen`. Do not edit: change the
//! content file and run `./build.sh`. Re-exported by `crate::defs`.

use crate::schema::*;

// Part ids.
pub const P_ELLIPSE: f32 = 0.0;
pub const P_PLAYER_BODY: f32 = 1.0;
pub const P_PLAYER_HEAD: f32 = 2.0;
pub const P_SCYTHE: f32 = 3.0;
pub const P_BLOT: f32 = 4.0;
pub const P_WISP: f32 = 5.0;
pub const P_BRUTE: f32 = 6.0;
pub const P_GEM: f32 = 7.0;
pub const P_SLASH: f32 = 8.0;
pub const P_TALISMAN: f32 = 9.0;
pub const P_BEAD: f32 = 10.0;
pub const P_SPLAT: f32 = 11.0;
pub const P_GRASS: f32 = 12.0;
pub const P_STONE: f32 = 13.0;
pub const P_HEART: f32 = 14.0;
pub const P_RING: f32 = 15.0;
pub const P_SCRAP: f32 = 16.0;
pub const P_GROUND_SHADOW: f32 = 18.0;

// Upgrade ids.
pub const U_SCYTHE: usize = 0;
pub const U_TALISMAN: usize = 1;
pub const U_BEADS: usize = 2;
pub const U_PULSE: usize = 3;
pub const U_VITALITY: usize = 4;
pub const U_SWIFT: usize = 5;
pub const U_LURE: usize = 6;
pub const U_MIGHT: usize = 7;
pub const U_HASTE: usize = 8;
pub const U_MENDING: usize = 9;
pub const N_UPGRADES: usize = 10;
#[rustfmt::skip]
pub const UPGRADE_NAMES: [&str; N_UPGRADES] = ["scythe", "talisman", "beads", "pulse", "vitality", "swift", "lure", "might", "haste", "mending"];
pub const UPGRADE_MAX: [u8; N_UPGRADES] = [6, 6, 6, 6, 5, 5, 5, 5, 5, 5];

// Martial arts: one behaviour each, with its numbers per level of the upgrade that levels it.
pub const A_SCYTHE: usize = 0;
pub const A_TALISMAN: usize = 1;
pub const A_BEADS: usize = 2;
pub const A_PULSE: usize = 3;
pub const N_ARTS: usize = 4;
#[rustfmt::skip]
pub const ART_NAMES: [&str; N_ARTS] = ["scythe", "talisman", "beads", "pulse"];
/// How many martial arts the player can hold at once.
pub const MAX_ARTS: usize = 6;
#[rustfmt::skip]
pub const ARTS: [ArtDef; N_ARTS] = [
    ArtDef { upgrade: U_SCYTHE, behaviour: B_ARC, part: P_SLASH, start_level: 1, first_delay: 0.6, speed: 0.0, life: 0.0, range: 0.0, levels: &[
        ArtLevel { damage: 20.0, cooldown: 0.9, count: 1, reach: 119.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 26.0, cooldown: 0.9, count: 1, reach: 133.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 32.0, cooldown: 0.9, count: 1, reach: 147.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 38.0, cooldown: 0.9, count: 1, reach: 161.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 44.0, cooldown: 0.9, count: 2, reach: 175.0, width: -2.0, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 50.0, cooldown: 0.9, count: 2, reach: 189.0, width: -2.0, pierce: 0, knockback: 240.0 },
    ] },
    ArtDef { upgrade: U_TALISMAN, behaviour: B_PROJECTILE, part: P_TALISMAN, start_level: 0, first_delay: 0.5, speed: 540.0, life: 1.3, range: 520.0, levels: &[
        ArtLevel { damage: 12.0, cooldown: 0.95, count: 1, reach: 0.0, width: 0.0, pierce: 1, knockback: 0.3 },
        ArtLevel { damage: 16.0, cooldown: 0.95, count: 2, reach: 0.0, width: 0.0, pierce: 1, knockback: 0.3 },
        ArtLevel { damage: 20.0, cooldown: 0.95, count: 2, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 24.0, cooldown: 0.95, count: 3, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 28.0, cooldown: 0.95, count: 3, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 32.0, cooldown: 0.95, count: 4, reach: 0.0, width: 0.0, pierce: 3, knockback: 0.3 },
    ] },
    ArtDef { upgrade: U_BEADS, behaviour: B_ORBIT, part: P_BEAD, start_level: 0, first_delay: 0.0, speed: 2.8, life: 0.0, range: 0.0, levels: &[
        ArtLevel { damage: 9.0, cooldown: 0.45, count: 2, reach: 84.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 12.0, cooldown: 0.45, count: 3, reach: 90.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 15.0, cooldown: 0.45, count: 4, reach: 96.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 18.0, cooldown: 0.45, count: 5, reach: 102.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 21.0, cooldown: 0.45, count: 6, reach: 108.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 24.0, cooldown: 0.45, count: 7, reach: 114.0, width: 0.0, pierce: 0, knockback: 160.0 },
    ] },
    ArtDef { upgrade: U_PULSE, behaviour: B_NOVA, part: P_RING, start_level: 0, first_delay: 2.0, speed: 0.0, life: 0.0, range: 0.0, levels: &[
        ArtLevel { damage: 16.0, cooldown: 4.2, count: 0, reach: 180.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 22.0, cooldown: 4.2, count: 0, reach: 210.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 28.0, cooldown: 4.2, count: 0, reach: 240.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 34.0, cooldown: 4.2, count: 0, reach: 270.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 40.0, cooldown: 4.2, count: 0, reach: 300.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 46.0, cooldown: 4.2, count: 0, reach: 330.0, width: 0.0, pierce: 0, knockback: 520.0 },
    ] },
];

// Magic elements. Id 0 means no element; `ELEMENTS[0]` is that empty entry.
pub const E_METAL: u8 = 1;
pub const E_WOOD: u8 = 2;
pub const E_WATER: u8 = 3;
pub const E_FIRE: u8 = 4;
pub const E_EARTH: u8 = 5;
pub const E_LIGHTNING: u8 = 6;
pub const E_HOLY: u8 = 7;
pub const N_ELEMENTS: usize = 7;
#[rustfmt::skip]
pub const ELEMENT_NAMES: [&str; N_ELEMENTS + 1] = ["none", "metal", "wood", "water", "fire", "earth", "lightning", "holy"];
#[rustfmt::skip]
pub const ELEMENTS: [ElementDef; N_ELEMENTS + 1] = [
    ElementDef::NONE,
    ElementDef { effect: EF_GUARD, tags: false, color: [0.80, 0.70, 0.36], power: 1.0, minor: 0.4, time: 2.0, cap: 12.0, rate: 6.0, reach: 0.0, rest: 0.0, count: 0 },
    ElementDef { effect: EF_REGROW, tags: false, color: [0.36, 0.62, 0.30], power: 1.5, minor: 0.0, time: 3.0, cap: 0.0, rate: 0.0, reach: 0.0, rest: 0.0, count: 0 },
    ElementDef { effect: EF_SURGE, tags: true, color: [0.25, 0.48, 0.70], power: 1.0, minor: 0.0, time: 0.8, cap: 0.0, rate: 0.0, reach: 0.0, rest: 0.0, count: 0 },
    ElementDef { effect: EF_BURN, tags: true, color: [0.82, 0.34, 0.18], power: 0.8, minor: 0.0, time: 3.0, cap: 0.0, rate: 0.5, reach: 0.0, rest: 0.0, count: 0 },
    ElementDef { effect: EF_ROOT, tags: true, color: [0.55, 0.42, 0.26], power: 260.0, minor: 0.0, time: 0.6, cap: 0.0, rate: 0.0, reach: 0.0, rest: 2.0, count: 0 },
    ElementDef { effect: EF_CHAIN, tags: true, color: [0.56, 0.40, 0.78], power: 0.3, minor: 0.0, time: 2.0, cap: 0.0, rate: 0.0, reach: 150.0, rest: 0.0, count: 3 },
    ElementDef { effect: EF_BANE, tags: true, color: [0.92, 0.86, 0.62], power: 0.6, minor: 0.3, time: 2.0, cap: 0.0, rate: 0.0, reach: 0.0, rest: 0.0, count: 0 },
];

// Reactions between two element tags on one enemy, tried in this order.
pub const R_BLAST: usize = 0;
pub const R_CONDUCT: usize = 1;
pub const R_STEAM: usize = 2;
pub const N_REACTIONS: usize = 3;
#[rustfmt::skip]
pub const REACTIONS: [ReactionDef; N_REACTIONS] = [
    ReactionDef { a: E_FIRE, b: E_LIGHTNING, kind: RK_BURST, power: 1.5, reach: 110.0, count: 0, color: [0.90, 0.50, 0.16] },
    ReactionDef { a: E_WATER, b: E_LIGHTNING, kind: RK_RELAY, power: 1.5, reach: 150.0, count: 3, color: [0.42, 0.52, 0.86] },
    ReactionDef { a: E_WATER, b: E_FIRE, kind: RK_BURST, power: 0.9, reach: 160.0, count: 0, color: [0.80, 0.80, 0.78] },
];

// Event types, read by the host for sound.
pub const EV_HIT: f32 = 1.0;
pub const EV_KILL: f32 = 2.0;
pub const EV_PICKUP: f32 = 3.0;
pub const EV_LEVELUP: f32 = 4.0;
pub const EV_HURT: f32 = 5.0;
pub const EV_SLASH: f32 = 6.0;
pub const EV_SHOOT: f32 = 7.0;
pub const EV_PULSE: f32 = 8.0;
pub const EV_DEATH: f32 = 9.0;
pub const EV_HEAL: f32 = 10.0;
pub const EV_ELITE: f32 = 11.0;
pub const EV_WIN: f32 = 12.0;
pub const EV_REACT: f32 = 13.0;
pub const EV_ATTACH: f32 = 14.0;

// Mission ids, and what each one asks for (seconds for `survive`, enemies for `kills`).
pub const M_ENDLESS: usize = 0;
pub const M_SURVIVE: usize = 1;
pub const M_KILLS: usize = 2;
pub const M_FORMATION: usize = 3;
pub const M_ARENA: usize = 4;
pub const M_RETRIEVAL: usize = 5;
pub const M_PURSUIT: usize = 6;
pub const M_ESCORT: usize = 7;
pub const M_HOLD: usize = 8;
pub const M_ESCAPE: usize = 9;
pub const M_BOSS: usize = 10;
pub const M_VENDETTA: usize = 11;
pub const N_MISSIONS: usize = 12;
#[rustfmt::skip]
pub const MISSION_NAMES: [&str; N_MISSIONS] = ["endless", "survive", "kills", "formation", "arena", "retrieval", "pursuit", "escort", "hold", "escape", "boss", "vendetta"];
#[rustfmt::skip]
pub const MISSION_GOAL: [f32; N_MISSIONS] = [0.0, 300.0, 1500.0, 5.0, 10.0, 3.0, 3000.0, 21000.0, 300.0, 18000.0, 12000.0, 6.0];

// Tunes: numbers the setup screen can change for a test run, with their range.
pub const T_SPAWN_RATE: usize = 0;
pub const T_ENEMY_SPEED: usize = 1;
pub const T_ENEMY_HP: usize = 2;
pub const T_SURVIVE_GOAL: usize = 3;
pub const T_KILLS_GOAL: usize = 4;
pub const T_FORMATION_GOAL: usize = 5;
pub const T_FORMATION_NODE_HP: usize = 6;
pub const T_FORMATION_DIST_MIN: usize = 7;
pub const T_FORMATION_DIST_MAX: usize = 8;
pub const T_FORMATION_SPAWN_COUNT: usize = 9;
pub const T_ARENA_GOAL: usize = 10;
pub const T_ARENA_SIZE: usize = 11;
pub const T_ARENA_HP_PER_WAVE: usize = 12;
pub const T_RETRIEVAL_GOAL: usize = 13;
pub const T_RETRIEVAL_DIST_MIN: usize = 14;
pub const T_RETRIEVAL_DIST_MAX: usize = 15;
pub const T_RETRIEVAL_SLOW: usize = 16;
pub const T_PURSUIT_GOAL: usize = 17;
pub const T_PURSUIT_LEG_LEN: usize = 18;
pub const T_PURSUIT_SPEED: usize = 19;
pub const T_PURSUIT_START_DIST: usize = 20;
pub const T_PURSUIT_PACK: usize = 21;
pub const T_ESCORT_LEG_LEN: usize = 22;
pub const T_ESCORT_SPEED: usize = 23;
pub const T_ESCORT_HP: usize = 24;
pub const T_ESCORT_FOLLOW: usize = 25;
pub const T_HOLD_GOAL: usize = 26;
pub const T_HOLD_RADIUS: usize = 27;
pub const T_HOLD_POINT_HP: usize = 28;
pub const T_HOLD_SHARE: usize = 29;
pub const T_ESCAPE_GOAL: usize = 30;
pub const T_ESCAPE_RING_COUNT: usize = 31;
pub const T_ESCAPE_AHEAD_RATE: usize = 32;
pub const T_ESCAPE_BRUTE_SHARE: usize = 33;
pub const T_BOSS_GOAL: usize = 34;
pub const T_BOSS_WARMUP: usize = 35;
pub const T_BOSS_WALK_SPEED: usize = 36;
pub const T_BOSS_SPAWN_SCALE: usize = 37;
pub const T_VENDETTA_GOAL: usize = 38;
pub const T_VENDETTA_DIST_MIN: usize = 39;
pub const T_VENDETTA_DIST_MAX: usize = 40;
pub const T_VENDETTA_GROUP_HP: usize = 41;
pub const T_ACCEPT_DIST: usize = 42;
pub const T_ACCEPT_TIME: usize = 43;
pub const T_ELEMENT_CARDS: usize = 44;
pub const T_START_ELEMENT: usize = 45;
pub const N_TUNES: usize = 46;
#[rustfmt::skip]
pub const TUNE_NAMES: [&str; N_TUNES] = ["spawn_rate", "enemy_speed", "enemy_hp", "survive_goal", "kills_goal", "formation_goal", "formation_node_hp", "formation_dist_min", "formation_dist_max", "formation_spawn_count", "arena_goal", "arena_size", "arena_hp_per_wave", "retrieval_goal", "retrieval_dist_min", "retrieval_dist_max", "retrieval_slow", "pursuit_goal", "pursuit_leg_len", "pursuit_speed", "pursuit_start_dist", "pursuit_pack", "escort_leg_len", "escort_speed", "escort_hp", "escort_follow", "hold_goal", "hold_radius", "hold_point_hp", "hold_share", "escape_goal", "escape_ring_count", "escape_ahead_rate", "escape_brute_share", "boss_goal", "boss_warmup", "boss_walk_speed", "boss_spawn_scale", "vendetta_goal", "vendetta_dist_min", "vendetta_dist_max", "vendetta_group_hp", "accept_dist", "accept_time", "element_cards", "start_element"];
#[rustfmt::skip]
pub const TUNE_DEFAULT: [f32; N_TUNES] = [1.0, 1.0, 1.0, 300.0, 1500.0, 5.0, 2000.0, 1300.0, 2000.0, 3.0, 10.0, 1.0, 0.1, 3.0, 5000.0, 8000.0, 0.12, 3000.0, 3600.0, 170.0, 800.0, 8.0, 3000.0, 70.0, 360.0, 240.0, 300.0, 160.0, 800.0, 3.0, 18000.0, 36.0, 20.0, 0.25, 12000.0, 90.0, 85.0, 0.4, 6.0, 16000.0, 20000.0, 240.0, 600.0, 3.0, 0.35, 0.0];
#[rustfmt::skip]
pub const TUNE_MIN: [f32; N_TUNES] = [0.0, 0.5, 0.25, 30.0, 100.0, 1.0, 250.0, 300.0, 300.0, 0.0, 1.0, 0.25, 0.0, 1.0, 500.0, 500.0, 0.0, 500.0, 1200.0, 100.0, 300.0, 0.0, 500.0, 30.0, 60.0, 100.0, 30.0, 60.0, 100.0, 1.0, 2000.0, 0.0, 0.0, 0.0, 2000.0, 0.0, 40.0, 0.0, 1.0, 2000.0, 2000.0, 60.0, 0.0, 0.0, 0.0, 0.0];
#[rustfmt::skip]
pub const TUNE_MAX: [f32; N_TUNES] = [4.0, 4.0, 4.0, 900.0, 5000.0, 8.0, 6000.0, 6000.0, 6000.0, 8.0, 10.0, 3.0, 0.5, 6.0, 12000.0, 12000.0, 0.3, 12000.0, 9600.0, 220.0, 2000.0, 24.0, 5000.0, 160.0, 1200.0, 500.0, 600.0, 400.0, 3000.0, 6.0, 30000.0, 120.0, 80.0, 0.5, 30000.0, 180.0, 160.0, 1.0, 12.0, 30000.0, 30000.0, 1200.0, 3000.0, 10.0, 1.0, 7.0];
// The tune that holds each mission's goal; N_TUNES where the goal cannot be changed.
#[rustfmt::skip]
pub const MISSION_GOAL_TUNE: [usize; N_MISSIONS] = [N_TUNES, T_SURVIVE_GOAL, T_KILLS_GOAL, T_FORMATION_GOAL, T_ARENA_GOAL, T_RETRIEVAL_GOAL, T_PURSUIT_GOAL, N_TUNES, T_HOLD_GOAL, T_ESCAPE_GOAL, T_BOSS_GOAL, T_VENDETTA_GOAL];

// HUD state buffer: size and the index of each field.
pub const STATE_FLOATS: usize = 80;
pub const S_HP: usize = 0;
pub const S_MAX_HP: usize = 1;
pub const S_XP: usize = 2;
pub const S_XP_NEXT: usize = 3;
pub const S_LEVEL: usize = 4;
pub const S_TIME: usize = 5;
pub const S_KILLS: usize = 6;
pub const S_CHOOSING: usize = 7;
pub const S_CHOICE0: usize = 8;
pub const S_OVER: usize = 11;
pub const S_CAM_X: usize = 12;
pub const S_CAM_Y: usize = 13;
pub const S_ENEMIES: usize = 14;
pub const S_RUNNING: usize = 15;
pub const S_UPGRADE0: usize = 16;
pub const S_MISSION: usize = 26;
pub const S_GOAL: usize = 27;
pub const S_PROGRESS: usize = 28;
pub const S_WON: usize = 29;
pub const S_ESCAPE: usize = 32;
pub const S_ESCAPED: usize = 33;
pub const S_CARRIED: usize = 36;
pub const S_ACCEPTING: usize = 40;
pub const S_ACCEPT_HELD: usize = 41;
pub const S_ACCEPT_NEED: usize = 42;
pub const S_ESCORT_HP: usize = 44;
pub const S_ESCORT_MAX_HP: usize = 45;
pub const S_ESCORT_FELL: usize = 46;
pub const S_POINT_HP: usize = 48;
pub const S_POINT_MAX: usize = 49;
pub const S_POINT_FELL: usize = 50;
pub const S_EXIT_X: usize = 52;
pub const S_EXIT_Y: usize = 53;
pub const S_EXIT_DIST: usize = 54;
pub const S_BOSS_WAIT: usize = 56;
pub const S_ATTACH: usize = 60;
pub const S_GUARD: usize = 61;
pub const S_GUARD_MAX: usize = 62;
pub const S_REGROW: usize = 63;
pub const S_ART_ELEMENT0: usize = 64;

pub const INK: [f32; 3] = [0.17, 0.15, 0.13];
