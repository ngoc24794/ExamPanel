import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, radixSelect } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C3');
const run = prepareRunDir('C3', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'C3', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  await clickTid(app.b, 'compare-plans-button'); await sleep(700);
  const cbs = await (await app.b.$('[role="dialog"]')).$$('[role="combobox"]');
  await cbs[0].click(); await sleep(300);
  const opts = await app.b.$$('[role="option"]'); const names = []; for (const o of opts) names.push(await o.getText()); R.note('plan_options', names);
  await app.b.keys('Escape'); await sleep(300);
  await radixSelect(app.b, cbs[0], 'Nhập từ bảng của tổ'); await radixSelect(app.b, cbs[1], 'Phương án #1'); await sleep(1500);
  const txt = await app.b.execute(() => document.querySelector('[role="dialog"]').innerText);
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C3-compare-dialog.txt'), txt); await app.shot('C3-compare-dialog');
  R.note('dialog_text_head', txt.slice(0, 1800));
  // engine numbers for both plans
  const out = {};
  for (const id of [1, 2]) { const gp = await app.invoke('get_plan', { id }); out[id] = { by_rule: gp.v.score_report.by_rule.map((r) => [r.rule, r.units, r.penalty]), total: gp.v.score_report.total, per_teacher: gp.v.score_report.per_teacher.map((t) => ({ id: t.teacher_id, total: t.total ?? t.count ?? t.assignments, raw: t })).slice(0, 2), seats: gp.v.assignments.map((a) => [a.exam_id, a.grade_id, a.subject_id, a.role, a.teacher_id]) }; }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C3-engine-plans.json'), JSON.stringify(out, null, 1));
});
