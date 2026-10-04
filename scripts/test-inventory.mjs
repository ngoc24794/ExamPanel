import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Parse raw `cargo test` output into structured test binary results.
 */
export function parseCargoTestOutput(rawOutput) {
  const lines = rawOutput.split(/\r?\n/);
  const binaries = [];
  let currentBinary = null;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // e.g. "     Running unittests src\lib.rs (target\debug\deps\exam_panel_core-f9cfb79fd29a0d4f.exe)"
    // or   "     Running tests\excel_tests.rs (target\debug\deps\excel_tests-0d4852aa5960ab72.exe)"
    // or   "     Running unittests src\bin\task5_comparison.rs (target\debug\deps\task5_comparison-xxx.exe)"
    const runningMatch = line.match(/^\s*Running\s+(.+?)\s+\((.+?)\)$/);
    if (runningMatch) {
      const sourcePath = runningMatch[1].replace(/\\/g, '/');
      const binPath = runningMatch[2].replace(/\\/g, '/');
      const binFilename = path.basename(binPath);

      // Determine crate and binary kind
      let crateName = 'unknown';
      let binaryName = sourcePath;

      if (binFilename.startsWith('exam_panel_core')) {
        crateName = 'exam-panel-core';
      } else if (binFilename.startsWith('exam_panel_storage')) {
        crateName = 'exam-panel-storage';
      } else if (binFilename.startsWith('exam_panel_service')) {
        crateName = 'exam-panel-service';
      } else if (binFilename.startsWith('exam_panel_app')) {
        crateName = 'exam-panel-app';
      } else {
        // Integration tests or bins might have filename like excel_tests-<hash>.exe
        // Infer from known binaries
        if (
          binFilename.startsWith('excel_tests') ||
          binFilename.startsWith('part_b_tests') ||
          binFilename.startsWith('pipeline_report_test') ||
          binFilename.startsWith('service_tests') ||
          binFilename.startsWith('trial_mode_tests') ||
          binFilename.startsWith('generate_types') ||
          binFilename.startsWith('import_q_data') ||
          binFilename.startsWith('phase11_report') ||
          binFilename.startsWith('task5_comparison') ||
          binFilename.startsWith('task6_h3') ||
          binFilename.startsWith('task7_perf') ||
          binFilename.startsWith('generate_sample_excels')
        ) {
          crateName = 'exam-panel-service';
        } else {
          crateName = 'other';
        }
      }

      currentBinary = {
        crate: crateName,
        source: sourcePath,
        bin: binFilename,
        passed: 0,
        failed: 0,
        ignored: 0,
        measured: 0,
        finished: false,
      };
      binaries.push(currentBinary);
      continue;
    }

    // e.g. "test result: ok. 75 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 38.00s"
    const resultMatch = line.match(
      /test result: (\w+)\.\s+(\d+)\s+passed;\s+(\d+)\s+failed;\s+(\d+)\s+ignored;/
    );
    if (resultMatch && currentBinary) {
      currentBinary.passed = parseInt(resultMatch[2], 10);
      currentBinary.failed = parseInt(resultMatch[3], 10);
      currentBinary.ignored = parseInt(resultMatch[4], 10);
      currentBinary.status = resultMatch[1];
      currentBinary.finished = true;
      currentBinary = null;
    }
  }

  return binaries;
}

/**
 * Format parsed binaries into inventory markdown/text table.
 */
export function formatInventory(title, binaries) {
  const crateMap = new Map();

  for (const b of binaries) {
    if (!crateMap.has(b.crate)) {
      crateMap.set(b.crate, { passed: 0, failed: 0, ignored: 0, binaries: [] });
    }
    const c = crateMap.get(b.crate);
    c.passed += b.passed;
    c.failed += b.failed;
    c.ignored += b.ignored;
    c.binaries.push(b);
  }

  let out = `## ${title}\n\n`;
  out += `### Crate Summary\n\n`;
  out += `| Crate | Passed | Failed | Ignored | Total Tests |\n`;
  out += `| :--- | :---: | :---: | :---: | :---: |\n`;

  let totalPassed = 0;
  let totalFailed = 0;
  let totalIgnored = 0;

  for (const [crate, c] of crateMap.entries()) {
    const total = c.passed + c.failed + c.ignored;
    totalPassed += c.passed;
    totalFailed += c.failed;
    totalIgnored += c.ignored;
    out += `| \`${crate}\` | ${c.passed} | ${c.failed} | ${c.ignored} | ${total} |\n`;
  }
  const grandTotal = totalPassed + totalFailed + totalIgnored;
  out += `| **Total** | **${totalPassed}** | **${totalFailed}** | **${totalIgnored}** | **${grandTotal}** |\n\n`;

  out += `### Per-Test-Binary Breakdown\n\n`;
  out += `| Crate | Binary Target | Passed | Failed | Ignored |\n`;
  out += `| :--- | :--- | :---: | :---: | :---: |\n`;

  for (const [crate, c] of crateMap.entries()) {
    for (const b of c.binaries) {
      out += `| \`${crate}\` | \`${b.source}\` | ${b.passed} | ${b.failed} | ${b.ignored} |\n`;
    }
  }

  out += `\n`;
  return { out, totalPassed, totalFailed, totalIgnored, crateMap };
}

// CLI execution
if (process.argv[1] && process.argv[1].endsWith('test-inventory.mjs')) {
  const inputFile = process.argv[2];
  let rawOutput = '';
  if (inputFile && fs.existsSync(inputFile)) {
    rawOutput = fs.readFileSync(inputFile, 'utf-8');
  } else {
    console.log('Running `cargo test --workspace` to capture current test inventory...');
    rawOutput = execSync('cargo test --workspace', { encoding: 'utf-8', maxBuffer: 10 * 1024 * 1024 });
  }

  const binaries = parseCargoTestOutput(rawOutput);
  const formatted = formatInventory('Current HEAD Test Inventory', binaries);
  console.log(formatted.out);
}
