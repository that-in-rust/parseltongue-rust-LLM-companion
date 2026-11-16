import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).parent
RULE_PATH = HERE / "itoa-expected-graph-rules.py"
BUILDER_PATH = HERE / "itoa-expected-graph-builder.py"
MANIFEST_PATH = HERE / "itoa-expected-graph-freeze.json"
RULE_SPEC = importlib.util.spec_from_file_location("itoa_expected_graph_rules", RULE_PATH)
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)


def read_file_digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_builder_into(output):
    return subprocess.run([sys.executable, str(BUILDER_PATH), "--output-directory", output],
                          check=False, capture_output=True, text=True)


class ExpectedGraphFreezeTests(unittest.TestCase):
    """REQ-RUST-044.3: the expected graph is frozen, reproducible from the pinned source, and deterministic."""

    def test_provenance_states_review_limits(self):
        summary = json.loads((HERE / "itoa-expected-graph-summary.json").read_text())
        self.assertEqual("FROZEN_EXPECTED_GRAPH", summary["artifact_status"])
        self.assertEqual("Devin", summary["provenance"]["author"])
        self.assertEqual("PINNED_SOURCE_READING", summary["provenance"]["basis"])
        self.assertEqual("AUTHOR_ONLY_NO_SECOND_REVIEWER", summary["provenance"]["review"])
        self.assertEqual("NONE", summary["provenance"]["independent_compiler_oracle_verification"])
        self.assertFalse(summary["production_input"])

    def test_relation_kinds_include_zeroes(self):
        counts = RULES.relation_counts([{"relation": "calls"}])
        self.assertEqual({"calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads", "generates", "expands"}, set(counts))
        self.assertEqual(0, counts["reexports"])
        self.assertEqual(0, counts["derives"])

    def test_dataset_output_is_deterministic(self):
        first = RULES.serialize_dataset({"z": 1, "a": {"y": 2, "x": 3}})
        second = RULES.serialize_dataset({"a": {"x": 3, "y": 2}, "z": 1})
        self.assertEqual(first, second)
        with tempfile.TemporaryDirectory() as left, tempfile.TemporaryDirectory() as right:
            for output in (left, right):
                self.assertEqual(0, run_builder_into(output).returncode)
            left_files = {path.name: path.read_bytes() for path in Path(left).iterdir()}
            right_files = {path.name: path.read_bytes() for path in Path(right).iterdir()}
        self.assertEqual(left_files, right_files)
        self.assertIn("itoa-expected-graph-edges.json", left_files)
        self.assertNotIn("itoa-expected-graph-freeze.json", left_files)

    def test_source_pin_digest_mismatch(self):
        with self.assertRaises(ValueError):
            RULES.verify_source_digests({"src/lib.rs": b"changed"}, {"src/lib.rs": "0" * 64})

    def test_primitive_vocabulary_matches_source(self):
        root = Path(__file__).resolve().parents[3]
        source = (root / "crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs").read_text()
        vocabulary = source.split("const HLD_BUILTIN_TYPE_NAMES:", 1)[1].split("];", 1)[0]
        source_names = set(re.findall(r'"([^"]+)"', vocabulary))
        self.assertEqual(source_names, RULES.BUILTIN_PRIMITIVES)

    def test_uncovered_constructs_are_named(self):
        full = json.loads((HERE / "itoa-expected-graph-nodes.json").read_text())
        coverage = full["source_coverage"]
        self.assertFalse(coverage["whole_source_coverage_claimed"])
        self.assertEqual(["operators", "literals"], coverage["known_uncovered_constructs"])

    def test_builder_pin_cli_supported(self):
        root = Path(__file__).resolve().parents[3]
        pin = root / "docs/phase-20261111/phase-one-benchmark-pin.json"
        source = Path("/home/ubuntu/repos/rust-tools/itoa")
        with tempfile.TemporaryDirectory() as output:
            completed = subprocess.run([sys.executable, str(BUILDER_PATH), "--pin-file", str(pin),
                                        "--source-root", str(source), "--output-directory", output],
                                       check=False, capture_output=True, text=True)
        self.assertEqual(0, completed.returncode, completed.stderr)

    def test_frozen_files_match_manifest(self):
        manifest = json.loads(MANIFEST_PATH.read_text())
        self.assertEqual({name: read_file_digest(HERE / name) for name in manifest["frozen_file_sha256"]},
                         manifest["frozen_file_sha256"])
        self.assertEqual({name: read_file_digest(HERE / name) for name in manifest["builder_file_sha256"]},
                         manifest["builder_file_sha256"])

    def test_builder_reproduces_frozen_files(self):
        manifest = json.loads(MANIFEST_PATH.read_text())
        with tempfile.TemporaryDirectory() as output:
            self.assertEqual(0, run_builder_into(output).returncode)
            rebuilt = {name: read_file_digest(Path(output) / name) for name in manifest["frozen_file_sha256"]}
        self.assertEqual(manifest["frozen_file_sha256"], rebuilt)


if __name__ == "__main__":
    unittest.main()
