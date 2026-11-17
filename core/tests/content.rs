//! Checks for `content/game.toml` and the code generated from it.
//!
//! The generator lives in `tools/gen` (a separate crate, so it builds even
//! when the core does not). Its pure functions are included here by path.

#[allow(dead_code)]
#[path = "../../tools/gen/src/content.rs"]
mod content;

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A repository file, with line endings normalised to LF.
fn read(rel: &str) -> String {
    let path = root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())).replace("\r\n", "\n")
}

fn real() -> content::Content {
    match content::load(&read(content::CONTENT_PATH)) {
        Ok(c) => c,
        Err(errs) => panic!("{} has problems:\n  {}", content::CONTENT_PATH, errs.join("\n  ")),
    }
}

/// Errors from loading the real content file with one edit applied.
fn errors_after(from: &str, to: &str) -> Vec<String> {
    let src = read(content::CONTENT_PATH);
    assert!(src.contains(from), "test edit `{from}` not found in the content file");
    match content::load(&src.replacen(from, to, 1)) {
        Ok(_) => panic!("content with `{from}` -> `{to}` was accepted"),
        Err(errs) => errs,
    }
}

fn assert_error(errs: &[String], needle: &str) {
    assert!(errs.iter().any(|e| e.contains(needle)), "expected an error containing `{needle}`, got {errs:#?}");
}

const STALE: &str = "is out of date; run ./build.sh (or cargo run --release --manifest-path tools/gen/Cargo.toml)";

#[test]
fn content_file_is_valid() {
    let c = real();
    assert!(!c.parts.is_empty() && !c.upgrades.is_empty() && !c.events.is_empty() && !c.state.is_empty());
}

#[test]
fn generated_rust_is_current() {
    assert!(read(content::RUST_OUT) == content::render_rust(&real()), "{} {STALE}", content::RUST_OUT);
}

#[test]
fn generated_js_is_current() {
    assert!(read(content::JS_OUT) == content::render_js(&real()), "{} {STALE}", content::JS_OUT);
}

#[test]
fn generated_shader_block_is_current() {
    let shader = read(content::WGSL_PATH);
    let fresh = content::splice_wgsl(&shader, &real()).unwrap();
    assert!(shader == fresh, "the generated block in {} {STALE}", content::WGSL_PATH);
}

#[test]
fn rendering_is_deterministic() {
    let c = real();
    assert_eq!(content::render_rust(&c), content::render_rust(&real()));
    assert_eq!(content::render_js(&c), content::render_js(&real()));
    for text in [content::render_rust(&c), content::render_js(&c), content::render_wgsl_block(&c)] {
        assert!(!text.contains('\r'));
        assert!(text.ends_with('\n'));
    }
    // Splicing twice changes nothing.
    let once = content::splice_wgsl(&read(content::WGSL_PATH), &c).unwrap();
    assert_eq!(content::splice_wgsl(&once, &c).unwrap(), once);
}

// ------------------------------------------------------- broken content

#[test]
fn duplicate_part_id_is_rejected() {
    let errs = errors_after("id = 18\nname = \"ground_shadow\"", "id = 16\nname = \"ground_shadow\"");
    assert_error(&errs, "duplicate part id 16");
}

#[test]
fn duplicate_names_are_rejected() {
    assert_error(&errors_after("name = \"wisp\"", "name = \"blot\""), "duplicate part name `blot`");
    assert_error(&errors_after("name = \"mending\"", "name = \"haste\""), "duplicate upgrade name `haste`");
    assert_error(&errors_after("name = \"kills\"\nindex", "name = \"hp\"\nindex"), "duplicate state name `hp`");
}

#[test]
fn duplicate_event_id_is_rejected() {
    assert_error(&errors_after("id = 11\nname = \"elite\"", "id = 10\nname = \"elite\""), "duplicate event id 10");
}

#[test]
fn upgrade_ids_must_have_no_gaps() {
    assert_error(&errors_after("id = 9\nname = \"mending\"", "id = 12\nname = \"mending\""), "without gaps");
}

