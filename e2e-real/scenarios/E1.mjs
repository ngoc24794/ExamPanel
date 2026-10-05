// Keyboard: tab order, visible focus, Esc closes dialogs, grid keyboard-only reach.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('E1');
const run = prepareRunDir('E1', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const active = () => { const e = document.activeElement; if (!e || e === document.body) return { body: true }; const cs = getComputedStyle(e); const r = e.getBoundingClientRect(); const ring = (cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) > 0) || (cs.boxShadow && cs.boxShadow !== 'none'); return { tag: e.tagName, tid: e.getAttribute('data-testid'), txt: (e.innerText || e.getAttribute('aria-label') || e.getAttribute('title') || e.value || '').trim().slice(0, 40), role: e.getAttribute('role'), ring: !!ring, opacity: cs.opacity, visible: r.width > 0 && r.height > 0 && cs.visibility !== 'hidden', inGridCell: !!e.closest('[data-testid^="q-grid-cell-"]') }; };
const out = { tab: {}, esc: {}, grid: {} };
await withApp({ scenario: 'E1', run }, async (app) => {
  for (const route of ['/', '/teachers', '/rules', '/assignments', '/settings']) {
    await app.nav(route); await sleep(1200); await app.b.execute(() => document.body.focus()); const seq = [];
    for (let i = 0; i < 45; i++) { await app.b.keys('Tab'); seq.push(await app.b.execute(active)); }
    out.tab[route] = seq;
    const noRing = seq.filter((s) => !s.body && s.visible && !s.ring).map((s) => `${s.tag}${s.tid ? '#' + s.tid : ''}:${s.txt}`);
    const invisible = seq.filter((s) => !s.body && !s.visible).length; const bodyHits = seq.filter((s) => s.body).length;
    R.check(`E1.tab-focus-visible${route}`, noRing.length === 0 ? 'pass' : 'fail', { tabs: seq.length, without_visible_ring: noRing.slice(0, 8), n_without: noRing.length, focus_on_hidden_elements: invisible, focus_lost_to_body: bodyHits }, 'every focused control shows an outline/ring', ['extracts/E1-tab-sequences.json']);
  }
  // Esc closes dialogs
  const dlgs = [['/teachers', 'create-teacher-btn', 'teacher'], ['/subjects', 'create-subject-btn', 'subject'], ['/exams', 'add-exam-btn', 'exam'], ['/assignments', 'run-optimizer-button', 'run-optimize'], ['/assignments', 'compare-plans-button', 'compare'], ['/assignments', 'import-plan-button', 'import-plan'], ['/teachers', 'import-excel-btn', 'import-excel'], ['/settings', 'restore-file-btn_skip', 'skip']];
  for (const [route, btn, name] of dlgs) { if (btn.endsWith('_skip')) continue; await app.nav(route); await sleep(900); await clickTid(app.b, btn); await sleep(800); const open = await app.b.execute(() => !!document.querySelector('[role="dialog"]')); await app.b.keys('Escape'); await sleep(600); const still = await app.b.execute(() => !!document.querySelector('[role="dialog"]')); out.esc[name] = { opened: open, closed_by_esc: open && !still }; }
  R.check('E1.esc-closes-all-dialogs', Object.values(out.esc).every((v) => v.opened && v.closed_by_esc), out.esc, 'Esc closes each modal');
  const fs2 = await app.nav('/assignments'); await sleep(900); await clickTid(app.b, 'feasibility-indicator'); await sleep(900); const sheetOpen = await app.b.execute(() => !!document.querySelector('[role="dialog"]')); await app.b.keys('Escape'); await sleep(600); out.esc.feasibility_sheet = { opened: sheetOpen, closed_by_esc: sheetOpen && !(await app.b.execute(() => !!document.querySelector('[role="dialog"]'))) };
  R.check('E1.esc-closes-feasibility-sheet', out.esc.feasibility_sheet.opened && out.esc.feasibility_sheet.closed_by_esc, out.esc.feasibility_sheet, 'Esc closes the sheet');
  // Grid keyboard-only reach on an editable copy
  await app.nav('/assignments'); await sleep(1500); await clickTid(app.b, 'create-edit-copy-button'); await sleep(3000);
  if (await (await tid(app.b, 'create-edit-copy-button')).isExisting()) { await clickTid(app.b, 'history-plans-button'); await sleep(700); await (await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"(Chỉnh sửa)")]')).click(); await sleep(800); await app.b.keys('Escape'); await sleep(800); }
  await app.b.execute(() => document.querySelector('[data-testid="q-plan-grid-table"]').scrollIntoView()); await app.b.execute(() => { document.activeElement && document.activeElement.blur(); const t = document.querySelector('[data-testid="view-toggle-detail"]'); t && t.focus(); });
  const seq = []; let reached = null;
  for (let i = 0; i < 80; i++) { await app.b.keys('Tab'); const a = await app.b.execute(active); seq.push(a); if (a.inGridCell && !reached) reached = { after_tabs: i + 1, el: a }; }
  out.grid.sequence_len = seq.length; out.grid.reached_cell = reached; out.grid.gridcell_focusables = seq.filter((s) => s.inGridCell).map((s) => ({ tag: s.tag, tid: s.tid, opacity: s.opacity, ring: s.ring }));
  const cellTabindex = await app.b.execute(() => [...document.querySelectorAll('[data-testid^="q-grid-cell-"]')].slice(0, 3).map((c) => ({ tabindex: c.getAttribute('tabindex'), role: c.getAttribute('role') })));
  out.grid.cell_attrs = cellTabindex;
  R.check('E1.grid-cells-reachable-by-keyboard', !!reached && out.grid.gridcell_focusables.some((s) => s.tag === 'TD' || s.tag === 'DIV'), { tab_stops_in_grid: out.grid.gridcell_focusables.length, first: reached, cell_attrs: cellTabindex }, 'phase-12 checklist K1.6: Enter on a cell opens candidate dialog => cells must be focusable', ['extracts/E1-tab-sequences.json']);
  const menuInvisibleOnFocus = out.grid.gridcell_focusables.filter((s) => s.tid === 'q-cell-menu-btn' && parseFloat(s.opacity) < 0.5).length;
  R.check('E1.grid-menu-button-visible-when-focused', out.grid.gridcell_focusables.length > 0 && menuInvisibleOnFocus === 0, { focusable_menu_buttons: out.grid.gridcell_focusables.length, focused_with_opacity_0: menuInvisibleOnFocus }, 'focused menu trigger must be visible', ['extracts/E1-tab-sequences.json']);
  await app.shot('E1-grid-keyboard', { screen: false });
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/E1-tab-sequences.json'), JSON.stringify(out, null, 1));
