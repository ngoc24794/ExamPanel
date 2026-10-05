import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5');
const toastText = (app) => app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].map((e) => e.innerText.replace(/\n/g, ' | ')));
async function ensureCopyView(app) {
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'create-edit-copy-button'); await sleep(3000);
  if (await (await tid(app.b, 'create-edit-copy-button')).isExisting()) { await clickTid(app.b, 'history-plans-button'); await sleep(700); await (await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"(Chỉnh sửa)")]')).click(); await sleep(800); await app.b.keys('Escape'); await sleep(900); }
}
const results = {};
for (const mode of ['noop-save', 'replace', 'swap']) {
  const run = prepareRunDir('C5b', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
  const db = path.join(run.dataDir, 'exam-panel.db');
  await withApp({ scenario: 'C5b', run }, async (app) => {
    await ensureCopyView(app);
    const pid = sql(db, "select id from plans where source='duplicate'")[0].id; const seats0 = JSON.stringify(sql(db, 'select * from assignments where plan_id=? order by exam_id,grade_id,subject_id,role,position', [pid]));
    if (mode === 'replace') { await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).click(); await sleep(1500); await app.b.keys('ArrowDown'); await app.b.keys('ArrowDown'); await app.b.keys('Enter'); await sleep(1000); }
    if (mode === 'swap') { await app.b.execute(() => { const c1 = document.querySelector('[data-testid="q-grid-cell-2-1-VL-reviewer-0"]'), c2 = document.querySelector('[data-testid="q-grid-cell-3-2-VL-reviewer-0"]'); const dt = new DataTransfer(); c1.dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: dt })); c2.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: dt })); c2.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt })); }); await sleep(1000); }
    let saveBtn = await tid(app.b, 'save-plan-assignments-button'); const dirty = await saveBtn.isExisting();
    let toasts = [], seats1 = seats0;
    if (dirty) { await saveBtn.click(); await sleep(1500); toasts = await toastText(app); seats1 = JSON.stringify(sql(db, 'select * from assignments where plan_id=? order by exam_id,grade_id,subject_id,role,position', [pid])); }
    results[mode] = { dirty, toasts, db_changed: seats0 !== seats1 };
    await app.shot('C5b-' + mode, { screen: false });
    if (mode === 'noop-save') { const gp = await app.invoke('update_plan_assignments', { id: pid, assignments: (await app.invoke('get_plan', { id: pid })).v.assignments }); results[mode].ipc_noop_update = gp.ok ? 'ok' : gp.e; }
  });
}
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5b-save-modes.json'), JSON.stringify(results, null, 1)); console.log(JSON.stringify(results, null, 1));
