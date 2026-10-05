// Per-scenario result recorder: writes extracts/<id>.json; matrix.md is generated from these.
import fs from 'node:fs';
import path from 'node:path';
import { OUT_ABS } from './harness.mjs';

export function recorder(id) {
  const file = path.join(OUT_ABS, 'extracts', `${id}.json`);
  fs.mkdirSync(path.dirname(file), { recursive: true });
  let data = { id, started: new Date().toISOString(), checks: [], notes: {} };
  if (fs.existsSync(file)) { try { data = JSON.parse(fs.readFileSync(file, 'utf8')); } catch {} }
  const save = () => fs.writeFileSync(file, JSON.stringify(data, null, 1));
  return {
    data,
    /** result: 'pass'|'fail'|'blocked'|'manual-required'|'not-verified' */
    check(step, result, observed, expected, evidence = []) {
      if (typeof result === 'boolean') result = result ? 'pass' : 'fail';
      data.checks = data.checks.filter((c) => c.step !== step);
      data.checks.push({ step, result, observed, expected, evidence });
      console.log(`[${id}] ${step}: ${result} | observed=${JSON.stringify(observed)?.slice(0, 200)} expected=${JSON.stringify(expected)?.slice(0, 120)}`);
      save();
    },
    note(k, v) { data.notes[k] = v; save(); },
    save,
  };
}
