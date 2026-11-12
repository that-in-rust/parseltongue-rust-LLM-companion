#!/usr/bin/env python3
"""Collect the pinned external harness's current PRIMARY path exports."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import time


def measure_primary_semantic_exports():
    repository = Path(__file__).resolve().parents[1]
    binary = repository / "tools/oracle-semantic-export-harness/target/release/oracle-semantic-export-harness"
    manifest = json.loads((repository / "docs/phase-20261111/corpus-source-pin-manifest.json").read_text())
    binary_digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    for name in ["rust-analyzer", "ripgrep", "tokio", "serde", "cargo-modules"]:
        root = Path.home() / "repos/rust-tools" / name
        output = repository / "e2e-runs/2026-11-11" / name / "m2-primary-path-exports"
        output.mkdir(parents=True, exist_ok=True)
        head = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
        started = time.monotonic()
        with (output / "oracle-export-error-log.txt").open("w") as error_log:
            result = subprocess.run(
                [str(binary), "--workspace-source-root-path", str(root),
                 "--oracle-path-export-output", str(output / "oracle-primary-semantic-rows.json")],
                stdout=error_log, stderr=error_log,
                env={**os.environ, "CARGO_NET_OFFLINE": "true"},
            )
        summary = {
            "corpus_repository_name_value": name,
            "corpus_source_commit_hash": head,
            "oracle_executable_sha256_digest": binary_digest,
            "oracle_export_exit_status": result.returncode,
            "oracle_export_elapsed_seconds": time.monotonic() - started,
            "oracle_configuration_kind_value": "PRIMARY-current-harness",
            "oracle_configuration_pending_items": [
                "exact mapped-unit manifests", "cached proc-macro injection",
                "expanded-source path provenance",
            ],
            "corpus_pin_manifest_value": manifest,
        }
        if result.returncode == 0:
            rows = json.loads((output / "oracle-primary-semantic-rows.json").read_text())
            with (output / "oracle-primary-semantic-rows.jsonl").open("w") as stream:
                for row in rows:
                    stream.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")
            summary["oracle_export_path_count"] = len(rows)
        (output / "oracle-export-measurement-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
        print(json.dumps({key: value for key, value in summary.items() if key != "corpus_pin_manifest_value"}), flush=True)


if __name__ == "__main__":
    measure_primary_semantic_exports()
