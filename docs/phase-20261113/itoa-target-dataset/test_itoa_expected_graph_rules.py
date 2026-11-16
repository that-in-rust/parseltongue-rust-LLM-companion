import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


RULE_PATH = Path(__file__).with_name("itoa-expected-graph-rules.py")
BUILDER_PATH = Path(__file__).with_name("itoa-expected-graph-builder.py")
RULE_SPEC = importlib.util.spec_from_file_location("itoa_expected_graph_rules", RULE_PATH)
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)


def make_node(label, selected):
    return {"label": label, "effective_visibility": "public" if selected else "private"}


def make_edge(source, relation, target, evidence, **fields):
    return {"source": source, "source_kind": "node", "relation": relation, "target": target,
            "target_kind": "node", "evidence_key": evidence, **fields}


class ReferenceGraphRulesTests(unittest.TestCase):
    """REQ-RUST-039.0: reference rows stay draft, blocked, and deterministic."""

    def test_candidate_inventory_rows_preserved(self):
        nodes = [make_node("root", True), make_node("private", False)]
        edges = [make_edge("root", "calls", "private", "site-a")]
        result = RULES.classify_candidates(nodes, edges)
        self.assertEqual(len(nodes), len(result["nodes"]))
        self.assertEqual(len(edges), len(result["edges"]))
        self.assertEqual(1, len(result.get("eligible_edges", [])))
        self.assertIn("blocked_edges", result)
        self.assertEqual("LLM_PARSED_DRAFT", result["edges"][0]["candidate_status"])

    def test_candidate_provenance_metadata_explicit(self):
        summary = json.loads(RULE_PATH.with_name("itoa-expected-graph-summary.json").read_text())
        provenance = summary["candidate_provenance"]
        self.assertEqual("LLM_PARSED_DRAFT", provenance["status"])
        self.assertEqual("Devin", provenance["author"])
        self.assertEqual("PINNED_SOURCE_READING", provenance["basis"])
        self.assertEqual("NOT_FROZEN", provenance["independent_compiler_oracle_verification"])

    def test_relation_kinds_include_zeroes(self):
        counts = RULES.relation_counts([make_edge("a", "calls", "b", "e1")])
        self.assertEqual({"calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads", "generates", "expands"}, set(counts))
        self.assertEqual(0, counts["reexports"])
        self.assertEqual(0, counts["derives"])

    def test_dangling_node_endpoints_block(self):
        nodes = [make_node("root", True)]
        edges = [
            make_edge("root", "calls", "missing-target", "target-site", target_kind="node"),
            make_edge("missing-source", "calls", "root", "source-site", source_kind="node"),
        ]
        result = RULES.classify_candidates(nodes, edges)
        self.assertEqual([], result["eligible_edges"])
        self.assertEqual(2, len(result["blocked_edges"]))
        self.assertTrue(all("DANGLING_NODE_ENDPOINT" in edge["blockers"]
                            for edge in result["blocked_edges"]))

    def test_nonextracted_statuses_stay_blocked(self):
        edges = [
            make_edge("a", "calls", "b", "ok", resolution_status="extracted", target_kind="node"),
            make_edge("a", "calls", "missing", "unresolved", resolution_status="unresolved"),
            make_edge("a", "calls", "proposal", "proposed", resolution_status="proposed"),
            make_edge("a", "calls", "core::item", "external", resolution_status="extracted",
                      target_kind="external", flags=["external_identity_unverified"]),
        ]
        result = RULES.classify_candidates([make_node("a", True), make_node("b", True)], edges)
        self.assertEqual(["ok"], [edge["evidence_key"] for edge in result.get("eligible_edges", [])])
        self.assertEqual(4, len(result["edges"]))
        self.assertEqual(3, len(result["blocked_edges"]))

    def test_unresolved_reason_not_asserted(self):
        row = RULES.classify_diagnostic_site({"reason_candidate": "external_source_missing", "reason_verified": False})
        self.assertEqual("NOT_ASSERTED", row.get("reason_claim_status"))
        self.assertEqual("BLOCKED", row.get("eligibility_status"))

    def test_wrong_target_count_rejected(self):
        expected = [{"semantic_identity": ("unit-a", "source", "target-b", "calls", "direct")}]
        actual = [{"semantic_identity": ("unit-a", "source", "target-c", "calls", "direct")}]
        result = RULES.compare_semantic_rows(expected, actual, canonical=True)
        self.assertEqual("FAIL", result["status"])
        self.assertTrue(result["missing"])
        self.assertTrue(result["extra"])

    def test_dataset_output_is_deterministic(self):
        first = RULES.serialize_dataset({"z": 1, "a": {"y": 2, "x": 3}})
        second = RULES.serialize_dataset({"a": {"x": 3, "y": 2}, "z": 1})
        self.assertEqual(first, second)
        with tempfile.TemporaryDirectory() as left, tempfile.TemporaryDirectory() as right:
            for output in (left, right):
                subprocess.run([sys.executable, str(BUILDER_PATH), "--output-directory", output],
                               check=True, capture_output=True, text=True)
            left_files = {path.name: path.read_bytes() for path in Path(left).iterdir()}
            right_files = {path.name: path.read_bytes() for path in Path(right).iterdir()}
        self.assertEqual(left_files, right_files)
        self.assertIn("itoa-expected-graph-edges.json", left_files)
        self.assertIn("itoa-unblocked-rows-subset.json", left_files)

    def test_source_pin_digest_mismatch(self):
        with self.assertRaises(ValueError):
            RULES.verify_source_digests({"src/lib.rs": b"changed"}, {"src/lib.rs": "0" * 64})

    def test_unflagged_contains_allowed(self):
        nodes = [make_node("area", True), make_node("item", True)]
        result = RULES.classify_candidates(nodes, [make_edge("area", "contains", "item", "site-a")])
        self.assertEqual(1, len(result["eligible_edges"]))

    def test_flagged_contains_remains_blocked(self):
        edge = make_edge("impl@93", "contains", "Buffer::new", "site-a", flags=["OD-7_containment"])
        self.assertIn("OD-7_OWNERSHIP_UNDECIDED", RULES._edge_blockers(edge, {"impl@93": [], "Buffer::new": []}))

    def test_primitive_builtin_is_allowed(self):
        nodes = [make_node("Buffer", True)]
        edge = make_edge("Buffer", "uses_type", "u8", "site-u8", target_kind="builtin")
        result = RULES.classify_candidates(nodes, [edge])
        self.assertEqual("ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC", result["edges"][0]["eligibility_status"])

    def test_unknown_builtin_target_blocked(self):
        nodes = [make_node("Buffer", True)]
        edge = make_edge("Buffer", "uses_type", "magic_type", "site-magic", target_kind="builtin")
        result = RULES.classify_candidates(nodes, [edge])
        self.assertIn("UNKNOWN_BUILTIN_IDENTITY", result["edges"][0]["blockers"])

    def test_primitive_vocabulary_matches_source(self):
        root = Path(__file__).resolve().parents[3]
        source = (root / "crates/parseltongue-core-row-types/src/hld-materialization-core-records.rs").read_text()
        vocabulary = source.split("const HLD_BUILTIN_TYPE_NAMES:", 1)[1].split("];", 1)[0]
        source_names = set(re.findall(r'"([^"]+)"', vocabulary))
        self.assertEqual(source_names, RULES.BUILTIN_PRIMITIVES)

    def test_candidate_coverage_gaps_explicit(self):
        full = json.loads(RULE_PATH.with_name("itoa-expected-graph-nodes.json").read_text())
        coverage = full.get("source_coverage", {})
        self.assertIn("whole_source_coverage_claimed", coverage)
        self.assertFalse(coverage.get("whole_source_coverage_claimed", True))
        self.assertTrue({"operators", "body_types", "literals", "repeated_same_line_sites"}
                        <= set(coverage["known_uncovered_constructs"]))

    def test_builder_pin_cli_supported(self):
        root = Path(__file__).resolve().parents[3]
        pin = root / "docs/phase-20261111/phase-one-benchmark-pin.json"
        source = Path("/home/ubuntu/repos/rust-tools/itoa")
        with tempfile.TemporaryDirectory() as output:
            completed = subprocess.run([sys.executable, str(BUILDER_PATH), "--pin-file", str(pin),
                                        "--source-root", str(source), "--output-directory", output],
                                       check=False, capture_output=True, text=True)
        self.assertEqual(0, completed.returncode, completed.stderr)


if __name__ == "__main__":
    unittest.main()
