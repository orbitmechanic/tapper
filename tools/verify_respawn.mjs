#!/usr/bin/env node
// Regression check for the dropped-mug life-loss freeze.
//
//   node tools/verify_respawn.mjs [url]
//
// Serves the first saloon patron, then pours into the now-empty lane so the
// full mug breaks at the door. A healthy build loses one shift, respawns the
// screen and returns to `phase: 0`; the old bug stayed on `phase: 1` and
// reloaded the screen forever.

import { spawn } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { setTimeout as sleep } from 'node:timers/promises';

const TARGET_URL = process.argv[2] || 'http://127.0.0.1:8123/';
const OUT = new URL('../shots/', import.meta.url).pathname;
mkdirSync(OUT, { recursive: true });

const chrome = spawn('chromium', [
  '--headless=new', '--no-sandbox',
  '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
  '--remote-debugging-port=9234',
  '--user-data-dir=/tmp/opencode/chrome-respawn',
  '--window-size=1200,1100',
  'about:blank',
], { stdio: 'ignore' });

async function findTarget() {
  for (let i = 0; i < 80; i++) {
    try {
      const list = await (await fetch('http://127.0.0.1:9234/json/list')).json();
      const page = list.find((target) => target.type === 'page');
      if (page) return page;
    } catch (_) {}
    await sleep(250);
  }
  throw new Error('no Chromium target');
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
  if (result.exceptionDetails) throw new Error(result.exceptionDetails.text);
  return result.result.value;
}

async function waitFor(expression, label, timeout = 30000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await evaluate(expression)) return;
    await sleep(350);
  }
  throw new Error(`timeout waiting for ${label}`);
}

async function pointerTap(selector) {
  await evaluate(`(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    const init = { pointerId: 1, bubbles: true, cancelable: true, isPrimary: true, pointerType: 'touch' };
    el.dispatchEvent(new PointerEvent('pointerdown', init));
    return true;
  })()`);
  await sleep(160);
  await evaluate(`(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    const init = { pointerId: 1, bubbles: true, cancelable: true, isPrimary: true, pointerType: 'touch' };
    el.dispatchEvent(new PointerEvent('pointerup', init));
    return true;
  })()`);
}

const state = () => evaluate('window.__tapperState');

let failed = false;
try {
  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.navigate', { url: TARGET_URL });
  await waitFor(`!!document.getElementById('start-overlay') && !document.getElementById('start-overlay').hidden`, 'boot', 45000);
  await evaluate(`document.getElementById('cabinet').scrollIntoView({ block: 'start' });`);
  await sleep(400);
  await pointerTap('#start-button');
  await waitFor('window.__tapperState && window.__tapperState.slot > 0', 'game start');
  await waitFor('window.__tapperState.p0 > 0', 'first lane-0 patron');
  console.log('patron up', JSON.stringify(await state()));

  await pointerTap('[data-pad="pour"]');
  await waitFor('window.__tapperState.p0 === 0 && window.__tapperState.mugs === 0', 'lane cleared and empty caught', 40000);
  console.log('lane cleared', JSON.stringify(await state()));

  // The lane is empty now. This mug has no catcher and must break at the door.
  await pointerTap('[data-pad="pour"]');
  await waitFor('window.__tapperState.lives === 2', 'mug break and life loss', 60000);
  const dead = await state();
  console.log('life lost', JSON.stringify(dead));

  await waitFor(
    'window.__tapperState.phase === 0 && window.__tapperState.lives === 2 && (window.__tapperState.p1 + window.__tapperState.p2 + window.__tapperState.p3) > 0',
    'respawn into a playable screen',
    30000,
  );
  const respawned = await state();
  console.log('respawned', JSON.stringify(respawned));

  await sleep(1200);
  const settled = await state();
  if (settled.phase !== 0) throw new Error('phase did not stay Normal after respawn');
  if (settled.lives !== 2) throw new Error('life count changed after respawn');

  const { data } = await send('Page.captureScreenshot', { format: 'png', fromSurface: true });
  writeFileSync(`${OUT}/web_respawn.png`, Buffer.from(data, 'base64'));
  console.log('PASS: life loss respawns instead of freezing');
} catch (error) {
  failed = true;
  console.error('FAIL:', error.message);
  console.error('state:', JSON.stringify(await state().catch(() => null)));
} finally {
  ws.close();
  chrome.kill('SIGTERM');
}

process.exit(failed ? 1 : 0);
