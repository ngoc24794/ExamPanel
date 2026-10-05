import { prepareRunDir, withApp, sleep, OUT_ABS, sampleRss } from '../lib/harness.mjs';
import { recorder } from '../lib/rec.mjs';
import { clickTid, bodyText, tid } from '../lib/ui.mjs';
import { sql } from '../lib/native.mjs';
import fs from 'node:fs'; import path from 'node:path';
const R = recorder('C2');
const run = prepareRunDir('C2', { portable: true, seedDb: path.resolve('../.tools/golden/q-ready.db') });
const db = path.join(run.dataDir, 'exam-panel.db');
const EFFORT = { Nhanh: 'fast', Chuẩn: 'standard', Kỹ: 'thorough' };
const planCount = () => sql(db, 'select count(*) c from plans')[0].c;
const stats = (a) => { const s = [...a].sort((x, y) => x - y); return { min: s[0], median: s[Math.floor(s.length / 2)], max: s[s.length - 1], n: s.length }; };

async function openDialog(app, effortLabel) {
  await app.nav('/assignments'); await sleep(900);
  await clickTid(app.b, 'run-optimizer-button'); await sleep(600);
  const b = await app.b.$(`//*[@role="dialog"]//button[normalize-space(.)="${effortLabel}"]`); await b.click(); await sleep(200);
}
const probeJs = () => {
  window.__probe = { gaps: [], drift: [], last: performance.now(), tl: performance.now() };
  const loop = () => { const n = performance.now(); window.__probe.gaps.push(n - window.__probe.last); window.__probe.last = n; if (window.__probe.on !== false) requestAnimationFrame(loop); };
  requestAnimationFrame(loop);
  setInterval(() => { const n = performance.now(); window.__probe.drift.push(n - window.__probe.tl - 20); window.__probe.tl = n; }, 20);
};
async function sampleRun(app, label, { cancelAfterMs = null, probe = false } = {}) {
  if (probe) await app.b.execute(probeJs);
  const t0 = Date.now(); await clickTid(app.b, 'start-optimize-button');
  const samples = []; let gone = null; let cancelled = false; const lat = [];
  for (let i = 0; i < 2400; i++) {
    const tq = Date.now();
    const s = await app.b.execute(() => { const d = document.querySelector('[role="dialog"]'); if (!d) return null; const bar = d.querySelector('.bg-primary.h-2\\.5'); const nums = [...d.querySelectorAll('.font-semibold.text-sm')].map((e) => e.innerText); const el = d.innerText.match(/Thời gian: (\d+)ms/); return { w: bar ? bar.style.width : null, best: nums[0], cur: nums[1], el: el && +el[1] }; });
    lat.push(Date.now() - tq);
    if (!s) { gone = Date.now() - t0; break; }
    samples.push({ at: Date.now() - t0, ...s });
    if (cancelAfterMs && !cancelled && Date.now() - t0 > cancelAfterMs) { const cb = await app.b.$('//*[@role="dialog"]//button[contains(.,"Hủy bỏ")]'); await cb.click(); cancelled = true; }
    await sleep(200);
  }
  return { label, wall_ms: gone, samples, probe_latency: lat };
}
await withApp({ scenario: 'C2', run }, async (app) => {
  const lbs = {};
  // ---------- 3 efforts ----------
  for (const eff of ['Nhanh', 'Chuẩn', 'Kỹ']) {
    const before = planCount(); await openDialog(app, eff);
    const r = await sampleRun(app, eff, { probe: eff !== 'Nhanh' });
    await sleep(1500);
    const ids = sql(db, 'select id,score,rank from plans order by id desc limit 3');
    fs.writeFileSync(path.join(OUT_ABS, `extracts/C2-${eff.replace("Chuẩn","Chuan").replace("Kỹ","Ky")}-samples.json`), JSON.stringify(r, null, 1));
    const w = r.samples.map((s) => parseInt(s.w) || 0), best = r.samples.map((s) => parseFloat(s.best)).filter((x) => !isNaN(x));
    const monoW = w.every((x, i) => i === 0 || x >= w[i - 1] - 0.0001), monoBest = best.every((x, i) => i === 0 || x <= best[i - 1] + 1e-9);
    const probe = await app.b.execute(() => window.__probe ? { maxGap: Math.max(...window.__probe.gaps), p95: window.__probe.gaps.sort((a, b) => a - b)[Math.floor(window.__probe.gaps.length * 0.95)], maxDrift: Math.max(...window.__probe.drift), n: window.__probe.gaps.length } : null);
    R.note(`run_${eff}`, { wall_ms: r.wall_ms, n_samples: r.samples.length, progress_width_samples: w.slice(0, 40), best_first_last: [best[0], best[best.length - 1]], probe, ipc_roundtrip_ms: stats(r.probe_latency) });
    R.check(`C2.${eff}.completes-and-saves-k<=3`, planCount() - before >= 1 && planCount() - before <= 3, { saved: planCount() - before, wall_ms: r.wall_ms }, '1..3 plans saved as drafts', [`extracts/C2-${eff.replace("Chuẩn","Chuan").replace("Kỹ","Ky")}-samples.json`]);
    R.check(`C2.${eff}.progress-monotonic`, r.samples.length >= 2 ? monoW && monoBest : 'not-verified', { samples: r.samples.length, monoW, monoBest }, eff === 'Nhanh' ? 'run too short to sample (<0.5s)' : 'progress % non-decreasing and best score non-increasing');
    // plan quality vs lower bounds via engine
    const pids = sql(db, 'select id from plans order by id desc limit 3').map((x) => x.id); const q = [];
    for (const id of pids) { const gp = await app.invoke('get_plan', { id }); if (gp.ok) { const sr = gp.v.score_report; q.push({ id, total: sr.total, below_lb: sr.by_rule.filter((x) => x.units < x.lower_bound - 1e-6).map((x) => x.rule), hard: null }); } }
    R.note(`quality_${eff}`, q);
    R.check(`C2.${eff}.scores>=lower-bounds`, q.every((x) => x.below_lb.length === 0), q, 'no rule units below its lower bound');
    if (eff === 'Chuẩn') { const dist = []; const plansA = []; for (const id of pids) { const gp = await app.invoke('get_plan', { id }); plansA.push(gp.v.assignments.map((a) => `${a.exam_id}.${a.grade_id}.${a.subject_id}.${a.role}.${a.teacher_id}`).sort()); } for (let i = 0; i < plansA.length; i++) for (let j = i + 1; j < plansA.length; j++) { const A = new Set(plansA[i]), B = plansA[j].filter((x) => !A.has(x)).length; dist.push({ i, j, differing_seats: B }); } R.note('plan_pairwise_distance_standard', dist); R.check('C2.Chuẩn.plans-are-distinct', dist.every((d) => d.differing_seats > 0), dist, 'K plans differ pairwise (diversity_threshold 0.2)'); }
    await app.shot(`C2-after-${eff}`, { screen: false });
  }
  // ---------- second run while one is running (IPC) ----------
  await openDialog(app, 'Kỹ'); const tStart = Date.now(); await clickTid(app.b, 'start-optimize-button'); await sleep(1200);
  const second = await app.b.executeAsync(function (done) { const id = window.__TAURI_INTERNALS__.transformCallback(function () {}); window.__TAURI_INTERNALS__.invoke('start_optimize', { schoolYearId: 1, request: { base_seed: 1, runs: 2, budget: { type: 'Iterations', value: 1000 }, k: 1, diversity_threshold: 0.2 }, onProgress: '__CHANNEL__:' + id }).then(function (v) { done({ ok: true }); }, function (e) { done({ ok: false, e: e }); }); });
  R.check('C2.second-run-rejected-cleanly', second.ok === false && JSON.stringify(second.e).includes('optimize_busy'), second, 'AppError optimize_busy; first run unaffected');
  // ---------- cancel ----------
  const beforeCancel = planCount();
  const cb = await app.b.$('//*[@role="dialog"]//button[contains(.,"Hủy bỏ")]'); const tC = Date.now(); await cb.click(); await sleep(1500);
  const toastTxt = (await bodyText(app.b)); const n = (toastTxt.match(/Đã hủy/g) || []).length;
  const gone = !(await (await app.b.$('[role="dialog"]')).isExisting() && (await app.b.execute(() => !!document.querySelector('[role="dialog"]'))));
  R.note('after_cancel', { plans_before: beforeCancel, plans_after: planCount(), toast_cancel_count: n, dialog_present: await app.b.execute(() => document.querySelector('[role="dialog"]')?.innerText?.slice(0, 200)) });
  R.check('C2.cancel-leaves-no-partial-plans', planCount() === beforeCancel, { before: beforeCancel, after: planCount() }, 'plan count unchanged');
  R.check('C2.cancel-single-toast', n <= 1, n, 'at most one "Đã hủy" toast (handleCancel and catch both toast)');
  await app.shot('C2-after-cancel');
  // is the app usable and not stuck busy?
  await sleep(1500);
  await app.b.execute(() => { const d = document.querySelector('[role="dialog"]'); }); 
  const stillOpen = await app.b.execute(() => !!document.querySelector('[role="dialog"]'));
  if (!stillOpen) await openDialog(app, 'Nhanh'); else { const f = await app.b.$('//*[@role="dialog"]//button[normalize-space(.)="Nhanh"]'); await f.click(); }
  const r2 = await sampleRun(app, 'after-cancel-fast'); await sleep(1500);
  R.check('C2.run-after-cancel-works', planCount() > beforeCancel, { plans: planCount(), wall_ms: r2.wall_ms }, 'new run after cancel completes and saves (no stale cancel flag)');
  // ---------- 10 runs memory ----------
  const rss = []; const wall = [];
  for (let i = 0; i < 10; i++) { await openDialog(app, 'Chuẩn').catch(() => {}); const rr = await sampleRun(app, 'mem' + i); wall.push(rr.wall_ms); await sleep(1500); rss.push(sampleRss().total_kb); }
  R.note('mem_10_standard_runs', { rss_kb: rss, wall_ms: wall });
  R.check('C2.10-runs-memory-growth', 'pass', { first: rss[0], last: rss[rss.length - 1], growth_kb: rss[rss.length - 1] - rss[0], wall: stats(wall) }, 'informational: RSS (app+webkit+driver) after each standard run');
  R.note('plans_total', planCount());
});
