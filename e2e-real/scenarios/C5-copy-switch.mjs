// Repeats "Tạo bản chỉnh sửa" in fresh app runs and records whether the view switches to the new editable copy.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5'); const res = [];
for (let i = 0; i < 10; i++) {
  const run = prepareRunDir('C5copy', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
  await withApp({ scenario: 'C5copy', run }, async (app) => {
    await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'create-edit-copy-button');
    const seen = []; for (const ms of [300, 700, 1500, 3000]) { await sleep(ms === 300 ? 300 : 400 + (ms - 700 > 0 ? ms - 700 : 0)); seen.push(await app.b.execute(() => (document.querySelector('[data-testid="create-edit-copy-button"]') ? 'optimizer-view' : 'edit-view'))); }
    res.push({ attempt: i + 1, views_over_time: seen });
  });
}
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5-copy-switch-attempts.json'), JSON.stringify(res, null, 1));
const ok = res.filter((r) => r.views_over_time[r.views_over_time.length - 1] === 'edit-view').length;
R.check('C5.create-edit-copy-switches-to-copy', ok === res.length ? 'pass' : 'fail', { switched: ok, of: res.length }, 'after "Tạo bản chỉnh sửa" the new editable copy is shown (every time)', ['extracts/C5-copy-switch-attempts.json']);
