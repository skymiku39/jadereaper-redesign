//! School enemies (docs/design.md 4.8.6, docs/martial-arts.md): elites and
//! masters that use the moves of their school, warned of on the ground first,
//! so the player can see them and step out; and disciples that throw. A move's
//! name is called out in red; felling one teaches the school's stolen routes
//! like any disciple (`routes.rs`).
//!
//! An enemy kind with a `strike` uses that `[[move]]`, and takes turns with
//! `strike2` when it has one: an arc toward the player or a ring around
//! itself, `reach` times `FOE_REACH` across; a negative knockback pulls the
//! player in. A wave throws its shots at the player, slower than the player's
//! own. A turret move sets down mechanism stakes that shoot at the player for
//! a while: one at the foe, one circling it (`width`), one that walks after
//! the player (`seek`), or a ring of them round the player that stretches a
//! net slowing him (`count` above 1, `slow`). A master (a kind that drops
//! manuals) goes through every move of the route its `strike` belongs to, in
//! order, and warns longer of the ultimate at the end. A kind with `throw`
//! throws a dart at the player every that many seconds from afar as it comes
//! on, without a warning. Only kinds that come with a `school_share` above 0 exist, so with
//! it at 0 nothing here runs.

use super::{Fx, Game};
use crate::defs::*;
use crate::fmath;
use core::f32::consts::TAU;

/// How far a foe's move reaches per 1 of the move's `reach`.
const FOE_REACH: f32 = 100.0;
/// Seconds of warning before an elite's move lands.
const WIND_UP: f32 = 0.6;
/// Seconds of warning before a master's ultimate lands.
const WIND_UP_ULTIMATE: f32 = 1.2;
/// Seconds between the end of one move and the start of the next.
const REST: f32 = 3.0;
/// The most foes tracked at once.
const MAX_FOES: usize = 16;
/// The colour of a foe's warning and blow.
const RED: [f32; 3] = [0.72, 0.16, 0.12];
/// A foe's shots fly at this share of the speed of the player's own, for as far.
const SHOT_PACE: f32 = 0.55;
/// How near a foe's shot must come to the player to hit him.
const SHOT_HIT: f32 = 22.0;
/// The most foe shots in the air at once.
const MAX_FOE_SHOTS: usize = 96;
/// The most mechanism stakes of foes at once.
pub const MAX_STAKES: usize = 8;
/// How near the player must be for a stake to shoot at him.
const STAKE_RANGE: f32 = 380.0;
/// The fewest seconds between a stake's shots; slower than the player's own mechanisms.
const STAKE_BEAT: f32 = 1.4;
/// How fast a walking stake follows the player.
const STAKE_FOLLOW: f32 = 70.0;
/// How fast a stake circles its foe, in radians a second.
const STAKE_CIRCLE: f32 = 2.0;
/// How far a thrower throws.
const THROW_RANGE: f32 = 320.0;
/// How fast a thrower's dart flies.
const THROW_SPEED: f32 = 300.0;
/// A dart's damage, as a share of the thrower's blow when it runs into the player.
const THROW_SHARE: f32 = 0.5;
/// The chance an elite (not a master) drops a manual of its school when it falls.
pub const ELITE_MANUAL: f32 = 0.2;

/// A foe that uses its school's moves.
#[derive(Clone, Copy)]
pub struct Foe {
    /// The enemy it is, by id.
    pub id: u32,
    /// Seconds left of the warning; 0 while not winding up.
    pub wind: f32,
    /// Seconds before it may start its next move.
    pub rest: f32,
    /// Which way the move goes, set when the warning starts.
    pub ax: f32,
    pub ay: f32,
    /// The move it is winding up or used last.
    pub mv: u16,
    /// How many moves it has used: a master's place in its route, or an
    /// elite's turn between its two moves.
    pub step: u8,
    /// Where the player stood when the warning started (a ring of stakes goes there).
    pub tx: f32,
    pub ty: f32,
}

/// A shot thrown by a foe at the player.
#[derive(Clone, Copy)]
pub struct FoeShot {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub dmg: f32,
}

/// A mechanism stake set down by a foe's turret move.
#[derive(Clone, Copy)]
pub struct Stake {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    /// Seconds before it shoots again.
    pub cd: f32,
    pub mv: u16,
    /// The foe it circles, by id, at radius `orbit`; 0 when it stands or walks.
    pub owner: u32,
    pub orbit: f32,
    pub angle: f32,
    pub dmg: f32,
}

/// The net of a foe's ring of stakes (天羅地網): the player walks slower inside.
#[derive(Clone, Copy)]
pub struct FoeNet {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub life: f32,
    pub slow: f32,
}

