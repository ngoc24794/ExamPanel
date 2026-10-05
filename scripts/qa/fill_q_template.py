#!/usr/bin/env python3
"""Fill the app-generated import template (v2) with the Q-shaped dataset. usage: fill_q_template.py <template.xlsx> <out.xlsx> [no-qui-override]"""
import sys, json, openpyxl, os
src, dst = sys.argv[1], sys.argv[2]
q = json.load(open(os.path.join(os.path.dirname(__file__), 'q-shaped.json')))
wb = openpyxl.load_workbook(src)
def clear(ws):
    ws.delete_rows(2, ws.max_row)
ws = wb['Phân hiệu']; clear(ws); ws.append([q['campus']['code'], q['campus']['name']])
ws = wb['Môn']; clear(ws)
for s in q['subjects']: ws.append(s)
ws = wb['Giáo viên']; clear(ws)
for i, (code, name, disp, grades, quota, maxt) in enumerate(q['teachers'], 1):
    ws.append([f'GV{i:03d}', name, disp, q['campus']['code'], grades, '1', 'Có', None, quota, maxt])
ws = wb['Môn đảm nhiệm']; clear(ws)
for i, t in enumerate(q['teachers'], 1):
    disp = t[2]
    if t[0] == 'NGHIA':
        ws.append([disp, 'CN', 'Ra đề', 'Mọi khối'])
    else:
        ws.append([disp, 'VL', 'Cả hai', 'Theo khối dạy'])
        ws.append([disp, 'CN', 'Phản biện', 'Mọi khối'])
ws = wb['Lịch vắng']; clear(ws)
wb.save(dst); print('wrote', dst)
