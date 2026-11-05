"""Check which original-model heads accepted the confirmed fallback sites."""

from pathlib import Path
import json
import subprocess
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from evaluate import BIN


def verify_original_head_contributions(work):
    model = HERE.parent / "model"
    cases = [
        json.loads(line)
        for line in (work / "full-tokio-cases.jsonl").read_text().splitlines()
    ]
    confirmed = {
        tuple(r["site"]): r
        for r in cases
        if r["outcomes"]["base"] == "confirmed" and r["local_rule"] is None
    }
    p = subprocess.run(
        [
            BIN,
            "real",
            model / "binding.onnx",
            model / "meta.json",
            work / "tokio-1.48.0",
        ],
        capture_output=True,
        check=True,
    )
    out = []
    for r in map(json.loads, p.stdout.splitlines()):
        if tuple(r["site"]) in confirmed:
            row = confirmed[tuple(r["site"])]
            assert (
                r["candidates"] == row["candidates"]
                and r["fused"] == row["picks"]["base"]
                and r["local_rule"] is None
            )
            out.append(
                {
                    k: r[k]
                    for k in (
                        "function",
                        "site",
                        "fused",
                        "lexical",
                        "encoder",
                        "margins",
                    )
                }
            )
    assert len(out) == len(confirmed)
    (work / "head-check.json").write_text(json.dumps(out, indent=2) + "\n")
    print(
        len(out),
        "confirmed sites;",
        sum(r["lexical"] is None and r["encoder"] is None for r in out),
        "accepted only when heads combined",
    )


if __name__ == "__main__":
    verify_original_head_contributions(Path(sys.argv[1]).resolve())
