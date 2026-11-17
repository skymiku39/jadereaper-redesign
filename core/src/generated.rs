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
pub const P_DIGIT: f32 = 17.0;
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
pub const U_JIELI: usize = 10;
pub const U_QIGUAN: usize = 11;
pub const U_WANFA: usize = 12;
pub const U_SWORD: usize = 13;
pub const U_SPEAR: usize = 14;
pub const N_UPGRADES: usize = 15;
#[rustfmt::skip]
pub const UPGRADE_NAMES: [&str; N_UPGRADES] = ["scythe", "talisman", "beads", "pulse", "vitality", "swift", "lure", "might", "haste", "mending", "jieli", "qiguan", "wanfa", "sword", "spear"];
pub const UPGRADE_MAX: [u8; N_UPGRADES] = [6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 3, 3, 3, 6, 6];

// Passive upgrades: the stat each one raises and by how much per level.
pub const N_PASSIVES: usize = 9;
#[rustfmt::skip]
pub const PASSIVES: [PassiveDef; N_PASSIVES] = [
    PassiveDef { upgrade: U_VITALITY, stat: ST_MAX_HP, amount: 20.0 },
    PassiveDef { upgrade: U_SWIFT, stat: ST_MOVE_SPEED, amount: 0.08 },
    PassiveDef { upgrade: U_LURE, stat: ST_PICKUP_RANGE, amount: 0.3 },
    PassiveDef { upgrade: U_MIGHT, stat: ST_DAMAGE, amount: 0.12 },
    PassiveDef { upgrade: U_HASTE, stat: ST_COOLDOWN, amount: 0.92 },
    PassiveDef { upgrade: U_MENDING, stat: ST_REGEN, amount: 0.4 },
    PassiveDef { upgrade: U_JIELI, stat: ST_SPLASH, amount: 0.15 },
    PassiveDef { upgrade: U_QIGUAN, stat: ST_REACH, amount: 0.1 },
    PassiveDef { upgrade: U_WANFA, stat: ST_ELEMENT, amount: 0.15 },
];

// Enemy kinds. Kind 0 fills whatever share of the horde the others leave.
pub const K_BLOT: u8 = 0;
pub const K_WISP: u8 = 1;
pub const K_BRUTE: u8 = 2;
pub const K_ELITE: u8 = 3;
pub const K_HOUND: u8 = 4;
pub const K_BANDIT: u8 = 5;
pub const K_CONSTABLE: u8 = 6;
pub const K_NEEDLER: u8 = 7;
pub const K_PUPPET: u8 = 8;
pub const K_BANDIT_CHIEF: u8 = 9;
pub const K_CONSTABLE_CHIEF: u8 = 10;
pub const K_CLAW_CRONE: u8 = 11;
pub const K_BANDIT_MASTER: u8 = 12;
pub const K_CONSTABLE_MASTER: u8 = 13;
pub const K_SECT_MASTER: u8 = 14;
pub const K_TANG_DISCIPLE: u8 = 15;
pub const K_SOLDIER: u8 = 16;
pub const K_GENERAL: u8 = 17;
pub const N_KINDS: usize = 18;
#[rustfmt::skip]
pub const KIND_NAMES: [&str; N_KINDS] = ["blot", "wisp", "brute", "elite", "hound", "bandit", "constable", "needler", "puppet", "bandit_chief", "constable_chief", "claw_crone", "bandit_master", "constable_master", "sect_master", "tang_disciple", "soldier", "general"];
#[rustfmt::skip]
pub const KINDS: [Kind; N_KINDS] = [
    Kind { hp: 14.0, speed: 62.0, r: 16.0, dmg: 5.0, xp: 1.0, part: P_BLOT, color: [0.33, 0.31, 0.30], hw: 18.0, hh: 18.0, rate: 5.0, corpse: false, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: N_TUNES, school: N_SCHOOLS, strike: N_MOVES, manual: false },
    Kind { hp: 7.0, speed: 118.0, r: 12.0, dmg: 4.0, xp: 1.0, part: P_WISP, color: [0.36, 0.40, 0.56], hw: 22.0, hh: 13.0, rate: 9.0, corpse: true, heavy: false, hover: true, from: 25.0, share: 0.3, ramp: 0.0, every: 0.0, share_tune: N_TUNES, school: N_SCHOOLS, strike: N_MOVES, manual: false },
    Kind { hp: 70.0, speed: 44.0, r: 26.0, dmg: 12.0, xp: 5.0, part: P_BRUTE, color: [0.58, 0.31, 0.24], hw: 30.0, hh: 30.0, rate: 3.0, corpse: false, heavy: true, hover: false, from: 60.0, share: 0.25, ramp: 400.0, every: 0.0, share_tune: N_TUNES, school: N_SCHOOLS, strike: N_MOVES, manual: false },
    Kind { hp: 600.0, speed: 56.0, r: 44.0, dmg: 20.0, xp: 40.0, part: P_BRUTE, color: [0.74, 0.25, 0.18], hw: 52.0, hh: 52.0, rate: 2.5, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 90.0, share_tune: N_TUNES, school: N_SCHOOLS, strike: N_MOVES, manual: false },
    Kind { hp: 10.0, speed: 215.0, r: 11.0, dmg: 6.0, xp: 1.0, part: P_WISP, color: [0.52, 0.20, 0.16], hw: 17.0, hh: 10.0, rate: 12.0, corpse: false, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_CHASERS, school: N_SCHOOLS, strike: N_MOVES, manual: false },
    Kind { hp: 22.0, speed: 70.0, r: 17.0, dmg: 6.0, xp: 2.0, part: P_BLOT, color: [0.50, 0.30, 0.20], hw: 19.0, hh: 19.0, rate: 6.0, corpse: false, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_HEIFENG, strike: N_MOVES, manual: false },
    Kind { hp: 26.0, speed: 66.0, r: 17.0, dmg: 6.0, xp: 2.0, part: P_BLOT, color: [0.22, 0.28, 0.42], hw: 19.0, hh: 19.0, rate: 5.0, corpse: false, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_LIUSHANMEN, strike: N_MOVES, manual: false },
    Kind { hp: 12.0, speed: 104.0, r: 12.0, dmg: 5.0, xp: 2.0, part: P_WISP, color: [0.42, 0.24, 0.46], hw: 22.0, hh: 13.0, rate: 9.0, corpse: false, heavy: false, hover: true, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_JIUYOU, strike: N_MOVES, manual: false },
    Kind { hp: 34.0, speed: 48.0, r: 18.0, dmg: 7.0, xp: 2.0, part: P_BLOT, color: [0.38, 0.44, 0.36], hw: 20.0, hh: 20.0, rate: 3.0, corpse: true, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_JIUYOU, strike: N_MOVES, manual: false },
    Kind { hp: 240.0, speed: 58.0, r: 26.0, dmg: 9.0, xp: 12.0, part: P_BRUTE, color: [0.55, 0.24, 0.14], hw: 30.0, hh: 30.0, rate: 3.0, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 75.0, share_tune: T_SCHOOL_SHARE, school: SC_HEIFENG, strike: MV_KAITANG, manual: false },
    Kind { hp: 280.0, speed: 54.0, r: 26.0, dmg: 8.0, xp: 12.0, part: P_BRUTE, color: [0.16, 0.22, 0.40], hw: 30.0, hh: 30.0, rate: 3.0, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 105.0, share_tune: T_SCHOOL_SHARE, school: SC_LIUSHANMEN, strike: MV_QIANYANG, manual: false },
    Kind { hp: 200.0, speed: 72.0, r: 20.0, dmg: 8.0, xp: 12.0, part: P_WISP, color: [0.30, 0.40, 0.30], hw: 30.0, hh: 18.0, rate: 6.0, corpse: true, heavy: false, hover: true, from: 0.0, share: 0.0, ramp: 0.0, every: 135.0, share_tune: T_SCHOOL_SHARE, school: SC_JIUYOU, strike: MV_GOUHUN, manual: false },
    Kind { hp: 1100.0, speed: 52.0, r: 40.0, dmg: 14.0, xp: 40.0, part: P_BRUTE, color: [0.42, 0.16, 0.08], hw: 48.0, hh: 48.0, rate: 2.5, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 240.0, share_tune: T_SCHOOL_SHARE, school: SC_HEIFENG, strike: MV_SHEDE, manual: true },
    Kind { hp: 1200.0, speed: 50.0, r: 40.0, dmg: 12.0, xp: 40.0, part: P_BRUTE, color: [0.10, 0.14, 0.32], hw: 48.0, hh: 48.0, rate: 2.5, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 300.0, share_tune: T_SCHOOL_SHARE, school: SC_LIUSHANMEN, strike: MV_GUANMEN, manual: true },
    Kind { hp: 1000.0, speed: 64.0, r: 30.0, dmg: 12.0, xp: 40.0, part: P_WISP, color: [0.22, 0.30, 0.20], hw: 44.0, hh: 26.0, rate: 5.0, corpse: true, heavy: false, hover: true, from: 0.0, share: 0.0, ramp: 0.0, every: 360.0, share_tune: T_SCHOOL_SHARE, school: SC_JIUYOU, strike: MV_WANGUI, manual: true },
    Kind { hp: 16.0, speed: 92.0, r: 12.0, dmg: 5.0, xp: 2.0, part: P_WISP, color: [0.30, 0.34, 0.30], hw: 22.0, hh: 13.0, rate: 8.0, corpse: false, heavy: false, hover: true, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_TANGMEN, strike: N_MOVES, manual: false },
    Kind { hp: 30.0, speed: 60.0, r: 18.0, dmg: 7.0, xp: 2.0, part: P_BLOT, color: [0.40, 0.36, 0.24], hw: 19.0, hh: 19.0, rate: 4.0, corpse: false, heavy: false, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 0.0, share_tune: T_SCHOOL_SHARE, school: SC_BIANJUN, strike: N_MOVES, manual: false },
    Kind { hp: 1300.0, speed: 50.0, r: 40.0, dmg: 14.0, xp: 40.0, part: P_BRUTE, color: [0.36, 0.30, 0.14], hw: 48.0, hh: 48.0, rate: 2.5, corpse: false, heavy: true, hover: false, from: 0.0, share: 0.0, ramp: 0.0, every: 280.0, share_tune: T_SCHOOL_SHARE, school: SC_BIANJUN, strike: MV_HENGQIANG, manual: true },
];

