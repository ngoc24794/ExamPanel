import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5');
const run = prepareRunDir('C5d', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db'); const out = { slots: [] };
await withApp({ scenario: 'C5d', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'create-edit-copy-button'); await sleep(3000);
  if (await (await tid(app.b, 'create-edit-copy-button')).isExisting()) { await clickTid(app.b, 'history-plans-button'); await sleep(700); await (await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"(Chỉnh sửa)")]')).click(); await sleep(800); await app.b.keys('Escape'); await sleep(900); }
  const pid = sql(db, "select id from plans where source='duplicate'")[0].id;
  const A = (await app.invoke('get_plan', { id: pid })).v.assignments; const tmap = (await app.invoke('list_teachers', {})).v; const idByName = Object.fromEntries(tmap.map((t) => [t.full_name, t.id]));
  const base = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: A })).v.score_report.total; out.base = base;
  const names = ['Cô Hiền', 'Cô Lài', 'Thầy Phúc', 'Thầy Lộc', 'Cô Thư', 'Cô Na', 'Cô Bình', 'Cô Quí', 'Cô Tú', 'Cô Như', 'Cô Lan', 'Thầy Nghĩa'];
  for (const [cellId, slotKey] of [['q-grid-cell-1-1-VL-setter-0', [1, 1, 2, 'setter', 0]], ['q-grid-cell-3-2-VL-reviewer-0', [3, 2, 2, 'reviewer', 0]], ['q-grid-cell-2-3-CN-reviewer-0', [2, 3, 3, 'reviewer', 0]]]) {
    await (await tid(app.b, cellId)).click(); await sleep(1500);
    const dtext = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || ''); const lines = dtext.split('\n').map((l) => l.trim()).filter(Boolean);
    const rows = [];
    for (let i = 0; i < lines.length; i++) if (names.includes(lines[i])) { const nxt = lines.slice(i + 2, i + 6); if (/^\+|^-/.test(nxt[0] || '')) { const tot = parseFloat((nxt[1] || '').replace('Tổng: ', '')); const seat = A.find((a) => a.exam_id === slotKey[0] && a.grade_id === slotKey[1] && a.subject_id === slotKey[2] && a.role === slotKey[3] && a.position === slotKey[4]); const mod = A.map((a) => (a === seat ? { ...a, teacher_id: idByName[lines[i]] } : a)); const ev = await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: mod }); rows.push({ teacher: lines[i], ui_delta: nxt[0], ui_total: tot, engine_total: +ev.v.score_report.total.toFixed(2), engine_delta: +(ev.v.score_report.total - base).toFixed(2), match: Math.abs(tot - ev.v.score_report.total) < 0.06 }); } }
    out.slots.push({ cell: cellId, rows }); await app.b.keys('Escape'); await sleep(600);
  }
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5d-candidate-delta-vs-engine.json'), JSON.stringify(out, null, 1));
const all = out.slots.flatMap((s) => s.rows); const bad = all.filter((r) => !r.match);
R.check('C5.candidate-delta-equals-engine-delta', bad.length === 0, { compared: all.length, mismatching: bad.length, sample_bad: bad.slice(0, 4) }, 'candidate "Tổng" == evaluate_assignments(total) after the replacement', ['extracts/C5d-candidate-delta-vs-engine.json']);
console.log(JSON.stringify({ compared: all.length, bad: bad.length, sample: all.slice(0, 5) }, null, 1));
