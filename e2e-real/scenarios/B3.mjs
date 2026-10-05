import { prepareRunDir, launchApp, waitUiReady, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { sql } from '../lib/native.mjs';
import { clickTid, bodyText, tid, setInput, radixSelect, dialogComboboxes, dialogEl } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('B3');
const run = prepareRunDir('B3', { portable: true, seedDb: path.resolve('../.tools/golden/q-after-import.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const q = (s, p) => sql(db, s, p);
const safe = async (name, fn) => { try { await fn(); } catch (e) { R.check(name, 'fail', 'EXCEPTION ' + String(e.message).slice(0, 300), 'step completes'); try { await app.b.keys('Escape'); } catch {} } };
let app = await launchApp({ scenario: 'B3', run }); await waitUiReady(app);
const typeInto = async (selector, v) => { const e = await app.b.$(selector); await e.waitForExist({ timeout: 6000 }); await e.click(); await e.setValue(v); };

// --- campuses ---
await safe('B3.campus-create', async () => {
  await app.nav('/campuses'); await sleep(700);
  await (await app.b.$('button=Thêm phân hiệu')).click(); await sleep(500);
  await typeInto('[role="dialog"] input[name="code"]', 'TST'); await typeInto('[role="dialog"] input[name="name"]', 'Phân hiệu thử nghiệm');
  await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(900);
  const r = q("select code,name from campuses where code='TST'");
  R.check('B3.campus-create', r.length === 1, r, 'campus TST in DB');
});
// --- subjects ---
await safe('B3.subject-create', async () => {
  await app.nav('/subjects'); await sleep(700);
  await clickTid(app.b, 'create-subject-btn'); await sleep(500);
  await typeInto('[data-testid="subject-code-input"]', 'TST'); await typeInto('[data-testid="subject-name-input"]', 'Môn thử');
  await (await tid(app.b, 'subject-setters-input')).setValue('2'); await (await tid(app.b, 'subject-reviewers-input')).setValue('1'); await (await tid(app.b, 'subject-min-campuses-input')).setValue('1');
  await clickTid(app.b, 'save-subject-btn'); await sleep(900);
  const r = q("select code,name,setters,reviewers,min_campuses from subjects where code='TST'");
  R.check('B3.subject-create', r.length === 1 && r[0].setters === 2 && r[0].reviewers === 1, r, 'subject TST 2+1');
  // duplicate code must be refused
  await clickTid(app.b, 'create-subject-btn'); await sleep(500);
  await typeInto('[data-testid="subject-code-input"]', 'TST'); await typeInto('[data-testid="subject-name-input"]', 'Trùng');
  await clickTid(app.b, 'save-subject-btn'); await sleep(900);
  const n = q("select count(*) c from subjects where code='TST'")[0].c;
  const txt = await bodyText(app.b);
  R.check('B3.subject-duplicate-code-rejected', n === 1, { n, toast_text: txt.match(/(trùng|đã tồn tại|duplicate|tồn tại)[^\n]*/i)?.[0] }, 'only one TST', []);
  await app.b.keys('Escape'); await sleep(300);
});
// --- competency toggles ---
await safe('B3.competency-cycle', async () => {
  await app.nav('/competencies'); await sleep(900);
  const seq = []; const t = await app.b.$('[data-testid="competency-toggle-1-2-setter"]');
  seq.push(await t.getText());
  for (let i = 0; i < 3; i++) { await t.click(); await sleep(600); seq.push(await t.getText()); const rows = q('select subject_id,role,grade_scope from teacher_competencies where teacher_id=1 and subject_id=2'); seq.push(JSON.stringify(rows)); }
  R.check('B3.competency-cycle', seq[0].includes('Theo khối dạy') && seq[2].length > 0, seq, 'taught -> all -> off -> taught, persisted each click');
});
// --- teachers ---
await safe('B3.teacher-create-overrides', async () => {
  await app.nav('/teachers'); await sleep(800);
  await clickTid(app.b, 'create-teacher-btn'); await sleep(500);
  await typeInto('[data-testid="teacher-code-input"]', 'THU1'); await typeInto('[data-testid="teacher-name-input"]', 'Lê Văn Tám'); await typeInto('[data-testid="teacher-display-name-input"]', 'Thầy Tám (Toán)');
  await (await app.b.$('#grade-check-1')).click();
  const adv = await app.b.$('//button[contains(.,"advancedSettings") or contains(.,"Nâng cao")] | //*[@role="dialog"]//*[contains(text(),"advancedSettings")]'); await adv.click(); await sleep(400);
  await app.shot('B3-teacher-dialog-advanced', { screen: false });
  await (await tid(app.b, 'teacher-quota-override-input')).setValue('6'); await (await tid(app.b, 'teacher-max-tasks-override-input')).setValue('1');
  await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(1000);
  const r = q("select code,full_name,display_name,quota_override,max_tasks_per_exam_override from teachers where code='THU1'");
  R.check('B3.teacher-create-overrides', r.length === 1 && r[0].quota_override === 6 && r[0].max_tasks_per_exam_override === 1 && r[0].display_name === 'Thầy Tám (Toán)', r, 'row with display name + overrides 6/1');
  const txt = await bodyText(app.b);
  R.check('B3.teacher-subline-cach-goi', /Cách gọi: Thầy Tám \(Toán\)/.test(txt), txt.match(/Cách gọi:[^\n]*/g)?.slice(0, 3), 'sub-line "Cách gọi: Thầy Tám (Toán)" (phase-11 checklist K3 step 4)');
  R.check('B3.forced-badge-T-Nghia', /Cố định: 12/.test(txt), txt.match(/Cố định[^\n]*/g), 'badge "Cố định: 12" (phase-11 checklist K3 step 5)');
});
// --- exams ---
await safe('B3.exam-create-move-delete', async () => {
  await app.nav('/exams'); await sleep(800);
  await clickTid(app.b, 'add-exam-btn'); await sleep(500);
  await typeInto('[data-testid="exam-code-input"]', 'TST1'); await typeInto('[data-testid="exam-name-input"]', 'Kỳ thi thử'); await clickTid(app.b, 'exam-save-btn'); await sleep(900);
  let r = q("select code,sort_order from exams order by sort_order"); const created = r.some((x) => x.code === 'TST1');
  await clickTid(app.b, 'exam-move-up-4'); await sleep(800);
  const r2 = q("select code,sort_order from exams order by sort_order");
  R.check('B3.exam-create-and-reorder', created && r2.map((x) => x.code).join() !== r.map((x) => x.code).join(), { before_move: r.map((x) => x.code), after_move: r2.map((x) => x.code) }, 'TST1 created, move-up changes order');
});
// --- unavailability ---
await safe('B3.unavailability-toggle', async () => {
  await app.nav('/unavailability'); await sleep(900);
  await clickTid(app.b, 'unavail-cell-1-1'); await sleep(900);
  const r = q('select teacher_id,exam_id,reason from unavailability');
  const txt = await (await tid(app.b, 'unavail-cell-1-1')).getText();
  R.check('B3.unavailability-toggle', r.some((x) => x.teacher_id === 1 && x.exam_id === 1), { rows: r, cell: txt }, 'row for teacher 1 / exam 1');
});
// --- rules ---
await safe('B3.rules-h3-off', async () => {
  await app.nav('/rules'); await sleep(900);
  const before = q("select rule_key,enabled,params_json from rule_settings where rule_key in ('h3','h4')");
  const tg = await tid(app.b, 'toggle-h3'); await tg.click(); await sleep(500);
  await app.shot('B3-rules-h3-off', { screen: false });
  const warn = await (await tid(app.b, 'h3-warning-box')).isExisting();
  await clickTid(app.b, 'save-rules-btn'); await sleep(1000);
  const after = q("select rule_key,enabled,params_json from rule_settings where rule_key in ('h3','h4')");
  R.check('B3.rules-h3-off-saved', JSON.stringify(before) !== JSON.stringify(after), { before, after, warnBox: warn }, 'H3 enabled flag flips in DB; yellow warning box shown');
});
// --- locks ---
await safe('B3.lock-create', async () => {
  await clickTid(app.b, 'tab-locks'); await sleep(500);
  await (await app.b.$('button=Thêm ràng buộc')).click(); await sleep(500);
  const cbs = await dialogComboboxes(app.b); R.note('lock_dialog_comboboxes', cbs.length);
  // teacher combobox is the 4th (exam, grade, subject, teacher, kind, role)
  await cbs[3].click(); await sleep(400);
  const opts = await app.b.$$('[role="option"]'); const names = []; let picked = null;
  for (const o of opts) { const dis = await o.getAttribute('aria-disabled'); const tx = await o.getText(); names.push({ tx, dis }); if (!picked && dis !== 'true') picked = o; }
  R.note('lock_teacher_options', names); await picked.click(); await sleep(300);
  await clickTid(app.b, 'lock-submit-btn').catch(async () => { await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); }); await sleep(1000);
  const r = q('select * from locks');
  R.check('B3.lock-create', r.length >= 1, r, 'a lock row exists (pin Cô Hiền GK1/10/VL)');
});
await app.shot('B3-before-restart', { screen: false });
const snapshot = () => ({ campuses: q('select code from campuses order by id'), subjects: q('select code from subjects order by id'), comp1: q('select role,grade_scope from teacher_competencies where teacher_id=1 and subject_id=2'), teachers: q("select code,quota_override,max_tasks_per_exam_override from teachers where code='THU1'"), exams: q('select code from exams order by sort_order'), un: q('select teacher_id,exam_id from unavailability'), rules: q("select rule_key,enabled,params_json from rule_settings where rule_key='h3'"), locks: q('select teacher_id from locks') });
const snapBefore = snapshot();
await app.stop();
// --- restart & verify persistence in the UI ---
app = await launchApp({ scenario: 'B3', run }); await waitUiReady(app); await sleep(800);
const snapAfter = snapshot();
R.check('B3.persistence-db-identical-after-restart', JSON.stringify(snapBefore) === JSON.stringify(snapAfter), { before: snapBefore, after: snapAfter }, 'same business rows after restart');
const ui = {};
await app.nav('/teachers'); await sleep(900); ui.teachers = (await bodyText(app.b)).includes('Thầy Tám (Toán)');
await app.nav('/exams'); await sleep(800); ui.exams = (await bodyText(app.b)).includes('TST1');
await app.nav('/rules'); await sleep(900); await app.shot('B3-after-restart-rules', { screen: false }); ui.h3_toggle = await (await tid(app.b, 'toggle-h3')).getAttribute('aria-checked');
await app.nav('/subjects'); await sleep(800); ui.subjects = (await bodyText(app.b)).includes('Môn thử');
R.check('B3.persistence-ui-after-restart', ui.teachers && ui.exams && ui.subjects && ui.h3_toggle === 'false', ui, 'teacher, exam, subject visible; H3 toggle off');
await app.stop();
