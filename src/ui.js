// Paper UI: screens, HUD and menu focus. All DOM work lives here.
import { MISSIONS, MISSION_IDS, UPGRADES, ARTS, ELEMENTS } from './data.js';
import { formatTime, missionRecord } from './save.js';
import { t, getLang, setText } from './i18n.js';

const $ = (id) => document.getElementById(id);
const SCREENS = ['title', 'setup', 'settings', 'levelup', 'pause', 'over', 'won', 'error'];

/** An element's colour as CSS. `color` is 0..1 RGB. */
const elementCss = (element) => `rgb(${element.color.map((c) => Math.round(c * 255)).join(', ')})`;
/** The element with this id, or null for 0 (no element). Ids start at 1. */
const elementById = (id) => ELEMENTS[id - 1] || null;
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
    artElements = [], guard = 0, guardMax = 0, regrow = 0,
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
    // Before the mission is accepted, the label says what to do: go to the circle, then stay in it.
    if (accept) {
      label = accept.held > 0 ? t('hud.accept', { n: Math.ceil(accept.need - accept.held) }) : t('hud.acceptGo');
    }
    set('goalText', 'text', label);
    set('goalText', 'display', label ? '' : 'none');
    // The shrine's health, in the hold mission only.
    set('pointRow', 'display', holding ? '' : 'none');
    if (holding) {
      set('pointFill', 'width', `${Math.max(0, Math.min(100, (pointHp / pointMax) * 100)).toFixed(1)}%`);
      set('pointText', 'text', String(Math.ceil(pointHp)));
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
    const sig = `${getLang()}${upgrades.join(',')}|${artElements.join(',')}`;
    if (cache.chips !== sig) {
      cache.chips = sig;
      $('chips').replaceChildren(...upgrades.flatMap((lvl, id) => {
        if (!lvl) return [];
        const chip = document.createElement('div');
        chip.className = 'chip';
        chip.title = t(`upgrade.${id}.name`);
        const glyph = document.createElement('b');
        glyph.textContent = UPGRADES[id].glyph;
        chip.append(glyph, String(lvl));
        // An art that carries an element wears its colour and a small mark of it.
        const art = artOfUpgrade(id);
        const element = art ? elementById(artElements[art.id] || 0) : null;
        if (element) {
          chip.classList.add('chip-element');
          chip.style.setProperty('--element', elementCss(element));
          chip.title += ` · ${t(`element.${element.name}.name`)}`;
          const mark = document.createElement('i');
          mark.textContent = element.glyph;
          chip.append(mark);
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
  function levelUp(choices, upgrades, { attach = 0, artElements = [] } = {}) {
    const banner = $('levelBanner');
    const attaching = elementById(attach);
    if (attaching) {
      // The banner names the element, so it is not static text for applyStatic to reset.
      delete banner.dataset.i18n;
      setText(banner, t('levelup.attach', {
        name: t(`element.${attaching.name}.name`), glyph: attaching.glyph,
      }));
      $('cards').replaceChildren(...attachCards(upgrades, artElements));
    } else {
      banner.dataset.i18n = 'levelup.banner';
      setText(banner, t('levelup.banner'));
      $('cards').replaceChildren(...offerCards(choices, upgrades));
    }
    show('levelup');
  }

  /** One card per offered upgrade or element. A card's value is its slot. */
  function offerCards(choices, upgrades) {
    return choices.flatMap((id, slot) => {
      if (id < 0) return [];
      const card = document.createElement('button');
      card.className = 'card';
      card.dataset.action = 'choose';
      card.dataset.value = String(slot);
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
  function gameOver(mission, { time, kills, level }, save, beaten, reason = null) {
    const title = $('overTitle');
    title.dataset.i18n = reason || 'over.title';
    setText(title, t(title.dataset.i18n));
    const rows = [
      [t('over.survived'), formatTime(time), beaten?.time],
      [t('over.kills'), String(kills), beaten?.kills],
      [t('over.level'), String(level), beaten?.level],
    ];
    if (beaten) rows.push([t('over.bestTime'), formatTime(save.bestTime), false]);
    else rows.unshift([t('over.mission'), t(`mission.${mission.name}.name`), false]);
    statRows('stats', rows);
    $('recordStamp').hidden = !(beaten && (beaten.time || beaten.kills || beaten.level));
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

  function settings({ volume, muted }) {
    const on = Math.round(volume * 10);
    $('volPips').replaceChildren(...Array.from({ length: 10 }, (_, i) => {
      const pip = document.createElement('i');
      if (i < on) pip.className = 'on';
      return pip;
    }));
    for (const btn of document.querySelectorAll('button[data-action="mute"]')) {
      btn.textContent = t(muted ? 'settings.soundOff' : 'settings.soundOn');
    }
    setText($('langBtn'), t('settings.language', { name: t('lang.name') }));
  }

  function error(message) {
    $('errorText').textContent = message;
    show('error');
  }

  return {
    show, navigate, confirm, hud, pointers, showHud, levelUp, gameOver, missionWon, records, setup, settings, error,
    get screen() { return current; },
    get locked() { return performance.now() < lockedUntil; },
  };
}
