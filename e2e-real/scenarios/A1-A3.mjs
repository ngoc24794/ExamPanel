import { prepareRunDir, launchApp, waitUiReady, sleep, sampleRss, startDbus, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { procAgeMs, sql, sha256, windowList } from '../lib/native.mjs';
import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const stats = (a) => { const s = [...a].sort((x, y) => x - y); return { min: s[0], median: s[Math.floor(s.length / 2)], max: s[s.length - 1], all: a }; };
const treeOf = (d) => fs.existsSync(d) ? fs.readdirSync(d, { recursive: true }).sort() : [];

// ---------- A1 ----------
{
  const R = recorder('A1');
  const readyDriver = [], readyApp = [], rss5 = [], trees = [];
  for (let i = 0; i < 5; i++) {
    const run = prepareRunDir('A1', { portable: true });
    const app = await launchApp({ scenario: 'A1', run });
    const t = await waitUiReady(app);
    const pid = app.appPids()[0];
    readyDriver.push(t); readyApp.push(procAgeMs(pid));
    if (i === 0) await app.shot('A1-first-screen');
    await sleep(5000);
    const rs = sampleRss(); rss5.push(rs.total_kb); if (i === 0) R.note('rss_procs_run1', rs.procs);
    trees.push(treeOf(run.dataDir));
    await app.stop();
  }
  R.note('ready_ms_from_driver_launch', stats(readyDriver));
  R.note('ready_ms_from_app_process_start(approx)', stats(readyApp));
  R.note('idle_rss_kb_total_after_5s(app+webkit+driver)', stats(rss5));
  R.note('data_trees', trees);
  const expect = ['exam-panel.db', 'logs', 'logs/app.log'];
  R.check('A1.tree', expect.every((x) => trees[0].includes(x)), trees[0], expect);
  R.check('A1.db-name-matches-docs', trees[0].includes('exam-panel.db'), 'exam-panel.db', 'DOC-TOI.txt/SPEC/ADR: exam-panel.db (v0.1.0-checklist & user-guide 08 say exampanel.db)');
  R.check('A1.no-backups-dir-on-fresh-start', !trees[0].includes('backups'), trees[0], 'informational');
  R.check('A1.timing', 'pass', { driver: stats(readyDriver), app: stats(readyApp) }, 'informational only (<1.5 s on a user PC per v0.1.0 checklist; sandbox not comparable)');
}
// ---------- A2 ----------
{
  const R = recorder('A2');
  const run = prepareRunDir('A2', { portable: false });
  const app = await launchApp({ scenario: 'A2', run });
  await waitUiReady(app); await sleep(1500);
  const xdgApp = path.join(run.xdgData, 'ExamPanel/data');
  const t = { next_to_exe: treeOf(path.join(run.dir, 'app')), xdg: treeOf(xdgApp) };
  R.note('trees', t);
  R.check('A2.data-in-xdg', t.xdg.includes('exam-panel.db'), t.xdg, 'exam-panel.db under $XDG_DATA_HOME/ExamPanel/data');
  R.check('A2.nothing-next-to-exe', !t.next_to_exe.includes('data'), t.next_to_exe, 'no data/ beside exe');
  const info = await app.invoke('get_app_info');
  R.note('get_app_info', info);
  R.check('A2.app_info.is_portable=false', info.ok && info.v.is_portable === false, info.v && { is_portable: info.v.is_portable, data_dir: info.v.data_dir }, 'is_portable false');
  await app.shot('A2-nonportable');
  await app.stop();
}
// ---------- A3 ----------
{
  const R = recorder('A3');
  const run = prepareRunDir('A3', { portable: true });
  const dbus = startDbus(run.dir);
  const app = await launchApp({ scenario: 'A3', run, dbus });
  await waitUiReady(app); await sleep(1000);
  const before = app.appPids();
  const wBefore = windowList().map((w) => w.name);
  // second launch in the same dbus session as the same user
  const t0 = Date.now();
  let exit = null, out = '';
  try {
    out = execSync(`runuser -u qauser -- env HOME=${run.dir}/home XDG_DATA_HOME=${run.dir}/home/.local/share XDG_CONFIG_HOME=${run.dir}/home/.config XDG_CACHE_HOME=${run.dir}/home/.cache DISPLAY=:99 DBUS_SESSION_BUS_ADDRESS='${dbus.address}' WEBKIT_DISABLE_COMPOSITING_MODE=1 timeout 20 ${run.exe} 2>&1; echo EXIT=$?`).toString();
  } catch (e) { out = (e.stdout || '').toString() + String(e); }
  const dt = Date.now() - t0;
  fs.writeFileSync(path.join(run.dir, 'out/second-launch.txt'), out);
  const m = out.match(/EXIT=(\d+)/); exit = m ? Number(m[1]) : null;
  await sleep(1000);
  const after = app.appPids();
  const wAfter = windowList().map((w) => w.name);
  R.note('second_launch', { exit, duration_ms: dt, out: out.slice(-600), pids_before: before, pids_after: after, windows_before: wBefore, windows_after: wAfter });
  R.check('A3.second-exits-quickly-code0', exit === 0 && dt < 8000, { exit, dt }, 'exit 0 within seconds', [`.tools/qa-run/${path.basename(run.dir)}/out/second-launch.txt`]);
  R.check('A3.no-extra-app-process', after.length === before.length, { before, after }, 'same PIDs');
  R.check('A3.no-error-dialog', JSON.stringify(wBefore) === JSON.stringify(wAfter), { wBefore, wAfter }, 'no new windows');
  // second DB connection: check open file handles on db
  let fds = ''; try { fds = execSync(`ls -l /proc/${after[0]}/fd | grep -c exam-panel.db || true`).toString().trim(); } catch {}
  R.note('db_fd_count_first_process', fds);
  await app.shot('A3-after-second-launch');
  const ok = await app.b.execute(() => document.body.innerText.length > 50);
  R.check('A3.first-instance-still-usable', ok, ok, 'true');
  await app.stop();
}
