#!/bin/bash
# usage: scripts/qa/bg.sh <scenario.mjs> <logfile>  -- runs a scenario in the background with a done marker
cd "$(dirname "$0")/../../e2e-real" || exit 1
( timeout "${QA_TIMEOUT:-600}" node "scenarios/$1" > "$2" 2>&1; echo "EXIT=$?" >> "$2"; touch "$2.done" ) &
