#!/bin/bash
# final audit: kill everything the run spawned, prove none remain; product-code diff proof.
cd "$(dirname "$0")/../.." || exit 1
D=$(cat .tools/D)
{
echo "== $(date -u +%FT%TZ) killing spawned processes"
pkill -u qauser 2>/dev/null; pkill -x tauri-driver 2>/dev/null; pkill -x WebKitWebDriver 2>/dev/null; pkill -x openbox 2>/dev/null; pkill -x Xvfb 2>/dev/null; pkill -f "dbus-daemon --session" 2>/dev/null; pkill -x xclip 2>/dev/null
sleep 2
echo "== remaining matching processes (expect none):"
ps -eo pid,user,comm,args | grep -E "ExamPanel|WebKit|tauri-driver|Xvfb|openbox|dbus-daemon|xclip|chrome" | grep -v grep | grep -v "grep -E" || echo "(none)"
echo "== processes owned by qauser (expect none):"; ps -u qauser -o pid,comm,args --no-headers || true; echo "(end)"
echo "== X display :99 (expect not running):"; (DISPLAY=:99 xdpyinfo >/dev/null 2>&1 && echo "STILL UP") || echo "down"
} > "$D/process-cleanup.txt" 2>&1
START=$(cat "$D/start-commit.txt")
{ echo "# git diff --stat $START..HEAD -- crates src-tauri ui/src   (expected: empty)"; git diff --stat "$START"..HEAD -- crates src-tauri ui/src; echo "# (end)"; echo "# working tree vs start commit, same paths:"; git diff --stat "$START" -- crates src-tauri ui/src; echo "# also migrations/locales:"; git diff --stat "$START"..HEAD -- crates/storage/migrations ui/src/i18n/locales; echo "# (end)"; } > "$D/git-diff-product-code.txt" 2>&1
cat "$D/process-cleanup.txt" "$D/git-diff-product-code.txt"
