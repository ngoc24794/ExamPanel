// .tools/golden/q-plans.db = q-ready + Q's plan imported (excel, via UI) + 3 optimizer plans (Chuẩn, via UI)
import { prepareRunDir, withApp, sleep, REPO } from '../lib/harness.mjs';
import { clickTid } from '../lib/ui.mjs';
import { driveFileDialog, sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('mkgolden2', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
await withApp({ scenario: 'mkgolden2', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1000);
  await clickTid(app.b, 'import-plan-button'); await sleep(500); await clickTid(app.b, 'btn-browse-plan-file');
  await driveFileDialog(path.join(REPO, 'docs/reports/phase-12/plan-grid.xlsx')); await sleep(400);
  await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2000); await clickTid(app.b, 'btn-plan-import-apply'); await sleep(2500);
  await clickTid(app.b, 'run-optimizer-button'); await sleep(500);
  await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Chuẩn"]')).click(); await clickTid(app.b, 'start-optimize-button'); await sleep(4000);
});
const db = path.join(run.dataDir, 'exam-panel.db'); console.log(sql(db, 'select id,name,source,score,rank,is_final from plans'));
fs.copyFileSync(db, path.resolve('../.tools/golden/q-plans.db'));
