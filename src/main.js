// Host: loads the wasm core, runs the frame loop and ties input, sound,
// saving and the UI together. The game rules live in core/ (Rust).
import { createRenderer } from './renderer.js';
import { createRendererGL } from './renderer-gl.js';
import { createInput } from './input.js';
import { createAudio } from './audio.js';
import { createUI } from './ui.js';
import { loadSave, writeSave, recordRun, recordMission, clearRecords, tuneValue, setTune } from './save.js';
import {
  MISSIONS, MISSION_IDS, TUNES, STATE, STATE_FLOATS, UPGRADES, ARTS, MOVES, ROUTES, SCHOOLS, EVENT, KINDS,
} from './data.js';
import { pickLang, setLang, getLang, nextLang, t } from './i18n.js';
import { packReplay, unpackReplay } from './replay.js';

async function boot() {
  let canvas = document.getElementById('game');
  const save = loadSave();
  setLang(pickLang(save.lang));
  const audio = createAudio();
  audio.setVolume(save.volume);
  audio.setMuted(save.muted);
  audio.setMusic(save.music);

  // 'title' | 'playing' | 'levelup' | 'paused' | 'over' | 'won'
  let mode = 'title';
  // The mission of the current or last run; "again" and "restart" replay it.
  let mission = MISSIONS[MISSION_IDS.endless];
  // True while the run has a tune that is not at its default. Such a run is not recorded.
  let testRun = false;
  // The seed, mission name and tunes of the current run: with the core's log, its replay.
  let run = null;
  // True while a replay from a link plays.
  let replaying = false;
  let settingsFrom = 'title';
  // Seconds the game stands still after a heavy blow (hit-stop).
  let hitstop = 0;
  // Seconds until the next heartbeat while health is low.
  let heartbeat = 0;
  // A line the HUD shows for a few seconds, such as a commission's outcome: { key, until }.
  let notice = null;
  // The route moves already called out this run: a move's name is written the first time it is used.
  let calledOut = new Set();

  const ui = createUI({ onAction });

  let renderer, core;
  try {
    renderer = await pickRenderer();
    const bytes = await (await fetch('wasm/jadereaper_core.wasm', { cache: 'no-cache' })).arrayBuffer();
    core = (await WebAssembly.instantiate(bytes, {})).instance.exports;
  } catch (err) {
    console.error(err);
    ui.error(err.message || String(err));
    return;
  }

  // Made after the renderer, which may have swapped the canvas for a fresh one.
  const input = createInput({ onKey, surface: canvas });

  /**
   * WebGPU when the browser has it, else WebGL2 with the same look. `?renderer=webgl2`
   * asks for WebGL2 even where WebGPU works, to try it. When both fail, the
   * WebGPU reason is the one shown.
   */
  async function pickRenderer() {
    const forced = new URLSearchParams(location.search).get('renderer') === 'webgl2';
    let gpuError = null;
    if (!forced) {
      try {
        return await createRenderer(canvas);
      } catch (err) {
        gpuError = err;
      }
    }
    // A canvas that has handed out a WebGPU context cannot give a WebGL2 one; start from a fresh one.
    const fresh = canvas.cloneNode(false);
    canvas.replaceWith(fresh);
    canvas = fresh;
    try {
      const gl = await createRendererGL(canvas);
      if (gpuError) console.info('WebGPU is not available here, so the game draws with WebGL2:', gpuError.message);
      return gl;
    } catch (err) {
      throw gpuError || err;
    }
  }

  const seed = () => (Math.random() * 0xffffffff) >>> 0;
  // Views are rebuilt on every read because wasm memory can grow and detach them.
  const readState = () => new Float32Array(core.memory.buffer, core.jr_state_ptr(), STATE_FLOATS);
  const upgradeLevels = (s) => Array.from(s.subarray(STATE.UPGRADE0, STATE.UPGRADE0 + UPGRADES.length));
  // Per art id, the id of the element the art carries (0 for none).
  const artElements = (s) => Array.from(s.subarray(STATE.ART_ELEMENT0, STATE.ART_ELEMENT0 + ARTS.length));
  // Per art id: the route it follows (route id + 1, 0 for none), where it is in its move order, and how long that is.
  const artRoutes = (s) => Array.from(s.subarray(STATE.ART_ROUTE0, STATE.ART_ROUTE0 + ARTS.length));
  const artSteps = (s) => Array.from(s.subarray(STATE.ART_STEP0, STATE.ART_STEP0 + ARTS.length));
  const artOrders = (s) => Array.from(s.subarray(STATE.ART_ORDER0, STATE.ART_ORDER0 + ARTS.length));
  // Per art id, the weapon kind it is (weapon id + 1, 0 for none).
  const artKinds = (s) => Array.from(s.subarray(STATE.ART_KIND0, STATE.ART_KIND0 + ARTS.length));
  /** Show the level-up screen: the offered cards, the routes of an art, or the held arts while an element waits for one. */
  const showLevelUp = (s) => ui.levelUp(
    Array.from(s.subarray(STATE.CHOICE0, STATE.CHOICE0 + 3)), upgradeLevels(s),
    {
      attach: s[STATE.ATTACH], artElements: artElements(s),
      routePick: s[STATE.ROUTE_PICK], artRoutes: artRoutes(s), routesOn: s[STATE.ROUTES_ON] === 1,
      deckPick: s[STATE.DECK_PICK] === 1, deckOpen: Array.from(s.subarray(STATE.DECK_OPEN0, STATE.DECK_OPEN0 + 5)),
      lastDeck: s[STATE.LAST_DECK], ranks: Array.from(s.subarray(STATE.CHOICE_RANK0, STATE.CHOICE_RANK0 + 3)),
      artKinds: artKinds(s), manuals: s[STATE.MANUALS],
    },
  );
  /**
   * A stolen route was learned (偷師): say so, and keep it for later runs. A replay
   * plays with what was known then and changes nothing.
   */
  function onSteal(id) {
    const route = ROUTES[id];
    if (!route) return;
    const school = SCHOOLS[route.school];
    notice = {
      key: 'steal.notice',
      vars: { name: t(`route.${route.name}.name`), school: school ? t(`school.${school.name}`) : '' },
      until: clock + 4,
    };
    audio.play('learn');
    if (!replaying) {
      save.stolen = (Number(save.stolen) || 0) | (1 << id);
      writeSave(save);
    }
  }

  /** A master dropped a manual (秘笈): say which route opened a move, and keep all of them for later runs. */
  function onManual(id, manuals) {
    const route = ROUTES[id];
    if (!route) return;
    notice = { key: 'manual.notice', vars: { name: t(`route.${route.name}.name`) }, until: clock + 4 };
    audio.play('learn');
    if (!replaying) {
      save.manuals = manuals;
      writeSave(save);
    }
  }

  /** A route move was used: hit-stop for a heavy one, and its name the first time (an ultimate's every time). */
  function onMove(id) {
    const move = MOVES[id];
    if (!move) return;
    if (save.motion && move.stop) hitstop = Math.max(hitstop, move.ultimate ? 0.09 : 0.05);
    if (move.ultimate) audio.play('ultimate');
    const first = !calledOut.has(id);
    // A move used for the first time ever goes into the martial record (武學錄).
    if (first && !replaying) {
      const learned = Array.isArray(save.learned) ? save.learned : [];
      if (!learned.includes(id)) {
        save.learned = [...learned, id];
        writeSave(save);
      }
    }
    if (move.ultimate || first) {
      calledOut.add(id);
      ui.callout(t(`move.${move.name}.name`), move.ultimate ? (first ? 'big' : 'ultimate') : '');
    }
  }

  function toTitle() {
    core.jr_reset(seed());
    replaying = false;
    ui.replaying(false);
    // A replay link only plays once; reloading the title should not play it again.
    if (location.search.includes('replay=')) history.replaceState(null, '', location.pathname);
    mode = 'title';
    ui.showHud(false);
    ui.records(save);
    ui.show('title');
  }

  /** The tunes the setup screen of the current mission shows: its own, then the ones every mission has. */
  const missionTunes = () => [
    ...TUNES.filter((tune) => tune.mission === mission.name),
    ...TUNES.filter((tune) => tune.mission === 'any'),
  ];

  /** Show the setup screen of the mission with this id, or redraw it for the current mission. */
  function openSetup(id) {
    if (MISSIONS[Number(id)]) mission = MISSIONS[Number(id)];
    const rows = missionTunes().map((tune) => ({ tune, value: tuneValue(save, tune) }));
    const goal = rows.find(({ tune }) => tune.name === `${mission.name}_goal`)?.value ?? mission.goal;
    ui.setup(mission, rows, goal);
  }

  /** Move a tune one step down (-1) or up (1), staying on the steps counted from its minimum. */
  function stepTune(id, dir) {
    const tune = TUNES[id];
    if (!tune) return;
    const steps = Math.round((tuneValue(save, tune) - tune.min) / tune.step) + dir;
    const value = Math.min(tune.max, Math.max(tune.min, tune.min + steps * tune.step));
    // Whole steps of a decimal such as 0.05 pick up rounding noise; cut it off.
    setTune(save, tune, Number(value.toFixed(6)));
    openSetup();
  }

  /** Start a run of the current mission with the tunes its setup screen shows. */
  function startRun() {
    core.jr_tune_reset();
    testRun = false;
    const tunes = {};
    for (const tune of missionTunes()) {
      const value = tuneValue(save, tune);
      core.jr_tune(tune.id, value);
      tunes[tune.name] = value;
      if (value !== tune.def) testRun = true;
    }
    // Tunes the host sets itself: the stolen routes and the manuals gained in earlier runs.
    for (const name of ['stolen', 'manuals']) {
      const tune = TUNES.find((x) => x.name === name);
      if (!tune) continue;
      const value = Number(save[name]) || 0;
      core.jr_tune(tune.id, value);
      tunes[name] = value;
    }
    const s = seed();
    core.jr_start(s, mission.id);
    calledOut = new Set();
    run = { seed: s, mission: mission.name, tunes };
    replaying = false;
    ui.replaying(false);
    mode = 'playing';
    ui.showHud(true);
    ui.show(null);
  }

  /** Play the run a replay link holds. A link that cannot be read leaves the title as it is. */
  async function startReplay(text) {
    let r;
    try {
      r = await unpackReplay(text);
    } catch (err) {
      console.warn('This replay link cannot be read:', err);
      return;
    }
    const found = MISSIONS.find((m) => m.name === r.mission);
    if (!found) {
      console.warn(`This replay is of a mission this build does not have: ${r.mission}`);
      return;
    }
    mission = found;
    core.jr_tune_reset();
    for (const [name, value] of Object.entries(r.tunes)) {
      const tune = TUNES.find((t) => t.name === name);
      if (tune && Number.isFinite(value)) core.jr_tune(tune.id, value);
    }
    core.jr_start(r.seed, mission.id);
    calledOut = new Set();
    core.jr_replay_begin();
    for (let i = 0; i < r.log.length; i += 4) core.jr_replay_push(r.log[i], r.log[i + 1], r.log[i + 2], r.log[i + 3]);
    run = { seed: r.seed, mission: r.mission, tunes: r.tunes };
    // A replay is never recorded in the save.
    testRun = true;
    replaying = true;
    ui.replaying(true);
    mode = 'playing';
    ui.showHud(true);
    ui.show(null);
  }

  /** A link that plays the current run back, or null before any run. */
  async function replayLink() {
    if (!run) return null;
    const log = new Uint32Array(core.memory.buffer, core.jr_log_ptr(), core.jr_log_len() * 4).slice();
    const text = await packReplay({ ...run, log });
    return `${location.origin}${location.pathname}?replay=${text}`;
  }

  async function copyReplay() {
    const link = await replayLink();
    try {
      await navigator.clipboard.writeText(link);
      ui.replayCopied(true);
    } catch {
      ui.replayCopied(false);
    }
  }

  function setPaused(paused) {
    if (paused && mode === 'playing') {
      mode = 'paused';
      input.release();
      ui.show('pause');
    } else if (!paused && mode === 'paused') {
      mode = 'playing';
      ui.show(null);
    }
  }

  function saveSettings() {
    save.volume = audio.volume;
    save.muted = audio.muted;
    writeSave(save);
    ui.settings(save);
    applyFeel();
  }

  /** The music level after `level`: full, soft, off, and round again. Older saves hold true or false. */
  function nextMusic(level) {
    const now = level === true ? 1 : level === false ? 0 : Number(level) || 0;
    return now >= 1 ? 0.5 : now > 0 ? 0 : 1;
  }

  /** Hand the look settings to the core and the renderer: damage numbers, camera shake, quality. */
  function applyFeel() {
    core.jr_feel(save.numbers ? 1 : 0, save.motion ? 1 : 0);
    renderer.setQuality(save.quality);
  }

  function onAction(action, value) {
    audio.unlock();
    audio.play(action === 'choose' || action === 'start' ? 'confirm' : 'ui');
    switch (action) {
      case 'setup': openSetup(value); break;
      case 'tune': {
        const [id, dir] = String(value).split(':').map(Number);
        stepTune(id, dir);
        break;
      }
      case 'tune-reset':
        for (const tune of missionTunes()) setTune(save, tune, tune.def);
        openSetup();
        break;
      case 'start': startRun(); break;
      case 'copy-replay': copyReplay(); break;
      case 'resume': setPaused(false); break;
      case 'pause': setPaused(true); break;
      case 'restart': startRun(); break;
      case 'quit': toTitle(); break;
      case 'settings': settingsFrom = ui.screen; ui.show('settings'); break;
      case 'back': ui.show(settingsFrom); break;
      case 'record': ui.record(save); ui.show('record'); break;
      case 'record-back': ui.show('title'); break;
      case 'lang':
        setLang(nextLang(getLang()));
        save.lang = getLang();
        saveSettings();
        ui.records(save);
        break;
      case 'mute': audio.setMuted(!audio.muted); saveSettings(); break;
      case 'numbers': save.numbers = !save.numbers; saveSettings(); break;
      case 'motion': save.motion = !save.motion; saveSettings(); break;
      case 'music': save.music = nextMusic(save.music); audio.setMusic(save.music); saveSettings(); break;
      case 'quality': save.quality = !save.quality; saveSettings(); break;
      case 'vol-up': audio.setVolume(audio.volume + 0.1); saveSettings(); audio.play('confirm'); break;
      case 'vol-down': audio.setVolume(audio.volume - 0.1); saveSettings(); audio.play('confirm'); break;
      case 'clear-save':
        clearRecords(save);
        ui.records(save);
        break;
      case 'choose':
        if (mode === 'levelup') {
          core.jr_choose(Number(value));
          afterChoice();
        }
        break;
    }
  }

  /** The core may offer another choice straight away if several levels were gained at once. */
  function afterChoice() {
    // jr_choose reports an element put on an art, and jr_update clears events first, so play it now.
    const count = core.jr_event_count();
    if (count) {
      const ev = new Float32Array(core.memory.buffer, core.jr_events_ptr(), count * 4);
      for (let i = 0; i < count; i++) if (ev[i * 4] === EVENT.ATTACH) audio.play(EVENT.ATTACH);
    }
    core.jr_update(0);
    const s = readState();
    if (s[STATE.CHOOSING]) {
      showLevelUp(s);
    } else {
      mode = 'playing';
      ui.show(null);
    }
  }

  function back() {
    if (ui.screen === 'settings') onAction('back');
    else if (ui.screen === 'record') onAction('record-back');
    else if (ui.screen === 'setup') onAction('quit');
    else if (mode === 'paused') setPaused(false);
  }

  function onKey(key) {
    audio.unlock();
    if (key.pause) {
      if (mode === 'playing') setPaused(true);
      else back();
    } else if (key.nav && ui.screen) {
      if (ui.navigate(key.nav[0], key.nav[1])) audio.play('ui');
    } else if (key.pick !== undefined && mode === 'levelup' && !ui.locked) {
      const card = document.querySelectorAll('#cards .card')[key.pick];
      if (card) card.click();
    } else if (key.mute) {
      audio.setMuted(!audio.muted);
      saveSettings();
    }
  }

  window.addEventListener('pointerdown', () => audio.unlock());
  window.addEventListener('blur', () => setPaused(true));
  document.addEventListener('visibilitychange', () => { if (document.hidden) setPaused(true); });

  ui.settings(save);
  applyFeel();
  // Read the link before `toTitle`, which takes it out of the address bar.
  const link = new URLSearchParams(location.search).get('replay');
  toTitle();
  if (link) startReplay(link);

  let last = performance.now();
  let clock = 0;

  function frame(now) {
    const dt = Math.min(0.05, Math.max(0, (now - last) / 1000));
    last = now;

    const view = renderer.resize();
    core.jr_set_view(view.w, view.h);

    const pad = input.poll(dt);
    if (pad.pause) onKey({ pause: true });
    if (pad.back) back();
    if (ui.screen) {
      if (pad.nav && ui.navigate(pad.nav[0], pad.nav[1])) audio.play('ui');
      if (pad.confirm && !ui.locked) { audio.unlock(); ui.confirm(); }
    }

    // Paused freezes everything. Other menus keep the paper puppets moving.
    // Hit-stop holds the game still for a moment, puppets and all.
    let step = mode === 'paused' ? 0 : dt;
    if (step > 0 && hitstop > 0) {
      const held = Math.min(hitstop, step);
      hitstop -= held;
      step -= held;
    }
    clock += step;
    core.jr_input(mode === 'playing' ? pad.mx : 0, mode === 'playing' ? pad.my : 0);
    core.jr_update(step);

    const s = readState();
    if (mode === 'playing') {
      const count = core.jr_event_count();
      if (count) {
        const ev = new Float32Array(core.memory.buffer, core.jr_events_ptr(), count * 4);
        for (let i = 0; i < count; i++) {
          const type = ev[i * 4];
          audio.play(type);
          // A heavy enemy falling, or two elements reacting, stops the game for a beat.
          if (save.motion && type === EVENT.KILL && KINDS[ev[i * 4 + 3]]?.heavy) hitstop = Math.max(hitstop, 0.07);
          if (save.motion && type === EVENT.REACT) hitstop = Math.max(hitstop, 0.04);
          // The gamepad shakes with the blows the player takes and the heavy ones he lands.
          if (save.motion && type === EVENT.HURT) input.rumble(140, 0.7);
          if (save.motion && type === EVENT.KILL && KINDS[ev[i * 4 + 3]]?.heavy) input.rumble(80, 0.35);
          if (type === EVENT.LEVELUP) ui.flashLevel();
          if (type === EVENT.MOVE) onMove(ev[i * 4 + 3]);
          if (type === EVENT.STEAL) onSteal(ev[i * 4 + 3]);
          if (type === EVENT.MANUAL) onManual(ev[i * 4 + 3], s[STATE.MANUALS]);
          // A school elite winds up a move: its name in red, so the player knows what is coming.
          if (type === EVENT.FOE && MOVES[ev[i * 4 + 3]]) {
            ui.callout(t(`move.${MOVES[ev[i * 4 + 3]].name}.name`), 'foe');
            audio.play('warn');
          }
          if (type === EVENT.COMMISSION_DONE) notice = { key: 'commission.done', until: clock + 4 };
          if (type === EVENT.COMMISSION_FAILED) notice = { key: 'commission.failed', until: clock + 4 };
        }
      }
      // Low health: the edges of the screen redden, and a heartbeat sounds.
      const low = s[STATE.HP] > 0 && s[STATE.HP] < s[STATE.MAX_HP] * 0.25;
      ui.lowHealth(low);
      heartbeat -= dt;
      if (low && heartbeat <= 0) {
        audio.play('heartbeat');
        heartbeat = 0.9;
      }
      const accepting = s[STATE.ACCEPTING] === 1;
      ui.hud({
        hp: s[STATE.HP], maxHp: s[STATE.MAX_HP], xp: s[STATE.XP], xpNext: s[STATE.XP_NEXT],
        level: s[STATE.LEVEL], time: s[STATE.TIME], kills: s[STATE.KILLS], enemies: s[STATE.ENEMIES],
        // Until the mission is accepted the core plays no mission, so the HUD shows none either.
        upgrades: upgradeLevels(s), mission: accepting ? MISSIONS[MISSION_IDS.endless] : mission,
        // What the elements on the arts are doing: the metal guard and wood's slow healing.
        artElements: artElements(s), artRoutes: artRoutes(s), artSteps: artSteps(s), artOrders: artOrders(s),
        routesOn: s[STATE.ROUTES_ON] === 1, artKinds: artKinds(s),
        artQualities: Array.from(s.subarray(STATE.ART_QUALITY0, STATE.ART_QUALITY0 + ARTS.length)),
        guard: s[STATE.GUARD], guardMax: s[STATE.GUARD_MAX], regrow: s[STATE.REGROW],
        accept: accepting ? { held: s[STATE.ACCEPT_HELD], need: s[STATE.ACCEPT_NEED] } : null,
        commission: {
          phase: s[STATE.COMMISSION], left: s[STATE.COMMISSION_LEFT], need: s[STATE.COMMISSION_NEED],
          held: s[STATE.COMMISSION_HELD], hp: s[STATE.COMMISSION_HP], max: s[STATE.COMMISSION_MAX],
          kind: s[STATE.COMMISSION_KIND], band: s[STATE.COMMISSION_BAND], walked: s[STATE.COMMISSION_WALKED],
        },
        notice: notice && clock < notice.until ? (notice.vars ? notice : notice.key) : null,
        goal: s[STATE.GOAL], progress: s[STATE.PROGRESS],
        // Values of single missions; the HUD ignores them in the others.
        carried: s[STATE.CARRIED], escape: s[STATE.ESCAPE], bossWait: s[STATE.BOSS_WAIT],
        escortHp: s[STATE.ESCORT_HP], escortMaxHp: s[STATE.ESCORT_MAX_HP],
        pointHp: s[STATE.POINT_HP], pointMax: s[STATE.POINT_MAX],
        // Where the exit is on screen (escape mission), from the camera and the visible area in world units.
        exit: {
          dx: s[STATE.EXIT_X] - s[STATE.CAM_X], dy: s[STATE.EXIT_Y] - s[STATE.CAM_Y],
          w: view.w, h: view.h, dist: s[STATE.EXIT_DIST],
        },
      });
      // What the edge markers point at, for the distance tags beside them.
      const marks = new Float32Array(core.memory.buffer, core.jr_pointers_ptr(), core.jr_pointer_count() * 4);
      const tags = [];
      for (let i = 0; i < marks.length; i += 4) {
        tags.push({ dx: marks[i] - s[STATE.CAM_X], dy: marks[i + 1] - s[STATE.CAM_Y], dist: marks[i + 2] });
      }
      ui.pointers(tags, view);
      const run = { time: s[STATE.TIME], kills: s[STATE.KILLS], level: s[STATE.LEVEL], commissions: s[STATE.COMMISSION_DONE] };
      if (s[STATE.WON]) {
        mode = 'won';
        if (!testRun) recordMission(save, mission.name, true);
        ui.missionWon(mission, run, save);
      } else if (s[STATE.OVER]) {
        mode = 'over';
        // Only endless runs count towards the best time, kills and level.
        if (mission.id === MISSION_IDS.endless) {
          ui.gameOver(mission, run, save, testRun ? null : recordRun(save, run));
        } else {
          if (!testRun) recordMission(save, mission.name, false);
          // A mission can be lost without the player dying; the title then says why.
          let reason = null;
          if (s[STATE.ESCAPED] === 1) reason = 'over.escaped';
          else if (s[STATE.ESCORT_FELL] > 0) reason = 'over.escortFell';
          else if (s[STATE.POINT_FELL] === 1) reason = 'over.fell';
          ui.gameOver(mission, run, save, null, reason);
        }
      } else if (s[STATE.CHOOSING] && !replaying) {
        // A replay picks its cards from its log, so its level-ups are not shown.
        mode = 'levelup';
        showLevelUp(s);
      }
    }

    // The music steps back while a menu is open, and while nothing is being played.
    audio.duck(mode !== 'playing');
    if (mode !== 'playing') ui.lowHealth(false);
    renderer.draw(core.memory.buffer, core.jr_instances_ptr(), core.jr_instance_count(), s[STATE.CAM_X], s[STATE.CAM_Y], clock);
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  // Handy in the console: jadereaper.core, jadereaper.state().
  window.jadereaper = {
    core, state: () => Array.from(readState()), replayLink, renderer: renderer.kind,
    get mode() { return mode; }, get replaying() { return replaying; },
  };
}

boot();
