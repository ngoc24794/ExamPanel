#!/usr/bin/env python3
"""Copy per-run logs into the report folder and aggregate console errors / external requests / CSP (A9) into console-errors.txt."""
import os, sys, json, glob, shutil, re, collections
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')); D = open(f'{repo}/.tools/D').read().strip(); OUT = f'{repo}/{D}'
runs = sorted(glob.glob(f'{repo}/.tools/qa-run/*'))
os.makedirs(f'{OUT}/logs/hooks', exist_ok=True); os.makedirs(f'{OUT}/logs/app-logs', exist_ok=True); os.makedirs(f'{OUT}/logs/driver-logs', exist_ok=True); os.makedirs(f'{OUT}/logs/raw', exist_ok=True)
agg = collections.defaultdict(lambda: dict(count=0, runs=set())); ext = collections.defaultdict(set); csp = collections.defaultdict(set); applog = collections.defaultdict(lambda: dict(count=0, runs=set())); nrun = 0
for r in runs:
    name = os.path.basename(r); nrun += 1
    h = f'{r}/out/hook.json'
    if os.path.exists(h):
        shutil.copy(h, f'{OUT}/logs/hooks/{name}.json')
        try: data = json.load(open(h))
        except Exception: data = []
        for e in data:
            if not isinstance(e, dict): continue
            for c in e.get('console', []): agg[f"console.{c['l']}: {c['m'][:220]}"]['count'] += 1; agg[f"console.{c['l']}: {c['m'][:220]}"]['runs'].add(name)
            for x in e.get('errors', []): agg[f"window.error: {x['m'][:200]}"]['count'] += 1; agg[f"window.error: {x['m'][:200]}"]['runs'].add(name)
            for x in e.get('rejections', []): agg[f"unhandledrejection: {x[:200]}"]['count'] += 1; agg[f"unhandledrejection: {x[:200]}"]['runs'].add(name)
            for x in e.get('csp', []): csp[f"{x['d']} blocked {x['b']}"].add(name)
            for x in e.get('requests', []):
                if not re.match(r'^(ipc://localhost/|http://ipc\.localhost/|tauri://|/|blob:|data:)', x['u']): ext[x['u']].add(name)
    for pat, dst in (('app/data/logs/app.log', 'app-logs'), ('home/.local/share/ExamPanel/data/logs/app.log', 'app-logs')):
        p = f'{r}/{pat}'
        if os.path.exists(p):
            shutil.copy(p, f'{OUT}/logs/{dst}/{name}.log')
            for line in open(p, errors='replace'):
                if re.search(r'\b(ERROR|WARN|PANIC|panicked)\b', line): k = re.sub(r'^\[[^\]]*\]\[[^\]]*\]', '', line).strip()[:200]; applog[k]['count'] += 1; applog[k]['runs'].add(name)
    for p in glob.glob(f'{r}/out/tauri-driver.log'):
        txt = open(p, errors='replace').read()
        if txt.strip(): open(f'{OUT}/logs/driver-logs/{name}.log', 'w').write(txt[-20000:])
    for p in glob.glob(f'{r}/out/raw-*.log') + glob.glob(f'{r}/out/second-launch.txt'):
        shutil.copy(p, f'{OUT}/logs/raw/{name}-{os.path.basename(p)}')
lines = [f'# A9 — aggregated console errors/warnings, external requests and CSP violations over {nrun} app runs', '', 'Source: logs/hooks/*.json (page hook injected after each page load; events before the hook and across reloads are NOT captured), logs/app-logs/*.log (tauri-plugin-log file), logs/driver-logs, logs/raw.', '']
lines += ['## Page console errors / warnings / window errors / unhandled rejections (unique, with counts)']
if not agg: lines.append('(none captured)')
for k, v in sorted(agg.items(), key=lambda kv: -kv[1]['count']): lines.append(f"- x{v['count']} in {len(v['runs'])} run(s): {k}")
lines += ['', '## Requests to non-local hosts (fetch/XHR/WebSocket URLs seen by the hook)']
if not ext: lines.append('(none — every captured request was ipc://localhost/...)')
for k, v in ext.items(): lines.append(f"- {k}  (runs: {', '.join(sorted(v))}) — deliberate probe in E4" if 'E4' in ' '.join(v) else f"- {k}  (runs: {', '.join(sorted(v))})")
lines += ['', '## securitypolicyviolation events']
if not csp: lines.append('(none)')
for k, v in csp.items(): lines.append(f"- {k} (runs: {', '.join(sorted(v))})")
lines += ['', '## Rust-side log lines containing ERROR / WARN / panic (app.log)']
if not applog: lines.append('(none)')
for k, v in sorted(applog.items(), key=lambda kv: -kv[1]['count']): lines.append(f"- x{v['count']} in {len(v['runs'])} run(s): {k}")
open(f'{OUT}/console-errors.txt', 'w').write('\n'.join(lines) + '\n'); print('\n'.join(lines)[:3000])