// Martial arts: one behaviour each, with its numbers per level of the upgrade that levels it.
pub const A_SCYTHE: usize = 0;
pub const A_TALISMAN: usize = 1;
pub const A_BEADS: usize = 2;
pub const A_PULSE: usize = 3;
pub const A_SWORD: usize = 4;
pub const A_SPEAR: usize = 5;
pub const N_ARTS: usize = 6;
#[rustfmt::skip]
pub const ART_NAMES: [&str; N_ARTS] = ["scythe", "talisman", "beads", "pulse", "sword", "spear"];
/// How many martial arts the player can hold at once.
pub const MAX_ARTS: usize = 6;
#[rustfmt::skip]
pub const ARTS: [ArtDef; N_ARTS] = [
    ArtDef { upgrade: U_SCYTHE, behaviour: B_ARC, part: P_SLASH, start_level: 1, first_delay: 0.6, speed: 0.0, life: 0.0, range: 0.0, routes_only: false, levels: &[
        ArtLevel { damage: 20.0, cooldown: 0.9, count: 1, reach: 119.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 26.0, cooldown: 0.9, count: 1, reach: 133.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 32.0, cooldown: 0.9, count: 1, reach: 147.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 38.0, cooldown: 0.9, count: 1, reach: 161.0, width: -0.15, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 44.0, cooldown: 0.9, count: 2, reach: 175.0, width: -2.0, pierce: 0, knockback: 240.0 },
        ArtLevel { damage: 50.0, cooldown: 0.9, count: 2, reach: 189.0, width: -2.0, pierce: 0, knockback: 240.0 },
    ] },
    ArtDef { upgrade: U_TALISMAN, behaviour: B_PROJECTILE, part: P_TALISMAN, start_level: 0, first_delay: 0.5, speed: 540.0, life: 1.3, range: 520.0, routes_only: false, levels: &[
        ArtLevel { damage: 12.0, cooldown: 0.95, count: 1, reach: 0.0, width: 0.0, pierce: 1, knockback: 0.3 },
        ArtLevel { damage: 16.0, cooldown: 0.95, count: 2, reach: 0.0, width: 0.0, pierce: 1, knockback: 0.3 },
        ArtLevel { damage: 20.0, cooldown: 0.95, count: 2, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 24.0, cooldown: 0.95, count: 3, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 28.0, cooldown: 0.95, count: 3, reach: 0.0, width: 0.0, pierce: 2, knockback: 0.3 },
        ArtLevel { damage: 32.0, cooldown: 0.95, count: 4, reach: 0.0, width: 0.0, pierce: 3, knockback: 0.3 },
    ] },
    ArtDef { upgrade: U_BEADS, behaviour: B_ORBIT, part: P_BEAD, start_level: 0, first_delay: 0.0, speed: 2.8, life: 0.0, range: 0.0, routes_only: false, levels: &[
        ArtLevel { damage: 9.0, cooldown: 0.45, count: 2, reach: 84.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 12.0, cooldown: 0.45, count: 3, reach: 90.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 15.0, cooldown: 0.45, count: 4, reach: 96.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 18.0, cooldown: 0.45, count: 5, reach: 102.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 21.0, cooldown: 0.45, count: 6, reach: 108.0, width: 0.0, pierce: 0, knockback: 160.0 },
        ArtLevel { damage: 24.0, cooldown: 0.45, count: 7, reach: 114.0, width: 0.0, pierce: 0, knockback: 160.0 },
    ] },
    ArtDef { upgrade: U_PULSE, behaviour: B_NOVA, part: P_RING, start_level: 0, first_delay: 2.0, speed: 0.0, life: 0.0, range: 0.0, routes_only: false, levels: &[
        ArtLevel { damage: 16.0, cooldown: 4.2, count: 0, reach: 180.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 22.0, cooldown: 4.2, count: 0, reach: 210.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 28.0, cooldown: 4.2, count: 0, reach: 240.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 34.0, cooldown: 4.2, count: 0, reach: 270.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 40.0, cooldown: 4.2, count: 0, reach: 300.0, width: 0.0, pierce: 0, knockback: 520.0 },
        ArtLevel { damage: 46.0, cooldown: 4.2, count: 0, reach: 330.0, width: 0.0, pierce: 0, knockback: 520.0 },
    ] },
    ArtDef { upgrade: U_SWORD, behaviour: B_THRUST, part: P_TALISMAN, start_level: 0, first_delay: 0.4, speed: 0.0, life: 0.0, range: 0.0, routes_only: true, levels: &[
        ArtLevel { damage: 18.0, cooldown: 0.7, count: 0, reach: 220.0, width: 12.0, pierce: 0, knockback: 180.0 },
        ArtLevel { damage: 24.0, cooldown: 0.7, count: 0, reach: 250.0, width: 13.0, pierce: 0, knockback: 180.0 },
        ArtLevel { damage: 30.0, cooldown: 0.7, count: 0, reach: 270.0, width: 14.0, pierce: 0, knockback: 180.0 },
        ArtLevel { damage: 36.0, cooldown: 0.7, count: 0, reach: 290.0, width: 15.0, pierce: 0, knockback: 180.0 },
        ArtLevel { damage: 42.0, cooldown: 0.7, count: 0, reach: 310.0, width: 16.0, pierce: 0, knockback: 180.0 },
        ArtLevel { damage: 48.0, cooldown: 0.7, count: 0, reach: 330.0, width: 17.0, pierce: 0, knockback: 180.0 },
    ] },
    ArtDef { upgrade: U_SPEAR, behaviour: B_THRUST, part: P_TALISMAN, start_level: 0, first_delay: 0.6, speed: 0.0, life: 0.0, range: 0.0, routes_only: true, levels: &[
        ArtLevel { damage: 22.0, cooldown: 1.0, count: 0, reach: 300.0, width: 10.0, pierce: 0, knockback: 260.0 },
        ArtLevel { damage: 28.0, cooldown: 1.0, count: 0, reach: 330.0, width: 11.0, pierce: 0, knockback: 260.0 },
        ArtLevel { damage: 34.0, cooldown: 1.0, count: 0, reach: 360.0, width: 12.0, pierce: 0, knockback: 260.0 },
        ArtLevel { damage: 40.0, cooldown: 1.0, count: 0, reach: 390.0, width: 13.0, pierce: 0, knockback: 260.0 },
        ArtLevel { damage: 46.0, cooldown: 1.0, count: 0, reach: 420.0, width: 14.0, pierce: 0, knockback: 260.0 },
        ArtLevel { damage: 52.0, cooldown: 1.0, count: 0, reach: 450.0, width: 15.0, pierce: 0, knockback: 260.0 },
    ] },
];

