// Generated from content/game.toml by tools/gen. Do not edit: change the
// content file and run ./build.sh. Imported through src/data.js.

/** Part ids by name. A name is also the key for an image override in assets/parts.json. */
export const PART_IDS = {
  ellipse: 0,
  player_body: 1,
  player_head: 2,
  scythe: 3,
  blot: 4,
  wisp: 5,
  brute: 6,
  gem: 7,
  slash: 8,
  talisman: 9,
  bead: 10,
  splat: 11,
  grass: 12,
  stone: 13,
  heart: 14,
  ring: 15,
  scrap: 16,
  digit: 17,
  ground_shadow: 18,
};

/** Upgrades, indexed by id. `glyph` is the brush mark shown on cards and HUD chips.
 * Names and descriptions are in src/i18n.js as upgrade.<id>.name and .desc. */
export const UPGRADES = [
  { name: 'scythe', glyph: '鐮', max: 6 },
  { name: 'talisman', glyph: '符', max: 6 },
  { name: 'beads', glyph: '珠', max: 6 },
  { name: 'pulse', glyph: '波', max: 6 },
  { name: 'vitality', glyph: '命', max: 5 },
  { name: 'swift', glyph: '風', max: 5 },
  { name: 'lure', glyph: '引', max: 5 },
  { name: 'might', glyph: '力', max: 5 },
  { name: 'haste', glyph: '速', max: 5 },
  { name: 'mending', glyph: '癒', max: 5 },
  { name: 'jieli', glyph: '借', max: 3 },
  { name: 'qiguan', glyph: '貫', max: 3 },
  { name: 'wanfa', glyph: '歸', max: 3 },
];

/** Enemy kinds, indexed by id. `heavy` kinds are hard to push and stop the game for a moment when they fall. */
export const KINDS = [
  { id: 0, name: 'blot', heavy: false },
  { id: 1, name: 'wisp', heavy: false },
  { id: 2, name: 'brute', heavy: true },
  { id: 3, name: 'elite', heavy: true },
  { id: 4, name: 'hound', heavy: false },
  { id: 5, name: 'bandit', heavy: false },
  { id: 6, name: 'constable', heavy: false },
  { id: 7, name: 'needler', heavy: false },
  { id: 8, name: 'puppet', heavy: false },
  { id: 9, name: 'bandit_chief', heavy: true },
  { id: 10, name: 'constable_chief', heavy: true },
  { id: 11, name: 'claw_crone', heavy: false },
  { id: 12, name: 'bandit_master', heavy: true },
  { id: 13, name: 'constable_master', heavy: true },
  { id: 14, name: 'sect_master', heavy: false },
];

/** Martial arts, indexed by id. `upgrade` is the id of the upgrade that levels the art, and
 * whose name, glyph and card text it shows. */
export const ARTS = [
  { id: 0, name: 'scythe', upgrade: 0, behaviour: 'arc' },
  { id: 1, name: 'talisman', upgrade: 1, behaviour: 'projectile' },
  { id: 2, name: 'beads', upgrade: 2, behaviour: 'orbit' },
  { id: 3, name: 'pulse', upgrade: 3, behaviour: 'nova' },
];

/** How many martial arts the player can hold at once. */
export const MAX_ARTS = 6;

/** Weapon kinds (兵器種類), indexed by id: `art` is the art id it is a kind of, `forms` its
 * form tags, `start` the kind an art comes in first. Names are weapon.<name> in src/i18n.js. */
export const WEAPONS = [
  { id: 0, name: 'dandao', art: 0, glyph: '刀', forms: ['short', 'light'], start: true },
  { id: 1, name: 'shuangdao', art: 0, glyph: '雙', forms: ['short', 'light', 'paired'], start: false },
  { id: 2, name: 'dadao', art: 0, glyph: '斬', forms: ['long', 'heavy'], start: false },
  { id: 3, name: 'lian', art: 0, glyph: '鐮', forms: ['short', 'light'], start: false },
  { id: 4, name: 'qimeigun', art: 2, glyph: '棍', forms: [], start: true },
  { id: 5, name: 'changgun', art: 2, glyph: '長', forms: ['long'], start: false },
  { id: 6, name: 'biao', art: 1, glyph: '鏢', forms: ['light'], start: true },
  { id: 7, name: 'feidao', art: 1, glyph: '飛', forms: ['light'], start: false },
  { id: 8, name: 'zhen', art: 1, glyph: '針', forms: ['light'], start: false },
  { id: 9, name: 'tushou', art: 3, glyph: '拳', forms: ['short', 'light'], start: true },
  { id: 10, name: 'hushou', art: 3, glyph: '護', forms: ['short', 'heavy'], start: false },
  { id: 11, name: 'zhaotao', art: 3, glyph: '爪', forms: ['short', 'light'], start: false },
];

