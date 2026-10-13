# Phase 12.2 — "Nhập file mẫu xong không tìm thấy phương án tối ưu"

- **Reported symptom:** after importing the workbook downloaded from **Tải file mẫu**, the optimizer
  answered *"Không tìm thấy phương án nào thỏa mãn sau khi tìm kiếm"* while the feasibility indicator
  stayed green.
- **Environment:** real desktop app data directory `%APPDATA%\ExamPanel\data` (Windows), database
  snapshot copied on 2026-10-13 (`exam-panel.db`, 180 224 bytes).
- **Artifacts (all produced by the commands below, never retyped):**

| File | Produced by |
| --- | --- |
| [`real-data-diagnosis.txt`](real-data-diagnosis.txt) | `cargo run -q -p exam-panel-service --example real_data_diagnosis` against a copy of the reporter's database |
| [`reimport-fixed-template.txt`](reimport-fixed-template.txt) | importing the corrected template over the reporter's database (`AppService::preview_import` + `apply_import` + `solve_hard`) |
| [`template-checks-before-data-fix.txt`](template-checks-before-data-fix.txt) | `cargo test -p exam-panel-service --test excel_tests test_standard_template_feasibility` (new checks added, sample data not yet fixed) |
| [`core-feasibility-tests.txt`](core-feasibility-tests.txt) | `cargo test -p exam-panel-core feasibility` |
| [`excel-template-tests.txt`](excel-template-tests.txt) | `cargo test -p exam-panel-service --test excel_tests` |
| [`check-all.txt`](check-all.txt) | `pnpm check-all` (exit 0) |

## Root cause of the user-visible failure

The imported snapshot holds 18 teachers, 4 campuses, 2 subjects and 24 panels (60 seats):
VL = 12 panels × 3 = 36 seats, CN = 12 panels × 2 = 24 seats. Two independent necessary conditions
were missing from the pre-solve checks, so the green indicator lied and the solver could only stop
with the opaque `solver_exhausted`:

1. **Per-subject capacity** — the CN subject is taught by GV016–GV018 only, whose max quotas sum to
   `4 + 4 + 12 = 20`, below the 24 CN seats (`real-data-diagnosis.txt` lines 7 and 13).
2. **Campus rules force one teacher over the H4 per-exam cap** — CN teachers sit in PH1 (GV017) and
   PH2 (GV016, GV018), so with `min_campuses = 2` every CN panel needs GV017: 3 panels per exam
   against the H4 limit of 2 (lines 8–11). The existing per-exam max-flow cannot see this because it
   ignores campuses.

Both checks were absent, so `check_feasibility` returned `is_feasible = true, errors = 0` while
`solve_hard` exhausted after 3 nodes / 12 backtracks with all 24 panels unfilled.

## Changes

- `crates/core/src/feasibility/mod.rs`
  - F4 **`insufficient_subject_capacity`** (error): for every subject, the summed `hi_t` of its
    competent teachers must cover the subject's seats.
  - F4 **`h3_forced_over_exam_cap`** (error): per teacher and exam, the number of panels the teacher
    cannot be left out of (because removing them leaves fewer campuses, fewer teachers than seats, or
    too few role-eligible teachers) is compared against the same effective H4 limit the solver uses;
    only evaluated when H4 and H3 are enabled and the panel already passed per-panel adequacy.
  - Two unit tests: `test_diagnostic_insufficient_subject_capacity`,
    `test_diagnostic_h3_forced_over_exam_cap` (the latter also asserts `solve_hard` fails on the
    original instance and succeeds once H3 is relaxed).
- `crates/service/src/excel/template.rs` — the bundled sample is now schedulable: CN row uses
  `min_campuses = 1` and GV016–GV018 carry explicit quotas `8` (8 × 3 = the 24 CN seats). The guide
  sheet documents both traps (minimum campuses per panel, explicit quotas must cover a subject's
  seats, blank quota keeps the stored value).
- `crates/service/src/excel/import.rs` — a third defect blocked the recovery path: the change
  detection ignored `"Cách gọi"`, `"Chỉ tiêu riêng"` and `"Số việc tối đa mỗi kỳ"`, so editing only
  those columns and importing again reported every teacher row as `Unchanged` and wrote nothing
  (the subject change to `min_campuses` did apply, which hid part of the problem). The optional
  columns now follow the `COALESCE` semantics of `apply_import`: a filled cell must match, an empty
  cell keeps the stored value.
- `crates/service/tests/excel_tests.rs` — `test_standard_template_feasibility_and_generate_downloads`
  now applies the preview, runs `solve_hard`, asserts 60/60 assignments and zero violations, so an
  unschedulable sample can no longer ship; `test_reimport_detects_quota_and_max_task_changes` covers
  the optional-column update and idempotency.
- `ui/src/i18n/locales/{vi,en}.json` + `ui/src/i18n/i18n.test.ts` — messages for both new
  diagnostics; `errors.problem_infeasible` now reports the error count and points at the feasibility
  indicator.
- `docs/SPEC.md` §4.1/§4.2/§10.2, `docs/DECISIONS.md` ADR-0052 (the two new checks), ADR-0053
  (bundled templates must be schedulable and are guarded by a test) and ADR-0054 (re-import change
  detection).

## Recovery paths for the affected school

**Simplest (no manual editing):** download the template again (**Tải file mẫu**) and import it over the
existing data. `reimport-fixed-template.txt` shows the result against the reporter's own database:
3 teacher rows and 1 subject row updated, feasibility clean, `solve_hard: OK assignments=60 nodes=25`.

**Manual editing of an existing school file:** both variants below were run against the reporter's
database (`real-data-diagnosis.txt`):

| Fix | Result |
| --- | --- |
| **A** — sheet *Môn*: CN `Số phân hiệu tối thiểu = 1`; sheet *Giáo viên*: `Chỉ tiêu riêng` GV016 = GV017 = GV018 = `8` | `is_feasible=true`, `solve_hard: OK assignments=60 nodes=25 violations=0` |
| **B** — keep CN at 2 campuses, add one CN teacher at PH3, `Chỉ tiêu riêng` 6 for each of the four CN teachers (6 × 4 = 24) | `is_feasible=true`, `solve_hard: OK assignments=60 nodes=25 violations=0` |

Adding CN teachers *without* explicit quotas is not enough: capacity rises above 24 but the pattern
still fails (reproduced separately: three extra CN teachers exhausted the 50 000-node budget, one
extra teacher exhausted the search space).

## Verification

`pnpm check-all` exits 0 (`check-all.txt`): version/ADR checks, `cargo fmt --check`,
`cargo clippy -- -D warnings`, `cargo test`, the typegen test, `pnpm -C ui lint`, `typecheck`,
147 UI tests in 34 files, and the production UI build.
