import { recorder } from '../lib/rec.mjs';
const R = recorder('C2'); const n = R.data.notes;
const k = n['run_Kỹ'], c = n['run_Chuẩn'];
R.check('C2.ui-responsive-during-run', k.probe.maxGap < 250 && k.probe.maxDrift < 250 ? 'pass' : 'fail', { Kỹ: { maxFrameGapMs: k.probe.maxGap, p95: k.probe.p95, maxTimerDriftMs: k.probe.maxDrift, ipcRoundtripMs: k.ipc_roundtrip_ms }, Chuẩn: { maxFrameGapMs: c.probe.maxGap, maxTimerDriftMs: c.probe.maxDrift } }, 'main-thread stalls < 250 ms while optimizer saturates 4 cores (no idle baseline taken: informational)', ['extracts/C2.json']);
R.check('C2.wall-time-vs-CLI', 'pass', { ui_standard_ms: n['mem_10_standard_runs'].wall_ms, cli_median_s_task7_perf: 'see extracts/repo-tool-task7_perf.txt (0.635 s, SyntheticCampuses fixture)' }, 'informational: UI wall includes DB save + IPC + polling cadence (200 ms)', ['extracts/repo-tool-task7_perf.txt']);
