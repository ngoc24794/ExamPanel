// Mark final / stale / pin & forbid from the grid / rename / delete — real app, golden DB with plans.
import { prepareRunDir, withApp, sleep, OUT_ABS, REPO } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { driveFileDialog, sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5');
const run = prepareRunDir('C5g', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const toasts = (app) => app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | ')));
const planBadges = (app) => app.b.execute(() => [...document.querySelectorAll('[data-testid^="plan-item-"]')].map((e) => ({ id: e.getAttribute('data-testid'), t: e.innerText.replace(/\n+/g, ' | ').slice(0, 140) })));
async function openHistory(app) { await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'history-plans-button'); await sleep(800); }
async function menuAction(app, planId, label) { await app.b.execute((id) => document.querySelector(`[data-testid="plan-menu-${id}"]`).dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' })), planId); await sleep(500); const it = await app.b.$(`//*[@role="menuitem"][contains(.,"${label}")]`); await it.click(); await sleep(1500); }
await withApp({ scenario: 'C5g', run }, async (app) => {
  await openHistory(app);
  const before = await planBadges(app); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5g-history-before.json'), JSON.stringify(before, null, 1)); await app.shot('C5g-history', { screen: false });
  // 1) Mark final on the valid imported Q plan (id 1)
  await menuAction(app, 1, 'Đánh dấu chính thức'); const t1 = await toasts(app);
  const fin = sql(db, 'select id,is_final from plans where is_final=1');
  R.check('C5.mark-final-allowed-when-valid', fin.length === 1 && fin[0].id === 1, { toasts: t1, final_rows: fin }, 'valid plan (0 hard violations, not stale) becomes final; badge "Chính thức"', ['extracts/C5g-history-before.json']);
  const badges = await planBadges(app); R.note('badges_after_final', badges);
  await app.shot('C5g-after-final', { screen: false });
  // 2) change data -> stale
  await app.nav('/unavailability'); await sleep(900); await clickTid(app.b, 'unavail-cell-1-2'); await sleep(900);
  await openHistory(app); const staleBadges = await planBadges(app); R.note('badges_after_data_change', staleBadges);
  const staleDb = await app.invoke('plan_status', { id: 1 }); R.note('plan_status_1_after_change', staleDb);
  R.check('C5.stale-badge-after-data-change', staleBadges.some((b) => /Dữ liệu đã đổi|Stale/i.test(b.t)), staleBadges.map((b) => b.t.slice(0, 80)), 'plans show "Dữ liệu đã đổi" after marking a teacher unavailable', ['screenshots/C5g-stale-history-page.png']);
  await app.shot('C5g-stale-history', { screen: false });
  // 3) mark final on a stale plan must be blocked
  await menuAction(app, 2, 'Đánh dấu chính thức'); const t3 = await toasts(app);
  const fin2 = sql(db, 'select id from plans where is_final=1');
  R.check('C5.mark-final-blocked-when-stale', fin2.length === 1 && fin2[0].id === 1 && t3.some((x) => /thay đổi|lỗi thời|dữ liệu/i.test(x)), { toasts: t3, final_rows: fin2 }, 'blocked with a translated message; final stays plan 1', ['screenshots/C5g-stale-history-page.png']);
  // 4) hard-invalid plan: clear C Quí's override, import Q's table, mark final => blocked "plan_invalid"
  await app.nav('/teachers'); await sleep(900);
  const rows = await app.b.$$('tr'); for (const r of rows) { if ((await r.getText()).includes('Cô Quí')) { for (const b of await r.$$('button')) { const lab = (await b.getAttribute('aria-label')) || (await b.getAttribute('title')) || ''; if (lab.includes('Chỉnh sửa')) { await app.b.execute((el) => el.click(), b); break; } } break; } }
  await sleep(700); const adv = await app.b.$('//*[@role="dialog"]//*[contains(text(),"advancedSettings")]'); if (await adv.isExisting()) await adv.click(); await sleep(300);
  await (await tid(app.b, 'teacher-max-tasks-override-input')).clearValue(); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(1200);
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'import-plan-button'); await sleep(500); await clickTid(app.b, 'btn-browse-plan-file');
  await driveFileDialog(path.join(REPO, 'docs/reports/phase-12/plan-grid.xlsx')); await sleep(500); await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500);
  const canApply = await (await tid(app.b, 'btn-plan-import-apply')).isEnabled(); R.note('import_with_hard_violation_can_apply', canApply);
  if (canApply) { await clickTid(app.b, 'btn-plan-import-apply'); await sleep(2500); }
  await app.shot('C5g-invalid-plan', { screen: false });
  const invalidId = sql(db, 'select max(id) m from plans')[0].m;
  { const gpi = (await app.invoke('get_plan', { id: invalidId })).v; const evi = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: gpi.assignments })).v; fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5g-invalid-plan-hard-violations.json'), JSON.stringify(evi.hard_violations, null, 1)); R.note('invalid_plan_hard_violations', evi.hard_violations.map((h) => `${h.rule}:${h.code} teacher=${h.teacher} ${JSON.stringify(h.params)}`)); }
  const hdr = await bodyText(app.b); R.note('invalid_plan_header', hdr.match(/Điều kiện bắt buộc:[^\n]*/)?.[0]);
  // temporarily un-stale: nothing to do; the plan is created against current data so only invalidity should block
  await openHistory(app); await menuAction(app, invalidId, 'Đánh dấu chính thức'); const t4 = await toasts(app);
  const fin3 = sql(db, 'select id from plans where is_final=1');
  R.check('C5.mark-final-blocked-with-hard-violations', fin3.length === 1 && fin3[0].id === 1 && t4.length > 0, { toasts: t4, final_rows: fin3, header: hdr.match(/Điều kiện bắt buộc:[^\n]*/)?.[0] }, 'blocked: "không thể đánh dấu... vi phạm"', ['screenshots/C5g-invalid-plan-page.png']);
  await app.shot('C5g-final-blocked', { screen: false });
  // 5) pin & forbid from the matrix cell menu
  await app.b.keys('Escape'); await sleep(300);
  await app.nav('/assignments'); await sleep(1500);
  await app.b.execute(() => document.querySelector('[data-testid="q-grid-cell-1-2-VL-reviewer-0"] [data-testid="q-cell-menu-btn"]').dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' }))); await sleep(500);
  const mi = await app.b.$$('[role="menuitem"]'); const labels = []; for (const m of mi) labels.push(await m.getText()); R.note('cell_menu_items', labels);
  const pin = await app.b.$('//*[@role="menuitem"][contains(.,"Ghim")]'); await app.b.execute((el) => el.click(), pin); await sleep(1200);
  const locks1 = sql(db, 'select * from locks'); R.check('C5.pin-from-matrix-creates-lock', locks1.some((l) => l.kind === 'pin'), locks1, 'locks row kind=pin for that seat', ['extracts/C5.json']);
  await app.b.execute(() => document.querySelector('[data-testid="q-grid-cell-3-3-VL-reviewer-0"] [data-testid="q-cell-menu-btn"]').dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' }))); await sleep(500);
  const fb = await app.b.$('//*[@role="menuitem"][contains(.,"Cấm")]'); await app.b.execute((el) => el.click(), fb); await sleep(1200);
  const locks2 = sql(db, 'select * from locks'); R.check('C5.forbid-from-matrix-creates-lock', locks2.some((l) => l.kind === 'forbid'), locks2, 'locks row kind=forbid');
  const pinIcon = await app.b.execute(() => !!document.querySelector('[data-testid="pin-slot-icon"]')); R.check('C5.pin-icon-shown-in-grid', pinIcon, pinIcon, 'pin icon visible in the pinned cell');
  await app.shot('C5g-pinned', { screen: false });
});
