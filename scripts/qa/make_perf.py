#!/usr/bin/env python3
"""perf.txt — informational sandbox numbers, all read from extracts/*.json (A1, C2, E5) and repo tool output."""
import json, os, re, sys
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')); D = open(f'{repo}/.tools/D').read().strip(); OUT = f'{repo}/{D}'; EX = f'{OUT}/extracts'
a1 = json.load(open(f'{EX}/A1.json'))['notes']; c2 = json.load(open(f'{EX}/C2.json'))['notes']
e5 = json.load(open(f'{EX}/E5-perf.json')) if os.path.exists(f'{EX}/E5-perf.json') else {}
L = ['# perf.txt — INFORMATIONAL. Sandbox: 4 vCPU Xeon 2.1 GHz, 16 GB, software GL (LIBGL_ALWAYS_SOFTWARE), Xvfb, WebKitGTK 2.52.6. NOT comparable to a user PC or WebView2.', '']
def st(d): return f"min {d['min']} / median {d['median']} / max {d['max']}"
L += ['## Startup (A1, 5 cold portable runs; source extracts/A1.json)',
      f"- process start -> rendered UI (ms, from /proc start time): {st(a1['ready_ms_from_app_process_start(approx)'])}  all={a1['ready_ms_from_app_process_start(approx)']['all']}",
      f"- tauri-driver launch -> rendered UI (ms, includes WebKitWebDriver spawn): {st(a1['ready_ms_from_driver_launch'])}",
      f"- idle RSS after 5 s, KB (app + WebKit processes + driver): {st(a1['idle_rss_kb_total_after_5s(app+webkit+driver)'])}",
      '- RSS breakdown run 1 (KB): ' + ', '.join(f"{p['comm']}={p['rss_kb']}" for p in a1['rss_procs_run1']), '']
L += ['## Optimizer through the UI (C2; source extracts/C2.json)']
for k in ('run_Nhanh', 'run_Chuẩn', 'run_Kỹ'):
    r = c2[k]; L.append(f"- {k[4:]}: wall {r['wall_ms']} ms; progress samples {r['n_samples']}" + (f"; main-thread max frame gap {r['probe']['maxGap']} ms, p95 {r['probe']['p95']} ms, max timer drift {r['probe']['maxDrift']} ms; IPC round-trip while running {st(r['ipc_roundtrip_ms'])} ms" if r.get('probe') else ''))
m = c2['mem_10_standard_runs']; ws = sorted(m['wall_ms']); L += [f"- 10 consecutive Chuẩn runs: wall ms {m['wall_ms']} (min {ws[0]} / median {ws[len(ws)//2]} / max {ws[-1]}); RSS KB after each run {m['rss_kb']}; growth first->last {m['rss_kb'][-1]-m['rss_kb'][0]} KB"]
t7 = open(f'{EX}/repo-tool-task7_perf.txt', encoding='utf-8').read(); mm = re.search(r'optimize Summary.*?Min:\s+([\d.]+) s\s+- Median:\s+([\d.]+) s\s+- Max:\s+([\d.]+) s', t7, re.S)
if mm: L += [f"- CLI benchmark on the same box (repo tool task7_perf, 8x200k, SyntheticCampuses fixture): min {mm.group(1)} / median {mm.group(2)} / max {mm.group(3)} s  (extracts/repo-tool-task7_perf.txt)"]
L += ['']
if e5:
    L += ['## Files / IPC (E5; source extracts/E5-perf.json)']
    for k, v in e5.items():
        if isinstance(v, dict) and 'median' in v: L.append(f"- {k}: {st(v)} (n={v['n']})")
    for k in ('idle_rss_kb', 'rss_after_kb', 'db_size_bytes', 'export_xlsx_bytes', 'backup_bytes', 'backups_dir_bytes'): L.append(f"- {k}: {e5.get(k)}")
open(f'{OUT}/perf.txt', 'w', encoding='utf-8').write('\n'.join(L) + '\n'); print('\n'.join(L))
