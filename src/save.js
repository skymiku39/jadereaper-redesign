// Records and settings, kept in localStorage. The game still works if
// storage is blocked: values just do not persist.

const KEY = 'jadereaper.save.v1';

const DEFAULTS = {
  bestTime: 0,
  bestKills: 0,
  bestLevel: 0,
  runs: 0,
  totalKills: 0,
  // Per mission name: { tries, wins }. The numbers above are for endless only.
  missions: {},
  // Per tune name: the value last set on a mission's setup screen. Only changed tunes are kept.
  tuning: {},
  volume: 0.6,
  muted: false,
  // Damage numbers, and camera shake with hit-stop.
  numbers: true,
  motion: true,
  music: true,
  lang: null, // null: use DEFAULT_LANG from i18n.js
};

export function loadSave() {
  try {
    const raw = localStorage.getItem(KEY);
    const data = raw ? JSON.parse(raw) : {};
    const save = { ...DEFAULTS, ...(data && typeof data === 'object' ? data : {}) };
    // Older saves have no mission records or tuning; never share the default objects.
    const plain = (v) => (v && typeof v === 'object' && !Array.isArray(v) ? { ...v } : {});
    save.missions = plain(save.missions);
    save.tuning = plain(save.tuning);
    return save;
  } catch {
    return { ...DEFAULTS, missions: {}, tuning: {} };
  }
}

/** The value of a tune: the saved one if it is a number, kept inside the tune's range, else the default. */
export function tuneValue(save, tune) {
  const v = save.tuning[tune.name];
  return typeof v === 'number' && Number.isFinite(v) ? Math.min(tune.max, Math.max(tune.min, v)) : tune.def;
}

/** Remember a tune's value. The default is not stored, so a later change of the default takes effect. */
export function setTune(save, tune, value) {
  if (value === tune.def) delete save.tuning[tune.name];
  else save.tuning[tune.name] = value;
  writeSave(save);
}

/** How often a mission was tried and won. */
export function missionRecord(save, name) {
  const r = save.missions[name];
  return { tries: Number(r?.tries) || 0, wins: Number(r?.wins) || 0 };
}

/** Fold a finished mission run into the save. */
export function recordMission(save, name, won) {
  const r = missionRecord(save, name);
  save.missions[name] = { tries: r.tries + 1, wins: r.wins + (won ? 1 : 0) };
  writeSave(save);
}

/** Forget every record but keep the settings. */
export function clearRecords(save) {
  Object.assign(save, { bestTime: 0, bestKills: 0, bestLevel: 0, runs: 0, totalKills: 0, missions: {} });
  writeSave(save);
}

export function writeSave(save) {
  try {
    localStorage.setItem(KEY, JSON.stringify(save));
  } catch {
    // Private window or storage disabled.
  }
}

/** Fold a finished run into the save. Returns which records were beaten. */
export function recordRun(save, { time, kills, level }) {
  const beaten = {
    time: time > save.bestTime,
    kills: kills > save.bestKills,
    level: level > save.bestLevel,
  };
  save.bestTime = Math.max(save.bestTime, time);
  save.bestKills = Math.max(save.bestKills, kills);
  save.bestLevel = Math.max(save.bestLevel, level);
  save.runs += 1;
  save.totalKills += kills;
  writeSave(save);
  return beaten;
}

export function formatTime(seconds) {
  const s = Math.floor(seconds);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}