/// The move a foe of enemy kind `kind` uses as its `step`-th: a master
/// (a kind that drops manuals) goes through the whole route its `strike`
/// belongs to, in order, the ultimate last, and round again; an elite takes
/// turns between its `strike` and `strike2`, or always uses its `strike`.
pub(crate) fn foe_move(kind: usize, step: u8) -> usize {
    let k = &KINDS[kind];
    let s = k.strike;
    if s >= N_MOVES {
        return s;
    }
    if !k.manual {
        return if k.strike2 < N_MOVES && step % 2 == 1 { k.strike2 } else { s };
    }
    let r = &ROUTES[MOVES[s].route];
    r.first + step as usize % r.n.max(1)
}

/// How near the player must be for a foe to start move `m`: its reach, how
/// far its shots fly, or how far its stakes shoot.
pub(crate) fn foe_reach(m: usize) -> f32 {
    let def = &MOVES[m];
    match def.shape {
        MS_WAVE => def.speed.max(300.0) * def.life.max(0.3) * 0.8,
        MS_TURRET => STAKE_RANGE * 0.8,
        _ => def.reach.abs() * FOE_REACH,
    }
}

/// How a foe's move `m` hits: all round, or the cosine bound of its arc. A
/// thrust is a narrow arc, or a fan as wide as its `spread`, or all round; so
/// is a wave's warning.
fn foe_shape(m: usize) -> (bool, f32) {
    let def = &MOVES[m];
    match def.shape {
        MS_RING => (true, -2.0),
        MS_TURRET => (true, -2.0),
        MS_THRUST | MS_WAVE if def.spread >= 1.0 => (true, -2.0),
        MS_THRUST | MS_WAVE if def.spread > 0.0 => (false, fmath::cos(def.spread * core::f32::consts::PI)),
        MS_THRUST | MS_WAVE => (false, 0.9),
        _ => (def.width < -1.0, def.width),
    }
}

impl Game {
    /// Whether an enemy stands still, winding up a move.
    #[cfg(test)]
    pub(crate) fn winding(&self, id: u32) -> bool {
        self.foes.iter().any(|f| f.id == id && f.wind > 0.0)
    }

    /// One step of the foes: track new ones, forget fallen ones, start warnings
    /// and land the moves whose warning ran out; then the throwers, the shots
    /// and the stakes.
    pub(crate) fn foes_step(&mut self, dt: f32) {
        // Track every enemy with a move, and forget the ones that are gone.
        self.foes.retain(|f| self.enemies.iter().any(|e| e.id == f.id));
        for e in &self.enemies {
            if KINDS[e.kind as usize].strike < N_MOVES
                && self.foes.len() < MAX_FOES
                && !self.foes.iter().any(|f| f.id == e.id)
            {
                let mv = foe_move(e.kind as usize, 0) as u16;
                self.foes.push(Foe { id: e.id, wind: 0.0, rest: 1.5, ax: 1.0, ay: 0.0, mv, step: 0, tx: 0.0, ty: 0.0 });
            }
        }
        let (px, py) = (self.player.x, self.player.y);
        for n in 0..self.foes.len() {
            let f = self.foes[n];
            let Some(i) = self.enemies.iter().position(|e| e.id == f.id) else { continue };
            let (ex, ey, kind) = (self.enemies[i].x, self.enemies[i].y, self.enemies[i].kind as usize);
            if f.wind > 0.0 {
                let wind = f.wind - dt;
                self.foes[n].wind = wind.max(0.0);
                if wind <= 0.0 {
                    self.foes[n].rest = REST;
                    if KINDS[kind].manual || KINDS[kind].strike2 < N_MOVES {
                        self.foes[n].step = f.step.wrapping_add(1);
                    }
                    self.foe_strike(i, f);
                }
                continue;
            }
            let m = foe_move(kind, f.step);
            let reach = foe_reach(m);
            let rest = f.rest - dt;
            self.foes[n].rest = rest.max(0.0);
            let (dx, dy) = (px - ex, py - ey);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if rest <= 0.0 && d < reach * 1.1 {
                // Start the warning: the shape of the move on the ground, and its name.
                let (ax, ay) = (dx / d, dy / d);
                let wind = if MOVES[m].ultimate && KINDS[kind].manual { WIND_UP_ULTIMATE } else { WIND_UP };
                let warned = Foe { wind, ax, ay, mv: m as u16, tx: px, ty: py, ..self.foes[n] };
                self.foes[n] = warned;
                self.foe_warn(ex, ey, &warned, wind);
                self.event(EV_FOE, ex, ey, m as f32);
            }
        }
        self.foe_throws(dt);
        self.tick_stakes(dt);
        self.move_foe_shots(dt);
        self.tick_foe_net(dt);
    }

