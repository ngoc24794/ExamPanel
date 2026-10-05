import { prepareRunDir, withApp, sleep, REPO, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { driveFileDialog, sql, key, xdo, typeText } from '../lib/native.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C1');
const run = prepareRunDir('C1', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'C1', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1200);
  await clickTid(app.b, 'import-plan-button'); await sleep(700);
  await app.shot('C1-import-plan-dialog');
  const xlsx = path.join(REPO, 'docs/reports/phase-12/plan-grid.xlsx');
  await clickTid(app.b, 'btn-browse-plan-file');
  const r = await driveFileDialog(xlsx, { shotName: 'C1-open-dialog' });
  R.check('C1.excel-open-dialog', r.ok, r, 'GTK open dialog accepts path', ['screenshots/C1-open-dialog.png']);
  await sleep(500);
  await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500);
  await app.shot('C1-plan-preview');
  const txt = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-plan-preview-excel.txt'), txt);
  console.log(txt.slice(0, 2500));
  for (const tab of ['tab-preview-issues', 'tab-preview-assignments']) { await clickTid(app.b, tab); await sleep(500); await app.shot('C1-preview-' + tab); fs.writeFileSync(path.join(OUT_ABS, `extracts/C1-${tab}.txt`), await bodyText(app.b)); }
});
