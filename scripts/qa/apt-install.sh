#!/bin/bash
# Installs system deps for the real-app QA run (logged by caller into setup.log)
export DEBIAN_FRONTEND=noninteractive
apt-get install -y build-essential curl wget file libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev xvfb openbox dbus dbus-x11 xdotool xclip scrot imagemagick webkit2gtk-driver poppler-utils python3-venv fonts-noto-core fonts-noto-ui-core fonts-dejavu-core x11-utils xdg-utils sqlite3 pkg-config
