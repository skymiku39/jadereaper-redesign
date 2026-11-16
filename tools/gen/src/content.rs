//! Reads `content/game.toml`, checks it, and renders the generated code.
//!
//! Pure functions only (text in, text out), so the generator binary and the
//! tests in `core/tests/content.rs` share them. No file access here.
//!
//! The content file uses a small subset of TOML: top-level `key = value`
//! lines and `[[table]]` entries holding `key = value` lines. Values are
//! integers, decimals, basic "strings", booleans and single-line arrays.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Paths relative to the repository root.
pub const CONTENT_PATH: &str = "content/game.toml";
pub const RUST_OUT: &str = "core/src/generated.rs";
pub const JS_OUT: &str = "src/generated.js";
pub const WGSL_PATH: &str = "src/shader.wgsl";

pub const WGSL_BEGIN: &str = "// BEGIN GENERATED from content/game.toml by tools/gen. Do not edit; run ./build.sh.";
pub const WGSL_END: &str = "// END GENERATED";

/// Image slots per part in the shader (`PartRects` in src/shader.wgsl and
/// the rect buffer in src/renderer.js). Part ids must fit.
pub const PART_SLOTS: u32 = 32;

// ------------------------------------------------------------------ parsing

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    /// A decimal, kept as written so generated literals match the source.
    Num(String),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
}

impl Value {
    fn kind(&self) -> &'static str {
        match self {
            Value::Int(_) => "an integer",
            Value::Num(_) => "a decimal",
            Value::Str(_) => "a string",
            Value::Bool(_) => "a boolean",
            Value::Array(_) => "an array",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub key: String,
    pub value: Value,
    pub line: usize,
}

/// The top-level keys (name "") or one `[[name]]` entry.
#[derive(Clone, Debug)]
pub struct Table {
    pub name: String,
    pub line: usize,
    pub entries: Vec<Entry>,
}

/// Parse the TOML subset. The first table is the top level.
pub fn parse(src: &str) -> Result<Vec<Table>, String> {
    let mut tables = vec![Table { name: String::new(), line: 0, entries: Vec::new() }];
    for (i, raw) in src.lines().enumerate() {
        let line = i + 1;
        let text = raw.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        if let Some(rest) = text.strip_prefix("[[") {
            let end = rest.find("]]").ok_or(format!("line {line}: missing ]] in table header"))?;
            let name = rest[..end].trim();
            let after = rest[end + 2..].trim();
            if !after.is_empty() && !after.starts_with('#') {
                return Err(format!("line {line}: unexpected text after table header"));
            }
            if !is_name(name) {
                return Err(format!("line {line}: bad table name `{name}`"));
            }
            tables.push(Table { name: name.to_string(), line, entries: Vec::new() });
            continue;
        }
        if text.starts_with('[') {
            return Err(format!("line {line}: only [[table]] headers are supported"));
        }
        let eq = text.find('=').ok_or(format!("line {line}: expected `key = value`"))?;
        let key = text[..eq].trim();
        if !is_name(key) {
            return Err(format!("line {line}: bad key `{key}`"));
        }
        let mut cur = Cursor { s: &text.as_bytes()[eq + 1..], i: 0, line };
        cur.skip_ws();
        let value = cur.value()?;
        cur.skip_ws();
        if cur.i < cur.s.len() && cur.s[cur.i] != b'#' {
            return Err(format!("line {line}: unexpected text after value"));
        }
        let table = tables.last_mut().unwrap();
        if table.entries.iter().any(|e| e.key == key) {
            return Err(format!("line {line}: duplicate key `{key}`"));
        }
        table.entries.push(Entry { key: key.to_string(), value, line });
    }
    Ok(tables)
}

struct Cursor<'a> {
    s: &'a [u8],
    i: usize,
    line: usize,
}

impl Cursor<'_> {
    fn skip_ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] == b' ' || self.s[self.i] == b'\t') {
            self.i += 1;
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        let line = self.line;
        match self.s.get(self.i) {
            None => Err(format!("line {line}: missing value")),
            Some(b'"') => self.string(),
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.s.get(self.i) == Some(&b']') {
                        self.i += 1;
                        return Ok(Value::Array(items));
                    }
                    items.push(self.value()?);
                    self.skip_ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {}
                        _ => return Err(format!("line {line}: expected , or ] in array")),
                    }
                }
            }
            Some(_) => {
                let start = self.i;
                while self.i < self.s.len() && !b" \t,]#".contains(&self.s[self.i]) {
                    self.i += 1;
                }
                let word = std::str::from_utf8(&self.s[start..self.i]).unwrap();
                match word {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ if word.contains(['.', 'e', 'E']) && word.parse::<f64>().is_ok() => {
                        Ok(Value::Num(word.to_string()))
                    }
                    _ => word
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| format!("line {line}: cannot read value `{word}`")),
                }
            }
        }
    }

    fn string(&mut self) -> Result<Value, String> {
        let line = self.line;
        self.i += 1;
        let mut out = Vec::new();
        loop {
            match self.s.get(self.i) {
                None => return Err(format!("line {line}: unterminated string")),
                Some(b'"') => {
                    self.i += 1;
                    return Ok(Value::Str(String::from_utf8(out).unwrap()));
                }
                Some(b'\\') => {
                    let c = match self.s.get(self.i + 1) {
                        Some(b'"') => b'"',
                        Some(b'\\') => b'\\',
                        _ => return Err(format!("line {line}: only \\\" and \\\\ escapes are supported")),
                    };
                    out.push(c);
                    self.i += 2;
                }
                Some(&c) => {
                    out.push(c);
                    self.i += 1;
                }
            }
        }
    }
}

/// Lowercase snake_case: a letter, then letters, digits or underscores.
fn is_name(s: &str) -> bool {
    let mut b = s.bytes();
    matches!(b.next(), Some(b'a'..=b'z')) && b.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
}

// ------------------------------------------------------------------ content