/** Schools (門派), indexed by id. Names are in src/i18n.js as school.<name>. */
export const SCHOOLS = [
  { id: 0, name: 'duanyue' },
  { id: 1, name: 'changfeng' },
  { id: 2, name: 'caobang' },
  { id: 3, name: 'tangmen' },
  { id: 4, name: 'xuanzhong' },
  { id: 5, name: 'qiushui' },
  { id: 6, name: 'heifeng' },
  { id: 7, name: 'liushanmen' },
  { id: 8, name: 'jiuyou' },
];

/** Routes (套路), indexed by id. `art` is the art id that learns it; `moves` its move ids in
 * order, the last one the ultimate when `ultimate` is set. Text is route.<name>.name and .desc. */
export const ROUTES = [
  { id: 0, name: 'bengyue', art: 0, school: 0, beat: 1.0, start: true, moves: [0, 1, 2, 3], ultimate: true, suits: [2], zenith: 7 },
  { id: 1, name: 'jinghong', art: 0, school: 1, beat: 0.5, start: true, moves: [4, 5, 6, 7, 8, 9], ultimate: true, suits: [0, 1], zenith: 8 },
  { id: 2, name: 'poyu', art: 2, school: 1, beat: 1.0, start: true, moves: [10, 11, 12, 13], ultimate: true, suits: [5], zenith: 10 },
  { id: 3, name: 'fanjiang', art: 2, school: 2, beat: 1.1, start: true, moves: [14, 15, 16, 17, 18], ultimate: true, suits: [5], zenith: 11 },
  { id: 4, name: 'ershiba', art: 1, school: 3, beat: 0.95, start: true, moves: [19, 20, 21, 22, 23], ultimate: true, suits: [6, 7], zenith: 5 },
  { id: 5, name: 'chuanyang', art: 1, school: 1, beat: 1.3, start: true, moves: [24, 25, 26, 27], ultimate: true, suits: [7, 6], zenith: 9 },
  { id: 6, name: 'muguchenzhong', art: 3, school: 4, beat: 2.2, start: true, moves: [28, 29, 30, 31], ultimate: true, suits: [9, 10], zenith: 7 },
  { id: 7, name: 'zhuoyu', art: 3, school: 5, beat: 0.5, start: true, moves: [32, 33, 34, 35, 36], ultimate: true, suits: [9], zenith: 8 },
  { id: 8, name: 'lulin', art: 0, school: 6, beat: 0.9, start: false, moves: [37, 38, 39], ultimate: true, suits: [2, 0], zenith: 4 },
  { id: 9, name: 'shuihuo', art: 2, school: 7, beat: 1.2, start: false, moves: [40, 41, 42, 43, 44], ultimate: true, suits: [4], zenith: 6 },
  { id: 10, name: 'yinhun', art: 1, school: 8, beat: 0.6, start: false, moves: [45, 46, 47, 48], ultimate: true, suits: [8], zenith: 12 },
  { id: 11, name: 'shesou', art: 3, school: 8, beat: 1.6, start: false, moves: [49, 50, 51], ultimate: true, suits: [11], zenith: 6 },
];

