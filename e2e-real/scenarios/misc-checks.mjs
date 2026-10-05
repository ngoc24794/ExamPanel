// Checks that could not be (fully) executed on Linux, recorded explicitly so the matrix never implies coverage.
import { recorder } from '../lib/rec.mjs';
let R = recorder('A3');
R.check('A3.second-launch-focuses-first-window', 'not-verified', 'second launch exits 0 in 49 ms and creates no window; X11 active-window/focus under openbox was not asserted', 'first window raised/focused (tauri-plugin-single-instance callback show()+set_focus())', ['extracts/A3.json']);
R.check('A3.no-second-db-connection', 'not-verified', 'only one app PID exists; open file descriptors on exam-panel.db were not enumerated by this harness', 'single connection', ['extracts/A3.json']);
R = recorder('C5');
R.check('C5.drag-swap-with-real-mouse', 'manual-required', 'WebKitWebDriver native dragAndDrop did not trigger HTML5 DnD; synthetic DragEvent used (see C5.drag-swap)', 'real mouse drag on a physical machine / WebView2', ['extracts/C5.json']);
R = recorder('D2');
R.check('D2.real-app-print-dialog-and-pdf', 'blocked', 'window.print() is a no-op in the WebKitGTK app and WebDriver Print Page is unsupported (RA-030)', 'Print dialog -> PDF from the real app', ['screenshots/D2-gtk-print-dialog.png', 'extracts/D2.json']);
R.check('D2.notices-filter-toggle-all-vs-with-tasks', 'not-verified', 'filter control exists on /print/notices (visible in screenshots) but toggling was not driven; PDF replay used default filter (12 teachers all with duties)', 'both filters give correct page counts', ['extracts/D2-pdfchecks.json']);
R = recorder('B2');
R.check('B2.real-school-file-15-50-teachers', 'not-verified', 'no real school data available (rule 3); Q-shaped 12-teacher synthetic data used', 'import of a real 15-50 teacher file (v0.1.0 checklist §3)', []);
R = recorder('A4');
R.check('A4.windows-usb-write-protect', 'manual-required', 'Windows/USB behaviour cannot be exercised on Linux', 'see windows-manual-checklist.md W5', ['windows-manual-checklist.md']);
console.log('ok');
{ const R2 = recorder('B2');
  R2.check('B2.preview-feasibility-diagnostics', 'not-verified', 'preview text of the feasible Q data shows only teacher/campus/absence tabs and statuses (extracts/B2-preview-text.txt); no feasibility simulation block appeared and the negative case (teacher without competency, H3 pool reduction: teacher_no_competency / h3_reviewer_pool_reduced) was not exercised by this run', 'warnings with diagnostic codes when the imported data would make the problem infeasible (phase-12 checklist K4.4)', ['extracts/B2-preview-text.txt']); }
{ const R3 = recorder('B2'); const fs = await import('node:fs'); const path = await import('node:path'); const { OUT_ABS } = await import('../lib/harness.mjs');
  const t = fs.readFileSync(path.join(OUT_ABS, 'extracts/B2c-upsert-preview.txt'), 'utf8'); const row = (await (async () => { const j = JSON.parse(fs.readFileSync(path.join(OUT_ABS, 'extracts/B2.json'), 'utf8')); return (j.checks.find((c) => c.step === 'B2.upsert-preview-new-teacher-row') || {}).observed || {}; })()).gv999_row || '';
  R3.check('B2.preview-active-column-shows-Có/Không', /(Có|Đang dạy)\s*-?\s*$/.test(row) ? 'pass' : 'fail', { gv999_row: row, expected_last_cells: 'Hệ số tải 1 | Đang dạy Có | Ghi chú -' }, 'column "Đang dạy" shows Có/Không (sheet value "Có")', ['extracts/B2c-upsert-preview.txt', 'screenshots/B2-import-preview-page.png']); }
