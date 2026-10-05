import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C6');
const run = prepareRunDir('C6', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const seatsOf = (pid) => sql(db, 'select exam_id,grade_id,subject_id,role,position,teacher_id from assignments where plan_id=?', [pid]);
async function keep(app, cellId) { await app.b.execute((id) => document.querySelector(`[data-testid="${id}"] [data-testid="q-cell-menu-btn"]`).dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' })), cellId); await sleep(500); const it = await app.b.$('//*[@role="menuitem"][contains(.,"Giữ ô")]'); await app.b.execute((el) => el.click(), it); await sleep(500); }
await withApp({ scenario: 'C6', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  const srcPlan = 2; const src = seatsOf(srcPlan);
  const keepCells = [['q-grid-cell-1-1-VL-setter-0', [1, 1, 2, 'setter', 0]], ['q-grid-cell-1-1-VL-setter-1', [1, 1, 2, 'setter', 1]], ['q-grid-cell-2-2-VL-reviewer-0', [2, 2, 2, 'reviewer', 0]], ['q-grid-cell-3-3-CN-reviewer-0', [3, 3, 3, 'reviewer', 0]], ['q-grid-cell-4-1-VL-setter-0', [4, 1, 2, 'setter', 0]]];
  for (const [id] of keepCells) await keep(app, id);
  const bannerGrid = await bodyText(app.b); const keptIcons = await app.b.execute(() => document.querySelectorAll('[data-testid^="q-grid-cell-"] svg.text-emerald-500').length);
  R.check('C6.grid-view-feedback-for-kept-seats', /Đã chọn giữ/.test(bannerGrid) || keptIcons >= 5, { banner_in_grid_view: /Đã chọn giữ/.test(bannerGrid), bookmark_icons_in_cells: keptIcons }, 'phase-8 step 9: banner "Đã chọn giữ: N ô" (appears only in Chi tiết view)', ['screenshots/C6-kept-page.png']);
  await app.shot('C6-kept', { screen: false });
  await clickTid(app.b, 'view-toggle-detail'); await sleep(900); const bannerDetail = await bodyText(app.b);
  R.check('C6.detail-view-banner-shows-kept-count', /Đã chọn giữ: 5 ô/.test(bannerDetail), bannerDetail.match(/Đã chọn giữ:[^\n]*/)?.[0], 'Đã chọn giữ: 5 ô'); await app.shot('C6-kept-detail', { screen: false });
  await clickTid(app.b, 'view-toggle-grid'); await sleep(900);
  const before = sql(db, 'select count(*) c from plans')[0].c;
  await clickTid(app.b, 'view-toggle-detail'); await sleep(800); await clickTid(app.b, 'reoptimize-kept-button'); await sleep(700);
  R.note('dialog_text', await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText.slice(0, 300)));
  const startBtn = await app.b.$('//*[@role="dialog"]//button[not(contains(.,"Hủy")) and not(contains(.,"Close")) and (contains(.,"Bắt đầu") or contains(.,"Tối ưu"))]'); await startBtn.click(); await sleep(4000);
  const plans = sql(db, 'select id,name,source,score,run_params_json from plans order by id'); const newPlans = plans.filter((p) => p.id > 4);
  R.check('C6.new-plan-created', newPlans.length >= 1, newPlans.map((p) => ({ id: p.id, name: p.name, source: p.source, score: p.score })), 'at least one new plan');
  const results = [];
  for (const np of newPlans) { const ns = seatsOf(np.id); const lost = keepCells.filter(([, k]) => { const a = src.find((x) => x.exam_id === k[0] && x.grade_id === k[1] && x.subject_id === k[2] && x.role === k[3] && x.position === k[4]); return !ns.some((x) => x.exam_id === k[0] && x.grade_id === k[1] && x.subject_id === k[2] && x.role === k[3] && x.teacher_id === a.teacher_id); }).map(([c]) => c); const samePos = keepCells.filter(([, k]) => { const a = src.find((x) => x.exam_id === k[0] && x.grade_id === k[1] && x.subject_id === k[2] && x.role === k[3] && x.position === k[4]); return ns.some((x) => x.exam_id === k[0] && x.grade_id === k[1] && x.subject_id === k[2] && x.role === k[3] && x.position === k[4] && x.teacher_id === a.teacher_id); }).length; const changed = ns.filter((x) => !src.some((y) => y.exam_id === x.exam_id && y.grade_id === x.grade_id && y.subject_id === x.subject_id && y.role === x.role && y.teacher_id === x.teacher_id)).length; results.push({ plan: np.id, kept_seats_lost: lost, kept_at_same_position: samePos, of: keepCells.length, seats_changed_elsewhere: changed }); }
  R.note('kept_results', results);
  R.check('C6.kept-seats-unchanged', results.length > 0 && results.every((r) => r.kept_seats_lost.length === 0), results, 'every kept seat still held by the same teacher in the same panel/role', ['extracts/C6.json']);
  R.check('C6.kept-seats-same-position', results.length > 0 && results.every((r) => r.kept_at_same_position === r.of), results, 'kept setter seats keep their slot position (Đề 1 / Đề 2)', ['extracts/C6.json']);
  R.check('C6.rest-was-reoptimised', results.some((r) => r.seats_changed_elsewhere > 0), results, 'other seats actually change (source plan already at LB so may legitimately stay): informational');
  await app.shot('C6-after', { screen: false });
});
