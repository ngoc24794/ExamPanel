// i18n scan: every route + key dialogs, vi and en, on the golden Q DB with plans.
import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, tid } from '../lib/ui.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('E2');
const run = prepareRunDir('E2', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const scan = () => {
  const out = { text: document.body.innerText, attrs: [] };
  document.querySelectorAll('[aria-label],[title],[placeholder],[alt]').forEach((e) => ['aria-label', 'title', 'placeholder', 'alt'].forEach((a) => { const v = e.getAttribute(a); if (v) out.attrs.push(v); }));
  return out;
};
const KEY = /(?<![\w@/.\-])[a-z][A-Za-z0-9]*(?:\.[A-Za-z][A-Za-z0-9]*){1,3}(?![\w@/\-]|\.(?:xlsx|db|json|png|pdf|txt|md))/g;
const states = [];
await withApp({ scenario: 'E2', run }, async (app) => {
  const routes = ['/', '/teachers', '/campuses', '/subjects', '/competencies', '/exams', '/unavailability', '/rules', '/assignments', '/statistics', '/settings'];
  for (const lang of ['vi', 'en']) {
    await app.nav('/settings'); await sleep(700); await clickTid(app.b, `lang-${lang}-btn`); await sleep(600);
    for (const r of routes) {
      await app.nav(r); await sleep(1100);
      if (r === '/rules') { for (const tab of ['tab-hard-rules', 'tab-soft-rules', 'tab-quotas', 'tab-locks']) { await clickTid(app.b, tab); await sleep(500); states.push({ lang, where: r + '#' + tab, ...(await app.b.execute(scan)) }); } continue; }
      if (r === '/statistics') { const cb = await app.b.$('[role="combobox"]'); if (await cb.isExisting()) { await cb.click(); await sleep(300); const o = await app.b.$('[role="option"]'); if (await o.isExisting()) { await o.click(); await sleep(1500); } } }
      states.push({ lang, where: r, ...(await app.b.execute(scan)) });
      await app.shot(`E2-${lang}-${r.replace(/\W/g, '') || 'overview'}`, { screen: false });
    }
    // dialogs
    const dlgs = [['/teachers', 'create-teacher-btn', 'teacher-dialog'], ['/subjects', 'create-subject-btn', 'subject-dialog'], ['/exams', 'add-exam-btn', 'exam-dialog'], ['/assignments', 'run-optimizer-button', 'run-dialog'], ['/assignments', 'compare-plans-button', 'compare-dialog'], ['/assignments', 'import-plan-button', 'import-plan-dialog'], ['/teachers', 'import-excel-btn', 'import-excel-dialog'], ['/teachers', 'quota-preview-btn', 'quota-dialog'], ['/', 'feasibility-indicator', 'feasibility-sheet']];
    for (const [route, btn, name] of dlgs) { await app.nav(route); await sleep(900); try { await clickTid(app.b, btn); await sleep(900); states.push({ lang, where: name, ...(await app.b.execute(scan)) }); await app.shot(`E2-${lang}-${name}`, { screen: false }); await app.b.keys('Escape'); await sleep(400); await app.b.keys('Escape'); await sleep(300); } catch (e) { states.push({ lang, where: name, error: String(e.message).slice(0, 100), text: '', attrs: [] }); } }
  }
  await app.nav('/settings'); await clickTid(app.b, 'lang-vi-btn');
});
fs.writeFileSync(path.join(OUT_ABS, 'extracts/E2-dom-scan.json'), JSON.stringify(states.map((s) => ({ lang: s.lang, where: s.where, error: s.error, text_len: s.text.length })), null, 1));
// analysis
const findings = { raw_keys: {}, forbidden: {}, mixed: {}, hard_coded_other_language: {} };
const VI_CHARS = /[ăâđêôơưàáạảãèéẹẻẽìíịỉĩòóọỏõùúụủũỳýỵỷỹ]/i;
const EN_WORDS = /\b(Close|Cancel|Save|Delete|Edit|Search|Loading|Error|Success|Toggle|Refresh|Select|Back|Next|Apply|Import|Export|Download|Backup|Restore|Settings|Teachers|Subjects|Exams|Rules|Statistics|Assignments)\b/;
for (const s of states) {
  const all = (s.text || '') + '\n' + (s.attrs || []).join('\n');
  for (const m of all.matchAll(KEY)) { const k = m[0]; if (/^\d|^v\d/.test(k)) continue; (findings.raw_keys[k] ||= new Set()).add(`${s.lang}:${s.where}`); }
  if (s.lang === 'vi') { for (const re of [/cơ sở/gi, /Kế hoạch/g, /Ràng buộc (cứng|mềm)/g]) for (const m of all.matchAll(re)) (findings.forbidden[m[0]] ||= new Set()).add(`${s.lang}:${s.where}`); }
  if (s.lang === 'en') { const lines = (s.text || '').split('\n').filter((l) => VI_CHARS.test(l)); for (const l of lines) (findings.mixed['en-page-has-vietnamese: ' + l.trim().slice(0, 80)] ||= new Set()).add(`en:${s.where}`); }
  if (s.lang === 'vi') { const lines = (s.text || '').split('\n').filter((l) => EN_WORDS.test(l) && !VI_CHARS.test(l) && l.length < 60); for (const l of lines) (findings.mixed['vi-page-has-english: ' + l.trim()] ||= new Set()).add(`vi:${s.where}`); }
}
const ser = Object.fromEntries(Object.entries(findings).map(([k, v]) => [k, Object.fromEntries(Object.entries(v).map(([a, b]) => [a, [...b]]))]));
fs.writeFileSync(path.join(OUT_ABS, 'extracts/E2-i18n-findings.json'), JSON.stringify(ser, null, 1));
R.check('E2.no-raw-i18n-keys', Object.keys(ser.raw_keys).length === 0, ser.raw_keys, 'no untranslated key patterns a.b.c / {{', ['extracts/E2-i18n-findings.json']);
R.check('E2.forbidden-terms-vi', Object.keys(ser.forbidden).length === 0, ser.forbidden, 'no "cơ sở", "Kế hoạch", "Ràng buộc cứng/mềm" in vi UI', ['extracts/E2-i18n-findings.json']);
R.check('E2.en-mode-has-no-vietnamese', Object.keys(ser.mixed).filter((k) => k.startsWith('en-')).length === 0, Object.keys(ser.mixed).filter((k) => k.startsWith('en-')).slice(0, 30), 'EN pages contain no Vietnamese strings', ['extracts/E2-i18n-findings.json']);
R.check('E2.vi-mode-has-no-english', Object.keys(ser.mixed).filter((k) => k.startsWith('vi-')).length === 0, Object.keys(ser.mixed).filter((k) => k.startsWith('vi-')).slice(0, 30), 'VI pages contain no English UI words', ['extracts/E2-i18n-findings.json']);
