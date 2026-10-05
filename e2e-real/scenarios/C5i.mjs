import { prepareRunDir, withApp, sleep } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import path from 'node:path';
const run = prepareRunDir('C5i', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const snap = (tag) => console.log(tag, JSON.stringify({ tg1: sql(db, 'select grade_id from teacher_grades where teacher_id=1 and school_year_id=1 order by grade_id').map((x) => x.grade_id), comp1: sql(db, 'select subject_id,role,grade_scope from teacher_competencies where teacher_id=1 order by 1,2'), un: sql(db, 'select teacher_id,exam_id from unavailability') }));
await withApp({ scenario: 'C5i', run }, async (app) => {
  snap('start');
  await app.nav('/unavailability'); await sleep(900); await clickTid(app.b, 'unavail-cell-1-2'); await sleep(900); snap('after-unavail');
  await app.nav('/teachers'); await sleep(900);
  const rows = await app.b.$$('tr'); let which = null;
  for (const r of rows) { const tx = await r.getText(); if (tx.includes('Cô Quí')) { which = tx.slice(0, 60); for (const b of await r.$$('button')) { const lab = (await b.getAttribute('aria-label')) || (await b.getAttribute('title')) || ''; if (lab.includes('Chỉnh sửa')) { await app.b.execute((el) => el.click(), b); break; } } break; } }
  console.log('row matched:', JSON.stringify(which)); await sleep(700);
  console.log('dialog title:', await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText.split('\n').slice(0, 4).join(' | ')));
  const adv = await app.b.$('//*[@role="dialog"]//*[contains(text(),"advancedSettings")]'); if (await adv.isExisting()) await adv.click(); await sleep(300);
  await (await tid(app.b, 'teacher-max-tasks-override-input')).clearValue(); await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(1200);
  snap('after-qui-edit');
  const a = (await app.invoke('get_plan', { id: 1 })).v.assignments; const ev = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: a })).v; console.log('hard now:', ev.hard_violations.map((h) => `${h.rule}:${h.code} t=${h.teacher}`));
});
