"""Regenerate, label with rust-analyzer, fit on train only, evaluate frozen weights."""
import collections
import hashlib
import json
import os
import platform
import re
import subprocess
import tempfile
from pathlib import Path

import numpy as np
from sklearn.linear_model import LogisticRegression

from corpus import generate_paired_source_cases
from oracle import collect_rust_analyzer_labels

HERE = Path(__file__).resolve().parent
BINARY = HERE.parent / "target/release/learned-binding"


def run_binding_cli_batch(mode, root, rows, weights=None):
    args = [BINARY, mode] + ([weights] if weights else [])
    paths = [root / "src" / (r["case"] + ".rs") for r in rows]
    result = subprocess.run(args + paths, check=True, capture_output=True, text=True)
    outputs = [json.loads(line) for line in result.stdout.splitlines()]
    assert len(outputs) == len(rows)
    return outputs


def identify_ranker_feature_input(example):
    # Ignore candidate order; the same pair reversed is not novel input to a
    # shared scorer. Keep the scope and return position markers in the tokens.
    return tuple(sorted(tuple(sorted(v)) for v in example["features"]))


def fit_training_pairs_only(root, train, dest):
    assert all(r["split"] == "train" for r in train)
    extracted = run_binding_cli_batch("features", root, train)
    vocab = sorted({w for e in extracted for v in e["features"] for w in v})
    index = {word: n for n, word in enumerate(vocab)}
    x = np.zeros((len(train), len(vocab)))
    y = []
    for i, (r, e) in enumerate(zip(train, extracted)):
        for sign, feats in zip([1, -1], e["features"]):
            for word in feats:
                x[i, index[word]] += sign
        y.append(e["candidates"].index(r["ra_target"].split("::")[0]) == 0)
    clf = LogisticRegression(C=1.0, fit_intercept=False, solver="lbfgs", max_iter=1000, tol=1e-9)
    clf.fit(x, y)
    weights = {w: float(v) for w, v in zip(vocab, clf.coef_[0]) if abs(v) > 1e-12}
    dest.write_text(json.dumps(weights, indent=2, sort_keys=True) + "\n")
    return {identify_ranker_feature_input(e) for e in extracted}


def score_heldout_binding_predictions(root, rows, dest, train_inputs):
    predictions = run_binding_cli_batch("predict", root, rows, dest)
    control = run_binding_cli_batch("features", root, rows)
    stats, pairs, details = {}, collections.defaultdict(list), []
    for r, p, e in zip(rows, predictions, control):
        group = r["split"] + "/" + r["kind"]
        s = stats.setdefault(group, dict(n=0, correct_link=0, wrong_link=0, abstain=0,
                                         correct_abstain=0, simple_right=0, simple_abstain=0, first_right=0,
                                         feature_input_seen_in_train=0))
        s["n"] += 1
        s["feature_input_seen_in_train"] += identify_ranker_feature_input(e) in train_inputs
        target = r["ra_target"]
        mod = target.split("::")[0] if target else None
        s["first_right"] += (e["candidates"][0] == mod)
        simple = e["candidates"][e["simple"]] if e["simple"] is not None else None
        s["simple_right"] += simple is not None and simple == mod
        s["simple_abstain"] += simple is None
        if p["target"] is None:
            s["abstain"] += 1
            s["correct_abstain"] += target is None
        elif p["target"] == target:
            s["correct_link"] += 1
        else:
            s["wrong_link"] += 1
        if r["pair"] is not None:
            pairs[group, r["pair"]].append(p["target"] == target)
        details.append(dict(case=r["case"], sha256=r["sha256"], oracle=target,
                            predicted=p["target"], score=p["score"], simple_module=simple))
    for group, s in stats.items():
        ps = [vals for (g, _), vals in pairs.items() if g == group]
        assert all(len(v) == 2 for v in ps)
        if ps:
            s["pairs_both_right"] = sum(all(v) for v in ps)
    return stats, details


def measure_full_process_memory(root, heldout, model):
    # Measure the CLI, including source parsing, learnt weights, fixed ONNX
    # resolver, runtime libraries and NDJSON serialization. Never a live teacher.
    mac = platform.system() == "Darwin"
    assert mac or platform.system() == "Linux"
    paths = [root / "src" / (r["case"] + ".rs") for r in heldout]
    rss, digest = [], None
    for _ in range(5):
        p = subprocess.run(["/usr/bin/time", "-l" if mac else "-v", BINARY, "predict",
                            model, *paths], check=True, capture_output=True, env={**os.environ, "LC_ALL":"C"})
        raw = re.search(rb"(\d+)\s+maximum resident set size" if mac else
                        rb"Maximum resident set size \(kbytes\):\s*(\d+)", p.stderr)
        if not raw:
            raise RuntimeError("no RSS measurement")
        rss.append(int(raw[1]) * (1 if mac else 1024))
        assert len(p.stdout.splitlines()) == len(heldout)
        current = hashlib.sha256(p.stdout).hexdigest()
        assert digest in (None, current)
        digest = current
    return rss


def run_full_binding_experiment(root):
    rows = generate_paired_source_cases(root)
    if len({r["sha256"] for r in rows}) != len(rows):
        raise RuntimeError("duplicate source file")
    collect_rust_analyzer_labels(root, rows)
    train = [r for r in rows if r["split"] == "train"]
    heldout = [r for r in rows if r["split"] != "train"]
    weights = HERE / "weights.json"
    train_inputs = fit_training_pairs_only(root, train, weights)
    stats, details = score_heldout_binding_predictions(root, heldout, weights, train_inputs)
    peak = measure_full_process_memory(root, heldout, weights)
    report = dict(platform=platform.platform(),
                  rust_analyzer=subprocess.check_output(["rust-analyzer", "--version"], text=True).strip(),
                  train=len(train), unique_train_feature_pairs=len(train_inputs),
                  test=len(heldout), model_bytes=weights.stat().st_size,
                  model_sha256=hashlib.sha256(weights.read_bytes()).hexdigest(), groups=stats,
                  max_rss_bytes=peak, peak_budget_bytes=100_000_000)
    (HERE / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    (HERE / "predictions.jsonl").write_text("".join(json.dumps(v) + "\n" for v in details))
    print(json.dumps(report, indent=2))
    if max(peak) >= 100_000_000:
        raise SystemExit("RSS gate failed")


if __name__ == "__main__":
    with tempfile.TemporaryDirectory(prefix="learned-binding-") as tmp:
        run_full_binding_experiment(Path(tmp))