    /// Show on the ground where foe `f` at (x, y) will land its move, for `life` seconds.
    fn foe_warn(&mut self, x: f32, y: f32, f: &Foe, life: f32) {
        let m = f.mv as usize;
        let def = &MOVES[m];
        if def.shape == MS_TURRET {
            // Where the stakes go: a ring round the player, a circle round the foe, or at its feet.
            let (cx, cy, r) = if def.count > 1 {
                (f.tx, f.ty, def.offset * 100.0)
            } else if def.width > 0.0 {
                (x, y, def.width)
            } else {
                (x, y, 40.0)
            };
            self.foe_mark(cx, cy, f.ax, f.ay, m, r, 0.35, life);
            return;
        }
        self.foe_mark(x, y, f.ax, f.ay, m, foe_reach(m), 0.35, life);
    }

    /// Draw a foe's move at (x, y): faint while it warns, solid when it lands.
    #[allow(clippy::too_many_arguments)]
    fn foe_mark(&mut self, x: f32, y: f32, ax: f32, ay: f32, m: usize, reach: f32, alpha: f32, life: f32) {
        let (ring, _) = foe_shape(m);
        self.add_fx(Fx {
            x,
            y: y - 6.0,
            vx: 0.0,
            vy: 0.0,
            life,
            max: life,
            part: if ring { P_RING } else { P_SLASH },
            hw: reach,
            hh: reach,
            rot: fmath::atan2(ay, ax),
            vr: 0.0,
            grow: 0.0,
            color: [RED[0], RED[1], RED[2], alpha],
            seed: 0.6,
            top: false,
        });
    }

    /// Foe `i` lands the move `f` warned of.
    fn foe_strike(&mut self, i: usize, f: Foe) {
        let m = f.mv as usize;
        let (ax, ay) = (f.ax, f.ay);
        let def = &MOVES[m];
        let dmg = KINDS[self.enemies[i].kind as usize].dmg * def.power.max(0.5);
        match def.shape {
            MS_WAVE => return self.foe_volley(i, m, ax, ay, dmg),
            MS_TURRET => return self.set_stakes(i, f, dmg),
            _ => {}
        }
        let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
        let reach = def.reach.abs() * FOE_REACH;
        self.foe_mark(ex, ey, ax, ay, m, reach, 0.85, 0.25);
        let (dx, dy) = (self.player.x - ex, self.player.y - ey);
        let d = (dx * dx + dy * dy).sqrt().max(0.001);
        let (ring, width) = foe_shape(m);
        let inside = d < reach + 14.0 && (ring || d < 30.0 || (dx * ax + dy * ay) / d > width);
        if !inside {
            return;
        }
        // A pull drags the player toward the foe; a push throws him back.
        let shove = if def.knockback < 0.0 { -(d - 30.0).clamp(0.0, 90.0) } else { 40.0 * def.knockback };
        self.player.x += dx / d * shove;
        self.player.y += dy / d * shove;
        self.hurt_player(dmg);
    }

    /// Foe `i` throws the shots of wave move `m` toward (ax, ay): fanned over
    /// its `spread`, or evenly all round with a spread of 1 or more.
    fn foe_volley(&mut self, i: usize, m: usize, ax: f32, ay: f32, dmg: f32) {
        let def = &MOVES[m];
        let (ex, ey) = (self.enemies[i].x, self.enemies[i].y);
        let n = def.count.max(1);
        let base = fmath::atan2(ay, ax);
        let speed = def.speed.max(300.0) * SHOT_PACE;
        let life = def.life.max(0.3) / SHOT_PACE;
        for k in 0..n {
            let a = if def.spread >= 1.0 {
                base + k as f32 / n as f32 * TAU
            } else if n > 1 {
                base + (k as f32 / (n - 1) as f32 - 0.5) * def.spread * TAU
            } else {
                base
            };
            self.foe_shot(ex, ey, fmath::cos(a), fmath::sin(a), speed, life, dmg);
        }
    }

    /// A foe's shot from (x, y) along (dx, dy).
    #[allow(clippy::too_many_arguments)]
    fn foe_shot(&mut self, x: f32, y: f32, dx: f32, dy: f32, speed: f32, life: f32, dmg: f32) {
        if self.foe_shots.len() >= MAX_FOE_SHOTS {
            return;
        }
        self.foe_shots.push(FoeShot { x, y, vx: dx * speed, vy: dy * speed, life, dmg });
    }

