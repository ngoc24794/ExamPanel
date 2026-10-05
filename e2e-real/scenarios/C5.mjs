import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import { sql, key } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C5');
const run = prepareRunDir('C5', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const cellText = async (app, id) => (await (await tid(app.b, id)).getText()).split('\n')[0].trim();
const dbSeats = (pid) => sql(db, 'select exam_id,grade_id,subject_id,role,position,teacher_id from assignments where plan_id=? order by exam_id,grade_id,subject_id,role,position', [pid]);
await withApp({ scenario: 'C5', run }, async (app) => {
  await app.nav('/assignments'); await sleep(1500);
  const planName0 = await app.b.execute(() => document.querySelector('[data-testid="create-edit-copy-button"]')?.closest('div.flex')?.parentElement?.innerText?.slice(0, 80));
  // ---- optimizer plan immutable ----
  await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).click(); await sleep(600);
  let dlg = await app.b.execute(() => !!document.querySelector('[role="dialog"]'));
  const draggableOpt = await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).getAttribute('draggable');
  R.check('C5.optimizer-plan-immutable', !dlg && draggableOpt !== 'true' && (await (await tid(app.b, 'create-edit-copy-button')).isExisting()), { candidateDialogOpened: dlg, draggable: draggableOpt, offersEditCopy: true }, 'cell click does nothing; "Tạo bản chỉnh sửa" offered', ['extracts/C5.json']);
  // ---- duplicate ----
  const plansBefore = sql(db, 'select count(*) c from plans')[0].c;
  await clickTid(app.b, 'create-edit-copy-button'); await sleep(2800);
  const plans = sql(db, 'select id,name,source from plans order by id'); const copy = plans[plans.length - 1];
  R.check('C5.create-edit-copy-no-name-prompt', plans.length === plansBefore + 1 && copy.source === 'duplicate', { copy, note: 'phase-8 checklist step 4 expects a name prompt dialog; none appeared; name auto-set' }, 'source=duplicate; (checklist: asks for a name)', ['extracts/C5.json']);
  // intermittent bug (RA-018): the view may stay on the optimizer plan; pick the copy via history when that happens
  let viewSwitched = !(await (await tid(app.b, 'create-edit-copy-button')).isExisting());
  R.note('view_switched_to_copy_automatically', viewSwitched);
  if (!viewSwitched) { await clickTid(app.b, 'history-plans-button'); await sleep(700); const it = await app.b.$('//*[@data-testid="plans-history-list"]//*[contains(text(),"(Chỉnh sửa)")]'); await it.click(); await sleep(800); await app.b.keys('Escape'); await sleep(900); }
  const pid = copy.id; const seats0 = dbSeats(pid);
  await app.shot('C5-edit-copy', { screen: false });
  const draggable = await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).getAttribute('draggable');
  const tabindex = await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).getAttribute('tabindex');
  R.note('cell_attrs_editable', { draggable, tabindex, role: await (await tid(app.b, 'q-grid-cell-1-1-VL-setter-0')).getAttribute('role') });
  // ---- forced seat immovable ----
  const forcedCell = await tid(app.b, 'q-grid-cell-1-1-CN-setter-0'); const fd = await forcedCell.getAttribute('draggable'); await forcedCell.click(); await sleep(500);
  const dlgF = await app.b.execute(() => !!document.querySelector('[role="dialog"]'));
  R.check('C5.forced-seat-T-Nghia-immovable', !dlgF && fd !== 'true', { text: await cellText(app, 'q-grid-cell-1-1-CN-setter-0'), dialogOpened: dlgF, draggable: fd }, 'no candidate dialog, not draggable');
  // ---- candidate list ----
  const target = 'q-grid-cell-1-1-VL-setter-0'; const tx = async (id) => app.b.execute((i) => document.querySelector(`[data-testid="${i}"]`).textContent.trim(), id);
  const before = await tx(target);
  await (await tid(app.b, target)).click(); await sleep(1500);
  const dtext = await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText || null);
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/C5-candidate-dialog.txt'), dtext || ''); await app.shot('C5-candidate-modal');
  R.check('C5.candidate-dialog-opens-on-click', !!dtext, dtext && dtext.slice(0, 160), 'dialog "Chọn giáo viên thay thế"');
  const lines = (dtext || '').split('\n').map((l) => l.trim()).filter(Boolean);
  const names = ['Cô Hiền', 'Cô Lài', 'Thầy Phúc', 'Thầy Lộc', 'Cô Thư', 'Cô Na', 'Cô Bình', 'Cô Quí', 'Cô Tú', 'Cô Như', 'Cô Lan', 'Thầy Nghĩa'];
  const ui = {}; for (let i = 0; i < lines.length; i++) if (names.includes(lines[i])) { const nxt = lines.slice(i + 2, i + 6); ui[lines[i]] = /^[HS]\d+:|exceeded|_/.test(nxt[0] || '') ? { disabled: true, reason: nxt[0] } : { disabled: false, delta: nxt[0], total: nxt[1] }; }
  R.note('ui_candidates', ui);
  // independent engine check: replace the teacher in the slot and ask evaluate_assignments for hard violations
  const gp = await app.invoke('get_plan', { id: pid }); const A = gp.v.assignments; const tmap = (await app.invoke('list_teachers', {})).v; const idByName = Object.fromEntries(tmap.map((t) => [t.full_name, t.id]));
  const slotA = A.find((a) => a.exam_id === 1 && a.grade_id === 1 && a.subject_id === 2 && a.role === 'setter' && a.position === 0); const curId = slotA.teacher_id;
  const mism = [];
  for (const n of names) { const tid2 = idByName[n]; if (!ui[n]) continue; const mod = A.map((a) => (a === slotA ? { ...a, teacher_id: tid2 } : a)); const ev = await app.invoke('evaluate_assignments', { schoolYearId: 1, assignments: mod }); const hv = ev.ok ? ev.v.hard_violations.length : -1; const uiDis = ui[n].disabled; if (n === tmap.find((t) => t.id === curId)?.full_name) continue; if ((hv > 0) !== uiDis) mism.push({ n, engine_hard: hv, ui_disabled: uiDis }); }
  R.check('C5.candidate-disabled-iff-hard-violation(engine re-check)', mism.length === 0, { checked: Object.keys(ui).length, mismatches: mism }, 'disabled candidates have a hard violation; enabled ones none (independent evaluate_assignments)', ['extracts/C5-candidate-dialog.txt']);
  const raw = Object.values(ui).filter((x) => x.disabled).map((x) => x.reason);
  R.check('C5.candidate-reasons-human-readable', raw.every((r) => !/^[HS]\d+: [a-z_]+$/.test(r)) ? 'pass' : 'fail', raw, 'reason in Vietnamese words (checklist: "kèm lý do vi phạm")', ['extracts/C5-candidate-dialog.txt', 'screenshots/C5-candidate-modal-page.png']);
  // ---- keyboard: ArrowDown + Enter inside the modal applies the highlighted valid candidate ----
  await app.b.keys('ArrowDown'); await sleep(200); await app.b.keys('ArrowDown'); await sleep(200);
  const sel = await app.b.execute(() => { const d = document.querySelector('[role="dialog"]'); const a = d.querySelector('[aria-selected="true"],[data-selected="true"],.ring-2,.bg-accent'); return a ? a.innerText.replace(/\n+/g, ' | ').slice(0, 120) : null; });
  R.note('keyboard_selected_after_2xArrowDown', sel);
  await app.b.keys('Enter'); await sleep(1200);
  const after = await tx(target); const dlgAfter = await app.b.execute(() => !!document.querySelector('[role="dialog"]'));
  R.check('C5.keyboard-replace-in-modal', before !== after && !dlgAfter, { before, after, modalClosed: !dlgAfter, highlighted: sel }, 'Arrow keys move, Enter applies a valid candidate, cell updates', ['screenshots/C5-candidate-modal-page.png']);
  const dirtyShown = await app.b.execute(() => !!document.querySelector('[data-testid="save-plan-assignments-button"]'));
  R.check('C5.live-feedback-save-button-appears', dirtyShown, dirtyShown, 'Save/Discard appear after an edit');
  R.check('C5.edit-not-persisted-before-save', JSON.stringify(dbSeats(pid)) === JSON.stringify(seats0), 'unchanged in DB', 'DB unchanged until Save');
  // ---- undo / redo (buttons + ctrl+z / ctrl+y) ----
  const undoBtn = await app.b.$('button[title^="Hoàn tác"]'); const redoBtn = await app.b.$('button[title^="Làm lại"]');
  R.note('undo_redo_buttons_found', { undo: await undoBtn.isExisting(), redo: await redoBtn.isExisting() });
  await undoBtn.click(); await sleep(600); const afterUndo = await tx(target);
  await redoBtn.click(); await sleep(600); const afterRedo = await tx(target);
  R.check('C5.undo-redo-buttons', afterUndo === before && afterRedo === after, { before, afterUndo, afterRedo }, 'undo restores, redo reapplies');
  await app.b.execute(() => document.body.focus()); await app.b.keys(['Control', 'z']); await sleep(600); const kUndo = await tx(target);
  await app.b.keys(['Control', 'y']); await sleep(600); const kRedo = await tx(target);
  R.check('C5.undo-redo-keyboard-ctrl+z/ctrl+y', kUndo === before && kRedo === after, { before, kUndo, kRedo }, 'Ctrl+Z / Ctrl+Y (phase-8 step 7)');
  // ---- drag swap: pick a swap that the engine says is hard-valid (evaluate_swap), then drag ----
  const curA = (await app.invoke('get_plan', { id: pid })).v.assignments; // DB state (pre-save)
  const vlSetters = [];
  for (const e of [1, 2, 3, 4]) for (const g of [1, 2, 3]) for (const pos of [0, 1]) vlSetters.push({ exam_id: e, grade_id: g, subject_id: 2, role: 'setter', position: pos });
  let pair = null;
  outer: for (let i = 0; i < vlSetters.length; i++) for (let j = i + 1; j < vlSetters.length; j++) { if (vlSetters[i].exam_id === 1 && vlSetters[i].grade_id === 1 && vlSetters[i].position === 0) continue; const r = await app.invoke('evaluate_swap', { schoolYearId: 1, assignments: curA, slotA: vlSetters[i], slotB: vlSetters[j] }); if (r.ok && r.v.hard_violations.length === 0 && r.v.delta_score < 12) { pair = [vlSetters[i], vlSetters[j]]; break outer; } }
  R.note('valid_swap_pair_found_by_engine', pair);
  const idOf = (s) => `q-grid-cell-${s.exam_id}-${s.grade_id}-VL-${s.role}-${s.position}`;
  const a1 = idOf(pair[0]), a2 = idOf(pair[1]); const t1 = await tx(a1), t2 = await tx(a2);
  const e1 = await tid(app.b, a1), e2 = await tid(app.b, a2); await e1.scrollIntoView({ block: 'center' });
  await e1.dragAndDrop(e2); await sleep(1200);
  let n1 = await tx(a1), n2 = await tx(a2); let dndMode = 'webdriver-dragAndDrop';
  if (!(n1 === t2 && n2 === t1)) { // fallback: synthetic DataTransfer events (labelled)
    await app.b.execute((i1, i2) => { const c1 = document.querySelector(`[data-testid="${i1}"]`), c2 = document.querySelector(`[data-testid="${i2}"]`); const dt = new DataTransfer(); c1.dispatchEvent(new DragEvent('dragstart', { bubbles: true, dataTransfer: dt })); c2.dispatchEvent(new DragEvent('dragover', { bubbles: true, cancelable: true, dataTransfer: dt })); c2.dispatchEvent(new DragEvent('drop', { bubbles: true, cancelable: true, dataTransfer: dt })); }, a1, a2); await sleep(1000); dndMode = 'synthetic-DragEvent (WebDriver native drag did not trigger; real mouse drag => manual-required)'; n1 = await tx(a1); n2 = await tx(a2);
  }
  R.check('C5.drag-swap', n1 === t2 && n2 === t1, { mode: dndMode, a1: [t1, '->', n1], a2: [t2, '->', n2], pair }, 'two seats exchange teachers (hard-valid swap chosen via evaluate_swap)', []);
  // ---- save ----
  await clickTid(app.b, 'save-plan-assignments-button'); await sleep(1800);
  const seats1 = dbSeats(pid); const changed = seats1.filter((s, i) => JSON.stringify(s) !== JSON.stringify(seats0[i])).length;
  const pl = sql(db, 'select score from plans where id=?', [pid])[0];
  R.check('C5.save-persists', changed >= 3 && !(await app.b.execute(() => [...document.querySelectorAll('[data-sonner-toast]')].some((e) => /đã tồn tại/.test(e.innerText)))), { seats_changed: changed, score_after: pl.score }, 'DB assignments updated (1 replace + 2 swapped seats = 3)');
  const hdr = await app.b.execute(() => document.querySelector('[data-testid="assignments-page"]').innerText.match(/Tổng điểm phạt: [\d.]+/)?.[0]);
  R.check('C5.score-in-header-equals-db', hdr && Math.abs(parseFloat(hdr.split(': ')[1]) - pl.score) < 0.01, { header: hdr, db: pl.score }, 'header score == saved score');
  fs.writeFileSync(path.join(run.dir, 'out/state.json'), JSON.stringify({ pid }));
  await app.shot('C5-after-save');
});
