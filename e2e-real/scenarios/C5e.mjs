import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('C5e', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db'); const out = {};
await withApp({ scenario: 'C5e', run }, async (app) => {
  const pid = 2; // first optimizer plan; evaluate_candidates works on any assignment list
  const A = (await app.invoke('get_plan', { id: pid })).v.assignments; const tmap = (await app.invoke('list_teachers', {})).v;
  const slot = { exam_id: 1, grade_id: 1, subject_id: 2, role: 'setter', position: 0 };
  const base = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: A })).v.score_report;
  const cands = await app.invoke('evaluate_candidates', { schoolYearId: 1, assignments: A, slot });
  out.base_total = base.total; out.base_by_rule = Object.fromEntries(base.by_rule.map((r) => [r.rule, r.penalty]));
  out.cands = [];
  for (const c of cands.v) { const seat = A.find((a) => a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2 && a.role === 'setter' && a.position === 0); const mod = A.map((a) => (a === seat ? { ...a, teacher_id: c.teacher_id } : a)); const ev = (await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: mod })).v; out.cands.push({ teacher: tmap.find((t) => t.id === c.teacher_id)?.full_name, cand: c, full_total: ev.score_report.total, full_hard: ev.hard_violations.map((h) => h.code || h), by_rule: Object.fromEntries(ev.score_report.by_rule.map((r) => [r.rule, r.penalty])) }); }
  out.seat = A.find((a) => a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2 && a.role === 'setter' && a.position === 0);
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5e-evaluate-candidates-vs-full.json'), JSON.stringify(out, null, 1));
for (const c of out.cands) console.log(c.teacher, JSON.stringify(c.cand).slice(0, 220), '| full_total', c.full_total.toFixed(2), 'hard', JSON.stringify(c.full_hard).slice(0, 80));
console.log('base', out.base_total);