/** Moves (式), indexed by id. Text is move.<name>.name and .desc. */
export const MOVES = [
  { id: 0, name: 'hengyun', route: 0, ultimate: false, stop: false },
  { id: 1, name: 'huishou', route: 0, ultimate: false, stop: false },
  { id: 2, name: 'wanren', route: 0, ultimate: false, stop: true },
  { id: 3, name: 'wuyue', route: 0, ultimate: true, stop: true },
  { id: 4, name: 'yipie', route: 1, ultimate: false, stop: false },
  { id: 5, name: 'pianruo', route: 1, ultimate: false, stop: false },
  { id: 6, name: 'shuangyan', route: 1, ultimate: false, stop: false },
  { id: 7, name: 'hongfei', route: 1, ultimate: false, stop: false },
  { id: 8, name: 'xueni', route: 1, ultimate: false, stop: false },
  { id: 9, name: 'zhaoying', route: 1, ultimate: true, stop: true },
  { id: 10, name: 'shouwei', route: 2, ultimate: false, stop: false },
  { id: 11, name: 'fengjuan', route: 2, ultimate: false, stop: false },
  { id: 12, name: 'luoye', route: 2, ultimate: false, stop: false },
  { id: 13, name: 'bafang', route: 2, ultimate: true, stop: true },
  { id: 14, name: 'chaosheng', route: 3, ultimate: false, stop: false },
  { id: 15, name: 'jingtao', route: 3, ultimate: false, stop: false },
  { id: 16, name: 'nishui', route: 3, ultimate: false, stop: false },
  { id: 17, name: 'dajiang', route: 3, ultimate: false, stop: false },
  { id: 18, name: 'changhe', route: 3, ultimate: true, stop: true },
  { id: 19, name: 'sanxing', route: 4, ultimate: false, stop: false },
  { id: 20, name: 'wuxing', route: 4, ultimate: false, stop: false },
  { id: 21, name: 'qixing', route: 4, ultimate: false, stop: false },
  { id: 22, name: 'xingluo', route: 4, ultimate: false, stop: false },
  { id: 23, name: 'zhoutian', route: 4, ultimate: true, stop: true },
  { id: 24, name: 'moshi', route: 5, ultimate: false, stop: false },
  { id: 25, name: 'sanjian', route: 5, ultimate: false, stop: false },
  { id: 26, name: 'shuangdiao', route: 5, ultimate: false, stop: false },
  { id: 27, name: 'houyi', route: 5, ultimate: true, stop: true },
  { id: 28, name: 'chenzhong', route: 6, ultimate: false, stop: false },
  { id: 29, name: 'yuyin', route: 6, ultimate: false, stop: false },
  { id: 30, name: 'geshan', route: 6, ultimate: false, stop: false },
  { id: 31, name: 'hongzhong', route: 6, ultimate: true, stop: true },
  { id: 32, name: 'qiecuo', route: 7, ultimate: false, stop: false },
  { id: 33, name: 'zhuomo', route: 7, ultimate: false, stop: false },
  { id: 34, name: 'tashan', route: 7, ultimate: false, stop: false },
  { id: 35, name: 'yushi', route: 7, ultimate: false, stop: false },
  { id: 36, name: 'ningwei', route: 7, ultimate: true, stop: true },
  { id: 37, name: 'menghu', route: 8, ultimate: false, stop: false },
  { id: 38, name: 'kaitang', route: 8, ultimate: false, stop: true },
  { id: 39, name: 'shede', route: 8, ultimate: true, stop: true },
  { id: 40, name: 'qianyang', route: 9, ultimate: false, stop: false },
  { id: 41, name: 'yuqin', route: 9, ultimate: false, stop: false },
  { id: 42, name: 'shengdong', route: 9, ultimate: false, stop: false },
  { id: 43, name: 'dacao', route: 9, ultimate: false, stop: false },
  { id: 44, name: 'guanmen', route: 9, ultimate: true, stop: true },
  { id: 45, name: 'huangquan', route: 10, ultimate: false, stop: false },
  { id: 46, name: 'guimen', route: 10, ultimate: false, stop: false },
  { id: 47, name: 'yanwang', route: 10, ultimate: false, stop: false },
  { id: 48, name: 'shidian', route: 10, ultimate: true, stop: true },
  { id: 49, name: 'gouhun', route: 11, ultimate: false, stop: false },
  { id: 50, name: 'poguan', route: 11, ultimate: false, stop: false },
  { id: 51, name: 'wangui', route: 11, ultimate: true, stop: true },
];

/** Magic elements. Ids start at 1 (0 means no element), so element `id` is ELEMENTS[id - 1].
 * `color` is linear-ish RGB, 0..1. `tags` says whether a hit leaves the element's tag on the enemy.
 * Names and descriptions are in src/i18n.js as element.<name>.name and .desc. */
