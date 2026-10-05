// Append a finding (JSON on stdin) to scripts/qa/findings-src.json; assigns RA-NNN ids.
import fs from 'node:fs';
const f = new URL('./findings-src.json', import.meta.url).pathname;
const list = fs.existsSync(f) ? JSON.parse(fs.readFileSync(f, 'utf8')) : [];
const inp = JSON.parse(fs.readFileSync(0, 'utf8'));
const items = Array.isArray(inp) ? inp : [inp];
for (const it of items) { it.id = 'RA-' + String(list.length + 1).padStart(3, '0'); list.push(it); console.log('added', it.id, it.title); }
fs.writeFileSync(f, JSON.stringify(list, null, 1));
