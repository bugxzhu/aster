#!/usr/bin/env python3
"""Measure startup of a binary that reports timing with ASTER_TIMING=1.

Definitions (see benchmarks/results/SCHEMA.md, section "Timing caliber"):
  process_to_main_ms      spawn (parent) -> MAIN_START (child main entry)
  window_create_ms        MAIN_START -> WINDOW_READY (window and UI created, before event loop)
  first_frame_ms          WINDOW_READY -> FIRST_FRAME (first frame rendered)
  total_ms                spawn -> FIRST_FRAME (what a user waits for)

Usage: measure-startup.py <binary> [runs] [-- extra args...]
"""
import json
import os
import subprocess
import sys
import time

def one_run(binary, extra):
    env = dict(os.environ, ASTER_TIMING="1")
    spawn_us = time.time_ns() // 1000
    proc = subprocess.Popen([binary, *extra], stdout=subprocess.DEVNULL,
                            stderr=subprocess.PIPE, env=env, text=True)
    marks = {}
    deadline = time.time() + 15
    try:
        for line in proc.stderr:
            if "=" in line and line.split("=")[0] in ("MAIN_START_US", "WINDOW_READY_US", "FIRST_FRAME_US"):
                key, value = line.strip().split("=", 1)
                marks[key] = int(value)
            if "FIRST_FRAME_US" in marks or time.time() > deadline:
                break
    finally:
        proc.kill()
        proc.wait()
    if not {"MAIN_START_US", "WINDOW_READY_US", "FIRST_FRAME_US"} <= marks.keys():
        raise RuntimeError(f"incomplete timing marks: {marks}")
    m, w, f = marks["MAIN_START_US"], marks["WINDOW_READY_US"], marks["FIRST_FRAME_US"]
    return {
        "process_to_main_ms": (m - spawn_us) / 1000,
        "window_create_ms": (w - m) / 1000,
        "first_frame_ms": (f - w) / 1000,
        "total_ms": (f - spawn_us) / 1000,
    }

def summarize(values):
    values = sorted(values)
    n = len(values)
    median = values[n // 2] if n % 2 else (values[n // 2 - 1] + values[n // 2]) / 2
    return {"min": round(values[0], 1), "median": round(median, 1), "max": round(values[-1], 1)}

def main():
    if len(sys.argv) < 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    binary = sys.argv[1]
    runs = int(sys.argv[2]) if len(sys.argv) > 2 and sys.argv[2].isdigit() else 10
    extra = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    samples = []
    for i in range(runs):
        samples.append(one_run(binary, extra))
        print(f"run {i + 1}/{runs}: total={samples[-1]['total_ms']:.1f} ms", file=sys.stderr)
    keys = samples[0].keys()
    result = {key: {"runs": [round(s[key], 1) for s in samples], **summarize([s[key] for s in samples])}
              for key in keys}
    print(json.dumps({"binary": binary, "args": extra, "runs": runs, "timing_ms": result}, indent=2))

if __name__ == "__main__":
    main()
