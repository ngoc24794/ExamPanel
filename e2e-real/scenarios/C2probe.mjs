import { prepareRunDir, withApp, sleep, OUT_ABS, sampleRss } from '../lib/harness.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C2probe', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'C2probe', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1200);
  await clickTid(app.b, 'run-optimizer-button'); await sleep(700);
  await app.shot('C2probe-dialog');
  console.log((await app.b.execute(() => document.querySelector('[role="dialog"]').innerText)).slice(0, 700));
  console.log(await app.b.execute(() => [...document.querySelectorAll('[role="dialog"] button')].map((b) => b.innerText.trim() + '|' + (b.getAttribute('data-testid') || ''))));
  // choose fast
  const fastBtn = await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Nhanh"]'); await fastBtn.click();
  const t0 = Date.now(); await clickTid(app.b, 'start-optimize-button');
  const samples = [];
  for (let i = 0; i < 400; i++) {
    const s = await app.b.execute(() => { const d = document.querySelector('[role="dialog"]'); if (!d) return null; const pb = d.querySelector('[role="progressbar"]'); return { t: performance.now(), txt: d.innerText.replace(/\n+/g, ' | ').slice(0, 300), pb: pb && (pb.getAttribute('aria-valuenow') || pb.getAttribute('data-value') || (pb.firstElementChild && pb.firstElementChild.style.cssText)) }; });
    if (!s) { samples.push({ gone: Date.now() - t0 }); break; }
    samples.push({ at: Date.now() - t0, ...s }); await sleep(250);
  }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C2probe-fast-samples.json'), JSON.stringify(samples, null, 1));
  console.log('samples', samples.length, JSON.stringify(samples.slice(0, 3)), JSON.stringify(samples.slice(-2)));
  console.log('plans', sql(db, 'select id,name,source,score,rank from plans'));
  await sleep(1500); await app.shot('C2probe-after-fast');
});
