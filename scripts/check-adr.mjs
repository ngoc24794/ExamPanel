import fs from 'node:fs';

const content = fs.readFileSync('docs/DECISIONS.md', 'utf8');
const regex = /^## ADR-(\d+):/gm;
let match;
const numbers = [];
while ((match = regex.exec(content)) !== null) {
  numbers.push(parseInt(match[1], 10));
}

if (numbers.length === 0) {
  console.error('No ADR headers found in docs/DECISIONS.md');
  process.exit(1);
}

const errors = [];
for (let i = 0; i < numbers.length; i++) {
  const expected = i + 1;
  const actual = numbers[i];
  if (actual !== expected) {
    errors.push(
      `ADR numbering mismatch at index ${i}: expected ADR-${String(expected).padStart(4, '0')}, found ADR-${String(actual).padStart(4, '0')}`
    );
  }
}

if (errors.length > 0) {
  console.error('ADR validation failed:');
  for (const err of errors) {
    console.error(' - ' + err);
  }
  process.exit(1);
}

console.log(`✓ All ${numbers.length} ADRs are strictly unique and sequential (ADR-0001 to ADR-${String(numbers.length).padStart(4, '0')}).`);
