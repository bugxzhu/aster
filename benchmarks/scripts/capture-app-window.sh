#!/usr/bin/env bash
# Capture one on-screen window of a running process (macOS).
# Requires Screen Recording permission for the terminal application
# (System Settings > Privacy & Security > Screen Recording).
# Usage: capture-app-window.sh <pid> <output.png>
set -euo pipefail
pid="$1"; out="$2"
wid=$(python3 - "$pid" <<'PY'
import sys, Quartz
pid = int(sys.argv[1])
wins = Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionOnScreenOnly, Quartz.kCGNullWindowID)
for w in wins:
    if w.get("kCGWindowOwnerPID") == pid and w.get("kCGWindowLayer") == 0:
        print(w.get("kCGWindowNumber"))
        break
PY
)
[ -n "$wid" ] || { echo "no on-screen window for pid $pid" >&2; exit 2; }
screencapture -x -o -l "$wid" "$out"
echo "captured window $wid -> $out"
