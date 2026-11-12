"""Persist fresh S1/S2 inventory pages and reject incomplete index coverage."""

import json
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

SCOPE_PAGE_LIMIT = 20
MISSED_FILE_PAGE_LIMIT = 200


class CoverageInventoryBlockedError(RuntimeError):
    """Coverage output is absent, stale, malformed, or incomplete."""


def run_indexer_command_with_guard(
    command_values, before_after_callback=None, response_file_path=None, command_event_callback=None
):
    if before_after_callback is not None:
        before_after_callback("before", command_values)
    if command_event_callback is not None:
        command_event_callback("before", command_values, {"status": "STARTED"})
    completed = None
    command_error = None
    try:
        completed = subprocess.run(command_values, capture_output=True, check=False)
    except Exception as error:
        command_error = error
        raise
    finally:
        try:
            if before_after_callback is not None:
                before_after_callback("after", command_values)
        finally:
            if command_event_callback is not None:
                if completed is None:
                    outcome = {
                        "status": "ERROR",
                        "return_code": None,
                        "exception_type": type(command_error).__name__ if command_error is not None else None,
                    }
                else:
                    stdout_size = len(completed.stdout if isinstance(completed.stdout, bytes)
                                      else str(completed.stdout).encode("utf-8"))
                    stderr_size = len(completed.stderr if isinstance(completed.stderr, bytes)
                                      else str(completed.stderr).encode("utf-8"))
                    outcome = {
                        "status": "COMPLETED" if completed.returncode == 0 else "FAILED",
                        "return_code": completed.returncode,
                        "stdout_bytes": stdout_size,
                        "stderr_bytes": stderr_size,
                    }
                command_event_callback("after", command_values, outcome)
    if completed.returncode != 0:
        if response_file_path is not None:
            record_fresh_response_file(response_file_path, completed.stdout)
            error_path = Path(response_file_path).with_name(Path(response_file_path).stem + "-error-log.txt")
            record_fresh_response_file(error_path, completed.stderr)
        raise CoverageInventoryBlockedError(
            f"indexer command exited {completed.returncode}: {command_values!r}: {completed.stderr.strip()}"
        )
    if response_file_path is not None:
        record_fresh_response_file(response_file_path, completed.stdout)
    return completed.stdout


def read_json_response_from_command(
    command_values, before_after_callback=None, response_file_path=None, command_event_callback=None
):
    response_text = run_indexer_command_with_guard(
        command_values, before_after_callback, response_file_path, command_event_callback
    )
    try:
        response = json.loads(response_text)
    except (TypeError, json.JSONDecodeError) as error:
        raise CoverageInventoryBlockedError(f"indexer returned malformed JSON: {error}") from error
    if not isinstance(response, dict):
        raise CoverageInventoryBlockedError("indexer JSON response must be an object")
    return response, response_text


def record_fresh_response_file(output_path, response_text):
    if output_path.exists():
        raise CoverageInventoryBlockedError(f"refusing to reuse prior response: {output_path}")
    if isinstance(response_text, bytes):
        output_path.write_bytes(response_text)
    else:
        output_path.write_text(response_text)


def unwrap_boundary_response(page_record):
    if isinstance(page_record, dict) and "response" in page_record:
        return page_record["response"], page_record
    return page_record, page_record


