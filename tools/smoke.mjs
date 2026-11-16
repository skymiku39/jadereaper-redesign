// Smoke test: load the game in headless Chromium and fail on any console
// error, page error or failed request.
//
//     npm run smoke
//
// With WebGPU, it starts a run and plays a few seconds of its accept phase,
// then starts every mission in turn and plays a few seconds of each. This includes the CI
// runner, which has no GPU but gets a software adapter from Chromium. Without
// WebGPU, it checks that the "Cannot start" screen shows the WebGPU reason.

import { chromium } from 'playwright';
import { readFile } from 'node:fs/promises';
import { serve } from './serve.mjs';
import { unpackReplay } from '../src/replay.js';
import { MISSIONS, TUNES } from '../src/generated.js';

/** Play a replay link on the committed wasm in Node up to `ticks` steps; returns the state hash there. */
async function replayInNode(link, ticks) {
  const r = await unpackReplay(new URL(link).searchParams.get('replay'));
  const bytes = await readFile(new URL('../wasm/jadereaper_core.wasm', import.meta.url));
  const core = (await WebAssembly.instantiate(bytes, {})).instance.exports;
  core.jr_tune_reset();
  for (const [name, value] of Object.entries(r.tunes)) core.jr_tune(TUNES.find((t) => t.name === name).id, value);
  core.jr_start(r.seed, MISSIONS.find((m) => m.name === r.mission).id);
  core.jr_replay_begin();
  for (let i = 0; i < r.log.length; i += 4) core.jr_replay_push(r.log[i], r.log[i + 1], r.log[i + 2], r.log[i + 3]);
  while (core.jr_ticks() < ticks) core.jr_update(1 / 60);
  return core.jr_state_hash() >>> 0;
}

const PLAY_MS = 3000;
const MISSION_MS = 2500;
const TIMEOUT_MS = 20000;
/** A screen ignores clicks for its first 300 ms (INPUT_LOCK_MS in src/ui.js). */
const LOCK_MS = 450;

const server = await serve(0);
// The server listens on 127.0.0.1 only; "localhost" can resolve to ::1 first.
const url = `http://127.0.0.1:${server.address().port}/`;
const browser = await chromium.launch({ args: ['--enable-unsafe-webgpu'] });
const problems = [];
let consoleErrors = [];
let outcome;

