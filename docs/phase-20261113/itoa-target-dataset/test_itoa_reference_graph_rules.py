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
LADDER_PATH = Path(__file__).with_name("itoa-backward-table-ladder.py")
LADDER_SPEC = importlib.util.spec_from_file_location("itoa_backward_table_ladder", LADDER_PATH)
LADDER = importlib.util.module_from_spec(LADDER_SPEC)
LADDER_SPEC.loader.exec_module(LADDER)


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

    def test_mixed_generation_exports_error(self):
        headers = [{"generation": "gen-a", "status": "OK"}, {"generation": "gen-b", "status": "OK"}]
        result = RULES.validate_export_generation(headers, {"syntax-spans": "complete"})
        self.assertEqual("ERROR", result["status"])

    def test_export_failure_stays_error(self):
        result = RULES.validate_export_generation([], {}, errors=["export failed"])
        self.assertEqual("ERROR", result["status"])

    def test_invalid_export_payload_errors(self):
        header = {"export_record_kind_value": "header", "export_owned_table_name": "table",
                  "export_scope_identity_record": {"run_generation_digest_value": "g",
                                                   "stage_attempt_sequence_value": "a"}}
        row = {"export_record_kind_value": "row", "export_row_payload_value": "{invalid"}
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_path = Path(temporary_directory)
            export_path = temporary_path / "table.jsonl"
            export_path.write_text(json.dumps(header) + "\n" + json.dumps(row) + "\n")
            with self.assertRaisesRegex(ValueError, "invalid row payload"):
                LADDER.load_export(export_path)
            export_directory = temporary_path / "exports"
            export_directory.mkdir()
            export_path = export_directory / "table.jsonl"
            export_path.write_text(json.dumps(header) + "\n" + json.dumps(row) + "\n")
            root = Path(__file__).resolve().parents[3]
            pin_file = root / "docs/phase-20261111/phase-one-benchmark-pin.json"
            pin = json.loads(pin_file.read_text())
            arguments = ["itoa-backward-table-ladder.py", "--export-directory", str(export_directory),
                         "--output-directory", str(temporary_path / "report"),
                         "--manifest-file", str(RULE_PATH.with_name("itoa-table-contract-manifest.json")),
                         "--pin-file", str(pin_file), "--source-root", pin["benchmark_source_checkout_path"]]
            with patch.object(sys, "argv", arguments), redirect_stdout(io.StringIO()):
                LADDER.main()
            report = json.loads((temporary_path / "report/itoa-backward-table-report.json").read_text())
            self.assertEqual("ERROR", report["export_integrity"]["status"])
            self.assertIn("invalid row payload JSON in table.jsonl", report["errors"][0])
            self.assertEqual("BLOCKED", report["verification_status"])

    def test_missing_marker_stays_blocked(self):
        headers = [{"generation": "gen-a", "status": "OK"}]
        result = RULES.validate_export_generation(headers, {"signature-lowering": "missing"})
        self.assertEqual("BLOCKED", result["status"])

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
        self.assertIn("itoa-table-contract-manifest.json", left_files)
        self.assertIn("itoa-local-verification-subset.json", left_files)

    def test_source_pin_digest_mismatch(self):
        with self.assertRaises(ValueError):
            RULES.verify_source_digests({"src/lib.rs": b"changed"}, {"src/lib.rs": "0" * 64})

    def test_syntax_semantic_gap_reported(self):
        syntax = [{"source_file": "src/lib.rs", "source_sha256": "digest", "span_byte_start_offset": 2506,
                   "span_byte_end_offset": 2519}]
        result = RULES.checkpoint_status(syntax, [], ("src/lib.rs", "digest", 2506, 2519))
        self.assertEqual("PRESENT", result.get("syntax_evidence_status"))
        self.assertEqual("MISSING", result.get("semantic_row_status"))

    def test_contract_manifest_blocks_unbuilt(self):
        contracts = [{"table_name": "impl-header-index-table", "status": "PROPOSED_NOT_IMPLEMENTED"}]
        statuses = RULES.contract_manifest_statuses(contracts)
        self.assertEqual("BLOCKED/NOT_IMPLEMENTED", statuses["impl-header-index-table"])

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

    def test_od9_does_not_suppress(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        output = json.loads(RULE_PATH.with_name("itoa-expected-dependency-edges.json").read_text())
        policy = manifest["collapse_policy"]
        collapsed = [row for row in output["rows"] if row["via"] == "collapsed"]
        self.assertFalse(policy["od9_suppression_applied"])
        self.assertEqual(RULES.COLLAPSIBLE_RELATIONS, set(policy["current_decided_relations"]))
        self.assertEqual(len(collapsed), policy["derived_collapsed_rows"])

    def test_wrong_target_site_rejected(self):
        syntax = [{"source_file": "src/lib.rs", "source_sha256": "digest", "span_byte_start_offset": 10,
                   "span_byte_end_offset": 20}]
        semantic = [{"source_file": "src/lib.rs", "start_byte": 10, "end_byte": 20,
                     "resolution": "resolved", "target_identity": "Other::new"}]
        result = RULES.checkpoint_status(syntax, semantic, ("src/lib.rs", "digest", 10, 20))
        self.assertNotEqual("PRESENT", result["semantic_row_status"])

    def test_unresolved_target_site_blocked(self):
        syntax = [{"source_file": "src/lib.rs", "source_sha256": "digest", "span_byte_start_offset": 10,
                   "span_byte_end_offset": 20}]
        semantic = [{"source_file": "src/lib.rs", "start_byte": 10, "end_byte": 20,
                     "resolution": "unresolved", "target_identity": None}]
        result = RULES.checkpoint_status(syntax, semantic, ("src/lib.rs", "digest", 10, 20))
        self.assertNotEqual("PRESENT", result["semantic_row_status"])

    def test_checkpoint_source_bytes_checked(self):
        source = b"prefixBuffer::new()suffix"
        digest = hashlib.sha256(source).hexdigest()
        result = RULES.verify_checkpoint_source(source, digest, 6, 19, "Buffer::new()")
        self.assertEqual("PRESENT", result["source_evidence_status"])
        mismatch = RULES.verify_checkpoint_source(source, "0" * 64, 6, 19, "Buffer::new()")
        self.assertNotEqual("PRESENT", mismatch["source_evidence_status"])

    def test_registry_owners_are_exact(self):
        root = Path(__file__).resolve().parents[3]
        registry_path = root / "crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs"
        body = registry_path.read_text().split("const STAGE_OWNED_TABLE_NAMES:", 1)[1].split("];", 1)[0]
        pairs = re.findall(r'\(\s*"([^"\n]+)"\s*,\s*"([^"\n]+)"\s*,?\s*\)', body, re.S)
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        observed = {row["table_name"]: row["owner"] for row in manifest["contracts"]
                    if row.get("record_kind") == "registered_stage_table"}
        self.assertEqual(dict(pairs), observed)
        registered = [row for row in manifest["contracts"]
                      if row.get("record_kind") == "registered_stage_table"]
        self.assertTrue(all(row.get("provenance_citations") for row in registered))

    def test_payload_fields_have_provenance(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        contracts = {row["table_name"]: row for row in manifest["contracts"]}
        item = contracts["syntax-namespace-input-items"]
        item_fields = item.get("observed_payload_fields", [])
        self.assertIn("binding_lexical_scope_key", item_fields)
        self.assertIn("binding_target_identity_key", item_fields)
        self.assertTrue(item.get("provenance_citations"))
        path = contracts["path-resolution-site-records"]
        path_fields = path.get("observed_payload_fields", [])
        self.assertIn("site_source_range_start", path_fields)
        self.assertIn("path_target_symbol_rows", path_fields)
        self.assertNotIn("site_occurrence_key", path_fields)
        self.assertTrue(path.get("provenance_citations"))

    def test_payload_citations_are_exact(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        contracts = {row["table_name"]: row for row in manifest["contracts"]}
        item = contracts["syntax-namespace-input-items"]
        self.assertEqual("OBSERVED_SOURCE_SERIALIZER_FIELDS", item["payload_schema_status"])
        self.assertNotIn("binding_resolution_evidence_values", item["observed_payload_fields"])
        self.assertEqual("crates/parseltongue-syntax-span-extractor/src/syntax-namespace-record-codec.rs",
                         item["field_provenance_citations"][0]["path"])
        path = contracts["path-resolution-site-records"]
        self.assertEqual("OBSERVED_SERIALIZED_EXPORT_FIELDS", path["payload_schema_status"])
        citation = path["field_provenance_citations"][0]
        self.assertTrue(citation["path"].endswith("exported-table-rows/path-resolution-site-records.json"))
        self.assertEqual("200527fb15e4e88d064c12943ffaaaa0accaa121bc08fffe76a89fab5cf6339a",
                         citation["generation"])

    def test_storage_keys_match_schema(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        registered = [row for row in manifest["contracts"]
                      if row.get("record_kind") == "registered_stage_table"]
        expected_fields = ["run_generation_digest_value", "stage_attempt_sequence_value",
                           "stage_row_record_key", "stage_unit_work_key", "stage_row_payload_text"]
        self.assertTrue(registered)
        self.assertTrue(all(row["storage_schema_status"] == "EXACT_SQL_DDL"
                            and row["required_fields"] == expected_fields
                            and row["key_contract"]["primary_key"] == expected_fields[:3]
                            for row in registered))

    def test_content_store_not_control(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        self.assertNotIn("file-source-content-records", manifest.get("publication_control_only_tables", []))
        source_relations = manifest.get("source_context_relations", [])
        self.assertTrue(any(row.get("table_name") == "file-source-content-records"
                            for row in source_relations))
        content_contract = next(row for row in source_relations
                                if row.get("table_name") == "file-source-content-records")
        self.assertEqual(["file_content_digest_full", "file_source_content_bytes"],
                         content_contract["required_fields"])
        self.assertTrue(content_contract.get("provenance_citations"))
        rows = RULES.build_ladder_rows(manifest, {})
        content = next(row for row in rows if row["name"] == "file-source-content-records")
        self.assertEqual("source_content_store_relation", content["record_kind"])
        self.assertEqual("NOT_APPLICABLE_CONTENT_STORE", content["stage_marker_status"])

    def test_typed_input_is_record(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        self.assertNotIn("hld-typed-materialization-input-record",
                         {row["table_name"] for row in manifest["contracts"]})
        record = next(row for row in manifest["record_contracts"]
                      if row.get("record_kind") == "typed_interface"
                      and row.get("symbol") == "HldMaterializationInputRecord")
        self.assertEqual(["CompleteTypedInput", "IncompleteTypedInput"], record["completeness_enum_values"])
        self.assertEqual("CompleteTypedInput", record["input_semantics"]["required_completeness_value"])
        self.assertIn("may be empty", record["input_semantics"]["input_vectors"])

    def test_candidate_coverage_gaps_explicit(self):
        full = json.loads(RULE_PATH.with_name("itoa-full-graph-nodes.json").read_text())
        coverage = full.get("source_coverage", {})
        self.assertIn("whole_source_coverage_claimed", coverage)
        self.assertFalse(coverage.get("whole_source_coverage_claimed", True))
        self.assertTrue({"operators", "body_types", "literals", "repeated_same_line_sites"}
                        <= set(coverage["known_uncovered_constructs"]))

    def test_ladder_lists_outputs_first(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        rows = RULES.build_ladder_rows(manifest, {})
        expected = ["hld-current-interface-nodes", "hld-current-dependency-edges", "hld-edge-evidence-paths",
                    "hld-file-level-rollup", "hld-folder-level-rollup"]
        self.assertEqual(expected, [row["name"] for row in rows[:5]])
        self.assertTrue(all("export_status" in row and "stage_marker_status" in row
                            and "comparison_status" in row and "assertion_or_blocker" in row
                            and "backward_contract_layer" in row for row in rows))

    def test_ladder_contract_dag_order(self):
        manifest = json.loads(RULE_PATH.with_name("itoa-table-contract-manifest.json").read_text())
        names = [row["name"] for row in RULES.build_ladder_rows(manifest, {})]
        contract_names = [contract["table_name"] for contract in manifest["contracts"]]
        reported_contract_names = [name for name in names if name in set(contract_names)]
        self.assertEqual(len(contract_names), len(set(contract_names)))
        self.assertEqual(len(contract_names), len(reported_contract_names))
        self.assertEqual(set(contract_names), set(reported_contract_names))
        self.assertLess(names.index("edge-evidence-record-table"), names.index("lowered-signature-type-table"))
        self.assertLess(names.index("lowered-signature-type-table"), names.index("module-scope-binding-table"))
        self.assertLess(names.index("module-scope-binding-table"), names.index("syntax-namespace-input-items"))
        self.assertLess(names.index("syntax-namespace-input-items"), names.index("span-reference-text-table"))
        self.assertLess(names.index("span-reference-text-table"), names.index("file-source-content-records"))
        self.assertLess(names.index("file-source-content-records"), names.index("crate-dependency-graph-table"))
        self.assertGreater(names.index("stage-run-generation-records"), names.index("crate-dependency-graph-table"))

    def test_missing_required_export_blocks(self):
        result = RULES.validate_export_inventory({}, ["hld-current-interface-nodes"])
        self.assertEqual("BLOCKED", result["status"])

    def test_hld_outputs_required_exports(self):
        outputs = ["hld-current-interface-nodes", "hld-current-dependency-edges", "hld-edge-evidence-paths",
                   "hld-file-level-rollup", "hld-folder-level-rollup"]
        result = RULES.build_backward_report({"hld_output_table_names": outputs, "contracts": []}, {}, {})
        self.assertEqual(set(outputs), set(result["export_integrity"]["missing_required_exports"]))

    def test_summary_scope_mismatch_errors(self):
        exports = {"table": {"table_name": "table", "status": "OK", "generation": "g1", "attempt": "a1", "rows": []}}
        markers = {"owner": {"status": "complete", "evidence_kind": "banner_summary",
                             "generation": "g2", "attempt": "a1"}}
        result = RULES.validate_export_inventory(exports, ["table"], markers)
        self.assertIn("stage summary scope mismatch", " ".join(result["errors"]))
        self.assertEqual("BLOCKED", result["frozen_stage_gate"])

    def test_summary_scope_missing_unverified(self):
        exports = {"table": {"table_name": "table", "status": "OK", "generation": "g1", "attempt": "a1", "rows": []}}
        markers = {"owner": {"status": "complete", "evidence_kind": "banner_summary"}}
        result = RULES.validate_export_inventory(exports, ["table"], markers)
        self.assertEqual("UNVERIFIED", result["summary_scope_status"])
        self.assertEqual("BLOCKED", result["frozen_stage_gate"])

    def test_mismatched_header_name_errors(self):
        exports = {"expected": {"table_name": "wrong", "status": "OK", "generation": "g", "attempt": "a", "rows": []}}
        result = RULES.validate_export_inventory(exports, ["expected"])
        self.assertEqual("ERROR", result["status"])

    def test_mismatched_row_generation_errors(self):
        exports = {"table": {"table_name": "table", "status": "OK", "generation": "g", "attempt": "a",
                             "rows": [{"run_generation_digest_value": "other"}]}}
        result = RULES.validate_export_inventory(exports, ["table"])
        self.assertEqual("ERROR", result["status"])

    def test_stage_attempt_mismatch_errors(self):
        exports = {"a": {"table_name": "a", "status": "OK", "generation": "g", "attempt": "a1", "rows": []},
                   "b": {"table_name": "b", "status": "OK", "generation": "g", "attempt": "a2", "rows": []}}
        result = RULES.validate_export_inventory(exports, ["a", "b"])
        self.assertEqual("ERROR", result["status"])

    def test_marker_summary_is_unfrozen(self):
        result = RULES.validate_export_inventory({}, [], marker_evidence={
            "crate-graph": {"status": "complete", "source": "banner_summary"}})
        self.assertEqual("BLOCKED", result["frozen_stage_gate"])

    def test_ladder_verifier_stays_blocked(self):
        result = RULES.build_backward_report({}, {}, {})
        self.assertEqual("BLOCKED", result["verification_status"])

    def test_empty_exports_remain_blocked(self):
        outputs = ["hld-current-interface-nodes", "hld-current-dependency-edges",
                   "hld-edge-evidence-paths", "hld-file-level-rollup", "hld-folder-level-rollup"]
        exports = {name: {"table_name": name, "status": "OK", "generation": "g1", "attempt": "a1", "rows": []}
                   for name in outputs}
        markers = {"hld-materialization": {"status": "complete", "evidence_kind": "banner_summary",
                                          "generation": "g1", "attempt": "a1"}}
        result = RULES.build_backward_report({"hld_output_table_names": outputs, "contracts": []},
                                             exports, markers)
        self.assertEqual("VALID", result["export_integrity"]["status"])
        self.assertEqual("MATCHED_UNFROZEN", result["export_integrity"]["summary_scope_status"])
        self.assertEqual("BLOCKED", result["frozen_stage_gate"])
        self.assertEqual("BLOCKED", result["verification_status"])

    def test_ladder_manifest_is_immutable(self):
        source_dir = RULE_PATH.parent
        with tempfile.TemporaryDirectory() as temporary:
            bundle = Path(temporary) / "bundle"
            exports = Path(temporary) / "exports"
            outputs = Path(temporary) / "outputs"
            bundle.mkdir(); exports.mkdir()
            for name in ("itoa-backward-table-ladder.py", "itoa-reference-graph-rules.py"):
                shutil.copy2(source_dir / name, bundle / name)
            manifest_path = bundle / "itoa-table-contract-manifest.json"
            manifest_path.write_text('{"sentinel":"do-not-mutate"}\n')
            before = manifest_path.read_bytes()
            completed = subprocess.run([sys.executable, str(bundle / "itoa-backward-table-ladder.py"),
                                        "--export-directory", str(exports), "--output-directory", str(outputs),
                                        "--pin-file", str(Path(__file__).resolve().parents[3] / "docs/phase-20261111/phase-one-benchmark-pin.json"),
                                        "--source-root", "/home/ubuntu/repos/rust-tools/itoa"],
                                       check=False, capture_output=True, text=True)
            report = json.loads((outputs / "itoa-backward-table-report.json").read_text())
            self.assertEqual(before, manifest_path.read_bytes())
            self.assertEqual("BLOCKED", report["verification_status"])

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
