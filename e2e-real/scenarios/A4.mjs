import { prepareRunDir, launchRaw, sleep } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { waitWindow, screenShot, key, windowList, xdo } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const R = recorder('A4');
const tree = (d) => fs.existsSync(d) ? fs.readdirSync(d, { recursive: true }).sort() : [];
for (const choice of ['ok', 'cancel']) {
  const run = prepareRunDir('A4', { portable: true });
  execSync(`chmod 555 ${run.dir}/app`);
  // sanity: qauser really cannot write
  let canWrite = true; try { execSync(`runuser -u qauser -- touch ${run.dir}/app/x 2>&1`); } catch { canWrite = false; }
  R.note('qauser_can_write_app_dir_' + choice, canWrite);
  const h = launchRaw(run);
  const w = await waitWindow(/Lỗi ghi dữ liệu|ExamPanel - /, 20000);
  R.note('dialog_window_' + choice, w);
  const shot = screenShot(`A4-${choice}-dialog`);
  R.check(`A4.${choice}.dialog-shown`, !!w && /di động/.test(w.name), w, 'dialog titled "ExamPanel - Lỗi ghi dữ liệu di động"', [shot]);
  const wl = windowList(); R.note('windows_' + choice, wl);
  if (choice === 'ok') {
    key('Return');
  } else {
    key('Escape');
  }
  await sleep(6000);
  const shot2 = screenShot(`A4-${choice}-after`);
  const xdg = path.join(run.xdgData, 'ExamPanel/data');
  const state = { exited: h.st.exited, windows: windowList().map((x) => x.name), xdg_tree: tree(xdg), exe_dir_tree: tree(path.join(run.dir, 'app')) };
  R.note('after_' + choice, state);
  if (choice === 'ok') {
    R.check('A4.ok.fallback-to-app-data', state.xdg_tree.includes('exam-panel.db') && !state.exited, state, 'DB created under XDG data dir; app still running', [shot2]);
  } else {
    R.check('A4.cancel.app-exits-cleanly', state.exited && state.exited.code === 0 && state.xdg_tree.length === 0, state, 'exit code 0, no data created', [shot2]);
  }
  h.kill(); execSync(`chmod 755 ${run.dir}/app`);
}
