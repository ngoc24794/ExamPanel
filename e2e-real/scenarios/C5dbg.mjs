import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import path from 'node:path';
const run = prepareRunDir('C5dbg', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
await withApp({ scenario: 'C5dbg', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  await clickTid(app.b, 'create-edit-copy-button'); await sleep(2500);
  await app.shot('C5dbg-after-copy', { screen: false });
  const info = await app.b.execute(() => { const c = document.querySelector('[data-testid="q-grid-cell-1-1-VL-setter-0"]'); return { draggable: c.getAttribute('draggable'), txt: c.textContent, header: document.querySelector('[data-testid="assignments-page"]').innerText.slice(0, 600) }; });
  console.log(JSON.stringify(info, null, 1));
  const c = await tid(app.b, 'q-grid-cell-1-1-VL-setter-0'); await c.click(); await sleep(1500);
  console.log('dialogs:', await app.b.execute(() => [...document.querySelectorAll('[role="dialog"]')].map((d) => d.innerText.slice(0, 300))));
  await app.shot('C5dbg-after-click', { screen: false });
});
