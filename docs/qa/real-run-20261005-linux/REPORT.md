# REPORT — real-app QA run on Linux (WebKitGTK), commit 5a19e0df

**Verdict.** The release binary built cleanly on Linux (252 s, sha256 `96ebbaf71e519dad…`, no missing libraries; no unblock patch was needed) and was really launched and driven through WebDriver (tauri-driver + WebKitWebDriver) as an unprivileged user under Xvfb — 43 findings, **1 × S1, 15 × S2**, no S0 observed. The existing mock suite passes 14/14 and `cargo test` passes (170 tests, exit 0) — yet the real app shows behaviours the mock cannot (cancel saves partial plans, incremental Δ-score computed for the wrong seat, stale badges not shown, startup dialogs deadlock, backup rotation by file name…). Q's ground truth is reproduced exactly where the engine is involved (60 seats, totals, S1/S2/S5/S6/S8/S9, 260.36 / 116.36, the single C Quí CK2 violation without the override) but several screens that present or export it are wrong for the two-subject model (compare dialog, statistics heatmaps, Excel "Theo giáo viên", Excel "Tiêu chí" names).

All counts and quoted numbers are produced by scripts/qa/*.py from the artifact files in this folder (matrix.md, findings.json, extracts/, perf.txt); the narrative (ranking, backlog, hypotheses, questions) is the analyst's judgement and cites finding IDs. Linux/WebKitGTK ≠ Windows/WebView2: see §6.

## 1. Counts

### Findings by severity
| Severity | Count |
|---|---|
| S0 | 0 |
| S1 | 1 |
| S2 | 15 |
| S3 | 19 |
| S4 | 8 |

### Findings by category
| Category | Count |
|---|---|
| Q-fit | 6 |
| accessibility | 1 |
| bug | 14 |
| docs | 4 |
| environment | 1 |
| i18n/text | 9 |
| mock-vs-real gap | 4 |
| packaging/runtime | 3 |
| spec-mismatch | 1 |

### Coverage (from [matrix.md](matrix.md))
| Level | pass | fail | blocked | manual-required | not-verified | not-covered |
|---|---|---|---|---|---|---|
| Scenario rows (28) | 5 | 22 | 1 | 0 | 0 | 0 |
| Existing checklist steps (53) | 9 | 37 | 4 | 3 | 0 | 0 |
| Individual checks (274) | 213 | 47 | 4 | 2 | 8 | 0 |

Scenario rows are "fail" as soon as one of their checks fails, so they overstate failure; the individual-check line is the better measure. Checklist rows inherit the worst result of the scenarios they map to.

## 2. Top 10 findings

| # | ID | Sev | Area | Title | What Q would notice |
|---|---|---|---|---|---|
| 1 | [RA-001](findings.md#ra-001) | S1 | app shell | Startup error dialogs deadlock the main thread on Linux (blank window, no dialog) — read-only portable folder, DB newer than app, corrupted  | On a locked USB stick or after a corrupted file, ExamPanel opens a white/black window and never responds, with no message. |
| 2 | [RA-019](findings.md#ra-019) | S2 | core | Candidate/replace Δ-score is computed for the OTHER setter seat when DB positions differ from the engine's internal setter order (evaluate_c | The suggested replacement says +5 but really costs +18; the best-looking candidate is not the best. |
| 3 | [RA-011](findings.md#ra-011) | S2 | ui | Cancel during optimization still saves partial plans (incl. a 312.36-penalty plan), shows both 'Đã hủy' and 'Đã tạo 3 phương án mới thành cô | Q presses Cancel, gets a 'cancelled' message AND three new plans, one of them clearly worse. |
| 4 | [RA-031](findings.md#ra-031) | S2 | storage | Automatic backup rotation and 'newest first' ordering sort by file NAME (reason prefix before timestamp): newest backups are pruned while ol | After a few imports and a restore, the backup taken just before the latest import may already be gone. |
| 5 | [RA-036](findings.md#ra-036) | S2 | packaging | Release build ships the dev-tools feature: the `seed_demo` IPC command is live and writes demo data into the REAL database | none visible — hidden command; risk is data pollution if any script injection ever reached the webview. |
| 6 | [RA-013](findings.md#ra-013) | S2 | ui | Plan compare dialog is subject-blind: wrong 'Khoảng cách sai khác', cells hide the VL reviewer / merge subjects' setters | Compare screen hides part of Q's VL panels and reports a wrong distance. |
| 7 | [RA-027](findings.md#ra-027) | S2 | ui | Statistics heatmaps are subject-blind and truncated to the first 11 teachers; columns are numbered, T Nghĩa is missing | T Nghĩa (12 duties) is missing from the heatmaps and the pair counts do not match Q's sheet. |
| 8 | [RA-028](findings.md#ra-028) | S2 | export | Excel 'Theo giáo viên' sheet: 'Cùng ban với' lists members of BOTH subjects' panels (wrong in all 60 rows) | A teacher's own sheet row lists colleagues from the other subject's panel. |
| 9 | [RA-009](findings.md#ra-009) | S2 | export | Excel export 'Tiêu chí' sheet labels S3-S8 with wrong criterion names, truncates S8 violation count, and omits lower bounds | Q opens the 'Tiêu chí' sheet and sees S8 described as 'Phân hiệu chính cho môn học' although the school has no campuses; criteria  |
| 10 | [RA-018](findings.md#ra-018) | S2 | ui | 'Tạo bản chỉnh sửa' creates the copy but usually leaves the view on the read-only optimizer plan (switches in 2 of 10 runs) | Q presses 'Tạo bản chỉnh sửa' and nothing seems to happen. |

Ranking rationale: wrong scheduling information shown to Q (RA-019, RA-013, RA-027, RA-028, RA-009), data-safety (RA-031, RA-001), cancel/edit flows that silently misbehave only in the real app (RA-011, RA-018), and shipped dev surface (RA-036).

## 3. What was verified OK (real app)

- Portable/non-portable data placement, single-instance hand-over (second launch exits 0 in 49 ms, no extra process), v4→v5 migration with all rows preserved, trial mode isolation (logical DB content unchanged), template download → fill → import wizard → automatic backup, Q's plan via Excel and via pasted TSV (xclip + Ctrl+V), 60 seats, totals, score breakdown identical in UI / IPC engine / repo report tool (Excel differs only in S8 units rounding), Optimiser Nhanh/Chuẩn/Kỹ (K=3, scores ≥ lower bounds, plans distinct, busy rejection `optimize_busy`, UI main thread stays responsive, 10 runs RSS growth 424 KB), optimizer plans immutable, forced T Nghĩa seats immovable, candidate disabled flags for 12/12 candidates equal to an independent hard-constraint re-check, undo/redo (buttons and Ctrl+Z/Ctrl+Y), save, mark-final blocked for stale and invalid plans, pin/forbid locks, kept seats preserved, per-teacher statistics = DB counts, Excel grid cells = Q's 60 seats, draft marker, A4 landscape + fit-to-width, backup now / restore (file and automatic list) with UI reset, rejection of garbage / newer / truncated files with DB untouched, CSP blocks external fetch/img/script/inline, fs/shell/process/opener(open_path) denied by ACL, Esc closes all dialogs, focus rings on all controls, window minimum 1024×700, size/position remembered.

## 4. Not verified / blocked, and why

| Item | Status | Reason (evidence) |
|---|---|---|
| A4/A5/A6 dialog options (use app-data / exit / restore latest backup) | blocked | the dialogs never render on Linux — RA-001 ([backtrace](logs/A4-hang-main-thread-backtrace.txt)); DB-hash-unchanged for A5 was verified |
| Native drag-and-drop with a real mouse | manual-required | WebKitWebDriver does not synthesise HTML5 DnD; synthetic DragEvent used |
| Real print dialog / PDF from the app | blocked | `window.print()` no-op on WebKitGTK and WebDriver Print Page unsupported (RA-030); print output checked on a Chromium replay proxy (MOCK-DATA-REPLAY, not WebView2) |
| Notices "all teachers" vs "only with duties" filter, Print on final from Windows | not-verified | see matrix rows |
| Real school file with 15–50 teachers | not-verified | no real data allowed; Q-shaped 12-teacher synthetic data used |
| Single-instance focus, DB file descriptors | not-verified | not asserted by the harness |
| WebView2 missing message, SmartScreen, NSIS, portable ZIP on Windows, icon, Windows dialogs, DPI 125/150 %, Microsoft Print to PDF, read-only USB, paths with diacritics in the user profile, Windows font rendering | manual-required | all in [windows-manual-checklist.md](windows-manual-checklist.md) |
| Startup timing vs the 1.5 s target | informational | sandbox: median 1.45 s process-start→rendered UI; not comparable to a user PC ([perf.txt](perf.txt)) |

## 5. Harness integrity notes

- Page hook misses events before injection and across reloads; compensated with app.log / driver logs / raw launch logs ([console-errors.txt](console-errors.txt)).
- Where an expectation failed I checked both sides: C4/D4 first version compared About-commit with git HEAD (HEAD moved because of QA commits) — the correct expectation is the build commit (matches); B3 preset "Cân bằng" first compared H3 too (initial DB had H3 off) — soft weights compared instead, H3 side effect recorded. All other failures are product behaviour with evidence.
- Product code unchanged: [git-diff-product-code.txt](git-diff-product-code.txt) (empty diff); no `.tools/unblock.patch` was needed; processes: [process-cleanup.txt](process-cleanup.txt).

## 6. Linux-vs-Windows limits that apply

- Engine: WebKitGTK 2.52.6 (Safari engine) vs WebView2 (Chromium). Findings are tagged `engine-agnostic | webkitgtk-specific | unknown`; only RA-024 (Invalid Date) and RA-030 (print) are webkitgtk-specific; RA-001 is Linux-confirmed but likely relevant on Windows (same `blocking_show` in `setup()`).
- Native dialogs were GTK (English chrome), not Windows common dialogs; fonts were Noto/DejaVu (environment), so diacritics/cut-off issues in print or grids may differ on Windows.
- No installer, no WebView2 bootstrapper, no Windows ACL/USB behaviour, no DPI scaling, no Defender/SmartScreen.
## 7. Prioritised fix backlog (suggested regression test per item)

Hypotheses are marked (H). "Mock gap" = the existing mock suite passes the same step, so the test must use the real service (tauri-driver smoke or service-level test), or a mock that models latency / real-shaped errors.

### Must fix before v0.2.0
| Area | Items | Regression test |
|---|---|---|
| app shell | **RA-001** startup dialogs hang the event loop on Linux (H: `blocking_show()` inside `setup()`); also confirm on Windows | CI job: launch release exe under Xvfb (a) in a read-only folder, (b) with `user_version=99`, (c) with a corrupted DB; assert a dialog window appears within 5 s and the process exits/continues after the button (xdotool); or move the checks before `Builder::run` and unit-test the decision function |
| app shell / packaging | **RA-036** `default = ["dev-tools"]` ships `seed_demo`; **RA-034** `opener:default` lacks `open_path` (data-folder button always errors; H: same on Windows) | tauri-driver smoke: `invoke('seed_demo')` must return `not_supported`; clicking "Mở thư mục dữ liệu" must not show an ACL error; build script uses `--no-default-features` |
| core | **RA-019** candidate Δ computed for the other setter when DB positions ≠ engine order (H: `IncrementalState::new` canonical ordering); **RA-023** spurious H2 after absence (H: availability-aware eligibility used by H2) | core unit tests with a plan whose setter positions are reversed vs teacher-id order: for every candidate and both positions `delta == full evaluate − base`; validator test "absent teacher ⇒ exactly one h5, no h2" |
| service + ui | **RA-011** Cancel saves partial plans (real backend returns Ok, only the mock throws `cancelled`) | service test: cancel flag ⇒ `Err(cancelled)` and `save_optimize_result` never called; tauri-driver test: Cancel at 0.7 s ⇒ plan count unchanged, single toast |
| storage | **RA-031** rotation/ordering by file name, same-second overwrite; **RA-007** no pre-migration backup (should-fix if Q accepts the written spec) | unit tests: 16 backups of 2 reasons with interleaved timestamps ⇒ kept set == newest 10, list sorted by time desc, two backups in one second both survive; opening `v4_synthetic.db` creates a backup |
| ui (editing) | **RA-020** header/“Hợp lệ” not recomputed after replace/swap, invalid swap accepted, cryptic save error; **RA-018** copy not selected (query-invalidation race); **RA-022** stale badge cache | component tests with an async mock (50–200 ms latency, delayed list refetch); Playwright against the real-shaped fake; assert header equals `evaluate_assignments` after each edit and swap creating H1 is blocked |
| multi-subject views/exports | **RA-013** compare, **RA-027** statistics (also truncated to 11 teachers), **RA-028** Excel "Cùng ban với", **RA-009** Excel "Tiêu chí" names/units | Vitest with the 2-subject Q fixture: distance == #changed seats (39), VL/CN panels shown separately, heatmaps 12×12 with names; calamine test comparing both Excel sheets to the engine for the Q plan |
| ui (settings) | **RA-033** language from Settings not persisted | Vitest: Settings buttons call `setLanguage`; e2e restart test |

### Should fix
- **Q-fit:** RA-014 (grid at 1280×720), RA-015 (GK1/CK1 labels, blank second CN cell), RA-017 (S3 constant 144 for one campus), RA-026, RA-029 (invented org defaults), RA-039 (focusable seats, visible focus).
- **i18n:** RA-040 (10 missing keys — add a CI test that every `t('…')` key exists in both locales), RA-042 (hard-coded Vietnamese / English "Close"), RA-002, RA-010, RA-021, RA-025, RA-032 (map engine codes/messages to translated text).
- **Robustness / data:** RA-012 (aggregate progress), RA-024 (ISO dates — fixes WebKitGTK), RA-041 (disable submit while pending), RA-008 (seed_defaults without sequence bump).
- **Docs:** RA-006, RA-037, RA-038, RA-005 (route test artefacts away from tracked docs).

### Could fix
RA-003, RA-004, RA-016, RA-035, RA-030 (in-app "Save as PDF" if Linux/macOS builds are ever supported).

## 8. Hypotheses to confirm
1. RA-001: `blocking_show()` on the main thread also deadlocks (or not) on Windows — manual W5/W1.
2. RA-019: root cause is the setter ordering in `IncrementalState` (candidates for position 0 equal the full Δ of position 1 in every probed case); confirm by logging `old_t_idx`.
3. RA-034: capability `opener:default` has no `allow-open-path` ⇒ same ACL error on Windows.
4. RA-024: WebView2 parses `YYYY-MM-DD HH:MM:SS`; Linux/macOS do not.
5. RA-023: H2 reuses an availability-aware eligibility helper.
6. RA-018/RA-022: React Query invalidation timing (mock resolves synchronously) — try `await queryClient.invalidateQueries` before `setSelectedPlanId`.
7. Windows print: `window.print()` honours `@page` landscape and background graphics (watermark / grey headers) — manual W9.

## 9. Questions for Q (raised by what the app showed)
1. Total penalties include S3 = 144 for a school without campuses (Q's plan 260.36 vs 116.36 without S3). Should S3 be hidden/disabled when there is a single "Chưa phân hiệu" campus?
2. The paper sheet says GK1/CK1/GK2/CK2 — should the on-screen grid and totals use these codes (the Excel export already does) instead of "Giữa kỳ 1"?
3. Is the order of the two setters ("Đề", "Đề") meaningful on paper? The optimiser/engine seems to treat it as unordered (kept seats come back in another order).
4. Statistics/compare: should a "ban đề" mean the per-subject panel (24 panels, VL 3 members, CN 2) or the merged exam×grade? Should T Nghĩa appear in the heatmaps?
5. Printing: which header text, signer name and place should be pre-filled when Settings is empty (today invented text is printed)?
6. Automatic backups: keep the newest 10 overall, or 10 per kind (import / restore / migration)? Is a backup before a version upgrade expected?
7. "Tránh làm chung" (avoid pair) was in the release checklist — is that feature still wanted?
8. Does the school ever run on Linux/macOS? (Decides whether RA-030 and WebKit-specific items matter.)
9. For the real data: is H3 really off (no campus)? Should C Quí's "3 việc trong một kỳ" exception be kept as a hard override, or should the tool warn instead of showing a violation?