#[test]
fn undefined_reference_is_rejected() {
    assert_error(&errors_after("len = \"upgrade\"", "len = \"upgrades\""), "not a defined table");
}

#[test]
fn overlapping_state_is_rejected() {
    assert_error(&errors_after("name = \"over\"\nindex = 11", "name = \"over\"\nindex = 10"), "overlaps `choice`");
    assert_error(&errors_after("state_floats = 128", "state_floats = 20"), "ends past state_floats");
}

#[test]
fn routes_and_moves_are_checked() {
    let c = real();
    assert!(!c.routes.is_empty() && !c.moves.is_empty() && !c.schools.is_empty());
    assert_error(&errors_after("art = \"scythe\"\nschool", "art = \"axe\"\nschool"), "art `axe`, which is not defined");
    assert_error(
        &errors_after("school = \"duanyue\"\nbeat", "school = \"nowhere\"\nbeat"),
        "school `nowhere`, which is not defined",
    );
    assert_error(
        &errors_after("route = \"bengyue\"\nshape = \"arc\"", "route = \"lost\"\nshape = \"arc\""),
        "route `lost`",
    );
    assert_error(&errors_after("shape = \"wave\"", "shape = \"spin\""), "`shape` is \"spin\"");
    assert_error(
        &errors_after(
            "name = \"hengyun\"\nroute = \"bengyue\"\nshape = \"arc\"\n",
            "name = \"hengyun\"\nroute = \"bengyue\"\nshape = \"arc\"\nultimate = true\n",
        ),
        "only have its last move as its ultimate",
    );
}

#[test]
fn weapon_kinds_are_checked() {
    let c = real();
    assert!(!c.weapons.is_empty());
    assert_error(&errors_after("suits = \"dadao\"", "suits = \"spear\""), "suits weapon `spear`, which is not defined");
    assert_error(&errors_after("suits = \"dadao\"", "suits = \"biao\""), "which is a kind of `talisman`");
    assert_error(&errors_after("name = \"dandao\"\nart = \"scythe\"", "name = \"dandao\"\nart = \"axe\""), "art `axe`");
    assert_error(
        &errors_after("glyph = \"雙\"\nshort = true", "glyph = \"雙\"\nstart = true\nshort = true"),
        "exactly one weapon kind",
    );
}

#[test]
fn every_route_move_and_school_has_text() {
    let c = real();
    let i18n = read("src/i18n.js");
    for r in &c.routes {
        for key in [format!("'route.{}.name'", r.name), format!("'route.{}.desc'", r.name)] {
            assert_eq!(i18n.matches(&key).count(), 2, "{key} should appear once per language in src/i18n.js");
        }
    }
    for m in &c.moves {
        for key in [format!("'move.{}.name'", m.name), format!("'move.{}.desc'", m.name)] {
            assert_eq!(i18n.matches(&key).count(), 2, "{key} should appear once per language in src/i18n.js");
        }
    }
    for s in &c.schools {
        let key = format!("'school.{}'", s.name);
        assert_eq!(i18n.matches(&key).count(), 2, "{key} should appear once per language in src/i18n.js");
    }
    for w in &c.weapons {
        let key = format!("'weapon.{}'", w.name);
        assert_eq!(i18n.matches(&key).count(), 2, "{key} should appear once per language in src/i18n.js");
    }
}

#[test]
fn passives_are_checked() {
    let c = real();
    assert!(!c.passives.is_empty());
    assert_error(&errors_after("stat = \"regen\"", "stat = \"luck\""), "must be one of");
    assert_error(&errors_after("name = \"mending\"\nstat", "name = \"mended\"\nstat"), "needs an [[upgrade]]");
    // Without its passive, the mending upgrade would do nothing.
    assert_error(
        &errors_after("name = \"mending\"\nstat", "name = \"mended\"\nstat"),
        "neither an [[art]] nor a [[passive]]",
    );
    assert_error(&errors_after("name = \"lure\"\nstat", "name = \"scythe\"\nstat"), "both a martial art and a passive");
}

