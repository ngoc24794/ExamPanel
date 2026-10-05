#!/bin/bash
# Start/stop the virtual display stack: Xvfb :99 (1920x1080x24) + openbox.
# Usage: scripts/qa/stack.sh start|stop|status
case "$1" in
start)
  pgrep -x Xvfb >/dev/null || (nohup Xvfb :99 -screen 0 1920x1080x24 -nolisten tcp >/tmp/qa-xvfb.log 2>&1 &)
  for i in $(seq 1 20); do DISPLAY=:99 xdpyinfo >/dev/null 2>&1 && break; sleep 0.5; done
  pgrep -x openbox >/dev/null || (DISPLAY=:99 nohup openbox >/tmp/qa-openbox.log 2>&1 &)
  sleep 1; DISPLAY=:99 xdpyinfo | grep -E "dimensions|depth of root" ;;
stop)
  pkill -x openbox; pkill -x Xvfb; echo stopped ;;
status) pgrep -a "Xvfb|openbox" ;;
esac
