//! Routes (套路): what a martial art turns into from level 3 (docs/design.md 4.8.3).
//!
//! Which routes exist, their moves and every number come from `[[route]]` and
//! `[[move]]` in `content/game.toml`, through `ROUTES` and `MOVES`. With the
//! `routes` tune on, an art that reaches level 3 takes a route, and from then
//! on uses one move per beat, in turn: its basic move and the moves learned so
//! far, then, once every move is learned, the moves alone and the ultimate
//! last. Off, nothing here runs and the arts grow as they did before.
//!
//! Moves are measured against the art's own numbers at level 2: a move's
//! `power` is a share of that damage and its `reach` a share of that reach.

use super::{Fx, Game, Shot, MAX_SHOTS};
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// The colour of a move while its art carries no element.
const JADE: [f32; 3] = [0.25, 0.62, 0.48];
/// A move order entry for the art's basic move.
pub const BASIC: u16 = u16::MAX;
/// The longest a move order can be: the basic move and eight moves.
pub const MOST_STEPS: usize = 9;
/// How many delayed hits can wait at once.
const MAX_DELAYED: usize = 64;

/// A hit that lands a moment after its move: a later sweep of an arc, or a mark.
#[derive(Clone, Copy)]
pub struct Delayed {
    /// Seconds until it lands.
    pub left: f32,
    pub art: u8,
    pub mv: u16,
    /// Which sweep or mark of the move it is, from 0.
    pub k: u8,
    /// Where a mark lands; unused by a sweep, which lands where the player is then.
    pub x: f32,
    pub y: f32,
    /// The aim when the move began.
    pub ax: f32,
    pub ay: f32,
    /// The damage factor of the move when it began (inner arts, the weapon, its height).
    pub might: f32,
    /// The element of the ultimate at its height it belongs to; 0 for none (`zenith.rs`).
    pub zen: u8,
}

impl Game {
    /// Whether routes are on for this run.
    pub(crate) fn routes_on(&self) -> bool {
        self.tune[T_ROUTES] > 0.5
    }

    /// The routes an art can take now, in id order, at most three: the ones
    /// known from the start and the stolen ones learned so far.
    pub(crate) fn routes_of(&self, a: usize) -> impl Iterator<Item = usize> {
        let stolen = self.stolen;
        let kind = self.kind_of(a);
        (0..N_ROUTES)
            .filter(move |&r| {
                let def = &ROUTES[r];
                def.art == a && (def.start || stolen & (1 << r) != 0) && Self::kind_fits(kind, r)
            })
            .take(3)
    }

    /// Whether an art of weapon kind `kind` can learn route `r`: a route limited
    /// to some kinds (限用) only by those, any other by any kind but a mechanism.
    pub(crate) fn kind_fits(kind: Option<usize>, r: usize) -> bool {
        let only = ROUTES[r].only;
        match kind {
            Some(w) if only != 0 => only & (1 << w) != 0,
            Some(w) => WEAPONS[w].forms & FORM_MECHANISM == 0,
            None => only == 0,
        }
    }

    /// Where stolen route `r`'s manuals are kept: its place among the stolen
    /// routes in id order (two bits each, twelve at most); none for a route
    /// known from the start.
    pub fn manual_slot(r: usize) -> Option<usize> {
        if ROUTES[r].start {
            return None;
        }
        let slot = (0..r).filter(|&q| !ROUTES[q].start).count();
        (slot < 12).then_some(slot)
    }

    /// How many manuals (秘笈) route `r` has: moves opened beyond the second, 0 to 3.
    pub(crate) fn manuals_of(&self, r: usize) -> usize {
        match Self::manual_slot(r) {
            Some(slot) => ((self.manuals >> (2 * slot)) & 3) as usize,
            None => 0,
        }
    }