    /// Foe `i` sets down the stakes of the turret move `f` warned of.
    fn set_stakes(&mut self, i: usize, f: Foe, dmg: f32) {
        let m = f.mv as usize;
        let def = &MOVES[m];
        let (ex, ey, id) = (self.enemies[i].x, self.enemies[i].y, self.enemies[i].id);
        let count = def.count.clamp(1, MAX_STAKES);
        for k in 0..count {
            let angle = k as f32 / count as f32 * TAU;
            let (x, y) = if count > 1 {
                let r = def.offset * 100.0;
                (f.tx + fmath::cos(angle) * r, f.ty + fmath::sin(angle) * r * 0.8)
            } else {
                (ex, ey)
            };
            if self.stakes.len() >= MAX_STAKES {
                self.stakes.remove(0);
            }
            let orbit = if count == 1 { def.width.max(0.0) } else { 0.0 };
            let owner = if orbit > 0.0 { id } else { 0 };
            self.stakes.push(Stake { x, y, life: def.life.max(0.5), cd: 0.5, mv: m as u16, owner, orbit, angle, dmg });
        }
        if def.slow > 0.0 && count > 1 {
            // The ring of stakes is a net round where the player stood.
            let r = def.offset * 100.0 + 20.0;
            self.foe_net = Some(FoeNet { x: f.tx, y: f.ty, r, life: def.life.max(0.5), slow: def.slow.min(0.9) });
        }
    }

