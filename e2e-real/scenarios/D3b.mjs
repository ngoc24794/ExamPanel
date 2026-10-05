import { prepareRunDir, withApp, sleep, OUT_ABS } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path'; import { execSync } from 'node:child_process';
const R = recorder('D3');
const run = prepareRunDir('D3b', { portable: true, seedDb: path.resolve('../.tools/golden/q-plans.db') });
const db = path.join(run.dataDir, 'exam-panel.db'); const bdir = path.join(run.dataDir, 'backups');
const tmpl = path.join(run.dir, 'out/q-filled.xlsx'); fs.copyFileSync(path.join(OUT_ABS, 'extracts/B2-q-filled.xlsx'), tmpl); execSync(`chown -R qauser:qauser ${run.dir}`);
const list = () => (fs.existsSync(bdir) ? fs.readdirSync(bdir) : []).map((f) => ({ f, ts: +f.match(/(\d+)\.db$/)[1], reason: f.match(/backup-(.*)-\d+\.db/)[1] }));
const log = [];
await withApp({ scenario: 'D3b', run }, async (app) => {
  const snap = path.join(run.dir, 'out/snap.db'); const r0 = await app.invoke('backup_database', { targetPath: snap }); if (!r0.ok) throw new Error('backup ' + JSON.stringify(r0.e));
  // same-second collision
  await app.invoke('restore_database', { sourcePath: snap }); await app.invoke('restore_database', { sourcePath: snap });
  log.push({ step: 'two restores within <1 s', files: list().length, reasons: list().map((x) => x.reason) });
  for (let i = 0; i < 6; i++) { await sleep(1100); const r = await app.invoke('restore_database', { sourcePath: snap }); if (!r.ok) throw new Error(JSON.stringify(r.e)); }
  log.push({ step: 'after 1 + 6 spaced restores (pre-restore)', n: list().length });
  for (let i = 0; i < 9; i++) { await sleep(1100); const pv = await app.invoke('preview_import', { schoolYearId: 1, filePath: tmpl, mode: 'upsert' }); if (!pv.ok) throw new Error('preview ' + JSON.stringify(pv.e)); const ap = await app.invoke('apply_import', { schoolYearId: 1, preview: pv.v }); if (!ap.ok) throw new Error('apply ' + JSON.stringify(ap.e)); }
  const L = list().sort((a, b) => a.ts - b.ts || a.f.localeCompare(b.f));
  log.push({ step: 'after 9 pre-import', n: L.length, files: L.map((x) => `${x.reason}@${x.ts}`) });
  const lst = await app.invoke('list_backups', {}); log.push({ step: 'list_backups order (UI order)', rows: lst.v.map((x) => x.filename.replace('exampanel-backup-', '')) });
  // what a "keep the newest 10" policy would keep
  const newest10 = L.slice(-10).map((x) => `${x.reason}@${x.ts}`); log.push({ step: 'expected newest 10 by time (from creation log is unavailable once pruned)', note: 'compare kept set vs time order' });
  fs.writeFileSync(path.join(OUT_ABS, 'extracts/D3b-rotation.json'), JSON.stringify(log, null, 1));
  const kept = list(); const reasons = kept.reduce((m, x) => ((m[x.reason] = (m[x.reason] || 0) + 1), m), {});
  R.check('D3.auto-backups-rotate-to-10', kept.length <= 10, { files: kept.length, by_reason: reasons }, 'at most 10 automatic backups', ['extracts/D3b-rotation.json']);
  // newest created = the 9 pre-import (all newer than the pre-restore ones): keeping newest 10 => 9 pre-import + 1 pre-restore
  const imp = reasons['pre-import'] || 0, res = reasons['pre-restore'] || 0;
  R.check('D3.rotation-keeps-the-NEWEST-10', imp === 9 && res === 1, { kept_pre_import: imp, kept_pre_restore: res, expected: '9 pre-import (newest) + 1 pre-restore', ordering_in_list_backups: lst.v.slice(0, 4).map((x) => x.filename) }, 'rotation by time', ['extracts/D3b-rotation.json']);
  const first = lst.v[0].filename; const newestFile = kept.sort((a, b) => b.ts - a.ts)[0].f;
  R.check('D3.list-backups-newest-first', first === newestFile, { first_in_list: first, truly_newest: newestFile }, 'list sorted newest first (UI says so)', ['extracts/D3b-rotation.json']);
  const sameSecond = log[0].files; R.check('D3.same-second-backups-not-overwritten', sameSecond === 2, { files_after_two_back_to_back_restores: sameSecond }, 'two restores within one second yield two pre-restore backups', ['extracts/D3b-rotation.json']);
});
