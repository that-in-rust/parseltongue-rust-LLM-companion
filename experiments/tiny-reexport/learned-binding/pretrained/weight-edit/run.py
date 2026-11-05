"""One direct edit to two pretrained embedding rows. No fitting or threshold search."""

from collections import Counter
from hashlib import sha256
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys

import numpy as np
import onnx
from onnx import numpy_helper

HERE = Path(__file__).resolve().parent
BASE = HERE.parent / "model"
sys.path.insert(0, str(HERE.parent))
from check import check_source_binding_controls
from evaluate import BIN, lookup_real_definition_sites
from prepare import CRATES, prepare_real_crate_sources

BASE_SHA = "a1c586649ffb9fdebe56139fd78f79bb5a7586379b432c9538db9d12f1f58dd8"
FRESH = json.loads((HERE / "releases.json").read_text())


def edit_role_embedding_contrast(work):
    source = (BASE / "binding.onnx").read_bytes()
    assert sha256(source).hexdigest() == BASE_SHA
    model = onnx.load_from_string(source)
    weight = next(
        x
        for x in model.graph.initializer
        if x.name == "embeddings.word_embeddings.weight_quantized"
    )
    original = numpy_helper.to_array(weight)
    tokens = json.loads((BASE / "meta.json").read_text())["tokens"]
    ids = [tokens["M"], tokens["O"]]
    a, b = original[ids].astype(float)
    mid, half = (a + b) / 2, (a - b) / 2
    changed = original.copy()
    changed[ids] = np.clip(np.rint([mid + 4 * half, mid - 4 * half]), 0, 255).astype(
        np.uint8
    )
    assert original.dtype == np.uint8 and np.flatnonzero(
        np.any(changed != original, axis=1)
    ).tolist() == sorted(ids)
    # Assert *every other byte* of the protobuf would be identical after undoing the edit.
    old_bytes = weight.raw_data
    weight.raw_data = changed.tobytes()
    onnx.checker.check_model(model)
    dest = work / "binding.onnx"
    onnx.save(model, dest)
    weight.raw_data = old_bytes
    assert model.SerializeToString() == source
    shutil.copyfile(BASE / "meta.json", work / "meta.json")
    return dict(
        base_sha256=BASE_SHA,
        edited_sha256=sha256(dest.read_bytes()).hexdigest(),
        checkpoint_bytes=dest.stat().st_size,
        tensor=weight.name,
        ids=ids,
        changed_uint8_weights=int(np.count_nonzero(original[ids] != changed[ids])),
    )


def measure_rust_candidate_predictions(model, root):
    mac = platform.system() == "Darwin"
    args = [
        "/usr/bin/time",
        "-l" if mac else "-v",
        BIN,
        "real",
        model / "binding.onnx",
        model / "meta.json",
        root,
    ]
    rx = (
        rb"(\d+)\s+maximum resident set size"
        if mac
        else rb"Maximum resident set size \(kbytes\):\s*(\d+)"
    )
    peaks = []
    last = None
    for _ in range(3):
        p = subprocess.run(
            args, capture_output=True, check=True, env={**os.environ, "LC_ALL": "C"}
        )
        peaks.append(int(re.search(rx, p.stderr)[1]) * (1 if mac else 1024))
        assert last is None or last == p.stdout
        last = p.stdout
    info = next(json.loads(s) for s in p.stderr.splitlines() if s.startswith(b"{"))
    rows = [json.loads(s) for s in last.splitlines()]
    assert len(rows) == info["eligible"]
    return rows, peaks, info


def score_single_candidate_outcome(index, row, oracle):
    if index is None:
        return "abstained"
    if len(oracle) != 1:
        return "unverified"
    return "confirmed" if tuple(row["candidates"][index]) == oracle[0] else "wrong"