#[derive(Clone, Debug)]
pub struct Part {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Upgrade {
    pub id: u32,
    pub name: String,
    pub max: u32,
    pub glyph: String,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Mission {
    pub id: u32,
    pub name: String,
    /// What the mission asks for, in its own unit (seconds for `survive`, enemies for `kills`). 0 for none.
    pub goal: u32,
}

/// A number of a mission that the setup screen can change for a test run.
/// The four values are kept as written so generated literals match the source.
#[derive(Clone, Debug)]
pub struct Tune {
    pub id: u32,
    pub name: String,
    /// The mission whose setup screen shows it, or "any" for every mission.
    pub mission: String,
    pub default: String,
    pub min: String,
    pub max: String,
    pub step: String,
}

/// The numbers of an enemy, in the order of `Enemy::numbers`.
pub const ENEMY_KEYS: [&str; 8] = ["hp", "speed", "radius", "damage", "xp", "half_width", "half_height", "pose_rate"];

/// The optional numbers of how an enemy joins the horde, in the order of `Enemy::spawn`. Each defaults to 0.
pub const ENEMY_SPAWN_KEYS: [&str; 4] = ["from", "share", "ramp", "every"];

/// An enemy kind: its numbers, how it is drawn and how it joins the horde.
#[derive(Clone, Debug)]
pub struct Enemy {
    pub id: u32,
    pub name: String,
    pub part: String,
    pub color: [String; 3],
    /// One value per key of `ENEMY_KEYS`, as written.
    pub numbers: Vec<String>,
    pub corpse: bool,
    pub heavy: bool,
    pub hover: bool,
    /// One value per key of `ENEMY_SPAWN_KEYS`, as written.
    pub spawn: Vec<String>,
    /// The tune that holds its share of the horde instead of `share`, if any.
    pub share_tune: Option<String>,
}

/// What a martial art does. The core has one function per behaviour.
pub const BEHAVIOURS: [&str; 4] = ["arc", "projectile", "orbit", "nova"];

/// The numbers of a martial art that can differ per level, in the order of `Art::levels`.
pub const ART_LEVEL_KEYS: [&str; 7] = ["damage", "cooldown", "count", "reach", "width", "pierce", "knockback"];

/// The level keys that are whole numbers.
const ART_WHOLE_KEYS: [&str; 2] = ["count", "pierce"];

/// A martial art (武學): one behaviour with its numbers. Its level is the level of the upgrade of the same name.
#[derive(Clone, Debug)]
pub struct Art {
    pub id: u32,
    pub name: String,
    pub behaviour: String,
    /// The part drawn for its swing, shot, bead or ring.
    pub part: String,
    /// The level the player holds it at when a run starts; 0 for not held.
    pub start_level: u32,
    pub first_delay: String,
    pub speed: String,
    pub life: String,
    pub range: String,
    /// One list per key of `ART_LEVEL_KEYS`, each with one value per level, as written.
    pub levels: Vec<Vec<String>>,
}

/// What an element does on a hit. The core has one piece of code per effect.
/// `guard` and `regrow` act on the player and leave no tag on the enemy.
pub const EFFECTS: [&str; 7] = ["guard", "regrow", "surge", "burn", "root", "chain", "bane"];

/// The optional numbers of an element, in the order of `Element::numbers`. Each defaults to 0.
pub const ELEMENT_KEYS: [&str; 7] = ["power", "minor", "time", "cap", "rate", "reach", "rest"];

/// A magic element (魔法) that a martial art can carry.
#[derive(Clone, Debug)]
pub struct Element {
    pub id: u32,
    pub name: String,
    pub glyph: String,
    pub color: [String; 3],
    pub effect: String,
    /// One value per key of `ELEMENT_KEYS`, as written.
    pub numbers: Vec<String>,
    pub count: u32,
}

impl Element {
    /// Whether a hit leaves this element's tag on the enemy.
    pub fn tags(&self) -> bool {
        self.effect != "guard" && self.effect != "regrow"
    }
}

/// How a reaction plays out. The core has one piece of code per kind.
pub const REACTION_KINDS: [&str; 2] = ["burst", "relay"];

/// What happens when an enemy holds the tags of two elements at once.
#[derive(Clone, Debug)]
pub struct Reaction {
    pub id: u32,
    pub name: String,
    pub a: String,
    pub b: String,
    pub kind: String,
    pub power: String,
    pub reach: String,
    pub count: u32,
    pub color: [String; 3],
}

#[derive(Clone, Debug)]
pub struct StateField {
    pub name: String,
    pub index: u32,
    /// Some for a run of floats, None for a single float.
    pub len: Option<u32>,
}

/// Checked content, each list sorted by id (state by index).
#[derive(Clone, Debug)]
pub struct Content {
    pub ink: [String; 3],
    pub state_floats: u32,
    /// How many martial arts the player can hold at once.
    pub max_arts: u32,
    pub parts: Vec<Part>,
    pub upgrades: Vec<Upgrade>,
    pub enemies: Vec<Enemy>,
    pub arts: Vec<Art>,
    pub elements: Vec<Element>,
    pub reactions: Vec<Reaction>,
    pub events: Vec<Event>,
    pub missions: Vec<Mission>,
    pub tunes: Vec<Tune>,
    pub state: Vec<StateField>,
}

/// Reads typed fields out of one table and collects errors.
struct Fields<'a> {
    t: &'a Table,
    used: Vec<&'a str>,
    errs: &'a mut Vec<String>,
}

impl<'a> Fields<'a> {
    fn get(&mut self, key: &'a str) -> Option<&'a Entry> {
        self.used.push(key);
        self.t.entries.iter().find(|e| e.key == key)
    }

    fn where_(&self) -> String {
        if self.t.name.is_empty() {
            "top level".to_string()
        } else {
            format!("[[{}]] at line {}", self.t.name, self.t.line)
        }
    }

    fn missing(&mut self, key: &str) {
        let w = self.where_();
        self.errs.push(format!("{w}: missing `{key}`"));
    }

    fn wrong(&mut self, e: &Entry, want: &str) {
        self.errs.push(format!("line {}: `{}` must be {want}, not {}", e.line, e.key, e.value.kind()));
    }

    fn uint(&mut self, key: &'a str) -> Option<u32> {
        let e = match self.get(key) {
            Some(e) => e,
            None => {
                self.missing(key);
                return None;
            }
        };
        match e.value {
            Value::Int(v) if (0..=u32::MAX as i64).contains(&v) => Some(v as u32),
            _ => {
                self.wrong(e, "a whole number of 0 or more");
                None
            }
        }
    }

    fn string(&mut self, key: &'a str) -> Option<String> {
        let e = match self.get(key) {
            Some(e) => e,
            None => {
                self.missing(key);
                return None;
            }
        };
        match &e.value {
            Value::Str(s) => Some(s.clone()),
            _ => {
                self.wrong(e, "a string");
                None
            }
        }
    }

    /// A whole number or a decimal, as written.
    fn number(&mut self, key: &'a str) -> Option<String> {
        let e = match self.get(key) {
            Some(e) => e,
            None => {
                self.missing(key);
                return None;
            }
        };
        match &e.value {
            Value::Int(v) => Some(v.to_string()),
            Value::Num(s) => Some(s.clone()),
            _ => {
                self.wrong(e, "a number");
                None
            }
        }
    }

    /// A number that may be left out, in which case it is 0.
    fn number_or_zero(&mut self, key: &'a str) -> Option<String> {
        if self.t.entries.iter().any(|e| e.key == key) {
            self.number(key)
        } else {
            self.used.push(key);
            Some("0".to_string())
        }
    }

    /// A whole number that may be left out, in which case it is 0.
    fn uint_or_zero(&mut self, key: &'a str) -> Option<u32> {
        if self.t.entries.iter().any(|e| e.key == key) {
            self.uint(key)
        } else {
            self.used.push(key);
            Some(0)
        }
    }

    /// A true or false that may be left out, in which case it is false.
    fn flag(&mut self, key: &'a str) -> Option<bool> {
        match self.get(key) {
            None => Some(false),
            Some(e) => match e.value {
                Value::Bool(b) => Some(b),
                _ => {
                    self.wrong(e, "true or false");
                    None
                }
            },
        }
    }

    /// Three numbers from 0 to 1, as written.
    fn rgb(&mut self, key: &'a str) -> Option<[String; 3]> {
        let e = match self.get(key) {
            Some(e) => e,
            None => {
                self.missing(key);
                return None;
            }
        };
        let items = match &e.value {
            Value::Array(items) if items.len() == 3 => items,
            _ => {
                self.wrong(e, "an array of 3 numbers");
                return None;
            }
        };
        let mut out: Vec<String> = Vec::new();
        for v in items {
            let text = match v {
                Value::Int(i) => i.to_string(),
                Value::Num(s) => s.clone(),
                _ => break,
            };
            match text.parse::<f64>() {
                Ok(x) if (0.0..=1.0).contains(&x) => out.push(text),
                _ => break,
            }
        }
        if out.len() != 3 {
            self.errs.push(format!("line {}: `{key}` values must be numbers from 0 to 1", e.line));
            return None;
        }
        Some([out[0].clone(), out[1].clone(), out[2].clone()])
    }

    /// One of a fixed set of words.
    fn word(&mut self, key: &'a str, allowed: &[&str]) -> Option<String> {
        let line = self.t.entries.iter().find(|e| e.key == key).map_or(self.t.line, |e| e.line);
        let w = self.string(key)?;
        if !allowed.contains(&w.as_str()) {
            self.errs.push(format!("line {line}: `{key}` is \"{w}\", but must be one of: {}", allowed.join(", ")));
            return None;
        }
        Some(w)
    }

    /// A number for every level, or a list with one number per level, kept as
    /// written with its line. A key left out is 0 at every level.
    fn per_level(&mut self, key: &'a str) -> Option<(Vec<String>, usize)> {
        let e = match self.get(key) {
            Some(e) => e,
            None => return Some((vec!["0".to_string()], self.t.line)),
        };
        let text = |v: &Value| match v {
            Value::Int(i) => Some(i.to_string()),
            Value::Num(s) => Some(s.clone()),
            _ => None,
        };
        let list: Option<Vec<String>> = match &e.value {
            Value::Array(items) if !items.is_empty() => items.iter().map(text).collect(),
            Value::Array(_) => None,
            v => text(v).map(|s| vec![s]),
        };
        match list {
            Some(list) => Some((list, e.line)),
            None => {
                self.wrong(e, "a number or a list of numbers");
                None
            }
        }
    }

    fn name(&mut self) -> Option<String> {
        let line = self.t.entries.iter().find(|e| e.key == "name").map_or(self.t.line, |e| e.line);
        let n = self.string("name")?;
        if !is_name(&n) {
            self.errs.push(format!("line {line}: name `{n}` must be lowercase snake_case"));
            return None;
        }
        Some(n)
    }

    /// Report keys this table has but nothing read.
    fn finish(self) {
        for e in &self.t.entries {
            if !self.used.contains(&e.key.as_str()) {
                self.errs.push(format!("line {}: unknown key `{}`", e.line, e.key));
            }
        }
    }
}

/// Table names the content file may use, also valid as a `len` reference.
const TABLES: [&str; 10] =
    ["part", "upgrade", "enemy", "art", "element", "reaction", "event", "mission", "tune", "state"];

/// Parse and check the content file. Returns every problem found.
pub fn load(src: &str) -> Result<Content, Vec<String>> {
    let tables = parse(src).map_err(|e| vec![e])?;
    let mut errs = Vec::new();

    // Top level.
    let mut f = Fields { t: &tables[0], used: Vec::new(), errs: &mut errs };
    let ink = f.rgb("ink");
    let state_floats = f.uint("state_floats");
    let max_arts = f.uint("max_arts");
    f.finish();
    if max_arts == Some(0) {
        errs.push("`max_arts` must be 1 or more".to_string());
    }

    let mut parts = Vec::new();
    let mut upgrades = Vec::new();
    // An art with its per-level lists as written: (art, [(values, line) per level key], line).
    type RawLevels = Vec<(Vec<String>, usize)>;
    let mut arts: Vec<(Art, RawLevels, usize)> = Vec::new();
    let mut elements = Vec::new();
    let mut reactions = Vec::new();
    let mut enemies = Vec::new();
    let mut events = Vec::new();
    let mut missions = Vec::new();
    let mut tunes = Vec::new();
    // (name, index, len given as written, line)
    let mut raw_state: Vec<(String, u32, Option<Value>, usize)> = Vec::new();

    for t in &tables[1..] {
        let mut f = Fields { t, used: Vec::new(), errs: &mut errs };
        match t.name.as_str() {
            "part" => {
                if let (Some(id), Some(name)) = (f.uint("id"), f.name()) {
                    if id >= PART_SLOTS {
                        f.errs.push(format!("line {}: part id {id} must be below {PART_SLOTS}", t.line));
                    }
                    parts.push((Part { id, name }, t.line));
                }
            }
            "upgrade" => {
                let (id, name, max, glyph) = (f.uint("id"), f.name(), f.uint("max"), f.string("glyph"));
                if let (Some(id), Some(name), Some(max), Some(glyph)) = (id, name, max, glyph) {
                    if !(1..=255).contains(&max) {
                        f.errs.push(format!("line {}: upgrade `max` must be 1 to 255", t.line));
                    }
                    if glyph.chars().count() != 1 {
                        f.errs.push(format!("line {}: upgrade `glyph` must be one character", t.line));
                    }
                    upgrades.push((Upgrade { id, name, max, glyph }, t.line));
                }
            }
            "enemy" => {
                let (id, name, part, color) = (f.uint("id"), f.name(), f.string("part"), f.rgb("color"));
                let numbers: Option<Vec<String>> = ENEMY_KEYS.iter().map(|key| f.number(key)).collect();
                let (corpse, heavy, hover) = (f.flag("corpse"), f.flag("heavy"), f.flag("hover"));
                let spawn: Option<Vec<String>> = ENEMY_SPAWN_KEYS.iter().map(|key| f.number_or_zero(key)).collect();
                let share_tune = match f.get("tune") {
                    None => Some(None),
                    Some(e) => match &e.value {
                        Value::Str(s) => Some(Some(s.clone())),
                        _ => {
                            f.wrong(e, "a string");
                            None
                        }
                    },
                };
                if let (
                    Some(id),
                    Some(name),
                    Some(part),
                    Some(color),
                    Some(numbers),
                    Some(corpse),
                    Some(heavy),
                    Some(hover),
                    Some(spawn),
                    Some(share_tune),
                ) = (id, name, part, color, numbers, corpse, heavy, hover, spawn, share_tune)
                {
                    let enemy = Enemy { id, name, part, color, numbers, corpse, heavy, hover, spawn, share_tune };
                    enemies.push((enemy, t.line));
                }
            }
            "art" => {
                let (id, name) = (f.uint("id"), f.name());
                let (behaviour, part) = (f.word("behaviour", &BEHAVIOURS), f.string("part"));
                let start_level = f.uint_or_zero("start_level");
                let first_delay = f.number_or_zero("first_delay");
                let (speed, life, range) =
                    (f.number_or_zero("speed"), f.number_or_zero("life"), f.number_or_zero("range"));
                let raw: Option<RawLevels> = ART_LEVEL_KEYS.iter().map(|key| f.per_level(key)).collect();
                if let (
                    Some(id),
                    Some(name),
                    Some(behaviour),
                    Some(part),
                    Some(start_level),
                    Some(first_delay),
                    Some(speed),
                    Some(life),
                    Some(range),
                    Some(raw),
                ) = (id, name, behaviour, part, start_level, first_delay, speed, life, range, raw)
                {
                    let art = Art {
                        id,
                        name,
                        behaviour,
                        part,
                        start_level,
                        first_delay,
                        speed,
                        life,
                        range,
                        levels: Vec::new(),
                    };
                    arts.push((art, raw, t.line));
                }
            }
            "element" => {
                let (id, name, glyph) = (f.uint("id"), f.name(), f.string("glyph"));
                let (color, effect) = (f.rgb("color"), f.word("effect", &EFFECTS));
                let numbers: Option<Vec<String>> = ELEMENT_KEYS.iter().map(|key| f.number_or_zero(key)).collect();
                let count = f.uint_or_zero("count");
                if let (Some(id), Some(name), Some(glyph), Some(color), Some(effect), Some(numbers), Some(count)) =
                    (id, name, glyph, color, effect, numbers, count)
                {
                    if glyph.chars().count() != 1 {
                        f.errs.push(format!("line {}: element `glyph` must be one character", t.line));
                    }
                    if id == 0 {
                        f.errs.push(format!("line {}: element id 0 is reserved for no element", t.line));
                    }
                    elements.push((Element { id, name, glyph, color, effect, numbers, count }, t.line));
                }
            }
            "reaction" => {
                let (id, name) = (f.uint("id"), f.name());
                let (a, b) = (f.string("a"), f.string("b"));
                let kind = f.word("kind", &REACTION_KINDS);
                let (power, reach) = (f.number("power"), f.number_or_zero("reach"));
                let (count, color) = (f.uint_or_zero("count"), f.rgb("color"));
                if let (
                    Some(id),
                    Some(name),
                    Some(a),
                    Some(b),
                    Some(kind),
                    Some(power),
                    Some(reach),
                    Some(count),
                    Some(color),
                ) = (id, name, a, b, kind, power, reach, count, color)
                {
                    reactions.push((Reaction { id, name, a, b, kind, power, reach, count, color }, t.line));
                }
            }
            "event" => {
                if let (Some(id), Some(name)) = (f.uint("id"), f.name()) {
                    if id == 0 {
                        f.errs.push(format!("line {}: event id 0 is reserved for no event", t.line));
                    }
                    events.push((Event { id, name }, t.line));
                }
            }
            "mission" => {
                if let (Some(id), Some(name), Some(goal)) = (f.uint("id"), f.name(), f.uint("goal")) {
                    missions.push((Mission { id, name, goal }, t.line));
                }
            }
            "tune" => {
                let (id, name, mission) = (f.uint("id"), f.name(), f.string("mission"));
                let (default, min) = (f.number("default"), f.number("min"));
                let (max, step) = (f.number("max"), f.number("step"));
                if let (Some(id), Some(name), Some(mission), Some(default), Some(min), Some(max), Some(step)) =
                    (id, name, mission, default, min, max, step)
                {
                    tunes.push((Tune { id, name, mission, default, min, max, step }, t.line));
                }
            }
            "state" => {
                let len = f.get("len").map(|e| e.value.clone());
                if let (Some(name), Some(index)) = (f.name(), f.uint("index")) {
                    raw_state.push((name, index, len, t.line));
                }
            }
            other => {
                f.errs.push(format!("line {}: unknown table [[{other}]]", t.line));
                f.used.extend(t.entries.iter().map(|e| e.key.as_str()));
            }
        }
        f.finish();
    }

    // Unique ids and names within each table.
    fn unique<'a>(errs: &mut Vec<String>, what: &str, items: impl Iterator<Item = (u32, &'a str, usize)> + Clone) {
        let mut ids: BTreeMap<u32, usize> = BTreeMap::new();
        let mut names: BTreeMap<&str, usize> = BTreeMap::new();
        for (id, name, line) in items {
            if let Some(first) = ids.insert(id, line) {
                errs.push(format!("line {line}: duplicate {what} id {id} (first at line {first})"));
            }
            if let Some(first) = names.insert(name, line) {
                errs.push(format!("line {line}: duplicate {what} name `{name}` (first at line {first})"));
            }
        }
    }
    unique(&mut errs, "part", parts.iter().map(|(p, l)| (p.id, p.name.as_str(), *l)));
    unique(&mut errs, "upgrade", upgrades.iter().map(|(u, l)| (u.id, u.name.as_str(), *l)));
    unique(&mut errs, "event", events.iter().map(|(e, l)| (e.id, e.name.as_str(), *l)));
    unique(&mut errs, "mission", missions.iter().map(|(m, l)| (m.id, m.name.as_str(), *l)));
    let mut state_names: BTreeMap<&str, usize> = BTreeMap::new();
    for (name, _, _, line) in &raw_state {
        if let Some(first) = state_names.insert(name, *line) {
            errs.push(format!("line {line}: duplicate state name `{name}` (first at line {first})"));
        }
    }

