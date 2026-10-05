// MOCK-DATA-REPLAY print proxy: real UI bundle (ui/dist) in Chromium; IPC answered with recorded REAL Rust outputs. NOT the real app, NOT WebView2.
import { createRequire } from 'node:module';
import http from 'node:http'; import fs from 'node:fs'; import path from 'node:path';
import { OUT_ABS, REPO } from '../lib/harness.mjs';
const require = createRequire(path.join(REPO, 'ui/package.json'));
const { chromium } = require('@playwright/test');
const replay = JSON.parse(fs.readFileSync(path.join(OUT_ABS, 'extracts/replay-ipc.json'), 'utf8'));
const dist = path.join(REPO, 'ui/dist'); const mime = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.json': 'application/json' };
const srv = http.createServer((q, r) => { let f = path.join(dist, decodeURIComponent(q.url.split('?')[0])); if (q.url === '/' || !fs.existsSync(f) || fs.statSync(f).isDirectory()) f = path.join(dist, 'index.html'); r.writeHead(200, { 'content-type': mime[path.extname(f)] || 'application/octet-stream' }); fs.createReadStream(f).pipe(r); }).listen(0);
const port = srv.address().port;
const browser = await chromium.launch({ executablePath: '/opt/pw-browsers/chromium-1194/chrome-linux/chrome', args: ['--no-sandbox'] });
const results = [];
for (const [name, route, landscape, planId] of [['plan-final', '/print/plan/1', true, 1], ['plan-draft', '/print/plan/2', true, 2], ['notices-final', '/print/notices/1', false, 1], ['notices-draft', '/print/notices/2', false, 2]]) {
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, locale: 'vi-VN' }); const page = await ctx.newPage(); const missing = [];
  await page.addInitScript((replay) => { window.__replayMissing = []; window.__TAURI_INTERNALS__ = { transformCallback: () => 1, metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } }, invoke: async (cmd, args) => { if (cmd === 'get_plan') return replay.get_plan[args.id]; if (cmd in replay) return replay[cmd]; if (cmd === 'plan_status') return { is_stale: false, hard_violations: [], score_now: replay.get_plan[args.id]?.score_report }; window.__replayMissing.push(cmd); throw { code: 'replay_missing', params: { cmd } }; } }; }, replay);
  const errs = []; page.on('console', (m) => { if (['error', 'warning'].includes(m.type())) errs.push(m.text().slice(0, 200)); }); page.on('pageerror', (e) => errs.push('pageerror ' + e.message));
  await page.goto(`http://127.0.0.1:${port}/#${route}`); await page.waitForTimeout(2500);
  const miss = await page.evaluate(() => window.__replayMissing);
  const pdf = path.join(OUT_ABS, `pdfs/D2-chromium-replay-${name}.pdf`);
  await page.pdf({ path: pdf, format: 'A4', landscape, printBackground: true, preferCSSPageSize: true });
  await page.screenshot({ path: path.join(OUT_ABS, `screenshots/D2-chromium-replay-${name}-screen.png`), fullPage: false });
  results.push({ name, route, missing_commands: miss, console: errs.slice(0, 5) }); await ctx.close();
}
await browser.close(); srv.close();
fs.writeFileSync(path.join(OUT_ABS, 'extracts/D2-chromium-replay-run.json'), JSON.stringify(results, null, 1)); console.log(JSON.stringify(results, null, 1));
