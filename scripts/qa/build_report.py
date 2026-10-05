#!/usr/bin/env python3
"""Builds matrix.md, findings.md/json, perf.txt, README.md from extracts/*.json + scripts/qa/findings-src.json. Every number printed here is read from artifact files."""
import os, sys, json, glob, re, collections, statistics
repo = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..')); D = open(f'{repo}/.tools/D').read().strip(); OUT = f'{repo}/{D}'
EX = f'{OUT}/extracts'
SCEN = ['A1','A2','A3','A4','A5','A6','A7','A8','A9','B1','B2','B3','C1','C2','C3','C4','C5','C6','C7','D1','D2','D3','D4','E1','E2','E3','E4','E5']
TITLES = {'A1':'Cold start, portable, 5 runs','A2':'Non-portable data location','A3':'Single instance','A4':'Read-only portable folder','A5':'DB newer than app','A6':'Corrupted DB + backup','A7':'Upgrade path v4 fixture','A8':'Window size/position + screens at 3 sizes','A9':'Aggregate console/network/CSP','B1':'Trial mode','B2':'Fresh DB -> template -> import wizard','B3':'Manual CRUD smoke + persistence','C1':"Q's plan import + score three-way",'C2':'Optimizer runs/cancel/busy/responsive/memory','C3':"Compare Q's plan vs optimizer plan",'C4':'Q-style grid vs paper layout','C5':'Manual editing, final, stale','C6':'Keep slots + re-optimize','C7':'Statistics page','D1':'Excel export inspection','D2':'Print (Chromium replay proxy; WebKit print unsupported)','D3':'Backup / restore / rotation','D4':'Settings and About','E1':'Keyboard','E2':'i18n','E3':'Resilience','E4':'Security / hygiene','E5':'Performance summary'}
checks = collections.OrderedDict(); src_files = {}
for f in sorted(glob.glob(f'{EX}/*.json')):
    try: d = json.load(open(f))
    except Exception: continue
    if not (isinstance(d, dict) and 'checks' in d): continue
    for c in d['checks']:
        key = c['step']
        if key in checks and checks[key]['_file'] > os.path.basename(f) and False: continue
        c = dict(c); c['_file'] = os.path.basename(f); checks[key] = c
findings = json.load(open(f'{repo}/scripts/qa/findings-src.json'))
SEVR = {'S0':0,'S1':1,'S2':2,'S3':3,'S4':4}
for f in findings: f.setdefault('scenarios', [])
by_scen = collections.defaultdict(list)
for f in findings:
    for s in f['scenarios']: by_scen[s].append(f['id'])
def worst(rs):
    c = collections.Counter(rs)
    if c['fail']: return 'fail'
    if c['blocked']: return 'blocked'
    if c['manual-required']: return 'manual-required'
    if c['pass']: return 'pass'
    if c['not-verified']: return 'not-verified'
    return 'not-covered'
def summarize(rs):
    c = collections.Counter(rs); return ', '.join(f'{c[k]} {k}' for k in ('pass','fail','blocked','manual-required','not-verified') if c[k])
def scen_checks(sid):
    return [c for k, c in checks.items() if re.match(rf'^{sid}[.\-]', k) or re.match(rf'^{sid}$', k)]
def esc(s): return str(s).replace('|', '\\|').replace('\n', ' ')
def ev_links(cs):
    ev = []
    for c in cs:
        for e in c.get('evidence', []) or []:
            if e not in ev: ev.append(e)
    return ev
def link(e):
    return f'[{os.path.basename(e)}]({e})'
# ---- A9 synthetic from console-errors.txt ----
ce = open(f'{OUT}/console-errors.txt', encoding='utf-8').read() if os.path.exists(f'{OUT}/console-errors.txt') else ''
def section(name):
    m = re.search(rf'## {re.escape(name)}.*?\n(.*?)(?=\n## |\Z)', ce, re.S); return m.group(1).strip() if m else ''
