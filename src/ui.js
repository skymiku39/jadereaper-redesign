// Paper UI: screens, HUD and menu focus. All DOM work lives here.
import { MISSIONS, MISSION_IDS, UPGRADES, ARTS, WEAPONS, ELEMENTS, SCHOOLS, ROUTES, MOVES } from './data.js';
import { formatTime, missionRecord } from './save.js';
import { t, getLang, setText } from './i18n.js';

const $ = (id) => document.getElementById(id);
const SCREENS = ['title', 'setup', 'settings', 'record', 'levelup', 'pause', 'over', 'won', 'error'];

/** An element's colour as CSS. `color` is 0..1 RGB. */
const elementCss = (element) => `rgb(${element.color.map((c) => Math.round(c * 255)).join(', ')})`;
/** The element with this id, or null for 0 (no element). Ids start at 1. */
const elementById = (id) => ELEMENTS[id - 1] || null;
/**
 * How far a route can be learned: all its moves when it is known from the start; a stolen one
 * two, and one more per manual. Manuals are kept two bits per stolen route, in id order.
 */
function movesOpen(route, manuals) {
  if (route.start) return route.moves.length;
  return Math.min(route.moves.length, 2 + ((manuals >>> (2 * manualSlot(route))) & 3));
}

/** Where a stolen route's manuals are kept: its place among the stolen routes, in id order. */
function manualSlot(route) {
  return ROUTES.filter((r) => !r.start && r.id < route.id).length;
}

/** The martial art an upgrade levels, or undefined for a plain stat upgrade. */
const artOfUpgrade = (upgradeId) => ARTS.find((art) => art.upgrade === upgradeId);

/** A card face: one span per [class, text] pair. */
function cardParts(card, parts) {
  for (const [cls, text] of parts) {
    const span = document.createElement('span');
    span.className = cls;
    span.textContent = text;
    card.append(span);
  }
}

/** The values a mission's text can mention, such as its time limit. */
const missionVars = (mission) => ({ time: formatTime(mission.goal), goal: mission.goal });
const INPUT_LOCK_MS = 300;

