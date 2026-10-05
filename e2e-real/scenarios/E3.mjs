// Resilience: double-clicks, navigating away during runs, closing the window during optimisation, SIGKILL during writes.
import { prepareRunDir, launchApp, waitUiReady, withApp, sleep, OUT_ABS, REPO, RUN_USER } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql, key, xdo, windowList, driveFileDialog } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const R = recorder('E3');
const gold = path.resolve('../.tools/golden/q-plans.db');
const dbOf = (run) => path.join(run.dataDir, 'exam-panel.db');
const integ = (f) => sql(f, 'pragma integrity_check')[0].integrity_check;
const dbl = (app, sel) => app.b.execute((s) => { const e = document.querySelector(s); e.click(); e.click(); }, sel);
const toasts = (app) => app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | ')));
// ---------- 1a. double click on the exam dialog Save (own run: adding an exam changes feasibility) ----------
{
  const run = prepareRunDir('E3dcExam', { portable: true, seedDb: gold }); const db = dbOf(run);
  await withApp({ scenario: 'E3dcExam', run }, async (app) => {
    // exam Save
    await app.nav('/exams'); await sleep(900); await clickTid(app.b, 'add-exam-btn'); await sleep(500);
    await (await tid(app.b, 'exam-code-input')).setValue('DBL1'); await (await tid(app.b, 'exam-name-input')).setValue('Kỳ thi nhấp đúp');
    await dbl(app, '[data-testid="exam-save-btn"]'); await sleep(1500);
    const exams = sql(db, "select count(*) c from exams where code='DBL1'")[0].c; R.check('E3.double-click-exam-save', exams === 1, { rows_created: exams, toasts: await toasts(app) }, 'exactly one exam, no raw error', []);
  });
}
// ---------- 1b. double clicks on run/import/apply ----------
{
  const run = prepareRunDir('E3dc', { portable: true, seedDb: gold }); const db = dbOf(run);
  await withApp({ scenario: 'E3dc', run }, async (app) => {
    // optimizer start
    await app.nav('/assignments'); await sleep(1500); const p0 = sql(db, 'select count(*) c from plans')[0].c;
    await clickTid(app.b, 'run-optimizer-button'); await sleep(900); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Chuẩn"]')).click();
    await dbl(app, '[data-testid="start-optimize-button"]'); await sleep(6000); const p1 = sql(db, 'select count(*) c from plans')[0].c; const tt = await toasts(app);
    R.check('E3.double-click-start-optimize', p1 - p0 <= 3 && p1 - p0 >= 1, { plans_added: p1 - p0, toasts: tt }, 'one run (<=3 plans), the second click rejected quietly (optimize_busy) — no duplicate plans', []);
    // plan import apply
    await app.nav('/assignments'); await sleep(1200); await clickTid(app.b, 'import-plan-button'); await sleep(500); await clickTid(app.b, 'btn-browse-plan-file'); await driveFileDialog(path.join(REPO, 'docs/reports/phase-12/plan-grid.xlsx')); await sleep(500);
    await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500); const q0 = sql(db, "select count(*) c from plans where source='manual'")[0].c;
    await dbl(app, '[data-testid="btn-plan-import-apply"]'); await sleep(3000); const q1 = sql(db, "select count(*) c from plans where source='manual'")[0].c;
    R.check('E3.double-click-plan-import-apply', q1 - q0 === 1, { manual_plans_added: q1 - q0, toasts: await toasts(app) }, 'one imported plan', []);
    // template import apply (teachers)
    await app.nav('/teachers'); await sleep(900); await clickTid(app.b, 'import-excel-btn'); await sleep(500); await clickTid(app.b, 'import-browse-btn'); await driveFileDialog(path.join(OUT_ABS, 'extracts/B2-q-filled.xlsx')); await sleep(500); await clickTid(app.b, 'import-run-preview-btn'); await sleep(2500);
    const b0 = fs.existsSync(path.join(run.dataDir, 'backups')) ? fs.readdirSync(path.join(run.dataDir, 'backups')).filter((f) => /pre-import/.test(f)).length : 0;
    await dbl(app, '[data-testid="import-apply-btn"]'); await sleep(3000); const b1 = fs.readdirSync(path.join(run.dataDir, 'backups')).filter((f) => /pre-import/.test(f)).length; const tcount = sql(db, 'select count(*) c from teachers')[0].c;
    R.check('E3.double-click-excel-import-apply', tcount === 12 && b1 - b0 <= 1, { teachers: tcount, pre_import_backups_added: b1 - b0, toasts: await toasts(app) }, 'still 12 teachers; at most one backup (a second run would duplicate/rename)', []);
  });
  R.check('E3.integrity-after-double-clicks', integ(db) === 'ok', integ(db), 'ok');
}
// ---------- 2. navigate away during an optimisation run ----------
{
  const run = prepareRunDir('E3nav', { portable: true, seedDb: gold }); const db = dbOf(run);
  await withApp({ scenario: 'E3nav', run }, async (app) => {
    await app.nav('/assignments'); await sleep(1200); const p0 = sql(db, 'select count(*) c from plans')[0].c;
    await clickTid(app.b, 'run-optimizer-button'); await sleep(500); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Kỹ"]')).click(); await clickTid(app.b, 'start-optimize-button'); await sleep(900);
    await app.nav('/teachers'); await sleep(3500); const p1 = sql(db, 'select count(*) c from plans')[0].c; const tt = await toasts(app);
    // can a new run start afterwards (busy flag released)?
    await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'run-optimizer-button'); await sleep(500); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Nhanh"]')).click(); await clickTid(app.b, 'start-optimize-button'); await sleep(2500); const p2 = sql(db, 'select count(*) c from plans')[0].c; const tt2 = await toasts(app);
    R.check('E3.navigate-away-during-run', true, { plans_before: p0, plans_after_nav_away: p1, plans_after_next_run: p2, toasts_after_nav: tt, toasts_after_next: tt2 }, 'run cancelled/finished cleanly; busy flag released (next run saves plans) — plans saved by the aborted run are noted', []);
    R.check('E3.next-run-works-after-navigating-away', p2 > p1, { before: p1, after: p2 }, 'not stuck in optimize_busy');
    R.check('E3.navigate-away-saves-no-partial-plans', p1 === p0, { plans_before: p0, after: p1 }, 'aborted run saves nothing (same expectation as Cancel)', []);
  });
}
// ---------- 3. close the window during optimisation ----------
{
  const run = prepareRunDir('E3close', { portable: true, seedDb: gold }); const db = dbOf(run);
  const app = await launchApp({ scenario: 'E3close', run }); await waitUiReady(app);
  await app.nav('/assignments'); await sleep(1200); const p0 = sql(db, 'select count(*) c from plans')[0].c;
  await clickTid(app.b, 'run-optimizer-button'); await sleep(500); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Kỹ"]')).click(); await clickTid(app.b, 'start-optimize-button'); await sleep(1200);
  const w = windowList().find((x) => x.name === 'ExamPanel'); xdo('windowfocus', w.id); key('alt+F4'); const t0 = Date.now(); let alive = true; while (Date.now() - t0 < 15000) { alive = (() => { try { return execSync(`pgrep -u ${RUN_USER} -f ${JSON.stringify(run.exe)}`).toString().trim().length > 0; } catch { return false; } })(); if (!alive) break; await sleep(300); }
  const dt = Date.now() - t0; try { app.driver.kill('SIGTERM'); } catch {} try { execSync(`pkill -u ${RUN_USER} -f ${JSON.stringify(run.exe)}`); } catch {} await sleep(500);
  R.check('E3.close-window-during-run', !alive, { exited: !alive, ms_to_exit: dt }, 'process exits within seconds after window close while optimiser runs', []);
  R.check('E3.integrity-after-close-during-run', integ(db) === 'ok' && sql(db, 'select count(*) c from plans')[0].c >= p0, { integrity: integ(db), plans_before: p0, plans_after: sql(db, 'select count(*) c from plans')[0].c }, 'integrity ok');
  const leftover = fs.readdirSync(run.dataDir).filter((f) => /journal|wal|shm/.test(f)); R.note('leftover_sidecar_files', leftover);
}
// ---------- 4. SIGKILL during writes ----------
{
  const run = prepareRunDir('E3kill', { portable: true, seedDb: gold }); const db = dbOf(run); const rounds = [];
  for (let i = 0; i < 6; i++) {
    const app = await launchApp({ scenario: 'E3kill', run }); await waitUiReady(app);
    // heavy write loop inside the page: toggle unavailability for many teachers/exams, and restore backups (large write)
    await app.b.execute(() => { window.__stop = false; (async () => { const inv = window.__TAURI_INTERNALS__.invoke; let n = 0; while (!window.__stop && n < 4000) { for (const t of [1, 2, 3, 4, 5, 6]) for (const e of [1, 2, 3, 4]) { await inv('set_unavailability', { unavailability: { teacher_id: t, exam_id: e, reason: 'k' + n } }).catch(() => {}); n++; } } })(); });
    await sleep(500 + Math.floor(Math.random() * 1500)); const pid = app.appPids()[0];
    execSync(`kill -9 ${pid}`); await sleep(500); try { app.driver.kill('SIGTERM'); } catch {} try { execSync(`pkill -u ${RUN_USER} -f ${JSON.stringify(run.exe)}`); } catch {}
    const hot = fs.readdirSync(run.dataDir).filter((f) => /journal/.test(f)); await sleep(600);
    const app2 = await launchApp({ scenario: 'E3kill', run }); const ready = await waitUiReady(app2); await sleep(500);
    const ok = integ(db); const un = sql(db, 'select count(*) c from unavailability')[0].c; const pl = sql(db, 'select count(*) c from plans')[0].c;
    rounds.push({ round: i + 1, killed_pid: pid, hot_journal_after_kill: hot, relaunch_ready_ms: ready, integrity: ok, unavailability_rows: un, plans: pl }); await app2.stop();
  }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/E3-sigkill-rounds.json'), JSON.stringify(rounds, null, 1));
  R.check('E3.sigkill-during-write-recovers', rounds.every((r) => r.integrity === 'ok' && r.relaunch_ready_ms) && rounds.every((r) => r.plans === 4), { rounds: rounds.length, integrity_ok: rounds.filter((r) => r.integrity === 'ok').length, hot_journals_seen: rounds.filter((r) => r.hot_journal_after_kill.length).length, plans: rounds.map((r) => r.plans) }, 'after 6 SIGKILLs mid-write: relaunch OK, integrity_check ok, plans intact', ['extracts/E3-sigkill-rounds.json']);
}