#[test]
fn enemies_are_checked() {
    let c = real();
    assert!(!c.enemies.is_empty());
    assert_error(
        &errors_after("name = \"wisp\"\npart = \"wisp\"", "name = \"wisp\"\npart = \"ghost\""),
        "which is not defined",
    );
    assert_error(&errors_after("id = 3\nname = \"elite\"", "id = 8\nname = \"elite\""), "enemy ids must run");
    assert_error(&errors_after("corpse = true", "corpse = 1"), "true or false");
    assert_error(&errors_after("pose_rate = 9\n", ""), "missing `pose_rate`");
    // Enemy 0 is what the others leave of the horde, so it has no share of its own.
    assert_error(&errors_after("pose_rate = 5\n", "pose_rate = 5\nshare = 0.5\n"), "takes no spawn keys");
    assert_error(&errors_after("from = 25\n", "from = 25\ntune = \"wisps\"\n"), "which is not defined");
}

#[test]
fn arts_are_checked() {
    let c = real();
    assert!(!c.arts.is_empty());
    for art in &c.arts {
        let upgrade = c.upgrades.iter().find(|u| u.name == art.name).expect("an art without its upgrade");
        assert_eq!(art.levels.len(), content::ART_LEVEL_KEYS.len());
        assert!(art.levels.iter().all(|values| values.len() == upgrade.max as usize));
    }
    assert_error(&errors_after("behaviour = \"arc\"", "behaviour = \"lunge\""), "must be one of");
    assert_error(&errors_after("part = \"slash\"", "part = \"sword\""), "which is not defined");
    assert_error(&errors_after("name = \"scythe\"\nbehaviour", "name = \"sabre\"\nbehaviour"), "needs an [[upgrade]]");
    assert_error(
        &errors_after("damage = [20, 26, 32, 38, 44, 50]", "damage = [20, 26, 32]"),
        "has 3 values for `damage`",
    );
    assert_error(&errors_after("count = [1, 1, 1, 1, 2, 2]", "count = [1, 1, 1, 1, 2, 2.5]"), "must be whole numbers");
    assert_error(&errors_after("start_level = 1", "start_level = 9"), "starts above its top level");
    assert_error(
        &errors_after("id = 3\nname = \"pulse\"\nbehaviour", "id = 7\nname = \"pulse\"\nbehaviour"),
        "art ids must run",
    );
    assert_error(&errors_after("max_arts = 6", "max_arts = 0"), "`max_arts` must be 1 or more");
}

#[test]
fn elements_and_reactions_are_checked() {
    let c = real();
    assert_eq!(c.elements.first().map(|e| e.id), Some(1), "element ids start at 1; 0 means none");
    assert_error(&errors_after("effect = \"burn\"", "effect = \"melt\""), "must be one of");
    assert_error(&errors_after("glyph = \"火\"", "glyph = \"火焰\""), "one character");
    assert_error(&errors_after("id = 7\nname = \"holy\"", "id = 9\nname = \"holy\""), "element ids must run 1 to");
    assert_error(&errors_after("color = [0.82, 0.34, 0.18]", "color = [0.82, 0.34, 1.8]"), "from 0 to 1");
    assert_error(
        &errors_after("a = \"fire\"\nb = \"lightning\"", "a = \"fire\"\nb = \"frost\""),
        "which is not defined",
    );
    assert_error(
        &errors_after("a = \"fire\"\nb = \"lightning\"", "a = \"fire\"\nb = \"fire\""),
        "two different elements",
    );
    // Metal acts on the player and leaves no tag, so it cannot react.
    assert_error(&errors_after("a = \"fire\"\nb = \"lightning\"", "a = \"fire\"\nb = \"metal\""), "leaves no tag");
    assert_error(
        &errors_after("a = \"fire\"\nb = \"lightning\"", "a = \"fire\"\nb = \"water\""),
        "already have a reaction",
    );
    assert_error(&errors_after("kind = \"relay\"", "kind = \"echo\""), "must be one of");
    // A relay sets a waiting surge off, so one of its two elements must leave one.
    assert_error(
        &errors_after(
            "a = \"fire\"\nb = \"lightning\"\nkind = \"burst\"",
            "a = \"fire\"\nb = \"lightning\"\nkind = \"relay\"",
        ),
        "must have the surge effect",
    );
}

