#!/usr/bin/env python3
"""Three/four-way comparison of Q's plan score: UI text vs IPC engine vs Excel 'Tiêu chí' vs repo report tool. usage: threeway.py <extracts_dir>"""
import json, re, sys, os
d = sys.argv[1]
ui = {}
for m in re.finditer(r'\nS(\d+)\n([\d.]+)', open(f'{d}/C1-ui-detail-view.txt', encoding='utf-8').read()):
    ui.setdefault('S' + m.group(1), float(m.group(2)))
ev = json.load(open(f'{d}/C1-ipc-evaluate_assignments.json'))['v']['score_report']
eng = {r['rule'].upper(): (r['units'], r['penalty']) for r in ev['by_rule']}
xl = json.load(open(f'{d}/C1-q-plan-export.json'))
tc = next(s for s in xl['sheets'] if s['title'] == 'Tiêu chí')['rows'][1:]
excel = {r[0]: (r[4], r[5]) for r in tc if r[0].startswith('S')}
repo = {}
for line in open(f'{d}/repo-tool-task5_comparison.md', encoding='utf-8'):
    m = re.match(r'\| (S\d+) .*?\|\s*([\d.]+) \|\s*([\d.]+) \|\s*([\d.]+) \|\s*([\d.]+) \|\s*([\d.]+) \|', line)
    if m: repo[m.group(1)] = (float(m.group(4)), float(m.group(5)))
rows = []; mism = []
for k in [f'S{i}' for i in range(1, 11)]:
    r = dict(rule=k, ui_pen=ui.get(k), engine_units=eng[k][0], engine_pen=eng[k][1], excel_viol=excel.get(k, (None, None))[0], excel_pen=excel.get(k, (None, None))[1], repo_units=repo.get(k, (None, None))[0], repo_pen=repo.get(k, (None, None))[1])
    rows.append(r)
    pens = [x for x in (r['ui_pen'], r['engine_pen'], r['excel_pen'], r['repo_pen']) if x is not None]
    if max(pens) - min(pens) > 0.06: mism.append((k, 'penalty', pens))
    us = [x for x in (r['engine_units'], r['excel_viol'], r['repo_units']) if x is not None]
    if max(us) - min(us) > 0.011: mism.append((k, 'units', us))
tot = dict(engine=ev['total'], ui=float(re.search(r'Tổng điểm phạt: ([\d.]+)', open(f'{d}/C1-ui-detail-view.txt', encoding='utf-8').read()).group(1)))
out = dict(rows=rows, totals=tot, mismatches=mism)
json.dump(out, open(f'{d}/C1-threeway.json', 'w'), indent=1, ensure_ascii=False)
print(json.dumps(tot)); [print(r) for r in rows]; print('MISMATCHES', mism)