// Weapon kinds (兵器種類): the kinds each art's weapon comes in, with form tags as bits (FORM_*).
pub const W_DANDAO: usize = 0;
pub const W_SHUANGDAO: usize = 1;
pub const W_DADAO: usize = 2;
pub const W_LIAN: usize = 3;
pub const W_QIMEIGUN: usize = 4;
pub const W_CHANGGUN: usize = 5;
pub const W_BIAO: usize = 6;
pub const W_FEIDAO: usize = 7;
pub const W_ZHEN: usize = 8;
pub const W_TUSHOU: usize = 9;
pub const W_HUSHOU: usize = 10;
pub const W_ZHAOTAO: usize = 11;
pub const W_XIUJIAN: usize = 12;
pub const W_JIGUANXIA: usize = 13;
pub const W_CHANGJIAN: usize = 14;
pub const W_SHUANGJIAN: usize = 15;
pub const W_RUANJIAN: usize = 16;
pub const W_CHANGQIANG: usize = 17;
pub const W_HUAQIANG: usize = 18;
pub const W_JI: usize = 19;
pub const N_WEAPONS: usize = 20;
#[rustfmt::skip]
pub const WEAPON_NAMES: [&str; N_WEAPONS] = ["dandao", "shuangdao", "dadao", "lian", "qimeigun", "changgun", "biao", "feidao", "zhen", "tushou", "hushou", "zhaotao", "xiujian", "jiguanxia", "changjian", "shuangjian", "ruanjian", "changqiang", "huaqiang", "ji"];
#[rustfmt::skip]
pub const WEAPONS: [WeaponDef; N_WEAPONS] = [
    WeaponDef { art: A_SCYTHE, forms: FORM_SHORT | FORM_LIGHT, start: true, trait_: WT_NONE },
    WeaponDef { art: A_SCYTHE, forms: FORM_SHORT | FORM_LIGHT | FORM_PAIRED, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SCYTHE, forms: FORM_LONG | FORM_HEAVY, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SCYTHE, forms: FORM_SHORT | FORM_LIGHT, start: false, trait_: WT_HOOK },
    WeaponDef { art: A_BEADS, forms: 0, start: true, trait_: WT_NONE },
    WeaponDef { art: A_BEADS, forms: FORM_LONG, start: false, trait_: WT_NONE },
    WeaponDef { art: A_TALISMAN, forms: FORM_LIGHT, start: true, trait_: WT_NONE },
    WeaponDef { art: A_TALISMAN, forms: FORM_LIGHT, start: false, trait_: WT_PIERCE },
    WeaponDef { art: A_TALISMAN, forms: FORM_LIGHT, start: false, trait_: WT_SWARM },
    WeaponDef { art: A_PULSE, forms: FORM_SHORT | FORM_LIGHT, start: true, trait_: WT_NONE },
    WeaponDef { art: A_PULSE, forms: FORM_SHORT | FORM_HEAVY, start: false, trait_: WT_NONE },
    WeaponDef { art: A_PULSE, forms: FORM_SHORT | FORM_LIGHT, start: false, trait_: WT_NONE },
    WeaponDef { art: A_TALISMAN, forms: FORM_LIGHT | FORM_MECHANISM, start: false, trait_: WT_NONE },
    WeaponDef { art: A_TALISMAN, forms: FORM_HEAVY | FORM_MECHANISM, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SWORD, forms: FORM_LIGHT, start: true, trait_: WT_NONE },
    WeaponDef { art: A_SWORD, forms: FORM_LIGHT | FORM_PAIRED, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SWORD, forms: FORM_LIGHT, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SPEAR, forms: FORM_LONG, start: true, trait_: WT_NONE },
    WeaponDef { art: A_SPEAR, forms: FORM_LIGHT, start: false, trait_: WT_NONE },
    WeaponDef { art: A_SPEAR, forms: FORM_LONG | FORM_HEAVY, start: false, trait_: WT_TIP },
];

