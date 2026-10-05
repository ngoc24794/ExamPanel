// Window: min size, size/position persistence, all screens at 3 sizes with overflow/clipping metrics.
import { prepareRunDir, launchApp, waitUiReady, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { xdo, windowList, screenShot, key } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('A8');
const run = prepareRunDir('A8', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const geom = () => { const w = windowList().find((x) => x.name === 'ExamPanel'); if (!w) return null; const m = w.geom.match(/Position: (-?\d+),(-?\d+).*Geometry: (\d+)x(\d+)/); return { id: w.id, x: +m[1], y: +m[2], w: +m[3], h: +m[4] }; };
const configDir = path.join(run.dir, 'home/.config'); const wsFiles = () => { try { return execLs(configDir); } catch { return []; } };
import { execSync } from 'node:child_process';
const execLs = (d) => execSync(`find ${d} -type f`).toString().trim().split('\n').filter(Boolean);
const metrics = () => {
  const clipped = [], overflow = [];
  document.querySelectorAll('main *').forEach((el) => { const cs = getComputedStyle(el); if (cs.display === 'inline') return; const t = (el.innerText || '').trim(); if (!t || t.length > 120) return; if (el.scrollWidth > el.clientWidth + 2 && /(hidden|clip)/.test(cs.overflowX) && cs.textOverflow !== 'ellipsis') clipped.push(`${el.tagName}:${t.slice(0, 40)}`); });
  document.querySelectorAll('main button, main a').forEach((b) => { const p = b.closest('[class*="rounded"]'); if (!p || p === b) return; const r = b.getBoundingClientRect(), pr = p.getBoundingClientRect(); if (r.width && (r.right > pr.right + 1 || r.left < pr.left - 1)) overflow.push(`${b.innerText.trim().slice(0, 40)} (child right ${Math.round(r.right)} > card right ${Math.round(pr.right)})`); });
  return { hscroll: document.documentElement.scrollWidth > innerWidth + 1, docW: document.documentElement.scrollWidth, vw: innerWidth, vh: innerHeight, clipped: [...new Set(clipped)].slice(0, 10), child_overflows_card: [...new Set(overflow)].slice(0, 10), mainScroll: (() => { const m = document.querySelector('main'); return { sh: m.scrollHeight, ch: m.clientHeight, sw: m.scrollWidth, cw: m.clientWidth }; })() };
};
let app = await launchApp({ scenario: 'A8', run }); await waitUiReady(app); await sleep(800);
const g0 = geom(); R.note('initial_geometry', g0);
// ---- minimum size ----
xdo('windowsize', g0.id, '700', '450'); await sleep(900); const gMin = geom(); const inner = await app.b.getWindowSize(); const innerVp = await app.b.execute(() => [innerWidth, innerHeight]);
R.check('A8.min-size-1024x700-enforced', gMin.w >= 1024 && innerVp[1] >= 600, { requested: '700x450', x11_outer: `${gMin.w}x${gMin.h}`, wd_window: inner, viewport: innerVp }, 'window cannot be made smaller than 1024x700 (tauri.conf minWidth/minHeight)', ['extracts/A8.json']);
// ---- screens at 3 sizes ----
const routes = ['/', '/teachers', '/campuses', '/subjects', '/competencies', '/exams', '/unavailability', '/rules', '/assignments', '/statistics', '/settings'];
const table = {};
for (const [w, h] of [[1024, 700], [1280, 720], [1920, 1080]]) {
  await app.b.setWindowSize(w, h); await sleep(900); const vp = await app.b.execute(() => [innerWidth, innerHeight]);
  for (const r of routes) { await app.nav(r); await sleep(1100); if (r === '/statistics') { const cb = await app.b.$('[role="combobox"]'); if (await cb.isExisting()) { await cb.click(); await sleep(300); const o = await app.b.$('[role="option"]'); if (await o.isExisting()) { await o.click(); await sleep(1200); } } } const m = await app.b.execute(metrics); (table[`${w}x${h}`] ||= {})[r] = { ...m, vp }; await app.shot(`A8-${w}x${h}-${r.replace(/\W/g, '') || 'overview'}`, { screen: false }); }
}
fs.writeFileSync(path.join(OUT_ABS, 'extracts/A8-screens-metrics.json'), JSON.stringify(table, null, 1));
for (const [sz, rs] of Object.entries(table)) { const bad = Object.entries(rs).filter(([, m]) => m.hscroll || m.clipped.length || m.child_overflows_card.length).map(([r, m]) => ({ route: r, hscroll: m.hscroll, clipped: m.clipped.slice(0, 3), overflow: m.child_overflows_card.slice(0, 3) })); R.check(`A8.screens-usable-${sz}`, bad.length === 0 ? 'pass' : 'fail', { routes_with_issues: bad.length, details: bad.slice(0, 6) }, 'no horizontal page scroll, no clipped text, no child overflowing its card (heuristic metrics) on all 11 screens', ['extracts/A8-screens-metrics.json']); }
// ---- size/position remembered across restart ----
await app.b.setWindowSize(1100, 760); await sleep(500); xdo('windowmove', geom().id, '60', '40'); await sleep(900); const gSet = geom();
R.note('geometry_before_close', gSet);
// graceful close through the window manager (Alt+F4 -> CloseRequested)
xdo('windowfocus', gSet.id); await sleep(300); key('alt+F4'); await sleep(3500);
const alive = (() => { try { return execSync(`pgrep -u qauser -f ${JSON.stringify(run.exe)}`).toString().trim().length > 0; } catch { return false; } })();
R.check('A8.alt-f4-closes-app', !alive, { still_running: alive }, 'window close exits the process (single window app)');
const state = fs.existsSync(configDir) ? execSync(`find ${configDir} -name "*.json" -o -name ".window-state*"`).toString().trim() : ''; R.note('window_state_files', state);
try { app.driver.kill('SIGTERM'); } catch {} try { execSync(`pkill -u qauser -f ${JSON.stringify(run.exe)}`); } catch {} await sleep(800);
app = await launchApp({ scenario: 'A8', run }); await waitUiReady(app); await sleep(1500); const gAfter = geom();
R.check('A8.size-position-remembered', gAfter && Math.abs(gAfter.w - gSet.w) <= 4 && Math.abs(gAfter.h - gSet.h) <= 40 && Math.abs(gAfter.x - gSet.x) <= 6, { before: gSet, after: gAfter, state_file: state }, 'tauri-plugin-window-state restores size/position (X11 frame offsets tolerated)', ['extracts/A8.json']);
await app.stop();
