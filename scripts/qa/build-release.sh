#!/bin/bash
# Real Linux release build exactly as docs/TECH.md describes (no bundle). Output captured by caller.
cd "$(dirname "$0")/../.."
start=$(date +%s)
pnpm tauri build --no-bundle
rc=$?
echo "BUILD_EXIT=$rc DURATION_S=$(( $(date +%s)-start ))"