try {
  const page = await browser.newPage();
  page.on('console', (msg) => {
    if (msg.type() === 'error') consoleErrors.push(msg.text());
  });
  page.on('pageerror', (err) => problems.push(`page error: ${err.message}`));
  page.on('requestfailed', (req) => problems.push(`request failed: ${req.url()} (${req.failure()?.errorText})`));
  page.on('response', (res) => {
    if (res.status() >= 400) problems.push(`HTTP ${res.status()}: ${res.url()}`);
  });

  await page.goto(url);
  await page.locator('#title:not([hidden]), #error:not([hidden])').first().waitFor({ timeout: TIMEOUT_MS });

  if (await page.locator('#error').isVisible()) {
    // The page's own i18n module gives the expected messages in its language.
    const allowed = await page.evaluate(async () => {
      const { t } = await import('./src/i18n.js');
      return [t('error.noWebgpu'), t('error.noAdapter')];
    });
    const shown = (await page.locator('#errorText').textContent()).trim();
    const noWebgpu = allowed.includes(shown);
    if (!noWebgpu) problems.push(`"Cannot start" screen with an unexpected reason: ${shown}`);
    // main.js logs the reason it shows, so that one console error is expected.
    consoleErrors = consoleErrors.filter((text) => !text.includes(shown));
    outcome = noWebgpu ? `no WebGPU here; the "Cannot start" screen shows: ${shown}` : 'the game could not start';
  } else {
    // A screen ignores clicks for its first 300 ms (INPUT_LOCK_MS in src/ui.js),
    // so a click right after the title appears can be dropped.
    await page.waitForTimeout(500);
    // The title screen has one button per mission; any of them will do. It
    // opens the mission's setup screen, which has the same lock before its
    // start button works.
    await page.locator('#title [data-action="setup"]').first().click();
    await page.locator('#setup').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(500);
    await page.locator('#setup [data-action="start"]').click();
    await page.locator('#hud').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(PLAY_MS);
    if (await page.locator('#error').isVisible()) problems.push('the "Cannot start" screen appeared during the run');
    if (!(await page.locator('#hud').isVisible())) problems.push('the HUD disappeared during the run');

    // Then every mission in turn, with the accept phase turned off so the
    // mission itself runs, and the level-up cards answered if they come.
    const { missions, acceptTime, S } = await page.evaluate(async () => {
      const d = await import('./src/data.js');
      return { missions: d.MISSIONS, acceptTime: d.TUNES.find((t) => t.name === 'accept_time'), S: d.STATE };
    });
    let played = 0;
    let replayChecked = 'its replay link was not checked';
    for (const mission of missions) {
      // Leave the run in progress: pause it, unless it has already ended, then quit to the title.
      if (!(await page.locator('#over, #won, #levelup').locator('visible=true').count())) await page.keyboard.press('Escape');
      if (await page.locator('#levelup').isVisible()) {
        await page.waitForTimeout(LOCK_MS);
        await page.locator('#cards .card').first().click();
        await page.keyboard.press('Escape');
      }
      await page.waitForTimeout(LOCK_MS);
      await page.locator('.screen:not([hidden]) [data-action="quit"]').click();
      await page.locator('#title').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
      await page.waitForTimeout(LOCK_MS);
      await page.locator(`#title [data-action="setup"][data-value="${mission.id}"]`).click();
      await page.locator('#setup').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
      await page.waitForTimeout(LOCK_MS);
      // The setup screen remembers the value, so this only does something the first time.
      const steps = Math.round((acceptTime.max - acceptTime.min) / acceptTime.step);
      for (let i = 0; i < steps && mission === missions[0]; i++) {
        await page.locator(`#tunes button[data-value="${acceptTime.id}:-1"]`).click();
      }
      await page.locator('#setup [data-action="start"]').click();
      await page.locator('#hud').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
      await page.keyboard.down('KeyD');
      await page.waitForTimeout(MISSION_MS);
      await page.keyboard.up('KeyD');
      const s = await page.evaluate(() => window.jadereaper.state());
      if (s[S.ACCEPTING]) problems.push(`${mission.name}: still in the accept phase`);
      else if (s[S.MISSION] !== mission.id) problems.push(`${mission.name}: the core plays mission ${s[S.MISSION]}`);
      else if (!(s[S.TIME] > 0.5)) problems.push(`${mission.name}: the clock stands at ${s[S.TIME]}`);
      if (await page.locator('#error').isVisible()) problems.push(`${mission.name}: the "Cannot start" screen appeared`);
      if (problems.length) break;
      played++;

      // The first mission's replay link must play the same run: pause, note
      // where the run is, and play the link back on the wasm in Node to the same step.
      if (mission === missions[0] && !(await page.locator('#over, #won').locator('visible=true').count())) {
        if (await page.locator('#levelup').isVisible()) {
          await page.waitForTimeout(LOCK_MS);
          await page.locator('#cards .card').first().click();
        }
        await page.keyboard.press('Escape');
        await page.locator('#pause').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
        const recorded = await page.evaluate(async () => ({
          ticks: window.jadereaper.core.jr_ticks(),
          hash: window.jadereaper.core.jr_state_hash() >>> 0,
          link: await window.jadereaper.replayLink(),
        }));
        const replayed = await replayInNode(recorded.link, recorded.ticks);
        if (replayed !== recorded.hash) {
          problems.push(`the replay link plays another run: hash ${replayed} at step ${recorded.ticks}, the run had ${recorded.hash}`);
        } else {
          replayChecked = `its replay link played back to the same state at step ${recorded.ticks}`;
        }
        await page.keyboard.press('Escape');
      }
    }
    outcome = `WebGPU available; played ${PLAY_MS / 1000} s of a run, then ${played} of ${missions.length} missions for ${MISSION_MS / 1000} s each; ${replayChecked}`;
  }
} catch (err) {
  problems.push(`smoke test failed: ${err.message}`);
} finally {
  await browser.close();
  server.close();
}

problems.push(...consoleErrors.map((text) => `console error: ${text}`));
if (outcome) console.log(`smoke: ${outcome}`);
if (problems.length) {
  for (const problem of problems) console.error(`  ${problem}`);
  console.error(`smoke: failed with ${problems.length} problem(s)`);
  process.exitCode = 1;
} else {
  console.log('smoke: ok');
}
