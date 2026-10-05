import { prepareRunDir, launchApp, waitUiReady, launchRaw, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { waitWindow, screenShot, windowList, xdo, sha256, PY } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, execFileSync } from 'node:child_process';
const tree = (d) => fs.existsSync(d) ? fs.readdirSync(d, { recursive: true }).sort() : [];
const mainThreadState = (pid) => { try { const t = `/proc/${pid}/task/${pid}`; return { state: fs.readFileSync(t + '/stat', 'utf8').split(') ')[1].split(' ')[0], wchan: fs.readFileSync(t + '/wchan', 'utf8') }; } catch { return null; } };

async function observe(R, tag, run, { wait = 12000 } = {}) {
  const h = launchRaw(run);
  await sleep(wait);
  const pids = (() => { try { return execSync(`pgrep -u qauser -f ${run.exe}`).toString().trim().split('\n').map(Number); } catch { return []; } })();
  const wins = windowList().map((w) => w.name);
  const shot = screenShot(`${tag}-after-${wait / 1000}s`);
  const st = { pids, windows: wins, exited: h.st.exited, main_thread: pids[0] ? mainThreadState(pids[0]) : null, log_tail: fs.readFileSync(h.logf, 'utf8').slice(-400) };
  R.note(tag + '_observed', st);
  return { h, st, shot };
}

// ----- A4 read-only portable folder -----
{
  const R = recorder('A4');
  const run = prepareRunDir('A4', { portable: true });
  execSync(`chmod 555 ${run.dir}/app`);
  let canWrite = true; try { execSync(`runuser -u qauser -- touch ${run.dir}/app/x 2>&1`, { stdio: 'pipe' }); } catch { canWrite = false; }
  R.note('qauser_can_write_exe_dir', canWrite);
  const { h, st, shot } = await observe(R, 'A4', run);
  const dialogVisible = st.windows.some((n) => /Lỗi ghi dữ liệu/.test(n));
  R.check('A4.dialog-shown', dialogVisible, { windows: st.windows, main_thread: st.main_thread }, 'Vietnamese dialog "ExamPanel - Lỗi ghi dữ liệu di động" with OK/Cancel', [`screenshots/A4-after-12s.png`, 'logs/A4-hang-main-thread-backtrace.txt']);
  R.check('A4.app-not-hung', !(st.main_thread && st.main_thread.wchan.includes('futex') && !dialogVisible), st.main_thread, 'UI loads or dialog shown', [`screenshots/A4-after-12s.png`]);
  R.check('A4.option-use-app-data-works', 'blocked', 'dialog never rendered; cannot press OK', 'DB created in app-data dir', ['logs/A4-hang-main-thread-backtrace.txt']);
  R.check('A4.option-exit-works', 'blocked', 'dialog never rendered; cannot press Cancel', 'exit code 0', []);
  R.note('xdg_tree', tree(path.join(run.xdgData, 'ExamPanel')));
  h.kill(); execSync(`chmod 755 ${run.dir}/app`);
}
// helper: create a valid DB by running the app once
async function makeDb(scenario) {
  const run = prepareRunDir(scenario, { portable: true });
  const app = await launchApp({ scenario, run }); await waitUiReady(app); await sleep(800); await app.stop();
  return run;
}
// ----- A5 DB newer than app -----
{
  const R = recorder('A5');
  const run = await makeDb('A5');
  const db = path.join(run.dataDir, 'exam-panel.db');
  execFileSync(PY, ['-c', `import sqlite3,sys;c=sqlite3.connect(sys.argv[1]);c.execute('PRAGMA user_version=99');c.commit()`, db]);
  execSync(`chown -R qauser:qauser ${run.dir}`);
  const before = sha256(db);
  const { h, st } = await observe(R, 'A5', run);
  h.kill(); await sleep(500);
  const after = sha256(db);
  const dialogVisible = st.windows.some((n) => /không tương thích|Phiên bản/.test(n));
  R.check('A5.friendly-message', dialogVisible, { windows: st.windows, main_thread: st.main_thread }, 'Vietnamese dialog about newer DB version', ['screenshots/A5-after-12s.png']);
  R.check('A5.no-crash-no-hang', !(st.main_thread && st.main_thread.wchan.includes('futex') && !dialogVisible), st.main_thread, 'process exits cleanly after message or UI shows an error');
  R.check('A5.db-hash-unchanged', before === after, { before, after }, 'identical sha256', []);
  // headless path: smoke-test shows what the non-dialog code path does for the same DB
  let smoke = '';
  try { smoke = execSync(`runuser -u qauser -- env HOME=${run.dir}/home XDG_DATA_HOME=${run.dir}/home/.local/share DISPLAY=:99 ${run.exe} --smoke-test 2>&1; echo EXIT=$?`).toString(); } catch (e) { smoke = String(e.stdout || e); }
  fs.writeFileSync(path.join(OUT_ABS, 'logs/A5-smoke-test-newer-db.txt'), smoke);
  R.note('smoke_test_tail', smoke.slice(-500));
  R.check('A5.db-hash-unchanged-after-smoke', sha256(db) === before, { before, after: sha256(db) }, 'identical sha256', ['logs/A5-smoke-test-newer-db.txt']);
}
// ----- A6 corrupted DB + automatic backup present -----
{
  const R = recorder('A6');
  const run = await makeDb('A6');
  const db = path.join(run.dataDir, 'exam-panel.db');
  fs.mkdirSync(path.join(run.dataDir, 'backups'), { recursive: true });
  const bk = path.join(run.dataDir, 'backups', 'exampanel-backup-manual-1700000000.db');
  fs.copyFileSync(db, bk);
  const fd = fs.openSync(db, 'r+'); fs.writeSync(fd, Buffer.from('THIS IS NOT A SQLITE FILE '.repeat(40)), 0); fs.closeSync(fd);
  execSync(`chown -R qauser:qauser ${run.dir}`);
  const { h, st } = await observe(R, 'A6', run);
  const dialogVisible = st.windows.some((n) => /Lỗi cơ sở dữ liệu/.test(n));
  R.check('A6.restore-offer-dialog', dialogVisible, { windows: st.windows, main_thread: st.main_thread }, 'dialog offering restore from latest backup', ['screenshots/A6-after-12s.png']);
  R.check('A6.restore-works', 'blocked', 'dialog never rendered', 'DB restored from backups/', []);
  h.kill();
}
