"""Score exact LSIF declaration positions after the source-only CLI has exited."""

from collections import Counter, defaultdict
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import sys
from urllib.parse import unquote, urlparse

from prepare import CRATES

HERE = Path(__file__).resolve().parent
BIN = HERE.parents[1] / "target/release/learned-binding"


def lookup_real_definition_sites(crate, dump, probes):
    docs, ranges, owner, nxt, defs, items = {}, {}, {}, {}, {}, defaultdict(list)
    with gzip.open(dump, "rt") as stream:
        for line in stream:
            r = json.loads(line)
            tag = r["label"]
            if tag == "document":
                path = Path(unquote(urlparse(r["uri"]).path)).resolve()
                docs[r["id"]] = (
                    path.relative_to(crate).as_posix()
                    if path.is_relative_to(crate)
                    else path.as_posix()
                )
            elif tag == "range":
                ranges[r["id"]] = (r["start"]["line"], r["start"]["character"])
            elif tag == "contains" and r["outV"] in docs:
                owner.update((v, r["outV"]) for v in r["inVs"])
            elif tag == "next":
                nxt[r["outV"]] = r["inV"]
            elif tag == "textDocument/definition":
                defs[r["outV"]] = r["inV"]
            elif tag == "item":
                items[r["outV"]].extend(r["inVs"])
    positions = defaultdict(list)
    for rid, doc in owner.items():
        if rid in ranges:
            positions[(docs[doc], *ranges[rid])].append(rid)
    sites = {}
    for row in probes:
        site = tuple(row["site"])
        targets = set()
        for start in positions[site]:
            v = start
            seen = set()
            while v is not None and v not in defs and v not in seen:
                seen.add(v)
                v = nxt.get(v)
            for t in items.get(defs.get(v), []):
                if t in owner and t in ranges:
                    targets.add((docs[owner[t]], *ranges[t]))
        sites[site] = sorted(targets)
    return sites


def run_measured_crate_sweep(work):
    mac = platform.system() == "Darwin"
    if not mac and platform.system() != "Linux":
        raise RuntimeError("needs macOS or GNU time")
    peaks = {}
    scans = {}
    all_rows = []
    for name in CRATES:
        samples = []
        signature = None
        for _ in range(3):
            p = subprocess.run(
                [
                    "/usr/bin/time",
                    "-l" if mac else "-v",
                    BIN,
                    "real",
                    work / "binding.onnx",
                    work / "meta.json",
                    work / name,
                ],
                check=True,
                capture_output=True,
                env={**os.environ, "LC_ALL": "C"},
            )
            rx = (
                rb"(\d+)\s+maximum resident set size"
                if mac
                else rb"Maximum resident set size \(kbytes\):\s*(\d+)"
            )
            peak = int(re.search(rx, p.stderr)[1]) * (1 if mac else 1024)
            samples.append(peak)
            current = hashlib.sha256(p.stdout).digest()
            assert signature is None or current == signature
            signature = current
        rows = [json.loads(l) for l in p.stdout.splitlines()]
        scans[name] = next(
            json.loads(l) for l in p.stderr.splitlines() if l.startswith(b"{")
        )
        assert (
            len(rows) == scans[name]["eligible"] and not scans[name]["parse_failures"]
        )
        peaks[name] = samples
        sites = lookup_real_definition_sites(
            (work / name).resolve(), work / (name + ".lsif.gz"), rows
        )
        for r in rows:
            ts = sites[tuple(r["site"])]
            # Retain exact in-crate locations; omit machine-specific absolute dependency paths.
            r["oracle"] = [
                (Path(f).name if Path(f).is_absolute() else f, l, c) for f, l, c in ts
            ]
            r["oracle_status"] = (
                "missing"
                if not ts
                else "multiple"
                if len(ts) > 1
                else "in_pair"
                if list(ts[0]) in r["candidates"]
                else "outside_pair"
            )
            for mode in ["fused", "lexical", "encoder", "rule", "local_rule"]:
                ix = r[mode]
                r[mode + "_outcome"] = (
                    "abstain"
                    if ix is None
                    else "unverified"
                    if len(ts) != 1
                    else "confirmed"
                    if tuple(r["candidates"][ix]) == ts[0]
                    else "wrong"
                )
        all_rows.extend(rows)

    def count_mode_outcome_totals(rows, mode):
        return dict(Counter(r[mode + "_outcome"] for r in rows))

    summary = {}
    for name in [*CRATES, "ALL"]:
        scope = [r for r in all_rows if name == "ALL" or r["crate"] == name]
        fallback = [r for r in scope if r["rule"] is None]
        strict = [r for r in scope if r["local_rule"] is None]
        summary[name] = dict(
            eligible=len(scope),
            oracle=dict(Counter(r["oracle_status"] for r in scope)),
            rules=count_mode_outcome_totals(scope, "rule"),
            local_rules=count_mode_outcome_totals(scope, "local_rule"),
            on_rule_abstentions={
                m: count_mode_outcome_totals(fallback, m)
                for m in ["fused", "lexical", "encoder"]
            },
            on_local_rule_abstentions=count_mode_outcome_totals(strict, "fused"),
            confirmed_only_with_encoder=sum(
                r["fused_outcome"] == "confirmed"
                and r["lexical_outcome"] != "confirmed"
                for r in fallback
            ),
        )
    report = dict(
        model_bytes=(work / "binding.onnx").stat().st_size,
        model_sha256=hashlib.sha256((work / "binding.onnx").read_bytes()).hexdigest(),
        platform=platform.platform(),
        rust_analyzer=subprocess.check_output(
            ["rust-analyzer", "--version"], text=True
        ).strip(),
        sha256_crates=json.loads((work / "archives.json").read_text()),
        scans=scans,
        peak_rss_bytes=peaks,
        metrics=summary,
    )
    (HERE / "real-results.json").write_text(json.dumps(report, indent=2) + "\n")
    (HERE / "real-predictions.jsonl").write_text(
        "".join(json.dumps(r) + "\n" for r in all_rows)
    )
    print(json.dumps(summary, indent=2))
    print("peak bytes", peaks)
    if max(map(max, peaks.values())) >= 100_000_000:
        raise SystemExit("RSS gate: FAIL")
    print("RSS gate: PASS")


if __name__ == "__main__":
    run_measured_crate_sweep(Path(sys.argv[1]).resolve())
