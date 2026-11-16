//! Balance tool: plays many runs natively with simple bots and prints how
//! they went. No drawing and no browser, so thousands of runs take seconds.
//!
//!     cargo run --release --manifest-path core/Cargo.toml --bin sim -- [options]
//!
//!   --mission <name>      the mission to play, or all (default endless)
//!   --seeds <n>           runs per row (default 24)
//!   --seconds <n>         a run stops here if nothing ended it (default 300)
//!   --bot <name>          still, circle, kite, seek, or all (default all)
//!                         seek heads for whatever the mission's edge markers
//!                         point at (its goal) while keeping clear of enemies
//!   --pick <name>         first, random, arts, passives, or all (default all)
//!   --element <name>      none, an element's name, or each (default none):
//!                         every martial art the bot holds carries that
//!                         element, and no element cards are offered
//!   --tune <name>=<v>     a tune, as on the setup screen; may be repeated
//!   --csv                 comma-separated values instead of a table
//!
//! Each row is one bot, one way of picking cards and one element. Columns:
//! how many of the runs were won and how many ended in death, the mean time
//! played, level reached, kills, kills per minute and health lost per minute.
//! `--element each` prints a row for no element and one per element, which is
//! how the numbers in `[[element]]` are compared.

use jadereaper_core::defs::*;
use jadereaper_core::game::{Game, TICK};
use std::thread;

#[derive(Clone, Copy, PartialEq)]
enum Bot {
    Still,
    Circle,
    Kite,
    Seek,
}

#[derive(Clone, Copy, PartialEq)]
enum Pick {
    First,
    Random,
    Arts,
    Passives,
}

const BOTS: [(&str, Bot); 4] =
    [("still", Bot::Still), ("circle", Bot::Circle), ("kite", Bot::Kite), ("seek", Bot::Seek)];
const PICKS: [(&str, Pick); 4] =
    [("first", Pick::First), ("random", Pick::Random), ("arts", Pick::Arts), ("passives", Pick::Passives)];

#[derive(Clone)]
struct Setup {
    mission: usize,
    seconds: f32,
    tune: [f32; N_TUNES],
    bot: Bot,
    pick: Pick,
    /// The element every held art carries; 0 for none.
    element: u8,
}

#[derive(Default, Clone, Copy)]
struct Outcome {
    won: bool,
    died: bool,
    time: f32,
    level: f32,
    kills: f32,
    hurt: f32,
}

/// Away from the enemies close by, each weighing more the nearer it is, and
/// the offset to the nearest enemy of all.
fn avoid(g: &Game) -> ((f32, f32), (f32, f32)) {
    let (px, py) = (g.player.x, g.player.y);
    let (mut ax, mut ay) = (0.0f32, 0.0f32);
    let mut nearest = (f32::MAX, 0.0f32, 0.0f32);
    for e in &g.enemies {
        let (dx, dy) = (px - e.x, py - e.y);
        let d2 = (dx * dx + dy * dy).max(1.0);
        if d2 < nearest.0 {
            nearest = (d2, dx, dy);
        }
        if d2 < 170.0 * 170.0 {
            ax += dx / d2;
            ay += dy / d2;
        }
    }
    ((ax, ay), (nearest.1, nearest.2))
}

fn unit((x, y): (f32, f32)) -> (f32, f32) {
    let len = (x * x + y * y).sqrt();
    if len > 0.0 {
        (x / len, y / len)
    } else {
        (0.0, 0.0)
    }
}

/// Where the bot wants to go, each axis in -1..1. `target` is the last place
/// the mission's markers pointed at, for the seeking bot.
fn steer(bot: Bot, g: &Game, tick: u32, target: Option<(f32, f32)>) -> (f32, f32) {
    match bot {
        Bot::Still => (0.0, 0.0),
        Bot::Circle => {
            // A square walked corner to corner; close enough to a circle and exact on every machine.
            const SIDES: [(f32, f32); 4] = [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)];
            SIDES[(tick / 90 % 4) as usize]
        }
        Bot::Kite => {
            let ((ax, ay), nearest) = avoid(g);
            if ax == 0.0 && ay == 0.0 {
                // Nothing close: walk at the nearest enemy, to fight and to pick gems up.
                unit((-nearest.0, -nearest.1))
            } else {
                unit((ax, ay))
            }
        }
        Bot::Seek => {
            let ((ax, ay), _) = avoid(g);
            let away = unit((ax, ay));
            let Some((tx, ty)) = target else { return away };
            let (dx, dy) = (tx - g.player.x, ty - g.player.y);
            // On the spot: stay, stepping aside only from what comes close.
            if dx * dx + dy * dy < 40.0 * 40.0 {
                return (away.0 * 0.6, away.1 * 0.6);
            }
            let toward = unit((dx, dy));
            unit((toward.0 + away.0 * 0.8, toward.1 + away.1 * 0.8))
        }
    }
}