    // Upgrade ids index tables, so they must be 0..n without gaps.
    let mut up_ids: Vec<u32> = upgrades.iter().map(|(u, _)| u.id).collect();
    up_ids.sort_unstable();
    up_ids.dedup();
    if up_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("upgrade ids must run 0 to {} without gaps", upgrades.len().saturating_sub(1)));
    }

    // Mission ids index tables too.
    let mut mission_ids: Vec<u32> = missions.iter().map(|(m, _)| m.id).collect();
    mission_ids.sort_unstable();
    mission_ids.dedup();
    if mission_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("mission ids must run 0 to {} without gaps", missions.len().saturating_sub(1)));
    }
    if missions.is_empty() {
        errs.push("at least one [[mission]] is needed".to_string());
    }

    // Tunes: ids index tables, each belongs to a mission, and the default lies in its range.
    unique(&mut errs, "tune", tunes.iter().map(|(t, l)| (t.id, t.name.as_str(), *l)));
    let mut tune_ids: Vec<u32> = tunes.iter().map(|(t, _)| t.id).collect();
    tune_ids.sort_unstable();
    tune_ids.dedup();
    if tune_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("tune ids must run 0 to {} without gaps", tunes.len().saturating_sub(1)));
    }
    for (t, line) in &tunes {
        let owner = missions.iter().find(|(m, _)| m.name == t.mission);
        if t.mission != "any" && owner.is_none() {
            errs.push(format!("line {line}: tune `{}` is for mission `{}`, which is not defined", t.name, t.mission));
        }
        let v = |s: &String| s.parse::<f64>().unwrap_or(f64::NAN);
        let (default, min, max, step) = (v(&t.default), v(&t.min), v(&t.max), v(&t.step));
        if default < min || default > max {
            errs.push(format!("line {line}: tune `{}` default must lie between its min and max", t.name));
        }
        if step <= 0.0 {
            errs.push(format!("line {line}: tune `{}` step must be above 0", t.name));
        }
        if let Some((m, _)) = owner {
            if t.name == format!("{}_goal", m.name) && default != m.goal as f64 {
                errs.push(format!(
                    "line {line}: tune `{}` default must equal the goal of mission `{}` ({})",
                    t.name, m.name, m.goal
                ));
            }
        }
    }

    // Enemies: ids index a table, each draws a defined part, and a share kept in a tune names a defined tune.
    unique(&mut errs, "enemy", enemies.iter().map(|(e, l)| (e.id, e.name.as_str(), *l)));
    let mut enemy_ids: Vec<u32> = enemies.iter().map(|(e, _)| e.id).collect();
    enemy_ids.sort_unstable();
    enemy_ids.dedup();
    if enemy_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("enemy ids must run 0 to {} without gaps", enemies.len().saturating_sub(1)));
    }
    if enemies.is_empty() {
        errs.push("at least one [[enemy]] is needed".to_string());
    }
    for (e, line) in &enemies {
        if !parts.iter().any(|(p, _)| p.name == e.part) {
            errs.push(format!("line {line}: enemy `{}` draws part `{}`, which is not defined", e.name, e.part));
        }
        if let Some(tune) = &e.share_tune {
            if !tunes.iter().any(|(t, _)| &t.name == tune) {
                errs.push(format!(
                    "line {line}: enemy `{}` takes its share from tune `{tune}`, which is not defined",
                    e.name
                ));
            }
        }
        if e.id == 0 && (e.spawn.iter().any(|v| v.parse::<f64>() != Ok(0.0)) || e.share_tune.is_some()) {
            errs.push(format!(
                "line {line}: enemy `{}` has id 0, so it fills whatever the others leave of the horde and takes no spawn keys",
                e.name
            ));
        }
    }

    // Martial arts: ids index a table, each levels with the upgrade of the same
    // name, and every per-level list has one value per level of that upgrade.
    unique(&mut errs, "art", arts.iter().map(|(a, _, l)| (a.id, a.name.as_str(), *l)));
    let mut art_ids: Vec<u32> = arts.iter().map(|(a, _, _)| a.id).collect();
    art_ids.sort_unstable();
    art_ids.dedup();
    if art_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("art ids must run 0 to {} without gaps", arts.len().saturating_sub(1)));
    }
    for (art, raw, line) in arts.iter_mut() {
        if !parts.iter().any(|(p, _)| p.name == art.part) {
            errs.push(format!("line {line}: art `{}` draws part `{}`, which is not defined", art.name, art.part));
        }
        let Some((upgrade, _)) = upgrades.iter().find(|(u, _)| u.name == art.name) else {
            errs.push(format!("line {line}: art `{}` needs an [[upgrade]] of the same name to level it", art.name));
            continue;
        };
        let max = upgrade.max as usize;
        if art.start_level as usize > max {
            errs.push(format!("line {line}: art `{}` starts above its top level {max}", art.name));
        }
        for (key, (values, at)) in ART_LEVEL_KEYS.iter().zip(raw.iter()) {
            if values.len() != 1 && values.len() != max {
                errs.push(format!(
                    "line {at}: art `{}` has {} values for `{key}`, but {max} levels (give one value or one per level)",
                    art.name,
                    values.len()
                ));
            }
            if ART_WHOLE_KEYS.contains(key) && values.iter().any(|v| v.parse::<u32>().is_err()) {
                errs.push(format!("line {at}: art `{}` `{key}` must be whole numbers of 0 or more", art.name));
            }
            let value = |level: usize| {
                if values.len() == 1 {
                    values[0].clone()
                } else {
                    values[level.min(values.len() - 1)].clone()
                }
            };
            art.levels.push((0..max).map(value).collect());
        }
    }
    if let Some(most) = max_arts {
        let held = arts.iter().filter(|(a, _, _)| a.start_level > 0).count();
        if held > most as usize {
            errs.push(format!("{held} arts are held at the start, but `max_arts` is {most}"));
        }
    }

    // Elements: ids run from 1 (0 means no element).
    unique(&mut errs, "element", elements.iter().map(|(e, l)| (e.id, e.name.as_str(), *l)));
    let mut element_ids: Vec<u32> = elements.iter().map(|(e, _)| e.id).collect();
    element_ids.sort_unstable();
    element_ids.dedup();
    if element_ids.iter().enumerate().any(|(i, id)| i as u32 + 1 != *id) {
        errs.push(format!("element ids must run 1 to {} without gaps", elements.len()));
    }

    // Reactions: each joins two different elements that leave a tag, and no pair has two.
    unique(&mut errs, "reaction", reactions.iter().map(|(r, l)| (r.id, r.name.as_str(), *l)));
    let mut reaction_ids: Vec<u32> = reactions.iter().map(|(r, _)| r.id).collect();
    reaction_ids.sort_unstable();
    reaction_ids.dedup();
    if reaction_ids.iter().enumerate().any(|(i, id)| i as u32 != *id) {
        errs.push(format!("reaction ids must run 0 to {} without gaps", reactions.len().saturating_sub(1)));
    }
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for (r, line) in &reactions {
        let mut surge = false;
        for side in [&r.a, &r.b] {
            match elements.iter().find(|(e, _)| &e.name == side) {
                None => errs
                    .push(format!("line {line}: reaction `{}` names element `{side}`, which is not defined", r.name)),
                Some((e, _)) if !e.tags() => errs.push(format!(
                    "line {line}: reaction `{}` names element `{side}`, which leaves no tag on enemies",
                    r.name
                )),
                Some((e, _)) => surge |= e.effect == "surge",
            }
        }
        if r.a == r.b {
            errs.push(format!("line {line}: reaction `{}` needs two different elements", r.name));
        }
        if r.kind == "relay" && !surge {
            errs.push(format!(
                "line {line}: reaction `{}` is a relay, so one of its elements must have the surge effect",
                r.name
            ));
        }
        let pair = if r.a <= r.b { (r.a.as_str(), r.b.as_str()) } else { (r.b.as_str(), r.a.as_str()) };
        if pairs.contains(&pair) {
            errs.push(format!("line {line}: elements `{}` and `{}` already have a reaction", pair.0, pair.1));
        }
        pairs.push(pair);
    }

    // State fields: resolve `len`, then check bounds and overlap.
    let count = |table: &str| match table {
        "part" => parts.len(),
        "upgrade" => upgrades.len(),
        "enemy" => enemies.len(),
        "art" => arts.len(),
        "element" => elements.len(),
        "reaction" => reactions.len(),
        "event" => events.len(),
        "mission" => missions.len(),
        "tune" => tunes.len(),
        _ => raw_state.len(),
    };
    let mut state = Vec::new();
    for (name, index, len, line) in &raw_state {
        let len = match len {
            None => None,
            Some(Value::Int(n)) if *n >= 1 && *n <= u32::MAX as i64 => Some(*n as u32),
            Some(Value::Str(table)) if TABLES.contains(&table.as_str()) => Some(count(table) as u32),
            Some(Value::Str(table)) => {
                errs.push(format!("line {line}: state `{name}` has len \"{table}\", which is not a defined table"));
                continue;
            }
            Some(v) => {
                errs.push(format!(
                    "line {line}: state `{name}` len must be a number of 1 or more or a table name, not {}",
                    v.kind()
                ));
                continue;
            }
        };
        state.push((StateField { name: name.clone(), index: *index, len }, *line));
    }
    state.sort_by_key(|(s, _)| s.index);
    let mut end = 0u64;
    let mut prev = "";
    for (s, line) in &state {
        let n = s.len.unwrap_or(1) as u64;
        if (s.index as u64) < end {
            errs.push(format!("line {line}: state `{}` at {} overlaps `{prev}`", s.name, s.index));
        }
        end = end.max(s.index as u64 + n);
        prev = &s.name;
        if let Some(total) = state_floats {
            if s.index as u64 + n > total as u64 {
                errs.push(format!("line {line}: state `{}` ends past state_floats ({total})", s.name));
            }
        }
    }

    if !errs.is_empty() {
        return Err(errs);
    }
    let mut parts: Vec<Part> = parts.into_iter().map(|(p, _)| p).collect();
    let mut upgrades: Vec<Upgrade> = upgrades.into_iter().map(|(u, _)| u).collect();
    let mut events: Vec<Event> = events.into_iter().map(|(e, _)| e).collect();
    parts.sort_by_key(|p| p.id);
    upgrades.sort_by_key(|u| u.id);
    events.sort_by_key(|e| e.id);
    let mut missions: Vec<Mission> = missions.into_iter().map(|(m, _)| m).collect();
    missions.sort_by_key(|m| m.id);
    let mut tunes: Vec<Tune> = tunes.into_iter().map(|(t, _)| t).collect();
    tunes.sort_by_key(|t| t.id);
    let mut enemies: Vec<Enemy> = enemies.into_iter().map(|(e, _)| e).collect();
    enemies.sort_by_key(|e| e.id);
    let mut arts: Vec<Art> = arts.into_iter().map(|(a, _, _)| a).collect();
    arts.sort_by_key(|a| a.id);
    let mut elements: Vec<Element> = elements.into_iter().map(|(e, _)| e).collect();
    elements.sort_by_key(|e| e.id);
    let mut reactions: Vec<Reaction> = reactions.into_iter().map(|(r, _)| r).collect();
    reactions.sort_by_key(|r| r.id);
    Ok(Content {
        ink: ink.unwrap(),
        state_floats: state_floats.unwrap(),
        max_arts: max_arts.unwrap(),
        parts,
        upgrades,
        enemies,
        arts,
        elements,
        reactions,
        events,
        missions,
        tunes,
        state: state.into_iter().map(|(s, _)| s).collect(),
    })
}

