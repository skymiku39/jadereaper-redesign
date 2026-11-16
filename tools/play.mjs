// Drive the game in headless Chromium for the tools: open the page, start a
// mission, walk a bot around, and take pictures that include the game view.
//
// A plain page screenshot shows the HTML UI over a blank canvas, because the
// WebGPU canvas of a headless browser is never presented. So the page is made
// to draw each frame into a texture of our own, which can be copied out.

import { chromium } from 'playwright';
import { writeFileSync } from 'node:fs';
import { serve } from './serve.mjs';

const TIMEOUT_MS = 20000;
/** A screen ignores clicks for its first 300 ms (INPUT_LOCK_MS in src/ui.js). */
const LOCK_MS = 450;

/** Runs in the page before the game loads. */
function capture() {
  const cap = { device: null, format: null, texture: null };
  const configure = GPUCanvasContext.prototype.configure;
  GPUCanvasContext.prototype.configure = function (options) {
    cap.device = options.device;
    cap.format = options.format;
    return configure.call(this, options);
  };
  // The game draws into this texture instead of the canvas's own.
  GPUCanvasContext.prototype.getCurrentTexture = function () {
    const { width, height } = this.canvas;
    if (!cap.texture || cap.texture.width !== width || cap.texture.height !== height) {
      cap.texture = cap.device.createTexture({
        size: [width, height],
        format: cap.format,
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
      });
    }
    return cap.texture;
  };
  /** The last frame the game drew, as a PNG data URL. */
  window.__gameView = async () => {
    const { device, texture, format } = cap;
    const { width, height } = texture;
    const row = Math.ceil((width * 4) / 256) * 256;
    const buffer = device.createBuffer({ size: row * height, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
    const encoder = device.createCommandEncoder();
    encoder.copyTextureToBuffer({ texture }, { buffer, bytesPerRow: row }, [width, height]);
    device.queue.submit([encoder.finish()]);
    await buffer.mapAsync(GPUMapMode.READ);
    const src = new Uint8Array(buffer.getMappedRange());
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    const image = ctx.createImageData(width, height);
    const bgra = format.startsWith('bgra');
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const s = y * row + x * 4;
        const d = (y * width + x) * 4;
        image.data[d] = src[s + (bgra ? 2 : 0)];
        image.data[d + 1] = src[s + 1];
        image.data[d + 2] = src[s + (bgra ? 0 : 2)];
        image.data[d + 3] = 255;
      }
    }
    ctx.putImageData(image, 0, 0);
    buffer.unmap();
    return canvas.toDataURL('image/png');
  };
}

/**
 * Open the game. `tuning` holds tune values by name, as the setup screen would
 * have saved them. Returns the page and helpers; `problems` collects console
 * errors, page errors and failed requests.
 */
export async function openGame({ width = 1280, height = 720, lang = 'zh-Hant', tuning = {} } = {}) {
  const server = await serve(0);
  // The server listens on 127.0.0.1 only; "localhost" can resolve to ::1 first.
  const url = `http://127.0.0.1:${server.address().port}/`;
  const browser = await chromium.launch({ args: ['--enable-unsafe-webgpu'] });
  const page = await browser.newPage({ viewport: { width, height } });
  const problems = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error') problems.push(`console error: ${msg.text()}`);
  });
  page.on('pageerror', (err) => problems.push(`page error: ${err.message}`));
  page.on('requestfailed', (req) => problems.push(`request failed: ${req.url()} (${req.failure()?.errorText})`));
  page.on('response', (res) => {
    if (res.status() >= 400) problems.push(`HTTP ${res.status()}: ${res.url()}`);
  });
  await page.addInitScript(capture);
  await page.addInitScript(([save]) => localStorage.setItem('jadereaper.save.v1', save), [
    JSON.stringify({ tuning, lang, muted: true }),
  ]);

  const close = async () => {
    await browser.close();
    server.close();
  };
  try {
    await page.goto(url);
    await page.locator('#title:not([hidden]), #error:not([hidden])').first().waitFor({ timeout: TIMEOUT_MS });
  } catch (err) {
    await close();
    throw err;
  }
  const webgpu = !(await page.locator('#error').isVisible());

  /** The tables of src/data.js, for looking ids up by name. */
  const data = webgpu
    ? await page.evaluate(async () => {
      const d = await import('./src/data.js');
      return {
        missions: d.MISSIONS, arts: d.ARTS, elements: d.ELEMENTS, upgrades: d.UPGRADES.length, state: d.STATE,
      };
    })
    : null;

  /** The HUD state array of the core. */
  const state = () => page.evaluate(() => window.jadereaper.state());

  /** Start a mission from the title screen, through its setup screen. */
  async function start(mission) {
    const id = data.missions.find((m) => m.name === mission)?.id;
    if (id === undefined) throw new Error(`no mission named ${mission}`);
    await page.waitForTimeout(LOCK_MS);
    await page.locator(`#title [data-action="setup"][data-value="${id}"]`).click();
    await page.locator('#setup').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
    await page.waitForTimeout(LOCK_MS);
    await page.locator('#setup [data-action="start"]').click();
    await page.locator('#hud').waitFor({ state: 'visible', timeout: TIMEOUT_MS });
  }

  /**
   * Give the player a martial art at a level with an element, by name
   * (`give('pulse', 3, 'fire')`; no element name for none).
   */
  async function give(art, level, element) {
    const a = data.arts.find((x) => x.name === art);
    const e = element ? data.elements.find((x) => x.name === element) : { id: 0 };
    if (!a || !e) throw new Error(`cannot give ${art} with ${element}`);
    await page.evaluate(([i, l, el]) => window.jadereaper.core.jr_give(i, l, el), [a.id, level, e.id]);
  }

  /**
   * Play for `seconds`: walk in a square (or stand still), and take the first
   * card of every level-up; an element card goes on the first art. Stops
   * early when the run ends. `each` is called about four times a second.
   */
  async function play(seconds, { still = false, each = null } = {}) {
    const keys = ['KeyD', 'KeyS', 'KeyA', 'KeyW'];
    const steps = Math.ceil(seconds * 4);
    for (let i = 0; i < steps; i++) {
      if (await page.locator('#levelup').isVisible()) {
        await page.waitForTimeout(LOCK_MS);
        await page.locator('#cards .card').first().click();
        continue;
      }
      if ((await page.locator('#over').isVisible()) || (await page.locator('#won').isVisible())) return;
      const key = keys[Math.floor(i / 4) % 4];
      if (!still) await page.keyboard.down(key);
      await page.waitForTimeout(250);
      if (!still) await page.keyboard.up(key);
      if (each) await each(i / 4);
    }
  }

  /** Save a picture of the page with the game view behind the UI. */
  async function shot(path) {
    const view = await page.evaluate(() => window.__gameView());
    await page.evaluate((src) => new Promise((resolve) => {
      const img = document.createElement('img');
      img.id = 'gameViewShot';
      img.onload = resolve;
      img.src = src;
      // Sits exactly where the canvas is, under the UI.
      const canvas = document.getElementById('game');
      img.style.cssText = getComputedStyle(canvas).cssText;
      canvas.before(img);
      canvas.style.visibility = 'hidden';
    }), view);
    writeFileSync(path, await page.screenshot());
    await page.evaluate(() => {
      document.getElementById('gameViewShot').remove();
      document.getElementById('game').style.visibility = '';
    });
  }

  return { page, problems, webgpu, data, state, start, give, play, shot, close };
}