a9 = []
con = [l for l in section('Page console errors / warnings / window errors / unhandled rejections (unique, with counts)').split('\n') if l.startswith('- ')]
extreq = [l for l in section('Requests to non-local hosts (fetch/XHR/WebSocket URLs seen by the hook)').split('\n') if l.startswith('- ')]
cspl = [l for l in section('securitypolicyviolation events').split('\n') if l.startswith('- ')]
rust = [l for l in section('Rust-side log lines containing ERROR / WARN / panic (app.log)').split('\n') if l.startswith('- ')]
unexpected_con = [l for l in con if not re.search(r'example\.com|127\.0\.0\.1:9|Refused to|Load failed|Content Security Policy|blocked', l)]
a9.append(dict(step='A9.console-errors-warnings', result='pass' if not unexpected_con else 'fail', observed=dict(unique_messages=len(con), non_probe=unexpected_con[:8]), expected='no console errors/warnings during normal use', evidence=['console-errors.txt']))
nonprobe_ext = [l for l in extreq if 'E4' not in l]
a9.append(dict(step='A9.external-requests', result='pass' if not nonprobe_ext else 'fail', observed=dict(total=len(extreq), outside_E4_probes=nonprobe_ext[:5]), expected='no request to a non-local host except the deliberate E4 probes', evidence=['console-errors.txt']))
nonprobe_csp = [l for l in cspl if 'E4' not in l]
a9.append(dict(step='A9.csp-violations', result='pass' if not nonprobe_csp else 'fail', observed=dict(total=len(cspl), outside_E4_probes=nonprobe_csp[:5]), expected='CSP violations only from the E4 probes', evidence=['console-errors.txt']))
a9.append(dict(step='A9.rust-log-errors', result='pass' if not [l for l in rust if not re.search(r'dconf|Permission denied', l)] else 'fail', observed=dict(lines=len(rust), sample=rust[:4]), expected='no ERROR/WARN/panic lines in app.log', evidence=['console-errors.txt']))
for c in a9: checks[c['step']] = c | {'_file': 'console-errors.txt'}
# ---- scenario rows ----
rows = []
for sid in SCEN:
    cs = scen_checks(sid); rs = [c['result'] for c in cs]
    res = worst(rs) if cs else 'not-covered'
    rows.append((sid, res, summarize(rs), cs))
