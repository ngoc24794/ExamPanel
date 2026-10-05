#!/usr/bin/env python3
"""D1: verify the exported plan workbook (C1-q-plan-export.json from inspect_xlsx.py) against Q's plan (q-plan-task.json, derived from the task text). Writes extracts/D1.json in recorder format."""
import json, sys, collections, os
d = sys.argv[1]
x = json.load(open(f'{d}/C1-q-plan-export.json')); q = json.load(open(f'{d}/q-plan-task.json'))['seats']
checks = []
def chk(step, ok, observed, expected, evidence=None):
    checks.append(dict(step=step, result='pass' if ok is True else ('fail' if ok is False else ok), observed=observed, expected=expected, evidence=evidence or ['extracts/C1-q-plan-export.json']))
S = {s['title']: s for s in x['sheets']}
names = list(S)
chk('D1.sheets', names == ['Bảng phân công (mẫu tổ)', 'Phân công', 'Theo giáo viên', 'Thống kê', 'Tiêu chí'], names, 'phase-12 checklist: first sheet "Bảng phân công (mẫu tổ)"; phase-9: Phân công / Theo giáo viên / Thống thống kê / Tiêu chí (checklist v0.1.0 says MA_TRAN / TONG_HOP: stale)')
g = S[names[0]]
chk('D1.first-sheet-page-setup', g['orientation'] == 'landscape' and g['paper_size'] == 9 and g['fit_to_page'], dict(orientation=g['orientation'], paper=g['paper_size'], fit_to_page=g['fit_to_page'], fit_w=g['fit_to_width'], fit_h=g['fit_to_height']), 'A4 landscape, fit to 1 page wide')
chk('D1.first-sheet-merged-header', {'A7:B8', 'C7:D7', 'E7:F7', 'G7:H7'} <= set(g['merged']) and 'A4:Q4' in g['merged'], sorted(g['merged'])[:12], 'merged "Kì thi/khối" A7:B8 and grade groups C7:D7,E7:F7,G7:H7; title A4:Q4')
rows = g['rows']
hdr = rows[6]
chk('D1.header-text', hdr[0] == 'Kì thi/khối' and hdr[2:8] == ['Khối 10', None, 'Khối 11', None, 'Khối 12', None] and hdr[9:17] == ['GV', 'Tổng lượt n.vụ', 'Đề', 'PB', 'GK1', 'CK1', 'GK2', 'CK2'], hdr, 'Kì thi/khối | Khối 10..12 | spacer | GV | Tổng lượt n.vụ | Đề | PB | GK1..CK2')
draft = any('BẢN NHÁP' in str(c) for r in rows[:6] for c in r if c)
chk('D1.draft-marker-for-non-final', draft, [c for r in rows[:6] for c in r if c and 'NHÁP' in str(c)], '[BẢN NHÁP] on non-final plans')
# grid values vs expected seats
cols = {10: (2, 3), 11: (4, 5), 12: (6, 7)}  # (VL col, CN col) zero-based in rows
exams = ['GK1', 'CK1', 'GK2', 'CK2']; bad = []
exp = collections.defaultdict(dict)
for s in q: exp[(s['exam'], s['grade'], s['subject'], s['role'], s['pos'])] = s['teacher']
r0 = 8
for ei, e in enumerate(exams):
    for k, (role, pos) in enumerate([('setter', 0), ('setter', 1), ('reviewer', 0)]):
        row = rows[r0 + ei * 3 + k]
        for gr, (cv, cc) in cols.items():
            for subj, c in (('VL', cv), ('CN', cc)):
                got = row[c]; want = exp.get((e, gr, subj, role, pos))
                if (got or None) != (want or None): bad.append(dict(cell=f'{e}/{gr}/{subj}/{role}{pos}', got=got, want=want))
