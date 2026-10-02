import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { pathToFileURL } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

async function main() {
  const pwUrl = pathToFileURL(path.resolve(rootDir, 'ui/node_modules/@playwright/test/index.mjs')).href;
  const pw = await import(pwUrl);
  const chromium = pw.chromium;

  const viDir = path.resolve(rootDir, 'docs/user-guide/vi');
  const files = fs.readdirSync(viDir).filter(f => f.endsWith('.md')).sort();

  let fullHtml = `<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="utf-8">
<title>ExamPanel - Hướng Dẫn Sử Dụng v0.1.0</title>
<style>
  @page {
    size: A4;
    margin: 20mm 15mm 20mm 15mm;
    @bottom-center {
      content: counter(page);
    }
  }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    color: #1e293b;
    line-height: 1.6;
    font-size: 11pt;
    margin: 0;
    padding: 0;
  }
  .cover {
    height: 90vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    page-break-after: always;
  }
  .cover h1 {
    font-size: 28pt;
    color: #0f172a;
    margin-bottom: 8px;
    font-weight: 800;
  }
  .cover .subtitle {
    font-size: 14pt;
    color: #475569;
    margin-bottom: 24px;
  }
  .cover .badge {
    display: inline-block;
    padding: 6px 16px;
    background-color: #2563eb;
    color: #ffffff;
    border-radius: 9999px;
    font-weight: 600;
    font-size: 11pt;
    margin-bottom: 40px;
  }
  .cover .meta {
    margin-top: 60px;
    font-size: 10pt;
    color: #64748b;
  }
  .chapter {
    page-break-after: always;
  }
  .chapter:last-child {
    page-break-after: avoid;
  }
  h1 {
    color: #0f172a;
    font-size: 18pt;
    border-bottom: 2px solid #e2e8f0;
    padding-bottom: 8px;
    margin-top: 0;
  }
  h2 {
    color: #1e293b;
    font-size: 14pt;
    margin-top: 20px;
    margin-bottom: 8px;
  }
  h3 {
    color: #334155;
    font-size: 12pt;
    margin-top: 14px;
    margin-bottom: 6px;
  }
  p, ul, ol {
    margin-top: 6px;
    margin-bottom: 10px;
  }
  li {
    margin-bottom: 4px;
  }
  code {
    background-color: #f1f5f9;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: Consolas, monospace;
    font-size: 9.5pt;
    color: #0f172a;
  }
  pre {
    background-color: #0f172a;
    color: #f8fafc;
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    font-size: 9pt;
  }
  pre code {
    background-color: transparent;
    color: inherit;
    padding: 0;
  }
  hr {
    border: none;
    border-top: 1px solid #e2e8f0;
    margin: 20px 0;
  }
  strong {
    color: #0f172a;
  }
</style>
</head>
<body>
<div class="cover">
  <h1>ExamPanel</h1>
  <div class="subtitle">Phần mềm phân công ra đề và phản biện đề kiểm tra</div>
  <div class="badge">Phiên bản v0.1.0 • Tài liệu hướng dẫn sử dụng chính thức</div>
  <div class="meta">
    <p>Dành cho Ban Giám hiệu, Tổ trưởng chuyên môn và Giáo viên</p>
    <p>Phát hành: Tháng 10 / 2026</p>
  </div>
</div>
`;

  for (const file of files) {
    const content = fs.readFileSync(path.join(viDir, file), 'utf8');
    // Simple markdown to html conversion
    let html = content
      .replace(/^### (.*$)/gim, '<h3>$1</h3>')
      .replace(/^## (.*$)/gim, '<h2>$1</h2>')
      .replace(/^# (.*$)/gim, '<h1>$1</h1>')
      .replace(/\*\*(.*?)\*\*/gim, '<strong>$1</strong>')
      .replace(/\*(.*?)\*/gim, '<em>$1</em>')
      .replace(/`([^`]+)`/gim, '<code>$1</code>')
      .replace(/^---$/gim, '<hr/>')
      .replace(/^\s*-\s+(.*$)/gim, '<li>$1</li>')
      .replace(/(<li>.*<\/li>)/s, '<ul>$1</ul>')
      .replace(/\n\n+/g, '</p><p>');

    html = `<div class="chapter"><p>${html}</p></div>`;
    fullHtml += html;
  }

  fullHtml += `</body></html>`;

  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.setContent(fullHtml, { waitUntil: 'load' });
  const outPdf = path.resolve(rootDir, 'docs/user-guide/ExamPanel-HuongDan-0.1.0.pdf');
  await page.pdf({
    path: outPdf,
    format: 'A4',
    margin: { top: '20mm', bottom: '20mm', left: '15mm', right: '15mm' },
    printBackground: true,
    displayHeaderFooter: true,
    headerTemplate: '<div></div>',
    footerTemplate: '<div style="font-size:9pt; text-align:center; width:100%; color:#94a3b8;">ExamPanel v0.1.0 — Trang <span class="pageNumber"></span> / <span class="totalPages"></span></div>'
  });
  await browser.close();
  console.log(`PDF successfully generated at: ${outPdf}`);
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
