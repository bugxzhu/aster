#!/usr/bin/env python3
"""Idle memory of a GUI application.

Starts the binary, waits until its window has shown its first frame (ASTER_TIMING marks), then
waits IDLE_SECONDS and samples resident memory (RSS) of:
  - the process itself
  - all descendant processes (for example WebKit helper processes of a Tauri app)
Both values are reported; the process-tree total is the fair figure for WebView applications.

Usage: measure-memory.py <binary> [runs] [idle_seconds] [-- extra args...]
"""
import json
import os
import subprocess
import sys
import time

def descendants(pid):
    out = subprocess.run(["ps", "-axo", "pid=,ppid="], capture_output=True, text=True).stdout
    children = {}
    for line in out.splitlines():
        p, pp = line.split()
        children.setdefault(int(pp), []).append(int(p))
    result, stack = [], [pid]
    while stack:
        for c in children.get(stack.pop(), []):
            result.append(c)
            stack.append(c)
    return result

def rss_kb(pids):
    if not pids:
        return 0
    out = subprocess.run(["ps", "-o", "rss=", "-p", ",".join(map(str, pids))],
                         capture_output=True, text=True).stdout
    return sum(int(x) for x in out.split() if x.isdigit())

def one_run(binary, extra, idle):
    env = dict(os.environ, ASTER_TIMING="1")
    proc = subprocess.Popen([binary, *extra], stdout=subprocess.DEVNULL,
                            stderr=subprocess.PIPE, env=env, text=True)
    try:
        deadline = time.time() + 20
        for line in proc.stderr:
            if line.startswith("FIRST_FRAME_US"):
                break
            if time.time() > deadline:
                raise RuntimeError("first frame not reported")
        time.sleep(idle)
        own = rss_kb([proc.pid])
        tree = rss_kb([proc.pid] + descendants(proc.pid))
        return {"process_rss_kb": own, "tree_rss_kb": tree, "descendant_count": len(descendants(proc.pid))}
    finally:
        proc.kill()
        proc.wait()

def main():
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    binary = sys.argv[1]
    runs = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2].isdigit() else 3
    idle = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else 30
    extra = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    samples = []
    for i in range(runs):
        samples.append(one_run(binary, extra, idle))
        print(f"run {i + 1}/{runs}: {samples[-1]}", file=sys.stderr)
    def med(key):
        vals = sorted(s[key] for s in samples)
        n = len(vals)
        return vals[n // 2] if n % 2 else (vals[n // 2 - 1] + vals[n // 2]) / 2
    print(json.dumps({
        "binary": binary, "runs": runs, "idle_seconds": idle, "args": extra,
        "samples": samples,
        "median": {"process_rss_kb": med("process_rss_kb"), "tree_rss_kb": med("tree_rss_kb")},
    }, indent=2))

if __name__ == "__main__":
    main()
