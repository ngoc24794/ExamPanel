import { prepareRunDir, withApp, sleep, REPO, OUT_ABS, App } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { driveFileDialog, sql, key, xdo } from '../lib/native.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const R = recorder('C1');
const EXP_UNITS = { s1: 1, s2: 1, s3: 36, s4: 0, s5: 1, s6: 8, s7: 0, s8: 4.545, s9: 9, s10: 0 };
const EXP_PEN = { S1: 10, S2: 3, S3: 0, S4: 0, S5: 6, S6: 16, S7: 0, S8: 36.36, S9: 45, S10: 0 };
const near = (a, b, e = 0.06) => Math.abs(a - b) <= e;
const run = prepareRunDir('C1', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const parseDetail = (t) => { const o = {}; const m = [...t.matchAll(/\nS(\d+)\n([\d.]+)/g)]; for (const x of m) if (!(('S' + x[1]) in o)) o['S' + x[1]] = parseFloat(x[2]); return o; };
let planId;
await withApp({ scenario: 'C1', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1200);
  // ---- Excel import ----
  await clickTid(app.b, 'import-plan-button'); await sleep(600); await clickTid(app.b, 'btn-browse-plan-file');
  await driveFileDialog(path.join(REPO, 'docs/reports/phase-12/plan-grid.xlsx'));
  await sleep(400); await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500);
  const pv = await bodyText(app.b); const mScore = pv.match(/Điểm phạt ước tính: ([\d.]+)/); const mSeats = pv.match(/Tổng số vị trí nạp: (\d+)/);
  R.check('C1.excel-preview-60-seats', mSeats && +mSeats[1] === 60, mSeats && mSeats[1], '60', ['extracts/C1-plan-preview-excel.txt']);
  R.check('C1.excel-preview-total-score-116.36', mScore && near(+mScore[1], 116.36, 0.05), mScore && mScore[1], '116.36 (S3 not applicable with one campus, RA-017; was 260.36)  [preview shows 1 decimal]');
  await clickTid(app.b, 'btn-plan-import-apply'); await sleep(3000);
  const plans = sql(db, 'select id,name,source,run_params_json from plans'); planId = plans[plans.length - 1].id;
  R.check('C1.plan-saved-source-manual-origin-import', plans.length === 1 && plans[0].source === 'manual' && /"origin":"import"/.test(plans[0].run_params_json || ''), plans, 'one plan, source=manual, origin=import');
  // ---- UI numbers (detail view) ----
  await clickTid(app.b, 'view-toggle-detail'); await sleep(900);
  const dt = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-ui-detail-view.txt'), dt);
  const ui = parseDetail(dt); R.note('ui_penalties', ui);
  for (const k of Object.keys(EXP_PEN)) R.check('C1.ui-' + k + '-penalty', ui[k] !== undefined && near(ui[k], EXP_PEN[k], 0.06), ui[k], EXP_PEN[k], ['extracts/C1-ui-detail-view.txt']);
  const total = dt.match(/Tổng điểm phạt: ([\d.]+)/); R.check('C1.ui-total', total && near(+total[1], 116.36, 0.011), total && total[1], '116.36 (S3 not applicable, RA-017)');
  R.check('C1.ui-hard-valid', /Điều kiện bắt buộc: Hợp lệ \(0 vi phạm\)/.test(dt), dt.match(/Điều kiện bắt buộc:[^\n]*/)?.[0], 'Hợp lệ (0 vi phạm)');
  // ---- engine (IPC get_plan) ----
  const gp = await app.invoke('get_plan', { id: planId });
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-ipc-get_plan.json'), JSON.stringify(gp, null, 1));
  const sr = gp.ok ? gp.v.score_report : null; R.note('ipc_has_score_report', !!sr);
  const ev = await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: gp.ok ? gp.v.assignments : [] });
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-ipc-evaluate_assignments.json'), JSON.stringify(ev, null, 1));
  const rep = ev.ok ? (ev.v.score_report || ev.v.report || ev.v) : null;
  R.note('ipc_evaluate_shape', ev.ok ? Object.keys(ev.v) : ev);
  // ---- export to Excel via UI ----
  await clickTid(app.b, 'view-toggle-grid'); await sleep(600);
  await clickTid(app.b, 'export-excel-button');
  const xl = path.join(run.dir, 'out/q-plan-export.xlsx');
  const dr = await driveFileDialog(xl, { shotName: 'C1-export-save-dialog' }); await sleep(2000);
  R.check('C1.export-dialog-and-file', dr.ok && fs.existsSync(xl), { dr, size: fs.existsSync(xl) && fs.statSync(xl).size }, 'xlsx written', ['screenshots/C1-export-save-dialog.png']);
  if (fs.existsSync(xl)) fs.copyFileSync(xl, path.join(OUT_ABS, 'extracts/C1-q-plan-export.xlsx'));
});
