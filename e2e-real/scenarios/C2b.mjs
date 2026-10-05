import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, bodyText } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C2');
const run = prepareRunDir('C2b', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'C2b', run }, async (app) => {
  await app.nav('/assignments'); await sleep(900);
  await clickTid(app.b, 'run-optimizer-button'); await sleep(500);
  await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Kỹ"]')).click();
  await clickTid(app.b, 'start-optimize-button'); await sleep(700);
  const t0 = Date.now(); await (await app.b.$('//*[@role="dialog"]//button[contains(.,"Hủy bỏ")]')).click();
  const log = [];
  for (let i = 0; i < 20; i++) { log.push({ t: Date.now() - t0, toasts: await app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | '))), dialog: await app.b.execute(() => !!document.querySelector('[role="dialog"]')), plans: sql(db, 'select count(*) c from plans')[0].c, hash: await app.b.execute(() => location.hash) }); await sleep(150); }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C2b-cancel-timeline.json'), JSON.stringify(log, null, 1));
  await app.shot('C2b-after-cancel');
  const rows = sql(db, 'select id,name,source,score,rank,run_params_json from plans'); R.note('plans_after_cancel', rows);
  R.check('C2.real-cancel-saves-partial-plans(finding)', rows.length === 0 ? 'pass' : 'fail', { plans_saved: rows.length, rows: rows.map((r) => ({ id: r.id, score: r.score })), first_toasts: log.filter((l) => l.toasts.length).slice(0, 3) }, 'expected 0 plans after Cancel (phase-8 checklist step 2; task C2)', ['extracts/C2b-cancel-timeline.json', 'screenshots/C2b-after-cancel-page.png']);
});
