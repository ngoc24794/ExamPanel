import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C3probe', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
await withApp({ scenario: 'C3probe', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  await clickTid(app.b, 'compare-plans-button'); await sleep(800);
  const d = await app.b.execute(() => { const x = document.querySelector('[role="dialog"]'); return { text: x.innerText.slice(0, 1500), combos: [...x.querySelectorAll('[role=combobox],select')].map((e) => e.innerText || e.tagName) }; });
  console.log(JSON.stringify(d, null, 1)); await app.shot('C3probe-dialog');
});
