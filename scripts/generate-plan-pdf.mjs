import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');

async function main() {
  const playwrightPath = path.join(repoRoot, 'ui/node_modules/@playwright/test/index.mjs');
  const { chromium } = await import(pathToFileURL(playwrightPath).href);

  const html = `<!DOCTYPE html>
<html lang="vi">
<head>
  <meta charset="UTF-8">
  <title>Bảng phân công ra đề và phản biện đề kiểm tra</title>
  <style>
    @page {
      size: A4 landscape;
      margin: 8mm;
    }
    * {
      box-sizing: border-box;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    }
    body {
      margin: 0;
      padding: 10px;
      color: #000;
      background: #fff;
      font-size: 11px;
      line-height: 1.2;
    }
    .header-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      margin-bottom: 12px;
      font-size: 11px;
    }
    .header-left {
      text-align: center;
      font-weight: 600;
      text-transform: uppercase;
    }
    .header-left .sub {
      text-decoration: underline;
      font-weight: 700;
    }
    .header-right {
      text-align: center;
      font-weight: 600;
    }
    .header-right .motto {
      font-style: italic;
      text-decoration: underline;
      font-weight: 500;
    }
    .title-block {
      text-align: center;
      margin-bottom: 14px;
    }
    .title {
      font-size: 15px;
      font-weight: 800;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin: 0 0 4px 0;
    }
    .subtitle {
      font-size: 11px;
      font-style: italic;
      color: #333;
      margin: 0;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      border: 1.5px solid #222;
      text-align: center;
      font-size: 10px;
    }
    th, td {
      border: 1px solid #444;
      padding: 3px 4px;
      vertical-align: middle;
    }
    th {
      background-color: #e2e8f0;
      font-weight: 700;
    }
    .sub-th {
      background-color: #f1f5f9;
      font-weight: 600;
    }
    .exam-block {
      font-weight: 700;
      background-color: #f8fafc;
    }
    .role-cell {
      font-weight: 600;
      background-color: #f8fafc;
    }
    .spacer-col {
      width: 6px;
      border-top: none;
      border-bottom: none;
      background: #fff;
      padding: 0;
    }
    .text-left {
      text-align: left;
    }
    .total-row {
      font-weight: 700;
      background-color: #e2e8f0;
    }
    .sig-grid {
      display: grid;
      grid-template-columns: 1fr 1fr;
      margin-top: 14px;
      font-size: 11px;
    }
    .sig-right {
      text-align: center;
      margin-left: auto;
      width: 250px;
    }
  </style>
</head>
<body>
  <div class="header-grid">
    <div class="header-left">
      <div>TRƯỜNG THPT CHUYÊN</div>
      <div class="sub">TỔ CHUYÊN MÔN TOÁN</div>
    </div>
    <div class="header-right">
      <div>CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM</div>
      <div class="motto">Độc lập - Tự do - Hạnh phúc</div>
    </div>
  </div>

  <div class="title-block">
    <div class="title">[BẢN NHÁP] BẢNG PHÂN CÔNG RA ĐỀ VÀ PHẢN BIỆN ĐỀ KIỂM TRA</div>
    <div class="subtitle">Năm học: 2026-2027 — Phương án: Phương án chuẩn của Q (mẫu tổ)</div>
  </div>

  <table>
    <thead>
      <tr>
        <th rowspan="2" colspan="2" style="width: 80px;">Kì thi/khối</th>
        <th colspan="2">Khối 10</th>
        <th colspan="2">Khối 11</th>
        <th colspan="2">Khối 12</th>
        <th rowspan="2" class="spacer-col"></th>
        <th rowspan="2" style="width: 85px;" class="text-left">GV</th>
        <th rowspan="2" style="width: 65px;">Tổng lượt n.vụ</th>
        <th rowspan="2" style="width: 32px;">Đề</th>
        <th rowspan="2" style="width: 32px;">PB</th>
        <th rowspan="2" style="width: 35px;">GK1</th>
        <th rowspan="2" style="width: 35px;">CK1</th>
        <th rowspan="2" style="width: 35px;">GK2</th>
        <th rowspan="2" style="width: 35px;">CK2</th>
      </tr>
      <tr>
        <th class="sub-th" style="width: 65px;">VL</th>
        <th class="sub-th" style="width: 65px;">CN</th>
        <th class="sub-th" style="width: 65px;">VL</th>
        <th class="sub-th" style="width: 65px;">CN</th>
        <th class="sub-th" style="width: 65px;">VL</th>
        <th class="sub-th" style="width: 65px;">CN</th>
      </tr>
    </thead>
    <tbody>
      <!-- GK1 -->
      <tr>
        <td rowspan="3" class="exam-block">GK1</td>
        <td class="role-cell">Đề</td>
        <td>C Hiền</td><td>T Nghĩa</td>
        <td>T Lộc</td><td>T Nghĩa</td>
        <td>C Bình</td><td>T Nghĩa</td>
        <td class="spacer-col"></td>
        <td class="text-left">C Hiền</td>
        <td><strong>5</strong></td><td>3</td><td>2</td>
        <td>2</td><td>1</td><td>1</td><td>1</td>
      </tr>
      <tr>
        <td class="role-cell">Đề</td>
        <td>C Lài</td><td></td>
        <td>C Thư</td><td></td>
        <td>T Phúc</td><td></td>
        <td class="spacer-col"></td>
        <td class="text-left">C Lài</td>
        <td><strong>5</strong></td><td>3</td><td>2</td>
        <td>2</td><td>2</td><td>1</td><td>0</td>
      </tr>
      <tr>
        <td class="role-cell">P.Biện</td>
        <td>T Phúc</td><td>C Hiền</td>
        <td>C Na</td><td>C Lài</td>
        <td>C Quí</td><td>T Lộc</td>
        <td class="spacer-col"></td>
        <td class="text-left">T Phúc</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>2</td><td>0</td><td>1</td><td>1</td>
      </tr>

      <!-- CK1 -->
      <tr>
        <td rowspan="3" class="exam-block">CK1</td>
        <td class="role-cell">Đề</td>
        <td>C Hiền</td><td>T Nghĩa</td>
        <td>C Lài</td><td>T Nghĩa</td>
        <td>C Lan</td><td>T Nghĩa</td>
        <td class="spacer-col"></td>
        <td class="text-left">T Lộc</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>2</td><td>0</td><td>1</td><td>1</td>
      </tr>
      <tr>
        <td class="role-cell">Đề</td>
        <td>C Tú</td><td></td>
        <td>C Bình</td><td></td>
        <td>C Quí</td><td></td>
        <td class="spacer-col"></td>
        <td class="text-left">C Thư</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>1</td><td>1</td><td>2</td><td>0</td>
      </tr>
      <tr>
        <td class="role-cell">P.Biện</td>
        <td>C Như</td><td>C Tú</td>
        <td>C Thư</td><td>C Lan</td>
        <td>C Lài</td><td>C Bình</td>
        <td class="spacer-col"></td>
        <td class="text-left">C Na</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>1</td><td>0</td><td>1</td><td>2</td>
      </tr>

      <!-- GK2 -->
      <tr>
        <td rowspan="3" class="exam-block">GK2</td>
        <td class="role-cell">Đề</td>
        <td>C Tú</td><td>T Nghĩa</td>
        <td>C Na</td><td>T Nghĩa</td>
        <td>C Lan</td><td>T Nghĩa</td>
        <td class="spacer-col"></td>
        <td class="text-left">C Bình</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>1</td><td>2</td><td>0</td><td>1</td>
      </tr>
      <tr>
        <td class="role-cell">Đề</td>
        <td>C Như</td><td></td>
        <td>C Thư</td><td></td>
        <td>C Lài</td><td></td>
        <td class="spacer-col"></td>
        <td class="text-left">C Quí</td>
        <td><strong>6</strong></td><td>2</td><td>4</td>
        <td>1</td><td>1</td><td>1</td><td>3</td>
      </tr>
      <tr>
        <td class="role-cell">P.Biện</td>
        <td>T Lộc</td><td>C Như</td>
        <td>C Hiền</td><td>C Thư</td>
        <td>C Quí</td><td>T Phúc</td>
        <td class="spacer-col"></td>
        <td class="text-left">C Tú</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>0</td><td>2</td><td>1</td><td>1</td>
      </tr>

      <!-- CK2 -->
      <tr>
        <td rowspan="3" class="exam-block">CK2</td>
        <td class="role-cell">Đề</td>
        <td>C Na</td><td>T Nghĩa</td>
        <td>T Lộc</td><td>T Nghĩa</td>
        <td>T Phúc</td><td>T Nghĩa</td>
        <td class="spacer-col"></td>
        <td class="text-left">C Như</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>0</td><td>1</td><td>2</td><td>1</td>
      </tr>
      <tr>
        <td class="role-cell">Đề</td>
        <td>C Như</td><td></td>
        <td>C Quí</td><td></td>
        <td>C Hiền</td><td></td>
        <td class="spacer-col"></td>
        <td class="text-left">C Lan</td>
        <td><strong>4</strong></td><td>2</td><td>2</td>
        <td>0</td><td>2</td><td>1</td><td>1</td>
      </tr>
      <tr>
        <td class="role-cell">P.Biện</td>
        <td>C Tú</td><td>C Quí</td>
        <td>C Bình</td><td>C Na</td>
        <td>C Lan</td><td>C Quí</td>
        <td class="spacer-col"></td>
        <td class="text-left">T Nghĩa</td>
        <td><strong>12</strong></td><td>12</td><td>0</td>
        <td>3</td><td>3</td><td>3</td><td>3</td>
      </tr>

      <!-- Total Row -->
      <tr class="total-row">
        <td colspan="8"></td>
        <td class="spacer-col"></td>
        <td class="text-left"><strong>Tổng cộng</strong></td>
        <td><strong>60</strong></td><td><strong>36</strong></td><td><strong>24</strong></td>
        <td><strong>15</strong></td><td><strong>15</strong></td><td><strong>15</strong></td><td><strong>15</strong></td>
      </tr>
    </tbody>
  </table>

  <div class="sig-grid">
    <div>
      <p style="font-style: italic; color: #555; margin: 0;">
        * Lưu ý: Mọi cán bộ được phân công có trách nhiệm bảo mật đề thi và thực hiện đúng tiến độ.
      </p>
    </div>
    <div class="sig-right">
      <div style="font-style: italic; margin-bottom: 4px;">Hà Nội, ngày 04 tháng 10 năm 2026</div>
      <div style="font-weight: 700; text-transform: uppercase; margin-bottom: 50px;">TỔ TRƯỞNG CHUYÊN MÔN</div>
      <div style="font-weight: 700;">Nguyễn Văn A</div>
    </div>
  </div>
</body>
</html>`;

  const outputPath = path.join(repoRoot, 'docs/reports/phase-12/plan-grid.pdf');
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.setContent(html, { waitUntil: 'networkidle' });
  await page.pdf({
    path: outputPath,
    format: 'A4',
    landscape: true,
    printBackground: true,
    margin: {
      top: '8mm',
      bottom: '8mm',
      left: '8mm',
      right: '8mm',
    },
  });
  await browser.close();

  console.log(`Successfully generated ${outputPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