chk('D1.grid-values-equal-Q-plan', not bad, dict(mismatches=bad[:6], n=len(bad)), '60 seats equal Q ground truth (independent parse of task text); CN second Đề blank', ['extracts/C1-q-plan-export.json', 'extracts/q-plan-task.json'])
tot = rows[r0 + 12]
chk('D1.total-row', 'Tổng cộng' in tot and [c for c in tot if isinstance(c, (int, float))] == [60, 36, 24, 15, 15, 15, 15], [c for c in tot if c is not None], 'Tổng cộng 60 / 36 / 24 / 15 x4')
cnt = collections.Counter(s['teacher'] for s in q); bad2 = []
for r in rows[r0:r0 + 12]:
    name = r[9]; vals = r[10:17]
    if cnt[name] != vals[0]: bad2.append((name, vals[0], cnt[name]))
chk('D1.per-teacher-totals-equal-Q', not bad2 and len([1 for r in rows[r0:r0 + 12] if r[9]]) == 12, bad2, 'GV totals equal Q totals (Hiền 5, Lài 5, ... Quí 6, Nghĩa 12)')
# sign-off block
tail = [c for r in rows[r0 + 13:] for c in r if c]
chk('D1.signature-block', any('Tổ trưởng' in str(c) for c in tail) and any('ngày' in str(c) for c in tail), tail, 'place/date + signer title (+ name when configured in Settings)')
hdr_txt = [c for r in rows[:3] for c in r if c]
chk('D1.header-org-info-defaults', 'not-verified', hdr_txt, 'Org info was not configured in this run; export printed fallback "TRƯỜNG THPT CHUYÊN / TỔ CHUYÊN MÔN TOÁN" -> see finding on invented defaults')
# per-teacher sheet: "Cùng ban với" must contain only the same subject panel
tp = S['Theo giáo viên']['rows'][1:]
by = collections.defaultdict(list)
for s in q: by[(s['exam'], s['grade'], s['subject'])].append(s)
tmap = {}  # full name -> display; use display via names in sheet col 1 (full) vs q uses display; map by order of GV code in template (GV001..)
disp = ['C Hiền', 'C Lài', 'T Phúc', 'T Lộc', 'C Thư', 'C Na', 'C Bình', 'C Quí', 'C Tú', 'C Như', 'C Lan', 'T Nghĩa']
full = {'Cô Hiền': 'C Hiền', 'Cô Lài': 'C Lài', 'Thầy Phúc': 'T Phúc', 'Thầy Lộc': 'T Lộc', 'Cô Thư': 'C Thư', 'Cô Na': 'C Na', 'Cô Bình': 'C Bình', 'Cô Quí': 'C Quí', 'Cô Tú': 'C Tú', 'Cô Như': 'C Như', 'Cô Lan': 'C Lan', 'Thầy Nghĩa': 'T Nghĩa'}
subj = {'Vật lí': 'VL', 'Công nghệ': 'CN'}; rolevn = {'Ra đề': 'setter', 'Phản biện': 'reviewer'}; badp = []
for r in tp:
    me = full[r[1]]; key = (r[4], int(r[5].split()[-1]), subj[r[3]]); members = set(s['teacher'] for s in by[key]) - {me}
    shown = set(full[m.split(' (')[0]] for m in r[7].split('; ') if m)
    if shown != members: badp.append(dict(row=f'{r[1]} {r[4]} {r[5]} {r[3]}', shown=sorted(shown), expected=sorted(members)))
chk('D1.theo-giao-vien-cung-ban-voi-same-subject-only', not badp, dict(n_rows=len(tp), n_wrong=len(badp), first=badp[:3]), '"Cùng ban với" lists only members of the same subject panel (phase-11 checklist step 7)', ['extracts/C1-q-plan-export.json'])
chk('D1.theo-giao-vien-row-count', len(tp) == 60, len(tp), '60 rows (one per seat)')
st = S['Thống kê']['rows'][1:]; chk('D1.thong-ke-equals-Q-totals', all(cnt[full[r[1]]] == r[5] for r in st) and len(st) == 12, [(r[1], r[5]) for r in st][:3], '12 rows, totals equal Q')
json.dump(dict(id='D1', checks=checks, notes={}), open(f'{d}/D1.json', 'w'), ensure_ascii=False, indent=1)
for c in checks: print(c['result'], c['step'], json.dumps(c['observed'], ensure_ascii=False)[:230])
