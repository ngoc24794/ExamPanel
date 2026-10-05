#!/bin/bash
# Prints the environment report (run from repo root).
cd "$(dirname "$0")/../.."
echo "date: $(date -u +%FT%TZ)"
echo "HEAD: $(git rev-parse HEAD)"; git log -5 --oneline
echo "--- os"; . /etc/os-release; echo "$PRETTY_NAME"; uname -a
echo "--- cpu"; lscpu | grep -E "Model name|^CPU\(s\)"
echo "--- ram"; free -m | head -2
echo "--- disk"; df -h . | tail -1
echo "--- whoami"; whoami; echo "run user: $(id qauser)"
echo "--- network (HTTP code through proxy)"; cat "$(cat .tools/D)/logs/net.txt"
echo "--- tools"; echo "node $(node -v)"; echo "pnpm $(pnpm -v)"; rustc -V; cargo -V; python3 --version
cargo install --list --root .tools/cargo-tools 2>&1 | head -3; echo "gdb: $(gdb --version | head -1)"; echo "poppler: $(pdftotext -v 2>&1 | head -1)"; echo "imagemagick: $(import -version | head -1)"; echo "xdotool: $(dpkg-query -W -f='${Version}' xdotool)"; echo "playwright chromium used for mock suite + print replay: /opt/pw-browsers/chromium-1194"
echo "WebKitGTK: $(dpkg-query -W -f='${Version}' libwebkit2gtk-4.1-0)"; echo "webkit2gtk-driver: $(dpkg-query -W -f='${Version}' webkit2gtk-driver)"
echo "xvfb: $(dpkg-query -W -f='${Version}' xvfb)"; echo "openbox: $(dpkg-query -W -f='${Version}' openbox)"
echo "chromium (playwright preinstalled): $(ls /opt/pw-browsers)"
echo "--- fonts with Vietnamese coverage (fc-list :lang=vi, count)"; fc-list :lang=vi | wc -l; fc-list :lang=vi family | sort -u | head -8
echo "--- env for app runs"; echo "DISPLAY=:99 WEBKIT_DISABLE_COMPOSITING_MODE=1 LIBGL_ALWAYS_SOFTWARE=1 NO_AT_BRIDGE=1 (set by e2e-real/lib/harness.mjs)"
