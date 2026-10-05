# Findings

43 findings. Severity: S0 data loss/crash/wrong result/hard-constraint violation shipped · S1 blocks a core flow · S2 major usability / Q workflow not met · S3 minor · S4 cosmetic. Evidence paths are relative to this folder. `engine` tags: engine-agnostic | webkitgtk-specific | unknown. Causes are **hypotheses** unless stated.

| ID | Sev | Category | Area | Engine | Title |
|---|---|---|---|---|---|
| RA-001 | S1 | bug | app shell | engine-agnostic (Linux GTK confirmed; Wi | Startup error dialogs deadlock the main thread on Linux (blank window, no dialog) — read-only portable folder, DB newer than app, corrupted DB |
| RA-007 | S2 | spec-mismatch | storage | engine-agnostic | Opening an older (v4) database migrates it in place with no automatic pre-migration backup |
| RA-009 | S2 | bug | export | engine-agnostic | Excel export 'Tiêu chí' sheet labels S3-S8 with wrong criterion names, truncates S8 violation count, and omits lower bounds |
| RA-011 | S2 | mock-vs-real gap | ui | engine-agnostic | Cancel during optimization still saves partial plans (incl. a 312.36-penalty plan), shows both 'Đã hủy' and 'Đã tạo 3 phương án mới thành công', and closes the dialog |
| RA-013 | S2 | mock-vs-real gap | ui | engine-agnostic | Plan compare dialog is subject-blind: wrong 'Khoảng cách sai khác', cells hide the VL reviewer / merge subjects' setters |
| RA-014 | S2 | Q-fit | ui | engine-agnostic (layout/CSS widths; WebK | Q-style grid does not fit 1280x720: Khối 12 columns and last totals column are clipped, page needs vertical scroll |
| RA-018 | S2 | mock-vs-real gap | ui | engine-agnostic | 'Tạo bản chỉnh sửa' creates the copy but usually leaves the view on the read-only optimizer plan (switches in 2 of 10 runs) |
| RA-019 | S2 | bug | core | engine-agnostic | Candidate/replace Δ-score is computed for the OTHER setter seat when DB positions differ from the engine's internal setter order (evaluate_candidates disagrees with full evaluate) |
| RA-020 | S2 | bug | ui | engine-agnostic | Header score and 'Hợp lệ' do not update after edits (replace or drag-swap); an invalid drag-swap is accepted silently and Save fails with 'Dữ liệu đã tồn tại, không thể tạo trùng lặp' |
| RA-022 | S2 | mock-vs-real gap | ui | engine-agnostic | 'Dữ liệu đã đổi' (stale) badge/banner do not appear after changing data until the window is reloaded |
| RA-027 | S2 | bug | ui | engine-agnostic | Statistics heatmaps are subject-blind and truncated to the first 11 teachers; columns are numbered, T Nghĩa is missing |
| RA-028 | S2 | bug | export | engine-agnostic | Excel 'Theo giáo viên' sheet: 'Cùng ban với' lists members of BOTH subjects' panels (wrong in all 60 rows) |
| RA-031 | S2 | bug | storage | engine-agnostic | Automatic backup rotation and 'newest first' ordering sort by file NAME (reason prefix before timestamp): newest backups are pruned while older ones of another kind are kept; same-second backups overwrite each other |
| RA-033 | S2 | bug | ui | engine-agnostic | Language chosen on the Settings page is not persisted (reverts to Vietnamese after restart); only the header switcher persists |
| RA-034 | S2 | bug | app shell | engine-agnostic (capability file is plat | 'Mở thư mục dữ liệu' always fails: 'Command plugin:opener\|open_path not allowed by ACL' (capability opener:default lacks open_path); 'Mở thư mục nhật ký' gives no feedback |
| RA-036 | S2 | packaging/runtime | packaging | engine-agnostic | Release build ships the dev-tools feature: the `seed_demo` IPC command is live and writes demo data into the REAL database |
| RA-002 | S3 | i18n/text | ui | engine-agnostic | Import preview: 'Đang dạy' column shows 'Mới' (status key) instead of Có/Không |
| RA-005 | S3 | environment | docs | engine-agnostic | Existing mock Playwright suite: cannot launch pinned browser in offline CDN sandbox, and a run rewrites 94 tracked docs files |
| RA-010 | S3 | i18n/text | ui | engine-agnostic | Import plan preview shows hard violation as raw code 'h4: max_tasks_per_exam_exceeded' without teacher/exam/context |
| RA-012 | S3 | bug | ui | engine-agnostic | Optimization progress bar and best score go backwards with Chuẩn/Kỹ (progress is per parallel wave, not overall) |
| RA-015 | S3 | Q-fit | ui | engine-agnostic | Q-grid deviations from the paper layout: 'Khối Khối 10', exam names instead of GK1/CK1/GK2/CK2, '—' instead of blank second CN 'Đề' cell |
| RA-017 | S3 | Q-fit | ui | engine-agnostic | With a single placeholder campus, S3 adds a constant 144 penalty and puts a warning icon on every VL seat |
| RA-021 | S3 | i18n/text | ui | engine-agnostic | Candidate dialog shows raw engine codes as reasons (e.g. 'H4: max_setter_tasks_per_exam_exceeded', 'H2: unqualified_grade') instead of Vietnamese text |
| RA-023 | S3 | bug | core | engine-agnostic | Marking a teacher unavailable produces an extra spurious H2 'unqualified_grade' hard violation for the same seat |
| RA-024 | S3 | bug | ui | webkitgtk-specific (Chromium/WebView2 pa | Plan history shows 'Invalid Date' for every plan (WebKitGTK cannot parse SQLite 'YYYY-MM-DD HH:MM:SS') |
| RA-026 | S3 | Q-fit | ui | engine-agnostic | Re-optimize around kept seats: setter order (Đề 1/Đề 2) of kept seats is not preserved, new plan reuses an existing name, kept-seat banner only exists in 'Chi tiết' view, k/effort not selectable |
| RA-029 | S3 | Q-fit | export | engine-agnostic | Export/print fall back to invented organisation text 'TRƯỜNG THPT CHUYÊN / TỔ CHUYÊN MÔN TOÁN / Hà Nội' when Settings org info is empty |
| RA-030 | S3 | packaging/runtime | app shell | webkitgtk-specific (WebView2 on Windows  | 'In / Xuất PDF' does nothing in the Linux/WebKitGTK app (no print dialog, no PDF); WebDriver Print Page unsupported |
| RA-032 | S3 | i18n/text | ui | engine-agnostic | Restore dialog shows English engine messages and calls a newer-version file 'corrupted or wrong format' |
| RA-037 | S3 | docs | docs | engine-agnostic | Data-location documentation contradicts the implementation (SPEC §6 / ADR 'writable folder => portable' vs marker-file requirement; v0.1.0 checklist %APPDATA% path) |
| RA-038 | S3 | docs | docs | engine-agnostic | Release/QA checklists describe UI and files that no longer exist (stale steps) |
| RA-039 | S3 | accessibility | ui | engine-agnostic | Keyboard-only use of the plan grid: seats are not focusable and the only tab stops are 60 invisible (opacity 0) menu buttons |
| RA-040 | S3 | i18n/text | ui | engine-agnostic | 10 i18n keys are missing from BOTH locale files, so raw keys are shown (common.all, rules.h3Warning, rules.h4MaxTasksPerExam, rules.h4MaxSetterPerExam, rules.s1MaxTasksMode, rules.s1ModeAuto, teachers.advancedSettings, common.close, feasibility.forcedPlacementsTitle/Desc) |
| RA-042 | S3 | i18n/text | ui | engine-agnostic | English UI is incomplete: ~20 hard-coded Vietnamese strings remain on Overview, Competencies, Unavailability, Statistics, Settings, Compare/Import/Quota dialogs and the feasibility sheet; Vietnamese UI shows English 'Close' |
| RA-043 | S3 | packaging/runtime | app shell | engine-agnostic | data/logs/app.log is empty in all 112 runs: no startup line, no IPC/DB errors, nothing for the hung startup dialogs — only a deliberate E4 probe line was ever written |
| RA-003 | S4 | i18n/text | ui | engine-agnostic | Grid header reads 'Khối Khối 10' (duplicated word) in Q-style grid |
| RA-004 | S4 | Q-fit | ui | engine-agnostic | Production dashboard still shows developer 'Kết nối Hệ thống Lõi' ping panel ('pong from ExamPanel core') |
| RA-006 | S4 | docs | docs | engine-agnostic | DB file name inconsistent in docs: code/SPEC/ADR/DOC-TOI say exam-panel.db; v0.1.0 checklist and user guide say exampanel.db |
| RA-008 | S4 | bug | storage | engine-agnostic | Every DB open rewrites the database file (sqlite_sequence for 'grades' grows by 3 per open via seed_defaults) |
| RA-016 | S4 | i18n/text | ui | engine-agnostic | S8 chip prints raw float '2.545454545454545' in the plan summary bar |
| RA-025 | S4 | i18n/text | ui | engine-agnostic | Mark-final errors raise two toasts: the translated sentence plus the raw code ('plan_stale' / 'plan_invalid'); hard-violation count differs between toast and header |
| RA-035 | S4 | docs | ui | engine-agnostic | 'Third-party licences' dialog shows a 3-line summary, not the THIRD_PARTY_NOTICES text (and is English-only) |
| RA-041 | S4 | bug | ui | engine-agnostic | Double-click on 'Lưu' (Exam dialog) saves once but shows two red 'Dữ liệu đã tồn tại, không thể tạo trùng lặp' toasts plus the success toast |

<a id="ra-001"></a>
## RA-001 — Startup error dialogs deadlock the main thread on Linux (blank window, no dialog) — read-only portable folder, DB newer than app, corrupted DB

- **Severity:** S1  **Category:** bug  **Area:** app shell  **Engine:** engine-agnostic (Linux GTK confirmed; Windows not verified)
- **Scenarios:** A4, A5, A6
- **Reproduction:** 1) Copy release exe + ExamPanel.portable to a folder, chmod 555 the folder (as unprivileged user). 2) Launch the exe under Xvfb. 3) Wait 12 s. (Same hang for DB with PRAGMA user_version=99 and for a corrupted exam-panel.db with a backup present.)
- **Expected:** Vietnamese modal dialog (read-only: Use app-data / Exit; newer DB: friendly message; corrupted: offer restore) as coded in src-tauri/src/lib.rs setup().
- **Actual:** A blank black main window stays open forever; no dialog window exists (xdotool lists only 'ExamPanel'); process alive with main thread blocked (futex). gdb backtrace of the main thread: exam_panel_app::run::{closure#4} (setup) -> tauri_plugin_dialog::MessageDialogBuilder::blocking_show -> mpmc Receiver::recv -> Thread::park, i.e. blocking_show() is called on the event-loop thread inside setup() before the loop can service the dialog request. User sees nothing and must kill the process.
- **Evidence:** [A4.json](extracts/A4.json), [A5.json](extracts/A5.json), [A6.json](extracts/A6.json), [A4-hang-main-thread-backtrace.txt](logs/A4-hang-main-thread-backtrace.txt), [A4-after-12s.png](screenshots/A4-after-12s.png), [A5-after-12s.png](screenshots/A5-after-12s.png), [A6-after-12s.png](screenshots/A6-after-12s.png)
- **Suspected cause (hypothesis):** src-tauri/src/lib.rs setup(): app.dialog()...blocking_show() on the main thread. tauri-plugin-dialog docs say blocking_show must not run on the main thread. Fix idea: show the dialog from a spawned thread or use the non-blocking show() + channel, or run the checks before tauri::Builder::run using rfd directly. May behave differently on Windows (MessageBox) — must be tested there (see windows-manual-checklist).
- **Mock suite passes the same step?** Mock suite cannot reach these startup paths at all.
- **What Q would notice:** On a locked USB stick or after a corrupted file, ExamPanel opens a white/black window and never responds, with no message.

<a id="ra-007"></a>
## RA-007 — Opening an older (v4) database migrates it in place with no automatic pre-migration backup

- **Severity:** S2  **Category:** spec-mismatch  **Area:** storage  **Engine:** engine-agnostic
- **Scenarios:** A7
- **Reproduction:** Copy crates/storage/fixtures/v4_synthetic.db to <exe>/data/exam-panel.db, launch the app.
- **Expected:** (Task expectation) automatic backup in data/backups before migration. NOTE: SPEC/ADR-0042 do not promise one — expectation may be wrong; but release checklist section 7 upgrade path is exactly this case.
- **Actual:** user_version 4 -> 5, all row counts preserved, plans identical (A7 passes data checks), UI shows all 12 teachers and the plan — but data/backups does not exist: the only copy of the pre-upgrade DB is overwritten.
- **Evidence:** [A7.json](extracts/A7.json)
- **Suspected cause (hypothesis):** Store::open_at runs run_migrations without calling create_automatic_backup (crates/storage/src/store.rs). Believe the app matches the written spec, but a pre-migration backup is cheap data-loss insurance for a v0.1.0->v0.2.0 upgrade.
- **Mock suite passes the same step?** Not reachable in mock.
- **What Q would notice:** If an upgrade ever fails midway, Q has no automatic copy of the old file.

<a id="ra-009"></a>
## RA-009 — Excel export 'Tiêu chí' sheet labels S3-S8 with wrong criterion names, truncates S8 violation count, and omits lower bounds

- **Severity:** S2  **Category:** bug  **Area:** export  **Engine:** engine-agnostic
- **Scenarios:** C1, D1
- **Reproduction:** Import Q's table (docs/reports/phase-12/plan-grid.xlsx), click Xuất Excel, open sheet 'Tiêu chí'.
- **Expected:** Rule names identical to the Rules screen/SPEC (S3 Phản biện độc lập phân hiệu, S4 Đa dạng cặp ra đề, S5 Tránh phản biện chéo lặp lại, S6 Giãn cách kỳ thi liên tiếp, S7 Luân chuyển khối lớp, S8 Cân bằng tải trọng thực tế); 'Số vi phạm' = engine units (S8 4.545); phase-9 checklist says the sheet shows lower bounds.
- **Actual:** S3 'Tránh liên tiếp 2 kỳ làm cùng vai trò', S4 'Tránh liên tiếp 2 kỳ làm cùng khối', S5 'Tránh cặp đôi lặp lại', S6 'Tránh ban thi toàn giáo viên dạy khối đó', S7 'Phân bổ đều giữa các khối', S8 'Phân hiệu chính cho môn học' (!) — descriptions of other rules; S8 'Số vi phạm' = 4 while contribution = 36.36 (=4.545x8); no lower-bound column. Numeric contributions themselves match the UI and engine (S1 10, S2 3, S3 144, S5 6, S6 16, S8 36.36, S9 45).
- **Evidence:** [C1-q-plan-export.json](extracts/C1-q-plan-export.json), [C1-ui-detail-view.txt](extracts/C1-ui-detail-view.txt), [probe-forms.json](extracts/probe-forms.json)
- **Suspected cause (hypothesis):** crates/service/src/excel/export.rs lines ~554-559 hard-coded rule label table out of date vs ui locale names; units cast to integer.
- **Mock suite passes the same step?** Mock suite does not open exported xlsx content.
- **What Q would notice:** Q opens the 'Tiêu chí' sheet and sees S8 described as 'Phân hiệu chính cho môn học' although the school has no campuses; criteria names differ from the app.

<a id="ra-011"></a>
## RA-011 — Cancel during optimization still saves partial plans (incl. a 312.36-penalty plan), shows both 'Đã hủy' and 'Đã tạo 3 phương án mới thành công', and closes the dialog

- **Severity:** S2  **Category:** mock-vs-real gap  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C2, E3
- **Reproduction:** Assignments > Chạy tối ưu > Kỹ > Bắt đầu > after ~0.7 s click 'Hủy bỏ'. Inspect plans table and toasts.
- **Expected:** phase-8 checklist step 2 and task C2: run stops, 'Đã hủy' only, no plans saved.
- **Actual:** 3 plans were saved (scores 176.36, 176.36 and 312.36 — the last is an unfinished anneal) and both toasts appear within 0.5 s; dialog closes and the first plan opens. Same on a second run: plan count 9 -> 12 after Cancel. Leaving the page while a run is in progress behaves the same: E3 started Kỹ, navigated to Giáo viên after 0.9 s and 3 plans were still saved (4 -> 7) with a success toast 'Đã tạo 3 phương án mới thành công' shown on the other page; the busy flag is released afterwards (next run works).
- **Evidence:** [C2b-cancel-timeline.json](extracts/C2b-cancel-timeline.json), [C2.json](extracts/C2.json), [C2b-after-cancel-page.png](screenshots/C2b-after-cancel-page.png), [E3.json](extracts/E3.json)
- **Suspected cause (hypothesis):** crates/core/src/optimize/mod.rs optimize(): on cancel the annealers return their best-so-far and optimize() still returns Ok(plans). RunOptimizeDialog.tsx only treats error code 'cancelled' as a cancel, but only the MOCK API (ui/src/lib/api/mock.ts:1153) ever throws that code; the real backend never does. Fix: service.run_optimize should return AppError cancelled when the flag is set (and the UI must not save).
- **Mock suite passes the same step?** YES — mock throws 'cancelled' so the mock e2e/unit tests pass; real backend returns Ok.
- **What Q would notice:** Q presses Cancel, gets a 'cancelled' message AND three new plans, one of them clearly worse.

<a id="ra-013"></a>
## RA-013 — Plan compare dialog is subject-blind: wrong 'Khoảng cách sai khác', cells hide the VL reviewer / merge subjects' setters

- **Severity:** S2  **Category:** mock-vs-real gap  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C3
- **Reproduction:** Golden Q plan (imported) vs optimizer plan #1 > Phân công > So sánh phương án.
- **Expected:** Distance = number of seats whose teacher differs (39 for this pair, computed from the DB assignments with subject_id); each (exam,grade) cell shows the VL and CN panels separately.
- **Actual:** 'Khoảng cách sai khác: 33 ô thay đổi'; cell GK1/Khối 10 for Q's plan reads 'A: Thầy Nghĩa, Cô Hiền, Cô Lài | PB: Cô Hiền' — the VL reviewer T Phúc is not shown, both subjects' setters are merged; the spec/phase-11 checklist says per-subject. Engine per-rule numbers in the same dialog are correct. No per-teacher differences section exists.
- **Evidence:** [C3-compare-dialog.txt](extracts/C3-compare-dialog.txt), [C3-engine-plans.json](extracts/C3-engine-plans.json), [C3.json](extracts/C3.json), [C3-compare-dialog-page.png](screenshots/C3-compare-dialog-page.png)
- **Suspected cause (hypothesis):** ui/src/pages/assignments/PlanCompareModal.tsx comparison useMemo keys by exam+grade+role only (reviewerA = assignA.find(role==='reviewer')) and ignores subject_id; core plan_distance (crates/core/src/optimize/mod.rs) also keys (exam,grade,role,teacher) without subject (36 vs 39 seat-aware).
- **Mock suite passes the same step?** Mock fixtures/tests for compare are single-subject-shaped.
- **What Q would notice:** Compare screen hides part of Q's VL panels and reports a wrong distance.

<a id="ra-014"></a>
## RA-014 — Q-style grid does not fit 1280x720: Khối 12 columns and last totals column are clipped, page needs vertical scroll

- **Severity:** S2  **Category:** Q-fit  **Area:** ui  **Engine:** engine-agnostic (layout/CSS widths; WebKitGTK scrollbar/font metrics may shift pixels slightly)
- **Scenarios:** A8, C4
- **Reproduction:** Golden Q DB, window 1280x720, Phân công > Bảng tổ with any 6-column plan.
- **Expected:** Layout spec (phase-12 checklist K1.3, task C4): whole grid visible at 1280x720 without scrolling, totals panel attached on the right.
- **Actual:** Grid table 684 px inside a clipped container: Khối 12 header and VL/CN cells cut off at x~855; totals panel clips 'Cuối kỳ 2'; grid starts at y=364 and ends at y=843 (> 720), main scroller 865 vs 664 px. At 1920x1080 the grid fits but the totals panel still has a horizontal scrollbar overlapping the 'Tổng cộng' row and content is capped at ~1150 px wide. A8 sweep: at 1024x700 the page stacks the totals panel under the grid and the whole 6-column grid fits (so the clipped two-column layout is specific to widths where the side-by-side layout starts, e.g. 1280); the A8 overflow heuristic flags /assignments at 1280x720 (child right 940 > card right 856).
- **Evidence:** [C4.json](extracts/C4.json), [C4-grid-structure.json](extracts/C4-grid-structure.json), [C4-grid-1280x720-light-page.png](screenshots/C4-grid-1280x720-light-page.png), [C4-grid-1920x1080-dark-page.png](screenshots/C4-grid-1920x1080-dark-page.png), [A8-screens-metrics.json](extracts/A8-screens-metrics.json), [A8-1024x700-assignments-page.png](screenshots/A8-1024x700-assignments-page.png)
- **Suspected cause (hypothesis):** ui/src/pages/assignments/QPlanGrid.tsx fixed min-widths + header block (title, S-chips, toolbar) occupying ~360 px above the grid; max-w container on AssignmentsPage.
- **Mock suite passes the same step?** Mock e2e runs at 1280x800 viewport and only screenshots; no bounding-box assertion.
- **What Q would notice:** On a 1280x720 laptop Q cannot see Khối 12 without scrolling sideways.

<a id="ra-018"></a>
## RA-018 — 'Tạo bản chỉnh sửa' creates the copy but usually leaves the view on the read-only optimizer plan (switches in 2 of 10 runs)

- **Severity:** S2  **Category:** mock-vs-real gap  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C5
- **Reproduction:** Fresh app on a DB with optimizer plans > Phân công > 'Tạo bản chỉnh sửa' > wait 4 s. Repeat in 10 fresh launches.
- **Expected:** Edit toolbar (undo/redo/save) appears on the new 'Phương án #1 (Chỉnh sửa)' copy every time (phase-8 checklist step 4).
- **Actual:** Plan 'Phương án #N (Chỉnh sửa)' is saved (history count +1) but the page keeps showing the optimizer plan with the same button; only 2/10 attempts switched. A user clicks again and creates duplicate copies. The checklist also expects a name prompt; the app silently names the copy with a hard-coded Vietnamese suffix.
- **Evidence:** [C5-copy-switch-attempts.json](extracts/C5-copy-switch-attempts.json), [C5.json](extracts/C5.json)
- **Suspected cause (hypothesis):** AssignmentsPage.tsx: handleCreateEditableCopy calls setSelectedPlanId(newId) right after mutateAsync, while the effect at lines 87-99 resets selection to plans[0] when the new id is not yet in the cached plans list (query invalidation race). Mock API resolves synchronously so it always works there.
- **Mock suite passes the same step?** YES — mock suite passes.
- **What Q would notice:** Q presses 'Tạo bản chỉnh sửa' and nothing seems to happen.

<a id="ra-019"></a>
## RA-019 — Candidate/replace Δ-score is computed for the OTHER setter seat when DB positions differ from the engine's internal setter order (evaluate_candidates disagrees with full evaluate)

- **Severity:** S2  **Category:** bug  **Area:** core  **Engine:** engine-agnostic
- **Scenarios:** C5
- **Reproduction:** Golden Q DB (optimizer plan #1 or any plan whose setters are not stored in teacher-id order). Via UI: open an editable copy, click GK1/Khối 10/VL first 'Đề' cell (C Như, position 0) and read Δ for Cô Lài (+11.0, Tổng 187.4), Thầy Phúc (+5.0), Thầy Lộc (+5.0), Cô Như (+9.0 although she already holds the seat). Independently call evaluate_assignments after really replacing that seat: +18.0 (194.36) for all three, 0 for Cô Như. Doing the same for position 1 returns +18 — the values swap: incremental(position 0) == full(replace position 1) and vice-versa (table in extracts).
- **Expected:** delta_score/new_total == full evaluate() of the edited plan (documented 'correctness guarantee' in eval_edit.rs and unit test test_evaluate_candidates_delta_matches_full_eval).
- **Actual:** For valid candidates, Δ is computed for the other setter seat of the same panel; the candidate for the current holder shows a non-zero Δ; ordering of the list (sorted by Δ) is therefore wrong. Hard-violation flags are correct (12/12 match an independent re-check). After 'Áp dụng' and Save the true score is 194.36, not the previewed 181.4.
- **Evidence:** [C5f-incremental-seat-mapping.json](extracts/C5f-incremental-seat-mapping.json), [C5e-evaluate-candidates-vs-full.json](extracts/C5e-evaluate-candidates-vs-full.json), [C5d-candidate-delta-vs-engine.json](extracts/C5d-candidate-delta-vs-engine.json), [C5c-live-feedback.json](extracts/C5c-live-feedback.json), [C5.json](extracts/C5.json)
- **Suspected cause (hypothesis):** crates/core/src/optimize/eval_edit.rs evaluate_candidates(): SlotRole::Setter1/Setter2 taken from IncrementalState::new(), which seems to order a panel's setters canonically (e.g. by teacher id) instead of by Assignment.position, while the hard-violation path replaces the n-th setter in array order. Same mapping would affect evaluate_swap and reoptimize 'keep slot' (check C6).
- **Mock suite passes the same step?** Mock candidate evaluation is a JS stub; Rust unit test uses solver-generated plans whose order already matches.
- **What Q would notice:** The suggested replacement says +5 but really costs +18; the best-looking candidate is not the best.

<a id="ra-020"></a>
## RA-020 — Header score and 'Hợp lệ' do not update after edits (replace or drag-swap); an invalid drag-swap is accepted silently and Save fails with 'Dữ liệu đã tồn tại, không thể tạo trùng lặp'

- **Severity:** S2  **Category:** bug  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** A9, C5
- **Reproduction:** Editable copy: (a) replace GK1/10/VL first Đề by Thầy Phúc -> header stays 'Tổng điểm phạt: 176.36' (true 194.36 appears only after Save). (b) Drag C Na (CK1/10 VL P.Biện slot) onto T Lộc's seat in GK2/11 VL so C Na is both setter and reviewer of that panel -> header still 'Hợp lệ (0 vi phạm)', S-chips unchanged; 'Lưu thay đổi' fails with toast 'Dữ liệu đã tồn tại, không thể tạo trùng lặp' and the edit stays dirty.
- **Expected:** phase-8 step 5/6: score and hard-violation breakdown update immediately; a swap that creates a duplicate teacher in a panel is rejected or flagged (evaluate_swap IPC exists); save error is understandable.
- **Actual:** No live update; invalid swap applied; cryptic DB-constraint message (assignments PK is (plan,exam,grade,subject,teacher) — role is not part of the key, so the same teacher cannot be setter and reviewer of a panel; the UI never checks first). The failed save also leaves an unhandled promise rejection in the page ({code: duplicate_entry, detail: 'UNIQUE constraint failed: assignments.plan_id, …teacher_id'}), see console-errors.txt.
- **Evidence:** [C5c-live-feedback.json](extracts/C5c-live-feedback.json), [C5b-save-modes.json](extracts/C5b-save-modes.json), [C5b-swap-page.png](screenshots/C5b-swap-page.png), [C5-after-save-page.png](screenshots/C5-after-save-page.png), [console-errors.txt](console-errors.txt)
- **Suspected cause (hypothesis):** QPlanGrid.handleSwapSlots calls onUpdateAssignments without evaluate_swap/evaluate_assignments; AssignmentsPage keeps summary from loaded plan until save.
- **Mock suite passes the same step?** Mock plans are never invalid after swap; mock e2e does not assert header after edit.
- **What Q would notice:** Q drags names around and the score at the top does not move; one swap cannot be saved and shows a database-style message.

<a id="ra-022"></a>
## RA-022 — 'Dữ liệu đã đổi' (stale) badge/banner do not appear after changing data until the window is reloaded

- **Severity:** S2  **Category:** mock-vs-real gap  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C5
- **Reproduction:** Open Phân công (plans listed) > Lịch bận > mark Cô Hiền absent for CK1 > back to Phân công > Lịch sử phương án.
- **Expected:** phase-8 step 11: badge 'Dữ liệu đã đổi' + banner right after returning.
- **Actual:** Backend list_plans reports is_stale=true for all 4 plans (and mark-final is correctly blocked with plan_stale), but the history list shows no badge and no banner after SPA navigation; they appear only after location.reload().
- **Evidence:** [C5h-stale-badge.json](extracts/C5h-stale-badge.json), [C5h-history-after-change-page.png](screenshots/C5h-history-after-change-page.png), [C5h-history-after-reload-page.png](screenshots/C5h-history-after-reload-page.png)
- **Suspected cause (hypothesis):** React Query cache for plans list/plan status is not invalidated by mutations in Unavailability/Teachers/Exams (features/plans hooks).
- **Mock suite passes the same step?** Mock API computes stale synchronously on read, mock e2e passes.
- **What Q would notice:** After editing teachers, the old plan still looks current until the app is restarted; only 'final' marking refuses.

<a id="ra-027"></a>
## RA-027 — Statistics heatmaps are subject-blind and truncated to the first 11 teachers; columns are numbered, T Nghĩa is missing

- **Severity:** S2  **Category:** bug  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C7
- **Reproduction:** Golden Q DB > Thống kê > choose Q's plan > scroll to 'Ma trận tần suất đồng hành' and 'Ma trận phản biện - ra đề'.
- **Expected:** checklist phase-8 step 12 / task C7: matrices for all teachers; counts equal independent counts per 24 subject panels (VL 2+1, CN 1+1) — e.g. Cô Hiền & Cô Lài share 1 panel in Q's plan.
- **Actual:** Per-teacher table is correct (12 rows; totals/Ra đề/Phản biện/exams/q equal DB counts and the Excel 'Thống kê' sheet). Both heatmaps list only 11 teachers (Thầy Nghĩa, the 12th, is absent) with column headers '1..11' (teacher ids) instead of names; values do not match any per-panel recount (e.g. Hiền–Lài shows 2, true 1; 56 of 110 co-working cells and 11 of 110 review cells differ from the per-subject-panel recount). Panel summary says 'Tổng số: 12 ban đề (3 thành viên)' although there are 24 panels (VL 3 members, CN 2).
- **Evidence:** [C7.json](extracts/C7.json), [C7-ui-tables.json](extracts/C7-ui-tables.json), [C7-stats-bottom-page.png](screenshots/C7-stats-bottom-page.png), [C7probe-text.txt](extracts/C7probe-text.txt)
- **Suspected cause (hypothesis):** ui/src/pages/StatisticsPage.tsx: coWorkingMatrix keys panels by `${exam_id}_${grade_id}` (merges VL and CN, double-counts teachers present in both), reviewRelationMatrix uses panelAssign.find(role==='reviewer') (first reviewer only), campusMixStats groups by exam×grade, and the matrices render `teachers.slice(0, 11)` with header {teacher.id}.
- **Mock suite passes the same step?** Mock demo = 11 teachers single subject, so truncation and merge are invisible.
- **What Q would notice:** T Nghĩa (12 duties) is missing from the heatmaps and the pair counts do not match Q's sheet.

<a id="ra-028"></a>
## RA-028 — Excel 'Theo giáo viên' sheet: 'Cùng ban với' lists members of BOTH subjects' panels (wrong in all 60 rows)

- **Severity:** S2  **Category:** bug  **Area:** export  **Engine:** engine-agnostic
- **Scenarios:** D1, C1
- **Reproduction:** Export Q's plan; open sheet 'Theo giáo viên'; compare 'Cùng ban với' with Q's VL/CN panels.
- **Expected:** phase-11 checklist step 7: members of the same subject panel only (e.g. Cô Bình GK1/Khối 12 VL -> T Phúc, C Quí).
- **Actual:** All 60 rows list the union of the VL and CN panels of that exam×grade: Cô Bình GK1/12 VL shows 'Thầy Nghĩa (Ra đề); Thầy Phúc (Ra đề); Cô Quí (Phản biện); Thầy Lộc (Phản biện)'. (Other sheets are correct: grid cells equal Q's 60 seats, totals 60/36/24/15x4, per-teacher totals.)
- **Evidence:** [D1.json](extracts/D1.json), [C1-q-plan-export.json](extracts/C1-q-plan-export.json), [q-plan-task.json](extracts/q-plan-task.json)
- **Suspected cause (hypothesis):** crates/service/src/excel/export.rs builds the co-members list by (exam_id, grade_id) without subject_id. Same pattern as RA-013/RA-027; check print notices (D2).
- **Mock suite passes the same step?** Excel content not asserted in mock suite; golden calamine test uses single-subject/other sheet.
- **What Q would notice:** A teacher's own sheet row lists colleagues from the other subject's panel.

<a id="ra-031"></a>
## RA-031 — Automatic backup rotation and 'newest first' ordering sort by file NAME (reason prefix before timestamp): newest backups are pruned while older ones of another kind are kept; same-second backups overwrite each other

- **Severity:** S2  **Category:** bug  **Area:** storage  **Engine:** engine-agnostic
- **Scenarios:** D3
- **Reproduction:** Real app, data dir with 7 'pre-restore' automatic backups (older) then 9 successive Excel imports (each creates 'pre-import'). Inspect data/backups and Settings > Danh sách bản sao lưu tự động. Also run two restore_database calls within one second.
- **Expected:** phase-9 step 9 / SPEC: keep the 10 NEWEST automatic backups, list newest first; every operation keeps its own safety copy.
- **Actual:** After 16 backups the 10 kept are 7 older pre-restore + only 3 pre-import (the 3 newest imports; the 6 backups taken right before the earlier imports were deleted); the UI list shows pre-restore-…110 on top and the truly newest file (pre-import-…120) last. Two restores in the same second produced a single file (second silently overwrote the first).
- **Evidence:** [D3b-rotation.json](extracts/D3b-rotation.json), [D3.json](extracts/D3.json)
- **Suspected cause (hypothesis):** crates/storage/src/backup.rs: filename = exampanel-backup-{reason}-{epoch_secs}.db; list_automatic_backups sorts by filename descending and prune_automatic_backups keeps the first 10 of that order; no sub-second/uniqueness suffix.
- **Mock suite passes the same step?** Mock backups list is a static array.
- **What Q would notice:** After a few imports and a restore, the backup taken just before the latest import may already be gone.

<a id="ra-033"></a>
## RA-033 — Language chosen on the Settings page is not persisted (reverts to Vietnamese after restart); only the header switcher persists

- **Severity:** S2  **Category:** bug  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** D4
- **Reproduction:** Settings > English (and Dark). Quit and relaunch (same portable folder).
- **Expected:** phase task D4: language and theme persist across restarts.
- **Actual:** Theme persists (dark after restart). Language reverts to Vietnamese. Using the header language dropdown instead persists correctly (English after restart).
- **Evidence:** [D4.json](extracts/D4.json), [D4-after-restart-page.png](screenshots/D4-after-restart-page.png)
- **Suspected cause (hypothesis):** ui/src/pages/SettingsPage.tsx lines ~215/225 call i18n.changeLanguage() directly instead of setLanguage() from ui/src/i18n (which writes localStorage + api.setLanguage). Also the Settings page shows hard-coded Vietnamese help text under the language row.
- **Mock suite passes the same step?** Mock e2e does not restart the app.
- **What Q would notice:** Q switches to English in Settings, restarts, and is back in Vietnamese (or the reverse for an English-speaking colleague).

<a id="ra-034"></a>
## RA-034 — 'Mở thư mục dữ liệu' always fails: 'Command plugin:opener|open_path not allowed by ACL' (capability opener:default lacks open_path); 'Mở thư mục nhật ký' gives no feedback

- **Severity:** S2  **Category:** bug  **Area:** app shell  **Engine:** engine-agnostic (capability file is platform independent)
- **Scenarios:** D4, E4
- **Reproduction:** Settings > Mở thư mục dữ liệu.
- **Expected:** File manager opens the data folder (Linux: xdg-open; no handler is an environment limit and should show a friendly message).
- **Actual:** Toast with the raw English ACL error 'Command plugin:opener|open_path not allowed by ACL'. The Windows build uses the same capability so the button is expected to fail there too (to confirm manually). 'Mở thư mục nhật ký' (custom Rust command open_log_folder) shows no toast at all.
- **Evidence:** [D4.json](extracts/D4.json), [D4-open-data-folder-page.png](screenshots/D4-open-data-folder-page.png), [E4-capability-probes.json](extracts/E4-capability-probes.json)
- **Suspected cause (hypothesis):** src-tauri/capabilities/default.json grants opener:default only; SettingsPage calls openPath from @tauri-apps/plugin-opener which needs opener:allow-open-path (with a scope).
- **Mock suite passes the same step?** Mock opener is a no-op.
- **What Q would notice:** The 'open data folder' button shows an English error.

<a id="ra-036"></a>
## RA-036 — Release build ships the dev-tools feature: the `seed_demo` IPC command is live and writes demo data into the REAL database

- **Severity:** S2  **Category:** packaging/runtime  **Area:** packaging  **Engine:** engine-agnostic
- **Scenarios:** E4, Step0
- **Reproduction:** Build exactly as docs/TECH.md describes (`pnpm tauri build [--no-bundle]`), launch on an empty data folder, run `window.__TAURI_INTERNALS__.invoke('seed_demo')` from the webview (WebDriver executeScript).
- **Expected:** Task/ADR: release contains no dev-tools (no Developer route — correct — and no seed_demo command).
- **Actual:** Command executes: empty exam-panel.db goes from 0 to 12 teachers / 4 campuses (and fails with 'foreign_key_violation' on a populated DB). `strings` of the binary does not contain the 'dev-tools feature not enabled' stub text, i.e. the feature is compiled in. The Developer page itself is correctly absent (import.meta.env.DEV false) and fs/shell/process/opener(open_path, file:) are denied by ACL.
- **Evidence:** [E4.json](extracts/E4.json), [build-inspect.txt](build-inspect.txt), [E4-capability-probes.json](extracts/E4-capability-probes.json)
- **Suspected cause (hypothesis):** src-tauri/Cargo.toml: `[features] default = ["dev-tools"]`; scripts/build-portable.ps1 runs plain `pnpm tauri build` with default features. Fix: remove from default (enable only in `tauri dev`) or build releases with `--no-default-features`; add a CI assertion that seed_demo returns not_supported.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** none visible — hidden command; risk is data pollution if any script injection ever reached the webview.

<a id="ra-002"></a>
## RA-002 — Import preview: 'Đang dạy' column shows 'Mới' (status key) instead of Có/Không

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** B2
- **Reproduction:** Teachers > Nhập từ Excel > choose the filled Q template > Xem trước kết quả > tab Giáo viên.
- **Expected:** Column 'Đang dạy' shows Có/Không (the sheet says 'Có').
- **Actual:** Every active teacher row shows 'Mới' (inactive would show 'Bỏ qua'), duplicating the status badge.
- **Evidence:** [B2-import-preview-screen.png](screenshots/B2-import-preview-screen.png), [B2-preview-text.txt](extracts/B2-preview-text.txt)
- **Suspected cause (hypothesis):** ui/src/pages/teachers/ImportWizardModal.tsx ~line 404: row.active ? t('import.statusNew') : t('import.statusSkipped') — wrong i18n keys.
- **Mock suite passes the same step?** Mock suite passes (no assertion on this column).
- **What Q would notice:** Preview shows 'Mới' under 'Đang dạy' for every teacher, which looks like an error.

<a id="ra-005"></a>
## RA-005 — Existing mock Playwright suite: cannot launch pinned browser in offline CDN sandbox, and a run rewrites 94 tracked docs files

- **Severity:** S3  **Category:** environment  **Area:** docs  **Engine:** engine-agnostic
- **Scenarios:** Step0
- **Reproduction:** pnpm -C ui test:e2e in a sandbox where the Playwright CDN is blocked; then git status after a run with a working browser.
- **Expected:** Suite runs; working tree stays clean.
- **Actual:** (1) Default run: 14/14 fail with 'Executable doesn't exist ... chromium_headless_shell-1243' (sandbox has 1194; CDN 403) — environment. (2) With QA wrapper config using the preinstalled Chromium: 14/14 pass, but the run modifies 94 tracked files (docs/screenshots/**, docs/reports/phase-9/*.pdf) because the specs write their screenshots/PDFs into tracked docs folders.
- **Evidence:** [mock-e2e-attempt1-default-config.txt](logs/mock-e2e-attempt1-default-config.txt), [mock-e2e-baseline.txt](mock-e2e-baseline.txt), [mock-suite-tracked-files-modified.txt](logs/mock-suite-tracked-files-modified.txt), [mock-suite-tracked-files-modified-count.txt](logs/mock-suite-tracked-files-modified-count.txt)
- **Suspected cause (hypothesis):** ui/e2e/*.spec.ts use fixed output paths under docs/. Suggest output dir under test-results/ unless an env flag is set.
- **Mock suite passes the same step?** This is the mock suite itself.
- **What Q would notice:** none

<a id="ra-010"></a>
## RA-010 — Import plan preview shows hard violation as raw code 'h4: max_tasks_per_exam_exceeded' without teacher/exam/context

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C1
- **Reproduction:** Clear C Quí's 'Số việc tối đa mỗi kỳ' override in Teachers, then Assignments > Nhập từ bảng có sẵn > paste Q TSV > Xem trước > tab Cảnh báo & Lỗi.
- **Expected:** One Vietnamese message naming C Quí, CK2 and 3 > 2 (the single hard violation expected from the Q ground truth).
- **Actual:** Count is right (exactly 1) but the text is the raw engine code 'h4: max_tasks_per_exam_exceeded' with no teacher/exam.
- **Evidence:** [C1-issues-no-qui-override.txt](extracts/C1-issues-no-qui-override.txt), [C1-issues-no-qui-override-page.png](screenshots/C1-issues-no-qui-override-page.png)
- **Suspected cause (hypothesis):** ImportPlanModal issues list prints code+raw key, no i18n mapping for violation codes.
- **Mock suite passes the same step?** Mock hard-violation fixtures may use plain text.
- **What Q would notice:** Q sees a code, not who/where.

<a id="ra-012"></a>
## RA-012 — Optimization progress bar and best score go backwards with Chuẩn/Kỹ (progress is per parallel wave, not overall)

- **Severity:** S3  **Category:** bug  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C2
- **Reproduction:** Run Kỹ (16 runs) and sample the dialog every ~200 ms.
- **Expected:** Progress bar monotonic 0->100 %, 'Điểm tốt nhất' never increases, elapsed time monotonic.
- **Actual:** Bar goes 92 % -> 40 % -> 100 % -> 49 % -> 28 % ...; best score shown 176.36 then 180.36; 'Thời gian' resets 851 ms -> 404 -> 200 ms. Samples in extracts. (16 runs execute as waves of 4 on this 4-core box; each progress event is that one run's counters.)
- **Evidence:** [C2-Ky-samples.json](extracts/C2-Ky-samples.json), [C2.json](extracts/C2.json)
- **Suspected cause (hypothesis):** Progress events carry per-run iteration/best/elapsed (anneal.rs progress sink); the dialog divides by one run's iteration budget and shows the latest event. Aggregate across runs or show 'run n/N'. The bar also has no role=progressbar (a11y).
- **Mock suite passes the same step?** Mock emits a single synthetic monotonic stream.
- **What Q would notice:** Progress bar jumps back several times during 'Kỹ'.

<a id="ra-015"></a>
## RA-015 — Q-grid deviations from the paper layout: 'Khối Khối 10', exam names instead of GK1/CK1/GK2/CK2, '—' instead of blank second CN 'Đề' cell

- **Severity:** S3  **Category:** Q-fit  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C4
- **Reproduction:** Open any plan in Bảng tổ.
- **Expected:** Per layout spec: grade groups 'Khối 10..12'; totals columns GK1|CK1|GK2|CK2; CN's second 'Đề' cell blank.
- **Actual:** Group headers 'Khối Khối 10/11/12' (also RA-003); rows and totals columns use 'Giữa kỳ 1'/'Cuối kỳ 1'/... ; CN second Đề cell contains '—'. Matches: header 'Kì thi/khối', VL/CN sub-columns, Đề/Đề/P.Biện rows, totals GV|Tổng lượt|Đề|PB, '2*'/'3**' markers, T Nghĩa 'cố định' tag, Tổng cộng 60/36/24/15x4.
- **Evidence:** [C4.json](extracts/C4.json), [C4-grid-structure.json](extracts/C4-grid-structure.json)
- **Suspected cause (hypothesis):** QPlanGrid uses exam.name not exam.code; placeholder glyph for empty slots.
- **Mock suite passes the same step?** Mock fixture exam names are the same.
- **What Q would notice:** Q's sheet says GK1/CK1; the app says 'Giữa kỳ 1'.

<a id="ra-017"></a>
## RA-017 — With a single placeholder campus, S3 adds a constant 144 penalty and puts a warning icon on every VL seat

- **Severity:** S3  **Category:** Q-fit  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C1, C4
- **Reproduction:** Golden Q DB (one campus 'Chưa phân hiệu'): open any plan.
- **Expected:** A rule that cannot be improved (campus independence with 1 campus) should not dominate totals or add noise (repo comparison.md also notes S3=36 units for nocampus).
- **Actual:** S3 = 36 units / 144.0 penalty in every plan (55 % of Q's 260.36); every VL cell shows a soft-violation triangle; users compare totals that are 144 higher than the meaningful number (116.36 for Q's plan).
- **Evidence:** [C1-ui-detail-view.txt](extracts/C1-ui-detail-view.txt), [C4-grid-1920x1080-dark-page.png](screenshots/C4-grid-1920x1080-dark-page.png), [repo-tool-task5_comparison.md](extracts/repo-tool-task5_comparison.md)
- **Suspected cause (hypothesis):** S3 stays enabled by default when campuses==1; suggest auto-disable or hide when only one campus exists.
- **Mock suite passes the same step?** Mock demo has 2-4 campuses.
- **What Q would notice:** Q sees 260 while the real number to compare is 116, and warning triangles everywhere.

<a id="ra-021"></a>
## RA-021 — Candidate dialog shows raw engine codes as reasons (e.g. 'H4: max_setter_tasks_per_exam_exceeded', 'H2: unqualified_grade') instead of Vietnamese text

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C5, E2
- **Reproduction:** Editable copy > click any seat > look at disabled candidates.
- **Expected:** Reasons in words (phase-8 checklist step 6.4: 'kèm lý do vi phạm ràng buộc cứng').
- **Actual:** Codes such as 'H1: duplicate_teacher_in_panel, H4: max_tasks_per_exam_exceeded, H4: max_setter_tasks_per_exam_exceeded' (also 'h4: max_tasks_per_exam_exceeded' in import preview, RA-010).
- **Evidence:** [C5-candidate-dialog.txt](extracts/C5-candidate-dialog.txt), [C5.json](extracts/C5.json), [C5-candidate-modal-page.png](screenshots/C5-candidate-modal-page.png)
- **Suspected cause (hypothesis):** CandidateSelectModal prints v.rule + v.code with no i18n lookup.
- **Mock suite passes the same step?** Mock returns pre-translated text.
- **What Q would notice:** Reasons look like programmer codes.

<a id="ra-023"></a>
## RA-023 — Marking a teacher unavailable produces an extra spurious H2 'unqualified_grade' hard violation for the same seat

- **Severity:** S3  **Category:** bug  **Area:** core  **Engine:** engine-agnostic
- **Scenarios:** C5
- **Reproduction:** Golden Q plan (0 violations). Lịch bận: mark Cô Hiền absent in CK1 (she holds CK1/Khối 10 VL setter). Evaluate plan.
- **Expected:** exactly one violation: h5 teacher_unavailable (exam 2).
- **Actual:** two: h5:teacher_unavailable AND h2:unqualified_grade for teacher 1 / CK1 / grade 1 / VL, although Cô Hiền teaches Khối 10 (teacher_grades 1,2,3) and has VL competency; baseline and 'C Quí override cleared only' runs show no h2.
- **Evidence:** [C5j-hard-violation-attribution.json](extracts/C5j-hard-violation-attribution.json), [C5i.json](extracts/C5i.json)
- **Suspected cause (hypothesis):** crates/core validate H2 probably uses an availability-aware eligibility helper, so absence is reported twice with a wrong reason.
- **Mock suite passes the same step?** Mock validation is canned.
- **What Q would notice:** Violation count and explanation are inflated; the 'unqualified' reason is untrue.

<a id="ra-024"></a>
## RA-024 — Plan history shows 'Invalid Date' for every plan (WebKitGTK cannot parse SQLite 'YYYY-MM-DD HH:MM:SS')

- **Severity:** S3  **Category:** bug  **Area:** ui  **Engine:** webkitgtk-specific (Chromium/WebView2 parses this format; fix is still cheap: normalise to ISO 8601 'T…Z')
- **Scenarios:** C5
- **Reproduction:** Phân công > Lịch sử phương án.
- **Expected:** Creation date/time.
- **Actual:** '… | Điểm phạt: 176.36 | Invalid Date | Mở phương án'.
- **Evidence:** [C5g-history-before.json](extracts/C5g-history-before.json), [C5g-history-page.png](screenshots/C5g-history-page.png)
- **Suspected cause (hypothesis):** PlansHistoryList.tsx new Date(plan.created_at).toLocaleString() on a datetime('now') string without 'T'/'Z'; Safari/WebKit returns Invalid Date.
- **Mock suite passes the same step?** Mock created_at values are ISO strings; Chromium proxy would not show it.
- **What Q would notice:** On WebView2 probably fine (unverified); on Linux/macOS builds every plan has no date.

<a id="ra-026"></a>
## RA-026 — Re-optimize around kept seats: setter order (Đề 1/Đề 2) of kept seats is not preserved, new plan reuses an existing name, kept-seat banner only exists in 'Chi tiết' view, k/effort not selectable

- **Severity:** S3  **Category:** Q-fit  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C6
- **Reproduction:** Optimizer plan #1 > keep 5 seats via cell menu (GK1/10 VL Đề1 C Như, Đề2 C Na, CK1/11 VL P.Biện, GK2/12 CN P.Biện, CK2/10 VL Đề1) > 'Tối ưu lại phần còn lại'.
- **Expected:** phase-8 step 9: banner 'Đã chọn giữ: N ô', choose number of plans, kept seats unchanged.
- **Actual:** Kept (teacher, panel, role) pairs are preserved 5/5 (good) but only 2/5 are at the same slot position (the 3 setter seats come back in another order); the new plan is again named 'Phương án #1' (two plans with the same name in history); in the default 'Bảng tổ' view only small bookmark icons mark the kept seats (banner 'Đã chọn giữ: 5 ô' exists only in 'Chi tiết'); the dialog offers no k/effort (fixed k=1, 4x50k iterations).
- **Evidence:** [C6.json](extracts/C6.json), [C6-kept-page.png](screenshots/C6-kept-page.png), [C6-kept-detail-page.png](screenshots/C6-kept-detail-page.png)
- **Suspected cause (hypothesis):** reoptimize_from pins by (exam,grade,subject,teacher,role) not position and the annealer re-emits setters in canonical order (same ordering as RA-019); ReoptimizeDialog hard-codes request.
- **Mock suite passes the same step?** Mock keeps positions.
- **What Q would notice:** Q keeps C Như as Đề 1 and gets her back as Đề 2.

<a id="ra-029"></a>
## RA-029 — Export/print fall back to invented organisation text 'TRƯỜNG THPT CHUYÊN / TỔ CHUYÊN MÔN TOÁN / Hà Nội' when Settings org info is empty

- **Severity:** S3  **Category:** Q-fit  **Area:** export  **Engine:** engine-agnostic
- **Scenarios:** D1
- **Reproduction:** Fresh DB without saving 'Thông tin đơn vị' > Xuất Excel; open sheet 1 header.
- **Expected:** Blank or placeholder prompting the user to configure the school; never another school's name.
- **Actual:** Header shows 'TRƯỜNG THPT CHUYÊN' / 'TỔ CHUYÊN MÔN TOÁN', signature 'Hà Nội, ngày ... tháng ... năm 20...' although nothing was configured. The print routes (/print/plan/:id, WebKitGTK screenshot) fall back to different invented text: 'TRƯỜNG THPT CHUYÊN / TỔ TOÁN - TIN', place 'Hà Nội' and signer 'Nguyễn Văn A' — inconsistent with the Excel fallback ('TỔ CHUYÊN MÔN TOÁN') and a fictitious signer name printed on an official sheet.
- **Evidence:** [D1.json](extracts/D1.json), [C1-q-plan-export.json](extracts/C1-q-plan-export.json), [D2-gtk-print-dialog.png](screenshots/D2-gtk-print-dialog.png)
- **Suspected cause (hypothesis):** Hard-coded fallbacks in the exporter/print pages.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** Q's physics/technology group would print 'TỔ CHUYÊN MÔN TOÁN' unless they notice.

<a id="ra-030"></a>
## RA-030 — 'In / Xuất PDF' does nothing in the Linux/WebKitGTK app (no print dialog, no PDF); WebDriver Print Page unsupported

- **Severity:** S3  **Category:** packaging/runtime  **Area:** app shell  **Engine:** webkitgtk-specific (WebView2 on Windows expected to open the print UI — not verified here)
- **Scenarios:** D2
- **Reproduction:** Open /#/print/plan/1 in the Linux release app, click 'In / Xuất PDF' (or press Ctrl+P).
- **Expected:** A print dialog / print-to-file path so the A4 PDF can be produced (phase-9 checklist step 5).
- **Actual:** Nothing happens: no new X window within 3.5 s, page stays responsive. Separately WebKitWebDriver answers 'Unknown command /print' so the W3C Print Page command cannot be used; print output was verified via a Chromium proxy that replays real IPC data (labelled MOCK-DATA-REPLAY).
- **Evidence:** [D2-gtk-print-dialog.png](screenshots/D2-gtk-print-dialog.png), [D2.json](extracts/D2.json), [D2-chromium-replay-run.json](extracts/D2-chromium-replay-run.json)
- **Suspected cause (hypothesis):** wry/WebKitGTK does not wire window.print() to a GtkPrintOperation. Windows is the supported target, so this is informational unless Linux builds are in scope; add an in-app 'Save as PDF' (Tauri-side printing) if Linux/macOS matter.
- **Mock suite passes the same step?** Chromium mock suite prints fine.
- **What Q would notice:** none on Windows (must be re-verified there).

<a id="ra-032"></a>
## RA-032 — Restore dialog shows English engine messages and calls a newer-version file 'corrupted or wrong format'

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** D3
- **Reproduction:** Settings > Phục hồi từ tệp > choose (a) a text file, (b) a valid DB with PRAGMA user_version=99, (c) a truncated DB.
- **Expected:** Clear Vietnamese reasons; newer version: 'cần ExamPanel mới hơn'. All three are correctly rejected (confirm disabled, live DB hash unchanged).
- **Actual:** Heading 'Tệp dữ liệu bị hỏng hoặc không đúng định dạng' for all three plus raw English details: 'Integrity check query failed: file is not a database', 'Unsupported future database version: 99 > supported 5', 'Integrity check query failed: database disk image is malformed'.
- **Evidence:** [D3-restore-dialog-garbage.txt](extracts/D3-restore-dialog-garbage.txt), [D3-restore-dialog-newer_version.txt](extracts/D3-restore-dialog-newer_version.txt), [D3-restore-dialog-truncated.txt](extracts/D3-restore-dialog-truncated.txt), [D3.json](extracts/D3.json)
- **Suspected cause (hypothesis):** BackupValidationSummary.error is a raw Rust string; UI prints it unmapped.
- **Mock suite passes the same step?** Mock returns Vietnamese text.
- **What Q would notice:** English technical sentence in the middle of a Vietnamese dialog.

<a id="ra-037"></a>
## RA-037 — Data-location documentation contradicts the implementation (SPEC §6 / ADR 'writable folder => portable' vs marker-file requirement; v0.1.0 checklist %APPDATA% path)

- **Severity:** S3  **Category:** docs  **Area:** docs  **Engine:** engine-agnostic
- **Scenarios:** A1, A2
- **Reproduction:** Compare docs/SPEC.md §6, docs/DECISIONS.md ADR (writability probe), docs/release/v0.1.0-checklist.md §2 with crates/storage/src/paths.rs and A2.
- **Expected:** One consistent rule.
- **Actual:** SPEC §6: if the exe folder is writable the DB is stored in <exe>/data. Real app (A2, exe folder writable by the user, no marker): data goes to $XDG_DATA_HOME/ExamPanel/data and nothing is created beside the exe — portable mode needs ExamPanel.portable (paths.rs; only mentioned in ADR-0034 and DOC-TOI). v0.1.0 checklist §2 expects installed data in `%APPDATA%\com.exampanel.desktop\`; code uses `%APPDATA%\ExamPanel\data` (APP_DIR_NAME) and tauri identifier is vn.exampanel.app. An undocumented EXAMPANEL_DATA_DIR env override also exists.
- **Evidence:** [A2.json](extracts/A2.json), [A1.json](extracts/A1.json), [docs-grep-dbname.txt](logs/docs-grep-dbname.txt)
- **Suspected cause (hypothesis):** Docs not updated when the marker rule replaced the probe rule.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** Q copies the folder to another PC without the marker file and the data is left behind.

<a id="ra-038"></a>
## RA-038 — Release/QA checklists describe UI and files that no longer exist (stale steps)

- **Severity:** S3  **Category:** docs  **Area:** docs  **Engine:** engine-agnostic
- **Scenarios:** A7, B1, B2, C5, C6, D1
- **Reproduction:** Walk docs/qa/phase-8/9/11/12 checklists and docs/release/v0.1.0-checklist.md in the real app.
- **Expected:** Steps match the product.
- **Actual:** (1) v0.1.0 §3: 'Avoid Pair' feature — does not exist (only soft rule S4); button 'Chốt phương án' is 'Đánh dấu chính thức'. (2) v0.1.0 §4: Excel sheets MA_TRAN/TONG_HOP — real: 'Bảng phân công (mẫu tổ)', 'Phân công', 'Theo giáo viên', 'Thống kê', 'Tiêu chí'. (3) v0.1.0 §6: demo = 18 teachers/2 campuses — real demo = 12 Q-shaped teachers, 4 campuses (B1). (4) phase-9 step 1: template has 4 sheets — real 6 (Hướng dẫn, Phân hiệu, Giáo viên, Lịch vắng, Môn, Môn đảm nhiệm). (5) phase-8 step 4 asks for a copy-name dialog — none. (6) phase-8 step 9 'Đã chọn giữ' banner exists only in Chi tiết view (RA-026). (7) phase-12 K1.6 'Enter on a cell opens candidate dialog' — cells are not keyboard-focusable (see E1). (8) phase-11 step 3.4 sub-line text matches; step 2 cycle order matches.
- **Evidence:** [B1.json](extracts/B1.json), [B2-template-downloaded.xlsx](extracts/B2-template-downloaded.xlsx), [C1-q-plan-export.json](extracts/C1-q-plan-export.json), [C5.json](extracts/C5.json), [E1.json](extracts/E1.json)
- **Suspected cause (hypothesis):** Checklists not maintained with the phases.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** none

<a id="ra-039"></a>
## RA-039 — Keyboard-only use of the plan grid: seats are not focusable and the only tab stops are 60 invisible (opacity 0) menu buttons

- **Severity:** S3  **Category:** accessibility  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** E1, C5
- **Reproduction:** Editable plan copy > Tab through the Bảng tổ grid.
- **Expected:** phase-12 checklist K1.6: 'Enter on a cell opens the candidate dialog'; focus visible.
- **Actual:** Cells (<td data-testid=q-grid-cell-…>) have no tabindex/role; the first stop inside the grid is the per-cell menu button (class opacity-0 group-hover:opacity-100) — 59 of 60 focused buttons have computed opacity 0, so focus is invisible; 60 tab stops precede the totals panel, with no accessible name on the triggers. Once the candidate dialog is opened (mouse) ArrowDown/Enter/Esc work (C5). Other screens: every focused control shows a ring (5 routes x 45 Tabs), Esc closes all 8 dialogs/sheets tested.
- **Evidence:** [E1.json](extracts/E1.json), [E1-tab-sequences.json](extracts/E1-tab-sequences.json), [E1-grid-keyboard-page.png](screenshots/E1-grid-keyboard-page.png)
- **Suspected cause (hypothesis):** QPlanGrid.tsx td has onClick only; menu trigger hidden unless :hover (needs focus-visible:opacity-100, tabindex/role=button on the cell, aria-label).
- **Mock suite passes the same step?** Mock e2e uses mouse clicks.
- **What Q would notice:** none unless Q uses the keyboard.

<a id="ra-040"></a>
## RA-040 — 10 i18n keys are missing from BOTH locale files, so raw keys are shown (common.all, rules.h3Warning, rules.h4MaxTasksPerExam, rules.h4MaxSetterPerExam, rules.s1MaxTasksMode, rules.s1ModeAuto, teachers.advancedSettings, common.close, feasibility.forcedPlacementsTitle/Desc)

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** E2, B3, C5
- **Reproduction:** Open /competencies (subject filter), /rules (H3 warning, H4 labels, S1 mode), Teachers > Thêm giáo viên (advanced section), Teachers > Xem chỉ tiêu (close), feasibility sheet, Phân công > Chi tiết (subject filter 'common.all (2)') — in both vi and en.
- **Expected:** AGENTS.md §3: 100 % i18n, hierarchical keys, both locale files updated.
- **Actual:** The literal keys are printed ('common.all', 'rules.h4MaxTasksPerExam:', 'teachers.advancedSettings', …). Code uses `t('key') || 'Vietnamese fallback'` but i18next returns the key itself when missing, so the fallback never applies. vi.json and en.json both have 777 keys and neither contains these 10.
- **Evidence:** [E2.json](extracts/E2.json), [E2-i18n-findings.json](extracts/E2-i18n-findings.json), [E2-vi-competencies-page.png](screenshots/E2-vi-competencies-page.png), [E2-en-rules-page.png](screenshots/E2-en-rules-page.png)
- **Suspected cause (hypothesis):** Keys referenced in RulesPage.tsx/TeachersPage.tsx/CompetenciesPage.tsx/PlanMatrixView.tsx/FeasibilitySheet were never added to locales; no test asserts key existence. Suggested regression test: static scan of t('…') keys against both locale files in CI.
- **Mock suite passes the same step?** Mock e2e asserts on testids/regex, never on missing-key output.
- **What Q would notice:** Raw words like 'common.all' and 'teachers.advancedSettings' on screen.

<a id="ra-042"></a>
## RA-042 — English UI is incomplete: ~20 hard-coded Vietnamese strings remain on Overview, Competencies, Unavailability, Statistics, Settings, Compare/Import/Quota dialogs and the feasibility sheet; Vietnamese UI shows English 'Close'

- **Severity:** S3  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** E2
- **Reproduction:** Settings > English; visit every route and open the dialogs listed in the extract.
- **Expected:** AGENTS.md §3: never hard-code user-facing strings; every string through t('domain.key') in both locales.
- **Actual:** See curated list: e.g. sidebar subtitle 'Phân công ra đề', 'Năm học 2026 - 2027', '0 lỗi, 12 cảnh báo', 'Thao tác', 'Giảm', whole compare-dialog description/score lines, quota dialog labels, 'Hủy'. Vietnamese mode shows 'Close' (dialog close button). Additionally found by reading the source (not observed on screen in EN): hard-coded 'Trống' (empty seat) in the grid, 'Bạn có chắc chắn muốn xóa phương án này?', 'Bỏ chọn tất cả', toasts built as string concatenations (e.g. 'Download template thành công!' in EN, 'Lỗi tạo tệp mẫu: …', 'Lỗi sao lưu: …').
- **Evidence:** [E2-hardcoded-vietnamese-in-en-curated.txt](extracts/E2-hardcoded-vietnamese-in-en-curated.txt), [E2-i18n-findings.json](extracts/E2-i18n-findings.json), [E2-dom-scan.json](extracts/E2-dom-scan.json), [E2.json](extracts/E2.json)
- **Suspected cause (hypothesis):** Many components still contain Vietnamese literals or `t('x') || 'Vietnamese'` fallbacks (see grep `|| '` in ui/src). Suggested regression test: ESLint rule / grep for JSX text and `toast.*(` literals; run the E2 scan in CI against a Vitest/Playwright mock.
- **Mock suite passes the same step?** Mock e2e runs in Vietnamese only for most steps.
- **What Q would notice:** none (Q is Vietnamese) — blocks any English-speaking colleague.

<a id="ra-043"></a>
## RA-043 — data/logs/app.log is empty in all 112 runs: no startup line, no IPC/DB errors, nothing for the hung startup dialogs — only a deliberate E4 probe line was ever written

- **Severity:** S3  **Category:** packaging/runtime  **Area:** app shell  **Engine:** engine-agnostic
- **Scenarios:** A9, A1, A4, C5
- **Reproduction:** Run any scenario, then read <data>/logs/app.log (tauri-plugin-log folder target + stdout target are configured in src-tauri/src/lib.rs).
- **Expected:** At least startup (version, data dir, DB path/user_version) and error lines (failed save 'duplicate_entry', migration, backup/restore, panic hook) so Q can send a log when something goes wrong; the v0.1.0 checklist and Settings ('Mở thư mục nhật ký') advertise logs.
- **Actual:** collect_logs found 112 app.log files with 94 bytes in total — both bytes are my own `[webview][INFO] qa-probe` line sent through plugin:log|log in E4. No Rust-side `log::info!/error!` call fires in normal use, failed saves produce no entry, and the app never logs its own start.
- **Evidence:** [console-errors.txt](console-errors.txt), [E4-1.log](logs/app-logs/E4-1.log), [E4.json](extracts/E4.json)
- **Suspected cause (hypothesis):** Only the panic hook calls log::error!; services return AppError without logging; no log::info! at startup.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** The 'open logs folder' button leads to an empty file.

<a id="ra-003"></a>
## RA-003 — Grid header reads 'Khối Khối 10' (duplicated word) in Q-style grid

- **Severity:** S4  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** A7, C4
- **Reproduction:** Open /assignments with any plan in 'Bảng tổ' view.
- **Expected:** Header 'Khối 10' (spec: grade groups).
- **Actual:** 'Khối Khối 10', 'Khối Khối 11', 'Khối Khối 12'.
- **Evidence:** [A7-assignments-page.png](screenshots/A7-assignments-page.png)
- **Suspected cause (hypothesis):** Grade label is built as t('…grade') + grade.name where grade.name is already 'Khối 10' (ui/src/pages/assignments/QPlanGrid.tsx).
- **Mock suite passes the same step?** Mock fixtures use the same grade names; mock suite does not assert header text.
- **What Q would notice:** Q sees 'Khối Khối 10' at the top of every grade column.

<a id="ra-004"></a>
## RA-004 — Production dashboard still shows developer 'Kết nối Hệ thống Lõi' ping panel ('pong from ExamPanel core')

- **Severity:** S4  **Category:** Q-fit  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** A1, A7
- **Reproduction:** Launch release build > Tổng quan, scroll down.
- **Expected:** No developer diagnostics on the user dashboard (Developer route is correctly absent).
- **Actual:** Panel 'Kết nối Hệ thống Lõi — Kiểm tra phản hồi IPC giữa giao diện người dùng và lõi Rust' with 'pong from ExamPanel core v0.1.0' and status 'Hệ thống sẵn sàng'.
- **Evidence:** [A7.json](extracts/A7.json), [probe-screen.png](screenshots/probe-screen.png)
- **Suspected cause (hypothesis):** ui/src/pages/OverviewPage.tsx ping widget not gated by import.meta.env.DEV.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** A technical 'Rust core ping' box on the home page.

<a id="ra-006"></a>
## RA-006 — DB file name inconsistent in docs: code/SPEC/ADR/DOC-TOI say exam-panel.db; v0.1.0 checklist and user guide say exampanel.db

- **Severity:** S4  **Category:** docs  **Area:** docs  **Engine:** engine-agnostic
- **Scenarios:** A1, A7
- **Reproduction:** grep -rn 'exampanel.db\|exam-panel.db' docs DOC-TOI.txt crates/storage/src/paths.rs
- **Expected:** One name everywhere.
- **Actual:** Real app creates data/exam-panel.db (A1 data tree). docs/release/v0.1.0-checklist.md lines 19 and 89 and docs/user-guide/vi/08-che-do-dung-thu.md line 33 say exampanel.db; docs/qa/phase-9-checklist.md says backup default name exampanel-backup-YYYYMMDD-HHmm.db (to verify in D3).
- **Evidence:** [A1.json](extracts/A1.json), [docs-grep-dbname.txt](logs/docs-grep-dbname.txt)
- **Suspected cause (hypothesis):** docs drift.
- **Mock suite passes the same step?** n/a (mock/mock.ts also reports exampanel.db).
- **What Q would notice:** A tester following the release checklist looks for a file that does not exist.

<a id="ra-008"></a>
## RA-008 — Every DB open rewrites the database file (sqlite_sequence for 'grades' grows by 3 per open via seed_defaults)

- **Severity:** S4  **Category:** bug  **Area:** storage  **Engine:** engine-agnostic
- **Scenarios:** B1, A5
- **Reproduction:** Copy a fully initialised exam-panel.db, launch app, wait until ready, exit; sha256 before/after. Enter and exit trial mode.
- **Expected:** An already initialised, current-version DB is opened without byte changes (relevant for read-only media, and for the 'real DB hash unchanged' check in trial mode).
- **Actual:** sha256 differs after every launch and again after exit-trial-mode. Logical content identical; the only diff in `.dump` is sqlite_sequence grades 3->6->9 (+3 per open).
- **Evidence:** [B1.json](extracts/B1.json), [B1-dump-diff.txt](logs/B1-dump-diff.txt)
- **Suspected cause (hypothesis):** crates/storage/src/store.rs open_at()/reopen -> seed_defaults(): INSERT OR IGNORE of the 3 default grades bumps the AUTOINCREMENT counter every time. Effects: harmless today (grade ids will jump if a user adds a grade), but byte-level 'unchanged' invariants cannot hold, and every launch writes to the DB (journal file on USB).
- **Mock suite passes the same step?** Not reachable in mock.
- **What Q would notice:** none (internal)

<a id="ra-016"></a>
## RA-016 — S8 chip prints raw float '2.545454545454545' in the plan summary bar

- **Severity:** S4  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C4
- **Reproduction:** Open an optimizer plan; look at the S-chips under 'Tổng điểm phạt'.
- **Expected:** Rounded units (e.g. 2.55).
- **Actual:** 'S8 : 2.545454545454545 (Không thể giảm thêm)'; the detail view/compare dialog round to 1 decimal (36.4), the total to 2 decimals (260.36) so the visible parts do not add up to the total.
- **Evidence:** [C4-grid-1920x1080-dark-page.png](screenshots/C4-grid-1920x1080-dark-page.png), [C1-ui-detail-view.txt](extracts/C1-ui-detail-view.txt)
- **Suspected cause (hypothesis):** No formatting on units in the chip component.
- **Mock suite passes the same step?** Mock units are integers.
- **What Q would notice:** A 15-digit number in the header.

<a id="ra-025"></a>
## RA-025 — Mark-final errors raise two toasts: the translated sentence plus the raw code ('plan_stale' / 'plan_invalid'); hard-violation count differs between toast and header

- **Severity:** S4  **Category:** i18n/text  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** C5
- **Reproduction:** Try 'Đánh dấu chính thức' on a stale plan / a plan with violations.
- **Expected:** One translated toast.
- **Actual:** Toasts: 'Không thể đánh dấu chính thức khi dữ liệu bài toán đã thay đổi' + 'plan_stale'; for the invalid plan 'còn vi phạm điều kiện bắt buộc (1 vi phạm)' + 'plan_invalid' while the header says 'Vi phạm (3 lỗi)' (toast uses params.count default 1 when params lack count).
- **Evidence:** [C5.json](extracts/C5.json), [C5g-invalid-plan-hard-violations.json](extracts/C5g-invalid-plan-hard-violations.json)
- **Suspected cause (hypothesis):** PlansHistoryList.handleMarkFinal reads errorObj.params?.count ?? violations ?? 1 — backend plan_invalid params differ; global error handler also toasts the code.
- **Mock suite passes the same step?** Mock errors carry count.
- **What Q would notice:** Confusing double message with the wrong number.

<a id="ra-035"></a>
## RA-035 — 'Third-party licences' dialog shows a 3-line summary, not the THIRD_PARTY_NOTICES text (and is English-only)

- **Severity:** S4  **Category:** docs  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** D4
- **Reproduction:** Settings > About > Xem chi tiết (licenses-dialog-trigger).
- **Expected:** The notices open (v0.1.0 checklist / D4: 'THIRD_PARTY_NOTICES opens').
- **Actual:** Dialog says the full notices are 'preserved in the bundled THIRD_PARTY_NOTICES document' (44 KB in the repo root) but does not show or link it; in this run the text is English.
- **Evidence:** [D4-licenses-dialog.txt](extracts/D4-licenses-dialog.txt), [D4-licenses-page.png](screenshots/D4-licenses-page.png)
- **Suspected cause (hypothesis):** Static summary component.
- **Mock suite passes the same step?** n/a
- **What Q would notice:** Q cannot read the licences from inside the app.

<a id="ra-041"></a>
## RA-041 — Double-click on 'Lưu' (Exam dialog) saves once but shows two red 'Dữ liệu đã tồn tại, không thể tạo trùng lặp' toasts plus the success toast

- **Severity:** S4  **Category:** bug  **Area:** ui  **Engine:** engine-agnostic
- **Scenarios:** E3
- **Reproduction:** Kỳ thi > Thêm kỳ thi > fill > click Lưu twice quickly (two click events).
- **Expected:** Save button disabled while submitting; one success toast.
- **Actual:** Exactly one exam row is created (good) but toasts: 'Dữ liệu đã tồn tại, không thể tạo trùng lặp' x2 and 'Thêm kỳ thi thành công'.
- **Evidence:** [E3.json](extracts/E3.json)
- **Suspected cause (hypothesis):** Dialog submit not guarded (no isPending disable); duplicates rejected by DB UNIQUE.
- **Mock suite passes the same step?** Mock allows duplicates or is synchronous.
- **What Q would notice:** A harmless double click looks like an error.
