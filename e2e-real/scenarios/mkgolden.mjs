// Build .tools/golden/q-ready.db: real-UI import result + H3 switched off through the Rules screen (variant nocampus).
import { prepareRunDir, withApp, sleep } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('mkgolden', { portable: true, seedDb: path.resolve('../.tools/golden/q-after-import.db') });
await withApp({ scenario: 'mkgolden', run }, async (app) => {
  await app.nav('/rules'); await sleep(900);
  await (await tid(app.b, 'toggle-h3')).click(); await sleep(400);
  await clickTid(app.b, 'save-rules-btn'); await sleep(1000);
});
const db = path.join(run.dataDir, 'exam-panel.db');
console.log(sql(db, "select rule_key,enabled,params_json from rule_settings where rule_key='h3'"));
fs.copyFileSync(db, path.resolve('../.tools/golden/q-ready.db'));
