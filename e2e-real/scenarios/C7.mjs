import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C7');
const run = prepareRunDir('C7', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const teachers = sql(db, 'select id,full_name,display_name from teachers order by id'); const nameOf = Object.fromEntries(teachers.map((t) => [t.id, t.full_name]));
const A = sql(db, 'select exam_id,grade_id,subject_id,role,teacher_id from assignments where plan_id=1');
// independent expectations
const tot = {}, setter = {}, rev = {}, exams = {};
for (const a of A) { tot[a.teacher_id] = (tot[a.teacher_id] || 0) + 1; (a.role === 'setter' ? setter : rev)[a.teacher_id] = ((a.role === 'setter' ? setter : rev)[a.teacher_id] || 0) + 1; (exams[a.teacher_id] ||= new Set()).add(a.exam_id); }
const panels = {}; for (const a of A) (panels[`${a.exam_id}.${a.grade_id}.${a.subject_id}`] ||= []).push(a);
const cowork = {}, review = {}; for (const p of Object.values(panels)) { for (const x of p) for (const y of p) if (x.teacher_id !== y.teacher_id) { const k = `${x.teacher_id}>${y.teacher_id}`; cowork[k] = (cowork[k] || 0) + 1; } for (const r of p.filter((x) => x.role === 'reviewer')) for (const s of p.filter((x) => x.role === 'setter')) { const k = `${r.teacher_id}>${s.teacher_id}`; review[k] = (review[k] || 0) + 1; } }
let ui;
await withApp({ scenario: 'C7', run }, async (app) => {
  await app.nav('/statistics'); await sleep(1200); await (await app.b.$('[role="combobox"]')).click(); await sleep(400); await (await app.b.$('//*[@role="option"][contains(.,"Nhập từ bảng của tổ")]')).click(); await sleep(2500);
  ui = await app.b.execute(() => { const tb = [...document.querySelectorAll('table')].map((t) => [...t.querySelectorAll('tr')].map((r) => [...r.children].map((c) => c.innerText.replace(/\s+/g, ' ').trim()))); return { tables: tb, text: document.querySelector('[data-testid="statistics-page"]').innerText }; });
  await app.shot('C7-stats-top'); await app.b.execute(() => document.querySelector('main').scrollTo(0, 99999)); await sleep(500); await app.shot('C7-stats-bottom', { screen: false });
  const dark = await app.b.$('[data-testid="theme-dark-btn"]');
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/C7-ui-tables.json'), JSON.stringify(ui.tables, null, 1));
const tTotals = ui.tables.find((t) => t[0] && t[0][0] === 'Giáo viên') || ui.tables[0]; const bad = [];
for (const row of tTotals.slice(1)) { const nm = row[0].split(' Chưa')[0].trim(); const tcell = row.slice(1); const t = teachers.find((x) => x.full_name === nm); if (!t) { bad.push({ nm, why: 'unknown' }); continue; } const [camp, total, st, rv] = tcell; if (+total !== tot[t.id] || +st !== (setter[t.id] || 0) || +rv !== (rev[t.id] || 0)) bad.push({ nm, ui: { total, st, rv }, expected: { total: tot[t.id], st: setter[t.id], rv: rev[t.id] } }); const exp = [...exams[t.id]].sort().map((e) => ['GK1', 'CK1', 'GK2', 'CK2'][e - 1]); const uiEx = (row[row.length - 1] || '').split(',').map((s) => s.trim()).sort(); if (JSON.stringify(exp.sort()) !== JSON.stringify(uiEx)) bad.push({ nm, exams_ui: uiEx, exams_expected: exp }); }
R.check('C7.per-teacher-table-equals-DB', bad.length === 0 && tTotals.length - 1 === 12, { rows: tTotals.length - 1, mismatches: bad }, '12 rows; total/setter/reviewer/exams equal independent counts from assignments', ['extracts/C7-ui-tables.json']);
const quotaRows = tTotals.slice(1).map((r) => ({ nm: r[0].split(' Chưa')[0], q: r[r.length - 4], d: r[r.length - 3] })); const qbad = quotaRows.filter((r) => !(Math.abs(parseFloat(r.q) - (r.nm === 'Thầy Nghĩa' ? 12 : 48 / 11)) < 0.006));
R.check('C7.quota-q-values', qbad.length === 0, { bad: qbad, expected: 'q=4.36 (=48/11) for 11 teachers, 12.00 for T Nghĩa (override)' }, 'q matches 48/11 and 12', ['extracts/C7-ui-tables.json']);
// matrices
const matrices = ui.tables.filter((t) => t[0] && /^\d+$/.test(t[0][0] || '1') && t.length > 5); R.note('matrix_count', matrices.length);
const cm = ui.tables[1], rm = ui.tables[2];
function parseM(m) { const o = {}; const hdr = m[0]; for (const row of m.slice(1)) { const nm = row[0]; for (let j = 1; j < row.length; j++) o[`${nm}>${hdr[j - 1] ?? j}`] = row[j]; } return { rows: m.slice(1).map((r) => r[0]), cols: hdr.length }; }
const cmInfo = parseM(cm), rmInfo = parseM(rm);
R.note('matrix_dims', { cowork_rows: cmInfo.rows, cols: cmInfo.cols, review_rows: rmInfo.rows.length });
R.check('C7.matrices-include-all-12-teachers', cmInfo.rows.length === 12 && rmInfo.rows.length === 12, { cowork_rows: cmInfo.rows.length, review_rows: rmInfo.rows.length, missing: teachers.map((t) => t.full_name).filter((n) => !cmInfo.rows.includes(n)) }, '12x12 (T Nghĩa appears in all 12 CN panels)', ['extracts/C7-ui-tables.json', 'screenshots/C7-stats-bottom-page.png']);
// compare the cells for the 11 shown teachers (row i, col j in same order of rows list)
const idByName = Object.fromEntries(teachers.flatMap((t) => [[t.full_name, t.id], [t.display_name || t.full_name, t.id]])); // the heatmaps now print the short (display) names const mism = [];
for (let i = 1; i < cm.length; i++) for (let j = 1; j < cm[i].length; j++) { const a = idByName[cm[i][0]], b = idByName[cm[0].length === cm[i].length - 1 ? cmInfo.rows[j - 1] : cmInfo.rows[j - 1]]; if (i === j) continue; const exp = cowork[`${a}>${b}`] || 0; if (+cm[i][j] !== exp) mism.push({ a: cm[i][0], b: cmInfo.rows[j - 1], ui: cm[i][j], exp }); }
R.check('C7.cowork-matrix-values', mism.length === 0, { compared: 11 * 10, mismatches: mism.slice(0, 6), n_mismatch: mism.length }, 'cell = #panels containing both teachers (independent count)', ['extracts/C7-ui-tables.json']);
const mism2 = [];
for (let i = 1; i < rm.length; i++) for (let j = 1; j < rm[i].length; j++) { if (i === j) continue; const a = idByName[rm[i][0]], b = idByName[rmInfo.rows[j - 1]]; const exp = review[`${a}>${b}`] || 0; if (+rm[i][j] !== exp) mism2.push({ reviewer: rm[i][0], setter: rmInfo.rows[j - 1], ui: rm[i][j], exp }); }
R.check('C7.review-matrix-values', mism2.length === 0, { mismatches: mism2.slice(0, 6), n_mismatch: mism2.length }, 'cell = #(panel) with reviewer row and setter col', ['extracts/C7-ui-tables.json']);
const m3 = ui.text.match(/1 phân hiệu[^\n]*\n(\d+)\n(\d+)%/); R.check('C7.campus-composition-24-single', m3 && +m3[1] === 24, m3 && m3[0].replace(/\n/g, ' '), '24 subject panels (12 VL + 12 CN) with 1 campus (single placeholder); RA-027 counts panels per subject');
const hdrCells = cm[0]; R.check('C7.matrix-column-headers-are-names', hdrCells.some((c) => /[A-Za-zÀ-ỹ]/.test(c)), hdrCells, 'column headers identify teachers (UI prints 1..11)', ['screenshots/C7-stats-bottom-page.png']);