    /// How many of route `r`'s moves can be learned: all of a route known from
    /// the start; of a stolen one, two and one more per manual.
    pub(crate) fn moves_open(&self, r: usize) -> usize {
        let def = &ROUTES[r];
        if def.start {
            def.n
        } else {
            def.n.min(2 + self.manuals_of(r))
        }
    }

    /// A master of school `s` fell and dropped a manual of the school. It is
    /// kept for after the run, where the player chooses which of the school's
    /// stolen routes it opens a move of (design 4.5); the host keeps it and
    /// sets the `manuals` tune of later runs. Nothing opens in this run.
    pub(crate) fn gain_manual(&mut self, s: usize) {
        if self.routes_on() && (0..N_ROUTES).any(|r| ROUTES[r].school == s && Self::manual_slot(r).is_some()) {
            self.event(EV_MANUAL, self.player.x, self.player.y, s as f32);
        }
    }

    /// A disciple of school `s` fell to the player: every stolen route of that
    /// school not yet learned is learned now (偷師), and offered from now on.
    pub(crate) fn steal(&mut self, s: usize) {
        if !self.routes_on() {
            return;
        }
        for (r, def) in ROUTES.iter().enumerate().take(32) {
            if def.school == s && !def.start && self.stolen & (1 << r) == 0 {
                self.stolen |= 1 << r;
                self.event(EV_STEAL, self.player.x, self.player.y, r as f32);
            }
        }
    }

    /// The art an upgrade levels, if it is one.
    pub(crate) fn art_of_upgrade(id: usize) -> Option<usize> {
        ARTS.iter().position(|art| art.upgrade == id)
    }

    /// The route an art follows, if any.
    pub(crate) fn route_of(&self, a: usize) -> Option<usize> {
        match self.arts[a].route {
            0 => None,
            r => Some(r as usize - 1),
        }
    }

    /// The highest level an upgrade can reach in this run. An art on a route
    /// reaches 2 + the route's moves; anything else its own top.
    pub(crate) fn upgrade_top(&self, id: usize) -> u8 {
        if let Some(r) = Self::art_of_upgrade(id).and_then(|a| self.route_of(a)) {
            return (2 + self.moves_open(r)) as u8;
        }
        // An inner art has three layers.
        if self.inner_on() && Self::is_passive(id) {
            return UPGRADE_MAX[id].min(3);
        }
        UPGRADE_MAX[id]
    }

    /// The art whose route must be chosen before this upgrade can take its
    /// next level: one at level 2 with routes on and a route to take.
    pub(crate) fn route_needed(&self, id: usize) -> Option<usize> {
        let a = Self::art_of_upgrade(id)?;
        let waiting = self.routes_on() && self.up[id] == 2 && self.arts[a].route == 0;
        (waiting && self.routes_of(a).next().is_some()).then_some(a)
    }

    /// Put art `a` on route `r` (an index into `ROUTES`), starting its move order over.
    pub(crate) fn take_route(&mut self, a: usize, r: usize) {
        self.arts[a].route = r as u8 + 1;
        self.arts[a].step = 0;
        self.arts[a].timer = self.arts[a].timer.min(ROUTES[r].beat);
    }

    /// The art's move order now: `BASIC` or move ids, and how many there are.
    /// Empty for an art without a route or below level 3.
    pub(crate) fn move_order(&self, a: usize) -> ([u16; MOST_STEPS], usize) {
        let mut order = [BASIC; MOST_STEPS];
        let (Some(r), level) = (self.route_of(a), self.art_level(a)) else { return (order, 0) };
        if level < 3 {
            return (order, 0);
        }
        let def = &ROUTES[r];
        let ultimate = MOVES[def.first + def.n - 1].ultimate as usize;
        let regular = def.n - ultimate;
        let learned = (level - 2).min(def.n);
        let mut len = 0;
        if learned < regular {
            // Still learning: the basic move opens the order.
            len = 1;
            for m in 0..learned {
                order[len] = (def.first + m) as u16;
                len += 1;
            }
        } else {
            for m in 0..learned {
                order[len] = (def.first + m) as u16;
                len += 1;
            }
        }
        (order, len)
    }

