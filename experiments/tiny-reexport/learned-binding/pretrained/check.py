"""Positive and negative controls through the same release CLI as real crates."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile

BIN = Path(__file__).resolve().parents[2] / "target/release/learned-binding"


def check_source_binding_controls(model):
    prefix = "pub fn make() -> (/* \U0001f680 */ "
    with tempfile.TemporaryDirectory() as temp:
        root = Path(temp)
        (root / "src").mkdir()
        for qualifier in ("alpha::", "beta::", ""):
            source = (
                "pub mod alpha { pub struct Header; }\n"
                "pub mod beta { pub struct Header; }\n"
                + prefix
                + qualifier
                + "Header,) { loop {} }\n"
            )
            (root / "src/lib.rs").write_text(source)
            out = subprocess.run(
                [BIN, "real", model / "binding.onnx", model / "meta.json", root],
                capture_output=True,
                text=True,
                check=True,
            )
            row = json.loads(out.stdout)
            col = len((prefix + qualifier).encode("utf-16-le")) // 2
            assert row["site"] == ["src/lib.rs", 2, col], row
            if qualifier:
                target = qualifier[:-2]
                assert row["fused"] is not None and row["modules"][row["fused"]] == [
                    target
                ], row
            else:
                assert row["fused"] is None and row["local_rule"] is None, row
    print(
        "Passed: alternate real-scanner destinations, no-evidence abstention, UTF-16 positions."
    )


if __name__ == "__main__":
    check_source_binding_controls(Path(sys.argv[1]).resolve())