// ---------------------------------------------------------------- rendering

fn upper(name: &str) -> String {
    name.to_ascii_uppercase()
}

/// The constant suffix for a state field: runs of floats get a 0 suffix.
fn state_key(s: &StateField) -> String {
    if s.len.is_some() {
        format!("{}0", upper(&s.name))
    } else {
        upper(&s.name)
    }
}

/// A decimal literal for Rust and WGSL: `1` becomes `1.0`.
fn float_lit(s: &str) -> String {
    if s.contains(['.', 'e', 'E']) {
        s.to_string()
    } else {
        format!("{s}.0")
    }
}

/// A table of names, for the native tools to look ids up by name.
fn name_table<'a>(o: &mut String, table: &str, len: &str, names: impl Iterator<Item = &'a str>) {
    let list: Vec<String> = names.map(|n| format!("\"{n}\"")).collect();
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const {table}: [&str; {len}] = [{}];", list.join(", ")).unwrap();
}

/// `core/src/generated.rs`.
pub fn render_rust(c: &Content) -> String {
    let mut o = String::new();
    o.push_str("//! Generated from `content/game.toml` by `tools/gen`. Do not edit: change the\n");
    o.push_str("//! content file and run `./build.sh`. Re-exported by `crate::defs`.\n\n");
    o.push_str("use crate::schema::*;\n\n");

    o.push_str("// Part ids.\n");
    for p in &c.parts {
        writeln!(o, "pub const P_{}: f32 = {}.0;", upper(&p.name), p.id).unwrap();
    }

    o.push_str("\n// Upgrade ids.\n");
    for u in &c.upgrades {
        writeln!(o, "pub const U_{}: usize = {};", upper(&u.name), u.id).unwrap();
    }
    writeln!(o, "pub const N_UPGRADES: usize = {};", c.upgrades.len()).unwrap();
    name_table(&mut o, "UPGRADE_NAMES", "N_UPGRADES", c.upgrades.iter().map(|u| u.name.as_str()));
    let maxes: Vec<String> = c.upgrades.iter().map(|u| u.max.to_string()).collect();
    writeln!(o, "pub const UPGRADE_MAX: [u8; N_UPGRADES] = [{}];", maxes.join(", ")).unwrap();

    o.push_str("\n// Enemy kinds. Kind 0 fills whatever share of the horde the others leave.\n");
    for e in &c.enemies {
        writeln!(o, "pub const K_{}: u8 = {};", upper(&e.name), e.id).unwrap();
    }
    writeln!(o, "pub const N_KINDS: usize = {};", c.enemies.len()).unwrap();
    name_table(&mut o, "KIND_NAMES", "N_KINDS", c.enemies.iter().map(|e| e.name.as_str()));
    o.push_str("#[rustfmt::skip]\n");
    o.push_str("pub const KINDS: [Kind; N_KINDS] = [\n");
    for e in &c.enemies {
        let n = |key: &str| float_lit(&e.numbers[ENEMY_KEYS.iter().position(|k| *k == key).unwrap()]);
        let s = |key: &str| float_lit(&e.spawn[ENEMY_SPAWN_KEYS.iter().position(|k| *k == key).unwrap()]);
        let color: Vec<String> = e.color.iter().map(|v| float_lit(v)).collect();
        let tune = e.share_tune.as_ref().map_or("N_TUNES".to_string(), |t| format!("T_{}", upper(t)));
        writeln!(
            o,
            "    Kind {{ hp: {}, speed: {}, r: {}, dmg: {}, xp: {}, part: P_{}, color: [{}], hw: {}, hh: {}, rate: {}, corpse: {}, heavy: {}, hover: {}, from: {}, share: {}, ramp: {}, every: {}, share_tune: {} }},",
            n("hp"), n("speed"), n("radius"), n("damage"), n("xp"), upper(&e.part), color.join(", "),
            n("half_width"), n("half_height"), n("pose_rate"), e.corpse, e.heavy, e.hover,
            s("from"), s("share"), s("ramp"), s("every"), tune
        )
        .unwrap();
    }
    o.push_str("];\n");

    o.push_str("\n// Martial arts: one behaviour each, with its numbers per level of the upgrade that levels it.\n");
    for a in &c.arts {
        writeln!(o, "pub const A_{}: usize = {};", upper(&a.name), a.id).unwrap();
    }
    writeln!(o, "pub const N_ARTS: usize = {};", c.arts.len()).unwrap();
    name_table(&mut o, "ART_NAMES", "N_ARTS", c.arts.iter().map(|a| a.name.as_str()));
    writeln!(o, "/// How many martial arts the player can hold at once.").unwrap();
    writeln!(o, "pub const MAX_ARTS: usize = {};", c.max_arts).unwrap();
    o.push_str("#[rustfmt::skip]\n");
    o.push_str("pub const ARTS: [ArtDef; N_ARTS] = [\n");
    for a in &c.arts {
        writeln!(
            o,
            "    ArtDef {{ upgrade: U_{}, behaviour: B_{}, part: P_{}, start_level: {}, first_delay: {}, speed: {}, life: {}, range: {}, levels: &[",
            upper(&a.name),
            upper(&a.behaviour),
            upper(&a.part),
            a.start_level,
            float_lit(&a.first_delay),
            float_lit(&a.speed),
            float_lit(&a.life),
            float_lit(&a.range)
        )
        .unwrap();
        let levels = a.levels.first().map_or(0, |l| l.len());
        for level in 0..levels {
            let fields: Vec<String> = ART_LEVEL_KEYS
                .iter()
                .zip(&a.levels)
                .map(|(key, values)| {
                    let v = &values[level];
                    if ART_WHOLE_KEYS.contains(key) {
                        format!("{key}: {v}")
                    } else {
                        format!("{key}: {}", float_lit(v))
                    }
                })
                .collect();
            writeln!(o, "        ArtLevel {{ {} }},", fields.join(", ")).unwrap();
        }
        o.push_str("    ] },\n");
    }
    o.push_str("];\n");

    o.push_str("\n// Magic elements. Id 0 means no element; `ELEMENTS[0]` is that empty entry.\n");
    for e in &c.elements {
        writeln!(o, "pub const E_{}: u8 = {};", upper(&e.name), e.id).unwrap();
    }
    writeln!(o, "pub const N_ELEMENTS: usize = {};", c.elements.len()).unwrap();
    // Entry 0 is "no element".
    name_table(
        &mut o,
        "ELEMENT_NAMES",
        "N_ELEMENTS + 1",
        std::iter::once("none").chain(c.elements.iter().map(|e| e.name.as_str())),
    );
    let rgb = |c: &[String; 3]| c.iter().map(|v| float_lit(v)).collect::<Vec<String>>().join(", ");
    o.push_str("#[rustfmt::skip]\n");
    o.push_str("pub const ELEMENTS: [ElementDef; N_ELEMENTS + 1] = [\n");
    o.push_str("    ElementDef::NONE,\n");
    for e in &c.elements {
        let numbers: Vec<String> =
            ELEMENT_KEYS.iter().zip(&e.numbers).map(|(key, v)| format!("{key}: {}", float_lit(v))).collect();
        writeln!(
            o,
            "    ElementDef {{ effect: EF_{}, tags: {}, color: [{}], {}, count: {} }},",
            upper(&e.effect),
            e.tags(),
            rgb(&e.color),
            numbers.join(", "),
            e.count
        )
        .unwrap();
    }
    o.push_str("];\n");

    o.push_str("\n// Reactions between two element tags on one enemy, tried in this order.\n");
    for r in &c.reactions {
        writeln!(o, "pub const R_{}: usize = {};", upper(&r.name), r.id).unwrap();
    }
    writeln!(o, "pub const N_REACTIONS: usize = {};", c.reactions.len()).unwrap();
    o.push_str("#[rustfmt::skip]\n");
    o.push_str("pub const REACTIONS: [ReactionDef; N_REACTIONS] = [\n");
    for r in &c.reactions {
        writeln!(
            o,
            "    ReactionDef {{ a: E_{}, b: E_{}, kind: RK_{}, power: {}, reach: {}, count: {}, color: [{}] }},",
            upper(&r.a),
            upper(&r.b),
            upper(&r.kind),
            float_lit(&r.power),
            float_lit(&r.reach),
            r.count,
            rgb(&r.color)
        )
        .unwrap();
    }
    o.push_str("];\n");

    o.push_str("\n// Event types, read by the host for sound.\n");
    for e in &c.events {
        writeln!(o, "pub const EV_{}: f32 = {}.0;", upper(&e.name), e.id).unwrap();
    }

    o.push_str("\n// Mission ids, and what each one asks for (seconds for `survive`, enemies for `kills`).\n");
    for m in &c.missions {
        writeln!(o, "pub const M_{}: usize = {};", upper(&m.name), m.id).unwrap();
    }
    writeln!(o, "pub const N_MISSIONS: usize = {};", c.missions.len()).unwrap();
    name_table(&mut o, "MISSION_NAMES", "N_MISSIONS", c.missions.iter().map(|m| m.name.as_str()));
    let goals: Vec<String> = c.missions.iter().map(|m| format!("{}.0", m.goal)).collect();
    // One line however many missions there are, so rustfmt must leave it alone.
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const MISSION_GOAL: [f32; N_MISSIONS] = [{}];", goals.join(", ")).unwrap();

    o.push_str("\n// Tunes: numbers the setup screen can change for a test run, with their range.\n");
    for t in &c.tunes {
        writeln!(o, "pub const T_{}: usize = {};", upper(&t.name), t.id).unwrap();
    }
    writeln!(o, "pub const N_TUNES: usize = {};", c.tunes.len()).unwrap();
    name_table(&mut o, "TUNE_NAMES", "N_TUNES", c.tunes.iter().map(|t| t.name.as_str()));
    let column = |pick: fn(&Tune) -> &String| -> String {
        c.tunes.iter().map(|t| float_lit(pick(t))).collect::<Vec<String>>().join(", ")
    };
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const TUNE_DEFAULT: [f32; N_TUNES] = [{}];", column(|t| &t.default)).unwrap();
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const TUNE_MIN: [f32; N_TUNES] = [{}];", column(|t| &t.min)).unwrap();
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const TUNE_MAX: [f32; N_TUNES] = [{}];", column(|t| &t.max)).unwrap();
    o.push_str("// The tune that holds each mission's goal; N_TUNES where the goal cannot be changed.\n");
    let goal_tunes: Vec<String> = c
        .missions
        .iter()
        .map(|m| {
            let name = format!("{}_goal", m.name);
            c.tunes.iter().find(|t| t.name == name).map_or("N_TUNES".to_string(), |t| format!("T_{}", upper(&t.name)))
        })
        .collect();
    o.push_str("#[rustfmt::skip]\n");
    writeln!(o, "pub const MISSION_GOAL_TUNE: [usize; N_MISSIONS] = [{}];", goal_tunes.join(", ")).unwrap();

    o.push_str("\n// HUD state buffer: size and the index of each field.\n");
    writeln!(o, "pub const STATE_FLOATS: usize = {};", c.state_floats).unwrap();
    for s in &c.state {
        writeln!(o, "pub const S_{}: usize = {};", state_key(s), s.index).unwrap();
    }

    o.push('\n');
    let ink: Vec<String> = c.ink.iter().map(|v| float_lit(v)).collect();
    writeln!(o, "pub const INK: [f32; 3] = [{}];", ink.join(", ")).unwrap();
    o
}

