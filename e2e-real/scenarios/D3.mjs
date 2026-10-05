import { prepareRunDir, withApp, launchApp, waitUiReady, sleep, OUT_ABS, REPO } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { driveFileDialog, sql, sha256, screenShot, PY } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, execFileSync } from 'node:child_process';
const R = recorder('D3');
const run = prepareRunDir('D3', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db'); const bdir = path.join(run.dataDir, 'backups');
const logical = (f) => execSync(`sqlite3 ${JSON.stringify(f)} .dump | grep -v 'INSERT INTO sqlite_sequence' | sha256sum`).toString().split(' ')[0];
const counts = (f) => sql(f, "select (select count(*) from teachers) t,(select count(*) from plans) p,(select count(*) from school_years) y,(select count(*) from assignments) a")[0];
const integrity = (f) => sql(f, 'pragma integrity_check')[0].integrity_check;
const autoList = () => fs.existsSync(bdir) ? fs.readdirSync(bdir).sort() : [];
await withApp({ scenario: 'D3', run }, async (app) => {
  // ---------- D3.1 backup now via UI + dialog ----------
  await app.nav('/settings'); await sleep(1200);
  await clickTid(app.b, 'backup-now-btn'); const b1 = path.join(run.dir, 'out/backup-b1.db');
  const dr = await driveFileDialog(b1, { shotName: 'D3-backup-save-dialog' }); await sleep(2000);
  R.check('D3.backup-now-file-written', dr.ok && fs.existsSync(b1), { dr, size: fs.existsSync(b1) && fs.statSync(b1).size }, 'file saved via GTK dialog, size > 100 KB (phase-9 step 7)', ['screenshots/D3-backup-save-dialog.png']);
  R.check('D3.backup-integrity-and-counts', integrity(b1) === 'ok' && JSON.stringify(counts(b1)) === JSON.stringify(counts(db)), { integrity: integrity(b1), backup: counts(b1), live: counts(db) }, 'integrity_check ok; counts equal live DB');
  R.check('D3.backup-logical-equals-live', logical(b1) === logical(db), 'compared .dump (excluding sqlite_sequence)', 'identical');
  // ---------- mutate after the backup ----------
  const sy = (await app.invoke('create_school_year', { input: { name: '2027-2028', is_current: true, copy_grades_from: null } })); R.note('create_school_year_result', sy.ok ? 'ok' : sy.e);
  const ts = (await app.invoke('list_teachers', {})).v; await app.invoke('update_teacher', { teacher: { ...ts[0], full_name: 'ZZ-CHANGED-AFTER-BACKUP' } });
  const afterMut = counts(db); R.note('counts_after_mutation', afterMut);
  // ---------- D3.2 restore from chosen file via UI ----------
  await app.nav('/settings'); await sleep(800); const autoBefore = autoList();
  await clickTid(app.b, 'restore-file-btn'); const dr2 = await driveFileDialog(b1, { shotName: 'D3-restore-open-dialog' }); await sleep(1500);
  const dtext = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); fs.writeFileSync(path.join(OUT_ABS, 'extracts/D3-restore-dialog-valid.txt'), dtext); await app.shot('D3-restore-validate', { screen: false });
  R.check('D3.restore-validation-dialog-summary', /Số năm học|năm học/i.test(dtext) && dr2.ok, dtext.replace(/\n+/g, ' | ').slice(0, 300), 'dialog shows years/teachers/plans/schema version (phase-9 step 8.4)', ['extracts/D3-restore-dialog-valid.txt']);
  await clickTid(app.b, 'confirm-restore-btn'); await sleep(2500);
  const autoAfter = autoList(); R.check('D3.restore-creates-pre-restore-backup', autoAfter.length === autoBefore.length + 1 && autoAfter.some((f) => /pre-restore/.test(f)), { before: autoBefore, after: autoAfter }, 'new exampanel-backup-pre-restore-*.db', []);
  R.check('D3.restore-data-equals-backup', logical(db) === logical(b1) && JSON.stringify(counts(db)) === JSON.stringify(counts(b1)), { live: counts(db), backup: counts(b1) }, 'DB logical content == backup');
  // UI state reset: teachers page, years selector, plans
  await app.nav('/teachers'); await sleep(1500); const tt = await bodyText(app.b);
  R.check('D3.restore-ui-teachers-reset', !/ZZ-CHANGED/.test(tt) && /Cô Hiền/.test(tt), { shows_changed_name: /ZZ-CHANGED/.test(tt) }, 'teacher list shows restored names (cache cleared)');
  const hdr = await app.b.execute(() => document.querySelector('header')?.innerText?.replace(/\n+/g, ' | ')); R.note('header_after_restore', hdr);
  R.check('D3.restore-ui-current-year-reset', /2026/.test(hdr || '') && !/2027-2028|2027 - 2028/.test(hdr || ''), hdr, 'current year selector shows 2026-2027 again');
  await app.nav('/assignments'); await sleep(1500); const at = await bodyText(app.b);
  R.check('D3.restore-ui-plans-reset', /Lịch sử phương án \(4\)/.test(at), at.match(/Lịch sử phương án \(\d+\)/)?.[0], 'history shows 4 plans (as in backup)');
  // ---------- D3.3 rejection of invalid / newer / truncated files ----------
  const bad = {};
  fs.writeFileSync(path.join(run.dir, 'out/not-a-db.db'), 'this is not a sqlite database '.repeat(300)); bad.garbage = path.join(run.dir, 'out/not-a-db.db');
  fs.copyFileSync(b1, path.join(run.dir, 'out/newer.db')); execFileSync(PY, ['-c', "import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA user_version=99');c.commit()", path.join(run.dir, 'out/newer.db')]); bad.newer_version = path.join(run.dir, 'out/newer.db');
  const buf = fs.readFileSync(b1); fs.writeFileSync(path.join(run.dir, 'out/truncated.db'), buf.subarray(0, Math.floor(buf.length / 2))); bad.truncated = path.join(run.dir, 'out/truncated.db');
  execSync(`chown -R qauser:qauser ${run.dir}`);
  for (const [k, f] of Object.entries(bad)) {
    await app.nav('/settings'); await sleep(700); const h = sha256(db);
    await clickTid(app.b, 'restore-file-btn'); const d = await driveFileDialog(f); await sleep(1500);
    const txt = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); const disabled = await app.b.execute(() => document.querySelector('[data-testid="confirm-restore-btn"]')?.disabled);
    fs.writeFileSync(path.join(OUT_ABS, `extracts/D3-restore-dialog-${k}.txt`), txt); await app.shot(`D3-restore-reject-${k}`, { screen: false });
    R.check(`D3.reject-${k}`, disabled === true && sha256(db) === h, { confirm_disabled: disabled, db_unchanged: sha256(db) === h, dialog: txt.replace(/\n+/g, ' | ').slice(0, 260) }, 'confirm disabled, clear error, DB untouched', [`extracts/D3-restore-dialog-${k}.txt`, `screenshots/D3-restore-reject-${k}-page.png`]);
    await app.b.keys('Escape'); await sleep(500);
  }
  // ---------- D3.4 restore from the automatic list ----------
  await app.nav('/settings'); await sleep(1200);
  const rows = await app.b.$$('tr'); const autos = []; for (const r of rows) { const t = await r.getText(); if (/exampanel-backup-/.test(t)) autos.push(t.replace(/\n/g, ' ').slice(0, 100)); }
  R.note('auto_list_rows', autos);
  const restoreBtns = await app.b.$$('//tr[contains(.,"exampanel-backup-")]//button'); R.note('auto_restore_buttons', restoreBtns.length);
  await ts.length; await app.invoke('update_teacher', { teacher: { ...(await app.invoke('list_teachers', {})).v[1], full_name: 'ZZ-SECOND-CHANGE' } });
  if (restoreBtns.length) { await app.b.execute((el) => el.click(), restoreBtns[0]); await sleep(1500); const t2 = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); R.note('auto_restore_dialog', t2.replace(/\n+/g, ' | ').slice(0, 300)); await clickTid(app.b, 'confirm-restore-btn'); await sleep(2500); }
  const names = sql(db, 'select full_name from teachers').map((x) => x.full_name);
  R.check('D3.restore-from-automatic-list', restoreBtns.length > 0 && !names.some((n) => /ZZ-SECOND-CHANGE/.test(n)), { buttons: restoreBtns.length, names_has_second_change: names.some((n) => /ZZ-SECOND/.test(n)) }, 'automatic backup restores previous state (phase-9 step 9)');
});
