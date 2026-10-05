// Import wizard upsert/sync semantics (phase-9 steps 2-3) + restoring the v4 fixture through the Restore dialog (phase-11 step 8).
import { prepareRunDir, withApp, sleep, OUT_ABS, REPO } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { driveFileDialog, sql, PY } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, execFileSync } from 'node:child_process';
const R = recorder('B2');
const run = prepareRunDir('B2c', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') }); const db = path.join(run.dataDir, 'exam-panel.db');
const base = path.join(OUT_ABS, 'extracts/B2-q-filled.xlsx');
const py = `
import openpyxl, sys
wb = openpyxl.load_workbook(sys.argv[1]); ws = wb['Giáo viên']
if sys.argv[3] == 'upsert':
    ws.append(['GV999', 'Trần Văn Thử Nghiệm', 'T Thử', 'CPH', '10, 11', '1', 'Có', None, None, None])
else:
    keep = [r for r in ws.iter_rows(min_row=2, values_only=True) if r[2] not in ('C Lan', 'C Tú', 'C Như')]
    ws.delete_rows(2, ws.max_row)
    for r in keep: ws.append(list(r))
wb.save(sys.argv[2])`;
for (const m of ['upsert', 'sync']) execFileSync(PY, ['-c', py, base, path.join(run.dir, `out/${m}.xlsx`), m]);
execSync(`chown -R qauser:qauser ${run.dir}`);
const toasts = (app) => app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | ')));
async function openWizard(app, file, mode) {
  await app.nav('/teachers'); await sleep(1000); await clickTid(app.b, 'import-excel-btn'); await sleep(600); await clickTid(app.b, 'import-browse-btn'); await driveFileDialog(path.join(run.dir, `out/${file}`)); await sleep(500);
  if (mode === 'sync') await clickTid(app.b, 'mode-sync-card'); else await clickTid(app.b, 'mode-upsert-card');
  await clickTid(app.b, 'import-run-preview-btn'); await sleep(2500);
}
await withApp({ scenario: 'B2c', run }, async (app) => {
  // ---- upsert ----
  await openWizard(app, 'upsert.xlsx', 'upsert'); const t = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/B2c-upsert-preview.txt'), t); await app.shot('B2c-upsert-preview', { screen: false });
  const rowsTxt = await app.b.execute(() => [...document.querySelectorAll('[role="dialog"] tbody tr')].map((r) => r.innerText.replace(/\s+/g, ' ').trim()));
  const newRows = rowsTxt.filter((r) => /GV999/.test(r)); const statusCounts = { new: rowsTxt.filter((r) => /Mới/.test(r)).length, unchanged: rowsTxt.filter((r) => /Không đổi/.test(r)).length, update: rowsTxt.filter((r) => /Cập nhật/.test(r)).length };
  R.check('B2.upsert-preview-new-teacher-row', newRows.length === 1 && /Mới/.test(newRows[0]), { gv999_row: newRows[0], status_counts_in_teacher_tab: statusCounts, rows: rowsTxt.length }, 'GV999 shown with green "Mới"; other 12 rows "Không đổi" (phase-9 step 2)', ['extracts/B2c-upsert-preview.txt', 'screenshots/B2c-upsert-preview-page.png']);
  const filt = []; for (const lab of ['Mới', 'Cập nhật', 'Không đổi', 'Lỗi']) { const b = await app.b.$(`//*[@role="dialog"]//button[normalize-space(.)="${lab}"]`); if (await b.isExisting()) { await b.click(); await sleep(400); filt.push([lab, await app.b.execute(() => document.querySelectorAll('[role="dialog"] tbody tr').length)]); } }
  R.note('status_filter_row_counts', filt);
  const before = sql(db, 'select count(*) c from teachers')[0].c; await clickTid(app.b, 'import-apply-btn'); await sleep(2500); const tt = await toasts(app);
  R.check('B2.upsert-apply-creates-teacher-and-backup', sql(db, 'select count(*) c from teachers')[0].c === before + 1 && fs.readdirSync(path.join(run.dataDir, 'backups')).some((f) => /pre-import/.test(f)), { teachers_before: before, after: sql(db, 'select count(*) c from teachers')[0].c, toasts: tt, backups: fs.readdirSync(path.join(run.dataDir, 'backups')) }, 'GV999 created, pre-import backup, success toast with counts');
  R.check('B2.upsert-teacher-list-refreshes', /Trần Văn Thử Nghiệm|T Thử/.test(await bodyText(app.b)), 'checked list text', 'new teacher visible without reload');
  // ---- sync ----
  await sleep(4000); await openWizard(app, 'sync.xlsx', 'sync'); const dtab = await tid(app.b, 'tab-deactivated'); const dEx = await dtab.isExisting(); let dtext = '';
  if (dEx) { await dtab.click(); await sleep(600); dtext = await app.b.execute(() => document.querySelector('[role="dialog"]').innerText); }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/B2c-sync-preview-deactivated.txt'), dtext); await app.shot('B2c-sync-preview', { screen: false });
  const expectDeact = ['Cô Lan', 'Cô Tú', 'Cô Như', 'Trần Văn Thử Nghiệm'];
  R.check('B2.sync-preview-lists-teachers-to-deactivate', dEx && expectDeact.every((n) => dtext.includes(n)), { tab_present: dEx, names_found: expectDeact.filter((n) => dtext.includes(n)) }, 'tab "Ngừng hoạt động" lists the 3 removed teachers + GV999 (not in file)', ['extracts/B2c-sync-preview-deactivated.txt', 'screenshots/B2c-sync-preview-page.png']);
  await clickTid(app.b, 'import-apply-btn'); await sleep(2500);
  const act = sql(db, "select full_name,active from teachers where full_name in ('Cô Lan','Cô Tú','Cô Như','Trần Văn Thử Nghiệm') order by id"); const stillAssigned = sql(db, 'select count(*) c from assignments where plan_id=1 and teacher_id in (select id from teachers where active=0)')[0].c;
  R.check('B2.sync-apply-deactivates-not-deletes', act.length === 4 && act.every((r) => r.active === 0) && stillAssigned > 0, { rows: act, assignments_of_inactive_teachers_in_plan1: stillAssigned, total_teachers: sql(db, 'select count(*) c from teachers')[0].c }, 'active=0, rows kept, historical assignments kept (phase-9 step 3)');
  // ---- restore the v4 fixture through the Restore dialog ----
  await app.nav('/settings'); await sleep(1000); await clickTid(app.b, 'restore-file-btn'); await driveFileDialog(path.join(REPO, 'crates/storage/fixtures/v4_synthetic.db')); await sleep(1500);
  const rt = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); fs.writeFileSync(path.join(OUT_ABS, 'extracts/B2c-restore-v4-dialog.txt'), rt); await app.shot('B2c-restore-v4-dialog', { screen: false });
  await clickTid(app.b, 'confirm-restore-btn'); await sleep(3000);
  const uv = sql(db, 'pragma user_version')[0].user_version; const nT = sql(db, 'select count(*) c from teachers')[0].c; const plan = sql(db, 'select name,is_final from plans'); const subj = sql(db, 'select code from subjects').map((x) => x.code); const comp = sql(db, "select count(distinct teacher_id) c from teacher_competencies where subject_id=(select id from subjects where code='CHUNG')")[0].c;
  await app.nav('/teachers'); await sleep(1200); const tl = await bodyText(app.b);
  R.check('B2.restore-v4-fixture-migrates-to-v5', uv === 5 && nT === 12 && plan.length === 1 && /Nguyễn Văn A/.test(tl), { dialog: rt.replace(/\n+/g, ' | ').slice(0, 260), user_version_after: uv, teachers: nT, plans: plan, subjects: subj, teachers_with_CHUNG_competency: comp }, 'migration 0005 runs on restore; legacy teachers keep CHUNG competency; plan intact (phase-11 step 8)', ['extracts/B2c-restore-v4-dialog.txt']);
});
