#!/bin/bash
# usage: scripts/qa/run-seq.sh <logdir> <scenario.mjs>...  -- runs scenarios one after another (background-friendly), marker file <logdir>/ALL.done
cd "$(dirname "$0")/../../e2e-real" || exit 1
L="$1"; shift; mkdir -p "$L"; rm -f "$L"/ALL.done
for s in "$@"; do timeout "${QA_TIMEOUT:-700}" node "scenarios/$s" > "$L/$s.log" 2>&1; echo "EXIT=$?" >> "$L/$s.log"; pkill -u qauser 2>/dev/null; sleep 1; done
touch "$L/ALL.done"