fn is_art(upgrade: usize) -> bool {
    ARTS.iter().any(|a| a.upgrade == upgrade)
}

/// Answer the cards on offer.
fn choose(pick: Pick, g: &mut Game, rng: &mut Rng) {
    if g.attach > 0 {
        // An element card was taken: put it on the first held art without one, else on the first held.
        let held = |a: &usize| g.up[ARTS[*a].upgrade] > 0;
        let bare = (0..N_ARTS).filter(held).find(|&a| g.arts[a].element == 0);
        let any = (0..N_ARTS).find(|a| held(a));
        g.choose(bare.or(any).unwrap_or(0));
        return;
    }
    let offered: Vec<usize> = (0..3).filter(|&c| g.choices[c] >= 0).collect();
    let upgrade = |c: usize| g.choices[c] as usize;
    let want = match pick {
        Pick::First => offered.first().copied(),
        Pick::Random => Some(offered[rng.next_u32() as usize % offered.len().max(1)]),
        Pick::Arts => offered.iter().copied().find(|&c| upgrade(c) < N_UPGRADES && is_art(upgrade(c))),
        Pick::Passives => offered.iter().copied().find(|&c| upgrade(c) < N_UPGRADES && !is_art(upgrade(c))),
    };
    g.choose(want.or(offered.first().copied()).unwrap_or(0));
}

fn run(setup: &Setup, seed: u32) -> Outcome {
    let mut g = Game::new(seed);
    g.tune = setup.tune;
    g.start(setup.mission);
    let mut rng = Rng((seed ^ 0x51ed_270b) | 1);
    let mut hurt = 0.0;
    let ticks = (setup.seconds / TICK) as u32;
    let mut target = None;
    for tick in 0..ticks {
        // The seeking bot looks at the mission's markers now and then; they are made when the game is drawn.
        if setup.bot == Bot::Seek && tick % 15 == 0 {
            g.present();
            let (px, py) = (g.player.x, g.player.y);
            let d = |p: &[f32; 4]| (p[0] - px) * (p[0] - px) + (p[1] - py) * (p[1] - py);
            if let Some(p) = g.pointers.iter().min_by(|a, b| d(a).total_cmp(&d(b))) {
                target = Some((p[0], p[1]));
            }
        }
        if setup.element > 0 {
            for art in g.arts.iter_mut() {
                art.element = setup.element;
            }
        }
        g.input = steer(setup.bot, &g, tick, target);
        let before = g.player.hp;
        g.tick();
        hurt += (before - g.player.hp).max(0.0);
        // Several levels can come at once, and an element card asks twice.
        let mut asked = 0;
        while g.choosing && asked < 16 {
            choose(setup.pick, &mut g, &mut rng);
            asked += 1;
        }
        if g.over || g.won {
            break;
        }
    }
    Outcome { won: g.won, died: g.over, time: g.time, level: g.player.level as f32, kills: g.kills as f32, hurt }
}

