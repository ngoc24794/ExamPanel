# ExamPanel User Guide (English Edition)

**ExamPanel** is an automated scheduling and assignment application for high school exam panels (test authors and reviewers). It ensures fairness, workload balancing, campus quota satisfaction, and strict administrative compliance.

---

## 1. Quick Start

ExamPanel operates in 4 streamlined steps:
1. **Import Data:** Use the standardized Excel template (`.xlsx`) to import teacher rosters, exam terms, subjects, and grade levels.
2. **Configure Rules & Criteria:** Select an optimization preset (**Cân bằng toàn diện** / Balanced, **Ưu tiên phản hiệu** / Campus-focused, or **Kinh nghiệm** / Seniority-first) or customize weights for hard/soft constraints.
3. **Automated Solver:** Click **Tạo phân công** (Generate Plan). The mathematical solver (simulated annealing + constraint satisfaction) assigns authors and reviewers in seconds, guaranteeing 0 hard constraint violations and optimal fairness.
4. **Export & Print:** Export the full assignment matrix and summary to Excel (`.xlsx`), print A4 formal administrative plans, or generate individual teacher assignment notice slips.

---

## 2. Installation & Running

### Windows Portable Edition
- Extract `ExamPanel-0.1.0-windows-x64-portable.zip` anywhere (e.g. USB flash drive, Desktop, or Documents).
- The marker file `ExamPanel.portable` ensures all database files, logs, and backups stay within the `data/` folder next to `ExamPanel.exe`.
- Run `ExamPanel.exe`.
- *Note:* If Windows SmartScreen displays a warning, click **"More info"** and then **"Run anyway"**.

### Windows Installer (NSIS)
- Run `ExamPanel_0.1.0_x64-setup.exe`.
- Installs to user application data (`%LOCALAPPDATA%`), creates desktop and Start Menu shortcuts, and integrates a Microsoft Edge WebView2 bootstrapper.

---

## 3. Data Management & Templates

- **Excel Template:** Download the official template from the Import screen. Fill in:
  - `GIAO_VIEN`: Teacher code, name, campus (phân hiệu), quota, and skills.
  - `KY_THI`: Terms, grades (10, 11, 12), and exam sessions.
- **Validation:** ExamPanel validates roster structure, detects duplicate codes, and verifies quota sufficiency before saving.

---

## 4. Constraint Rules & Presets

- **Hard Constraints (Must be satisfied):**
  - No simultaneous authoring and reviewing for the same exam slot.
  - Campus quotas must be strictly respected.
  - Hard avoid-pairs (conflict of interest or mutual exclusion).
- **Soft Objectives (Balanced optimization):**
  - Workload spread evenly across academic terms.
  - Pair diversity (avoiding repeated reviewer-author partnerships).
  - Subject matter seniority and specialized test-format assignments.

---

## 5. Manual Tweaks & Plan Finalization

- **Interactive Matrix View:** View assignments by term, grade, and campus.
- **Drag & Drop / Swap:** Swap assignments between teachers with real-time constraint validation and penalty delta calculation.
- **Locking:** Pin confirmed assignments before re-running the solver.
- **Plan Finalization (Chốt phương án):** Freezes the plan into an official record and removes the draft watermark from printouts.

---

## 6. Export, Printing & Notices

- **Excel Workbook (`.xlsx`):** Generates two sheets: `MA_TRAN` (color-coded assignment matrix) and `TONG_HOP` (teacher summary and quota completion).
- **A4 Formal Printout:** Administrative letterhead adhering to Vietnamese educational document standards (Quốc hiệu, Tiêu ngữ, Tên trường, nơi nhận, chữ ký). Draft plans display a subtle watermark; finalized plans do not.
- **Individual Notice Slips:** Generates personalized notices for each teacher with confidentiality reminders and scheduled assignments.

---

## 7. Backup, Restore & Trial Mode

- **Automatic Backups:** Created on key actions and retained up to 10 historical snapshots in `data/backups/`.
- **Safe Restore:** Allows rolling back to any snapshot or importing a `.db` file from another machine. Current state is automatically cached before restoring.
- **Trial Mode Sandbox:** Settings ➔ "Dùng thử với dữ liệu mẫu" switches to `data/demo.db` (18 teachers, 2 campuses, 4 terms) with a persistent amber banner. Your real school database is never touched.

---

## 8. Troubleshooting & Logs

- **WebView2:** Required for UI rendering. Included in modern Windows 10/11. If missing, install Microsoft Edge WebView2 Evergreen Runtime.
- **Write-Protected USB:** If running in portable mode on a locked USB, ExamPanel alerts you and offers switching to local storage.
- **Logs:** Accessible via Settings ➔ "Mở thư mục nhật ký" (`data/logs/exampanel.log`).
