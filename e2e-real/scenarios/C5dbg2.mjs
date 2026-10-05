import { prepareRunDir, withApp, sleep } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import path from 'node:path';
const title = (app) => app.b.execute(() => document.querySelector('[data-testid="create-edit-copy-button"]') ? 'OPTIMIZER-VIEW' : 'EDIT-VIEW');
for (const preClick of [false, false, false, true, true, true]) {
  const run = prepareRunDir('C5dbg2', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
  await withApp({ scenario: 'C5dbg2', run }, async (app) => {
    await app.nav('/assignments'); await sleep(1500);
    if (preClick) { await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).click(); await sleep(500); }
    await clickTid(app.b, 'create-edit-copy-button'); await sleep(4000);
    console.log('preClick', preClick, '->', await title(app));
  });
}
