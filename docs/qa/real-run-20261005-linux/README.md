# Real-app QA run (Linux / WebKitGTK) — how to read and re-run

**What this is.** The real Linux release binary of ExamPanel (same Rust + React code as the Windows target, built with `pnpm tauri build --no-bundle`) driven under Xvfb through WebDriver (tauri-driver 2.1.0 + WebKitWebDriver 2.52.6). It exercises real Tauri IPC, real SQLite, real files, the real Rust solver and the real UI logic. **It is not WebView2 and not Windows** — see [windows-manual-checklist.md](windows-manual-checklist.md). Product code is untouched (`git-diff-product-code.txt`).

Start here: [REPORT.md](REPORT.md) → [findings.md](findings.md) → [matrix.md](matrix.md).

## Folder map
| Path | Content |
|---|---|
| `env.txt`, `setup.log`, `build.txt`, `build-inspect.txt` | environment, every install, build log, exe sha256/ldd/dev-tools probes |
| `mock-e2e-baseline.txt` | existing mock Playwright suite (14/14 pass with a QA wrapper config; default config cannot launch the pinned browser offline) |
| `matrix.md` | scenario + checklist-step coverage with evidence links |
| `findings.md` / `findings.json` | RA-001… |
| `perf.txt`, `console-errors.txt`, `process-cleanup.txt` | informational perf numbers, A9 aggregation, final process audit |
| `extracts/` | machine-written results per scenario (`<ID>.json`), parsed UI text, IPC dumps, xlsx inspections, repo-tool outputs |
| `screenshots/` | `*-page.png` (WebDriver page shot) and `*-screen.png` (full X screen incl. native dialogs) |
| `pdfs/` | Chromium-replay print PDFs (MOCK-DATA-REPLAY) and the mock suite's PDFs |
| `logs/` | per-run page-hook JSON, app.log copies, tauri-driver logs, raw launch logs, gdb backtrace |

## Re-running
Prerequisites (see `setup.log`): Ubuntu 24.04, `scripts/qa/apt-install.sh`, `cargo install tauri-driver --locked --root .tools/cargo-tools`, a Python venv at `.tools/venv` with `openpyxl`, an unprivileged user `qauser`, `cd e2e-real && pnpm install`.
```
scripts/qa/stack.sh start                 # Xvfb :99 1920x1080x24 + openbox
scripts/qa/build-release.sh               # real release build
cd e2e-real && node scenarios/<name>.mjs  # one scenario (fresh tauri-driver + app per run, isolated HOME/XDG in .tools/qa-run/<name>-<n>/)
scripts/qa/run-seq.sh <logdir> A1-A3.mjs ...   # batch
python3 scripts/qa/collect_logs.py && python3 scripts/qa/build_report.py   # aggregate
scripts/qa/stack.sh stop
```
Golden databases are produced by the scenarios themselves through the real UI: `B2a/B2b` (template → fill → import wizard), `mkgolden` (H3 off through Rules), `mkgolden2` (Q's plan imported + Chuẩn run), stored under `.tools/golden/` (not committed).

## Harness notes / limits (Reporting Integrity)
- Page hook (console/error/rejection/fetch/XHR/CSP) is injected after each page load; events before the hook and across reloads are missed — compensated by app.log, tauri-driver and raw launch logs.
- No `ui-ready` marker exists in the product (task assumption): readiness = `#root` has rendered text; timings are process-start→rendered-DOM measured from `/proc` start time.
- Native GTK dialogs are driven with xdotool (Ctrl+L, Ctrl+A, path, Return); results are verified by the resulting file/DB.
- WebKitWebDriver cannot synthesise HTML5 drag-and-drop; the drag-swap step uses synthetic DragEvents (labelled in `extracts/C5.json`), real-mouse DnD is `manual-required`.
- `window.print()` is a no-op in WebKitGTK and the WebDriver Print Page command is unsupported: print output was checked on a **Chromium replay proxy** (real `ui/dist` bundle, real Rust IPC results replayed) — labelled MOCK-DATA-REPLAY, not WebView2.
- `seed_demo`, `evaluate_candidates`, … were also called directly through `window.__TAURI_INTERNALS__.invoke` for independent re-checks; those calls are labelled "IPC" in the matrix.

## Re-runs and superseded results (honesty log)
Scenario scripts were iterated while the harness was being developed; `extracts/<ID>.json` keeps only the latest result per check name. Checks that were **removed because a later, corrected check replaces them** (not because they failed): `B1.real-db-hash-after-exit` (replaced by `…bytes-after-exit` + `…logical-content-after-exit`, after finding that the first byte comparison was taken before the app had opened the DB), `B3.rules-h3-off` (script bug: wrong column name), `B3.rules-balanced-restores-defaults` (initial DB had H3 off — replaced by the soft-weights comparison + a note about the H3 side effect), `B3.assignments-subject-filter` (regex counted block titles that the filtered view omits — replaced by `B3.assignments-subject-filter-narrows-matrix`), `D4.about-commit-equals-HEAD` (HEAD moved because of the QA commits — replaced by the comparison with the build commit), `C6.banner-shows-kept-count` (split into grid/detail checks), and the first-draft `A4.ok.*` / `A4.cancel.*` checks (written before the deadlock was understood). The first C2 run's absolute RSS numbers were inflated by a stray app left by a failed scenario; the harness now kills strays and C2 was re-run.