// Schools (門派).
pub const SC_DUANYUE: usize = 0;
pub const SC_CHANGFENG: usize = 1;
pub const SC_CAOBANG: usize = 2;
pub const SC_TANGMEN: usize = 3;
pub const SC_XUANZHONG: usize = 4;
pub const SC_QIUSHUI: usize = 5;
pub const SC_HEIFENG: usize = 6;
pub const SC_LIUSHANMEN: usize = 7;
pub const SC_JIUYOU: usize = 8;
pub const SC_QINGMING: usize = 9;
pub const SC_BAIMA: usize = 10;
pub const SC_BIANJUN: usize = 11;
pub const N_SCHOOLS: usize = 12;
#[rustfmt::skip]
pub const SCHOOL_NAMES: [&str; N_SCHOOLS] = ["duanyue", "changfeng", "caobang", "tangmen", "xuanzhong", "qiushui", "heifeng", "liushanmen", "jiuyou", "qingming", "baima", "bianjun"];

// Routes (套路): what an art turns into from level 3, and their moves in order.
pub const RT_BENGYUE: usize = 0;
pub const RT_JINGHONG: usize = 1;
pub const RT_POYU: usize = 2;
pub const RT_FANJIANG: usize = 3;
pub const RT_ERSHIBA: usize = 4;
pub const RT_CHUANYANG: usize = 5;
pub const RT_MUGUCHENZHONG: usize = 6;
pub const RT_ZHUOYU: usize = 7;
pub const RT_LULIN: usize = 8;
pub const RT_SHUIHUO: usize = 9;
pub const RT_YINHUN: usize = 10;
pub const RT_SHESOU: usize = 11;
pub const RT_JIUJI: usize = 12;
pub const RT_LIUYUN: usize = 13;
pub const RT_QIUSHUIJIAN: usize = 14;
pub const RT_XUEHE: usize = 15;
pub const RT_CHANGBAN: usize = 16;
pub const RT_YOULONG: usize = 17;
pub const RT_POJUN: usize = 18;
pub const N_ROUTES: usize = 19;
#[rustfmt::skip]
pub const ROUTE_NAMES: [&str; N_ROUTES] = ["bengyue", "jinghong", "poyu", "fanjiang", "ershiba", "chuanyang", "muguchenzhong", "zhuoyu", "lulin", "shuihuo", "yinhun", "shesou", "jiuji", "liuyun", "qiushuijian", "xuehe", "changban", "youlong", "pojun"];
#[rustfmt::skip]
pub const ROUTES: [RouteDef; N_ROUTES] = [
    RouteDef { art: A_SCYTHE, school: SC_DUANYUE, beat: 1.0, start: true, first: 0, n: 4, suits: 0x4, zenith: U_MIGHT, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SCYTHE, school: SC_CHANGFENG, beat: 0.5, start: true, first: 4, n: 6, suits: 0x3, zenith: U_HASTE, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_BEADS, school: SC_CHANGFENG, beat: 1.0, start: true, first: 10, n: 4, suits: 0x20, zenith: U_JIELI, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_BEADS, school: SC_CAOBANG, beat: 1.1, start: true, first: 14, n: 5, suits: 0x20, zenith: U_QIGUAN, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_TALISMAN, school: SC_TANGMEN, beat: 0.95, start: true, first: 19, n: 5, suits: 0xc0, zenith: U_SWIFT, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_TALISMAN, school: SC_CHANGFENG, beat: 1.3, start: true, first: 24, n: 4, suits: 0xc0, zenith: U_MENDING, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_PULSE, school: SC_XUANZHONG, beat: 2.2, start: true, first: 28, n: 4, suits: 0x600, zenith: U_MIGHT, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_PULSE, school: SC_QIUSHUI, beat: 0.5, start: true, first: 32, n: 5, suits: 0x200, zenith: U_HASTE, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SCYTHE, school: SC_HEIFENG, beat: 0.9, start: false, first: 37, n: 3, suits: 0x5, zenith: U_VITALITY, only: 0x0, rule: RL_CROWD },
    RouteDef { art: A_BEADS, school: SC_LIUSHANMEN, beat: 1.2, start: false, first: 40, n: 5, suits: 0x10, zenith: U_LURE, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_TALISMAN, school: SC_JIUYOU, beat: 0.6, start: false, first: 45, n: 4, suits: 0x100, zenith: U_WANFA, only: 0x0, rule: RL_OPENING },
    RouteDef { art: A_PULSE, school: SC_JIUYOU, beat: 1.6, start: false, first: 49, n: 3, suits: 0x800, zenith: U_LURE, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_TALISMAN, school: SC_TANGMEN, beat: 1.5, start: false, first: 52, n: 5, suits: 0x2000, zenith: U_SWIFT, only: 0x3000, rule: RL_NONE },
    RouteDef { art: A_SWORD, school: SC_QINGMING, beat: 0.55, start: true, first: 57, n: 5, suits: 0x14000, zenith: U_WANFA, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SWORD, school: SC_QIUSHUI, beat: 1.4, start: true, first: 62, n: 4, suits: 0x4000, zenith: U_MENDING, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SWORD, school: SC_JIUYOU, beat: 0.7, start: false, first: 66, n: 4, suits: 0xc000, zenith: U_VITALITY, only: 0x0, rule: RL_MOMENTUM },
    RouteDef { art: A_SPEAR, school: SC_BAIMA, beat: 0.9, start: true, first: 70, n: 5, suits: 0x60000, zenith: U_SWIFT, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SPEAR, school: SC_CAOBANG, beat: 0.6, start: true, first: 75, n: 4, suits: 0x40000, zenith: U_HASTE, only: 0x0, rule: RL_NONE },
    RouteDef { art: A_SPEAR, school: SC_BIANJUN, beat: 1.2, start: false, first: 79, n: 4, suits: 0xa0000, zenith: U_MIGHT, only: 0x0, rule: RL_NONE },
];
pub const MV_HENGYUN: usize = 0;
pub const MV_HUISHOU: usize = 1;
pub const MV_WANREN: usize = 2;
pub const MV_WUYUE: usize = 3;
pub const MV_YIPIE: usize = 4;
pub const MV_PIANRUO: usize = 5;
pub const MV_SHUANGYAN: usize = 6;
pub const MV_HONGFEI: usize = 7;
pub const MV_XUENI: usize = 8;
pub const MV_ZHAOYING: usize = 9;
pub const MV_SHOUWEI: usize = 10;
pub const MV_FENGJUAN: usize = 11;
pub const MV_LUOYE: usize = 12;
pub const MV_BAFANG: usize = 13;
pub const MV_CHAOSHENG: usize = 14;
pub const MV_JINGTAO: usize = 15;
pub const MV_NISHUI: usize = 16;
pub const MV_DAJIANG: usize = 17;
pub const MV_CHANGHE: usize = 18;
pub const MV_SANXING: usize = 19;
pub const MV_WUXING: usize = 20;
pub const MV_QIXING: usize = 21;
pub const MV_XINGLUO: usize = 22;
pub const MV_ZHOUTIAN: usize = 23;
pub const MV_MOSHI: usize = 24;
pub const MV_SANJIAN: usize = 25;
pub const MV_SHUANGDIAO: usize = 26;
pub const MV_HOUYI: usize = 27;
pub const MV_CHENZHONG: usize = 28;
pub const MV_YUYIN: usize = 29;
pub const MV_GESHAN: usize = 30;
pub const MV_HONGZHONG: usize = 31;
pub const MV_QIECUO: usize = 32;
pub const MV_ZHUOMO: usize = 33;
pub const MV_TASHAN: usize = 34;
pub const MV_YUSHI: usize = 35;
pub const MV_NINGWEI: usize = 36;
pub const MV_MENGHU: usize = 37;
pub const MV_KAITANG: usize = 38;
pub const MV_SHEDE: usize = 39;
pub const MV_QIANYANG: usize = 40;
pub const MV_YUQIN: usize = 41;
pub const MV_SHENGDONG: usize = 42;
pub const MV_DACAO: usize = 43;
pub const MV_GUANMEN: usize = 44;
pub const MV_HUANGQUAN: usize = 45;
pub const MV_GUIMEN: usize = 46;
pub const MV_YANWANG: usize = 47;
pub const MV_SHIDIAN: usize = 48;
pub const MV_GOUHUN: usize = 49;
pub const MV_POGUAN: usize = 50;
pub const MV_WANGUI: usize = 51;
pub const MV_GONGSHU: usize = 52;
pub const MV_JIUGONG: usize = 53;
pub const MV_MUNIU: usize = 54;
pub const MV_WANNU: usize = 55;
pub const MV_TIANLUO: usize = 56;
pub const MV_SHANGSHAN: usize = 57;
pub const MV_ROURUO: usize = 58;
pub const MV_DAXIANG: usize = 59;
pub const MV_ZHOUXING: usize = 60;
pub const MV_DAOFA: usize = 61;
pub const MV_CHANGTIAN: usize = 62;
pub const MV_WANGYANG: usize = 63;
pub const MV_HEBOGUANHAI: usize = 64;
pub const MV_TIANDI: usize = 65;
pub const MV_XUEJI: usize = 66;
pub const MV_SHAREN: usize = 67;
pub const MV_LIUCHENG: usize = 68;
pub const MV_DAOXUAN: usize = 69;
pub const MV_BAIMA: usize = 70;
pub const MV_HENGQIANG: usize = 71;
pub const MV_QIJIN: usize = 72;
pub const MV_DANQI: usize = 73;
pub const MV_YISHEN: usize = 74;
pub const MV_BAIWEI: usize = 75;
pub const MV_TAITOU: usize = 76;
pub const MV_RUHAI: usize = 77;
pub const MV_JIAOLONG: usize = 78;
pub const MV_HUANGSHA: usize = 79;
pub const MV_WANLI: usize = 80;
pub const MV_QINGHAI: usize = 81;
pub const MV_QINSHI: usize = 82;
pub const N_MOVES: usize = 83;
#[rustfmt::skip]
pub const MOVE_NAMES: [&str; N_MOVES] = ["hengyun", "huishou", "wanren", "wuyue", "yipie", "pianruo", "shuangyan", "hongfei", "xueni", "zhaoying", "shouwei", "fengjuan", "luoye", "bafang", "chaosheng", "jingtao", "nishui", "dajiang", "changhe", "sanxing", "wuxing", "qixing", "xingluo", "zhoutian", "moshi", "sanjian", "shuangdiao", "houyi", "chenzhong", "yuyin", "geshan", "hongzhong", "qiecuo", "zhuomo", "tashan", "yushi", "ningwei", "menghu", "kaitang", "shede", "qianyang", "yuqin", "shengdong", "dacao", "guanmen", "huangquan", "guimen", "yanwang", "shidian", "gouhun", "poguan", "wangui", "gongshu", "jiugong", "muniu", "wannu", "tianluo", "shangshan", "rouruo", "daxiang", "zhouxing", "daofa", "changtian", "wangyang", "heboguanhai", "tiandi", "xueji", "sharen", "liucheng", "daoxuan", "baima", "hengqiang", "qijin", "danqi", "yishen", "baiwei", "taitou", "ruhai", "jiaolong", "huangsha", "wanli", "qinghai", "qinshi"];
#[rustfmt::skip]
pub const MOVES: [MoveDef; N_MOVES] = [
    MoveDef { route: 0, shape: MS_ARC, power: 1.3, reach: 1.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 0, shape: MS_ARC, power: 1.2, reach: 1.0, width: -0.15, count: 2, spread: 0.5, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 0, shape: MS_ARC, power: 2.0, reach: 1.3, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: true },
    MoveDef { route: 0, shape: MS_ARC, power: 3.5, reach: 1.3, width: -2.0, count: 2, spread: 0.0, delay: 0.15, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.3, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 1, shape: MS_ARC, power: 0.7, reach: 0.9, width: 0.85, count: 1, spread: 0.0, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 1, shape: MS_ARC, power: 0.8, reach: 1.4, width: 0.9, count: 1, spread: 0.0, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 1, shape: MS_ARC, power: 0.6, reach: 1.0, width: 0.85, count: 2, spread: 0.0, delay: 0.1, knockback: 0.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 1, shape: MS_WAVE, power: 1.0, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.3, pierce: 3, speed: 600.0, life: 0.8, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 1, shape: MS_MARKS, power: 0.6, reach: 0.35, width: 0.0, count: 4, spread: 0.5, delay: 0.5, knockback: 0.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 1, shape: MS_WAVE, power: 1.2, reach: 0.0, width: 0.0, count: 7, spread: 0.25, delay: 0.0, knockback: 0.3, pierce: 5, speed: 620.0, life: 0.8, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 2, shape: MS_ARC, power: 1.6, reach: 1.2, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 2, shape: MS_ARC, power: 1.6, reach: 1.3, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 2, shape: MS_RING, power: 1.8, reach: 1.45, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 2, shape: MS_RING, power: 1.4, reach: 1.3, width: 0.0, count: 4, spread: 0.0, delay: 0.25, knockback: 1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.25, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 3, shape: MS_ARC, power: 1.6, reach: 2.0, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 3, shape: MS_RING, power: 2.4, reach: 0.8, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 2.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 3, shape: MS_ARC, power: 1.4, reach: 2.0, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 3, shape: MS_ARC, power: 1.8, reach: 2.2, width: -2.0, count: 2, spread: 0.0, delay: 0.12, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 3, shape: MS_ARC, power: 4.0, reach: 4.0, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 4, shape: MS_WAVE, power: 0.9, reach: 0.0, width: 0.0, count: 3, spread: 0.083, delay: 0.0, knockback: 0.3, pierce: 1, speed: 540.0, life: 1.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 4, shape: MS_WAVE, power: 0.8, reach: 0.0, width: 0.0, count: 5, spread: 0.167, delay: 0.0, knockback: 0.3, pierce: 1, speed: 540.0, life: 1.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 4, shape: MS_WAVE, power: 0.75, reach: 0.0, width: 0.0, count: 7, spread: 0.25, delay: 0.0, knockback: 0.3, pierce: 2, speed: 540.0, life: 1.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 4, shape: MS_WAVE, power: 0.8, reach: 0.0, width: 0.0, count: 3, spread: 0.12, delay: 0.0, knockback: 0.3, pierce: 1, speed: 560.0, life: 0.8, grow: 0.0, offset: 0.0, seek: 1, split: 3, ultimate: false, stop: false },
    MoveDef { route: 4, shape: MS_WAVE, power: 0.8, reach: 0.0, width: 0.0, count: 28, spread: 1.0, delay: 0.0, knockback: 0.3, pierce: 2, speed: 520.0, life: 1.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 5, shape: MS_WAVE, power: 2.0, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.3, pierce: 4, speed: 640.0, life: 1.1, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 5, shape: MS_WAVE, power: 1.6, reach: 0.0, width: 0.0, count: 3, spread: 0.0, delay: 0.1, knockback: 0.3, pierce: 3, speed: 680.0, life: 1.1, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 5, shape: MS_WAVE, power: 2.0, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.2, pierce: 99, speed: 700.0, life: 1.2, grow: 0.15, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 5, shape: MS_WAVE, power: 6.0, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.1, pierce: 999, speed: 1800.0, life: 0.7, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: true, stop: true },
    MoveDef { route: 6, shape: MS_ARC, power: 1.4, reach: 1.25, width: 0.5, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 6, shape: MS_RING, power: 0.9, reach: 1.0, width: 0.0, count: 2, spread: 0.0, delay: 0.5, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 6, shape: MS_RING, power: 2.0, reach: 0.57, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.8, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 1.05, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 6, shape: MS_RING, power: 4.0, reach: 1.9, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.2, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 7, shape: MS_RING, power: 0.6, reach: 0.43, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.3, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 7, shape: MS_RING, power: 0.5, reach: 0.43, width: 0.0, count: 2, spread: 0.0, delay: 0.12, knockback: 0.3, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 7, shape: MS_WAVE, power: 0.5, reach: 0.0, width: 0.0, count: 3, spread: 0.33, delay: 0.0, knockback: 0.3, pierce: 2, speed: 500.0, life: 0.5, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 7, shape: MS_RING, power: 0.5, reach: 0.43, width: 0.0, count: 2, spread: 0.0, delay: 0.2, knockback: 0.3, pierce: 0, speed: 0.0, life: 0.0, grow: 0.5, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 7, shape: MS_RING, power: 0.8, reach: 0.43, width: 0.0, count: 5, spread: 0.0, delay: 0.12, knockback: 0.4, pierce: 0, speed: 0.0, life: 0.0, grow: 0.65, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 8, shape: MS_ARC, power: 1.1, reach: 1.1, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 8, shape: MS_ARC, power: 1.8, reach: 1.0, width: 0.6, count: 1, spread: 0.0, delay: 0.0, knockback: 1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: true },
    MoveDef { route: 8, shape: MS_ARC, power: 3.0, reach: 1.3, width: -2.0, count: 2, spread: 0.0, delay: 0.12, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 9, shape: MS_ARC, power: 1.0, reach: 1.6, width: -2.0, count: 1, spread: 0.0, delay: 0.0, knockback: -1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 9, shape: MS_RING, power: 1.6, reach: 1.2, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.3, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 9, shape: MS_ARC, power: 1.4, reach: 1.5, width: -0.15, count: 2, spread: 0.5, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 9, shape: MS_ARC, power: 1.2, reach: 1.5, width: 0.3, count: 3, spread: 0.0, delay: 0.15, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 9, shape: MS_RING, power: 2.4, reach: 2.8, width: 0.0, count: 2, spread: 0.0, delay: 0.4, knockback: -2.0, pierce: 0, speed: 0.0, life: 0.0, grow: -0.55, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 10, shape: MS_WAVE, power: 0.5, reach: 0.0, width: 0.0, count: 4, spread: 1.0, delay: 0.0, knockback: 0.1, pierce: 1, speed: 520.0, life: 0.9, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 10, shape: MS_WAVE, power: 0.5, reach: 0.0, width: 0.0, count: 8, spread: 1.0, delay: 0.0, knockback: 0.1, pierce: 1, speed: 520.0, life: 0.9, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 10, shape: MS_WAVE, power: 0.6, reach: 0.0, width: 0.0, count: 6, spread: 0.4, delay: 0.0, knockback: 0.1, pierce: 2, speed: 600.0, life: 1.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 10, shape: MS_WAVE, power: 0.5, reach: 0.0, width: 0.0, count: 32, spread: 1.0, delay: 0.0, knockback: 0.1, pierce: 2, speed: 500.0, life: 1.1, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 11, shape: MS_RING, power: 0.5, reach: 1.25, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: -1.2, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 11, shape: MS_RING, power: 1.5, reach: 1.25, width: 0.0, count: 2, spread: 0.0, delay: 0.3, knockback: -0.6, pierce: 0, speed: 0.0, life: 0.0, grow: -0.6, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 11, shape: MS_RING, power: 3.0, reach: 3.0, width: 0.0, count: 2, spread: 0.0, delay: 0.6, knockback: -2.0, pierce: 0, speed: 0.0, life: 0.0, grow: -0.8, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 12, shape: MS_TURRET, power: 0.6, reach: 0.0, width: 120.0, count: 1, spread: 0.0, delay: 0.6, knockback: 0.0, pierce: 1, speed: 620.0, life: 6.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 12, shape: MS_TURRET, power: 0.45, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.5, knockback: 0.0, pierce: 1, speed: 620.0, life: 4.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 12, shape: MS_TURRET, power: 0.45, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.5, knockback: 0.0, pierce: 1, speed: 620.0, life: 5.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 12, shape: MS_TURRET, power: 0.45, reach: 0.0, width: 0.0, count: 1, spread: 0.0, delay: 0.5, knockback: 0.0, pierce: 1, speed: 620.0, life: 4.0, grow: 0.0, offset: 0.0, seek: 0, split: 12, ultimate: false, stop: false },
    MoveDef { route: 12, shape: MS_TURRET, power: 0.4, reach: 0.0, width: 0.0, count: 8, spread: 0.0, delay: 0.35, knockback: 0.0, pierce: 2, speed: 640.0, life: 3.0, grow: 0.0, offset: 1.6, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 13, shape: MS_THRUST, power: 0.7, reach: 1.0, width: 0.0, count: 2, spread: 0.06, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 13, shape: MS_THRUST, power: 0.6, reach: 1.0, width: 0.0, count: 2, spread: 0.0, delay: 0.12, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 13, shape: MS_THRUST, power: 0.6, reach: 1.1, width: 0.0, count: 2, spread: 0.0, delay: 0.4, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 13, shape: MS_THRUST, power: 0.5, reach: 0.6, width: 0.0, count: 6, spread: 1.0, delay: 0.0, knockback: 0.8, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 13, shape: MS_THRUST, power: 1.0, reach: 1.2, width: 0.0, count: 8, spread: 1.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 14, shape: MS_THRUST, power: 1.6, reach: 1.8, width: 1.5, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 14, shape: MS_RING, power: 1.2, reach: 0.4, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.8, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 1.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 14, shape: MS_THRUST, power: 2.5, reach: 2.0, width: 1.4, count: 1, spread: 0.0, delay: 0.0, knockback: 1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: true },
    MoveDef { route: 14, shape: MS_THRUST, power: 4.0, reach: 5.0, width: 2.2, count: 1, spread: 0.0, delay: 0.0, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: true, stop: true },
    MoveDef { route: 15, shape: MS_THRUST, power: 0.9, reach: 1.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 15, shape: MS_THRUST, power: 0.8, reach: 1.0, width: 0.0, count: 2, spread: 0.0, delay: 0.1, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 15, shape: MS_RING, power: 1.0, reach: 0.6, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 15, shape: MS_THRUST, power: 0.8, reach: 1.2, width: 0.0, count: 10, spread: 0.0, delay: 0.06, knockback: 0.4, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: true, stop: true },
    MoveDef { route: 16, shape: MS_ARC, power: 1.0, reach: 0.8, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 16, shape: MS_ARC, power: 1.2, reach: 1.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 16, shape: MS_THRUST, power: 0.4, reach: 0.8, width: 0.0, count: 7, spread: 1.0, delay: 0.0, knockback: 0.8, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: false, stop: false },
    MoveDef { route: 16, shape: MS_THRUST, power: 0.6, reach: 1.0, width: 0.0, count: 3, spread: 0.5, delay: 0.0, knockback: 0.8, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 16, shape: MS_THRUST, power: 0.5, reach: 1.0, width: 0.0, count: 12, spread: 1.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 0, split: 0, ultimate: true, stop: true },
    MoveDef { route: 17, shape: MS_THRUST, power: 0.8, reach: 1.0, width: 0.0, count: 3, spread: 0.08, delay: 0.0, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 17, shape: MS_THRUST, power: 0.9, reach: 1.0, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 2.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 17, shape: MS_THRUST, power: 0.7, reach: 1.1, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: -1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 17, shape: MS_THRUST, power: 0.6, reach: 1.2, width: 0.0, count: 6, spread: 0.0, delay: 0.3, knockback: 0.6, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: true, stop: true },
    MoveDef { route: 18, shape: MS_THRUST, power: 1.0, reach: 1.5, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 18, shape: MS_THRUST, power: 1.0, reach: 1.3, width: 0.0, count: 1, spread: 0.0, delay: 0.0, knockback: 3.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 18, shape: MS_THRUST, power: 0.9, reach: 1.3, width: 0.0, count: 3, spread: 0.05, delay: 0.0, knockback: 1.0, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: false, stop: false },
    MoveDef { route: 18, shape: MS_THRUST, power: 4.0, reach: 4.0, width: 3.0, count: 1, spread: 0.0, delay: 0.0, knockback: 1.5, pierce: 0, speed: 0.0, life: 0.0, grow: 0.0, offset: 0.0, seek: 1, split: 0, ultimate: true, stop: true },
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
pub const EV_COMMISSION: f32 = 15.0;
pub const EV_COMMISSION_DONE: f32 = 16.0;
pub const EV_COMMISSION_FAILED: f32 = 17.0;
pub const EV_MOVE: f32 = 18.0;
pub const EV_STEAL: f32 = 19.0;
pub const EV_FOE: f32 = 20.0;
pub const EV_MANUAL: f32 = 21.0;
pub const EV_ZENITH: f32 = 22.0;

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
pub const T_CHASERS: usize = 46;
pub const T_AMBUSH: usize = 47;
pub const T_ARENA_RADIUS: usize = 48;
pub const T_COMMISSIONS: usize = 49;
pub const T_ROUTES: usize = 50;
pub const T_DECKS: usize = 51;
pub const T_SCHOOL_SHARE: usize = 52;
pub const T_STOLEN: usize = 53;
pub const T_MANUALS: usize = 54;
pub const N_TUNES: usize = 55;
#[rustfmt::skip]
pub const TUNE_NAMES: [&str; N_TUNES] = ["spawn_rate", "enemy_speed", "enemy_hp", "survive_goal", "kills_goal", "formation_goal", "formation_node_hp", "formation_dist_min", "formation_dist_max", "formation_spawn_count", "arena_goal", "arena_size", "arena_hp_per_wave", "retrieval_goal", "retrieval_dist_min", "retrieval_dist_max", "retrieval_slow", "pursuit_goal", "pursuit_leg_len", "pursuit_speed", "pursuit_start_dist", "pursuit_pack", "escort_leg_len", "escort_speed", "escort_hp", "escort_follow", "hold_goal", "hold_radius", "hold_point_hp", "hold_share", "escape_goal", "escape_ring_count", "escape_ahead_rate", "escape_brute_share", "boss_goal", "boss_warmup", "boss_walk_speed", "boss_spawn_scale", "vendetta_goal", "vendetta_dist_min", "vendetta_dist_max", "vendetta_group_hp", "accept_dist", "accept_time", "element_cards", "start_element", "chasers", "ambush", "arena_radius", "commissions", "routes", "decks", "school_share", "stolen", "manuals"];
#[rustfmt::skip]
pub const TUNE_DEFAULT: [f32; N_TUNES] = [1.0, 1.0, 1.0, 300.0, 1500.0, 5.0, 2000.0, 1300.0, 2000.0, 3.0, 10.0, 1.0, 0.1, 3.0, 5000.0, 8000.0, 0.12, 3000.0, 3600.0, 170.0, 800.0, 8.0, 3000.0, 70.0, 360.0, 240.0, 300.0, 160.0, 800.0, 3.0, 18000.0, 36.0, 20.0, 0.25, 12000.0, 90.0, 85.0, 0.4, 6.0, 16000.0, 20000.0, 240.0, 600.0, 3.0, 0.35, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.04, 0.0, 0.0];
#[rustfmt::skip]
pub const TUNE_MIN: [f32; N_TUNES] = [0.0, 0.5, 0.25, 30.0, 100.0, 1.0, 250.0, 300.0, 300.0, 0.0, 1.0, 0.25, 0.0, 1.0, 500.0, 500.0, 0.0, 500.0, 1200.0, 100.0, 300.0, 0.0, 500.0, 30.0, 60.0, 100.0, 30.0, 60.0, 100.0, 1.0, 2000.0, 0.0, 0.0, 0.0, 2000.0, 0.0, 40.0, 0.0, 1.0, 2000.0, 2000.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
#[rustfmt::skip]
pub const TUNE_MAX: [f32; N_TUNES] = [4.0, 4.0, 4.0, 900.0, 5000.0, 8.0, 6000.0, 6000.0, 6000.0, 8.0, 10.0, 3.0, 0.5, 6.0, 12000.0, 12000.0, 0.3, 12000.0, 9600.0, 220.0, 2000.0, 24.0, 5000.0, 160.0, 1200.0, 500.0, 600.0, 400.0, 3000.0, 6.0, 30000.0, 120.0, 80.0, 0.5, 30000.0, 180.0, 160.0, 1.0, 12.0, 30000.0, 30000.0, 1200.0, 3000.0, 10.0, 1.0, 7.0, 0.6, 1.0, 3000.0, 1.0, 1.0, 1.0, 0.25, 16777215.0, 16777215.0];
// The tune that holds each mission's goal; N_TUNES where the goal cannot be changed.
#[rustfmt::skip]
pub const MISSION_GOAL_TUNE: [usize; N_MISSIONS] = [N_TUNES, T_SURVIVE_GOAL, T_KILLS_GOAL, T_FORMATION_GOAL, T_ARENA_GOAL, T_RETRIEVAL_GOAL, T_PURSUIT_GOAL, N_TUNES, T_HOLD_GOAL, T_ESCAPE_GOAL, T_BOSS_GOAL, T_VENDETTA_GOAL];

// HUD state buffer: size and the index of each field.
pub const STATE_FLOATS: usize = 136;
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
pub const S_DECK_PICK: usize = 16;
pub const S_DECK_OPEN0: usize = 17;
pub const S_LAST_DECK: usize = 22;
pub const S_CHOICE_RANK0: usize = 23;
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
pub const S_COMMISSION: usize = 70;
pub const S_COMMISSION_LEFT: usize = 71;
pub const S_COMMISSION_NEED: usize = 72;
pub const S_COMMISSION_HELD: usize = 73;
pub const S_COMMISSION_HP: usize = 74;
pub const S_COMMISSION_MAX: usize = 75;
pub const S_COMMISSION_DONE: usize = 76;
pub const S_COMMISSION_KIND: usize = 77;
pub const S_COMMISSION_BAND: usize = 78;
pub const S_COMMISSION_WALKED: usize = 79;
pub const S_ROUTE_PICK: usize = 80;
pub const S_ART_ROUTE0: usize = 81;
pub const S_ART_STEP0: usize = 87;
pub const S_ROUTES_ON: usize = 95;
pub const S_UPGRADE0: usize = 96;
pub const S_ART_KIND0: usize = 112;
pub const S_STOLEN: usize = 120;
pub const S_MANUALS: usize = 121;
pub const S_ART_QUALITY0: usize = 122;
pub const S_ART_ORDER0: usize = 128;

pub const INK: [f32; 3] = [0.17, 0.15, 0.13];