export const ELEMENTS = [
  { id: 1, name: 'metal', glyph: '金', color: [0.80, 0.70, 0.36], tags: false },
  { id: 2, name: 'wood', glyph: '木', color: [0.36, 0.62, 0.30], tags: false },
  { id: 3, name: 'water', glyph: '水', color: [0.25, 0.48, 0.70], tags: true },
  { id: 4, name: 'fire', glyph: '火', color: [0.82, 0.34, 0.18], tags: true },
  { id: 5, name: 'earth', glyph: '土', color: [0.55, 0.42, 0.26], tags: true },
  { id: 6, name: 'lightning', glyph: '雷', color: [0.56, 0.40, 0.78], tags: true },
  { id: 7, name: 'holy', glyph: '聖', color: [0.92, 0.86, 0.62], tags: true },
];

/** Reactions between the tags of elements `a` and `b` on one enemy. */
export const REACTIONS = [
  { id: 0, name: 'blast', a: 'fire', b: 'lightning' },
  { id: 1, name: 'conduct', a: 'water', b: 'lightning' },
  { id: 2, name: 'steam', a: 'water', b: 'fire' },
];

/** Event types reported by the core. */
export const EVENT = {
  HIT: 1,
  KILL: 2,
  PICKUP: 3,
  LEVELUP: 4,
  HURT: 5,
  SLASH: 6,
  SHOOT: 7,
  PULSE: 8,
  DEATH: 9,
  HEAL: 10,
  ELITE: 11,
  WIN: 12,
  REACT: 13,
  ATTACH: 14,
  COMMISSION: 15,
  COMMISSION_DONE: 16,
  COMMISSION_FAILED: 17,
  MOVE: 18,
  STEAL: 19,
  FOE: 20,
  MANUAL: 21,
  ZENITH: 22,
};

/** Missions, indexed by id. `goal` is what the mission asks for (seconds for survive, enemies for kills).
 * Names and descriptions are in src/i18n.js as mission.<name>.name and .desc. */
export const MISSIONS = [
  { id: 0, name: 'endless', goal: 0 },
  { id: 1, name: 'survive', goal: 300 },
  { id: 2, name: 'kills', goal: 1500 },
  { id: 3, name: 'formation', goal: 5 },
  { id: 4, name: 'arena', goal: 10 },
  { id: 5, name: 'retrieval', goal: 3 },
  { id: 6, name: 'pursuit', goal: 3000 },
  { id: 7, name: 'escort', goal: 21000 },
  { id: 8, name: 'hold', goal: 300 },
  { id: 9, name: 'escape', goal: 18000 },
  { id: 10, name: 'boss', goal: 12000 },
  { id: 11, name: 'vendetta', goal: 6 },
];

/** Mission ids by name. */
export const MISSION_IDS = {
  endless: 0,
  survive: 1,
  kills: 2,
  formation: 3,
  arena: 4,
  retrieval: 5,
  pursuit: 6,
  escort: 7,
  hold: 8,
  escape: 9,
  boss: 10,
  vendetta: 11,
};

/** Tunes, indexed by id: numbers the setup screen can change for a test run. `mission` is the
 * mission that shows the tune, or 'any'. Labels are in src/i18n.js as tune.<name>. */
