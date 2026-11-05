"""Measure fresh release CLI processes, never the compiler or this Python driver."""
import json
import os
import platform
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path


def measure_fresh_cpu_process(engine, fixture):
    system = platform.system()
    if system == "Darwin":
        flag, pattern, factor = "-l", r"(\d+)\s+maximum resident set size", 1
    elif system == "Linux":
        flag, pattern, factor = "-v", r"Maximum resident set size \(kbytes\):\s*(\d+)", 1024
    else:
        raise SystemExit("requires macOS time or GNU time on Linux")
    binary = Path(__file__).resolve().parent / "target/release/tiny-reexport"
    start = time.perf_counter()
    proc = subprocess.run(["/usr/bin/time", flag, binary, engine, fixture],
                          capture_output=True, env={**os.environ, "LC_ALL": "C"})
    elapsed = 1000 * (time.perf_counter() - start)
    if proc.returncode:
        raise SystemExit(proc.stderr.decode())
    rss = re.search(pattern, proc.stderr.decode())
    if not rss:
        raise SystemExit("could not read peak RSS from time")
    return proc.stdout, int(rss.group(1)) * factor, round(elapsed, 2)


if __name__ == "__main__":
    fixture = (Path(sys.argv[1]) if len(sys.argv) > 1 else
               Path(__file__).resolve().parent / "fixtures/round.rs")
    rss = {"native": [], "model": []}
    wall = {"native": [], "model": []}
    expected = None
    for run in range(5):
        for engine in (["native", "model"] if run % 2 == 0 else ["model", "native"]):
            graph, peak, ms = measure_fresh_cpu_process(engine, fixture)
            if expected is None:
                expected = graph
            if graph != expected:
                raise SystemExit(f"graph mismatch: {engine}, trial {run + 1}")
            rss[engine].append(peak)
            wall[engine].append(ms)
    print(json.dumps({"platform": platform.platform(), "fixture": fixture.name,
                      "graph": json.loads(expected), "peak_rss_bytes": rss,
                      "median_wall_ms_including_time_wrapper": {
                          k: statistics.median(v) for k, v in wall.items()}}, indent=2))
    if max(rss["model"]) >= 100_000_000:
        raise SystemExit("model failed the decimal 100 MB peak RSS gate")
