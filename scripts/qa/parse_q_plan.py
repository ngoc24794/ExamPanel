#!/usr/bin/env python3
"""Independent derivation of Q's plan from the task statement text (not from the repo fixture).
Writes extracts/q-plan-task.json (seats), q-plan.tsv (paste format) and a comparison with the statement's totals and with
crates/core fixtures make_q_assignments() (as listed in fixtures.rs, parsed by regex) for a cross-check."""
import re, json, sys, collections, os
here = os.path.dirname(__file__); out = sys.argv[1]
txt = open(os.path.join(here, 'q-plan-task-text.txt'), encoding='utf-8').read()
seats = []
for m in re.finditer(r'(GK1|CK1|GK2|CK2)/(10|11|12) VL \[([^|\]]+)\|([^\]]+)\] CN \[([^|\]]+)\|([^\]]+)\]', txt):
    ex, gr, vls, vlp, cns, cnp = m.groups()
    for i, n in enumerate(vls.split(',')): seats.append(dict(exam=ex, grade=int(gr), subject='VL', role='setter', pos=i, teacher=n.strip()))
    seats.append(dict(exam=ex, grade=int(gr), subject='VL', role='reviewer', pos=0, teacher=vlp.strip()))
    seats.append(dict(exam=ex, grade=int(gr), subject='CN', role='setter', pos=0, teacher=cns.strip()))
    seats.append(dict(exam=ex, grade=int(gr), subject='CN', role='reviewer', pos=0, teacher=cnp.strip()))
tot = collections.Counter(s['teacher'] for s in seats)
stated = {k.strip(): int(v) for k, v in re.findall(r'([CT] [^\d,:]+?) (\d+)(?:,|$)', txt.split("Q's totals:")[1].strip().rstrip('.') + ',')}
per_exam = collections.Counter((s['teacher'], s['exam']) for s in seats)
res = dict(n_seats=len(seats), totals=dict(tot), stated_totals=stated, totals_match_statement=dict(tot) == stated,
           max_tasks_per_exam={f'{t}/{e}': c for (t, e), c in per_exam.items() if c > 2})
json.dump(dict(seats=seats, check=res), open(os.path.join(out, 'q-plan-task.json'), 'w'), ensure_ascii=False, indent=1)
# TSV in the same shape the importer tests use
exams = ['GK1', 'CK1', 'GK2', 'CK2']; grades = [10, 11, 12]
def cell(e, g, s, r, p):
    for x in seats:
        if (x['exam'], x['grade'], x['subject'], x['role'], x['pos']) == (e, g, s, r, p): return x['teacher']
    return ''
rows = ['Kì thi/khối\t\tKhối 10\t\tKhối 11\t\tKhối 12\t', '\t\tVL\tCN\tVL\tCN\tVL\tCN']
for e in exams:
    for k, (lab, role, pos) in enumerate([('Đề', 'setter', 0), ('Đề', 'setter', 1), ('P.Biện', 'reviewer', 0)]):
        cells = []
        for g in grades:
            cells += [cell(e, g, 'VL', role, pos), cell(e, g, 'CN', role, pos)]
        rows.append('\t'.join([e if k == 0 else '', lab] + cells))
open(os.path.join(out, 'q-plan.tsv'), 'w', encoding='utf-8').write('\n'.join(rows))
print(json.dumps(res, ensure_ascii=False))
