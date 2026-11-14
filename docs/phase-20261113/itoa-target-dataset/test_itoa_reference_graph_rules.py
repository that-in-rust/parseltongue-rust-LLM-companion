import hashlib
import importlib.util
import io
import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch


RULE_PATH = Path(__file__).with_name("itoa-reference-graph-rules.py")
BUILDER_PATH = Path(__file__).with_name("itoa-target-dataset-builder.py")
RULE_SPEC = importlib.util.spec_from_file_location("itoa_reference_graph_rules", RULE_PATH)
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)


def make_node(label, selected):
    return {"label": label, "interface_selected": selected,
            "effective_visibility": "public" if selected else "private"}


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
        summary = json.loads(RULE_PATH.with_name("itoa-target-dataset-summary.json").read_text())
        provenance = summary["candidate_provenance"]
        self.assertEqual("LLM_PARSED_DRAFT", provenance["status"])
        self.assertEqual("Devin", provenance["author"])
        self.assertEqual("PINNED_SOURCE_READING", provenance["basis"])
        self.assertEqual("NOT_FROZEN", provenance["independent_compiler_oracle_verification"])

    def test_relation_kinds_include_zeroes(self):
        counts = RULES.relation_counts([make_edge("a", "calls", "b", "e1")])
        self.assertEqual({"calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads"}, set(counts))
        self.assertEqual(0, counts["reexports"])
        self.assertEqual(0, counts["derives"])

    def test_collapse_excludes_forbidden_hops(self):
        nodes = [make_node("public", True), make_node("use", True), make_node("hidden-a", False),
                 make_node("hidden-b", False), make_node("hidden-c", False), make_node("kept", True)]
        edges = [
            make_edge("public", "contains", "hidden-a", "contains-first"),
            make_edge("hidden-a", "calls", "kept", "call-after-contains"),
            make_edge("public", "calls", "hidden-b", "call-first"),
            make_edge("hidden-b", "imports", "kept", "imports-later"),
            make_edge("use", "imports", "hidden-c", "imports-first"),
            make_edge("hidden-c", "calls", "kept", "call-after-import"),
            make_edge("public", "contains", "kept", "direct-contains"),
            make_edge("use", "imports", "kept", "direct-import"),
        ]
        rows = RULES.derive_interface(nodes, edges)["edges"]
        self.assertTrue(all(row["via"] == "direct" for row in rows if row["relation"] in {"contains", "imports"}))
        self.assertFalse(any(row["via"] == "collapsed" and row["relation"] in {"contains", "imports"} for row in rows))
        self.assertEqual({("public", "contains", "kept"), ("use", "imports", "kept")},
                         {(row["source"], row["relation"], row["target"]) for row in rows})

    def test_forbidden_direct_external_targets(self):
        nodes = [make_node("public", True)]
        edges = [
            make_edge("public", "contains", "external::Item", "contains-external", target_kind="external"),
            make_edge("public", "contains", "u8", "contains-builtin", target_kind="builtin"),
            make_edge("public", "imports", "external::Item", "imports-external", target_kind="external"),
            make_edge("public", "imports", "u8", "imports-builtin", target_kind="builtin"),
        ]
        rows = RULES.derive_interface(nodes, edges)["edges"]
        self.assertEqual(
            {("public", "contains", "external::Item"), ("public", "contains", "u8"),
             ("public", "imports", "external::Item"), ("public", "imports", "u8")},
            {(row["source"], row["relation"], row["target"]) for row in rows},
        )
        self.assertTrue(all(row["via"] == "direct" for row in rows))

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

    def test_cycles_stop_repeating_nodes(self):
        nodes = [make_node("public", True), make_node("left", False), make_node("right", False), make_node("target", True)]
        edges = [make_edge("public", "calls", "left", "e1"),
                 make_edge("left", "calls", "right", "e2"),
                 make_edge("right", "calls", "left", "cycle"),
                 make_edge("right", "calls", "target", "e3")]
        rows = RULES.derive_interface(nodes, edges)["edges"]
        self.assertEqual(1, len(rows))
        self.assertEqual("collapsed", rows[0]["via"])
        self.assertEqual([["e1", "e2", "e3"]], rows[0]["evidence_paths"])

    def test_diamond_paths_keep_provenance(self):
        nodes = [make_node("public", True), make_node("left", False), make_node("right", False), make_node("target", True)]
        edges = [make_edge("public", "calls", "left", "e1"),
                 make_edge("public", "calls", "right", "e2"),
                 make_edge("left", "calls", "target", "e3"),
                 make_edge("right", "calls", "target", "e4")]
        rows = RULES.derive_interface(nodes, edges)["edges"]
        self.assertEqual(1, len(rows))
        self.assertEqual({("e1", "e3"), ("e2", "e4")}, {tuple(path) for path in rows[0]["evidence_paths"]})

    def test_direct_collapsed_keys_separate(self):
        nodes = [make_node("public", True), make_node("private", False), make_node("target", True)]
        edges = [make_edge("public", "calls", "target", "direct"),
                 make_edge("public", "calls", "private", "first"),
                 make_edge("private", "calls", "target", "last")]
        rows = RULES.derive_interface(nodes, edges)["edges"]
        self.assertEqual({"direct", "collapsed"}, {row["via"] for row in rows})

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

    def test_empty_graph_outputs_empty(self):
        self.assertEqual({"nodes": [], "edges": []}, RULES.derive_interface([], []))
        self.assertEqual({"counts": {}, "blocked_files": []}, RULES.rollup_unique_edges([], ["src/lib.rs"]))

    def test_rollups_count_unique_edges(self):
        rows = [
            {"edge_identity": ("src", "dst", "calls", "direct"), "source_file": "src/lib.rs", "path_count": 5},
            {"edge_identity": ("src", "dst", "calls", "direct"), "source_file": "src/lib.rs", "path_count": 2},
        ]
        result = RULES.rollup_unique_edges(rows, ["src/lib.rs"])
        self.assertEqual({"src/lib.rs": 1}, result.get("counts"))

    def test_rollup_edges_accounted_once(self):
        nodes = [
            {"label": "source", "file": "src/lib.rs"},
            {"label": "local", "file": "src/u128_ext.rs"},
        ]
        expected_edges = [
            {"source": "source", "relation": "calls", "target": "local", "via": "direct"},
            {"source": "source", "relation": "uses_type", "target": "u32", "via": "direct"},
        ]
        source_edges = [
            {"source": "source", "relation": "calls", "target": "local", "target_kind": "node"},
            {"source": "source", "relation": "uses_type", "target": "u32", "target_kind": "builtin"},
        ]
        result = RULES.group_expected_rollup_edges(
            expected_edges, source_edges, nodes, ["src/lib.rs", "src/u128_ext.rs"])
        expected_ids = [(row["source"], row["relation"], row["target"], row["via"])
                        for row in expected_edges]
        excluded = result["excluded_rollup_edges"]
        excluded_ids = [(row["source"], row["relation"], row["target"], row["via"])
                        for row in excluded]
        file_ids = [identity for group in result["file_groups"].values() for identity in group]
        folder_ids = [identity for group in result["folder_groups"].values() for identity in group]
        self.assertCountEqual(expected_ids, file_ids + excluded_ids)
        self.assertCountEqual(expected_ids, folder_ids + excluded_ids)
        self.assertEqual(len(expected_ids), len(file_ids) + len(excluded_ids))
        self.assertEqual(len(expected_ids), len(folder_ids) + len(excluded_ids))
        self.assertEqual("BUILTIN_BUCKET_MAPPING_NOT_IMPLEMENTED", excluded[0]["reason"])
        dataset = RULE_PATH.parent
        expected = json.loads((dataset / "itoa-expected-dependency-edges.json").read_text())["rows"]
        file_rollup = json.loads((dataset / "itoa-expected-file-rollup.json").read_text())
        folder_rollup = json.loads((dataset / "itoa-expected-folder-rollup.json").read_text())
        file_exclusions = file_rollup["excluded_rollup_edges"]
        folder_exclusions = folder_rollup["excluded_rollup_edges"]
        self.assertEqual(file_exclusions, folder_exclusions)
        self.assertEqual(len(expected), sum(row["edge_count"] for row in file_rollup["rows"])
                         + len(file_exclusions))
        self.assertEqual(len(expected), sum(row["edge_count"] for row in folder_rollup["rows"])
                         + len(folder_exclusions))
        self.assertEqual("BLOCKED", file_rollup["artifact_status"])
        self.assertEqual("BLOCKED", folder_rollup["artifact_status"])
        self.assertIn("BUILTIN_BUCKET_MAPPING_NOT_IMPLEMENTED",
                      {row["reason"] for row in file_exclusions})

    def test_macro_rollup_stays_blocked(self):
        rows = [{"eligibility_status": "BLOCKED", "blockers": ["MACRO_FILE_ASSIGNMENT_UNRESOLVED"],
                 "macro_invocation_file": "src/lib.rs"}]
        result = RULES.rollup_unique_edges(rows, ["src/lib.rs", "src/u128_ext.rs"])
        self.assertIn("src/lib.rs", result.get("blocked_files", []))

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
        self.assertIn("itoa-full-graph-edges.json", left_files)
        self.assertIn("itoa-local-verification-subset.json", left_files)

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

    def test_all_six_relations_collapse(self):
        relations = {"calls", "uses_type", "implements", "reexports", "derives", "reads"}
        nodes = [make_node("public-a", True), make_node("private", False), make_node("public-b", True)]
        edges = [edge for relation in relations for edge in (
            make_edge("public-a", relation, "private", f"{relation}-in"),
            make_edge("private", relation, "public-b", f"{relation}-out"),
        )]
        result = RULES.derive_interface(nodes, edges)
        self.assertEqual(relations, {row["relation"] for row in result["edges"] if row["via"] == "collapsed"})

    def test_candidate_coverage_gaps_explicit(self):
        full = json.loads(RULE_PATH.with_name("itoa-full-graph-nodes.json").read_text())
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
