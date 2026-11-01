#!/usr/bin/env node
// Browser smoke test for the staged web build.
//
//   node tools/verify_web.mjs [url]
//
// It boots Chromium headlessly (SwiftShader), starts a desktop game with a
// trusted mouse click, then emulates a 390x844 phone and drives the on-screen
// pad. Touch/pad checks dispatch PointerEvents directly to the buttons: this
// validates the game's HTML -> Rust bridge, which is what real taps drive.
// The headless CDP touch path is unreliable across Chromium versions, so it is
// not used for the assertion; the bridge itself has no keyboard dependency.

import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';

const TARGET_URL = process.argv[2] || 'http://127.0.0.1:8123/';
const OUT = new URL('../shots/', import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

const chrome = spawn('chromium', [
  '--headless=new', '--no-sandbox',
  '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
  '--remote-debugging-port=9222',
  '--user-data-dir=/tmp/opencode/chrome-verify',
  '--window-size=1200,1100',
  'about:blank',
], { stdio: 'ignore' });

async function findTarget() {
  for (let i = 0; i < 80; i++) {
    try {
      const response = await fetch('http://127.0.0.1:9222/json/list');
      const list = await response.json();
      const page = list.find((target) => target.type === 'page');
      if (page) return page;
    } catch (_) {}
    await sleep(250);
  }
  throw new Error('Chromium did not expose a page target');
}

const target = await findTarget();
const ws = new WebSocket(target.webSocketDebuggerUrl);
let nextId = 1;
const pending = new Map();
ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.id && pending.has(message.id)) {
    const { resolve, reject } = pending.get(message.id);
    pending.delete(message.id);
    if (message.error) reject(new Error(JSON.stringify(message.error)));
    else resolve(message.result);
  }
};
await new Promise((resolve, reject) => {
  ws.onopen = resolve;
  ws.onerror = reject;
});

function send(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    pending.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
}

async function evaluate(expression) {
  const result = await send('Runtime.evaluate', {
    expression,
    returnByValue: true,
    awaitPromise: true,
  });
  if (result.exceptionDetails) {
    throw new Error(result.exceptionDetails.exception?.description || result.exceptionDetails.text);
  }
  return result.result.value;
}

async function waitFor(expression, label, timeout = 15000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await evaluate(expression)) return;
    await sleep(300);
  }
  throw new Error(`timeout waiting for ${label}`);
}

async function screenshot(name) {
  const { data } = await send('Page.captureScreenshot', { format: 'png', fromSurface: true });
  writeFileSync(`${OUT}/${name}`, Buffer.from(data, 'base64'));
}

async function center(selector) {
  return evaluate(`(() => {
    const element = document.querySelector(${JSON.stringify(selector)});
    if (!element) return null;
    const rect = element.getBoundingClientRect();
    return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
  })()`);
}

async function pointerTap(selector) {
  const at = await center(selector);
  if (!at) throw new Error(`missing ${selector}`);
  await evaluate(`(() => {
    const element = document.querySelector(${JSON.stringify(selector)});
    const init = { pointerId: 1, bubbles: true, cancelable: true, isPrimary: true, pointerType: 'touch' };
    element.dispatchEvent(new PointerEvent('pointerdown', init));
    return true;
  })()`);
  await sleep(150);
  await evaluate(`(() => {
    const element = document.querySelector(${JSON.stringify(selector)});
    const init = { pointerId: 1, bubbles: true, cancelable: true, isPrimary: true, pointerType: 'touch' };
    element.dispatchEvent(new PointerEvent('pointerup', init));
    return true;
  })()`);
  return at;
}

function state() {
  return evaluate('window.__tapperState');
}

async function boot() {
  await send('Page.navigate', { url: TARGET_URL });
  await waitFor(
    `!document.getElementById('start-overlay').hidden`,
    'game boot',
    30000,
  );
  await evaluate(`document.getElementById('cabinet').scrollIntoView({ block: 'start' });`);
  await sleep(300);
}

let failed = false;
try {
  await send('Page.enable');
  await send('Runtime.enable');

  // Desktop.
  await boot();
  const startAt = await center('#start-button');
  await send('Input.dispatchMouseEvent', {
    type: 'mousePressed', x: startAt.x, y: startAt.y, button: 'left', clickCount: 1,
  });
  await send('Input.dispatchMouseEvent', {
    type: 'mouseReleased', x: startAt.x, y: startAt.y, button: 'left', clickCount: 1,
  });
  await waitFor('window.__tapperState && window.__tapperState.slot > 0', 'desktop game start');
  const desktop = await state();
  console.log('desktop state', JSON.stringify(desktop));
  if (desktop.lives !== 3) throw new Error('desktop run did not start with three lives');
  await screenshot('web_desktop.png');

  // Mobile.
  await send('Emulation.setDeviceMetricsOverride', {
    width: 390, height: 844, deviceScaleFactor: 1, mobile: true,
  });
  await send('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
  await boot();
  const pad = await evaluate(
    `getComputedStyle(document.getElementById('touch-controls')).display`,
  );
  if (pad !== 'flex') throw new Error(`touch pad is ${pad}, expected flex`);
  if (!(await evaluate(`document.body.classList.contains('touch')`))) {
    throw new Error('body is not in touch mode');
  }
  await screenshot('web_mobile_ready.png');

  await pointerTap('#start-button');
  await waitFor('window.__tapperState && window.__tapperState.slot > 0', 'mobile game start');

  const before = await state();
  await pointerTap('[data-pad="up"]');
  await waitFor(
    `window.__tapperState && window.__tapperState.lane !== ${before.lane}`,
    'lane change from touch pad',
    10000,
  );
  const afterLane = await state();
  console.log('mobile lane change', before.lane, '->', afterLane.lane);

  await pointerTap('[data-pad="pour"]');
  await waitFor(
    'window.__tapperState && window.__tapperState.mugs > 0',
    'pour from touch pad',
    10000,
  );
  const afterPour = await state();
  console.log('mobile pour state', JSON.stringify(afterPour));
  if (afterPour.mugs < 1) throw new Error('pour did not create a mug');

  const overflow = await evaluate(`({
    width: innerWidth,
    scrollWidth: document.documentElement.scrollWidth
  })`);
  if (overflow.scrollWidth > overflow.width + 1) {
    throw new Error(`horizontal overflow: ${JSON.stringify(overflow)}`);
  }
  await sleep(1200);
  await screenshot('web_mobile_game.png');
  console.log('mobile viewport', JSON.stringify(overflow));
  console.log('PASS: desktop boot, mobile pad, lane change and pour');
} catch (error) {
  failed = true;
  console.error('FAIL:', error.message);
} finally {
  ws.close();
  chrome.kill('SIGTERM');
}

process.exit(failed ? 1 : 0);
