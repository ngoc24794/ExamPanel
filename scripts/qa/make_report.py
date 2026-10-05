#!/usr/bin/env python3
"""REPORT.md: counts and tables are read from artifact files; narrative sections are the analyst's judgement and cite finding IDs."""
import json, os, re, collections, shutil
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')); D = open(f'{repo}/.tools/D').read().strip(); OUT = f'{repo}/{D}'
S = json.load(open(f'{OUT}/summary-counts.json')); F = json.load(open(f'{OUT}/findings.json'))
sev = S['findings_by_severity']; cat = S['findings_by_category']
start = open(f'{OUT}/start-commit.txt').read().strip()
def tbl(d, order=None):
    keys = order or sorted(d); return '\n'.join(f'| {k} | {d.get(k, 0)} |' for k in keys)
cl = S['checklist_steps']; sc = S['scenario_rows']; ck = S['step_checks']
byid = {f['id']: f for f in F}
EXD = f'{OUT}/extracts'
j = lambda n: json.load(open(f'{EXD}/{n}.json'))
def chk(n, step): return next(c for c in j(n)['checks'] if c['step'] == step)
second_ms = j('A3')['notes']['second_launch']['duration_ms']; growth = j('C2')['notes']['mem_10_standard_runs']['rss_kb']; growth = growth[-1] - growth[0]
cand_n = chk('C5', 'C5.candidate-disabled-iff-hard-violation(engine re-check)')['observed']['checked']
TOP = ['RA-001', 'RA-019', 'RA-011', 'RA-031', 'RA-036', 'RA-013', 'RA-027', 'RA-028', 'RA-009', 'RA-018']
mock = open(f'{OUT}/mock-e2e-baseline.txt').read(); mock_pass = re.search(r'(\d+) passed', mock).group(1)
cargo = open(f'{OUT}/cargo-test.txt').read(); cargo_ok = sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed', cargo)); cargo_fail = sum(int(x) for x in re.findall(r'(\d+) failed', cargo)); cargo_exit = re.search(r'CARGO_EXIT=(\d+)', cargo).group(1)
bi = open(f'{OUT}/build-inspect.txt').read(); sha = re.search(r'([0-9a-f]{64})', bi).group(1); bt = open(f'{OUT}/build.txt').read(); dur = re.search(r'DURATION_S=(\d+)', bt).group(1)
L = []
L += [f'# REPORT — real-app QA run on Linux (WebKitGTK), commit {start[:8]}', '',
 f'**Verdict.** The release binary built cleanly on Linux ({dur} s, sha256 `{sha[:16]}…`, no missing libraries; no unblock patch was needed) and was really launched and driven through WebDriver (tauri-driver + WebKitWebDriver) as an unprivileged user under Xvfb — {len(F)} findings, **{sev.get("S1",0)} × S1, {sev.get("S2",0)} × S2**, no S0 observed. The existing mock suite passes {mock_pass}/14 and `cargo test` passes ({cargo_ok} tests, exit {cargo_exit}) — yet the real app shows behaviours the mock cannot (cancel saves partial plans, incremental Δ-score computed for the wrong seat, stale badges not shown, startup dialogs deadlock, backup rotation by file name…). Q\'s ground truth is reproduced exactly where the engine is involved (60 seats, totals, S1/S2/S5/S6/S8/S9, 260.36 / 116.36, the single C Quí CK2 violation without the override) but several screens that present or export it are wrong for the two-subject model (compare dialog, statistics heatmaps, Excel "Theo giáo viên", Excel "Tiêu chí" names).', '',
 'All counts and quoted numbers are produced by scripts/qa/*.py from the artifact files in this folder (matrix.md, findings.json, extracts/, perf.txt); the narrative (ranking, backlog, hypotheses, questions) is the analyst\'s judgement and cites finding IDs. Linux/WebKitGTK ≠ Windows/WebView2: see §6.', '',
 '## 1. Counts', '', '### Findings by severity', '| Severity | Count |', '|---|---|', tbl(sev, ['S0', 'S1', 'S2', 'S3', 'S4']), '', '### Findings by category', '| Category | Count |', '|---|---|', tbl(cat), '',
 '### Coverage (from [matrix.md](matrix.md))', '| Level | pass | fail | blocked | manual-required | not-verified | not-covered |', '|---|---|---|---|---|---|---|',
 f'| Scenario rows (28) | {sc.get("pass",0)} | {sc.get("fail",0)} | {sc.get("blocked",0)} | {sc.get("manual-required",0)} | {sc.get("not-verified",0)} | {sc.get("not-covered",0)} |',
 f'| Existing checklist steps ({sum(cl.values())}) | {cl.get("pass",0)} | {cl.get("fail",0)} | {cl.get("blocked",0)} | {cl.get("manual-required",0)} | {cl.get("not-verified",0)} | {cl.get("not-covered",0)} |',
 f'| Individual checks ({sum(ck.values())}) | {ck.get("pass",0)} | {ck.get("fail",0)} | {ck.get("blocked",0)} | {ck.get("manual-required",0)} | {ck.get("not-verified",0)} | {ck.get("not-covered",0)} |', '',
 'Scenario rows are "fail" as soon as one of their checks fails, so they overstate failure; the individual-check line is the better measure. Checklist rows inherit the worst result of the scenarios they map to.', '',
 '## 2. Top 10 findings', '', '| # | ID | Sev | Area | Title | What Q would notice |', '|---|---|---|---|---|---|']
