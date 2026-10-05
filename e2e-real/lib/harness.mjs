// Real-app harness: one fresh tauri-driver + app per scenario, isolated HOME/XDG, unprivileged user.
import { spawn, execFileSync, execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import { remote } from 'webdriverio';

export const REPO = path.resolve(new URL('../..', import.meta.url).pathname);
export const EXE_SRC = process.env.EXAMPANEL_EXE || path.join(REPO, 'target/release/exam-panel-app');
export const TAURI_DRIVER = path.join(REPO, '.tools/cargo-tools/bin/tauri-driver');
export const RUN_USER = process.env.QA_USER || 'qauser';
export const DISPLAY = ':99';
export const OUT = fs.readFileSync(path.join(REPO, '.tools/D'), 'utf8').trim();
export const OUT_ABS = path.join(REPO, OUT);

export const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function freePort() {
  return new Promise((res) => {
    const s = net.createServer().listen(0, '127.0.0.1', () => { const p = s.address().port; s.close(() => res(p)); });
  });
}
function waitPort(port, ms = 20000) {
  const t0 = Date.now();
  return new Promise((res, rej) => {
    const tryOnce = () => {
      const c = net.connect(port, '127.0.0.1');
      c.on('connect', () => { c.destroy(); res(); });
      c.on('error', () => { c.destroy(); Date.now() - t0 > ms ? rej(new Error('port timeout ' + port)) : setTimeout(tryOnce, 150); });
    };
    tryOnce();
  });
}

/** Prepare a scenario dir: copy exe (+ marker), chown to the unprivileged user. */
export function prepareRunDir(scenario, { portable = true, seedDb = null, keepIfExists = false } = {}) {
  let n = 1;
  while (fs.existsSync(path.join(REPO, '.tools/qa-run', `${scenario}-${n}`))) n++;
  const dir = path.join(REPO, '.tools/qa-run', `${scenario}-${n}`);
  fs.mkdirSync(path.join(dir, 'home/.local/share'), { recursive: true });
  fs.mkdirSync(path.join(dir, 'home/.config'), { recursive: true });
  fs.mkdirSync(path.join(dir, 'home/.cache'), { recursive: true });
  fs.mkdirSync(path.join(dir, 'app'), { recursive: true });
  fs.copyFileSync(EXE_SRC, path.join(dir, 'app/ExamPanel'));
  fs.chmodSync(path.join(dir, 'app/ExamPanel'), 0o755);
  if (portable) fs.writeFileSync(path.join(dir, 'app/ExamPanel.portable'), '');
  if (seedDb) { fs.mkdirSync(path.join(dir, 'app/data'), { recursive: true }); fs.copyFileSync(seedDb, path.join(dir, 'app/data/exam-panel.db')); }
  fs.mkdirSync(path.join(dir, 'out'), { recursive: true });
  execFileSync('chown', ['-R', `${RUN_USER}:${RUN_USER}`, dir]);
  return { dir, n, exe: path.join(dir, 'app/ExamPanel'), dataDir: path.join(dir, 'app/data'), xdgData: path.join(dir, 'home/.local/share') };
}

export function isoEnv(dir, extra = {}) {
  return {
    HOME: path.join(dir, 'home'),
    XDG_DATA_HOME: path.join(dir, 'home/.local/share'),
    XDG_CONFIG_HOME: path.join(dir, 'home/.config'),
    XDG_CACHE_HOME: path.join(dir, 'home/.cache'),
    DISPLAY,
    WEBKIT_DISABLE_COMPOSITING_MODE: '1',
    LIBGL_ALWAYS_SOFTWARE: '1',
    NO_AT_BRIDGE: '1',
    ...extra,
  };
}

/** Start a private dbus session bus as the run user; returns {address, pid, stop()} */
export function startDbus(dir) {
  const out = execFileSync('runuser', ['-u', RUN_USER, '--', 'dbus-daemon', '--session', '--fork', '--print-address=1', '--print-pid=1'], { env: { ...process.env, HOME: path.join(dir, 'home') } }).toString().trim().split('\n');
  const address = out[0]; const pid = Number(out[1]);
  return { address, pid, stop() { try { process.kill(pid, 'SIGTERM'); } catch {} } };
}

const HOOK = `
(function(){
  if (window.__qaHook) return; window.__qaHook = { console: [], errors: [], rejections: [], requests: [], csp: [] };
  var H = window.__qaHook, t0 = Date.now();
  ['error','warn'].forEach(function(l){ var o = console[l]; console[l] = function(){ try { H.console.push({l:l, t:Date.now()-t0, m:Array.prototype.map.call(arguments,function(a){try{return typeof a==='string'?a:(a&&a.stack)||JSON.stringify(a)}catch(e){return String(a)}}).join(' ').slice(0,1500)}); } catch(e){} return o.apply(console, arguments); }; });
  window.addEventListener('error', function(e){ H.errors.push({m:String(e.message), src:e.filename, line:e.lineno}); });
  window.addEventListener('unhandledrejection', function(e){ H.rejections.push(String(e.reason && (e.reason.stack||e.reason.message||JSON.stringify(e.reason)) || e.reason).slice(0,1500)); });
  document.addEventListener('securitypolicyviolation', function(e){ H.csp.push({d:e.violatedDirective, b:e.blockedURI}); });
  var of = window.fetch; if (of) window.fetch = function(u){ try { H.requests.push({k:'fetch', u:String(u && u.url || u)}); } catch(e){} return of.apply(this, arguments); };
  var ox = XMLHttpRequest.prototype.open; XMLHttpRequest.prototype.open = function(m,u){ try { H.requests.push({k:'xhr', u:String(u)}); } catch(e){} return ox.apply(this, arguments); };
  var OW = window.WebSocket; if (OW) window.WebSocket = function(u,p){ try { H.requests.push({k:'ws', u:String(u)}); } catch(e){} return new OW(u,p); };
})();`;

export class App {
  constructor(o) { Object.assign(this, o); this.hookLog = []; this.shotN = 0; }

  async ensureHook() { try { await this.b.execute(new Function(HOOK)); } catch (e) { this.hookLog.push('hook-inject-fail ' + e.message); } }
  async collectHook() {
    try {
      const h = await this.b.execute(() => window.__qaHook ? JSON.parse(JSON.stringify(window.__qaHook)) : null);
      if (h) {
        this.hookLog.push({ at: new Date().toISOString(), ...h });
        // reset buffers so a later collect only returns new events
        await this.b.execute(() => { const H = window.__qaHook; if (H) { H.console = []; H.errors = []; H.rejections = []; H.requests = []; H.csp = []; } });
      }
      return h;
    } catch (e) { this.hookLog.push('collect-fail ' + e.message); return null; }
  }
  async shot(name, { screen = true } = {}) {
    const base = path.join(OUT_ABS, 'screenshots', `${this.scenario}-${name}`);
    fs.mkdirSync(path.dirname(base), { recursive: true });
    try { await this.b.saveScreenshot(base + '-page.png'); } catch (e) { this.hookLog.push('page-shot-fail ' + e.message); }
    if (screen) { try { execFileSync('import', ['-window', 'root', '-resize', '1920x1080>', base + '-screen.png'], { env: { ...process.env, DISPLAY } }); } catch {} }
    return base;
  }
  async nav(hashPath) { // SPA route change without reload
    await this.b.execute((p) => { window.history.pushState({}, '', p); window.dispatchEvent(new PopStateEvent('popstate')); }, hashPath);
    await sleep(400);
  }
  async invoke(cmd, args = {}) {
    const r = await this.b.executeAsync(function (cmd, args, done) {
      try { window.__TAURI_INTERNALS__.invoke(cmd, args).then(function (v) { done({ ok: true, v: v }); }, function (e) { done({ ok: false, e: e }); }); } catch (e) { done({ ok: false, e: String(e) }); }
    }, cmd, args);
    return r;
  }
  async stop({ keepDriver = false } = {}) {
    await this.collectHook();
    try { await this.b.deleteSession(); } catch {}
    await sleep(500);
    try { this.driver.kill('SIGTERM'); } catch {}
    await sleep(500);
    try { execSync(`pkill -u ${RUN_USER} -f ${JSON.stringify(this.exe)}`); } catch {}
    if (this.dbus && !keepDriver) this.dbus.stop();
    fs.writeFileSync(path.join(this.runDir, 'out/hook.json'), JSON.stringify(this.hookLog, null, 1));
  }
  appPids() { try { return execSync(`pgrep -u ${RUN_USER} -f ${JSON.stringify(this.exe)}`).toString().trim().split('\n').map(Number); } catch { return []; } }
}

/** Launch a fresh tauri-driver + app. opts: {scenario, run (prepareRunDir result), env, args, dbus, width, height} */
export async function launchApp(opts) {
  const { scenario, run, env = {}, args = [] } = opts;
  const dbus = opts.dbus || startDbus(run.dir);
  const pPort = await freePort(); const nPort = await freePort();
  const fullEnv = { ...process.env, ...isoEnv(run.dir, { DBUS_SESSION_BUS_ADDRESS: dbus.address, ...env }) };
  const logOut = fs.openSync(path.join(run.dir, 'out/tauri-driver.log'), 'a');
  const t0 = Date.now();
  const driver = spawn('runuser', ['-u', RUN_USER, '--', 'env', ...Object.entries(fullEnv).filter(([k]) => /^(HOME|XDG_|DISPLAY|DBUS_|WEBKIT_|LIBGL|NO_AT|PATH|LANG|LC_)/.test(k)).map(([k, v]) => `${k}=${v}`), TAURI_DRIVER, '--port', String(pPort), '--native-port', String(nPort), '--native-driver', '/usr/bin/WebKitWebDriver'], { stdio: ['ignore', logOut, logOut], detached: false });
  await waitPort(pPort);
  const b = await remote({
    hostname: '127.0.0.1', port: pPort, logLevel: 'error', connectionRetryCount: 1, connectionRetryTimeout: 60000,
    capabilities: { 'tauri:options': { application: run.exe, args } },
  });
  const app = new App({ b, driver, dbus, scenario, runDir: run.dir, exe: run.exe, startedAt: t0, run });
  await sleep(300);
  await app.ensureHook();
  return app;
}

/** Poll until app UI is rendered (no ui-ready marker exists in the product; see findings). Returns ms since launch. */
export async function waitUiReady(app, timeout = 30000) {
  const t0 = Date.now();
  while (Date.now() - t0 < timeout) {
    try {
      const ok = await app.b.execute(() => { const r = document.getElementById('root'); return !!(r && r.children.length > 0 && r.innerText.trim().length > 20); });
      if (ok) return Date.now() - app.startedAt;
    } catch {}
    await sleep(100);
  }
  return null;
}

/** Sample RSS (KB) of all processes of the run user whose cmd contains the exe path or WebKit */
export function sampleRss() {
  const out = execSync(`ps -u ${RUN_USER} -o pid=,rss=,comm=,args= --no-headers`).toString().trim().split('\n').filter(Boolean);
  const rows = out.map((l) => { const m = l.trim().match(/^(\d+)\s+(\d+)\s+(\S+)\s+(.*)$/); return m ? { pid: +m[1], rss: +m[2], comm: m[3], args: m[4] } : null; }).filter(Boolean);
  const rel = rows.filter((r) => /ExamPanel|WebKit/i.test(r.comm + r.args));
  return { total_kb: rel.reduce((s, r) => s + r.rss, 0), procs: rel.map((r) => ({ pid: r.pid, comm: r.comm, rss_kb: r.rss })) };
}