def summarize_binding_edit_results(rows):
    def count_test_arm_outcomes(arm, scope):
        return dict(Counter(r["outcomes"][arm] for r in scope))

    fallback = [r for r in rows if r["local_rule"] is None]
    return dict(
        eligible=len(rows),
        rule_abstentions=len(fallback),
        oracle=dict(Counter(r["oracle_status"] for r in rows)),
        on_rule_abstentions={
            k: count_test_arm_outcomes(k, fallback)
            for k in ("base", "edit", "gain_only")
        },
        all_sites={
            k: count_test_arm_outcomes(k, rows)
            for k in ("local_rule", "base", "edit", "gain_only")
        },
        new_confirmed=sum(
            r["outcomes"]["base"] != "confirmed"
            and r["outcomes"]["edit"] == "confirmed"
            for r in fallback
        ),
        new_wrong=sum(
            r["outcomes"]["base"] != "wrong" and r["outcomes"]["edit"] == "wrong"
            for r in fallback
        ),
        strict_order_reversals=sum(
            r["base_margin"] * r["edit_margin"] < 0 for r in fallback
        ),
        broken_base_ties=sum(
            r["base_margin"] == 0 and r["edit_margin"] != 0 for r in fallback
        ),
        changed_margins=sum(
            abs(r["base_margin"] - r["edit_margin"]) > 1e-5 for r in fallback
        ),
    )


def run_paired_binding_trials(work, previous=None):
    work.mkdir(exist_ok=True, parents=True)
    patch = edit_role_embedding_contrast(work)
    for model in (BASE, work):
        check_source_binding_controls(model)
    if not (work / "archives.json").exists():
        prepare_real_crate_sources(work, FRESH)
    assert json.loads((work / "archives.json").read_text()) == FRESH
    results, cases, sets = {}, [], {}
    for cohort, parent, crates in [("fresh", work, FRESH)] + (
        [("regression", previous, CRATES)] if previous else []
    ):
        group = []
        for crate in crates:
            root = (parent / crate).resolve()
            before, rss0, scan = measure_rust_candidate_predictions(BASE, root)
            after, rss1, scan1 = measure_rust_candidate_predictions(work, root)
            assert scan == scan1
            gold = lookup_real_definition_sites(
                root, parent / (crate + ".lsif.gz"), before
            )
            rows = []
            for a, b in zip(before, after, strict=True):
                for key in (
                    "site",
                    "candidates",
                    "local_rule",
                    "rule",
                    "path",
                    "modules",
                ):
                    assert a[key] == b[key]
                assert a["margins"][0] == b["margins"][0]
                targets = gold[tuple(a["site"])]
                m = a["margins"][2]
                # Same frozen 0.80 cutoff; this control changes confidence but never rank.
                gain = int(m < 0) if abs(4 * m) >= math.log(4) else None
                picks = dict(
                    base=a["fused"],
                    edit=b["fused"],
                    gain_only=gain,
                    local_rule=a["local_rule"],
                )
                status = (
                    "unverified"
                    if len(targets) != 1
                    else "in_pair"
                    if list(targets[0]) in a["candidates"]
                    else "outside_pair"
                )
                rows.append(
                    dict(
                        cohort=cohort,
                        crate=crate,
                        site=a["site"],
                        candidates=a["candidates"],
                        local_rule=a["local_rule"],
                        base_margin=m,
                        edit_margin=b["margins"][2],
                        picks=picks,
                        oracle=[
                            (Path(f).name if Path(f).is_absolute() else f, l, c)
                            for f, l, c in targets
                        ],
                        oracle_status=status,
                        outcomes={
                            k: score_single_candidate_outcome(v, a, targets)
                            for k, v in picks.items()
                        },
                    )
                )
            results[crate] = dict(
                scan=scan,
                base_rss_bytes=rss0,
                edit_rss_bytes=rss1,
                **summarize_binding_edit_results(rows),
            )
            group.extend(rows)
            print(crate, len(rows), max(rss1), flush=True)
        cases.extend(group)
        sets[cohort] = summarize_binding_edit_results(group)
    peak = max(max(c["edit_rss_bytes"]) for c in results.values())
    report = dict(
        patch=patch,
        platform=platform.platform(),
        oracle=subprocess.check_output(
            ["rust-analyzer", "--version"], text=True
        ).strip(),
        cohorts=sets,
        crates=results,
        peak_edited_rss_bytes=peak,
        within_100mb=peak < 100_000_000,
    )
    (work / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    (work / "cases.jsonl").write_text("".join(json.dumps(row) + "\n" for row in cases))
    print(json.dumps(sets, indent=2), "peak", peak)


if __name__ == "__main__":
    run_paired_binding_trials(
        Path(sys.argv[1]).resolve(),
        Path(sys.argv[2]).resolve() if len(sys.argv) > 2 else None,
    )