export const TUNES = [
  { id: 0, name: 'spawn_rate', mission: 'any', def: 1.0, min: 0.0, max: 4.0, step: 0.25 },
  { id: 1, name: 'enemy_speed', mission: 'any', def: 1.0, min: 0.5, max: 4.0, step: 0.25 },
  { id: 2, name: 'enemy_hp', mission: 'any', def: 1.0, min: 0.25, max: 4.0, step: 0.25 },
  { id: 3, name: 'survive_goal', mission: 'survive', def: 300, min: 30, max: 900, step: 30 },
  { id: 4, name: 'kills_goal', mission: 'kills', def: 1500, min: 100, max: 5000, step: 100 },
  { id: 5, name: 'formation_goal', mission: 'formation', def: 5, min: 1, max: 8, step: 1 },
  { id: 6, name: 'formation_node_hp', mission: 'formation', def: 2000, min: 250, max: 6000, step: 250 },
  { id: 7, name: 'formation_dist_min', mission: 'formation', def: 1300, min: 300, max: 6000, step: 100 },
  { id: 8, name: 'formation_dist_max', mission: 'formation', def: 2000, min: 300, max: 6000, step: 100 },
  { id: 9, name: 'formation_spawn_count', mission: 'formation', def: 3, min: 0, max: 8, step: 1 },
  { id: 10, name: 'arena_goal', mission: 'arena', def: 10, min: 1, max: 10, step: 1 },
  { id: 11, name: 'arena_size', mission: 'arena', def: 1.0, min: 0.25, max: 3.0, step: 0.25 },
  { id: 12, name: 'arena_hp_per_wave', mission: 'arena', def: 0.1, min: 0.0, max: 0.5, step: 0.05 },
  { id: 13, name: 'retrieval_goal', mission: 'retrieval', def: 3, min: 1, max: 6, step: 1 },
  { id: 14, name: 'retrieval_dist_min', mission: 'retrieval', def: 5000, min: 500, max: 12000, step: 500 },
  { id: 15, name: 'retrieval_dist_max', mission: 'retrieval', def: 8000, min: 500, max: 12000, step: 500 },
  { id: 16, name: 'retrieval_slow', mission: 'retrieval', def: 0.12, min: 0.0, max: 0.3, step: 0.02 },
  { id: 17, name: 'pursuit_goal', mission: 'pursuit', def: 3000, min: 500, max: 12000, step: 500 },
  { id: 18, name: 'pursuit_leg_len', mission: 'pursuit', def: 3600, min: 1200, max: 9600, step: 600 },
  { id: 19, name: 'pursuit_speed', mission: 'pursuit', def: 170, min: 100, max: 220, step: 10 },
  { id: 20, name: 'pursuit_start_dist', mission: 'pursuit', def: 800, min: 300, max: 2000, step: 100 },
  { id: 21, name: 'pursuit_pack', mission: 'pursuit', def: 8, min: 0, max: 24, step: 2 },
  { id: 22, name: 'escort_leg_len', mission: 'escort', def: 3000, min: 500, max: 5000, step: 250 },
  { id: 23, name: 'escort_speed', mission: 'escort', def: 70, min: 30, max: 160, step: 10 },
  { id: 24, name: 'escort_hp', mission: 'escort', def: 360, min: 60, max: 1200, step: 60 },
  { id: 25, name: 'escort_follow', mission: 'escort', def: 240, min: 100, max: 500, step: 20 },
  { id: 26, name: 'hold_goal', mission: 'hold', def: 300, min: 30, max: 600, step: 30 },
  { id: 27, name: 'hold_radius', mission: 'hold', def: 160, min: 60, max: 400, step: 20 },
  { id: 28, name: 'hold_point_hp', mission: 'hold', def: 800, min: 100, max: 3000, step: 100 },
  { id: 29, name: 'hold_share', mission: 'hold', def: 3, min: 1, max: 6, step: 1 },
  { id: 30, name: 'escape_goal', mission: 'escape', def: 18000, min: 2000, max: 30000, step: 2000 },
  { id: 31, name: 'escape_ring_count', mission: 'escape', def: 36, min: 0, max: 120, step: 6 },
  { id: 32, name: 'escape_ahead_rate', mission: 'escape', def: 20, min: 0, max: 80, step: 5 },
  { id: 33, name: 'escape_brute_share', mission: 'escape', def: 0.25, min: 0.0, max: 0.5, step: 0.05 },
  { id: 34, name: 'boss_goal', mission: 'boss', def: 12000, min: 2000, max: 30000, step: 2000 },
  { id: 35, name: 'boss_warmup', mission: 'boss', def: 90, min: 0, max: 180, step: 15 },
  { id: 36, name: 'boss_walk_speed', mission: 'boss', def: 85, min: 40, max: 160, step: 5 },
  { id: 37, name: 'boss_spawn_scale', mission: 'boss', def: 0.4, min: 0.0, max: 1.0, step: 0.1 },
  { id: 38, name: 'vendetta_goal', mission: 'vendetta', def: 6, min: 1, max: 12, step: 1 },
  { id: 39, name: 'vendetta_dist_min', mission: 'vendetta', def: 16000, min: 2000, max: 30000, step: 2000 },
  { id: 40, name: 'vendetta_dist_max', mission: 'vendetta', def: 20000, min: 2000, max: 30000, step: 2000 },
  { id: 41, name: 'vendetta_group_hp', mission: 'vendetta', def: 240, min: 60, max: 1200, step: 60 },
  { id: 42, name: 'accept_dist', mission: 'any', def: 600, min: 0, max: 3000, step: 100 },
  { id: 43, name: 'accept_time', mission: 'any', def: 3, min: 0, max: 10, step: 1 },
  { id: 44, name: 'element_cards', mission: 'any', def: 0.35, min: 0.0, max: 1.0, step: 0.05 },
  { id: 45, name: 'start_element', mission: 'any', def: 0, min: 0, max: 7, step: 1 },
  { id: 46, name: 'chasers', mission: 'any', def: 0.0, min: 0.0, max: 0.6, step: 0.05 },
  { id: 47, name: 'ambush', mission: 'any', def: 0.0, min: 0.0, max: 1.0, step: 0.1 },
  { id: 48, name: 'arena_radius', mission: 'any', def: 0, min: 0, max: 3000, step: 100 },
  { id: 49, name: 'commissions', mission: 'endless', def: 1, min: 0, max: 1, step: 1 },
  { id: 50, name: 'routes', mission: 'any', def: 1, min: 0, max: 1, step: 1 },
  { id: 51, name: 'decks', mission: 'any', def: 1, min: 0, max: 1, step: 1 },
  { id: 52, name: 'school_share', mission: 'any', def: 0.04, min: 0.0, max: 0.25, step: 0.01 },
  { id: 53, name: 'stolen', mission: 'host', def: 0, min: 0, max: 16777215, step: 1 },
  { id: 54, name: 'manuals', mission: 'host', def: 0, min: 0, max: 16777215, step: 1 },
];