export function createUI({ onAction }) {
  let current = null;
  let lockedUntil = 0;
  const cache = {};

  // Each element's colour as a CSS variable (--element-metal and so on), so style.css can
  // tint the guard and healing marks without its own copy of the colours.
  for (const element of ELEMENTS) {
    document.documentElement.style.setProperty(`--element-${element.name}`, elementCss(element));
  }

  document.addEventListener('click', (e) => {
    const btn = e.target.closest('button[data-action]');
    if (!btn || performance.now() < lockedUntil) return;
    onAction(btn.dataset.action, btn.dataset.value);
  });

  function buttons() {
    return current ? Array.from($(current).querySelectorAll('button')) : [];
  }

  /** Show one screen (or none). A short lock stops a held button from picking something by accident. */
  function show(name) {
    for (const id of SCREENS) $(id).hidden = id !== name;
    current = name || null;
    if (current) {
      lockedUntil = performance.now() + INPUT_LOCK_MS;
      const first = buttons()[0];
      if (first) first.focus({ preventScroll: true });
    } else if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }

  /** Move focus through the open screen's buttons. Returns true if focus moved. */
  function navigate(dx, dy) {
    const list = buttons();
    if (!list.length) return false;
    const i = list.indexOf(document.activeElement);
    const step = dx > 0 || dy > 0 ? 1 : -1;
    const next = i < 0 ? 0 : (i + step + list.length) % list.length;
    list[next].focus({ preventScroll: true });
    return true;
  }

  /** Click the focused button (gamepad confirm). */
  function confirm() {
    const el = document.activeElement;
    if (current && el instanceof HTMLButtonElement && $(current).contains(el)) {
      el.click();
      return true;
    }
    return false;
  }

  function set(id, prop, value) {
    const key = id + prop;
    if (cache[key] === value) return;
    cache[key] = value;
    if (prop === 'text') $(id).textContent = value;
    else $(id).style.setProperty(prop, value);
  }

  /** The arrow at the screen edge that points to the exit while it is off screen. `exit` is null in other missions. */
  function exitMarker(exit) {
    const margin = 0.06;
    const u = exit ? exit.dx / exit.w : 0;
    const v = exit ? exit.dy / exit.h : 0;
    const off = exit && (Math.abs(u) > 0.5 - margin || Math.abs(v) > 0.5 - margin);
    set('exitMarker', 'display', off ? '' : 'none');
    if (!off) return;
    const k = (0.5 - margin) / Math.max(Math.abs(u), Math.abs(v));
    set('exitMarker', 'left', `${((0.5 + u * k) * 100).toFixed(1)}%`);
    set('exitMarker', 'top', `${((0.5 + v * k) * 100).toFixed(1)}%`);
    set('exitMarker', 'transform', `translate(-50%, -50%) rotate(${Math.atan2(exit.dy, exit.dx).toFixed(3)}rad)`);
  }

  /**
   * A tag with the distance beside each edge-of-screen marker. `list` holds, per marker, where the
   * thing it points at is from the screen centre (`dx`, `dy`) and from the player (`dist`), in world
   * units; `view` is the visible area in the same units. A tag sits on the line from the centre to
   * the thing, further in than the marker.
   */
  function pointers(list, view) {
    const box = $('pointerTags');
    while (box.children.length < list.length) {
      const tag = document.createElement('div');
      tag.className = 'pointer-tag';
      box.append(tag);
    }
    for (let i = 0; i < box.children.length; i++) {
      const tag = box.children[i];
      const p = list[i];
      tag.hidden = !p;
      if (!p) continue;
      // World units from the screen edge. The markers sit 36 to 52 in; the top is kept clear of the HUD.
      const inset = 100;
      const hw = Math.max(1, view.w / 2 - inset);
      const hh = Math.max(1, view.h / 2 - (p.dy < 0 ? inset + 50 : inset));
      const k = Math.min(1, hw / Math.max(Math.abs(p.dx), 0.001), hh / Math.max(Math.abs(p.dy), 0.001));
      tag.style.left = `${((0.5 + (p.dx * k) / view.w) * 100).toFixed(2)}%`;
      tag.style.top = `${((0.5 + (p.dy * k) / view.h) * 100).toFixed(2)}%`;
      const text = String(Math.round(p.dist / 10) * 10);
      if (tag.textContent !== text) tag.textContent = text;
    }
  }

  /**
   * Draw the HUD. The values after `progress` belong to single missions and are ignored in the others.
   * `artElements` holds, per art id, the id of the element the art carries (0 for none); `guard` and
   * `guardMax` are the metal guard and its cap, `regrow` the seconds of wood healing left.
   */
  function hud({
    hp, maxHp, xp, xpNext, level, time, kills, enemies, upgrades, mission, goal, progress,
    carried, escape, escortHp, escortMaxHp, pointHp, pointMax, exit, bossWait, accept,
    artElements = [], artRoutes = [], artSteps = [], artOrders = [], routesOn = false, artKinds = [], artQualities = [],
    guard = 0, guardMax = 0, regrow = 0, commission = null, notice = null,
  }) {
    set('hpFill', 'width', `${Math.max(0, Math.min(100, (hp / maxHp) * 100)).toFixed(1)}%`);
    set('hpText', 'text', String(Math.ceil(hp)));
    // The metal guard: a strip under the health and its points beside the number.
    const guarded = guard > 0 && guardMax > 0;
    set('guardFill', 'display', guarded ? '' : 'none');
    set('guardText', 'display', guarded ? '' : 'none');
    if (guarded) {
      set('guardFill', 'width', `${Math.max(0, Math.min(100, (guard / guardMax) * 100)).toFixed(1)}%`);
      set('guardText', 'text', `+${Math.ceil(guard)}`);
    }
    // Wood's slow healing.
    set('regrowMark', 'display', regrow > 0 ? '' : 'none');
    set('xpFill', 'width', `${Math.max(0, Math.min(100, (xp / xpNext) * 100)).toFixed(1)}%`);
    set('levelText', 'text', t('hud.level', { n: level }));
    // A timed mission counts down to its goal; endless counts up.
    const timed = mission.id === MISSION_IDS.survive;
    const is = (name) => mission.id === MISSION_IDS[name];
    // The clock tag: hold shows time held / needed, escape the distance left to the exit.
    const holding = is('hold');
    const escaping = is('escape');
    let clock = formatTime(timed ? Math.ceil(goal - progress) : time);
    if (holding) clock = `${formatTime(Math.floor(progress))} / ${formatTime(goal)}`;
    else if (escaping) clock = String(Math.ceil(exit.dist / 10) * 10);
    set('timeText', 'text', clock);
    // The small label beside the clock says what the mission is counting.
    const boss = is('boss');
    const waiting = boss && bossWait > 0;
    let label = '';
    if (timed || holding || escaping) {
      label = t(`mission.${mission.name}.hud`);
    } else if (is('formation')) {
      label = t('mission.formation.hud', { n: progress, goal });
    } else if (is('arena')) {
      // The wave, and how many enemies of it are left.
      const wave = t('mission.arena.hud', { n: Math.min(progress + 1, goal), goal });
      label = enemies > 0 ? `${wave} · ${t('mission.arena.left', { n: enemies })}` : wave;
    } else if (is('retrieval')) {
      label = t('mission.retrieval.hud', { n: progress, goal, c: carried });
    } else if (is('pursuit')) {
      // What is left of the target's health, and how far along its route it is.
      label = t('mission.pursuit.hud', { left: Math.round(100 * (1 - progress / goal)), exit: Math.floor(100 * escape) });
    } else if (waiting) {
      label = t('hud.bossIn', { time: formatTime(Math.ceil(bossWait)) });
    } else if (boss) {
      label = t('hud.boss', { hp: Math.max(0, Math.ceil(goal - progress)) });
    }
    // A commission in endless: what it asks, and how long is left.
    const offered = commission?.phase === 1;
    const hunt = commission?.kind === 2;
    const escort = commission?.kind === 3;
    // The village, or the villager being escorted: either has a health bar.
    const village = commission?.phase === 2 && !hunt;
    const left = commission ? formatTime(Math.ceil(commission.left)) : '';
    if (offered) {
      label = commission.held > 0
        ? t('commission.accept', { n: Math.ceil(commission.need - commission.held) })
        : t(hunt ? 'commission.offerHunt' : escort ? 'commission.offerEscort' : 'commission.offer', { time: left });
    } else if (village && escort) {
      label = t('commission.escort', { n: Math.floor(commission.walked * 100), time: left });
    } else if (village) {
      label = t('commission.defend', { time: left });
    } else if (commission?.phase === 2) {
      label = t('commission.hunt', { n: commission.band, time: left });
    }
    // A notice is a text key, or { key, vars } for a line that names something.
    if (notice) label = typeof notice === 'string' ? t(notice) : t(notice.key, notice.vars);
    // Before the mission is accepted, the label says what to do: go to the circle, then stay in it.
    if (accept) {
      label = accept.held > 0 ? t('hud.accept', { n: Math.ceil(accept.need - accept.held) }) : t('hud.acceptGo');
    }
    set('goalText', 'text', label);
    set('goalText', 'display', label ? '' : 'none');
    // The shrine's health in the hold mission, or the village's while a commission defends it.
    set('pointRow', 'display', holding || village ? '' : 'none');
    if (holding || village) {
      const [hpNow, hpMax] = holding ? [pointHp, pointMax] : [commission.hp, commission.max];
      set('pointLabel', 'text', t(holding ? 'hud.shrine' : escort ? 'hud.villager' : 'hud.village'));
      set('pointFill', 'width', `${Math.max(0, Math.min(100, (hpNow / hpMax) * 100)).toFixed(1)}%`);
      set('pointText', 'text', String(Math.ceil(hpNow)));
    }
    exitMarker(escaping ? exit : null);
    // The boss's health bar, once it has arrived.
    set('bossBar', 'display', boss && !waiting ? '' : 'none');
    set('bossFill', 'width', `${Math.max(0, Math.min(100, (goal ? (goal - progress) / goal : 0) * 100)).toFixed(1)}%`);
    // A kill-count mission shows how many are left to go.
    const counted = mission.id === MISSION_IDS.kills;
    // The escort mission also shows the villager's health and how far it has walked.
    const escorting = mission.id === MISSION_IDS.escort;
    set('escortTag', 'display', escorting ? '' : 'none');
    if (escorting) {
      set('escortFill', 'width', `${Math.max(0, Math.min(100, (escortHp / escortMaxHp) * 100)).toFixed(1)}%`);
      set('escortText', 'text', `${Math.floor((progress / goal) * 100)}%`);
    }
    // The vendetta mission counts only its marked enemies.
    const marked = mission.id === MISSION_IDS.vendetta;
    const killsText = counted ? `${Math.min(kills, goal)} / ${goal}` : String(kills);
    set('killsText', 'text', marked ? `${progress} / ${goal}` : killsText);
    const sig = `${getLang()}${routesOn}${artKinds.join(',')}${artQualities.join(',')}${upgrades.join(',')}|${artElements.join(',')}|${artRoutes.join(',')}|${artSteps.join(',')}|${artOrders.join(',')}`;
    if (cache.chips !== sig) {
      cache.chips = sig;
      $('chips').replaceChildren(...upgrades.flatMap((lvl, id) => {
        if (!lvl) return [];
        const chip = document.createElement('div');
        chip.className = 'chip';
        const art = artOfUpgrade(id);
        chip.title = !art && routesOn ? t(`inner.${UPGRADES[id].name}.name`) : t(`upgrade.${id}.name`);
        // An art in a weapon kind shows the kind's mark and name.
        const kind = art ? WEAPONS[(artKinds[art.id] || 0) - 1] : null;
        const quality = art ? artQualities[art.id] || 0 : 0;
        if (kind) chip.title = `${t(`quality.${quality}`)}${t(`weapon.${kind.name}`)} · ${chip.title}`;
        if (quality === 1) chip.classList.add('chip-steel');
        if (quality === 2) chip.classList.add('chip-famous');
        const glyph = document.createElement('b');
        glyph.textContent = kind ? kind.glyph : UPGRADES[id].glyph;
        chip.append(glyph, String(lvl));
        // An art that carries an element wears its colour and a small mark of it.
        const element = art ? elementById(artElements[art.id] || 0) : null;
        if (element) {
          chip.classList.add('chip-element');
          chip.style.setProperty('--element', elementCss(element));
          chip.title += ` · ${t(`element.${element.name}.name`)}`;
          const mark = document.createElement('i');
          mark.textContent = element.glyph;
          chip.append(mark);
        }
        // An art on a route names it, and shows its move order as dots, the move just used lit.
        const route = art ? ROUTES[(artRoutes[art.id] || 0) - 1] : null;
        if (route) {
          chip.title = `${t(`route.${route.name}.name`)} · ${chip.title}`;
          const order = artOrders[art.id] || 0;
          if (order > 0) {
            const dots = document.createElement('span');
            dots.className = 'beats';
            const lit = ((artSteps[art.id] || 0) + order - 1) % order;
            for (let k = 0; k < order; k++) {
              const dot = document.createElement('em');
              if (k === lit) dot.className = 'on';
              dots.append(dot);
            }
            chip.append(dots);
          }
        }
        return [chip];
      }));
    }
  }

  function showHud(visible) {
    $('hud').hidden = !visible;
  }

  /**
   * The level-up screen. `choices` holds the three offered cards: an upgrade id, an element card
   * (`UPGRADES.length + element id - 1`) or -1 for an empty slot; `upgrades` the upgrade levels.
   * While `attach` is an element id, the screen instead asks which held art gets that element;
   * `artElements` holds, per art id, the element the art carries now (0 for none).
   */
  function levelUp(choices, upgrades, {
    attach = 0, artElements = [], routePick = 0, artRoutes = [], routesOn = false,
    deckPick = false, deckOpen = [], lastDeck = 0, ranks = [], artKinds = [], manuals = 0,
  } = {}) {
    const banner = $('levelBanner');
    const attaching = elementById(attach);
    const picking = ARTS[routePick - 1];
    if (deckPick) {
      delete banner.dataset.i18n;
      setText(banner, t('levelup.deck'));
      $('cards').replaceChildren(...deckCards(deckOpen));
      show('levelup');
      // The deck drawn from last is offered first.
      const last = document.querySelector(`#cards .card[data-value="${lastDeck}"]:not([disabled])`);
      if (last) last.focus({ preventScroll: true });
      return;
    }
    if (picking) {
      delete banner.dataset.i18n;
      setText(banner, t('levelup.route', { name: t(`upgrade.${picking.upgrade}.name`) }));
      $('cards').replaceChildren(...routeCards(choices, picking, artKinds[picking.id] || 0, manuals));
    } else if (attaching) {
      // The banner names the element, so it is not static text for applyStatic to reset.
      delete banner.dataset.i18n;
      setText(banner, t('levelup.attach', {
        name: t(`element.${attaching.name}.name`), glyph: attaching.glyph,
      }));
      $('cards').replaceChildren(...attachCards(upgrades, artElements));
    } else {
      banner.dataset.i18n = 'levelup.banner';
      setText(banner, t('levelup.banner'));
      $('cards').replaceChildren(...offerCards(choices, upgrades, artRoutes, routesOn, artKinds, ranks));
      // A fine card brings two layers at once: it says so and shines. (A weapon card's rank is its quality.)
      ranks.forEach((rank, slot) => {
        const card = document.querySelector(`#cards .card[data-value="${slot}"]`);
        if (!card || rank !== 1 || card.classList.contains('weapon-card')) return;
        card.classList.add('fine-card');
        const label = card.querySelector('.rank');
        if (label) label.textContent = t('levelup.fine');
      });
    }
    show('levelup');
  }

  /** One card per deck, in deck order; a deck with nothing to draw is greyed out. A card's value is its deck. */
  function deckCards(deckOpen) {
    const GLYPHS = ['兵', '武', '心', '法', '緣'];
    return GLYPHS.map((glyph, d) => {
      const card = document.createElement('button');
      card.className = 'card deck-card';
      card.dataset.action = 'choose';
      card.dataset.value = String(d);
      if (!deckOpen[d]) card.disabled = true;
      cardParts(card, [
        ['key', String(d + 1)],
        ['glyph', glyph],
        ['name', t(`deck.${d}.name`)],
        ['desc', t(`deck.${d}.desc`)],
      ]);
      return card;
    });
  }

  /** One card per route the art can take; one that suits the art's weapon kind says so. A card's value is its slot. */
  function routeCards(choices, art, kind = 0, manuals = 0) {
    return choices.flatMap((id, slot) => {
      const route = ROUTES[id];
      if (id < 0 || !route) return [];
      const card = document.createElement('button');
      card.className = 'card route-card';
      card.dataset.action = 'choose';
      card.dataset.value = String(slot);
      const school = SCHOOLS[route.school];
      const n = route.moves.length - (route.ultimate ? 1 : 0);
      const moves = t(route.ultimate ? 'levelup.movesUltimate' : 'levelup.moves', { n });
      const suits = route.suits.map((w) => t(`weapon.${WEAPONS[w].name}`)).join('、');
      const oneness = kind > 0 && route.suits.includes(kind - 1);
      if (oneness) card.classList.add('fine-card');
      // A stolen route opens two moves, and one more per manual (秘笈).
      const open = movesOpen(route, manuals);
      const shut = open < route.moves.length ? `\n${t('levelup.shut', { n: open })}` : '';
      cardParts(card, [
        ['key', String(slot + 1)],
        ['glyph', UPGRADES[art.upgrade].glyph],
        ['name', t(`route.${route.name}.name`)],
        ['rank', `${school ? t(`school.${school.name}`) : ''} · ${moves}${oneness ? ` · ${t('levelup.oneness')}` : ''}`],
        ['desc', `${t(`route.${route.name}.desc`)}${suits ? `\n${t('levelup.suits', { routes: suits })}` : ''}${shut}`],
      ]);
      return [card];
    });
  }

  /**
   * What an art's next level brings, when it is on a route or about to choose one:
   * [name, rank, desc] for its card, or null to show the plain upgrade card.
   */
  function routeCardText(art, lvl, artRoutes, routesOn) {
    const route = ROUTES[(artRoutes[art.id] || 0) - 1];
    if (!route) {
      const waiting = routesOn && lvl === 2 && ROUTES.some((r) => r.art === art.id && r.start);
      return waiting ? [t(`upgrade.${art.upgrade}.name`), t('levelup.chooseRoute'), t(`upgrade.${art.upgrade}.desc`)] : null;
    }
    const move = MOVES[route.moves[lvl - 2]];
    if (!move) return null;
    const regular = route.moves.length - (route.ultimate ? 1 : 0);
    const learn = t(move.ultimate ? 'levelup.ultimate' : 'levelup.learn', { name: t(`move.${move.name}.name`) });
    let formed = !move.ultimate && lvl - 2 === regular - 1 ? `\n${t('levelup.formed')}` : '';
    // The ultimate's card says what it needs to reach its height (極致).
    if (move.ultimate && route.zenith >= 0) {
      formed += `\n${t('levelup.zenith', { inner: t(`inner.${UPGRADES[route.zenith].name}.name`) })}`;
    }
    return [
      t(`route.${route.name}.name`),
      t('levelup.rank', { from: lvl, to: lvl + 1 }),
      `${learn}${formed}\n${t(`move.${move.name}.desc`)}`,
    ];
  }

  /**
   * The martial record (武學錄): per art, every route with its school and whether it is known,
   * stolen or not yet, its manuals, and its moves: a learned move shows its name and its line,
   * one not yet learned only the outline of its name. A manual kept from a run can be given
   * here to a stolen route of its school; `onGive` then keeps the save.
   */
  function record(save, onGive = () => {}) {
    const learned = new Set(Array.isArray(save.learned) ? save.learned : []);
    const stolen = Number(save.stolen) || 0;
    const manuals = Number(save.manuals) || 0;
    const el = (tag, cls, text) => {
      const node = document.createElement(tag);
      if (cls) node.className = cls;
      if (text !== undefined) node.textContent = text;
      return node;
    };
    const hidden = (text) => '？'.repeat([...text].length);
    const parts = ARTS.flatMap((art) => {
      const routes = ROUTES.filter((r) => r.art === art.id);
      if (!routes.length) return [];
      const head = el('h3', 'record-art', `${UPGRADES[art.upgrade].glyph} ${t(`upgrade.${art.upgrade}.name`)}`);
      const entries = routes.map((route) => {
        const known = route.start || (stolen & (1 << route.id)) !== 0;
        const school = SCHOOLS[route.school];
        const box = el('div', known ? 'record-route' : 'record-route unknown');
        const title = el('div', 'record-route-title');
        const name = t(`route.${route.name}.name`);
        title.append(el('b', '', known ? name : hidden(name)), el('span', 'record-tag', school ? t(`school.${school.name}`) : ''));
        const got = route.moves.filter((m) => learned.has(m)).length;
        let status = route.start ? t('record.native') : known ? t('record.stolen') : t('record.notYet');
        if (!route.start && known) {
          status += ` · ${t('record.manuals', { n: movesOpen(route, manuals), m: route.moves.length })}`;
        }
        title.append(el('span', 'record-tag', `${status} · ${t('record.count', { n: got, m: route.moves.length })}`));
        box.append(title);
        // A manual of the route's school, not yet given: give it to this route to open one more move.
        const spare = Number(save.scrolls?.[route.school]) || 0;
        if (!route.start && known && spare > 0 && movesOpen(route, manuals) < route.moves.length && manualSlot(route) < 12) {
          const give = el('button', 'record-give', t('record.give', { n: spare }));
          give.addEventListener('click', () => {
            save.manuals = (Number(save.manuals) || 0) + 2 ** (2 * manualSlot(route));
            save.scrolls[route.school] = spare - 1;
            onGive(save);
            record(save, onGive);
          });
          box.append(give);
        }
        for (const id of route.moves) {
          const move = MOVES[id];
          const moveName = t(`move.${move.name}.name`);
          const row = el('div', learned.has(id) ? 'record-move' : 'record-move unknown');
          if (learned.has(id)) {
            row.append(el('b', move.ultimate ? 'ultimate' : '', moveName), el('span', '', t(`move.${move.name}.desc`)));
          } else {
            row.append(el('b', '', hidden(moveName)));
          }
          box.append(row);
        }
        return box;
      });
      return [head, ...entries];
    });
    $('recordBody').replaceChildren(...parts);
  }

  /**
   * Write a move's name in brush strokes over the game for a moment. `kind` is 'big' for an
   * ultimate the first time, 'ultimate' for one used again, or '' for a move. Names shown at
   * once take the free rows over the player instead of covering each other.
   */
  function callout(text, kind = '') {
    // Rows above and below the player, nearest first, clear of the top HUD.
    const ROWS = [33, 67, 25, 75];
    const taken = new Set(Array.from(document.querySelectorAll('#hud .callout'), (el) => Number(el.dataset.row)));
    let row = 0;
    while (taken.has(row) && row < ROWS.length - 1) row++;
    const el = document.createElement('div');
    el.className = `callout ${kind}`.trim();
    el.dataset.row = String(row);
    el.style.top = `${ROWS[row]}%`;
    el.textContent = text;
    $('hud').append(el);
    setTimeout(() => el.remove(), kind === 'zenith' ? 1600 : kind === 'big' ? 1400 : 1000);
  }

  /** One card per offered upgrade or element. A card's value is its slot. */
  function offerCards(choices, upgrades, artRoutes = [], routesOn = false, artKinds = [], ranks = []) {
    return choices.flatMap((id, slot) => {
      if (id < 0) return [];
      const card = document.createElement('button');
      card.className = 'card';
      card.dataset.action = 'choose';
      card.dataset.value = String(slot);
      // A weapon kind card: a new art in that kind, or a held art changed to it.
      const weapon = WEAPONS[id - UPGRADES.length - ELEMENTS.length];
      if (id >= UPGRADES.length + ELEMENTS.length && weapon) {
        const art = ARTS[weapon.art];
        const current = WEAPONS[(artKinds[art.id] || 0) - 1];
        const name = t(`weapon.${weapon.name}`);
        const rank = upgrades[art.upgrade] > 0 && current
          ? t('levelup.swap', { from: t(`weapon.${current.name}`), to: name })
          : t('levelup.newWeapon', { art: t(`upgrade.${art.upgrade}.name`) });
        const forms = weapon.forms.map((f) => t(`form.${f}`)).join('・');
        const suits = ROUTES.filter((r) => r.suits.includes(weapon.id)).map((r) => t(`route.${r.name}.name`)).join('、');
        // Its rank is its quality: fine steel shines silver, a famous blade gold.
        const quality = ranks[slot] || 0;
        card.classList.add('route-card', 'weapon-card');
        if (quality === 1) card.classList.add('steel-card');
        if (quality === 2) card.classList.add('fine-card');
        cardParts(card, [
          ['key', String(slot + 1)],
          ['glyph', weapon.glyph],
          ['name', `${t(`quality.${quality}`)}${name}`],
          ['rank', rank],
          ['desc', [
            forms,
            weapon.trait && weapon.trait !== 'none' ? t(`trait.${weapon.trait}`) : '',
            suits ? t('levelup.suits', { routes: suits }) : '',
          ].filter(Boolean).join('\n')],
        ]);
        return [card];
      }
      if (id >= UPGRADES.length) {
        const element = elementById(id - UPGRADES.length + 1);
        if (!element) return [];
        card.classList.add('element-card');
        card.style.setProperty('--element', elementCss(element));
        cardParts(card, [
          ['key', String(slot + 1)],
          ['glyph', element.glyph],
          ['name', t(`element.${element.name}.name`)],
          ['rank', t('levelup.element')],
          ['desc', t(`element.${element.name}.desc`)],
        ]);
        return [card];
      }
      const lvl = upgrades[id];
      const art = artOfUpgrade(id);
      // With routes on, a passive is an inner art (心法): its card names the layer it brings.
      if (!art && routesOn) {
        const name = UPGRADES[id].name;
        card.classList.add('route-card');
        cardParts(card, [
          ['key', String(slot + 1)],
          ['glyph', UPGRADES[id].glyph],
          ['name', t(`inner.${name}.name`)],
          ['rank', lvl === 0 ? t('levelup.new') : t('levelup.layer', { n: lvl + 1 })],
          ['desc', t(`inner.${name}.${lvl + 1}`)],
        ]);
        return [card];
      }
      const routed = art ? routeCardText(art, lvl, artRoutes, routesOn) : null;
      if (routed) {
        card.classList.add('route-card');
        cardParts(card, [
          ['key', String(slot + 1)],
          ['glyph', UPGRADES[id].glyph],
          ['name', routed[0]],
          ['rank', routed[1]],
          ['desc', routed[2]],
        ]);
        return [card];
      }
      cardParts(card, [
        ['key', String(slot + 1)],
        ['glyph', UPGRADES[id].glyph],
        ['name', t(`upgrade.${id}.name`)],
        ['rank', lvl === 0 ? t('levelup.new') : t('levelup.rank', { from: lvl, to: lvl + 1 })],
        ['desc', t(`upgrade.${id}.desc`)],
      ]);
      return [card];
    });
  }

  /** One card per held art, in art id order. A card's value is the art's id. */
  function attachCards(upgrades, artElements) {
    const held = ARTS.filter((art) => upgrades[art.upgrade] > 0);
    return held.map((art, i) => {
      const card = document.createElement('button');
      card.className = 'card art-card';
      card.dataset.action = 'choose';
      card.dataset.value = String(art.id);
      const current = elementById(artElements[art.id] || 0);
      if (current) {
        card.classList.add('element-card');
        card.style.setProperty('--element', elementCss(current));
      }
      cardParts(card, [
        ['key', String(i + 1)],
        ['glyph', UPGRADES[art.upgrade].glyph],
        ['name', t(`upgrade.${art.upgrade}.name`)],
        ['rank', current ? t('levelup.replaces', { name: t(`element.${current.name}.name`) }) : t('levelup.empty')],
      ]);
      return card;
    });
  }

  function statRows(id, rows) {
    $(id).replaceChildren(...rows.flatMap(([label, value, best]) => {
      const dt = document.createElement('dt');
      dt.textContent = label;
      const dd = document.createElement('dd');
      dd.textContent = value;
      if (best) dd.className = 'best';
      return [dt, dd];
    }));
  }

  /**
   * The run was lost. `beaten` says which endless records fell; null for other missions.
   * `reason` is the text key of the title when the run ended without the player dying
   * (the pursuit target got away, the villager or the shrine fell); null for death.
   */
  function gameOver(mission, { time, kills, level, commissions = 0 }, save, beaten, reason = null) {
    const title = $('overTitle');
    title.dataset.i18n = reason || 'over.title';
    setText(title, t(title.dataset.i18n));
    const rows = [
      [t('over.survived'), formatTime(time), beaten?.time],
      [t('over.kills'), String(kills), beaten?.kills],
      [t('over.level'), String(level), beaten?.level],
    ];
    // Endless has commissions; the run's count of done ones.
    if (mission.id === MISSION_IDS.endless) rows.push([t('over.commissions'), String(commissions), false]);
    if (beaten) rows.push([t('over.bestTime'), formatTime(save.bestTime), false]);
    else rows.unshift([t('over.mission'), t(`mission.${mission.name}.name`), false]);
    statRows('stats', rows);
    $('recordStamp').hidden = !(beaten && (beaten.time || beaten.kills || beaten.level));
    resetReplayButtons();
    show('over');
  }

  /** The mission's goal was met. */
  function missionWon(mission, { time, kills, level }, save) {
    $('wonMission').textContent = t(`mission.${mission.name}.name`);
    statRows('wonStats', [
      [t('won.time'), formatTime(time), false],
      [t('over.kills'), String(kills), false],
      [t('over.level'), String(level), false],
      [t('won.wins'), String(missionRecord(save, mission.name).wins), false],
    ]);
    resetReplayButtons();
    show('won');
  }

  /** The mission buttons on the title screen: goal missions first, endless last. */
  function missions(save) {
    const order = [...MISSIONS.filter((m) => m.id !== MISSION_IDS.endless), MISSIONS[MISSION_IDS.endless]];
    const focused = document.activeElement?.dataset?.value;
    $('missions').replaceChildren(...order.map((mission) => {
      const btn = document.createElement('button');
      btn.className = 'mission';
      btn.dataset.action = 'setup';
      btn.dataset.value = String(mission.id);
      const name = document.createElement('span');
      name.textContent = t(`mission.${mission.name}.name`);
      const desc = document.createElement('small');
      const { wins } = missionRecord(save, mission.name);
      desc.textContent = t(`mission.${mission.name}.desc`, missionVars(mission))
        + (wins ? ` · ${t('mission.cleared', { n: wins })}` : '');
      btn.append(name, desc);
      return btn;
    }));
    // Rebuilding drops focus; put it back for keyboard and gamepad players.
    if (current === 'title') {
      const again = [...$('missions').children].find((b) => b.dataset.value === focused);
      (again || $('missions').firstElementChild).focus({ preventScroll: true });
    }
  }

  /** A tune's value as text: as many decimals as its step has. */
  function tuneText(tune, value) {
    const decimals = (String(tune.step).split('.')[1] || '').length;
    return value.toFixed(decimals);
  }

  /**
   * The screen before a run of `mission`: its description and one stepper per tune.
   * `rows` is a list of { tune, value }; `goal` is the mission's goal with the tunes applied.
   */
  function setup(mission, rows, goal) {
    $('setupTitle').textContent = t(`mission.${mission.name}.name`);
    $('setupDesc').textContent = t(`mission.${mission.name}.desc`, missionVars({ ...mission, goal }));
    const focused = document.activeElement?.dataset?.value;
    $('tunes').replaceChildren(...rows.map(({ tune, value }) => {
      const row = document.createElement('div');
      row.className = 'setting';
      const label = document.createElement('span');
      label.textContent = t(`tune.${tune.name}`);
      const stepper = document.createElement('div');
      stepper.className = 'stepper';
      const shown = document.createElement('b');
      shown.className = value === tune.def ? 'tune-value' : 'tune-value changed';
      shown.textContent = tuneText(tune, value);
      const step = (dir, sign, key) => {
        const btn = document.createElement('button');
        btn.dataset.action = 'tune';
        btn.dataset.value = `${tune.id}:${dir}`;
        btn.textContent = sign;
        btn.setAttribute('aria-label', t(key, { name: label.textContent }));
        return btn;
      };
      stepper.append(step(-1, '−', 'setup.lower'), shown, step(1, '+', 'setup.raise'));
      row.append(label, stepper);
      return row;
    }));
    if (current !== 'setup') {
      show('setup');
    } else if (focused) {
      // Rebuilding drops focus; put it back on the stepper button that was pressed.
      const again = [...$('tunes').querySelectorAll('button')].find((b) => b.dataset.value === focused);
      if (again) again.focus({ preventScroll: true });
    }
  }

  function records(save) {
    missions(save);
    $('records').textContent = save.runs
      ? t(save.runs === 1 ? 'records.one' : 'records.other', {
        time: formatTime(save.bestTime), kills: save.bestKills, level: save.bestLevel, runs: save.runs,
      })
      : '';
  }

  function settings({ volume, muted, numbers, motion, music, quality }) {
    const on = Math.round(volume * 10);
    $('volPips').replaceChildren(...Array.from({ length: 10 }, (_, i) => {
      const pip = document.createElement('i');
      if (i < on) pip.className = 'on';
      return pip;
    }));
    for (const btn of document.querySelectorAll('button[data-action="mute"]')) {
      btn.textContent = t(muted ? 'settings.soundOff' : 'settings.soundOn');
    }
    $('numbersBtn').textContent = t(numbers ? 'settings.numbersOn' : 'settings.numbersOff');
    $('motionBtn').textContent = t(motion ? 'settings.motionOn' : 'settings.motionOff');
    const level = music === true ? 1 : music === false ? 0 : Number(music) || 0;
    $('musicBtn').textContent = t(level >= 1 ? 'settings.musicOn' : level > 0 ? 'settings.musicSoft' : 'settings.musicOff');
    $('qualityBtn').textContent = t(quality ? 'settings.qualityHigh' : 'settings.qualityLow');
    setText($('langBtn'), t('settings.language', { name: t('lang.name') }));
  }

  /** Redden the edges of the screen while health is low. */
  function lowHealth(on) {
    if (cache.low === on) return;
    cache.low = on;
    document.body.classList.toggle('low-health', on);
  }

  /** Flash the experience bar on a level-up. */
  function flashLevel() {
    const bar = document.querySelector('#hud .xp');
    bar.classList.remove('flash');
    // Restart the animation even when the last one has not finished.
    void bar.offsetWidth;
    bar.classList.add('flash');
  }

  /** Mark the HUD while a replay plays. */
  function replaying(on) {
    $('replayTag').hidden = !on;
  }

  /** Say on the replay buttons whether the link was copied. */
  function replayCopied(ok) {
    for (const btn of document.querySelectorAll('button[data-action="copy-replay"]')) {
      btn.dataset.i18n = ok ? 'replay.copied' : 'replay.failed';
      setText(btn, t(btn.dataset.i18n));
    }
  }

  /** Put the replay buttons back to their first text. */
  function resetReplayButtons() {
    for (const btn of document.querySelectorAll('button[data-action="copy-replay"]')) {
      btn.dataset.i18n = 'replay.copy';
      setText(btn, t('replay.copy'));
    }
  }

  function error(message) {
    $('errorText').textContent = message;
    show('error');
  }

  return {
    show, navigate, confirm, hud, pointers, showHud, levelUp, gameOver, missionWon, records, setup, settings, error,
    replaying, replayCopied, lowHealth, flashLevel, callout, record,
    get screen() { return current; },
    get locked() { return performance.now() < lockedUntil; },
  };
}
