"""Persist complete S1/S2 inventories; fail rather than accepting partial pages."""
import json
import subprocess
import sys
from pathlib import Path


def collect_index_coverage_inventory(corpus_repository_short_name, corpus_output_directory_path):
    project_identifier_string_value = "home-ubuntu-repos-rust-tools-" + corpus_repository_short_name
    project_argument_vector_values = ["--project", project_identifier_string_value]
    for external_tool_command_name, output_artifact_file_name in [
        ("index_status", "codebase-index-status-values.txt"),
        ("get_graph_schema", "codebase-graph-schema-values.txt"),
    ]:
        output_source_artifact_path = corpus_output_directory_path / output_artifact_file_name
        if not output_source_artifact_path.exists():
            output_source_artifact_path.write_text(subprocess.check_output(
                ["codebase-memory-mcp", "cli", external_tool_command_name, *project_argument_vector_values], text=True
            ))
    scope_page_offset_value = 0
    scope_coverage_entry_rows = []
    scope_page_ordinal_value = 0
    scope_literal_sop_pass = True
    scope_output_generation_value = None
    while True:
        scope_output_artifact_path = corpus_output_directory_path / f"codebase-index-coverage-page-{scope_page_ordinal_value}.json"
        if not scope_output_artifact_path.exists():
            scope_output_artifact_path.write_text(subprocess.check_output([
                "codebase-memory-mcp", "cli", "check_index_coverage", *project_argument_vector_values,
                "--scopes", '["."]', "--scope-limit", "20", "--scope-offset", str(scope_page_offset_value),
                "--diagnostics", "full", "--format", "json",
            ], text=True))
        scope_output_result_value = json.loads(scope_output_artifact_path.read_text())
        scope_metadata_record_value = scope_output_result_value["metadata"]
        if scope_output_generation_value is None:
            scope_output_generation_value = scope_output_result_value["indexed_at"]
        elif scope_output_generation_value != scope_output_result_value["indexed_at"]:
            raise RuntimeError("scope pages cross index generations")
        if scope_metadata_record_value["recording_status"] != "complete" or not scope_metadata_record_value["generation_matches"]:
            raise RuntimeError(f"{corpus_repository_short_name}: incomplete or stale scope generation")
        scope_coverage_entry_rows.extend(scope_output_result_value["scopes"][0]["entries"])
        scope_page_ordinal_value += 1
        if not scope_output_result_value["has_more"]:
            break
        scope_next_offset_value = scope_output_result_value["next_offset"]
        if scope_next_offset_value <= scope_page_offset_value:
            raise RuntimeError("scope offset did not advance")
        scope_page_offset_value = scope_next_offset_value
    if len(scope_coverage_entry_rows) != scope_output_result_value["scope_total"]:
        raise RuntimeError("scope entry count does not match total")
    missed_page_cursor_value = None
    missed_source_file_rows = []
    missed_page_ordinal_value = 0
    while True:
        missed_output_artifact_path = corpus_output_directory_path / f"codebase-missed-source-page-{missed_page_ordinal_value}.json"
        if not missed_output_artifact_path.exists():
            missed_output_artifact_path.write_text(subprocess.check_output([
                "codebase-memory-mcp", "cli", "query_graph", *project_argument_vector_values,
                "--graph", "missed", "--query", "MATCH (f:File) RETURN f.file_path",
                "--max-rows", "200", "--format", "json",
                *(["--cursor", missed_page_cursor_value] if missed_page_cursor_value else []),
            ], text=True))
        missed_output_result_value = json.loads(missed_output_artifact_path.read_text())
        missed_source_file_rows.extend(missed_output_result_value["rows"])
        missed_page_ordinal_value += 1
        if not missed_output_result_value["has_more"]:
            break
        missed_next_cursor_value = missed_output_result_value["next_cursor"]
        if not missed_next_cursor_value or missed_next_cursor_value == missed_page_cursor_value:
            raise RuntimeError("missed graph cursor did not advance")
        missed_page_cursor_value = missed_next_cursor_value
    if len(missed_source_file_rows) != missed_output_result_value["total"]:
        raise RuntimeError("missed File inventory does not match total")
    index_output_result_value = json.loads((corpus_output_directory_path / "codebase-index-result-log.txt").read_text())
    coverage_summary_record_value = {
        "corpus_repository_short_name": corpus_repository_short_name,
        "index_generation_timestamp_value": scope_output_result_value["indexed_at"],
        "index_graph_node_count": index_output_result_value["nodes"],
        "index_graph_edge_count": index_output_result_value["edges"],
        "index_partial_file_count": index_output_result_value.get("parse_partial_count", 0),
        "index_unusable_file_count": index_output_result_value.get("parse_unusable_count", 0),
        "scope_coverage_entry_count": len(scope_coverage_entry_rows),
        "scope_coverage_page_count": scope_page_ordinal_value,
        "scope_literal_sop_pass": scope_literal_sop_pass,
        "missed_source_file_count": len(missed_source_file_rows),
        "missed_source_page_count": missed_page_ordinal_value,
    }
    (corpus_output_directory_path / "corpus-index-coverage-summary.json").write_text(
        json.dumps(coverage_summary_record_value, indent=2) + "\n"
    )
    return coverage_summary_record_value


if __name__ == "__main__":
    corpus_workspace_root_path = Path(__file__).resolve().parent.parent
    corpus_manifest_source_path = corpus_workspace_root_path / "docs/phase-20261111/corpus-source-pin-manifest.json"
    corpus_literal_sop_pass = True
    for corpus_source_record_value in json.loads(corpus_manifest_source_path.read_text()):
        corpus_summary_record_value = collect_index_coverage_inventory(
            corpus_source_record_value["corpus_repository_short_name"],
            corpus_workspace_root_path / corpus_source_record_value["corpus_evidence_directory_path"],
        )
        print(json.dumps(corpus_summary_record_value), flush=True)
        corpus_literal_sop_pass = corpus_literal_sop_pass and corpus_summary_record_value["scope_literal_sop_pass"]
    sys.exit(0 if corpus_literal_sop_pass else 1)