    /// One beat of an art on a route: when its timer runs out, use the next move.
    pub(crate) fn route_beat(&mut self, a: usize, dt: f32, might: f32, haste: f32) {
        let Some(r) = self.route_of(a) else { return };
        self.arts[a].timer -= dt;
        if self.arts[a].timer > 0.0 {
            return;
        }
        self.arts[a].timer = ROUTES[r].beat * haste * self.beat_factor() * self.kind_beat(a);
        let might = might * self.kind_power(a) * self.rule_power(r);
        let (order, len) = self.move_order(a);
        if len == 0 {
            return;
        }
        let step = self.arts[a].step as usize % len;
        self.arts[a].step = ((step + 1) % len) as u8;
        self.striking = a as u8 + 1;
        // The last move of the order closes it: the mountain lifter makes it
        // land harder, and piercing qi sends a blade after it.
        let last = step + 1 == len && len > 1;
        let might = if last { might * self.finisher() } else { might };
        match order[step] {
            BASIC => self.basic_move(a, might),
            m => {
                // An ultimate at its height (極致) lands harder, with its element's height.
                let zen = if MOVES[m as usize].ultimate { self.zenith_of(a) } else { None };
                if let Some(element) = zen {
                    self.zenith = element;
                    self.reach_height(m as usize, element);
                }
                self.use_move(a, m as usize, might * self.height_power());
                self.zenith = 0;
                if MOVES[m as usize].ultimate {
                    self.note_ultimate();
                }
            }
        }
        if last {
            self.qi_blade(a, Self::basic_numbers(a).damage * might);
        }
        self.striking = 0;
    }

    /// How much harder route `r`'s moves land now for its rule: the bandits'
    /// crowd (4% for each enemy within 200, at most 60%), or the blood river's
    /// momentum (5% for each kill of the last three seconds, at most ten).
    pub(crate) fn rule_power(&self, r: usize) -> f32 {
        match ROUTES[r].rule {
            RL_CROWD => {
                let (px, py) = (self.player.x, self.player.y);
                let near = self
                    .enemies
                    .iter()
                    .filter(|e| (e.x - px) * (e.x - px) + (e.y - py) * (e.y - py) < 200.0 * 200.0)
                    .count();
                1.0 + (near as f32 * 0.04).min(0.6)
            }
            RL_MOMENTUM => 1.0 + self.momentum as f32 * 0.05,
            _ => 1.0,
        }
    }

    /// An enemy fell: while an art follows a route with momentum, it adds a stack.
    pub(crate) fn note_kill(&mut self) {
        let any = (0..N_ARTS).any(|a| self.route_of(a).is_some_and(|r| ROUTES[r].rule == RL_MOMENTUM));
        if any {
            self.momentum = (self.momentum + 1).min(10);
            self.momentum_left = 3.0;
        }
    }

    /// Count the momentum down: it all goes once three seconds pass without a kill.
    pub(crate) fn tick_momentum(&mut self, dt: f32) {
        if self.momentum > 0 {
            self.momentum_left -= dt;
            if self.momentum_left <= 0.0 {
                self.momentum = 0;
            }
        }
    }

    /// Whether the art that is striking now follows a route that leaves openings (破綻).
    pub(crate) fn striking_opens(&self) -> bool {
        match self.striking {
            0 => false,
            a => self.route_of(a as usize - 1).is_some_and(|r| ROUTES[r].rule == RL_OPENING),
        }
    }

