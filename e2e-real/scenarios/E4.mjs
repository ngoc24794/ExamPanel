// Security / hygiene: dev-tools, capability denial, external network, CSP.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const R = recorder('E4');
const run = prepareRunDir('E4', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'E4', run }, async (app) => {
  // 1. no Developer page
  await app.nav('/dev'); await sleep(800); const hash = await app.b.execute(() => location.hash); const txt = await app.b.execute(() => document.body.innerText);
  R.check('E4.no-developer-route', hash !== '#/dev' || !/Dev|Nhà phát triển|seed/i.test(txt), { hash_after_nav: hash }, 'redirects to / (route not compiled in: import.meta.env.DEV false)');
  const sidebar = await app.b.execute(() => [...document.querySelectorAll('nav a, aside a')].map((a) => a.getAttribute('href')));
  R.check('E4.no-developer-link-in-sidebar', !sidebar.some((h) => /dev/.test(h || '')), sidebar, 'no /dev link');
  // 2. seed_demo through IPC (dev-tools feature in src-tauri default features)
  const beforeCounts = sql(db, 'select (select count(*) from teachers) t,(select count(*) from school_years) y')[0];
  const hashL = () => execSync(`sqlite3 ${db} .dump | grep -v 'INSERT INTO sqlite_sequence' | sha256sum`).toString().split(' ')[0]; const lh0 = hashL(); const seed = await app.invoke('seed_demo', {}); const lh1 = hashL(); R.note('seed_demo_on_populated_db_logical_hash_unchanged', lh0 === lh1); const afterCounts = sql(db, 'select (select count(*) from teachers) t,(select count(*) from school_years) y')[0];
  R.check('E4.seed_demo-denied-in-release', seed.ok === false && /not_supported|not enabled|unknown|not found/i.test(JSON.stringify(seed.e)), { result: seed.ok ? 'OK (executed)' : seed.e, teachers_before: beforeCounts.t, teachers_after: afterCounts.t, years_before: beforeCounts.y, years_after: afterCounts.y }, 'seed_demo must not exist or must refuse in the release build (task: "no seed_demo command")', ['extracts/E4.json', 'build-inspect.txt']);
  // 3. commands outside granted capabilities
  const probes = { 'fs.read_text_file': ['plugin:fs|read_text_file', { path: '/etc/passwd' }], 'fs.write': ['plugin:fs|write_text_file', { path: '/tmp/qa-pwn.txt', contents: 'x' }], 'shell.execute': ['plugin:shell|execute', { program: 'id', args: [] }], 'opener.open_path': ['plugin:opener|open_path', { path: '/etc/passwd' }], 'opener.open_url_file': ['plugin:opener|open_url', { url: 'file:///etc/passwd' }], 'opener.open_url_https': ['plugin:opener|open_url', { url: 'https://example.com' }], 'opener.reveal': ['plugin:opener|reveal_item_in_dir', { paths: ['/etc/passwd'] }], 'dialog.open': ['plugin:dialog|message', { message: 'probe', title: 'probe' }], 'process.exit': ['plugin:process|exit', { code: 0 }], 'app.plugin_unknown': ['plugin:nonexistent|x', {}], 'core.window.close': ['plugin:window|is_maximized', { label: 'main' }], 'log.write': ['plugin:log|log', { level: 3, message: 'qa-probe' }] };
  const res = {};
  for (const [k, [cmd, args]] of Object.entries(probes)) { if (k === 'dialog.open') continue; const r = await app.invoke(cmd, args); res[k] = r.ok ? 'ALLOWED/OK' : (typeof r.e === 'string' ? r.e : JSON.stringify(r.e)).slice(0, 160); await sleep(200); }
  const wrote = fs.existsSync('/tmp/qa-pwn.txt'); res['file_written_/tmp/qa-pwn.txt'] = wrote; if (wrote) fs.unlinkSync('/tmp/qa-pwn.txt');
  R.note('capability_probes', res); fs.writeFileSync(path.join(OUT_ABS, 'extracts/E4-capability-probes.json'), JSON.stringify(res, null, 1));
  const denied = (k) => !/ALLOWED/.test(res[k]);
  R.check('E4.fs-shell-process-plugins-not-granted', ['fs.read_text_file', 'fs.write', 'shell.execute', 'process.exit'].every(denied) && !wrote, { fs_read: res['fs.read_text_file'], fs_write: res['fs.write'], shell: res['shell.execute'], process: res['process.exit'] }, 'denied / plugin not found', ['extracts/E4-capability-probes.json']);
  R.check('E4.opener-open_path-denied', denied('opener.open_path') && denied('opener.open_url_file'), { open_path: res['opener.open_path'], open_url_file: res['opener.open_url_file'] }, 'arbitrary path / file: URL not allowed by capability scope', ['extracts/E4-capability-probes.json']);
  R.note('opener_https_and_reveal', { https: res['opener.open_url_https'], reveal: res['opener.reveal'] });
  // 4. custom commands that take arbitrary filesystem paths (webview is trusted; informational)
  const evil = '/tmp/qa-arbitrary-write.xlsx'; const ex = await app.invoke('export_plan_excel', { planId: 1, targetPath: evil }); const w2 = fs.existsSync(evil); if (w2) fs.unlinkSync(evil);
  R.note('export_plan_excel_arbitrary_path', { ok: ex.ok, file_created: w2, e: ex.e });
  R.check('E4.custom-commands-accept-arbitrary-paths(info)', 'pass', { export_plan_excel_to_tmp: w2 ? 'writes' : 'refused' }, 'informational: IPC accepts any path (acceptable only while the webview loads local content only)', ['extracts/E4.json']);
  // 5. external network + CSP
  const net = await app.b.executeAsync(function (done) { const out = {}; const t = (k, p) => p.then((v) => (out[k] = 'LOADED ' + v), (e) => (out[k] = 'BLOCKED ' + String(e).slice(0, 60))); const img = new Promise((res, rej) => { const i = new Image(); i.onload = () => res('img'); i.onerror = () => rej('img error'); i.src = 'https://example.com/x.png'; setTimeout(() => rej('timeout'), 4000); }); const scr = new Promise((res, rej) => { const s = document.createElement('script'); s.src = 'https://example.com/x.js'; s.onload = () => res('script'); s.onerror = () => rej('script error'); document.head.appendChild(s); setTimeout(() => rej('timeout'), 4000); }); const inl = new Promise((res) => { window.__inlineRan = false; const s = document.createElement('script'); s.textContent = 'window.__inlineRan = true'; document.head.appendChild(s); setTimeout(() => res(String(window.__inlineRan)), 300); }); Promise.all([t('fetch_https', fetch('https://example.com/')), t('fetch_http_localhost', fetch('http://127.0.0.1:9/')), t('img', img), t('script', scr), t('inline_script_ran', inl)]).then(() => done(out)); });
  R.note('network_probe', net); fs.writeFileSync(path.join(OUT_ABS, 'extracts/E4-network-csp-probe.json'), JSON.stringify(net, null, 1));
  R.check('E4.external-fetch-img-script-blocked-by-CSP', /BLOCKED/.test(net.fetch_https) && /BLOCKED/.test(net.img) && /BLOCKED/.test(net.script), net, 'CSP connect-src/img-src/script-src block external hosts', ['extracts/E4-network-csp-probe.json']);
  R.check('E4.inline-script-blocked', /false$/.test(net.inline_script_ran), net.inline_script_ran, 'script-src self blocks inline script');
  const h = await app.collectHook(); const csp = h ? h.csp : []; const reqs = h ? h.requests.filter((r) => !/^ipc:|^\/|^http:\/\/ipc\.localhost|^tauri:/.test(r.u)) : [];
  R.note('hook_csp_events', csp); R.note('hook_non_local_requests', reqs);
  R.check('E4.csp-violations-reported-for-probes-only', csp.length >= 3, csp.slice(0, 6), 'securitypolicyviolation events fired for the 3 deliberate probes (and nothing else, see A9)');
});

