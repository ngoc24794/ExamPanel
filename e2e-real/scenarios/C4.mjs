import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C4');
const run = prepareRunDir('C4', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const extract = () => {
  const tbl = document.querySelector('[data-testid="q-plan-grid-table"]'); const panel = document.querySelector('[data-testid="q-plan-totals-panel"]');
  const rows = [...tbl.querySelectorAll('tr')].map((tr) => [...tr.children].map((c) => ({ t: c.textContent.trim().replace(/\s+/g, ' '), cs: c.colSpan, rs: c.rowSpan })));
  const prow = [...panel.querySelectorAll('tr')].map((tr) => [...tr.children].map((c) => c.textContent.trim().replace(/\s+/g, ' ')));
  const gr = tbl.getBoundingClientRect(), pr = panel.getBoundingClientRect(); const root = document.querySelector('[data-testid="q-plan-grid-root"]').getBoundingClientRect();
  const sc = (el) => { let e = el; const out = []; while (e && e !== document.body) { const cs = getComputedStyle(e); if (/(auto|scroll)/.test(cs.overflowY) && e.scrollHeight > e.clientHeight + 1) out.push({ tag: e.tagName, cls: e.className.slice(0, 60), sh: e.scrollHeight, ch: e.clientHeight }); e = e.parentElement; } return out; };
  return { vw: innerWidth, vh: innerHeight, grid: { x: gr.x, y: gr.y, w: gr.width, h: gr.height, bottom: gr.bottom, right: gr.right }, panel: { x: pr.x, y: pr.y, w: pr.width, h: pr.height, bottom: pr.bottom, right: pr.right }, rootBottom: root.bottom, docScrollH: document.documentElement.scrollHeight, scrollers: sc(tbl), panelScrollers: sc(panel), rows, prow, bodyHasHScroll: document.documentElement.scrollWidth > innerWidth };
};
await withApp({ scenario: 'C4', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  // pick plan 1 (Q's own) if history list shows several: the latest created plan is shown by default; open history to pick
  const hist = await tid(app.b, 'history-plans-button'); await hist.click(); await sleep(600);
  const items = await app.b.$$('[data-testid="plans-history-list"] *'); const list = await app.b.execute(() => document.querySelector('[data-testid="plans-history-list"]')?.innerText);
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C4-history-list.txt'), list || ''); await app.shot('C4-history', { screen: false });
  const qItem = await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"Nhập từ bảng của tổ")]'); if (await qItem.isExisting()) await qItem.click(); await sleep(1000);
  await app.b.keys('Escape'); await sleep(500);
  const results = {};
  for (const [w, h] of [[1280, 720], [1920, 1080]]) {
    await app.b.setWindowSize(w, h); await sleep(900);
    for (const theme of ['light', 'dark']) {
      await app.nav('/settings'); await sleep(600); await clickTid(app.b, `theme-${theme}-btn`); await sleep(400);
      await app.nav('/assignments'); await sleep(1200);
      const ex = await app.b.execute(extract); results[`${w}x${h}-${theme}`] = ex;
      await app.shot(`C4-grid-${w}x${h}-${theme}`);
    }
  }
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C4-grid-structure.json'), JSON.stringify(results, null, 1));
  await app.nav('/settings'); await clickTid(app.b, 'theme-light-btn');
});