#[test]
fn every_element_has_card_text() {
    let c = real();
    let i18n = read("src/i18n.js");
    let langs = i18n
        .split("export const LANGS = [")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .map(|s| s.split(',').filter(|x| !x.trim().is_empty()).count())
        .expect("LANGS not found in src/i18n.js");
    for e in &c.elements {
        for field in ["name", "desc"] {
            let key = format!("'element.{}.{field}'", e.name);
            assert_eq!(i18n.matches(&key).count(), langs, "{key} should appear once per language in src/i18n.js");
        }
    }
    for name in refs(&i18n, "'element.") {
        assert!(
            c.elements.iter().any(|e| e.name == name),
            "src/i18n.js has text for element `{name}`, which is not defined"
        );
    }
}

#[test]
fn malformed_entries_are_rejected() {
    assert_error(&errors_after("name = \"wisp\"", "name = \"wisp\"\ncolour = 3"), "unknown key `colour`");
    assert_error(&errors_after("[[event]]", "[[sound]]"), "unknown table [[sound]]");
    assert_error(&errors_after("max = 5\nglyph = \"癒\"", "glyph = \"癒\""), "missing `max`");
    assert_error(&errors_after("id = 18", "id = 40"), "must be below 32");
    assert_error(&errors_after("name = \"wisp\"", "name = \"Wisp\""), "snake_case");
    assert_error(&errors_after("glyph = \"力\"", "glyph = \"力量\""), "one character");
    assert_error(&errors_after("ink = [0.17, 0.15, 0.13]", "ink = [0.17, 0.15]"), "`ink`");
}

// -------------------------------------------- references from other files

/// Identifiers that follow `prefix` where `prefix` is not part of a longer word.
fn refs(text: &str, prefix: &str) -> Vec<String> {
    let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut out = Vec::new();
    for (i, _) in text.match_indices(prefix) {
        if text[..i].chars().next_back().is_some_and(ident) {
            continue;
        }
        let rest = &text[i + prefix.len()..];
        let end = rest.find(|c: char| !ident(c)).unwrap_or(rest.len());
        if end > 0 {
            out.push(rest[..end].to_string());
        }
    }
    out
}

fn js_sources() -> Vec<(String, String)> {
    let mut files: Vec<String> = std::fs::read_dir(root().join("src"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".js") && n != "generated.js")
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|n| {
            let t = read(&format!("src/{n}"));
            (n, t)
        })
        .collect()
}

#[test]
fn js_uses_only_defined_state_and_event_names() {
    let c = real();
    let state: Vec<String> = c
        .state
        .iter()
        .map(|s| {
            let n = s.name.to_ascii_uppercase();
            if s.len.is_some() {
                format!("{n}0")
            } else {
                n
            }
        })
        .collect();
    let events: Vec<String> = c.events.iter().map(|e| e.name.to_ascii_uppercase()).collect();
    let mut seen = 0;
    for (file, text) in js_sources() {
        for name in refs(&text, "STATE.") {
            assert!(state.contains(&name), "src/{file} uses STATE.{name}, which content/game.toml does not define");
            seen += 1;
        }
        for name in refs(&text, "EVENT.") {
            assert!(events.contains(&name), "src/{file} uses EVENT.{name}, which content/game.toml does not define");
            seen += 1;
        }
    }
    assert!(seen > 0, "found no STATE./EVENT. uses; is the scan broken?");
}