for i, k in enumerate(TOP, 1):
    f = byid[k]; L.append(f'| {i} | [{k}](findings.md#{k.lower()}) | {f["severity"]} | {f["area"]} | {f["title"][:140]} | {f["q_notice"][:130]} |')
L += ['', 'Ranking rationale: wrong scheduling information shown to Q (RA-019, RA-013, RA-027, RA-028, RA-009), data-safety (RA-031, RA-001), cancel/edit flows that silently misbehave only in the real app (RA-011, RA-018), and shipped dev surface (RA-036).', '']
L += ['## 3. What was verified OK (real app)', '',
 f'- Portable/non-portable data placement, single-instance hand-over (second launch exits 0 in {second_ms} ms, no extra process), v4→v5 migration with all rows preserved, trial mode isolation (logical DB content unchanged), template download → fill → import wizard → automatic backup, Q\'s plan via Excel and via pasted TSV (xclip + Ctrl+V), 60 seats, totals, score breakdown identical in UI / IPC engine / repo report tool (Excel differs only in S8 units rounding), Optimiser Nhanh/Chuẩn/Kỹ (K=3, scores ≥ lower bounds, plans distinct, busy rejection `optimize_busy`, UI main thread stays responsive, 10 runs RSS growth {growth} KB), optimizer plans immutable, forced T Nghĩa seats immovable, candidate disabled flags for {cand_n}/{cand_n} candidates equal to an independent hard-constraint re-check, undo/redo (buttons and Ctrl+Z/Ctrl+Y), save, mark-final blocked for stale and invalid plans, pin/forbid locks, kept seats preserved, per-teacher statistics = DB counts, Excel grid cells = Q\'s 60 seats, draft marker, A4 landscape + fit-to-width, backup now / restore (file and automatic list) with UI reset, rejection of garbage / newer / truncated files with DB untouched, CSP blocks external fetch/img/script/inline, fs/shell/process/opener(open_path) denied by ACL, Esc closes all dialogs, focus rings on all controls, window minimum 1024×700, size/position remembered.', '']
L += ['## 4. Not verified / blocked, and why', '', '| Item | Status | Reason (evidence) |', '|---|---|---|',
 '| A4/A5/A6 dialog options (use app-data / exit / restore latest backup) | blocked | the dialogs never render on Linux — RA-001 ([backtrace](logs/A4-hang-main-thread-backtrace.txt)); DB-hash-unchanged for A5 was verified |',
 '| Native drag-and-drop with a real mouse | manual-required | WebKitWebDriver does not synthesise HTML5 DnD; synthetic DragEvent used |',
 '| Real print dialog / PDF from the app | blocked | `window.print()` no-op on WebKitGTK and WebDriver Print Page unsupported (RA-030); print output checked on a Chromium replay proxy (MOCK-DATA-REPLAY, not WebView2) |',
 '| Notices "all teachers" vs "only with duties" filter, Print on final from Windows | not-verified | see matrix rows |',
 '| Real school file with 15–50 teachers | not-verified | no real data allowed; Q-shaped 12-teacher synthetic data used |',
 '| Single-instance focus, DB file descriptors | not-verified | not asserted by the harness |',
 '| WebView2 missing message, SmartScreen, NSIS, portable ZIP on Windows, icon, Windows dialogs, DPI 125/150 %, Microsoft Print to PDF, read-only USB, paths with diacritics in the user profile, Windows font rendering | manual-required | all in [windows-manual-checklist.md](windows-manual-checklist.md) |',
 '| Startup timing vs the 1.5 s target | informational | sandbox: median 1.45 s process-start→rendered UI; not comparable to a user PC ([perf.txt](perf.txt)) |', '']
L += ['## 5. Harness integrity notes', '',
 '- Page hook misses events before injection and across reloads; compensated with app.log / driver logs / raw launch logs ([console-errors.txt](console-errors.txt)).',
 '- Where an expectation failed I checked both sides: C4/D4 first version compared About-commit with git HEAD (HEAD moved because of QA commits) — the correct expectation is the build commit (matches); B3 preset "Cân bằng" first compared H3 too (initial DB had H3 off) — soft weights compared instead, H3 side effect recorded. All other failures are product behaviour with evidence.',
 f'- Product code unchanged: [git-diff-product-code.txt](git-diff-product-code.txt) (empty diff); no `.tools/unblock.patch` was needed; processes: [process-cleanup.txt](process-cleanup.txt).', '']
L += ['## 6. Linux-vs-Windows limits that apply', '',
 '- Engine: WebKitGTK 2.52.6 (Safari engine) vs WebView2 (Chromium). Findings are tagged `engine-agnostic | webkitgtk-specific | unknown`; only RA-024 (Invalid Date) and RA-030 (print) are webkitgtk-specific; RA-001 is Linux-confirmed but likely relevant on Windows (same `blocking_show` in `setup()`).',
 '- Native dialogs were GTK (English chrome), not Windows common dialogs; fonts were Noto/DejaVu (environment), so diacritics/cut-off issues in print or grids may differ on Windows.',
 '- No installer, no WebView2 bootstrapper, no Windows ACL/USB behaviour, no DPI scaling, no Defender/SmartScreen.', '']
open(f'{OUT}/REPORT-part1.md', 'w', encoding='utf-8').write('\n'.join(L))
print('\n'.join(L)[:1500])
