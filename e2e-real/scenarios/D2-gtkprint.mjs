import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid } from '../lib/ui.mjs';
import { windowList, screenShot, key } from '../lib/native.mjs';
import path from 'node:path'; import fs from 'node:fs';
const run = prepareRunDir('D2g', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
await withApp({ scenario: 'D2g', run }, async (app) => {
  await app.nav('/print/plan/1'); await sleep(2500);
  await clickTid(app.b, 'print-btn'); await sleep(3500);
  console.log('windows:', JSON.stringify(windowList()));
  screenShot('D2-gtk-print-dialog');
  const jserr = await app.b.execute(() => 'ok').catch((e) => String(e.message));
  console.log('page responsive?', jserr);
  key('Escape'); await sleep(1000); console.log('windows after esc:', JSON.stringify(windowList()));
});
