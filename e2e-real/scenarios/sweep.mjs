// Visit every main route of the golden DB; dump text head + testids + screenshot. usage: node sweep.mjs <tag> [WxH]
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import fs from 'node:fs'; import path from 'node:path';
const tag = process.argv[2] || 'sweep'; const size = process.argv[3];
const run = prepareRunDir('sweep', { portable: true, seedDb: path.resolve('../.tools/golden/q-after-import.db') });
const routes = ['/', '/teachers', '/campuses', '/subjects', '/competencies', '/exams', '/unavailability', '/rules', '/assignments', '/statistics', '/settings'];
const out = {};
await withApp({ scenario: 'sweep', run }, async (app) => {
  if (size) { const [w, h] = size.split('x').map(Number); await app.b.setWindowSize(w, h); await sleep(800); }
  for (const r of routes) {
    await app.nav(r); await sleep(1200);
    out[r] = await app.b.execute(() => ({ text: document.body.innerText.slice(0, 1800), tids: [...document.querySelectorAll('[data-testid]')].map((e) => e.getAttribute('data-testid')), buttons: [...document.querySelectorAll('button')].map((b) => (b.innerText || b.getAttribute('aria-label') || '').trim()).filter(Boolean).slice(0, 60), sw: document.documentElement.scrollWidth, cw: document.documentElement.clientWidth }));
    await app.shot(`${tag}-${r.replace(/\W/g, '') || 'overview'}`, { screen: false });
  }
});
fs.writeFileSync(path.join(OUT_ABS, `extracts/${tag}.json`), JSON.stringify(out, null, 1));