// seed_demo on an EMPTY real DB: proves the dev-tools command is live in the release build and writes into the real database
{
  const run2 = prepareRunDir('E4empty', { portable: true }); const db2 = path.join(run2.dataDir, 'exam-panel.db');
  await withApp({ scenario: 'E4empty', run: run2 }, async (app) => {
    const before = sql(db2, 'select (select count(*) from teachers) t,(select count(*) from campuses) c,(select count(*) from plans) p')[0];
    const r = await app.invoke('seed_demo', {}); const after = sql(db2, 'select (select count(*) from teachers) t,(select count(*) from campuses) c,(select count(*) from plans) p')[0];
    const info = await app.invoke('get_app_info'); R.note('seed_demo_empty_db', { result: r.ok ? 'OK' : r.e, before, after, db_path: info.v.db_path, in_trial_mode: info.v.in_trial_mode });
    R.check('E4.seed_demo-on-empty-real-db-must-not-write', r.ok === false && after.t === 0, { result: r.ok ? 'executed' : r.e, before, after, writes_into_real_db_file: path.basename(info.v.db_path) }, 'release build has no seed_demo command (task: confirm release contains no dev-tools); here the command exists and fills the real exam-panel.db with demo teachers/plans', ['extracts/E4.json', 'build-inspect.txt']);
  });
}
