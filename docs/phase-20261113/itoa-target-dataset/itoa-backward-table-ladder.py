#!/usr/bin/env python3
"""Read-only, fail-closed report over exports and the preceding-table contract manifest."""

import argparse
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
RULE_SPEC = importlib.util.spec_from_file_location("itoa_reference_graph_rules", HERE / "itoa-reference-graph-rules.py")
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)
DEFAULT_EXPORTS = Path.home() / "hld-current-test-artifacts/20261113T180000Z-itoa-slice-2/ladder-baseline/exported-table-rows"
OUTPUTS = ["hld-current-interface-nodes", "hld-current-dependency-edges", "hld-edge-evidence-paths",
           "hld-file-level-rollup", "hld-folder-level-rollup"]


def load_export(path):
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    if not rows or rows[0].get("export_record_kind_value") != "header":
        raise ValueError(f"missing export header: {path.name}")
    header = rows[0]
    scope = header.get("export_scope_identity_record", {})
    if header.get("export_owned_table_name") != path.stem:
        raise ValueError(f"table-name header mismatch: {path.name}")
    payload_rows = []
    for envelope in rows[1:]:
        payload = envelope.get("export_row_payload_value", {})
        if isinstance(payload, str):
            try:
                payload = json.loads(payload)
            except json.JSONDecodeError as error:
                raise ValueError(f"invalid row payload JSON in {path.name}") from error
        payload_rows.append({**envelope, **(payload if isinstance(payload, dict) else {})})
    return {"table_name": header["export_owned_table_name"], "status": "OK",
            "generation": scope.get("run_generation_digest_value"),
            "attempt": scope.get("stage_attempt_sequence_value"), "rows": payload_rows}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--export-directory", type=Path, default=DEFAULT_EXPORTS)
    parser.add_argument("--output-directory", type=Path, default=HERE)
    parser.add_argument("--manifest-file", type=Path, default=HERE / "itoa-table-contract-manifest.json")
    parser.add_argument("--pin-file", type=Path, default=ROOT / "docs/phase-20261111/phase-one-benchmark-pin.json")
    parser.add_argument("--source-root", type=Path, default=Path("/home/ubuntu/repos/rust-tools/itoa"))
    parser.add_argument("--stage-summary-file", type=Path)
    args = parser.parse_args()
    manifest = json.loads(args.manifest_file.read_text())
    exports, errors = {}, []
    if args.export_directory.exists():
        export_files = set(args.export_directory.glob("*.json")) | set(args.export_directory.glob("*.jsonl"))
        for path in sorted(export_files):
            try:
                exports[path.stem] = load_export(path)
            except (OSError, ValueError, json.JSONDecodeError) as error:
                errors.append(f"{path.name}: {error}")
        for path in sorted(args.export_directory.glob("*.err")):
            if path.read_text().strip():
                errors.append(f"{path.name}: {path.read_text().strip()[:500]}")
    marker_evidence = {}
    if args.stage_summary_file:
        if not args.stage_summary_file.exists():
            errors.append(f"stage summary file missing: {args.stage_summary_file}")
        else:
            try:
                summary = json.loads(args.stage_summary_file.read_text())
                scope = summary.get("export_scope_identity_record", summary.get("scope_identity_record", {}))
                generation = summary.get("run_generation_digest_value", scope.get("run_generation_digest_value", summary.get("generation")))
                attempt = summary.get("stage_attempt_sequence_value", scope.get("stage_attempt_sequence_value", summary.get("attempt")))
                for group in ("stage_publication_status_values", "stage_implementation_status_values"):
                    for owner, status in summary.get(group, {}).items():
                        marker_evidence[owner] = {"status": status, "generation": generation,
                            "attempt": attempt, "evidence_kind": "banner_summary"}
            except (OSError, json.JSONDecodeError) as error:
                errors.append(f"stage summary invalid: {error}")
    reference_statuses = {}
    for item in manifest.get("reference_inputs", []):
        path = args.manifest_file.parent / item["name"]
        reference_statuses[item["name"]] = "LOCAL_REFERENCE_AVAILABLE" if path.is_file() else "LOCAL_REFERENCE_MISSING"
    report = RULES.build_backward_report(manifest, exports, marker_evidence, reference_statuses)
    if errors:
        report["export_integrity"]["status"] = "ERROR"
        report["export_integrity"]["errors"].extend(errors)
    pin = json.loads(args.pin_file.read_text())
    expected = {row["path"]: row["sha256"] for row in pin["benchmark_source_inventory_records"]}
    source_path = args.source_root / "src/lib.rs"
    source_bytes = source_path.read_bytes() if source_path.exists() else b""
    digest = expected.get("src/lib.rs", "")
    checkpoint = RULES.verify_checkpoint_source(source_bytes, digest, 2506, 2519, "Buffer::new()")
    checkpoint.update({"expected_relationship": "<Buffer as Default>::default -> Buffer::new",
                       "source_path": "src/lib.rs", "source_checked_path": str(source_path),
                       "source_pin_file": str(args.pin_file), "expected_source_sha256": digest,
                       "source_check_method": "local source bytes checked against pinned digest and byte range",
                       "semantic_row_status": "BLOCKED_NOT_CHECKED",
                       "semantic_comparison_status": "NOT_IMPLEMENTED_CANONICAL_TARGET_MAPPING",
                       "canonical_mapping_status": "NOT_IMPLEMENTED"})
    report.update({"checkpoint": checkpoint, "errors": errors,
                   "verification_status": "BLOCKED", "production_database_written": False,
                   "report_scope": "export integrity and reference contracts only; not canonical mapping or full HLD qualification"})
    output = args.output_directory
    output.mkdir(parents=True, exist_ok=True)
    (output / "itoa-backward-table-report.json").write_text(RULES.serialize_dataset(report))
    lines = ["# itoa backward-table verification", "", "Status: **BLOCKED**", "",
             f"Export integrity: {report['export_integrity']['status']}",
             f"Frozen stage gate: {report['frozen_stage_gate']} ({report['export_integrity']['frozen_stage_gate_reason']})",
             f"Stage summary scope: {report['export_integrity']['summary_scope_status']}",
             f"Source evidence: {checkpoint['source_evidence_status']}; generation link: {checkpoint['source_identity_link_status']}",
             f"Semantic target: {checkpoint['semantic_row_status']} ({checkpoint['semantic_comparison_status']})", "",
             "No canonical mapping/full HLD comparison is implemented; overall verification stays BLOCKED.", "",
             "## Backward ladder", ""]
    lines.extend(f"- `{row['name']}` — export `{row['export_status']}`, marker `{row['stage_marker_status']}`, comparison `{row['comparison_status']}`; {row['assertion_or_blocker']}" for row in report["ladder_rows"])
    (output / "itoa-backward-table-report.md").write_text("\n".join(lines) + "\n")
    print((output / "itoa-backward-table-report.md").read_text())


if __name__ == "__main__":
    main()
