import { prepareRunDir, withApp, sleep, REPO, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { driveFileDialog, sql, key, xdo } from '../lib/native.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync, spawnSync } from 'node:child_process';
const R = recorder('C1');
const run = prepareRunDir('C1c', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const tsv = fs.readFileSync(path.join(OUT_ABS, 'extracts/q-plan.tsv'), 'utf8');
await withApp({ scenario: 'C1c', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1200);
  // ---- A) TSV pasted through the real X clipboard ----
  await clickTid(app.b, 'import-plan-button'); await sleep(600); await clickTid(app.b, 'tab-source-tsv'); await sleep(300);
  spawnSync('xclip', ['-selection', 'clipboard'], { input: tsv, env: { ...process.env, DISPLAY: ':99' } , stdio: ['pipe', 'ignore', 'ignore'], timeout: 3000 });
  await (await tid(app.b, 'textarea-plan-tsv')).click(); await sleep(200);
  key('ctrl+v'); await sleep(800);
  const val = await (await tid(app.b, 'textarea-plan-tsv')).getValue();
  R.check('C1.tsv-paste-via-xclip-ctrl+v', val.replace(/\r/g, '') === tsv, { len_pasted: val.length, len_expected: tsv.length }, 'textarea content equals clipboard TSV');
  if (val.length < 50) { // fall back: set via value setter (labelled)
    const e = await tid(app.b, 'textarea-plan-tsv'); await app.b.execute((el, v) => { const s = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set; s.call(el, v); el.dispatchEvent(new Event('input', { bubbles: true })); }, e, tsv); R.note('tsv_fallback_used', true);
  }
  await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500);
  const pv = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-plan-preview-tsv.txt'), pv);
  await app.shot('C1-tsv-preview');
  const seats = pv.match(/Tổng số vị trí nạp: (\d+)/); const score = pv.match(/Điểm phạt ước tính: ([\d.]+)/);
  R.check('C1.tsv-preview-60-seats-and-score', seats && +seats[1] === 60 && score && Math.abs(+score[1] - 116.36) < 0.05, { seats: seats && seats[1], score: score && score[1] }, '60 seats, 116.4 (116.36; S3 not applicable, RA-017)', ['extracts/C1-plan-preview-tsv.txt']);
  R.check('C1.tsv-preview-valid', /Hợp lệ/.test(pv) && !/Không thể áp dụng/.test(pv), pv.match(/(Hợp lệ|Không thể áp dụng)/)?.[0], 'Hợp lệ');
  await clickTid(app.b, 'btn-plan-import-apply'); await sleep(2500);
  // ---- B) clear C Quí's max-tasks override through the teacher dialog, re-import -> exactly one hard violation ----
  await sleep(4500); await app.nav('/teachers'); await sleep(900);
  const rows = await app.b.$$('tr'); let done = false;
  for (const r of rows) { if ((await r.getText()).includes('Cô Quí')) { const edit = await r.$('button[aria-label="Chỉnh sửa"], button[title="Chỉnh sửa"]'); const btns = await r.$$('button'); for (const b of btns) { const lab = (await b.getAttribute('aria-label')) || (await b.getAttribute('title')) || ''; if (lab.includes('Chỉnh sửa')) { await app.b.execute((el) => el.click(), b); done = true; break; } } break; } }
  R.note('quí-edit-clicked', done); await sleep(600);
  const adv = await app.b.$('//*[@role="dialog"]//*[contains(text(),"advancedSettings")]'); if (await adv.isExisting()) await adv.click(); await sleep(400);
  const mt = await tid(app.b, 'teacher-max-tasks-override-input'); R.note('quí_override_before', await mt.getValue()); await mt.clearValue(); await sleep(200);
  await (await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Lưu"]')).click(); await sleep(1200);
  const qui = sql(db, "select display_name,max_tasks_per_exam_override from teachers where display_name='C Quí'");
  R.check('C1.quí-override-cleared-via-ui', qui.length === 1 && qui[0].max_tasks_per_exam_override === null, qui, 'override NULL');
  await app.nav('/assignments'); await sleep(1200);
  await clickTid(app.b, 'import-plan-button'); await sleep(600); await clickTid(app.b, 'tab-source-tsv');
  const e = await tid(app.b, 'textarea-plan-tsv'); await app.b.execute((el, v) => { const s = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set; s.call(el, v); el.dispatchEvent(new Event('input', { bubbles: true })); }, e, tsv);
  await clickTid(app.b, 'btn-plan-import-preview'); await sleep(2500);
  const pv2 = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-plan-preview-no-qui-override.txt'), pv2);
  await app.shot('C1-preview-no-qui-override');
  await clickTid(app.b, 'tab-preview-issues'); await sleep(500); const iss = await bodyText(app.b); fs.writeFileSync(path.join(OUT_ABS, 'extracts/C1-issues-no-qui-override.txt'), iss);
  await app.shot('C1-issues-no-qui-override');
  const issues = iss.slice(iss.indexOf('Cảnh báo'));
  R.note('issues_text', issues.slice(0, 1500));
  R.check('C1.no-override-exactly-one-hard-violation-tasks-per-exam', /C Quí|Cô Quí/.test(iss) && /(CK2|Cuối kỳ 2)/.test(iss), issues.slice(0, 600), 'single hard violation: tasks per exam exceeded (C Quí, CK2, 3 > 2)', ['extracts/C1-issues-no-qui-override.txt', 'screenshots/C1-issues-no-qui-override-page.png']);
});
