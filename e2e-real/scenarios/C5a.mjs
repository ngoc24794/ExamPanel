import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C5a', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'C5a', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  console.log('PLAN HEADER:', (await app.b.execute(() => document.querySelector('[data-testid="plan-matrix-view"], [data-testid="q-plan-grid-root"]')?.parentElement?.innerText?.slice(0, 300))));
  // click a cell on optimizer plan (immutable)
  const cell = await tid(app.b, 'q-grid-cell-1-1-VL-setter-0'); await cell.click(); await sleep(600);
  console.log('after click on optimizer plan cell, dialog?', await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText?.slice(0, 200)));
  console.log('toolbar buttons:', await app.b.execute(() => [...document.querySelectorAll('[data-testid="assignments-page"] button')].map((b) => (b.innerText || b.getAttribute('aria-label') || '').trim().slice(0, 30)).filter(Boolean).join(' | ')));
  await clickTid(app.b, 'create-edit-copy-button'); await sleep(700);
  console.log('copy dialog:', await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText?.slice(0, 400)));
  await app.shot('C5a-copy-dialog', { screen: false });
});
