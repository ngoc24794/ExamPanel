#!/usr/bin/env python3
"""D2 checks on the Chromium-replay PDFs (MOCK-DATA-REPLAY proxy). Writes extracts/D2-pdfchecks.json (recorder format)."""
import subprocess, json, re, sys, os
d = sys.argv[1]; checks = []
def chk(step, ok, observed, expected, evidence):
    checks.append(dict(step=step, result='pass' if ok is True else ('fail' if ok is False else ok), observed=observed, expected=expected, evidence=evidence))
def info(f): o = subprocess.run(['pdfinfo', f], capture_output=True, text=True).stdout; return dict(pages=int(re.search(r'Pages:\s+(\d+)', o).group(1)), size=re.search(r'Page size:\s+(.*)', o).group(1))
def txt(f): return subprocess.run(['pdftotext', '-layout', f, '-'], capture_output=True, text=True).stdout
P = lambda n: f'{d}/../pdfs/D2-chromium-replay-{n}.pdf'
ev = lambda n: [f'pdfs/D2-chromium-replay-{n}.pdf', f'extracts/D2-chromium-replay-{n}.txt']
pf, pd_, nf, nd = (info(P(n)) for n in ('plan-final', 'plan-draft', 'notices-final', 'notices-draft'))
chk('D2.plan-1-page-A4-landscape', pf['pages'] == 1 and pd_['pages'] == 1 and pf['size'].startswith('841') , dict(final=pf, draft=pd_), '1 page, A4 landscape (841.9 x 595 pt)  [Chromium replay, MOCK-DATA-REPLAY]', ev('plan-final'))
tf, td = txt(P('plan-final')), txt(P('plan-draft'))
chk('D2.plan-draft-marker-and-final-without', '[BẢN NHÁP]' in td and '[BẢN NHÁP]' not in tf, dict(draft_has_marker='[BẢN NHÁP]' in td, final_has_marker='[BẢN NHÁP]' in tf), 'draft has [BẢN NHÁP] + diagonal watermark (see page PNG); final has neither', ev('plan-draft') + ['screenshots/D2-pdf-plan-draft-1.png', 'screenshots/D2-pdf-plan-final-1.png'])
need = ['TRƯỜNG THPT THỬ NGHIỆM', 'TỔ VẬT LÍ - CÔNG NGHỆ', 'CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM', 'Độc lập - Tự do - Hạnh phúc', 'Kì thi/khối', 'P.Biện', 'Tổng cộng', 'Nguyễn Thị Thử', 'TỔ TRƯỞNG CHUYÊN MÔN', 'Đà Nẵng']
miss = [n for n in need if n not in tf]
chk('D2.plan-diacritics-and-signature-block', not miss and '�' not in tf, dict(missing=miss, replacement_chars='�' in tf), 'Vietnamese strings extract intact; org/place/signer from Settings printed', ev('plan-final'))
nums = [int(x) for x in re.findall(r'Tổng cộng\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+(\d+)', tf)[0]]
chk('D2.plan-print-totals', nums == [60, 36, 24, 15, 15, 15, 15], nums, '60/36/24/15x4', ev('plan-final'))
chk('D2.notices-one-page-per-teacher', nf['pages'] == 12 and nd['pages'] == 12 and nf['size'].startswith('594'), dict(final=nf, draft=nd), '12 pages (12 teachers with duties), A4 portrait', ev('notices-final'))
tn = txt(P('notices-final')); pages = tn.split('\f')
split_rows = [i + 1 for i, p in enumerate(pages) if p.strip() and ('STT' in p) and p.count('Kính gửi') != 1]
chk('D2.notices-each-page-single-teacher', not split_rows, dict(bad_pages=split_rows), 'each page has exactly one "Kính gửi" block (no table split across pages)', ev('notices-final'))
# notice content: Cô Hiền page: 5 tasks; same-subject members only
hien = next(p for p in pages if 'Cô Hiền' in p.split('Phân hiệu')[0])
rows = re.findall(r'\n\s*(\d)\s+(Giữa|Cuối) kỳ (\d)\s+Khối (\d+)\s+(Vật lí|Công nghệ)', hien)
chk('D2.notice-content-Hien-5-tasks', len(rows) == 5, [' '.join(r) for r in rows], '5 duties for Cô Hiền (Q totals)', ev('notices-final'))
tdr = txt(P('notices-draft'))
chk('D2.notices-draft-marker', 'NHÁP' in tdr and 'NHÁP' not in tn, dict(draft='NHÁP' in tdr, final='NHÁP' in tn), 'draft marker on non-final notices; none on final', ev('notices-draft'))
json.dump(dict(id='D2pdf', checks=checks, notes={}), open(f'{d}/D2-pdfchecks.json', 'w'), ensure_ascii=False, indent=1)
for c in checks: print(c['result'], c['step'], json.dumps(c['observed'], ensure_ascii=False)[:200])