    /// One step of the foes' stakes: circle, walk, shoot at the player, and burst at the end.
    fn tick_stakes(&mut self, dt: f32) {
        if self.stakes.is_empty() {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let mut i = 0;
        while i < self.stakes.len() {
            let mut s = self.stakes[i];
            let def = &MOVES[s.mv as usize];
            s.life -= dt;
            if s.owner != 0 {
                match self.enemies.iter().find(|e| e.id == s.owner) {
                    Some(e) => {
                        s.angle += STAKE_CIRCLE * dt;
                        s.x = e.x + fmath::cos(s.angle) * s.orbit;
                        s.y = e.y + fmath::sin(s.angle) * s.orbit * 0.8;
                    }
                    // Its foe fell: it stays where it is.
                    None => s.owner = 0,
                }
            } else if def.seek > 0 {
                let (dx, dy) = (px - s.x, py - s.y);
                let d = (dx * dx + dy * dy).sqrt();
                if d > 60.0 {
                    s.x += dx / d * STAKE_FOLLOW * dt;
                    s.y += dy / d * STAKE_FOLLOW * dt;
                }
            }
            let speed = def.speed.max(300.0) * SHOT_PACE;
            let life = STAKE_RANGE * 1.2 / speed;
            s.cd -= dt;
            if s.cd <= 0.0 {
                let (dx, dy) = (px - s.x, py - s.y);
                let d = (dx * dx + dy * dy).sqrt().max(0.001);
                if d < STAKE_RANGE {
                    s.cd = (def.delay * 3.0).max(STAKE_BEAT);
                    self.foe_shot(s.x, s.y, dx / d, dy / d, speed, life, s.dmg);
                } else {
                    s.cd = 0.2;
                }
            }
            if s.life <= 0.0 {
                // It ends: a burst all round, if it is made to.
                for k in 0..def.split.min(24) {
                    let a = k as f32 / def.split as f32 * TAU;
                    self.foe_shot(s.x, s.y, fmath::cos(a), fmath::sin(a), speed, life, s.dmg);
                }
                self.stakes.remove(i);
                continue;
            }
            self.stakes[i] = s;
            i += 1;
        }
    }

    /// The throwers: each throws a dart at the player every `throw` seconds
    /// while he is in range. When comes from the run's clock and the enemy's
    /// own seed, so it keeps no state of its own.
    fn foe_throws(&mut self, dt: f32) {
        let (px, py) = (self.player.x, self.player.y);
        let t = self.time;
        for i in 0..self.enemies.len() {
            let e = self.enemies[i];
            let k = &KINDS[e.kind as usize];
            if k.throw <= 0.0 {
                continue;
            }
            let p = t + e.seed * k.throw;
            if (p / k.throw).floor() == ((p - dt) / k.throw).floor() {
                continue;
            }
            let (dx, dy) = (px - e.x, py - e.y);
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            if d < THROW_RANGE {
                let dmg = k.dmg * THROW_SHARE;
                self.foe_shot(e.x, e.y, dx / d, dy / d, THROW_SPEED, THROW_RANGE * 1.2 / THROW_SPEED, dmg);
            }
        }
    }

    /// Move the foes' shots and let them hurt the player.
    fn move_foe_shots(&mut self, dt: f32) {
        if self.foe_shots.is_empty() {
            return;
        }
        let mut i = 0;
        while i < self.foe_shots.len() {
            let s = &mut self.foe_shots[i];
            s.x += s.vx * dt;
            s.y += s.vy * dt;
            s.life -= dt;
            let s = *s;
            let (dx, dy) = (self.player.x - s.x, self.player.y - s.y);
            let hit = self.player.iframe <= 0.0 && dx * dx + dy * dy < SHOT_HIT * SHOT_HIT;
            if s.life <= 0.0 || hit {
                self.foe_shots.swap_remove(i);
                if hit {
                    self.hurt_player(s.dmg);
                }
            } else {
                i += 1;
            }
        }
    }

    /// One step of a foe's net: it comes down when its time is up.
    fn tick_foe_net(&mut self, dt: f32) {
        if let Some(n) = &mut self.foe_net {
            n.life -= dt;
            if n.life <= 0.0 {
                self.foe_net = None;
            }
        }
    }

    /// How much of his speed the player keeps: less inside a foe's net.
    pub(crate) fn foe_net_pace(&self) -> f32 {
        match &self.foe_net {
            Some(n) => {
                let (dx, dy) = (self.player.x - n.x, (self.player.y - n.y) / 0.8);
                if dx * dx + dy * dy < n.r * n.r {
                    1.0 - n.slow
                } else {
                    1.0
                }
            }
            None => 1.0,
        }
    }

    /// A blow from a foe's move, through the player's grace, inner arts and guard.
    fn hurt_player(&mut self, dmg: f32) {
        if self.player.iframe > 0.0 {
            return;
        }
        let (px, py) = (self.player.x, self.player.y);
        let dmg = self.soften(dmg);
        self.player.hp -= self.guard_against(dmg);
        self.player.iframe = 0.8;
        self.shake = 1.0;
        self.event(EV_HURT, px, py, dmg);
        if self.player.hp <= 0.0 && !self.last_stand() {
            self.player.hp = 0.0;
            self.over = true;
            self.event(EV_DEATH, px, py, 0.0);
        }
    }

    /// Draw the foes' net, stakes and shots, in red ink.
    pub(crate) fn draw_foes(&mut self) {
        if let Some(n) = self.foe_net {
            let color = [RED[0], RED[1], RED[2], 0.5 * n.life.min(1.0)];
            self.put(n.x, n.y, n.r, n.r * 0.8, 0.0, P_RING, 0.0, 0.3, color);
            self.put(n.x, n.y, n.r * 0.6, n.r * 0.48, 0.0, P_RING, 0.0, 0.31, color);
        }
        for i in 0..self.stakes.len() {
            let s = self.stakes[i];
            let lift = if s.orbit > 0.0 { 46.0 } else { 16.0 };
            let size = if s.orbit > 0.0 { 9.0 } else { 13.0 };
            let color = [RED[0] * 0.8, RED[1], RED[2], 1.0];
            self.put(s.x, s.y - lift, size, size * 1.3, s.angle * 0.2, P_TALISMAN, 0.0, 0.2, color);
        }
        for i in 0..self.foe_shots.len() {
            let s = self.foe_shots[i];
            let rot = fmath::atan2(s.vy, s.vx);
            self.put(s.x, s.y - 14.0, 9.0, 3.5, rot, P_TALISMAN, 0.0, i as f32 * 0.17, [RED[0], RED[1], RED[2], 1.0]);
        }
    }

    /// The words the hash takes from the foes, when there are any.
    pub(crate) fn foes_hash(&self, mix: &mut impl FnMut(u32)) {
        if !self.foes.is_empty() {
            mix(self.foes.len() as u32);
            for f in &self.foes {
                mix(f.id);
                mix(f.wind.to_bits());
                mix(f.rest.to_bits());
                if f.step > 0 {
                    mix(f.step as u32);
                }
            }
        }
        if !self.foe_shots.is_empty() {
            mix(self.foe_shots.len() as u32);
            for s in &self.foe_shots {
                mix(s.x.to_bits());
                mix(s.y.to_bits());
            }
        }
        if !self.stakes.is_empty() {
            mix(self.stakes.len() as u32);
            for s in &self.stakes {
                mix(s.life.to_bits());
                mix(s.cd.to_bits());
            }
        }
        if let Some(n) = &self.foe_net {
            mix(n.life.to_bits());
        }
    }
}
