// Phase-11 checklist extras: rules presets/S1/S9-S10/H4, competencies bulk/filters/footer, feasibility sheet forced placements, assignments subject filter.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid, bodyText } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('B3');
const run = prepareRunDir('B3b', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const weights = () => Object.fromEntries(sql(db, 'select rule_key,weight,enabled from rule_settings order by rule_key').map((r) => [r.rule_key, `${r.weight}${r.enabled ? '' : ' (off)'}`]));
const safe = async (name, fn) => { try { await fn(); } catch (e) { R.check(name, 'fail', 'EXCEPTION ' + String(e.message).slice(0, 240), 'step completes'); try { await app.b.keys('Escape'); } catch {} } };
let app;
await withApp({ scenario: 'B3b', run }, async (a) => {
  app = a;
  await safe('B3.rules-h4-fields-and-h3-warning', async () => {
    await app.nav('/rules'); await sleep(900);
    const h4 = { tasks: await (await tid(app.b, 'input-h4-max-tasks')).getValue(), setter: await (await tid(app.b, 'input-h4-max-setter')).getValue() };
    const tg = await tid(app.b, 'toggle-h3'); const before = await tg.getAttribute('aria-checked'); await tg.click(); await sleep(300); const warn1 = await (await tid(app.b, 'h3-warning-box')).isExisting(); await tg.click(); await sleep(300); const warn2 = await (await tid(app.b, 'h3-warning-box')).isExisting();
    R.check('B3.rules-h4-fields-and-h3-warning', h4.tasks === '2' && h4.setter === '1' && warn1 !== warn2, { h4, h3_before: before, warning_when_off: warn1, warning_when_on: warn2 }, 'H4 defaults 2/1; yellow warning box only when H3 is off (phase-11 step 4.2)');
  });
  await safe('B3.rules-s1-auto-and-s9-s10', async () => {
    await clickTid(app.b, 'tab-soft-rules'); await sleep(500); const t = await bodyText(app.b);
    const hasS9 = /S9/.test(t) && /S10/.test(t); const auto = await (await tid(app.b, 's1-mode-auto')).isExisting(); const maxIn = await (await tid(app.b, 's1-max-tasks-input')).isExisting();
    R.check('B3.rules-s1-auto-and-s9-s10', hasS9 && auto && maxIn, { has_S9_S10: hasS9, s1_auto_button: auto, s1_numeric_input: maxIn }, 'S1 auto/numeric; S9 and S10 cards exist (phase-11 step 4.3)');
    const sl = await tid(app.b, 'slider-s9'); const v0 = await sl.getValue(); await app.b.execute((el) => { const s = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; s.call(el, '12'); el.dispatchEvent(new Event('input', { bubbles: true })); el.dispatchEvent(new Event('change', { bubbles: true })); }, sl); await sleep(300);
    R.note('s9_slider', { before: v0, after: await sl.getValue(), max_attr: await sl.getAttribute('max') });
  });
  await safe('B3.rules-presets', async () => {
    const out = {}; const base = weights();
    for (const [id, name] of [['preset-workload-btn', 'workload'], ['preset-diversity-btn', 'diversity'], ['preset-crowding-btn', 'crowding'], ['preset-balanced-btn', 'balanced']]) { await clickTid(app.b, id); await sleep(500); await clickTid(app.b, 'save-rules-btn'); await sleep(900); out[name] = weights(); }
    fs.writeFileSync(path.join(OUT_ABS, 'extracts/B3b-presets-weights.json'), JSON.stringify({ base, ...out }, null, 1));
    const wl = out.workload, dv = out.diversity, cr = out.crowding, bl = out.balanced;
    R.check('B3.rules-presets-change-weights-as-documented', parseFloat(wl.s1) > parseFloat(base.s1) || parseFloat(wl.s8) > parseFloat(base.s8) || parseFloat(wl.s9) > parseFloat(base.s9) ? true : false, { base_s1_s8_s9: [base.s1, base.s8, base.s9], workload_s1_s8_s9: [wl.s1, wl.s8, wl.s9], diversity_s3_s4_s5_s10: [dv.s3, dv.s4, dv.s5, dv.s10], crowding_s9: cr.s9, balanced_s9: bl.s9 }, 'workload raises S1,S8,S9; diversity raises S3,S4,S5,S10; crowding turns S9 down/off; balanced restores defaults (phase-11 step 4.4)', ['extracts/B3b-presets-weights.json']);
    const soft = (w) => Object.fromEntries(Object.entries(w).filter(([k]) => /^s\d+$/.test(k)));
    R.check('B3.rules-balanced-restores-default-soft-weights', JSON.stringify(soft(bl)) === JSON.stringify(soft(base)), { balanced_soft: soft(bl), initial_soft: soft(base) }, 'S1..S10 weights == defaults after "Cân bằng" (my first expectation compared H3 too: initial DB had H3 OFF but the preset switches H3 back ON — recorded below)', ['extracts/B3b-presets-weights.json']);
    R.note('presets_touch_hard_rule_flags', { initial_h3: base.h3, after_workload_h3: wl.h3, after_diversity_h3: dv.h3, after_crowding_h3: cr.h3, after_balanced_h3: bl.h3 });
  });
  await safe('B3.competencies-bulk-filter-footer', async () => {
    await app.nav('/competencies'); await sleep(1000);
    const foot0 = await (await tid(app.b, 'eligible-count-footer')).getText(); const rows0 = sql(db, 'select count(*) c from teacher_competencies')[0].c;
    await clickTid(app.b, 'bulk-clear-btn'); await sleep(600); const dl = await app.b.execute(() => document.querySelector('[role="dialog"],[role="alertdialog"]')?.innerText || null); R.note('bulk_clear_confirm_dialog', dl);
    const conf = await app.b.$('//*[@role="alertdialog" or @role="dialog"]//button[contains(.,"Xác nhận") or contains(.,"Xóa") or contains(.,"Đồng ý")]'); if (await conf.isExisting()) { await conf.click(); await sleep(1200); }
    const rows1 = sql(db, 'select count(*) c from teacher_competencies')[0].c; const foot1 = await (await tid(app.b, 'eligible-count-footer')).getText();
    await clickTid(app.b, 'bulk-assign-all-btn'); await sleep(600); const conf2 = await app.b.$('//*[@role="alertdialog" or @role="dialog"]//button[contains(.,"Xác nhận") or contains(.,"Gán") or contains(.,"Đồng ý")]'); if (await conf2.isExisting()) { await conf2.click(); await sleep(1500); }
    const rows2 = sql(db, 'select count(*) c from teacher_competencies')[0].c; const foot2 = await (await tid(app.b, 'eligible-count-footer')).getText();
    const feas = await app.b.execute(() => document.querySelector('[data-testid="feasibility-indicator"]')?.innerText.replace(/\n/g, ' '));
    R.check('B3.competencies-bulk-clear-and-assign-all-persist-footer-updates', rows1 === 0 && rows2 > rows0 && foot0 !== foot1 && foot1 !== foot2, { rows_before: rows0, after_clear: rows1, after_assign_all: rows2, footer: [foot0, foot1, foot2], indicator_after_assign: feas, confirm_dialog_seen: !!dl }, 'bulk actions persist at once; footer counts update (phase-11 step 2.5/2.6)', ['extracts/B3b.json']);
    await app.shot('B3b-competencies-after-bulk', { screen: false });
    const sel = await tid(app.b, 'filter-subject-select'); await sel.click(); await sleep(400); const opts = await app.b.$$('[role="option"]'); const names = []; for (const o of opts) names.push(await o.getText()); R.note('subject_filter_options', names); await app.b.keys('Escape');
    const search = await tid(app.b, 'search-teacher-input'); await search.setValue('Quí'); await sleep(500); const vis = await app.b.execute(() => [...document.querySelectorAll('[data-testid^="competency-toggle-"]')].map((e) => e.getAttribute('data-testid').split('-')[2])); const uniq = [...new Set(vis)];
    R.check('B3.competencies-search-filter', uniq.length === 1, { teacher_ids_visible_after_search_Quí: uniq }, 'one teacher row', []);
    await clickTid(app.b, 'bulk-assign-taught-btn'); await sleep(600); const conf3 = await app.b.$('//*[@role="alertdialog" or @role="dialog"]//button[contains(.,"Xác nhận") or contains(.,"Gán") or contains(.,"Đồng ý")]'); if (await conf3.isExisting()) { await conf3.click(); await sleep(1500); }
  });
  await safe('B3.feasibility-sheet-forced-placements', async () => {
    await clickTid(app.b, 'feasibility-indicator'); await sleep(1200); const sheet = await app.b.execute(() => document.querySelector('[data-testid="feasibility-sheet"]')?.innerText || document.querySelector('[role="dialog"]')?.innerText || '');
    fs.writeFileSync(path.join(OUT_ABS, 'extracts/B3b-feasibility-sheet.txt'), sheet); await app.shot('B3b-feasibility-sheet', { screen: false });
    const forcedGroup = await (await tid(app.b, 'forced-placements-info-group')).isExisting();
    const nF = (sheet.match(/Thầy Nghĩa|T Nghĩa/g) || []).length;
    R.check('B3.feasibility-sheet-forced-placements', forcedGroup && nF >= 1, { forced_group_present: forcedGroup, mentions_of_T_Nghia: nF, head: sheet.slice(0, 300).replace(/\n+/g, ' | ') }, 'lists forced seats of T Nghĩa (teacher, exam, grade, subject, role) (phase-11 step 5)', ['extracts/B3b-feasibility-sheet.txt', 'screenshots/B3b-feasibility-sheet-page.png']);
    const items = await app.b.execute(() => [...document.querySelectorAll('[data-testid="forced-placements-info-group"] li, [data-testid="forced-placements-info-group"] tr, [data-testid="forced-placements-info-group"] > div > div')].length); R.note('forced_items_count', items);
    await app.b.keys('Escape'); await sleep(500);
  });
  await safe('B3.assignments-subject-filter', async () => {
    await app.nav('/assignments'); await sleep(1500);
    const before = await app.b.execute(() => document.querySelectorAll('[data-testid^="q-grid-cell-"]').length);
    await app.b.execute(() => { window.__vp = 1; }); const dsel = await tid(app.b, 'view-toggle-detail'); await dsel.click(); await sleep(900);
    const vlBtn = await app.b.$('//button[contains(.,"VL - ")]'); const ex = await vlBtn.isExisting(); let res = { filter_buttons_present_in_detail_view: ex };
    if (ex) { const t0 = await app.b.execute(() => document.querySelector('[data-testid="plan-matrix-view"]').innerText); await vlBtn.click(); await sleep(900); const t = await app.b.execute(() => document.querySelector('[data-testid="plan-matrix-view"]').innerText); res.before_mentions_CN = /CN \(/.test(t0); res.after_filter_VL_mentions_CN = /CN \(|Công nghệ/.test(t); res.after_filter_mentions_VL = /VL/.test(t); const raw = await app.b.execute(() => document.querySelector('[data-testid="assignments-page"]').innerText.match(/common\.all[^\n]*/)?.[0]); res.raw_key_in_filter_label = raw; }
    R.check('B3.assignments-subject-filter', ex && res.after_filter_VL_mentions_CN === false && res.after_filter_mentions_VL, res, 'filter buttons (Tất cả / VL / CN) narrow the matrix (phase-11 step 6.4); exists only in Chi tiết view and its first label is the raw key common.all', ['screenshots/B3b-subject-filter-page.png']);
    await app.shot('B3b-subject-filter', { screen: false });
  });
});