/// Play every seed, spread over the machine's cores.
fn row(setup: &Setup, seeds: u32) -> Vec<Outcome> {
    let workers = thread::available_parallelism().map_or(4, |n| n.get()).min(seeds.max(1) as usize);
    thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|w| {
                scope.spawn(move || {
                    (0..seeds)
                        .filter(|s| *s as usize % workers == w)
                        .map(|s| (s, run(setup, 1000 + s * 7919)))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut all: Vec<(u32, Outcome)> = handles.into_iter().flat_map(|h| h.join().unwrap()).collect();
        all.sort_by_key(|(s, _)| *s);
        all.into_iter().map(|(_, o)| o).collect()
    })
}

fn find<T: Copy>(table: &[(&str, T)], name: &str, what: &str) -> Vec<T> {
    if name == "all" {
        return table.iter().map(|(_, v)| *v).collect();
    }
    match table.iter().find(|(n, _)| *n == name) {
        Some((_, v)) => vec![*v],
        None => fail(&format!("unknown {what} `{name}`")),
    }
}

fn fail(message: &str) -> ! {
    eprintln!("sim: {message}");
    std::process::exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut missions = vec![M_ENDLESS];
    let (mut seeds, mut seconds) = (24u32, 300.0f32);
    let (mut bots, mut picks) = (find(&BOTS, "all", "bot"), find(&PICKS, "all", "pick"));
    let mut elements: Vec<u8> = vec![0];
    let mut tune = TUNE_DEFAULT;
    let mut csv = false;
    let mut i = 0;
    while i < args.len() {
        let value =
            || args.get(i + 1).map(String::as_str).unwrap_or_else(|| fail(&format!("{} needs a value", args[i])));
        match args[i].as_str() {
            "--csv" => {
                csv = true;
                i += 1;
                continue;
            }
            "--mission" => {
                missions = match value() {
                    "all" => (0..N_MISSIONS).collect(),
                    name => {
                        vec![MISSION_NAMES.iter().position(|n| *n == name).unwrap_or_else(|| fail("unknown mission"))]
                    }
                }
            }
            "--seeds" => seeds = value().parse().unwrap_or_else(|_| fail("--seeds takes a whole number")),
            "--seconds" => seconds = value().parse().unwrap_or_else(|_| fail("--seconds takes a number")),
            "--bot" => bots = find(&BOTS, value(), "bot"),
            "--pick" => picks = find(&PICKS, value(), "pick"),
            "--element" => {
                elements = match value() {
                    "each" => (0..=N_ELEMENTS as u8).collect(),
                    name => {
                        vec![ELEMENT_NAMES.iter().position(|n| *n == name).unwrap_or_else(|| fail("unknown element"))
                            as u8]
                    }
                }
            }
            "--tune" => {
                let (name, v) = value().split_once('=').unwrap_or_else(|| fail("--tune takes name=value"));
                let id = TUNE_NAMES.iter().position(|n| *n == name).unwrap_or_else(|| fail("unknown tune"));
                let v: f32 = v.parse().unwrap_or_else(|_| fail("a tune's value is a number"));
                tune[id] = v.clamp(TUNE_MIN[id], TUNE_MAX[id]);
            }
            other => fail(&format!("unknown option {other}")),
        }
        i += 2;
    }
    // A row that fixes the element offers no element cards, so only that element is measured.
    let fixed = elements != [0];
    if fixed {
        tune[T_ELEMENT_CARDS] = 0.0;
    }

    if csv {
        println!("mission,bot,pick,element,runs,won,died,seconds,level,kills,kills_per_min,hurt_per_min");
    }
    for &mission in &missions {
        print_mission(mission, &bots, &picks, &elements, seeds, seconds, tune, csv, fixed);
    }
}

#[allow(clippy::too_many_arguments)]
fn print_mission(
    mission: usize,
    bots: &[Bot],
    picks: &[Pick],
    elements: &[u8],
    seeds: u32,
    seconds: f32,
    tune: [f32; N_TUNES],
    csv: bool,
    fixed: bool,
) {
    if !csv {
        println!(
            "{} · {seeds} runs a row · up to {seconds} s{}",
            MISSION_NAMES[mission],
            if fixed { " · no element cards" } else { "" }
        );
        println!(
            "{:<7} {:<9} {:<10} {:>5} {:>5} {:>8} {:>6} {:>7} {:>9} {:>9}",
            "bot", "pick", "element", "won", "died", "seconds", "level", "kills", "kills/min", "hurt/min"
        );
    }
    for &bot in bots {
        for &pick in picks {
            for &element in elements {
                let setup = Setup { mission, seconds, tune, bot, pick, element };
                let runs = row(&setup, seeds);
                let n = runs.len().max(1) as f32;
                let mean = |f: fn(&Outcome) -> f32| runs.iter().map(f).sum::<f32>() / n;
                let won = runs.iter().filter(|o| o.won).count();
                let died = runs.iter().filter(|o| o.died).count();
                let minutes: f32 = runs.iter().map(|o| o.time).sum::<f32>() / 60.0;
                let per_min = |total: f32| if minutes > 0.0 { total / minutes } else { 0.0 };
                let kills_per_min = per_min(runs.iter().map(|o| o.kills).sum());
                let hurt_per_min = per_min(runs.iter().map(|o| o.hurt).sum());
                let bot_name = BOTS.iter().find(|(_, b)| *b == bot).unwrap().0;
                let pick_name = PICKS.iter().find(|(_, p)| *p == pick).unwrap().0;
                let element_name = ELEMENT_NAMES[element as usize];
                if csv {
                    println!(
                        "{},{bot_name},{pick_name},{element_name},{},{won},{died},{:.1},{:.2},{:.1},{:.1},{:.1}",
                        MISSION_NAMES[mission],
                        runs.len(),
                        mean(|o| o.time),
                        mean(|o| o.level),
                        mean(|o| o.kills),
                        kills_per_min,
                        hurt_per_min
                    );
                } else {
                    println!(
                        "{bot_name:<7} {pick_name:<9} {element_name:<10} {won:>5} {died:>5} {:>8.1} {:>6.1} {:>7.0} {:>9.1} {:>9.1}",
                        mean(|o| o.time),
                        mean(|o| o.level),
                        mean(|o| o.kills),
                        kills_per_min,
                        hurt_per_min
                    );
                }
            }
        }
    }
}
