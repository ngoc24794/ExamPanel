import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, setInput } from '../lib/ui.mjs';
import { driveFileDialog, sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('D2');
const run = prepareRunDir('D1b', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
await withApp({ scenario: 'D1b', run }, async (app) => {
  await app.nav('/settings'); await sleep(900);
  const org = { 'org-school-name-input': 'TRƯỜNG THPT THỬ NGHIỆM', 'org-department-name-input': 'TỔ VẬT LÍ - CÔNG NGHỆ', 'org-place-name-input': 'Đà Nẵng', 'org-signer-title-input': 'TỔ TRƯỞNG CHUYÊN MÔN', 'org-signer-name-input': 'Nguyễn Thị Thử' };
  for (const [id, v] of Object.entries(org)) { const e = await tid(app.b, id); await e.click(); await e.clearValue(); await e.setValue(v); }
  await clickTid(app.b, 'save-org-info-btn'); await sleep(1000);
  const st = sql(db, "select key,value from settings where key like '%org%' or key like '%school%' or key like '%signer%'"); R.note('settings_rows', st);
  // mark plan 1 final through the UI
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'history-plans-button'); await sleep(700);
  await app.b.execute((id) => document.querySelector(`[data-testid="plan-menu-${id}"]`).dispatchEvent(new PointerEvent('pointerdown', { bubbles: true, button: 0, pointerType: 'mouse' })), 1); await sleep(500);
  await app.b.execute((el) => el.click(), await app.b.$('//*[@role="menuitem"][contains(.,"Đánh dấu chính thức")]')); await sleep(1500); await app.b.keys('Escape'); await sleep(500);
  R.check('D2.final-plan-set', sql(db, 'select is_final from plans where id=1')[0].is_final === 1, 'is_final=1', 'plan 1 final');
  // open the final plan explicitly through the history panel, then export
  await app.nav('/assignments'); await sleep(1500); if (!(await (await tid(app.b, 'plan-item-1')).isExisting())) { await clickTid(app.b, 'history-plans-button'); await sleep(700); } await (await tid(app.b, 'plan-item-1')).click(); await sleep(1000);
  const badge = await app.b.execute(() => document.querySelector('[data-testid="assignments-page"]').innerText.includes('Chính thức')); R.note('final_badge_visible_on_plan_header', badge);
  if (!(await app.b.execute(() => !!document.querySelector('[data-testid="export-excel-button"]')))) await app.b.keys('Escape');
  await sleep(500);
  await clickTid(app.b, 'export-excel-button'); const xl = path.join(run.dir, 'out/q-final-export.xlsx'); const dr = await driveFileDialog(xl, { shotName: 'D1b-export-dialog' }); await sleep(2000);
  R.check('D1.export-final-file', dr.ok && fs.existsSync(xl), dr, 'file written');
  if (fs.existsSync(xl)) fs.copyFileSync(xl, path.join(OUT_ABS, 'extracts/D1b-q-final-export.xlsx'));
  // print routes
  for (const [name, route] of [['plan', '/print/plan/1'], ['notices', '/print/notices/1']]) {
    await app.nav(route); await sleep(2500); await app.shot(`D2-print-${name}`, { screen: false });
    const info = await app.b.execute(() => ({ text: document.body.innerText.slice(0, 1500), pages: document.querySelectorAll('.print-page, [data-print-page], section.page').length, h: document.documentElement.scrollHeight, w: document.documentElement.scrollWidth }));
    fs.writeFileSync(path.join(OUT_ABS, `extracts/D2-print-${name}-dom.json`), JSON.stringify(info, null, 1));
    // WebDriver Print Page command
    try { const b64 = await app.b.printPage(name === 'plan' ? 'landscape' : 'portrait', 1, true, 21.0, 29.7, 1, 1, 1, 1, false); fs.writeFileSync(path.join(OUT_ABS, `pdfs/D2-real-${name}.pdf`), Buffer.from(b64, 'base64')); R.note(`printPage_${name}`, 'ok ' + b64.length); }
    catch (e) { R.note(`printPage_${name}`, 'FAILED: ' + String(e.message).slice(0, 300)); }
  }
});