    /// The art's numbers at level 2, which its moves are measured against.
    fn basic_numbers(a: usize) -> &'static ArtLevel {
        let levels = ARTS[a].levels;
        &levels[1.min(levels.len() - 1)]
    }

    /// The art's own move at level 2, as one step of its order. Circling
    /// beads circle all the time, so for them this step is a rest. A route
    /// that beats faster than the art's own cooldown uses it more often, so
    /// it hits and pushes that much more lightly.
    fn basic_move(&mut self, a: usize, might: f32) {
        let n = Self::basic_numbers(a);
        let (px, py) = (self.player.x, self.player.y);
        let light = self.route_of(a).map_or(1.0, |r| (ROUTES[r].beat / n.cooldown).min(1.0));
        match ARTS[a].behaviour {
            B_ARC => {
                let (ax, ay) = self.player.aim;
                let (reach, knock) = (n.reach * self.kind_reach(a), n.knockback * light * self.kind_knock(a));
                let dmg = self.second_hand(a, n.damage * might * light, reach, n.width, knock, ax, ay);
                self.sweep(a, dmg, reach, n.width, knock, ax, ay);
                self.event(EV_SLASH, px, py, 0.0);
            }
            B_PROJECTILE => {
                self.shoot_nearest(a, n, might * light);
            }
            B_THRUST => {
                let reach = n.reach * self.melee_reach() * self.kind_reach(a);
                let (dx, dy) = self.thrust_aim(reach * 1.2);
                let knock = n.knockback * light * self.kind_knock(a);
                let dmg = self.second_hand(a, n.damage * might * light, reach, n.width, knock, dx, dy);
                self.line(a, px, py, dx, dy, reach, n.width, dmg, knock);
                self.event(EV_SLASH, px, py, 0.0);
            }
            B_NOVA => {
                let (reach, knock) = (n.reach * self.kind_reach(a), n.knockback * light * self.kind_knock(a));
                self.ring(a, px, py, reach, n.damage * might * light, knock);
                self.event(EV_PULSE, px, py, 0.0);
            }
            _ => {}
        }
    }

    /// Use move `m` of art `a`.
    fn use_move(&mut self, a: usize, m: usize, might: f32) {
        // Its later hits land with the damage factor it began with.
        self.cast_might = might;
        let def = &MOVES[m];
        let (px, py) = (self.player.x, self.player.y);
        let (ax, ay) = self.player.aim;
        match def.shape {
            MS_ARC => {
                self.arc_sweep(a, m, 0, ax, ay, might);
                for k in 1..def.count.min(8) {
                    if def.delay > 0.0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.arc_sweep(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_SLASH, px, py, 0.0);
            }
            MS_WAVE => {
                // A seeking move aims at the nearest enemy, or along the aim when none is near.
                let (ax, ay) = if def.seek > 0 { self.toward_nearest().unwrap_or((ax, ay)) } else { (ax, ay) };
                for k in 0..def.count.clamp(1, 32) {
                    if def.delay > 0.0 && k > 0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.wave(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_SHOOT, px, py, 0.0);
            }
            MS_TURRET => {
                self.set_turrets(a, m, might);
                self.event(EV_SHOOT, px, py, 0.0);
            }
            MS_THRUST => {
                // A seeking thrust goes at the nearest enemy; a fan or a ring turns from there.
                let reach = Self::basic_numbers(a).reach * def.reach.max(0.1);
                let (ax, ay) = if def.seek > 0 { self.thrust_aim(reach * 1.2) } else { (ax, ay) };
                for k in 0..def.count.clamp(1, 16) {
                    if def.delay > 0.0 && k > 0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.thrust_of(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_SLASH, px, py, 0.0);
            }
            MS_RING => {
                self.ring_of(a, m, 0, ax, ay, might);
                for k in 1..def.count.min(8) {
                    if def.delay > 0.0 {
                        self.delay_hit(a, m, k, def.delay * k as f32, px, py, ax, ay);
                    } else {
                        self.ring_of(a, m, k, ax, ay, might);
                    }
                }
                self.event(EV_PULSE, px, py, 0.0);
            }
            _ => {
                // Marks: a line of them ahead, each striking a moment later.
                let gap = Self::basic_numbers(a).reach * def.spread;
                let color = self.art_color(a, JADE, 0.6);
                for k in 0..def.count.min(8) {
                    let d = gap * (k + 1) as f32;
                    let (x, y) = (px + ax * d, py + ay * d);
                    self.delay_hit(a, m, k, def.delay, x, y, ax, ay);
                    self.mark_fx(a, x, y, def.delay, color);
                }
                self.event(EV_SLASH, px, py, 0.0);
            }
        }
        self.event(EV_MOVE, px, py, m as f32);
    }

    /// Sweep `k` of an arc move, where the player stands now.
    fn arc_sweep(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        // Each later sweep turns by `spread` and reaches `grow` further.
        let turn = def.spread * TAU * k as f32;
        let (c, s) = (fmath::cos(turn), fmath::sin(turn));
        let (dx, dy) = (ax * c - ay * s, ax * s + ay * c);
        let reach = n.reach * def.reach * (1.0 + def.grow * k as f32) * self.melee_reach() * self.kind_reach(a);
        let dmg = n.damage * def.power * might;
        self.sweep(a, dmg, reach, def.width, n.knockback * def.knockback * self.kind_knock(a), dx, dy);
    }

    /// One sweep around the player towards (ax, ay): everything within `reach`
    /// and inside the `width` bound is hit and pushed away.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn sweep(&mut self, a: usize, dmg: f32, reach: f32, width: f32, knockback: f32, ax: f32, ay: f32) {
        let element = self.arts[a].element;
        let (px, py) = (self.player.x, self.player.y);
        let mut far: Option<(f32, u32)> = None;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - px, e.y - py);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            let r = KINDS[e.kind as usize].r;
            if d < reach + r && (d < 30.0 || (dx * ax + dy * ay) / d > width) {
                if far.is_none_or(|(fd, _)| d > fd) {
                    far = Some((d, e.id));
                }
                self.strike(i, dmg, dx / d * knockback, dy / d * knockback, element);
            }
        }
        self.hook(a, far.map(|(_, id)| id));
        self.formation_hit_arc(px, py, ax, ay, reach, width, dmg);
        self.player.attack = 0.25;
        let rot = fmath::atan2(ay, ax);
        let color = self.art_color(a, JADE, 0.9);
        // A sweep all round draws as two half swings back to back.
        let marks = if width < -1.0 { 2 } else { 1 };
        for k in 0..marks {
            self.add_fx(Fx {
                x: px,
                y: py - 24.0,
                vx: 0.0,
                vy: 0.0,
                life: 0.22,
                max: 0.22,
                part: P_SLASH,
                hw: reach,
                hh: reach,
                rot: rot + k as f32 * TAU * 0.5,
                vr: 5.0,
                grow: 60.0,
                color,
                seed: 0.3,
                top: true,
            });
        }
    }

    /// The way to the nearest enemy, if there is one.
    fn toward_nearest(&self) -> Option<(f32, f32)> {
        let (px, py) = (self.player.x, self.player.y);
        let mut best: Option<(f32, f32, f32)> = None;
        for e in &self.enemies {
            let (dx, dy) = (e.x - px, e.y - py);
            let d2 = dx * dx + dy * dy;
            if best.is_none_or(|(b, _, _)| d2 < b) {
                best = Some((d2, dx, dy));
            }
        }
        best.map(|(d2, dx, dy)| {
            let d = d2.sqrt().max(0.001);
            (dx / d, dy / d)
        })
    }

    /// Wave `k` of a wave move, its place in the fan around (ax, ay).
    fn wave(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        if self.shots.len() >= MAX_SHOTS {
            return;
        }
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let (px, py) = (self.player.x, self.player.y);
        let count = def.count.clamp(1, 32);
        // A fan of a whole turn has no ends, so its waves sit evenly round.
        let off = if count == 1 || def.delay > 0.0 {
            0.0
        } else if def.spread >= 1.0 {
            k as f32 / count as f32 * TAU
        } else {
            (k as f32 / (count - 1) as f32 - 0.5) * def.spread * TAU
        };
        let angle = fmath::atan2(ay, ax) + off;
        let (dx, dy) = (fmath::cos(angle), fmath::sin(angle));
        self.shots.push(Shot {
            x: px,
            y: py,
            vx: dx * def.speed,
            vy: dy * def.speed,
            life: def.life,
            dmg: n.damage * def.power * might,
            pierce: def.pierce.max(1) + self.kind_pierce(a),
            last: 0,
            art: a as u8,
            push: def.knockback,
            split: def.split.min(8) as u8,
            grow: def.grow,
            size: def.power.sqrt().clamp(0.7, 2.5),
            zen: self.zenith,
        });
    }

    /// Shot `s` reached the end of its flight: it bursts into smaller shots
    /// fanned around the way it flew, each doing half its damage.
    pub(crate) fn burst_shot(&mut self, s: usize) {
        let sh = &self.shots[s];
        let (x, y, vx, vy, n, dmg, art, zen) =
            (sh.x, sh.y, sh.vx, sh.vy, sh.split as usize, sh.dmg * 0.5, sh.art, sh.zen);
        let speed = (vx * vx + vy * vy).sqrt() * 0.8;
        let base = fmath::atan2(vy, vx);
        for k in 0..n {
            if self.shots.len() >= MAX_SHOTS {
                break;
            }
            let off = if n == 1 { 0.0 } else { (k as f32 / (n - 1) as f32 - 0.5) * TAU * 0.25 };
            let angle = base + off;
            self.shots.push(Shot {
                x,
                y,
                vx: fmath::cos(angle) * speed,
                vy: fmath::sin(angle) * speed,
                life: 0.35,
                dmg,
                pierce: 1,
                last: 0,
                art,
                push: 0.2,
                split: 0,
                grow: 0.0,
                size: 0.6,
                zen,
            });
        }
    }

    /// Thrust `k` of a thrust move from where the player stands now: turned by
    /// its place in the fan (or evenly round for a `spread` of 1), each later
    /// one reaching `grow` further; `width` above 0 widens it by that share.
    fn thrust_of(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let count = def.count.clamp(1, 16);
        let turn = if count == 1 || def.delay > 0.0 {
            0.0
        } else if def.spread >= 1.0 {
            k as f32 / count as f32 * TAU
        } else {
            (k as f32 / (count - 1) as f32 - 0.5) * def.spread * TAU
        };
        let (c, s) = (fmath::cos(turn), fmath::sin(turn));
        let (dx, dy) = (ax * c - ay * s, ax * s + ay * c);
        let reach = n.reach * def.reach * (1.0 + def.grow * k as f32) * self.melee_reach() * self.kind_reach(a);
        let width = n.width * if def.width > 0.0 { def.width } else { 1.0 };
        let knock = n.knockback * def.knockback * self.kind_knock(a);
        let (px, py) = (self.player.x, self.player.y);
        self.line(a, px, py, dx, dy, reach, width, n.damage * def.power * might, knock);
    }

    /// Ring `k` of a ring move: centred `offset` ahead along (ax, ay) from where
    /// the player stands now, each later ring reaching `grow` further.
    fn ring_of(&mut self, a: usize, m: usize, k: usize, ax: f32, ay: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let ahead = n.reach * def.offset;
        let (cx, cy) = (self.player.x + ax * ahead, self.player.y + ay * ahead);
        let radius = n.reach * def.reach * (1.0 + def.grow * k as f32) * self.melee_reach() * self.kind_reach(a);
        let knock = n.knockback * def.knockback * self.kind_knock(a);
        self.ring(a, cx, cy, radius, n.damage * def.power * might, knock);
    }

    /// Queue a hit of move `m` to land `left` seconds from now.
    #[allow(clippy::too_many_arguments)]
    fn delay_hit(&mut self, a: usize, m: usize, k: usize, left: f32, x: f32, y: f32, ax: f32, ay: f32) {
        if self.delayed.len() < MAX_DELAYED {
            let (might, zen) = (self.cast_might, self.zenith);
            self.delayed.push(Delayed { left, art: a as u8, mv: m as u16, k: k as u8, x, y, ax, ay, might, zen });
        }
    }

    /// Count down the delayed hits and land the ones that are due, oldest first.
    pub(crate) fn tick_delayed(&mut self, dt: f32) {
        if self.delayed.is_empty() {
            return;
        }
        let mut i = 0;
        while i < self.delayed.len() {
            self.delayed[i].left -= dt;
            if self.delayed[i].left > 0.0 {
                i += 1;
                continue;
            }
            let d = self.delayed.remove(i);
            let (a, m, k, might) = (d.art as usize, d.mv as usize, d.k as usize, d.might);
            // A hit of an ultimate at its height lands at its height too.
            self.zenith = d.zen;
            self.striking = d.art + 1;
            match MOVES[m].shape {
                MS_ARC => self.arc_sweep(a, m, k, d.ax, d.ay, might),
                MS_WAVE => self.wave(a, m, k, d.ax, d.ay, might),
                MS_RING => self.ring_of(a, m, k, d.ax, d.ay, might),
                MS_THRUST => self.thrust_of(a, m, k, d.ax, d.ay, might),
                _ => self.mark_strike(a, m, d.x, d.y, might),
            }
            self.zenith = 0;
            self.striking = 0;
        }
    }

    /// A mark of a marks move landing at (x, y).
    fn mark_strike(&mut self, a: usize, m: usize, x: f32, y: f32, might: f32) {
        let def = &MOVES[m];
        let n = Self::basic_numbers(a);
        let radius = n.reach * def.reach * self.kind_reach(a);
        let dmg = n.damage * def.power * might;
        let element = self.arts[a].element;
        for i in 0..self.enemies.len() {
            let e = &self.enemies[i];
            let (dx, dy) = (e.x - x, e.y - y);
            let r = radius + KINDS[e.kind as usize].r;
            if dx * dx + dy * dy < r * r {
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                self.strike(i, dmg, dx / d * 80.0, dy / d * 80.0, element);
            }
        }
        self.formation_hit_circle(x, y, radius, dmg);
        let color = self.art_color(a, JADE, 0.9);
        self.add_fx(Fx {
            x,
            y: y - 10.0,
            vx: 0.0,
            vy: 0.0,
            life: 0.2,
            max: 0.2,
            part: P_SLASH,
            hw: radius,
            hh: radius,
            rot: fmath::atan2(-1.0, 0.3),
            vr: 9.0,
            grow: 40.0,
            color,
            seed: 0.5,
            top: true,
        });
    }

    /// The scratch left by a mark until it strikes.
    fn mark_fx(&mut self, a: usize, x: f32, y: f32, life: f32, color: [f32; 4]) {
        let _ = a;
        self.add_fx(Fx {
            x,
            y: y - 4.0,
            vx: 0.0,
            vy: 0.0,
            life,
            max: life,
            part: P_SLASH,
            hw: 22.0,
            hh: 22.0,
            rot: 1.2,
            vr: 0.0,
            grow: 0.0,
            color,
            seed: 0.9,
            top: false,
        });
    }

    /// The words the hash takes from the routes, when any art follows one.
    pub(crate) fn routes_hash(&self, mix: &mut impl FnMut(u32)) {
        if self.arts.iter().all(|a| a.route == 0 && a.kind == 0) && self.delayed.is_empty() && self.route_pick == 0 {
            return;
        }
        for art in &self.arts {
            mix(art.route as u32);
            mix(art.step as u32);
            mix(art.timer.to_bits());
            mix(art.kind as u32);
            mix(art.quality as u32);
        }
        mix(self.route_pick as u32);
        mix(self.delayed.len() as u32);
        for d in &self.delayed {
            mix(d.left.to_bits());
            mix(d.mv as u32);
        }
    }
}