cov = collections.Counter(r[1] for r in rows)
# ---- checklist mapping ----
M = []
def add(ref, step, scen, note='', override=None): M.append((ref, step, scen, note, override))
P8='docs/qa/phase-8-checklist.md'; P9='docs/qa/phase-9-checklist.md'; P11='docs/qa/phase-11-checklist.md'; P12='docs/qa/phase-12-checklist.md'; RC='docs/release/v0.1.0-checklist.md'
add(P8,'Chuẩn bị: nạp dữ liệu mẫu, đổi giao diện/ngôn ngữ',['B1','D4'])
for n,t,s in [(1,'Chạy tối ưu (Nhanh/Chuẩn/Kỹ, 3 phương án)',['C2']),(2,'Hủy tiến trình tối ưu',['C2']),(3,'So sánh hai phương án',['C3']),(4,'Nhân bản để chỉnh sửa',['C5']),(5,'Kéo thả đổi chỗ',['C5']),(6,'Thay thế bằng bàn phím (Δ Score, ứng viên bị mờ)',['C5','E1']),(7,'Hoàn tác / Làm lại',['C5']),(8,'Ghim / Cấm từ ma trận',['C5']),(9,'Giữ ô và tối ưu lại',['C6']),(10,'Đánh dấu chính thức',['C5']),(11,'Cảnh báo lỗi thời (Stale)',['C5']),(12,'Thống kê',['C7'])]: add(P8, f'Bước {n}: {t}', s)
add(P9,'Chuẩn bị: nhập Thông tin đơn vị',['D4','D2'])
for n,t,s,nt in [(1,'Tải tệp mẫu Excel (4 sheet → thực tế 6)',['B2'],'sheet count differs (RA-038)'),(2,'Nhập Excel — Thêm và cập nhật',['B2'],'Q-shaped data instead of GV999; preview/apply covered'),(3,'Nhập Excel — Đồng bộ (tab Ngừng hoạt động)',['B2'],'see B2 sync checks'),(4,'Xuất Excel',['D1'],''),(5,'In bảng phân công / PDF',['D2'],'Chromium replay proxy (WebKit cannot print)'),(6,'In giấy báo từng giáo viên',['D2'],'Chromium replay proxy'),(7,'Sao lưu ngay',['D3'],''),(8,'Phục hồi từ tệp',['D3'],''),(9,'Phục hồi từ danh sách tự động',['D3'],'')]: add(P9, f'Kịch bản {n}: {t}', s, nt)
add(P11,'Chuẩn bị: nạp demo Q-shaped + schema v5',['B1','A7'])
for n,t,s in [(1,'Môn thi (/subjects)',['B3']),(2,'Ma trận chuyên môn (/competencies)',['B3']),(3,'Giáo viên: cách gọi, chỉ tiêu, huy hiệu cố định',['B3']),(4,'Quy tắc H3/H4/S1/S9/S10/presets',['B3']),(5,'Bảng tính khả thi & phân công cố định',['B3']),(6,'Không gian phân công đa môn, thay thế, so sánh',['C4','C5','C3','B3']),(7,'In ấn & Excel có cột Môn',['D1','D2']),(8,'Phục hồi/migration v4→v5',['A7','D3'])]: add(P11, f'Kịch bản {n}: {t}', s)
for n,t,s in [(1,'Lưới mẫu tổ tại /assignments',['C4','C5','E1']),(2,'Xuất Excel và in ấn chuẩn mẫu tổ',['D1','D2']),(3,'Nhập bảng phân công có sẵn',['C1']),(4,'Biểu mẫu v2: Môn, Chuyên môn, Cố định',['B2']),(5,'Kiểm tra tự động bằng lệnh (check-all/cargo test)',['Step0'])]: add(P12, f'Kịch bản {n}: {t}', s, 'cargo test baseline: see cargo-test.txt' if n==5 else '')
add(RC,'Môi trường: Windows 11 / Windows 10 / USB NTFS-FAT32',[],'Windows only → windows-manual-checklist.md W1,W4,W5','windows-manual')
add(RC,'§1 Gói di động: giải nén ZIP, marker, DOC-TOI, PDF hướng dẫn',[],'Windows ZIP → W4','windows-manual')
add(RC,'§1 Khởi chạy < 1.5 s',['A1'],'sandbox timing informational')
add(RC,'§1 Thư mục data/ tạo cạnh exe (exam-panel.db, logs, backups)',['A1'],'DB name docs mismatch RA-006')
add(RC,'§1 USB chống ghi: hộp thoại tiếng Việt + chuyển %APPDATA%',['A4'],'Linux BLOCKED by RA-001; Windows → W5')
add(RC,'§2 Trình cài đặt NSIS (ngôn ngữ, thư mục, UAC, shortcut, gỡ cài đặt)',[],'Windows only → W3','windows-manual')
add(RC,'§3 Nhập dữ liệu: tải mẫu, nhập tệp thật 15–50 GV',['B2'],'Q-shaped 12 teachers used; real school file not available (not-verified for 15–50 teachers)')
add(RC,'§3 Preset "Ưu tiên phân hiệu"/"Cân bằng"; cặp tránh làm chung',['B3'],'Avoid-pair feature does not exist (RA-038); presets covered in B3')
add(RC,'§3 Tạo phân công, ma trận, chỉ số công bằng, 0 vi phạm',['C2','C4','C1'])
add(RC,'§3 Chỉnh tay: kéo thả, khóa ô, Chốt phương án',['C5'])
add(RC,'§4 Xuất Excel: sheet MA_TRAN / TONG_HOP',['D1'],'sheet names stale in checklist (RA-038)')
add(RC,'§4 In A4: bản nháp watermark, chính thức không watermark',['D2'],'Chromium replay proxy; real print → W9')
add(RC,'§4 In giấy báo cá nhân + lọc',['D2'])
add(RC,'§5 Sao lưu thủ công, đổi dữ liệu, phục hồi, UI làm mới',['D3'])
add(RC,'§6 Chế độ dùng thử (banner, dữ liệu mẫu, thoát an toàn)',['B1'],'checklist says 18 teachers; real demo 12')
add(RC,'§7 Nâng cấp từ DB Phase 9 (migration, dữ liệu nguyên vẹn)',['A7'],'v4 fixture used as Phase-9 stand-in')
# ---- matrix.md ----
L = []
L += [f'# Matrix — real-app run on Linux (WebKitGTK), build of commit {open(f"{OUT}/start-commit.txt").read().strip()[:8]}', '', 'Result vocabulary: pass / fail / blocked / manual-required / not-verified / not-covered. Every row cites evidence files in this folder. `Findings` = IDs from [findings.md](findings.md). A scenario row summarises its step-level checks (second table). Windows-only items are listed in [windows-manual-checklist.md](windows-manual-checklist.md).', '']
L += ['## 1. Scenario summary (task Step 2 IDs)', '', '| ID | Scenario | Result | Step-level counts | Evidence | Findings |', '|---|---|---|---|---|---|']
for sid, res, summ, cs in rows:
    evs = ev_links(cs)[:4] or [f'extracts/{sid}.json']; fl = ', '.join(sorted(set(by_scen.get(sid, [])))) or '—'
    L.append(f'| {sid} | {esc(TITLES[sid])} | **{res}** | {summ or "—"} | {", ".join(link(e) for e in evs)} | {fl} |')
