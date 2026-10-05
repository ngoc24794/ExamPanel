import { prepareRunDir, launchApp, waitUiReady, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { windowList, screenShot } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, execFileSync, spawnSync } from 'node:child_process';
const R = recorder('D4');
const run = prepareRunDir('D4', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const head = fs.readFileSync(path.join(OUT_ABS, 'start-commit.txt'), 'utf8').trim(); // binary was built from the starting commit (git HEAD has since moved because of the QA commits)
const dirtyNow = execSync('git status --porcelain --untracked-files=no', { cwd: path.resolve('..') }).toString().trim().length > 0;
let app = await launchApp({ scenario: 'D4', run }); await waitUiReady(app);
const state = (a) => a.b.execute(() => ({ dark: document.documentElement.classList.contains('dark'), lang: document.documentElement.lang, title: document.querySelector('h1,h2')?.innerText, htmlClass: document.documentElement.className, ls: Object.fromEntries(Object.entries(localStorage)) }));
await app.nav('/settings'); await sleep(1000);
await clickTid(app.b, 'theme-dark-btn'); await sleep(400); await clickTid(app.b, 'lang-en-btn'); await sleep(800);
const s1 = await state(app); await app.shot('D4-settings-en-dark', { screen: false });
const info = await app.invoke('get_app_info'); R.note('app_info', info.v);
const t = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/D4-settings-en.txt'), t);
// About
const about = await app.b.execute(() => ({ v: document.querySelector('[data-testid="about-version"]')?.innerText, c: document.querySelector('[data-testid="about-commit"]')?.innerText, d: document.querySelector('[data-testid="about-build-date"]')?.innerText }));
R.note('about_section', about);
R.check('D4.about-version', /0\.1\.0/.test(about.v || ''), about.v, '0.1.0');
R.check('D4.about-commit-equals-build-commit(HEAD at build time)', !!about.c && head.startsWith((about.c.match(/[0-9a-f]{7,40}/) || [''])[0]) && (about.c.match(/[0-9a-f]{7,40}/) || [''])[0].length >= 7, { about_commit: about.c, git_HEAD: head.slice(0, 12), get_app_info_commit: info.v.commit_hash }, 'commit equals HEAD (build from clean tree)', ['extracts/D4.json']);
R.check('D4.about-dirty-false', !/dirty|\*|\+/i.test(about.c || '') && !dirtyNow, { about_commit_text: about.c, git_tracked_changes_now: dirtyNow, note: 'About shows no dirty flag; app_info has no dirty field' }, 'dirty=false visible', ['extracts/D4.json']);
// path + copy
const dp = await app.b.execute(() => document.querySelector('[data-testid="data-path-display"]').textContent.trim()); const badge = await (await tid(app.b, 'storage-mode-badge')).getText();
R.check('D4.storage-mode-and-data-path', /portable|di động/i.test(badge) && dp.startsWith(run.dataDir), { badge, shown_path: dp, expected: run.dataDir }, 'portable badge; path = <exe>/data');
spawnSync('xclip', ['-selection', 'clipboard'], { input: 'CLIPBOARD-SENTINEL', env: { ...process.env, DISPLAY: ':99' }, stdio: ['pipe', 'ignore', 'ignore'], timeout: 3000 });
await clickTid(app.b, 'copy-path-btn'); await sleep(800); let clip = ''; try { clip = execSync('xclip -selection clipboard -o', { env: { ...process.env, DISPLAY: ':99' }, timeout: 3000 }).toString(); } catch (e) { clip = 'ERR ' + e.message; }
const toastCopy = await app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText));
R.check('D4.copy-path-works', clip.trim() === dp.trim() || clip.trim().startsWith(run.dataDir), { clipboard: clip.slice(0, 120), toast: toastCopy }, 'clipboard holds data path', []);
// open folders (xdg-open)
const xo = (() => { try { return execSync('which xdg-open').toString().trim(); } catch { return 'missing'; } })();
for (const [id, name] of [['open-folder-btn', 'open-data-folder'], ['open-logs-btn', 'open-logs-folder']]) {
  const before = windowList().length; await clickTid(app.b, id); await sleep(2000);
  const tt = await app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | '))); const alive = await app.b.execute(() => document.body.innerText.length > 50);
  R.check(`D4.${name}-no-crash`, alive, { xdg_open: xo, toasts: tt, windows_before: before, windows_after: windowList().length, app_alive: alive }, 'environment: xdg-open may have no handler; app must survive and show feedback', []);
  await app.shot('D4-' + name, { screen: false });
}
// licenses
await clickTid(app.b, 'licenses-dialog-trigger'); await sleep(1200);
const lic = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); fs.writeFileSync(path.join(OUT_ABS, 'extracts/D4-licenses-dialog.txt'), lic); await app.shot('D4-licenses', { screen: false });
R.check('D4.third-party-notices-opens', lic.length > 500, { chars: lic.length, head: lic.slice(0, 160).replace(/\n+/g, ' | ') }, 'THIRD_PARTY_NOTICES text shown', ['extracts/D4-licenses-dialog.txt']);
await app.b.keys('Escape'); await sleep(400);
// restart persistence
const before = await state(app); await app.stop();
app = await launchApp({ scenario: 'D4', run }); await waitUiReady(app); await sleep(800);
const after = await state(app); await app.nav('/settings'); await sleep(800); await app.shot('D4-after-restart', { screen: false });
const t2 = await bodyText(app.b);
R.check('D4.theme-persists-across-restart', after.dark === true && before.dark === true, { before: before.dark, after: after.dark }, 'dark remains');
R.check('D4.language-persists-across-restart', /Settings/.test(t2) && !/Cài đặt Hệ thống/.test(t2), { before_lang: before.lang, after_lang: after.lang, has_english: /Settings/.test(t2) }, 'English remains');
R.note('localStorage_before_after', { before: before.ls, after: after.ls });
// header language switcher: does THAT persist?
R.note('header_lang_button_text', await app.b.execute(() => [...document.querySelectorAll('button')].filter((b) => /^(vi|en)$/i.test(b.innerText.trim())).map((b) => b.innerText.trim())));
await app.b.execute(() => [...document.querySelectorAll('button')].find((b) => /^(vi|en)$/i.test(b.innerText.trim())).click()); await sleep(500);
const labels = await app.b.execute(() => [...document.querySelectorAll('button')].filter((b) => /English|Tiếng Việt/.test(b.innerText)).map((b) => b.innerText.trim())); R.note('header_lang_menu', labels);
await app.b.execute(() => [...document.querySelectorAll('button')].find((b) => /English/.test(b.innerText) && b.className.includes('text-left')).click()); await sleep(800);
const viaHeader = await bodyText(app.b); await app.stop();
app = await launchApp({ scenario: 'D4', run }); await waitUiReady(app); await sleep(800); const t3 = await bodyText(app.b);
R.check('D4.language-via-header-switcher-persists-across-restart', /Overview/.test(t3), { english_after_header_switch_and_restart: /Overview/.test(t3), before_restart_english: /Overview/.test(viaHeader) }, 'English remains after restart when set with the header switcher');
await app.nav('/settings'); await clickTid(app.b, 'lang-vi-btn'); await clickTid(app.b, 'theme-light-btn');
await app.stop();
