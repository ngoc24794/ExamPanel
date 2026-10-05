// Helpers for native (GTK) windows via xdotool + X screen shots, and read-only inspection utilities.
import { execFileSync, execSync } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import { DISPLAY, OUT_ABS, REPO, sleep } from './harness.mjs';

const env = { ...process.env, DISPLAY };
export function xdo(...args) { try { return execFileSync('xdotool', args, { env }).toString().trim(); } catch (e) { return ''; } }
export function windowList() {
  const ids = xdo('search', '--onlyvisible', '--name', '.').split('\n').filter(Boolean);
  return ids.map((id) => ({ id, name: xdo('getwindowname', id), geom: xdo('getwindowgeometry', id).replace(/\n/g, ' ') }));
}
export async function waitWindow(nameRe, timeout = 15000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeout) { const w = windowList().find((w) => nameRe.test(w.name)); if (w) return w; await sleep(250); }
  return null;
}
export function screenShot(name) {
  const f = path.join(OUT_ABS, 'screenshots', name + '.png');
  fs.mkdirSync(path.dirname(f), { recursive: true });
  try { execFileSync('import', ['-window', 'root', f], { env }); } catch {}
  return f;
}
export function key(...k) { xdo('key', '--clearmodifiers', ...k); }
export function typeText(t) { xdo('type', '--clearmodifiers', '--delay', '20', t); }

export const PY = path.join(REPO, '.tools/venv/bin/python');
/** read-only sqlite query via python; returns parsed JSON rows */
export function sql(dbPath, query, params = []) {
  const code = `import sqlite3,json,sys\nc=sqlite3.connect('file:'+sys.argv[1]+'?mode=ro',uri=True)\nc.row_factory=sqlite3.Row\nr=[dict(x) for x in c.execute(sys.argv[2],json.loads(sys.argv[3]))]\nprint(json.dumps(r,ensure_ascii=False,default=str))`;
  const out = execFileSync(PY, ['-c', code, dbPath, query, JSON.stringify(params)]).toString();
  return JSON.parse(out);
}
export function sha256(f) { return execSync(`sha256sum ${JSON.stringify(f)}`).toString().split(' ')[0]; }
export function procAgeMs(pid) {
  try {
    const stat = fs.readFileSync(`/proc/${pid}/stat`, 'utf8'); const st = Number(stat.slice(stat.lastIndexOf(')') + 2).split(' ')[19]);
    const up = Number(fs.readFileSync('/proc/uptime', 'utf8').split(' ')[0]);
    return Math.round((up - st / 100) * 1000);
  } catch { return null; }
}

/** Drive a GTK file chooser (open or save): wait for the dialog, ctrl+L, type full path, Return. */
export async function driveFileDialog(fullPath, { timeout = 15000, shotName = null } = {}) {
  const t0 = Date.now(); let w = null;
  while (Date.now() - t0 < timeout) {
    w = windowList().find((x) => !/^ExamPanel$/.test(x.name)) || null;
    if (w) break; await sleep(250);
  }
  if (!w) return { ok: false, reason: 'dialog window not found', windows: windowList() };
  await sleep(700);
  if (shotName) screenShot(shotName);
  key('ctrl+l'); await sleep(300); key('ctrl+a'); await sleep(150);
  typeText(fullPath); await sleep(400);
  key('Return'); await sleep(900);
  // a second Return can be needed when the chooser first completes the folder; check window gone
  let still = windowList().find((x) => x.id === w.id);
  if (still) { key('Return'); await sleep(900); still = windowList().find((x) => x.id === w.id); }
  return { ok: !still, dialog: w.name, stillOpen: !!still };
}