fn js_str(s: &str) -> String {
    format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// `src/generated.js`.
pub fn render_js(c: &Content) -> String {
    let mut o = String::new();
    o.push_str("// Generated from content/game.toml by tools/gen. Do not edit: change the\n");
    o.push_str("// content file and run ./build.sh. Imported through src/data.js.\n\n");

    o.push_str("/** Part ids by name. A name is also the key for an image override in assets/parts.json. */\n");
    o.push_str("export const PART_IDS = {\n");
    for p in &c.parts {
        writeln!(o, "  {}: {},", p.name, p.id).unwrap();
    }
    o.push_str("};\n\n");

    o.push_str("/** Upgrades, indexed by id. `glyph` is the brush mark shown on cards and HUD chips.\n");
    o.push_str(" * Names and descriptions are in src/i18n.js as upgrade.<id>.name and .desc. */\n");
    o.push_str("export const UPGRADES = [\n");
    for u in &c.upgrades {
        writeln!(o, "  {{ name: {}, glyph: {}, max: {} }},", js_str(&u.name), js_str(&u.glyph), u.max).unwrap();
    }
    o.push_str("];\n\n");

    o.push_str("/** Martial arts, indexed by id. `upgrade` is the id of the upgrade that levels the art, and\n");
    o.push_str(" * whose name, glyph and card text it shows. */\n");
    o.push_str("export const ARTS = [\n");
    for a in &c.arts {
        let upgrade = c.upgrades.iter().find(|u| u.name == a.name).map_or(0, |u| u.id);
        writeln!(
            o,
            "  {{ id: {}, name: {}, upgrade: {}, behaviour: {} }},",
            a.id,
            js_str(&a.name),
            upgrade,
            js_str(&a.behaviour)
        )
        .unwrap();
    }
    o.push_str("];\n\n");
    o.push_str("/** How many martial arts the player can hold at once. */\n");
    writeln!(o, "export const MAX_ARTS = {};\n", c.max_arts).unwrap();

    o.push_str("/** Magic elements. Ids start at 1 (0 means no element), so element `id` is ELEMENTS[id - 1].\n");
    o.push_str(
        " * `color` is linear-ish RGB, 0..1. `tags` says whether a hit leaves the element's tag on the enemy.\n",
    );
    o.push_str(" * Names and descriptions are in src/i18n.js as element.<name>.name and .desc. */\n");
    o.push_str("export const ELEMENTS = [\n");
    for e in &c.elements {
        writeln!(
            o,
            "  {{ id: {}, name: {}, glyph: {}, color: [{}], tags: {} }},",
            e.id,
            js_str(&e.name),
            js_str(&e.glyph),
            e.color.join(", "),
            e.tags()
        )
        .unwrap();
    }
    o.push_str("];\n\n");

    o.push_str("/** Reactions between the tags of elements `a` and `b` on one enemy. */\n");
    o.push_str("export const REACTIONS = [\n");
    for r in &c.reactions {
        writeln!(o, "  {{ id: {}, name: {}, a: {}, b: {} }},", r.id, js_str(&r.name), js_str(&r.a), js_str(&r.b))
            .unwrap();
    }
    o.push_str("];\n\n");

    o.push_str("/** Event types reported by the core. */\n");
    o.push_str("export const EVENT = {\n");
    for e in &c.events {
        writeln!(o, "  {}: {},", upper(&e.name), e.id).unwrap();
    }
    o.push_str("};\n\n");

    o.push_str(
        "/** Missions, indexed by id. `goal` is what the mission asks for (seconds for survive, enemies for kills).\n",
    );
    o.push_str(" * Names and descriptions are in src/i18n.js as mission.<name>.name and .desc. */\n");
    o.push_str("export const MISSIONS = [\n");
    for m in &c.missions {
        writeln!(o, "  {{ id: {}, name: {}, goal: {} }},", m.id, js_str(&m.name), m.goal).unwrap();
    }
    o.push_str("];\n\n");

    o.push_str("/** Mission ids by name. */\n");
    o.push_str("export const MISSION_IDS = {\n");
    for m in &c.missions {
        writeln!(o, "  {}: {},", m.name, m.id).unwrap();
    }
    o.push_str("};\n\n");

    o.push_str("/** Tunes, indexed by id: numbers the setup screen can change for a test run. `mission` is the\n");
    o.push_str(" * mission that shows the tune, or 'any'. Labels are in src/i18n.js as tune.<name>. */\n");
    o.push_str("export const TUNES = [\n");
    for t in &c.tunes {
        writeln!(
            o,
            "  {{ id: {}, name: {}, mission: {}, def: {}, min: {}, max: {}, step: {} }},",
            t.id,
            js_str(&t.name),
            js_str(&t.mission),
            t.default,
            t.min,
            t.max,
            t.step
        )
        .unwrap();
    }
    o.push_str("];\n\n");

    o.push_str("/** Size of the state array written by the core, in floats. */\n");
    writeln!(o, "export const STATE_FLOATS = {};", c.state_floats).unwrap();
    o.push('\n');
    o.push_str("/** Indices into the state array. A run of floats has a 0 suffix (CHOICE0). */\n");
    o.push_str("export const STATE = {\n");
    for s in &c.state {
        writeln!(o, "  {}: {},", state_key(s), s.index).unwrap();
    }
    o.push_str("};\n\n");

    writeln!(o, "export const INK = [{}];", c.ink.join(", ")).unwrap();
    o
}

/// The lines between the markers in `src/shader.wgsl`, markers included.
pub fn render_wgsl_block(c: &Content) -> String {
    let mut o = String::new();
    o.push_str(WGSL_BEGIN);
    o.push('\n');
    let ink: Vec<String> = c.ink.iter().map(|v| float_lit(v)).collect();
    writeln!(o, "const INK = vec3f({});", ink.join(", ")).unwrap();
    for p in &c.parts {
        writeln!(o, "const PART_{}: i32 = {};", upper(&p.name), p.id).unwrap();
    }
    o.push_str(WGSL_END);
    o.push('\n');
    o
}

/// Replace the generated block in the shader source. Line endings become LF.
pub fn splice_wgsl(shader: &str, c: &Content) -> Result<String, String> {
    let src = shader.replace("\r\n", "\n");
    let begin = src.find(WGSL_BEGIN).ok_or(format!("{WGSL_PATH}: no begin marker `{WGSL_BEGIN}`"))?;
    if src[begin + 1..].contains(WGSL_BEGIN) {
        return Err(format!("{WGSL_PATH}: more than one begin marker"));
    }
    let end_rel = src[begin..].find(WGSL_END).ok_or(format!("{WGSL_PATH}: no end marker `{WGSL_END}`"))?;
    let mut end = begin + end_rel + WGSL_END.len();
    if src[end..].starts_with('\n') {
        end += 1;
    }
    if !src[..begin].is_empty() && !src[..begin].ends_with('\n') {
        return Err(format!("{WGSL_PATH}: the begin marker must start a line"));
    }
    Ok(format!("{}{}{}", &src[..begin], render_wgsl_block(c), &src[end..]))
}
