import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C7probe', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
await withApp({ scenario: 'C7probe', run }, async (app) => {
  await app.nav('/statistics'); await sleep(1200);
  const cb = await app.b.$('[role="combobox"]'); await cb.click(); await sleep(400);
  const opts = await app.b.$$('[role="option"]'); const names = []; for (const o of opts) names.push(await o.getText()); console.log('options', names);
  await (await app.b.$('//*[@role="option"][contains(.,"Nhập từ bảng của tổ")]')).click(); await sleep(2500);
  await app.shot('C7probe-stats'); const t = await app.b.execute(() => document.querySelector('[data-testid="statistics-page"]').innerText); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C7probe-text.txt'), t); console.log(t.slice(0, 3500));
  console.log(await app.b.execute(() => ({ tables: document.querySelectorAll('table').length, svgs: document.querySelectorAll('svg').length, canvases: document.querySelectorAll('canvas').length, tids: [...document.querySelectorAll('[data-testid]')].map((e) => e.getAttribute('data-testid')).slice(0, 30) })));
});
