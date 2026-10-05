import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('B3');
const run = prepareRunDir('B3c', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
await withApp({ scenario: 'B3c', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'view-toggle-detail'); await sleep(900);
  const txt = () => app.b.execute(() => document.querySelector('[data-testid="plan-matrix-view"]').innerText);
  const t0 = await txt(); const cnt = (t) => ({ setter_lines: (t.match(/\nĐề\n/g) || []).length, reviewer_lines: (t.match(/\nPB\n/g) || []).length, len: t.length });
  const res = { all: cnt(t0) };
  for (const [lab, key] of [['VL - ', 'VL'], ['CN - ', 'CN']]) { await (await app.b.$(`//button[contains(.,"${lab}")]`)).click(); await sleep(800); res[key] = cnt(await txt()); if (key === 'VL') fs.writeFileSync(path.join(OUT_ABS, 'extracts/B3c-matrix-text-VL-filter.txt'), await txt()); }
  const rawLabel = await app.b.execute(() => [...document.querySelectorAll('button')].filter((b) => /common\./.test(b.innerText)).map((b) => b.innerText));
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/B3c-subject-filter.json'), JSON.stringify({ res, rawLabel }, null, 1)); console.log(JSON.stringify({ res, rawLabel }));
  R.check('B3.assignments-subject-filter-narrows-matrix', res.all.setter_lines === 36 && res.all.reviewer_lines === 24 && res.VL.setter_lines === 24 && res.VL.reviewer_lines === 12 && res.CN.setter_lines === 12 && res.CN.reviewer_lines === 12, { res, raw_label: rawLabel }, 'seat lines: all 36 setters + 24 reviewers; VL filter 24 + 12; CN filter 12 + 12 (my first regex counted block titles that the filtered view omits)', ['extracts/B3c-subject-filter.json']);
});