/** Size of the state array written by the core, in floats. */
export const STATE_FLOATS = 128;

/** Indices into the state array. A run of floats has a 0 suffix (CHOICE0). */
export const STATE = {
  HP: 0,
  MAX_HP: 1,
  XP: 2,
  XP_NEXT: 3,
  LEVEL: 4,
  TIME: 5,
  KILLS: 6,
  CHOOSING: 7,
  CHOICE0: 8,
  OVER: 11,
  CAM_X: 12,
  CAM_Y: 13,
  ENEMIES: 14,
  RUNNING: 15,
  DECK_PICK: 16,
  DECK_OPEN0: 17,
  LAST_DECK: 22,
  CHOICE_RANK0: 23,
  MISSION: 26,
  GOAL: 27,
  PROGRESS: 28,
  WON: 29,
  ESCAPE: 32,
  ESCAPED: 33,
  CARRIED: 36,
  ACCEPTING: 40,
  ACCEPT_HELD: 41,
  ACCEPT_NEED: 42,
  ESCORT_HP: 44,
  ESCORT_MAX_HP: 45,
  ESCORT_FELL: 46,
  POINT_HP: 48,
  POINT_MAX: 49,
  POINT_FELL: 50,
  EXIT_X: 52,
  EXIT_Y: 53,
  EXIT_DIST: 54,
  BOSS_WAIT: 56,
  ATTACH: 60,
  GUARD: 61,
  GUARD_MAX: 62,
  REGROW: 63,
  ART_ELEMENT0: 64,
  COMMISSION: 70,
  COMMISSION_LEFT: 71,
  COMMISSION_NEED: 72,
  COMMISSION_HELD: 73,
  COMMISSION_HP: 74,
  COMMISSION_MAX: 75,
  COMMISSION_DONE: 76,
  COMMISSION_KIND: 77,
  COMMISSION_BAND: 78,
  COMMISSION_WALKED: 79,
  ROUTE_PICK: 80,
  ART_ROUTE0: 81,
  ART_STEP0: 87,
  ART_ORDER0: 91,
  ROUTES_ON: 95,
  UPGRADE0: 96,
  ART_KIND0: 112,
  ART_QUALITY0: 116,
  STOLEN: 120,
  MANUALS: 121,
};

export const INK = [0.17, 0.15, 0.13];
