#!/usr/bin/env python3
"""Adds keys to ui/src/i18n/locales/{vi,en}.json WITHOUT reformatting the files.

Usage: scripts/i18n-add.py a.b.key "vi text" "en text" [more triples...]
Keys are appended as the last entry of their (existing or new) section.
"""
import json, re, sys, os

ROOT = os.path.join(os.path.dirname(__file__), '..', 'ui', 'src', 'i18n', 'locales')


def insert(path, dotted, value):
    text = open(path, encoding='utf-8').read()
    data = json.loads(text)
    parts = dotted.split('.')
    node = data
    for p in parts[:-1]:
        node = node.get(p) if isinstance(node, dict) else None
        if node is None:
            break
    if isinstance(node, dict) and parts[-1] in node:
        return False  # already present
    lines = text.split('\n')
    # find the object that should receive the key: walk the parents
    start = 0
    depth = 0
    for p in parts[:-1]:
        pat = re.compile(r'^' + '  ' * depth + r'  ' + re.escape(json.dumps(p, ensure_ascii=False)) + r': \{\s*$')
        idx = next((i for i in range(start, len(lines)) if pat.match(lines[i])), None)
        if idx is None:
            raise SystemExit(f'{path}: section {p!r} not found at depth {depth}')
        start = idx + 1
        depth += 1
    # closing brace of that object
    close_pat = re.compile(r'^' + '  ' * depth + r'\},?\s*$')
    close = next(i for i in range(start, len(lines)) if close_pat.match(lines[i]))
    prev = close - 1
    if not lines[prev].rstrip().endswith(('{', ',')):
        lines[prev] = lines[prev].rstrip() + ','
    entry = '  ' * (depth + 1) + json.dumps(parts[-1], ensure_ascii=False) + ': ' + json.dumps(value, ensure_ascii=False)
    lines.insert(close, entry)
    new = '\n'.join(lines)
    json.loads(new)  # validate
    open(path, 'w', encoding='utf-8').write(new)
    return True


def main(argv):
    if len(argv) < 3 or (len(argv) - 1) % 3:
        raise SystemExit(__doc__)
    for i in range(1, len(argv), 3):
        key, vi, en = argv[i:i + 3]
        insert(os.path.join(ROOT, 'vi.json'), key, vi)
        insert(os.path.join(ROOT, 'en.json'), key, en)


if __name__ == '__main__':
    main(sys.argv)