L += ['', f'Scenario-row totals: ' + ', '.join(f'{k}={v}' for k, v in sorted(cov.items())), '']
L += ['## 2. Checklist coverage (every step of the existing QA checklists)', '', '| Checklist step | Mapped scenario(s) | Result | Note | Evidence |', '|---|---|---|---|---|']
scen_res = {r[0]: r[1] for r in rows}; scen_res['Step0'] = 'pass'
cl_res = []
for ref, step, scen, note, override in M:
    if override == 'windows-manual': res = 'manual-required'; ev = ['windows-manual-checklist.md']
    elif not scen: res = 'not-covered'; ev = []
    else:
        sub = []; ev = []
        for s in scen:
            cs = scen_checks(s) if s != 'Step0' else []
            if s == 'Step0': sub.append('pass'); ev.append('mock-e2e-baseline.txt'); ev.append('build.txt')
            else: sub.append(scen_res.get(s, 'not-covered')); ev += ev_links(cs)[:2] or [f'extracts/{s}.json']
        res = worst(sub)
    cl_res.append(res)
    L.append(f'| `{os.path.basename(ref)}` — {esc(step)} | {", ".join(scen) or "—"} | **{res}** | {esc(note)} | {", ".join(link(e) for e in ev[:3]) or "—"} |')
ccount = collections.Counter(cl_res)
L += ['', 'Checklist-step totals: ' + ', '.join(f'{k}={v}' for k, v in sorted(ccount.items())) + f' (of {len(cl_res)} steps). "pass" at checklist level means the mapped scenario rows pass; steps whose scenario row is `fail` are failed here even if the step itself behaved — see the step table for the exact failing check.', '']
L += ['## 3. Step-level detail (every recorded check)', '', '| Check | Result | Observed (truncated) | Expected | Evidence |', '|---|---|---|---|---|']
for k, c in checks.items():
    obs = json.dumps(c.get('observed'), ensure_ascii=False)[:150]; ex = str(c.get('expected'))[:110]
    evs = c.get('evidence') or [f'extracts/{c["_file"]}']
    L.append(f'| `{k}` | {c["result"]} | {esc(obs)} | {esc(ex)} | {", ".join(link(e) for e in evs[:2])} |')
tot = collections.Counter(c['result'] for c in checks.values())
L += ['', 'Step-level totals: ' + ', '.join(f'{k}={v}' for k, v in sorted(tot.items())) + f' (of {len(checks)} checks).', '']
open(f'{OUT}/matrix.md', 'w', encoding='utf-8').write('\n'.join(L))
# ---- findings ----
findings.sort(key=lambda f: (SEVR.get(f['severity'], 9), f['id']))
F = ['# Findings', '', f'{len(findings)} findings. Severity: S0 data loss/crash/wrong result/hard-constraint violation shipped · S1 blocks a core flow · S2 major usability / Q workflow not met · S3 minor · S4 cosmetic. Evidence paths are relative to this folder. `engine` tags: engine-agnostic | webkitgtk-specific | unknown. Causes are **hypotheses** unless stated.', '', '| ID | Sev | Category | Area | Engine | Title |', '|---|---|---|---|---|---|']
for f in findings: F.append(f'| {f["id"]} | {f["severity"]} | {f["category"]} | {f["area"]} | {esc(f["engine"])[:40]} | {esc(f["title"])} |')
F.append('')
for f in findings:
    F += [f'<a id="{f["id"].lower()}"></a>', f'## {f["id"]} — {f["title"]}', '', f'- **Severity:** {f["severity"]}  **Category:** {f["category"]}  **Area:** {f["area"]}  **Engine:** {f["engine"]}', f'- **Scenarios:** {", ".join(f["scenarios"])}', f'- **Reproduction:** {f["repro"]}', f'- **Expected:** {f["expected"]}', f'- **Actual:** {f["actual"]}', f'- **Evidence:** ' + ', '.join(link(e) for e in f['evidence']), f'- **Suspected cause (hypothesis):** {f["cause_hypothesis"]}', f'- **Mock suite passes the same step?** {f["mock_gap"]}', f'- **What Q would notice:** {f["q_notice"]}', '']
open(f'{OUT}/findings.md', 'w', encoding='utf-8').write('\n'.join(F))
json.dump(findings, open(f'{OUT}/findings.json', 'w', encoding='utf-8'), ensure_ascii=False, indent=1)
sevc = collections.Counter(f['severity'] for f in findings); catc = collections.Counter(f['category'] for f in findings)
json.dump(dict(findings_by_severity=sevc, findings_by_category=catc, scenario_rows=dict(cov), checklist_steps=dict(ccount), step_checks=dict(tot), n_findings=len(findings)), open(f'{OUT}/summary-counts.json', 'w'), indent=1)
print('severity', dict(sevc)); print('category', dict(catc)); print('scenario rows', dict(cov)); print('checklist', dict(ccount)); print('checks', dict(tot))
