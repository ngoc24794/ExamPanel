import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5');
const hdr = (app) => app.b.execute(() => { const t = document.querySelector('[data-testid="assignments-page"]').innerText; return { total: t.match(/Tổng điểm phạt: ([\d.]+)/)?.[1], hard: t.match(/Điều kiện bắt buộc: [^\n]*/)?.[0], chips: [...document.querySelectorAll('[data-testid="q-grid-summary-bar"] span, [data-testid="q-grid-summary-bar"] div')].map((e) => e.textContent.trim()).filter((x) => /^S\d+\s*:/.test(x)).slice(0, 10) }; });
const run = prepareRunDir('C5c', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db'); const log = {};
await withApp({ scenario: 'C5c', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'create-edit-copy-button'); await sleep(3000);
  if (await (await tid(app.b, 'create-edit-copy-button')).isExisting()) { await clickTid(app.b, 'history-plans-button'); await sleep(700); await (await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"(Chỉnh sửa)")]')).click(); await sleep(800); await app.b.keys('Escape'); await sleep(900); }
  const pid = sql(db, "select id from plans where source='duplicate'")[0].id;
  log.before = await hdr(app);
  await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).click(); await sleep(1500);
  const dtext = await app.b.execute(() => document.querySelector('[role="dialog"]').innerText); const m = dtext.match(/Thầy Phúc[\s\S]*?\+(\d+\.\d)\s*Tổng: ([\d.]+)/); log.ui_candidate_Phuc = m && { delta: m[1], total: m[2] };
  await app.b.keys('ArrowDown'); await app.b.keys('ArrowDown'); await app.b.keys('Enter'); await sleep(1500);
  log.after_replace_before_save = await hdr(app); log.cell = await app.b.execute(() => document.querySelector('[data-testid="q-grid-cell-1-1-VL-setter-0"]').textContent.trim());
  // what does the engine say for the current in-memory edit? (replace in DB-state copy)
  const gp = (await app.invoke('get_plan', { id: pid })).v.assignments; const tmap = (await app.invoke('list_teachers', {})).v; const phuc = tmap.find((t) => t.full_name === 'Thầy Phúc').id;
  const mod = gp.map((a) => (a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2 && a.role === 'setter' && a.position === 0 ? { ...a, teacher_id: phuc } : a));
  const ev = await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: mod }); log.engine_total_after_replace = ev.ok ? ev.v.score_report.total : ev; log.engine_hard = ev.ok ? ev.v.hard_violations.length : null;
  await clickTid(app.b, 'save-plan-assignments-button'); await sleep(1500);
  log.after_save = await app.b.execute(() => document.querySelector('[data-testid="assignments-page"]').innerText.match(/Tổng điểm phạt: ([\d.]+)/)?.[1]); log.db_score_after_save = sql(db, 'select score from plans where id=?', [pid])[0].score;
  await app.shot('C5c-after-save', { screen: false });
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5c-live-feedback.json'), JSON.stringify(log, null, 1)); console.log(JSON.stringify(log, null, 1));
