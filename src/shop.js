// The weapon shop and the inner arts bought with silver (銀兩, docs/design.md
// 4.12): what a player starts with, what the rest costs, and the silver a
// finished run earns. The core only learns what is owned, through the
// `kinds_owned` and `inner_owned` tunes the host sets at the start of a run.

import { UPGRADES, WEAPONS, ARTS } from './data.js';

/** What a weapon kind and an inner art cost. */
export const KIND_PRICE = 120;
export const INNER_PRICE = 200;

/** The inner arts every player has from the start (design 4.12). */
const FREE_INNER = ['mending', 'swift', 'lure', 'vitality', 'might'];

/** A kind comes without buying: an art's start kind, or a mechanism (it comes with its stolen route). */
export const kindFree = (w) => w.start || w.forms.includes('mechanism');

/** The inner arts (心法): every upgrade that is not a martial art's. */
export const innerArts = () => UPGRADES.map((u, id) => ({ ...u, id })).filter(({ id }) => !ARTS.some((a) => a.upgrade === id));

export const innerFree = (u) => FREE_INNER.includes(u.name);

/** The `kinds_owned` tune: the free kinds and the ones bought, as bits by weapon id. */
export function kindsOwned(save) {
  const bought = new Set(Array.isArray(save.kindsBought) ? save.kindsBought : []);
  return WEAPONS.reduce((bits, w) => (kindFree(w) || bought.has(w.name) ? bits + 2 ** w.id : bits), 0);
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

/** Buy a weapon kind or an inner art by name, if there is silver enough. Returns whether it was bought. */
export function buy(save, what, name) {
  const price = what === 'kind' ? KIND_PRICE : INNER_PRICE;
  const key = what === 'kind' ? 'kindsBought' : 'innerBought';
  const list = Array.isArray(save[key]) ? save[key] : [];
  if (list.includes(name) || (Number(save.silver) || 0) < price) return false;
  save.silver = (Number(save.silver) || 0) - price;
  save[key] = [...list, name];
  return true;
}
