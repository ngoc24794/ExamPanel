import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const out = {};
for (const mode of ['baseline', 'unavail-only', 'qui-only']) {
  const run = prepareRunDir('C5j', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
  await withApp({ scenario: 'C5j', run }, async (app) => {
    if (mode === 'unavail-only') { await app.nav('/unavailability'); await sleep(900); await clickTid(app.b, 'unavail-cell-1-2'); await sleep(900); }
    if (mode === 'qui-only') { await app.nav('/teachers'); await sleep(900); for (const r of await app.b.$$('tr')) { if ((await r.getText()).includes('Cô Quí')) { for (const b of await r.$$('button')) { const lab = (await b.getAttribute('aria-label')) || ''; if (lab.includes('Chỉnh sửa')) { await app.b.execute((el) => el.click(), b); break; } } break; } } await sleep(700); const adv = await app.b.$('//*[@role="dialog"]//*[contains(text(),"advancedSettings")]'); await adv.click(); await sleep(300); await (await tid(app.b, 'teacher-max-tasks-override-input')).clearValue(); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(1200); }
    const a = (await app.invoke('get_plan', { id: 1 })).v.assignments; const ev = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: a })).v;
    out[mode] = ev.hard_violations.map((h) => `${h.rule}:${h.code} t=${h.teacher} ${JSON.stringify(h.panel)} ${JSON.stringify(h.params)}`);
    const seats1 = a.filter((x) => x.teacher_id === 1).map((x) => `${x.exam_id}.${x.grade_id}.${x.subject_id}.${x.role}`); out[mode + '_teacher1_seats'] = seats1;
  });
}
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5j-hard-violation-attribution.json'), JSON.stringify(out, null, 1)); console.log(JSON.stringify(out, null, 1));
