// The weapon shop and the inner arts bought with silver (銀兩, docs/design.md
// 4.12): what a player starts with, what the rest costs, and the silver a
// finished run earns. The core only learns what is owned, through the
// `kinds_owned`, `famous_owned` and `inner_owned` tunes the host sets at the
// start of a run.

import { UPGRADES, WEAPONS, ARTS } from './data.js';

/** What a weapon kind, a famous blade (名器) and an inner art cost. */
export const KIND_PRICE = 120;
export const FAMOUS_PRICE = 300;
export const INNER_PRICE = 200;

/** The inner arts every player has from the start (design 4.12). */
const FREE_INNER = ['mending', 'swift', 'lure', 'vitality', 'might'];

/** A kind comes without buying: an art's start kind, or a mechanism (it comes with its stolen route). */
export const kindFree = (w) => w.start || w.forms.includes('mechanism');

/** Whether the player has weapon kind `w`: it is free or was bought. */
export const kindHeld = (save, w) => kindFree(w) || (Array.isArray(save.kindsBought) && save.kindsBought.includes(w.name));

/** The kinds that name a famous blade of their own (docs/martial-arts.md). */
export const famousKinds = () => WEAPONS.filter((w) => w.famous && w.famous !== 'none');

/** The inner arts (心法): every upgrade that is not a martial art's. */
export const innerArts = () => UPGRADES.map((u, id) => ({ ...u, id })).filter(({ id }) => !ARTS.some((a) => a.upgrade === id));

export const innerFree = (u) => FREE_INNER.includes(u.name);

/** The `kinds_owned` tune: the free kinds and the ones bought, as bits by weapon id. */
export function kindsOwned(save) {
  return WEAPONS.reduce((bits, w) => (kindHeld(save, w) ? bits + 2 ** w.id : bits), 0);
}

/**
 * The `famous_owned` tune: the kinds whose famous blade was bought, as bits by weapon id.
 * A weapon card of another kind with a named famous blade is at most fine steel; the core
 * lets a kind without one come famous regardless.
 */
export function famousOwned(save) {
  const bought = new Set(Array.isArray(save.famousBought) ? save.famousBought : []);
  return WEAPONS.reduce((bits, w) => (bought.has(w.name) ? bits + 2 ** w.id : bits), 0);
}

/** The `inner_owned` tune: every martial art's upgrade, the free inner arts and the ones learned, as bits by upgrade id. */
export function innerOwned(save) {
  const bought = new Set(Array.isArray(save.innerBought) ? save.innerBought : []);
  return UPGRADES.reduce((bits, u, id) => {
    const art = ARTS.some((a) => a.upgrade === id);
    return art || innerFree(u) || bought.has(u.name) ? bits + 2 ** id : bits;
  }, 0);
}

/**
 * The silver a finished run earns: 30 for each commission done, 60 for a mission won,
 * and 1 for every 30 seconds lived.
 */
export function silverEarned({ time = 0, commissions = 0 }, won) {
  return 30 * Math.max(0, Math.floor(commissions)) + (won ? 60 : 0) + Math.floor(Math.max(0, time) / 30);
}

const SHELVES = {
  kind: { price: KIND_PRICE, key: 'kindsBought' },
  famous: { price: FAMOUS_PRICE, key: 'famousBought' },
  inner: { price: INNER_PRICE, key: 'innerBought' },
};

/**
 * Buy a weapon kind, a famous blade (by its kind's name) or an inner art by name, if there is
 * silver enough. A famous blade needs its kind first. Returns whether it was bought.
 */
export function buy(save, what, name) {
  const shelf = SHELVES[what];
  if (!shelf) return false;
  const list = Array.isArray(save[shelf.key]) ? save[shelf.key] : [];
  if (list.includes(name) || (Number(save.silver) || 0) < shelf.price) return false;
  if (what === 'famous') {
    const w = famousKinds().find((x) => x.name === name);
    if (!w || !kindHeld(save, w)) return false;
  }
  save.silver = (Number(save.silver) || 0) - shelf.price;
  save[shelf.key] = [...list, name];
  return true;
}