def validate_scope_response_pages(scope_pages, require_complete=True, expected_generation=None):
    if not scope_pages:
        raise CoverageInventoryBlockedError("scope coverage returned no pages")
    generation = None
    expected_total = None
    accumulated_entries = 0
    seen_scope_entry_content = {}
    seen_offsets = set()
    previous_offset = None
    for page_index, page in enumerate(scope_pages):
        response, boundary = unwrap_boundary_response(page)
        try:
            metadata = response["metadata"]
            recording_status = metadata["recording_status"]
            generation_matches = metadata["generation_matches"]
            requested_offset = boundary.get("boundary_offset", response.get("requested_offset"))
            page_total = response["scope_total"]
            scope_rows = response["scopes"]
            entries = scope_rows[0]["entries"]
            has_more = response["has_more"]
        except (KeyError, IndexError, TypeError) as error:
            raise CoverageInventoryBlockedError(f"malformed scope page {page_index}: {error}") from error
        native_generation = response.get("index_generation", response.get("indexed_at"))
        if native_generation is not None and (
            not isinstance(native_generation, str) or not native_generation
        ):
            raise CoverageInventoryBlockedError("scope response has a malformed native generation")
        if native_generation is not None:
            if generation is None:
                generation = native_generation
            elif generation != native_generation:
                raise CoverageInventoryBlockedError("scope pages cross index generations")
        elif expected_generation is None:
            raise CoverageInventoryBlockedError("scope page has no native generation or boundary binding")
        if recording_status != "complete" or generation_matches is not True:
            raise CoverageInventoryBlockedError("scope page is stale or incomplete")
        if (not isinstance(requested_offset, int) or isinstance(requested_offset, bool)
                or requested_offset < 0 or requested_offset in seen_offsets):
            raise CoverageInventoryBlockedError("scope offset is malformed or repeated")
        if previous_offset is not None and requested_offset <= previous_offset:
            raise CoverageInventoryBlockedError("scope offsets did not strictly advance")
        if not isinstance(page_total, int) or isinstance(page_total, bool) or page_total < 0:
            raise CoverageInventoryBlockedError("scope total is malformed")
        if not isinstance(scope_rows, list) or len(scope_rows) != 1:
            raise CoverageInventoryBlockedError("scope query must return exactly one requested scope")
        if expected_total is None:
            expected_total = page_total
        elif expected_total != page_total:
            raise CoverageInventoryBlockedError("scope totals changed between pages")
        if not isinstance(entries, list) or not isinstance(has_more, bool):
            raise CoverageInventoryBlockedError("scope entries or pagination marker is malformed")
        for entry in entries:
            if not isinstance(entry, dict):
                raise CoverageInventoryBlockedError("scope entry is not an object")
            entry_path = entry.get("path")
            if not isinstance(entry_path, str) or not entry_path:
                raise CoverageInventoryBlockedError("scope entry has no path identity")
            if entry_path in seen_scope_entry_content:
                if seen_scope_entry_content[entry_path] == entry:
                    raise CoverageInventoryBlockedError(f"duplicate scope path identity: {entry_path}")
                raise CoverageInventoryBlockedError(f"conflicting scope content for path: {entry_path}")
            seen_scope_entry_content[entry_path] = entry
        seen_offsets.add(requested_offset)
        previous_offset = requested_offset
        accumulated_entries += len(entries)
        if accumulated_entries > expected_total:
            raise CoverageInventoryBlockedError("scope entries exceed the stable total")
        if has_more:
            maximum_page_count = max(1, (expected_total + SCOPE_PAGE_LIMIT - 1) // SCOPE_PAGE_LIMIT)
            if page_index + 1 >= maximum_page_count:
                raise CoverageInventoryBlockedError("scope pagination exceeds the stable total")
            try:
                next_offset = response["next_offset"]
            except KeyError as error:
                raise CoverageInventoryBlockedError("scope continuation has no next offset") from error
            if (not isinstance(next_offset, int) or isinstance(next_offset, bool)
                    or next_offset <= requested_offset or next_offset in seen_offsets):
                raise CoverageInventoryBlockedError("scope continuation offset does not advance")
            if page_index == len(scope_pages) - 1 and require_complete:
                raise CoverageInventoryBlockedError("scope page sequence ends before pagination completes")
        elif page_index != len(scope_pages) - 1:
            raise CoverageInventoryBlockedError("scope page sequence continues after completion")
        elif accumulated_entries != expected_total:
            raise CoverageInventoryBlockedError(
                f"scope entry count {accumulated_entries} does not match total {expected_total}"
            )
    if require_complete and accumulated_entries != expected_total:
        raise CoverageInventoryBlockedError(
            f"scope entry count {accumulated_entries} does not match total {expected_total}"
        )
    if expected_generation is not None and generation is not None and generation != expected_generation:
        raise CoverageInventoryBlockedError("scope response generation differs from boundary generation")
    return {
        "generation": expected_generation or generation,
        "total": expected_total,
        "entry_count": accumulated_entries,
    }


def validate_missed_response_pages(missed_pages, expected_generation, require_complete=True):
    if not missed_pages:
        raise CoverageInventoryBlockedError("missed-File query returned no pages")
    expected_total = None
    accumulated_rows = 0
    seen_missed_file_content = {}
    seen_cursors = set()
    previous_cursor = None
    for page_index, page in enumerate(missed_pages):
        response, boundary = unwrap_boundary_response(page)
        try:
            columns = response["columns"]
            rows = response["rows"]
            returned_rows = response["returned"]
            total_relation = response["total_relation"]
            has_more = response["has_more"]
            page_total = response["total"]
        except (KeyError, TypeError) as error:
            raise CoverageInventoryBlockedError(f"malformed missed-File page {page_index}: {error}") from error
        native_generation = response.get("index_generation", response.get("indexed_at"))
        if native_generation is not None and native_generation != expected_generation:
            raise CoverageInventoryBlockedError("missed-File pages are not bound to the fresh index generation")
        if columns != ["f.file_path"]:
            raise CoverageInventoryBlockedError("missed-File query columns do not match the path identity contract")
        if not isinstance(rows, list) or not isinstance(has_more, bool):
            raise CoverageInventoryBlockedError("missed-File rows or pagination marker is malformed")
        if (not isinstance(returned_rows, int) or isinstance(returned_rows, bool)
                or returned_rows != len(rows) or total_relation != "eq"):
            raise CoverageInventoryBlockedError("missed-File returned count or exact-total relation is malformed")
        if not isinstance(page_total, int) or isinstance(page_total, bool) or page_total < 0:
            raise CoverageInventoryBlockedError("missed-File total is malformed")
        if expected_total is None:
            expected_total = page_total
        elif expected_total != page_total:
            raise CoverageInventoryBlockedError("missed-File totals changed between pages")
        for row in rows:
            if not isinstance(row, list) or len(row) != 1:
                raise CoverageInventoryBlockedError("missed-File row does not match the path-column shape")
            file_path = row[0]
            if not isinstance(file_path, str) or not file_path:
                raise CoverageInventoryBlockedError("missed-File row has no path identity")
            if file_path in seen_missed_file_content:
                if seen_missed_file_content[file_path] == row:
                    raise CoverageInventoryBlockedError(f"duplicate missed-File path identity: {file_path}")
                raise CoverageInventoryBlockedError(f"conflicting missed-File content for path: {file_path}")
            seen_missed_file_content[file_path] = row
        accumulated_rows += len(rows)
        if accumulated_rows > expected_total:
            raise CoverageInventoryBlockedError("missed-File rows exceed the stable total")
        if has_more:
            maximum_page_count = max(1, (expected_total + MISSED_FILE_PAGE_LIMIT - 1)
                                     // MISSED_FILE_PAGE_LIMIT)
            if page_index + 1 >= maximum_page_count:
                raise CoverageInventoryBlockedError("missed-File pagination exceeds the stable total")
            try:
                next_cursor = response["next_cursor"]
            except KeyError as error:
                raise CoverageInventoryBlockedError("missed-File continuation has no cursor") from error
            if not isinstance(next_cursor, str) or not next_cursor or next_cursor in seen_cursors:
                raise CoverageInventoryBlockedError("missed-File cursor is malformed or repeated")
            boundary_cursor = boundary.get("boundary_cursor")
            if "response" in page and boundary_cursor != previous_cursor:
                raise CoverageInventoryBlockedError("missed-File request cursor does not match prior continuation")
            if next_cursor == previous_cursor or next_cursor == boundary_cursor:
                raise CoverageInventoryBlockedError("missed-File cursor did not advance")
            if page_index == len(missed_pages) - 1 and require_complete:
                raise CoverageInventoryBlockedError("missed-File sequence ends before pagination completes")
            seen_cursors.add(next_cursor)
            previous_cursor = next_cursor
        elif page_index != len(missed_pages) - 1:
            raise CoverageInventoryBlockedError("missed-File sequence continues after completion")
        elif accumulated_rows != expected_total:
            raise CoverageInventoryBlockedError(
                f"missed-File row count {accumulated_rows} does not match total {expected_total}"
            )
    if require_complete and accumulated_rows != expected_total:
        raise CoverageInventoryBlockedError(
            f"missed-File row count {accumulated_rows} does not match total {expected_total}"
        )
    return {"generation": expected_generation, "total": expected_total, "row_count": accumulated_rows}


def write_response_digest_manifest(
    report_directory_path, artifact_directory_path, file_paths, digest_file_callback, summary
):
    entries = []
    for file_path in sorted(file_paths):
        content = file_path.read_bytes()
        digest = digest_file_callback(file_path)
        if (not isinstance(digest, str) or len(digest) != 64
                or any(character not in "0123456789abcdef" for character in digest)):
            raise CoverageInventoryBlockedError(f"BLAKE3 callback returned an invalid digest for {file_path}")
        entries.append({
            "path": str(file_path.resolve()),
            "artifact_relative_path": file_path.relative_to(artifact_directory_path).as_posix(),
            "size_bytes": len(content),
            "blake3": digest,
        })
    manifest_path = report_directory_path / "coverage-response-digest-manifest.json"
    if manifest_path.exists():
        raise CoverageInventoryBlockedError(f"refusing to overwrite response digest manifest: {manifest_path}")
    manifest_path.write_text(json.dumps({
        "algorithm": "blake3",
        "artifact_root": str(artifact_directory_path.resolve()),
        "summary": summary,
        "files": entries,
    }, indent=2) + "\n")
    return manifest_path


def command_line_blake3_digest_provider():
    try:
        import blake3
    except ImportError:
        b3sum_path = shutil.which("b3sum")
        if b3sum_path is None:
            raise CoverageInventoryBlockedError("BLAKE3 provider unavailable (install blake3 or b3sum)")

        def digest_file(file_path):
            completed = subprocess.run([b3sum_path, str(file_path)], capture_output=True, check=False)
            if completed.returncode != 0:
                raise CoverageInventoryBlockedError(f"b3sum failed for {file_path}: {completed.stderr.decode(errors='replace')}")
            return completed.stdout.decode("ascii").split()[0]

        return digest_file

    def digest_file(file_path):
        digest = blake3.blake3()
        with Path(file_path).open("rb") as input_file:
            for content_chunk in iter(lambda: input_file.read(1024 * 1024), b""):
                digest.update(content_chunk)
        return digest.hexdigest()

    return digest_file


def collect_index_coverage_inventory(
    corpus_repository_short_name,
    corpus_output_directory_path,
    indexer_path="codebase-memory-mcp",
    project_identifier=None,
    expected_generation=None,
    before_after_callback=None,
    on_metadata_complete=None,
    raw_response_directory_path=None,
    digest_file_callback=None,
    command_event_callback=None,
):
    corpus_output_directory_path = Path(corpus_output_directory_path)
    corpus_output_directory_path.mkdir(parents=True, exist_ok=True)
    if raw_response_directory_path is None or digest_file_callback is None:
        raise CoverageInventoryBlockedError("external raw-response directory and BLAKE3 callback are required")
    raw_response_directory_path = Path(raw_response_directory_path)
    if raw_response_directory_path.resolve() == corpus_output_directory_path.resolve():
        raise CoverageInventoryBlockedError("raw index responses must stay outside the committed report directory")
    raw_response_directory_path.mkdir(parents=True, exist_ok=True)
    project_identifier = project_identifier or f"home-ubuntu-repos-rust-tools-{corpus_repository_short_name}"
    project_arguments = ["--project", project_identifier]
    response_files = []
    existing_responses = list(raw_response_directory_path.glob("codebase-*-values.txt"))
    existing_responses.extend(raw_response_directory_path.glob("codebase-*-page-*.json"))
    if existing_responses:
        raise CoverageInventoryBlockedError("refusing to reuse saved external index response pages")

    index_result_path = raw_response_directory_path / "codebase-index-result-log.txt"
    if not index_result_path.is_file():
        raise CoverageInventoryBlockedError("fresh index result is missing")
    try:
        index_result = json.loads(index_result_path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise CoverageInventoryBlockedError(f"fresh index result is malformed: {error}") from error
    if index_result.get("status") != "indexed":
        raise CoverageInventoryBlockedError("full index did not report a succeeded generation")
    index_generation = index_result.get("index_generation", index_result.get("indexed_at"))
    if not isinstance(index_generation, str) or not index_generation:
        raise CoverageInventoryBlockedError("fresh full index result has no native generation")
    if (index_result.get("indexed_at") is not None
            and index_result.get("index_generation") is not None
            and index_result["indexed_at"] != index_result["index_generation"]):
        raise CoverageInventoryBlockedError("fresh full index generation fields disagree")
    if not all(isinstance(index_result.get(key), int) and not isinstance(index_result[key], bool)
               and index_result[key] >= 0 for key in ("nodes", "edges")):
        raise CoverageInventoryBlockedError("fresh index result is missing graph totals")
    response_files.append(index_result_path)

    projects_path = raw_response_directory_path / "codebase-list-projects-values.txt"
    projects_text = run_indexer_command_with_guard(
        [indexer_path, "cli", "list_projects"], before_after_callback, projects_path,
        command_event_callback,
    )
    response_files.append(projects_path)
    projects_value = projects_text.decode("utf-8", errors="replace") if isinstance(projects_text, bytes) else projects_text
    if project_identifier not in projects_value:
        raise CoverageInventoryBlockedError("fresh index project is absent from list_projects output")

    status_path = raw_response_directory_path / "codebase-index-status-values.txt"
    status_result, status_text = read_json_response_from_command(
        [indexer_path, "cli", "index_status", *project_arguments, "--diagnostics", "full", "--format", "json"],
        before_after_callback, status_path, command_event_callback,
    )
    response_files.append(status_path)
    if status_result.get("status") not in {"ready", "indexed"}:
        raise CoverageInventoryBlockedError("index status is not ready")
    generation = status_result.get("indexed_at")
    if not isinstance(generation, str) or not generation:
        raise CoverageInventoryBlockedError("index status has no generation timestamp")
    if index_generation != generation:
        raise CoverageInventoryBlockedError("index status generation differs from completed full index")
    if (not all(isinstance(status_result.get(key), int) and not isinstance(status_result[key], bool)
                and status_result[key] >= 0 for key in ("nodes", "edges"))
            or status_result.get("nodes") != index_result.get("nodes")
            or status_result.get("edges") != index_result.get("edges")):
        raise CoverageInventoryBlockedError("index status totals differ from completed full index")

    schema_path = raw_response_directory_path / "codebase-graph-schema-values.txt"
    schema_result, schema_text = read_json_response_from_command(
        [indexer_path, "cli", "get_graph_schema", *project_arguments,
         "--format", "json", "--diagnostics", "full", "--limit", "100", "--offset", "0"],
        before_after_callback, schema_path, command_event_callback,
    )
    response_files.append(schema_path)
    if not isinstance(schema_result.get("node_labels"), list) or not isinstance(schema_result.get("edge_labels"), list):
        raise CoverageInventoryBlockedError("graph schema response is incomplete")
    if on_metadata_complete is not None:
        on_metadata_complete({
            "generation": generation,
            "node_count": status_result["nodes"],
            "edge_count": status_result["edges"],
            "schema_node_label_count": len(schema_result["node_labels"]),
            "schema_edge_label_count": len(schema_result["edge_labels"]),
        })

    scope_pages = []
    scope_offset = 0
    scope_page_index = 0
    while True:
        command = [indexer_path, "cli", "check_index_coverage", *project_arguments,
                   "--scopes", '["."]', "--scope-limit", str(SCOPE_PAGE_LIMIT), "--scope-offset", str(scope_offset),
                   "--diagnostics", "full", "--format", "json"]
        page_path = raw_response_directory_path / f"codebase-index-coverage-page-{scope_page_index}.json"
        page, page_text = read_json_response_from_command(
            command, before_after_callback, page_path, command_event_callback
        )
        response_files.append(page_path)
        scope_pages.append({"response": page, "boundary_offset": scope_offset})
        validate_scope_response_pages(scope_pages, require_complete=False, expected_generation=generation)
        if page.get("has_more") is False:
            break
        next_offset = page.get("next_offset")
        if (not isinstance(next_offset, int) or isinstance(next_offset, bool)
                or next_offset <= scope_offset):
            raise CoverageInventoryBlockedError("scope offset did not strictly advance")
        scope_offset = next_offset
        scope_page_index += 1
    scope_summary = validate_scope_response_pages(
        scope_pages, require_complete=True, expected_generation=generation
    )
    if scope_summary["generation"] != generation:
        raise CoverageInventoryBlockedError("scope generation differs from fresh index status")

    missed_pages = []
    cursor = None
    missed_page_index = 0
    seen_cursors = set()
    while True:
        if cursor is not None:
            if cursor in seen_cursors:
                raise CoverageInventoryBlockedError("missed-File cursor cycle detected before query")
            seen_cursors.add(cursor)
        command = [indexer_path, "cli", "query_graph", *project_arguments,
                   "--graph", "missed", "--query", "MATCH (f:File) RETURN f.file_path",
                   "--max-rows", str(MISSED_FILE_PAGE_LIMIT), "--format", "json"]
        if cursor is not None:
            command.extend(["--cursor", cursor])
        page_path = raw_response_directory_path / f"codebase-missed-source-page-{missed_page_index}.json"
        page, page_text = read_json_response_from_command(
            command, before_after_callback, page_path, command_event_callback
        )
        response_files.append(page_path)
        missed_pages.append({"response": page, "boundary_cursor": cursor})
        validate_missed_response_pages(missed_pages, generation, require_complete=False)
        if page.get("has_more") is False:
            break
        next_cursor = page.get("next_cursor")
        if (not isinstance(next_cursor, str) or not next_cursor or next_cursor == cursor
                or next_cursor in seen_cursors):
            raise CoverageInventoryBlockedError("missed-File cursor did not strictly advance")
        cursor = next_cursor
        missed_page_index += 1
    missed_summary = validate_missed_response_pages(missed_pages, generation, require_complete=True)
    final_status_path = raw_response_directory_path / "codebase-index-status-final-values.txt"
    final_status, final_status_text = read_json_response_from_command(
        [indexer_path, "cli", "index_status", *project_arguments, "--diagnostics", "full", "--format", "json"],
        before_after_callback, final_status_path, command_event_callback,
    )
    response_files.append(final_status_path)
    if (not all(isinstance(final_status.get(key), int) and not isinstance(final_status[key], bool)
                and final_status[key] >= 0 for key in ("nodes", "edges"))
            or final_status.get("indexed_at") != generation
            or final_status.get("nodes") != status_result.get("nodes")
            or final_status.get("edges") != status_result.get("edges")):
        raise CoverageInventoryBlockedError("index generation or totals changed during coverage pagination")
    coverage_summary = {
        "corpus_repository_short_name": corpus_repository_short_name,
        "project_identifier": project_identifier,
        "index_generation_timestamp_value": generation,
        "index_graph_node_count": index_result["nodes"],
        "index_graph_edge_count": index_result["edges"],
        "index_partial_file_count": index_result.get("parse_partial_count"),
        "index_unusable_file_count": index_result.get("parse_unusable_count"),
        "scope_coverage_entry_count": scope_summary["entry_count"],
        "scope_coverage_page_count": len(scope_pages),
        "scope_truncated_informational": any(
            unwrap_boundary_response(page)[0].get("scope_truncated") is True for page in scope_pages
        ),
        "scope_literal_sop_pass": True,
        "missed_source_file_count": missed_summary["row_count"],
        "missed_source_page_count": len(missed_pages),
    }
    digest_manifest_path = write_response_digest_manifest(
        corpus_output_directory_path, raw_response_directory_path, response_files,
        digest_file_callback, coverage_summary,
    )
    coverage_summary["response_digest_manifest_path"] = str(digest_manifest_path)
    coverage_summary["response_digest_manifest_algorithm"] = "blake3"
    summary_path = corpus_output_directory_path / "corpus-index-coverage-summary.json"
    if summary_path.exists():
        raise CoverageInventoryBlockedError("refusing to overwrite coverage summary")
    summary_path.write_text(json.dumps(coverage_summary, indent=2) + "\n")
    return coverage_summary


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository_names", nargs="*")
    parser.add_argument("--indexer-path", default="codebase-memory-mcp")
    parser.add_argument("--project-root", action="append", default=[], metavar="NAME=PATH")
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--artifact-root", type=Path,
                        default=Path.home() / "hld-current-test-artifacts")
    parser.add_argument("--run-id", default=f"coverage-{time.time_ns()}")
    arguments = parser.parse_args()
    if Path(arguments.run_id).name != arguments.run_id or arguments.run_id in {".", ".."}:
        parser.error("--run-id must be one safe path component")
    workspace_root = Path(__file__).resolve().parent.parent
    manifest_path = arguments.manifest or workspace_root / "docs/phase-20261111/corpus-source-pin-manifest.json"
    manifest = json.loads(manifest_path.read_text())
    root_overrides = {}
    for value in arguments.project_root:
        name, separator, root = value.partition("=")
        if not separator or not name or not root:
            parser.error("--project-root values must be NAME=PATH")
        root_overrides[name] = Path(root).resolve()
    selected = [record for record in manifest if not arguments.repository_names
                or record["corpus_repository_short_name"] in arguments.repository_names]
    try:
        digest_file_callback = command_line_blake3_digest_provider()
    except CoverageInventoryBlockedError as error:
        parser.error(str(error))
    artifact_run_directory = arguments.artifact_root.expanduser().absolute() / arguments.run_id
    artifact_run_directory.mkdir(parents=True, exist_ok=False)
    success = True
    for record in selected:
        name = record["corpus_repository_short_name"]
        output_directory = workspace_root / record["corpus_evidence_directory_path"]
        project_root = root_overrides.get(name, Path(record["corpus_source_checkout_path"]))
        project_identifier = str(project_root.resolve()).strip("/").replace("/", "-")
        artifact_repository_directory = artifact_run_directory / name
        artifact_repository_directory.mkdir(exist_ok=False)
        index_result_path = artifact_repository_directory / "codebase-index-result-log.txt"
        index_command = [arguments.indexer_path, "cli", "index_repository", "--repo-path",
                         str(project_root), "--mode", "full"]
        index_result = subprocess.run(index_command, capture_output=True, check=False)
        index_result_path.write_bytes(
            index_result.stdout if isinstance(index_result.stdout, bytes) else index_result.stdout.encode()
        )
        if index_result.returncode != 0:
            (artifact_repository_directory / "codebase-index-error-log.txt").write_bytes(
                index_result.stderr if isinstance(index_result.stderr, bytes) else index_result.stderr.encode()
            )
            print(json.dumps({"repository": name, "status": "BLOCKED",
                              "reason": "fresh full index command failed"}), flush=True)
            success = False
            continue
        summary = collect_index_coverage_inventory(
            name, output_directory, arguments.indexer_path, project_identifier=project_identifier,
            raw_response_directory_path=artifact_repository_directory,
            digest_file_callback=digest_file_callback,
        )
        print(json.dumps(summary), flush=True)
        success = success and summary["scope_literal_sop_pass"]
    return 0 if success else 1


if __name__ == "__main__":
    raise SystemExit(main())
