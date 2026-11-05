"""Re-grade frozen Tokio predictions under its full Cargo feature, without inference."""

from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import tomllib

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
from evaluate import lookup_real_definition_sites


def grade_full_tokio_predictions(work):
    name = "tokio-1.48.0"
    cases = [
        json.loads(line) for line in (work / "cases.jsonl").read_text().splitlines()
    ]
    tokio = [r for r in cases if r["crate"] == name]
    assert tokio
    with tempfile.TemporaryDirectory() as tmp:
        temp = Path(tmp).resolve()
        sources = temp / name
        shutil.copytree(work / name, sources, ignore=shutil.ignore_patterns("target"))
        manifest = sources / "Cargo.toml"
        old = manifest.read_text()
        expected = tomllib.loads(old)
        assert expected["features"]["default"] == [] and old.count("default = []") == 1
        expected["features"]["default"] = ["full"]
        new = old.replace("default = []", 'default = ["full"]')
        assert tomllib.loads(new) == expected
        manifest.write_text(new)
        dump = temp / "full.lsif.gz"
        with (
            gzip.open(dump, "wb", compresslevel=1) as dst,
            subprocess.Popen(
                ["rust-analyzer", "lsif", "--exclude-vendored-libraries", "."],
                cwd=sources,
                stdout=subprocess.PIPE,
            ) as child,
        ):
            shutil.copyfileobj(child.stdout, dst)
            if child.wait():
                raise RuntimeError("full-feature oracle failed")
        gold = lookup_real_definition_sites(sources, dump, tokio)
        for r in tokio:
            targets = gold[tuple(r["site"])]
            r["oracle"] = [
                (Path(f).name if Path(f).is_absolute() else f, l, c)
                for f, l, c in targets
            ]
            r["oracle_status"] = (
                "unverified"
                if len(targets) != 1
                else "in_pair"
                if list(targets[0]) in r["candidates"]
                else "outside_pair"
            )
            for arm, pick in r["picks"].items():
                r["outcomes"][arm] = (
                    "abstained"
                    if pick is None
                    else "unverified"
                    if len(targets) != 1
                    else "confirmed"
                    if tuple(r["candidates"][pick]) == targets[0]
                    else "wrong"
                )
    fresh = [r for r in cases if r["cohort"] == "fresh"]
    fallback = [r for r in fresh if r["local_rule"] is None]
    report = dict(
        predictions_sha256=hashlib.sha256(
            (work / "cases.jsonl").read_bytes()
        ).hexdigest(),
        tokio_manifest_delta='features.default: [] -> ["full"] (oracle copy only)',
        eligible=len(fresh),
        unresolved_by_rules=len(fallback),
        oracle=dict(Counter(r["oracle_status"] for r in fresh)),
        fallback_oracle=dict(Counter(r["oracle_status"] for r in fallback)),
        fallback={
            arm: dict(Counter(r["outcomes"][arm] for r in fallback))
            for arm in ("base", "edit", "gain_only")
        },
        newly_confirmed=sum(
            r["outcomes"]["edit"] == "confirmed"
            and r["outcomes"]["base"] != "confirmed"
            for r in fallback
        ),
        different_picks=sum(r["picks"]["base"] != r["picks"]["edit"] for r in fresh),
    )
    (work / "full-report.json").write_text(json.dumps(report, indent=2) + "\n")
    (work / "full-tokio-cases.jsonl").write_text(
        "".join(json.dumps(r) + "\n" for r in tokio)
    )
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    grade_full_tokio_predictions(Path(sys.argv[1]).resolve())
