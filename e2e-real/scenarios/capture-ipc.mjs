// Records REAL Rust IPC results (golden Q DB + org info + one final plan) so the real UI code can be replayed inside Chromium for print checks.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import fs from 'node:fs'; import path from 'node:path';
const run = prepareRunDir('capture', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const out = {};
await withApp({ scenario: 'capture', run }, async (app) => {
  const inv = async (cmd, args = {}) => { const r = await app.invoke(cmd, args); if (!r.ok) throw new Error(cmd + ' ' + JSON.stringify(r.e)); return r.v; };
  for (const [k, v] of Object.entries({ school_name: 'TRƯỜNG THPT THỬ NGHIỆM', department_name: 'TỔ VẬT LÍ - CÔNG NGHỆ', place_name: 'Đà Nẵng', signer_title: 'TỔ TRƯỞNG CHUYÊN MÔN', signer_name: 'Nguyễn Thị Thử' })) await inv('set_setting', { key: k, value: v });
  await inv('mark_final', { id: 1 });
  const sy = await inv('list_school_years'); const syId = sy.find((y) => y.is_current).id;
  out.list_school_years = sy; out.get_settings = await inv('get_settings'); out.list_campuses = await inv('list_campuses'); out.list_grades = await inv('list_grades');
  out.list_exams = await inv('list_exams', { schoolYearId: syId }); out.list_subjects = await inv('list_subjects', { schoolYearId: syId }); out.teachers_with_grades = await inv('teachers_with_grades', { schoolYearId: syId }); out.list_teachers = await inv('list_teachers', {});
  out.get_plan = {}; for (const id of [1, 2]) out.get_plan[id] = await inv('get_plan', { id });
  out.list_plans = await inv('list_plans', { schoolYearId: syId }); out.get_app_info = await inv('get_app_info');
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/replay-ipc.json'), JSON.stringify(out)); console.log(Object.keys(out), 'bytes', JSON.stringify(out).length);
