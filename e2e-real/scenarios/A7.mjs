import { prepareRunDir, launchApp, waitUiReady, sleep, REPO } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { sql, sha256 } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('A7');
const fixture = path.join(REPO, 'crates/storage/fixtures/v4_synthetic.db');
const run = prepareRunDir('A7', { portable: true, seedDb: fixture });
const db = path.join(run.dataDir, 'exam-panel.db');
const counts = (p) => { const tabs = sql(p, "select name from sqlite_master where type='table' and name not like 'sqlite_%' order by name").map((r) => r.name); const o = {}; for (const t of tabs) o[t] = sql(p, `select count(*) c from "${t}"`)[0].c; return o; };
const before = { user_version: sql(db, 'pragma user_version')[0].user_version, counts: counts(db), sha: sha256(db),
  teachers: sql(db, 'select id,full_name,campus_id,active from teachers order by id'), plans: sql(db, 'select id,name,source,is_final from plans order by id') };
fs.writeFileSync(path.join(run.dir, 'out/before.json'), JSON.stringify(before, null, 1));
const app = await launchApp({ scenario: 'A7', run });
const ready = await waitUiReady(app); await sleep(1500);
const after = { user_version: sql(db, 'pragma user_version')[0].user_version, counts: counts(db),
  teachers: sql(db, 'select id,full_name,campus_id,active from teachers order by id'), plans: sql(db, 'select id,name,source,is_final from plans order by id') };
R.note('before', { user_version: before.user_version, counts: before.counts }); R.note('after', { user_version: after.user_version, counts: after.counts });
R.check('A7.user_version-latest', after.user_version === 5, { before: before.user_version, after: after.user_version }, 'user_version 5 (migrations.rs latest)');
const lost = Object.entries(before.counts).filter(([t, c]) => (after.counts[t] ?? -1) < c);
R.check('A7.rows-preserved', lost.length === 0, lost, 'no table loses rows');
R.check('A7.teachers-identical', JSON.stringify(before.teachers) === JSON.stringify(after.teachers), { n_before: before.teachers.length, n_after: after.teachers.length }, 'identical teacher rows');
R.check('A7.plans-identical', JSON.stringify(before.plans) === JSON.stringify(after.plans), { before: before.plans, after: after.plans }, 'identical plan rows');
const bk = fs.existsSync(path.join(run.dataDir, 'backups')) ? fs.readdirSync(path.join(run.dataDir, 'backups')) : [];
R.check('A7.automatic-pre-migration-backup', bk.length > 0, bk, 'a pre-migration backup in data/backups (task expectation; SPEC/ADR do not promise it)');
// UI shows data
await app.nav('/teachers'); await sleep(1500);
const txt = await app.b.execute(() => document.body.innerText);
fs.writeFileSync(path.join(run.dir, 'out/teachers-page.txt'), txt);
const shown = before.teachers.filter((t) => txt.includes(t.full_name)).length;
R.check('A7.ui-shows-teachers', shown === before.teachers.length, { shown, total: before.teachers.length }, 'all teachers visible', []);
await app.shot('A7-teachers');
await app.nav('/assignments'); await sleep(2000); await app.shot('A7-assignments');
const t2 = await app.b.execute(() => document.body.innerText); fs.writeFileSync(path.join(run.dir, 'out/assignments-page.txt'), t2);
R.note('assignments_text_head', t2.slice(0, 800));
const info = await app.invoke('get_app_info'); R.note('app_info', info.v);
await app.stop();
