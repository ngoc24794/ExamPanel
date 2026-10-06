// RA-001 regression: the startup dialogs must be answerable and each answer must take effect.
// A4 read-only portable folder (OK -> fall back to the per-user data dir, Cancel -> exit 0),
// A5 database newer than the app (OK -> exit 1), A6 damaged DB with a backup (OK -> restored, app starts).
import { prepareRunDir, launchApp, waitUiReady, launchRaw, sleep } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { waitWindow, windowList, xdo, key, PY, sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, execFileSync } from 'node:child_process';

async function waitFor(fn, ms = 20000, step = 300) { const t0 = Date.now(); while (Date.now() - t0 < ms) { const v = fn(); if (v) return v; await sleep(step); } return null; }
async function makeDb(scenario) { const run = prepareRunDir(scenario, { portable: true }); const app = await launchApp({ scenario, run }); await waitUiReady(app); await sleep(800); await app.stop(); return run; }
// GTK OK/Cancel dialogs focus Cancel first: Tab moves to OK, Return activates it. A one-button
// message dialog is closed with Return alone.
const answer = async (w, k) => {
  await sleep(1000); xdo('windowfocus', w.id); await sleep(400);
  if (k === 'ok-of-two') { key('Tab'); await sleep(300); key('Return'); } else key(k);
};
const mainWindow = () => windowList().find((w) => /^ExamPanel$/.test(w.name));

{ // A4 OK
  const R = recorder('A4-actions');
  const run = prepareRunDir('A4ok', { portable: true });
  execSync(`chmod 555 ${run.dir}/app`);
  const h = launchRaw(run);
  const dlg = await waitWindow(/Lỗi ghi dữ liệu di động/, 20000);
  R.check('A4.ok.dialog-shown', !!dlg, dlg && dlg.name, 'dialog visible within 20 s');
  if (dlg) await answer(dlg, 'ok-of-two');
  const main = await waitFor(mainWindow, 25000);
  const db = path.join(run.xdgData, 'ExamPanel/data/exam-panel.db');
  R.check('A4.ok.app-continues-with-app-data', !!main && fs.existsSync(db) && h.st.exited === null, { main: !!main, db_exists: fs.existsSync(db), exited: h.st.exited }, 'main window shown, DB created in the per-user data dir, process alive');
  h.kill(); execSync(`chmod 755 ${run.dir}/app`);
}
{ // A4 Cancel
  const R = recorder('A4-actions');
  const run = prepareRunDir('A4cancel', { portable: true });
  execSync(`chmod 555 ${run.dir}/app`);
  const h = launchRaw(run);
  const dlg = await waitWindow(/Lỗi ghi dữ liệu di động/, 20000);
  if (dlg) await answer(dlg, 'Escape');
  const exited = await waitFor(() => h.st.exited, 15000);
  R.check('A4.cancel.app-exits-cleanly', !!exited && exited.code === 0, { exited, data_created: fs.existsSync(path.join(run.xdgData, 'ExamPanel/data/exam-panel.db')) }, 'exit code 0 and no data created');
  h.kill(); execSync(`chmod 755 ${run.dir}/app`);
}
{ // A5 newer DB: OK closes the app
  const R = recorder('A4-actions');
  const run = await makeDb('A5ok');
  const db = path.join(run.dataDir, 'exam-panel.db');
  execFileSync(PY, ['-c', `import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA user_version=99');c.commit()`, db]);
  execSync(`chown -R qauser:qauser ${run.dir}`);
  const h = launchRaw(run);
  const dlg = await waitWindow(/Phiên bản không tương thích/, 20000);
  R.check('A5.ok.dialog-shown', !!dlg, dlg && dlg.name, 'dialog visible');
  if (dlg) await answer(dlg, 'Return');
  const exited = await waitFor(() => h.st.exited, 15000);
  R.check('A5.ok.app-exits', !!exited && exited.code === 1, exited, 'exit code 1 after the message');
  h.kill();
}
{ // A6 damaged DB: OK restores the latest backup and the app starts
  const R = recorder('A4-actions');
  const run = await makeDb('A6ok');
  const db = path.join(run.dataDir, 'exam-panel.db');
  fs.mkdirSync(path.join(run.dataDir, 'backups'), { recursive: true });
  fs.copyFileSync(db, path.join(run.dataDir, 'backups', 'exampanel-backup-manual-1700000000.db'));
  const fd = fs.openSync(db, 'r+'); fs.writeSync(fd, Buffer.from('THIS IS NOT A SQLITE FILE '.repeat(40)), 0); fs.closeSync(fd);
  execSync(`chown -R qauser:qauser ${run.dir}`);
  const h = launchRaw(run);
  const dlg = await waitWindow(/Lỗi cơ sở dữ liệu/, 20000);
  R.check('A6.ok.dialog-shown', !!dlg, dlg && dlg.name, 'dialog visible');
  if (dlg) await answer(dlg, 'ok-of-two');
  const main = await waitFor(mainWindow, 25000);
  let integrity = null; try { integrity = sql(db, 'PRAGMA integrity_check')[0].integrity_check; } catch (e) { integrity = String(e).slice(0, 80); }
  R.check('A6.ok.restored-and-started', !!main && integrity === 'ok', { main: !!main, integrity }, 'main window shown and DB integrity ok after restore');
  h.kill();
}
