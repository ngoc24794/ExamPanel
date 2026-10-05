import { prepareRunDir, withApp, OUT_ABS } from '../lib/harness.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C5f', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const out = {};
await withApp({ scenario: 'C5f', run }, async (app) => {
  const A = (await app.invoke('get_plan', { id: 2 })).v.assignments; const tmap = (await app.invoke('list_teachers', {})).v; const nm = (id) => tmap.find((t) => t.id === id)?.full_name;
  const panel = A.filter((a) => a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2); out.panel_in_array_order = panel.map((a) => ({ role: a.role, position: a.position, teacher: nm(a.teacher_id) }));
  const base = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: A })).v.score_report.total;
  const full = async (seat, tid) => (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: A.map((a) => (a === seat ? { ...a, teacher_id: tid } : a)) })).v.score_report.total;
  const setters = A.filter((a) => a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2 && a.role === 'setter');
  out.rows = [];
  for (const pos of [0, 1]) {
    const cands = (await app.invoke('evaluate_candidates', { schoolYearId: 1, assignments: A, slot: { exam_id: 1, grade_id: 1, subject_id: 2, role: 'setter', position: pos } })).v;
    for (const c of cands.filter((c) => c.hard_violations.length === 0)) {
      const row = { slot_position: pos, candidate: nm(c.teacher_id), incremental_delta: +c.delta_score.toFixed(2) };
      for (const s of setters) row[`full_delta_replacing_${nm(s.teacher_id)}(pos ${s.position})`] = +((await full(s, c.teacher_id)) - base).toFixed(2);
      out.rows.push(row);
    }
  }
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5f-incremental-seat-mapping.json'), JSON.stringify(out, null, 1)); console.log(JSON.stringify(out, null, 1));