#[test]
fn shader_uses_only_defined_parts() {
    let c = real();
    let shader = read(content::WGSL_PATH);
    let begin = shader.find(content::WGSL_BEGIN).unwrap();
    let end = shader.find(content::WGSL_END).unwrap();
    let outside = format!("{}{}", &shader[..begin], &shader[end..]);
    let names: Vec<String> = c.parts.iter().map(|p| p.name.to_ascii_uppercase()).collect();
    let used = refs(&outside, "PART_");
    assert!(!used.is_empty(), "found no PART_ uses in the shader; is the scan broken?");
    for name in used {
        assert!(names.contains(&name), "src/shader.wgsl uses PART_{name}, which content/game.toml does not define");
    }
    // No literal part ids left in the switch.
    for line in outside.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("case ") {
            assert!(!rest.starts_with(|ch: char| ch.is_ascii_digit()), "literal case in shader: {t}");
        }
    }
}

#[test]
fn part_overrides_name_defined_parts() {
    let c = real();
    let json = read("assets/parts.json");
    // Keys are the strings followed by a colon.
    let mut keys = Vec::new();
    let mut parts = json.split('"');
    let mut inside = false;
    let mut last = "";
    for piece in parts.by_ref() {
        if inside {
            last = piece;
        } else if piece.trim_start().starts_with(':') && !last.is_empty() {
            keys.push(last.to_string());
        }
        inside = !inside;
    }
    for key in keys {
        assert!(c.parts.iter().any(|p| p.name == key), "assets/parts.json overrides `{key}`, which is not a part");
    }
}

#[test]
fn tunes_are_checked() {
    assert_error(
        &errors_after("name = \"kills_goal\"\nmission = \"kills\"", "name = \"kills_goal\"\nmission = \"cull\""),
        "not defined",
    );
    assert_error(&errors_after("default = 1500\nmin = 100", "default = 1500\nmin = 2000"), "between its min and max");
    assert_error(&errors_after("default = 1500\nmin = 100", "default = 1400\nmin = 100"), "must equal the goal");
    assert_error(&errors_after("max = 5000\nstep = 100", "max = 5000\nstep = 0"), "step must be above 0");
    assert_error(&errors_after("id = 4\nname = \"kills_goal\"", "id = 90\nname = \"kills_goal\""), "tune ids must run");
}

#[test]
fn every_tune_has_a_label() {
    let c = real();
    let i18n = read("src/i18n.js");
    let langs = i18n
        .split("export const LANGS = [")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .map(|s| s.split(',').filter(|x| !x.trim().is_empty()).count())
        .expect("LANGS not found in src/i18n.js");
    for t in &c.tunes {
        let key = format!("'tune.{}'", t.name);
        assert_eq!(i18n.matches(&key).count(), langs, "{key} should appear once per language in src/i18n.js");
    }
    for name in refs(&i18n, "'tune.") {
        assert!(
            c.tunes.iter().any(|t| t.name == name),
            "src/i18n.js has a label for tune `{name}`, which is not defined"
        );
    }
}

#[test]
fn every_upgrade_has_card_text() {
    let c = real();
    let i18n = read("src/i18n.js");
    let langs = i18n
        .split("export const LANGS = [")
        .nth(1)
        .and_then(|s| s.split(']').next())
        .map(|s| s.split(',').filter(|x| !x.trim().is_empty()).count())
        .expect("LANGS not found in src/i18n.js");
    for u in &c.upgrades {
        for field in ["name", "desc"] {
            let key = format!("'upgrade.{}.{field}'", u.id);
            assert_eq!(i18n.matches(&key).count(), langs, "{key} should appear once per language in src/i18n.js");
        }
    }
    for id in refs(&i18n, "'upgrade.") {
        let n: u32 = id.parse().unwrap_or_else(|_| panic!("odd upgrade key upgrade.{id}"));
        assert!(c.upgrades.iter().any(|u| u.id == n), "src/i18n.js has text for upgrade {n}, which is not defined");
    }
}
