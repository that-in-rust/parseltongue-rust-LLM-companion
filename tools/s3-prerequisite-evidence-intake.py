"""Pure validation for identity-bound S3 prerequisite evidence."""

from __future__ import annotations

from collections import Counter
from collections.abc import Mapping, Sequence
import hashlib
import json
import math
from pathlib import PurePosixPath
import re
from urllib.parse import unquote


GLOBAL_GATE_NAMES = (
    "fixture_suite", "build_evidence", "dependency_audit", "release_provenance",
)
ALL_GATE_NAMES = (*GLOBAL_GATE_NAMES, "p2_offline_qualification")
DEFAULT_REPOSITORY_ORDER = ("serde", "rust-analyzer", "ripgrep", "tokio", "cargo-modules")
RECEIPT_REFERENCE_KEYS = {"path", "size_bytes", "sha256", "blake3"}
LOG_DESCRIPTOR_KEYS = {*RECEIPT_REFERENCE_KEYS, "complete"}
CHILD_TEST_NAMES = (
    "test_child_stderr_live_visibility",
    "test_child_output_exit_preservation",
    "test_child_output_read_error",
)
CHILD_IGNORED_TEST = ("test_child_fixture_waits_release", "subprocess fixture only")
MANUAL_IGNORES = {
    ("crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs",
     "test_profile_json_bulk_batches", "manual JSON bulk insert profile"),
    ("crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs",
     "test_profile_bulk_batch_sizes", "manual bounded bulk insert profile"),
    ("crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs",
     "test_profile_bounded_unit_sizes", "manual bounded transaction profile"),
}
M0_TEST_NAMES = {
    "test_no_analysis_crates_linked", "test_macro_dependencies_exclude_salsa",
    "test_semantic_identity_collision_safety",
}
M1_TEST_NAMES = {
    "test_crate_graph_offline_only", "test_lockfile_guard_preserves_bytes",
    "test_metadata_failure_writes_nothing", "test_ingest_bytes_stay_identical",
    "test_unchanged_digest_skips_write", "test_span_gap_tiling_identity",
    "test_parse_error_stops_run", "test_non_utf8_stops_run",
    "test_cfg_from_rustc_print", "test_inactive_cfg_items_counted",
    "test_build_script_never_executed", "test_out_dir_include_unresolved",
    "test_cargo_env_from_metadata", "test_stage_rerun_is_identical",
    "test_stage_refuses_missing_input", "test_stage_table_access_restricted",
    "test_generation_publication_is_atomic", "test_concurrent_writers_lose_nothing",
    "test_conflict_retry_is_bounded", "test_in_flight_units_bounded",
    "test_cancel_then_rerun_identical", "test_cancel_publication_stays_consistent",
}
M2_TEST_NAMES = {
    "test_macro_import_fixed_point", "test_fixed_point_limit_errors",
    "test_module_resolution_fixture_rules", "test_lang_items_in_sysroot",
    "test_nameres_matches_oracle_paths", "test_oracle_disagreements_keep_reasons",
    "test_macro_rules_items_match", "test_body_macro_sites_match",
}
TEST_ROW_RE = re.compile(r"^test\s+(.+?)\s+\.\.\.\s+(ok|FAILED|ignored(?:,.*)?)\s*$")
TEST_SUMMARY_RE = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; (\d+) filtered out; finished in [^\n]+$"
)
HEX_DIGEST_RE = re.compile(r"^[0-9a-f]{64}$")


class S3EvidenceBlockedError(ValueError):
    """Evidence is absent or cannot establish the requested contract."""


class S3EvidenceViolationError(ValueError):
    """Observed evidence contradicts the qualification contract."""


class S3EvidenceQualificationToken:
    """Internal token for a complete, identity-bound prerequisite set."""

    def __init__(self, run_identifier, repository_name, identity_digest):
        self.run_identifier = run_identifier
        self.repository_name = repository_name
        self.identity_digest = identity_digest


def _strict_json_equal(left, right):
    if type(left) is not type(right):
        return False
    if isinstance(left, Mapping):
        return set(left) == set(right) and all(_strict_json_equal(left[key], right[key]) for key in left)
    if isinstance(left, list):
        return len(left) == len(right) and all(_strict_json_equal(a, b) for a, b in zip(left, right))
    return left == right


def parse_s3_evidence_json(document_text: str, document_name: str):
    def reject_duplicate_pairs(pairs):
        values = {}
        for key, value in pairs:
            if key in values:
                raise S3EvidenceBlockedError(f"duplicate JSON key in {document_name}: {key}")
            values[key] = value
        return values

    def reject_nonfinite_number(number_text):
        raise S3EvidenceBlockedError(f"nonfinite JSON number in {document_name}: {number_text}")

    if not isinstance(document_text, str):
        raise S3EvidenceBlockedError(f"{document_name} is not UTF-8 JSON text")
    try:
        return json.loads(
            document_text,
            object_pairs_hook=reject_duplicate_pairs,
            parse_constant=reject_nonfinite_number,
        )
    except (json.JSONDecodeError, UnicodeError) as error:
        raise S3EvidenceBlockedError(f"malformed JSON in {document_name}: {error}") from error


def validate_s3_evidence_reference(reference):
    if not isinstance(reference, Mapping) or set(reference) != RECEIPT_REFERENCE_KEYS:
        raise S3EvidenceBlockedError("receipt reference must contain only path, size, SHA-256, and BLAKE3")
    path_value = reference.get("path")
    if not isinstance(path_value, str) or not path_value or "\\" in path_value or "\x00" in path_value:
        raise S3EvidenceBlockedError("evidence paths must be nonempty relative POSIX paths")
    path_value_parts = path_value.split("/")
    path = PurePosixPath(path_value)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path_value_parts):
        raise S3EvidenceBlockedError(f"unsafe evidence path: {path_value}")
    if path.as_posix() != path_value:
        raise S3EvidenceBlockedError(f"evidence path is not canonical: {path_value}")
    size_value = reference.get("size_bytes")
    if isinstance(size_value, bool) or not isinstance(size_value, int) or size_value < 0:
        raise S3EvidenceBlockedError(f"invalid evidence size for {path_value}")
    for digest_name in ("sha256", "blake3"):
        digest_value = reference.get(digest_name)
        if not isinstance(digest_value, str) or not HEX_DIGEST_RE.fullmatch(digest_value):
            raise S3EvidenceBlockedError(f"invalid {digest_name} digest for {path_value}")
    return path_value


def validate_s3_evidence_receipt(
    receipt_document,
    expected_identity_binding,
    repository_binding=None,
    expected_family=None,
    file_bytes_by_path=None,
    blake3_by_path=None,
):
    if not isinstance(receipt_document, Mapping):
        raise S3EvidenceBlockedError("receipt JSON must be an object")
    if not isinstance(expected_identity_binding, Mapping):
        raise S3EvidenceBlockedError("runtime identity binding is unavailable")
    schema_version = receipt_document.get("schema_version")
    if isinstance(schema_version, bool) or schema_version != 1 or not isinstance(schema_version, int):
        raise S3EvidenceBlockedError("receipt schema_version must be integer 1")
    for numeric_name in ("elapsed_seconds", "timeout_seconds"):
        if numeric_name in receipt_document:
            number = receipt_document[numeric_name]
            if isinstance(number, bool) or not isinstance(number, (int, float)) or not math.isfinite(number):
                raise S3EvidenceBlockedError(f"receipt {numeric_name} must be a finite non-bool number")
    if "elapsed_seconds" in receipt_document and receipt_document["elapsed_seconds"] < 0:
        raise S3EvidenceBlockedError("receipt elapsed_seconds cannot be negative")
    if "timeout_seconds" in receipt_document and receipt_document["timeout_seconds"] <= 0:
        raise S3EvidenceBlockedError("receipt timeout_seconds must be positive")
    if receipt_document.get("timed_out") is True and receipt_document.get("exit_status") == 0:
        raise S3EvidenceBlockedError("timed-out receipt cannot claim exit status zero")

    required_keys = {
        "schema_version", "receipt_id", "family", "scope", "identity_binding", "argv", "cwd",
        "relevant_environment", "elapsed_seconds", "timeout_seconds", "completion_status",
        "timed_out", "exit_status", "stdout", "stderr",
    }
    if repository_binding is not None:
        required_keys.add("repository_binding")
    if set(receipt_document) != required_keys:
        raise S3EvidenceBlockedError(
            f"receipt fields do not match schema v1: missing={sorted(required_keys - set(receipt_document))}; "
            f"extra={sorted(set(receipt_document) - required_keys)}"
        )
    receipt_id = receipt_document.get("receipt_id")
    if not isinstance(receipt_id, str) or not receipt_id.strip():
        raise S3EvidenceBlockedError("receipt_id must be nonempty text")
    family = receipt_document.get("family")
    if not isinstance(family, str) or not family:
        raise S3EvidenceBlockedError("receipt family must be nonempty text")
    if expected_family is not None and family != expected_family:
        raise S3EvidenceViolationError("receipt family does not match its manifest gate")
    scope = receipt_document.get("scope")
    if not isinstance(scope, str) or not scope:
        raise S3EvidenceBlockedError("receipt scope must be text")
    receipt_binding = receipt_document.get("identity_binding")
    if not isinstance(receipt_binding, Mapping):
        raise S3EvidenceBlockedError("receipt identity_binding is missing or malformed")
    if not _strict_json_equal(receipt_binding, expected_identity_binding):
        raise S3EvidenceViolationError("receipt identity_binding does not match the runtime identity")
    expected_environment = expected_identity_binding.get("relevant_environment")
    if not isinstance(expected_environment, Mapping):
        raise S3EvidenceBlockedError("runtime environment identity is unavailable")
    receipt_environment = receipt_document.get("relevant_environment")
    if not isinstance(receipt_environment, Mapping):
        raise S3EvidenceBlockedError("receipt relevant_environment must be an object")
    if not _strict_json_equal(receipt_environment, expected_environment):
        raise S3EvidenceViolationError("receipt relevant_environment differs from the runtime binding")
    if repository_binding is not None:
        supplied_repository_binding = receipt_document.get("repository_binding")
        if not isinstance(supplied_repository_binding, Mapping):
            raise S3EvidenceBlockedError("receipt repository_binding is missing or malformed")
        if not _strict_json_equal(supplied_repository_binding, repository_binding):
            raise S3EvidenceViolationError("receipt repository_binding does not match the frozen repository")
    argv = receipt_document.get("argv")
    if not isinstance(argv, list) or not argv or any(not isinstance(value, str) for value in argv):
        raise S3EvidenceBlockedError("receipt argv must be a nonempty string array")
    cwd = receipt_document.get("cwd")
    if not isinstance(cwd, str) or not PurePosixPath(cwd).is_absolute():
        raise S3EvidenceBlockedError("receipt cwd must be absolute")
    configuration = expected_identity_binding.get("configuration", {})
    if not isinstance(configuration, Mapping):
        raise S3EvidenceBlockedError("runtime workspace configuration is unavailable")
    expected_cwd = (
        repository_binding.get("checkout_path") if repository_binding is not None else configuration.get("project_root")
    )
    if not isinstance(expected_cwd, str) or not PurePosixPath(expected_cwd).is_absolute():
        raise S3EvidenceBlockedError("runtime workspace root identity is unavailable")
    if cwd != expected_cwd:
        raise S3EvidenceViolationError("receipt cwd differs from the frozen workspace root")
    timed_out = receipt_document.get("timed_out")
    completion_status = receipt_document.get("completion_status")
    exit_status = receipt_document.get("exit_status")
    if not isinstance(timed_out, bool) or not isinstance(completion_status, str) or completion_status not in {"completed", "timed_out"}:
        raise S3EvidenceBlockedError("receipt completion and timeout status are invalid")
    if timed_out != (completion_status == "timed_out"):
        raise S3EvidenceBlockedError("receipt timeout fields disagree")
    if exit_status is not None and (isinstance(exit_status, bool) or not isinstance(exit_status, int)):
        raise S3EvidenceBlockedError("receipt exit_status must be an integer or null")
    if not timed_out and exit_status is None:
        raise S3EvidenceBlockedError("completed receipt requires an exit status")
    if timed_out and exit_status == 0:
        raise S3EvidenceBlockedError("timed-out receipt cannot claim successful exit")
    if receipt_document["elapsed_seconds"] > receipt_document["timeout_seconds"] and not timed_out:
        raise S3EvidenceBlockedError("receipt elapsed time exceeds timeout without timeout status")

    file_bytes_by_path = file_bytes_by_path or {}
    blake3_by_path = blake3_by_path or {}
    log_texts = {}
    log_paths = set()
    for stream_name in ("stdout", "stderr"):
        descriptor = receipt_document.get(stream_name)
        if not isinstance(descriptor, Mapping) or set(descriptor) != LOG_DESCRIPTOR_KEYS:
            raise S3EvidenceBlockedError(f"receipt {stream_name} descriptor is incomplete")
        if descriptor.get("complete") is not True:
            raise S3EvidenceBlockedError(f"receipt {stream_name} is not declared complete")
        path_value = validate_s3_evidence_reference({
            key: descriptor[key] for key in RECEIPT_REFERENCE_KEYS if key in descriptor
        })
        if path_value in log_paths:
            raise S3EvidenceViolationError("receipt stdout and stderr paths must be distinct")
        log_paths.add(path_value)
        if path_value not in file_bytes_by_path or path_value not in blake3_by_path:
            raise S3EvidenceBlockedError(f"receipt log file is missing: {path_value}")
        content = file_bytes_by_path[path_value]
        actual_sha256 = hashlib.sha256(content).hexdigest()
        if len(content) != descriptor["size_bytes"] or actual_sha256 != descriptor["sha256"]:
            raise S3EvidenceViolationError(f"receipt {stream_name} size/SHA-256 mismatch: {path_value}")
        if blake3_by_path[path_value] != descriptor["blake3"]:
            raise S3EvidenceViolationError(f"receipt {stream_name} BLAKE3 mismatch: {path_value}")
        try:
            log_texts[stream_name] = content.decode("utf-8")
        except UnicodeDecodeError as error:
            raise S3EvidenceBlockedError(f"receipt {stream_name} is not complete UTF-8 text") from error
    _validate_command_family_contract(receipt_document, expected_identity_binding)
    return {"receipt": dict(receipt_document), "logs": log_texts, "log_paths": log_paths}


def _validate_command_family_contract(receipt_document, expected_identity_binding):
    argv = receipt_document["argv"]
    executable_name = argv[0].replace("\\", "/").rsplit("/", 1)[-1]
    if executable_name != "cargo":
        raise S3EvidenceBlockedError("receipt command must invoke the Cargo executable directly")
    toolchain_versions = expected_identity_binding.get("toolchain_versions")
    cargo_identity = toolchain_versions.get("cargo") if isinstance(toolchain_versions, Mapping) else None
    expected_cargo_path = cargo_identity.get("path") if isinstance(cargo_identity, Mapping) else None
    if not isinstance(expected_cargo_path, str) or not expected_cargo_path:
        raise S3EvidenceBlockedError("frozen PATH-dispatched Cargo executable identity is unavailable")
    if argv[0] != expected_cargo_path:
        raise S3EvidenceBlockedError("receipt cargo executable differs from the frozen PATH-dispatched command")
    arguments = argv[1:]
    family = receipt_document.get("family")
    environment = receipt_document.get("relevant_environment", {})
    if any(re.search(r"(?:SKIP|FILTER|IGNORE|NO_NATIVE)", key, re.IGNORECASE) for key in environment):
        raise S3EvidenceBlockedError("native test skip/filter environment is unsupported")
    if environment.get("CARGO_BUILD_TARGET") or any(
        key.startswith("CARGO_TARGET_") and key != "CARGO_TARGET_DIR" and value
        for key, value in environment.items()
    ):
        raise S3EvidenceBlockedError("Cargo target overrides are unsupported for S3 evidence")
    if any(environment.get(key) for key in ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")):
        raise S3EvidenceBlockedError("compiler wrappers are unsupported for S3 evidence")
    for flags_name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        if "--cfg" in str(environment.get(flags_name, "")):
            raise S3EvidenceBlockedError("unknown rustc cfg overrides are unsupported")
    if family in {"p2_check", "p2_metadata"}:
        if any(environment.get(key) for key in ("CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS")):
            raise S3EvidenceBlockedError("P2 requires compiler-default target and cfg policy")
        if any(key.startswith("CARGO_TARGET_") and key != "CARGO_TARGET_DIR" and value
               for key, value in environment.items()):
            raise S3EvidenceBlockedError("P2 target-specific Cargo overrides are unsupported")
        if any(key.startswith("CARGO_PROFILE_") and value for key, value in environment.items()):
            raise S3EvidenceBlockedError("P2 Cargo profile overrides are unsupported")
    optional = {"--offline", "--locked"}
    if family == "fmt":
        filtered = [value for value in arguments if value not in optional]
        if filtered != ["fmt", "--all", "--", "--check"]:
            raise S3EvidenceBlockedError("fmt receipt must prove full-workspace cargo fmt --all -- --check")
    elif family == "clippy":
        filtered = [value for value in arguments if value not in optional]
        if filtered != ["clippy", "--message-format=json", "--workspace", "--all-targets", "--all-features",
                        "--", "-D", "warnings"]:
            raise S3EvidenceBlockedError("clippy receipt must use workspace/all-targets/all-features and -D warnings")
    elif family == "fixture_suite":
        filtered = [value for value in arguments if value not in optional]
        separator = filtered.index("--") if "--" in filtered else len(filtered)
        cargo_args = filtered[:separator]
        test_args = filtered[separator + 1:] if separator < len(filtered) else []
        if (not cargo_args or cargo_args[0] != "test" or "--workspace" not in cargo_args
                or any(value in cargo_args for value in ("--package", "-p", "--exclude", "--all", "--no-default-features"))
                or any(value not in {"--workspace", "--all-targets", "--all-features"} for value in cargo_args[1:])
                or any(value not in {"--nocapture"} for value in test_args)):
            raise S3EvidenceBlockedError("fixture receipt must be an unfiltered production-workspace cargo test")
    elif family == "dependency_audit":
        filtered = [value for value in arguments if value not in optional]
        if filtered != ["tree", "--package", "parseltongue-cli-dispatcher-binary", "--edges", "normal",
                        "--prefix", "none", "--format", "{p}"]:
            raise S3EvidenceBlockedError("dependency receipt must be the full-depth production normal tree")
    elif family == "release_build":
        filtered = [value for value in arguments if value not in optional]
        if filtered != ["build", "--release", "--package", "parseltongue-cli-dispatcher-binary",
                        "--bin", "parseltongue", "--message-format=json"]:
            raise S3EvidenceBlockedError("release receipt must build the production parseltongue binary as JSON")
    elif family == "p2_check":
        filtered = [value for value in arguments if value not in optional]
        required = ["check", "--workspace", "--lib", "--bins", "--message-format=json"]
        if filtered != required or not {"--offline", "--locked"}.issubset(arguments):
            raise S3EvidenceBlockedError("P2 receipt must be offline/locked workspace lib+bin cargo check")
    elif family == "p2_metadata":
        filtered = [value for value in arguments if value not in optional]
        if filtered != ["metadata", "--format-version", "1"] or not {"--offline", "--locked"}.issubset(arguments):
            raise S3EvidenceBlockedError("P2 metadata receipt must be offline/locked Cargo metadata v1")
    else:
        raise S3EvidenceBlockedError(f"unsupported S3 receipt family: {family}")


def derive_source_fixture_inventory(source_contents_by_path):
    tests = []
    ambiguous_reasons = []
    test_attribute = re.compile(
        r"(?m)(?P<attrs>(?:^[ \t]*#\[[^\]\n]+\][ \t]*\n)+)[ \t]*"
        r"(?:pub(?:\([^\n]*\))?[ \t]+)?(?:async[ \t]+)?fn[ \t]+"
        r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)[ \t]*\("
    )
    known_test_attributes = {"test", "tokio::test"}
    for path_value, source_text in sorted(source_contents_by_path.items()):
        function_matches = list(test_attribute.finditer(source_text))
        test_attribute_lines = list(re.finditer(r"(?m)^\s*#\[(?:tokio::)?test(?:[^\]]*)\]\s*$", source_text))
        for attribute_match in test_attribute_lines:
            if not any(match.start("attrs") <= attribute_match.start() < match.end("attrs")
                       for match in function_matches):
                ambiguous_reasons.append(f"test function could not be inventoried unambiguously in {path_value}")
        for match in function_matches:
            attributes = re.findall(r"#\[([^\]]+)\]", match.group("attrs"))
            test_attributes = [value.strip() for value in attributes if "test" in value]
            if not test_attributes:
                continue
            if any(value not in known_test_attributes for value in test_attributes):
                ambiguous_reasons.append(f"unsupported test attribute in {path_value}: {test_attributes}")
                continue
            ignore_match = next((re.fullmatch(r'ignore\s*=\s*"([^"]*)"', value.strip())
                                 for value in attributes if value.strip().startswith("ignore")), None)
            ignored_attribute = any(value.strip() == "ignore" or value.strip().startswith("ignore =")
                                    for value in attributes)
            ignore_reason = ignore_match.group(1) if ignore_match else None
            if ignored_attribute and ignore_reason is None:
                ambiguous_reasons.append(f"ignore reason is not explicit in {path_value}:{match.group('name')}")
            tests.append({
                "name": match.group("name"), "path": path_value,
                "ignore_reason": ignore_reason,
            })
        for macro_match in re.finditer(r"macro_rules!\s*\w+\s*\{", source_text):
            body_start = macro_match.end()
            nesting = 1
            cursor = body_start
            while cursor < len(source_text) and nesting:
                if source_text[cursor] == "{":
                    nesting += 1
                elif source_text[cursor] == "}":
                    nesting -= 1
                cursor += 1
            macro_body = source_text[body_start:cursor]
            if re.search(r"#\[(?:tokio::)?test\]|\bfn\s+\$", macro_body):
                ambiguous_reasons.append(f"macro-generated test inventory is ambiguous in {path_value}")
        if re.search(r"\b(?:proptest|rstest|test_case|paste)!", source_text):
            ambiguous_reasons.append(f"test-generating macro inventory is ambiguous in {path_value}")
    names = [test["name"] for test in tests]
    if len(names) != len(set((test["path"], test["name"]) for test in tests)):
        ambiguous_reasons.append("duplicate test function in one frozen Rust source file")
    return {"tests": tests, "ambiguous_reason": "; ".join(sorted(set(ambiguous_reasons))) or None}


def parse_fixture_test_output(stdout_text, stderr_text, source_test_inventory, milestone):
    if not isinstance(stdout_text, str) or not isinstance(stderr_text, str):
        raise S3EvidenceBlockedError("fixture stdout/stderr are not text")
    combined_text = stdout_text + "\n" + stderr_text
    rows = []
    summaries = []
    running_counts = []
    rows_since_summary = 0
    section_results = Counter()
    for line in combined_text.splitlines():
        running_match = re.match(r"^running (\d+) tests?$", line.strip())
        if running_match:
            running_counts.append(int(running_match.group(1)))
        row_match = TEST_ROW_RE.match(line.strip())
        if row_match:
            test_full_name, result_text = row_match.groups()
            rows.append({
                "full_name": test_full_name,
                "name": test_full_name.rsplit("::", 1)[-1],
                "result": "ignored" if result_text.startswith("ignored") else result_text,
                "reason": result_text.partition(",")[2].strip() if result_text.startswith("ignored,") else None,
            })
            rows_since_summary += 1
            if result_text == "ok":
                section_results["passed"] += 1
            elif result_text == "FAILED":
                section_results["failed"] += 1
            else:
                section_results["ignored"] += 1
        summary_match = TEST_SUMMARY_RE.match(line.strip())
        if summary_match:
            state, passed, failed, ignored, measured, filtered = summary_match.groups()
            summary = {
                "state": state, "passed": int(passed), "failed": int(failed),
                "ignored": int(ignored), "measured": int(measured), "filtered": int(filtered),
                "rows": rows_since_summary, "row_results": dict(section_results),
            }
            summaries.append(summary)
            rows_since_summary = 0
            section_results = Counter()
    observed_failure = (
        any(row["result"] == "FAILED" for row in rows)
        or any(summary["failed"] or summary["state"] == "FAILED" for summary in summaries)
    )
    if observed_failure:
        raise S3EvidenceViolationError("fixture output reports failed tests")
    if not summaries or rows_since_summary:
        raise S3EvidenceBlockedError("fixture output has no complete libtest summary")
    if len(summaries) != len(running_counts):
        raise S3EvidenceBlockedError("fixture output has incomplete running/summary sections")
    if any(summary["rows"] != running_counts[index] for index, summary in enumerate(summaries)):
        raise S3EvidenceBlockedError("fixture output rows do not match libtest running counts")
    if any(summary["rows"] != summary["passed"] + summary["failed"] + summary["ignored"]
           for summary in summaries):
        raise S3EvidenceBlockedError("fixture output rows do not match libtest summary counts")
    if any(any(summary["row_results"].get(key, 0) != summary[key]
               for key in ("passed", "failed", "ignored")) for summary in summaries):
        raise S3EvidenceBlockedError("fixture output per-section result counts do not match test rows")
    if any(summary["filtered"] or summary["measured"] for summary in summaries):
        raise S3EvidenceBlockedError("fixture output is filtered or contains measured tests")
    if any(summary["state"] != "ok" for summary in summaries):
        raise S3EvidenceBlockedError("fixture output summary state is unsupported")
    if not isinstance(source_test_inventory, Mapping):
        raise S3EvidenceBlockedError("frozen source test inventory is malformed")
    ambiguous_reason = source_test_inventory.get("ambiguous_reason")
    if ambiguous_reason is not None and not isinstance(ambiguous_reason, str):
        raise S3EvidenceBlockedError("frozen source test inventory ambiguity marker is malformed")
    if ambiguous_reason:
        raise S3EvidenceBlockedError(ambiguous_reason)
    inventory_tests = source_test_inventory.get("tests")
    if not isinstance(inventory_tests, list):
        raise S3EvidenceBlockedError("frozen source test inventory is unavailable")
    source_by_name = {}
    for test in inventory_tests:
        if (not isinstance(test, Mapping) or not isinstance(test.get("name"), str)
                or not isinstance(test.get("path"), str)
                or (test.get("ignore_reason") is not None and not isinstance(test.get("ignore_reason"), str))):
            raise S3EvidenceBlockedError("frozen source test inventory contains malformed entries")
        source_by_name.setdefault(test["name"], []).append(test)
    row_counts = Counter(row["name"] for row in rows)
    if len({row["full_name"] for row in rows}) != len(rows):
        raise S3EvidenceViolationError("fixture output contains duplicate test rows")
    if set(row_counts) != set(source_by_name):
        raise S3EvidenceBlockedError(
            f"fixture/source test inventory differs: missing={sorted(set(source_by_name) - set(row_counts))}; "
            f"unexpected={sorted(set(row_counts) - set(source_by_name))}"
        )
    if any(row_counts[name] != len(tests) for name, tests in source_by_name.items()):
        raise S3EvidenceBlockedError("fixture output does not account for every frozen source test")
    required_tests = M0_TEST_NAMES | M1_TEST_NAMES
    if milestone == "M2":
        required_tests |= M2_TEST_NAMES
    elif milestone != "M1":
        raise S3EvidenceBlockedError(f"unsupported milestone for fixture inventory: {milestone}")
    missing_required = sorted(required_tests - set(source_by_name))
    if missing_required:
        raise S3EvidenceBlockedError(f"required SPEC §4 fixture tests are absent: {missing_required}")
    for row in rows:
        source_matches = source_by_name[row["name"]]
        source_ignores = [test.get("ignore_reason") for test in source_matches]
        if row["result"] == "ok":
            if any(reason is not None for reason in source_ignores):
                raise S3EvidenceBlockedError(f"ignored source test executed unexpectedly: {row['name']}")
            continue
        if row["result"] == "FAILED":
            raise S3EvidenceViolationError(f"fixture test failed: {row['full_name']}")
        if row["name"] == CHILD_IGNORED_TEST[0]:
            if source_matches[0].get("path") != "crates/parseltongue-cli-dispatcher-binary/tests/workspace-cli-contract-tests.rs" or source_ignores != [CHILD_IGNORED_TEST[1]]:
                raise S3EvidenceBlockedError("claimed child ignore is absent or differs from frozen source")
            active = {test["name"] for test in rows if test["result"] == "ok"}
            if not set(CHILD_TEST_NAMES).issubset(active):
                raise S3EvidenceBlockedError("conditional child ignore lacks the three active child checks")
            if row.get("reason") != CHILD_IGNORED_TEST[1]:
                raise S3EvidenceBlockedError("libtest child ignore reason differs from frozen source")
            continue
        if len(source_matches) != 1:
            raise S3EvidenceBlockedError(f"ignored test identity is ambiguous: {row['name']}")
        allowlist_key = (source_matches[0].get("path"), row["name"], source_matches[0].get("ignore_reason"))
        if allowlist_key not in MANUAL_IGNORES or row.get("reason") != allowlist_key[2]:
            raise S3EvidenceBlockedError(f"ignored test is not an exact source-backed manual profile: {row['name']}")
    return {"rows": rows, "summaries": summaries, "status": "PASS"}


def validate_s3_prerequisite_evidence(
    manifest_document,
    file_bytes_by_path,
    blake3_by_path,
    expected_identity_binding,
    expected_repository_bindings,
    source_test_inventory,
    milestone,
):
    if not isinstance(manifest_document, Mapping):
        raise S3EvidenceBlockedError("S3 evidence manifest must be an object")
    if set(manifest_document) != {
        "schema_version", "run_id", "milestone", "identity_binding", "global_prerequisites", "repositories",
    }:
        raise S3EvidenceBlockedError("manifest fields do not match schema v1")
    schema_version = manifest_document.get("schema_version")
    if isinstance(schema_version, bool) or not isinstance(schema_version, int) or schema_version != 1:
        raise S3EvidenceBlockedError("manifest schema_version must be integer 1")
    run_identifier = manifest_document.get("run_id")
    if not isinstance(run_identifier, str) or not run_identifier:
        raise S3EvidenceBlockedError("manifest run_id must be nonempty text")
    if not isinstance(manifest_document.get("milestone"), str):
        raise S3EvidenceBlockedError("manifest milestone must be text")
    if manifest_document.get("milestone") != milestone:
        raise S3EvidenceViolationError("manifest milestone does not match the current run")
    manifest_binding = manifest_document.get("identity_binding")
    if not isinstance(manifest_binding, Mapping):
        raise S3EvidenceBlockedError("manifest identity_binding is missing or malformed")
    if not _strict_json_equal(manifest_binding, expected_identity_binding):
        raise S3EvidenceViolationError("manifest identity_binding does not match the runtime identity")
    if expected_identity_binding.get("run_id") != run_identifier:
        raise S3EvidenceViolationError("manifest run_id differs from the runtime identity")
    if expected_identity_binding.get("milestone") != milestone:
        raise S3EvidenceViolationError("runtime identity does not bind the requested milestone")
    global_prerequisites = manifest_document.get("global_prerequisites")
    repositories = manifest_document.get("repositories")
    gate_references = []
    gate_observations = {name: [] for name in GLOBAL_GATE_NAMES}
    gate_observations["p2_offline_qualification"] = []

    if not isinstance(global_prerequisites, Mapping):
        for gate_name in GLOBAL_GATE_NAMES:
            gate_observations[gate_name].append((
                "BLOCKED", "manifest global_prerequisites must be an object", "production_workspace",
            ))
    else:
        missing_global_gates = set(GLOBAL_GATE_NAMES) - set(global_prerequisites)
        extra_global_gates = set(global_prerequisites) - set(GLOBAL_GATE_NAMES)
        for gate_name in missing_global_gates:
            gate_observations[gate_name].append((
                "BLOCKED", f"manifest prerequisite is absent: {gate_name}", "production_workspace",
            ))
        if extra_global_gates:
            for gate_name in GLOBAL_GATE_NAMES:
                gate_observations[gate_name].append((
                    "BLOCKED", f"unsupported manifest prerequisite families: {sorted(extra_global_gates)}",
                    "production_workspace",
                ))
        fixture_references = global_prerequisites.get("fixture_suite")
        if not isinstance(fixture_references, list) or not fixture_references:
            gate_observations["fixture_suite"].append((
                "BLOCKED", "fixture_suite must reference at least one receipt", "production_workspace",
            ))
        else:
            gate_references.extend(("fixture_suite", reference, "fixture_suite", "production_workspace", None)
                                   for reference in fixture_references)
        build_references = global_prerequisites.get("build_evidence")
        if not isinstance(build_references, Mapping):
            gate_observations["build_evidence"].append((
                "BLOCKED", "build_evidence must contain distinct fmt and clippy receipts", "production_workspace",
            ))
        else:
            if set(build_references) != {"fmt", "clippy"}:
                gate_observations["build_evidence"].append((
                    "BLOCKED", "build_evidence requires exactly fmt and clippy receipts", "production_workspace",
                ))
            for family in ("fmt", "clippy"):
                if family in build_references:
                    gate_references.append(("build_evidence", build_references[family], family,
                                            "production_workspace", None))
                else:
                    gate_observations["build_evidence"].append((
                        "BLOCKED", f"build_evidence receipt is absent: {family}", "production_workspace",
                    ))
    for gate_name, family in (("dependency_audit", "dependency_audit"),
                              ("release_provenance", "release_build")):
        references = global_prerequisites.get(gate_name) if isinstance(global_prerequisites, Mapping) else None
        if not isinstance(references, list):
            gate_observations[gate_name].append((
                "BLOCKED", f"{gate_name} must reference one independent receipt", "production_workspace",
            ))
        else:
            if len(references) != 1:
                gate_observations[gate_name].append((
                    "BLOCKED", f"{gate_name} requires exactly one independent receipt", "production_workspace",
                ))
            gate_references.extend((gate_name, reference, family, "production_workspace", None)
                                   for reference in references)
    if not isinstance(repositories, Mapping):
        for repository_name in expected_repository_bindings:
            gate_observations["p2_offline_qualification"].append((
                "BLOCKED", "manifest repositories must be an object", repository_name,
            ))
    else:
        if set(repositories) != set(expected_repository_bindings):
            reason = "manifest repository set differs from pinned repositories"
            for repository_name in expected_repository_bindings:
                gate_observations["p2_offline_qualification"].append(("BLOCKED", reason, repository_name))
        for repository_name, repository_binding in expected_repository_bindings.items():
            repository_value = repositories.get(repository_name)
            if not isinstance(repository_value, Mapping):
                gate_observations["p2_offline_qualification"].append((
                    "BLOCKED", f"repository evidence is absent or malformed: {repository_name}", repository_name,
                ))
                continue
            if set(repository_value) != {"repository_binding", "p2_offline_qualification"}:
                gate_observations["p2_offline_qualification"].append((
                    "BLOCKED", f"repository evidence shape is invalid: {repository_name}", repository_name,
                ))
            supplied_binding = repository_value.get("repository_binding")
            if not isinstance(supplied_binding, Mapping):
                gate_observations["p2_offline_qualification"].append((
                    "BLOCKED", f"repository binding is absent or malformed: {repository_name}", repository_name,
                ))
            elif not _strict_json_equal(supplied_binding, repository_binding):
                gate_observations["p2_offline_qualification"].append((
                    "FAIL", f"repository binding mismatch: {repository_name}", repository_name,
                ))
            p2_references = repository_value.get("p2_offline_qualification")
            if not isinstance(p2_references, list):
                gate_observations["p2_offline_qualification"].append((
                    "BLOCKED", f"P2 receipts are absent or malformed: {repository_name}", repository_name,
                ))
                continue
            if len(p2_references) != 2:
                gate_observations["p2_offline_qualification"].append((
                    "BLOCKED", f"P2 requires one metadata and one check receipt: {repository_name}", repository_name,
                ))
            gate_references.extend(("p2_offline_qualification", reference, None, repository_name,
                                    repository_binding) for reference in p2_references)

    valid_gate_references = []
    receipt_paths = []
    for gate_name, reference, family, scope, repository_binding in gate_references:
        try:
            receipt_path = validate_s3_evidence_reference(reference)
        except S3EvidenceViolationError as error:
            gate_observations[gate_name].append(("FAIL", str(error), scope))
        except S3EvidenceBlockedError as error:
            gate_observations[gate_name].append(("BLOCKED", str(error), scope))
        else:
            receipt_paths.append(receipt_path)
            valid_gate_references.append((gate_name, reference, family, scope, repository_binding))
    if len(receipt_paths) != len(set(receipt_paths)):
        duplicate_paths = {path for path in receipt_paths if receipt_paths.count(path) > 1}
        seen_paths = set()
        filtered_references = []
        for entry in valid_gate_references:
            path = entry[1]["path"]
            if path in duplicate_paths:
                gate_observations[entry[0]].append((
                    "FAIL", f"receipt path is duplicated or shared across gates: {path}", entry[3],
                ))
                if path in seen_paths:
                    continue
                seen_paths.add(path)
            filtered_references.append(entry)
        valid_gate_references = filtered_references
        receipt_paths = list(dict.fromkeys(receipt_paths))
    gate_references = valid_gate_references

    receipt_records = []
    receipt_ids = set()
    receipt_contents = set()
    used_paths = set(receipt_paths)
    file_bytes_by_path = file_bytes_by_path or {}
    blake3_by_path = blake3_by_path or {}

    def process_reference(gate_name, reference, expected_family, scope, repository_binding):
        receipt_path = reference["path"]
        if receipt_path not in file_bytes_by_path or receipt_path not in blake3_by_path:
            raise S3EvidenceBlockedError(f"receipt file is missing at intake: {receipt_path}")
        receipt_bytes = file_bytes_by_path[receipt_path]
        if not isinstance(receipt_bytes, bytes):
            raise S3EvidenceBlockedError(f"receipt file is not raw bytes: {receipt_path}")
        actual_sha256 = hashlib.sha256(receipt_bytes).hexdigest()
        actual_blake3 = blake3_by_path[receipt_path]
        if len(receipt_bytes) != reference["size_bytes"] or actual_sha256 != reference["sha256"]:
            raise S3EvidenceViolationError(f"receipt reference size/SHA-256 mismatch: {receipt_path}")
        if actual_blake3 != reference["blake3"]:
            raise S3EvidenceViolationError(f"receipt reference BLAKE3 mismatch: {receipt_path}")
        receipt_digest = (len(receipt_bytes), actual_sha256, actual_blake3)
        if receipt_digest in receipt_contents:
            raise S3EvidenceViolationError("receipt contents cannot be duplicated")
        receipt_contents.add(receipt_digest)
        try:
            receipt_text = receipt_bytes.decode("utf-8")
        except UnicodeDecodeError as error:
            raise S3EvidenceBlockedError(f"receipt is not UTF-8 JSON: {receipt_path}") from error
        receipt_document = parse_s3_evidence_json(receipt_text, receipt_path)
        if not isinstance(receipt_document, Mapping):
            raise S3EvidenceBlockedError(f"receipt JSON must be an object: {receipt_path}")
        actual_family = receipt_document.get("family")
        if not isinstance(actual_family, str) or not actual_family:
            raise S3EvidenceBlockedError(f"receipt family is missing or malformed: {receipt_path}")
        receipt_scope = receipt_document.get("scope")
        if not isinstance(receipt_scope, str) or not receipt_scope:
            raise S3EvidenceBlockedError(f"receipt scope is missing or malformed: {receipt_path}")
        if scope != receipt_scope:
            raise S3EvidenceViolationError(f"receipt scope mismatch: {receipt_path}")
        receipt_id = receipt_document.get("receipt_id")
        if not isinstance(receipt_id, str) or not receipt_id.strip():
            raise S3EvidenceBlockedError(f"receipt_id is missing or malformed: {receipt_path}")
        if receipt_id in receipt_ids:
            raise S3EvidenceViolationError(f"duplicate receipt_id: {receipt_id}")
        log_paths = [receipt_document.get(name, {}).get("path")
                     for name in ("stdout", "stderr") if isinstance(receipt_document.get(name), Mapping)]
        for log_path in log_paths:
            if isinstance(log_path, str):
                if log_path in used_paths:
                    raise S3EvidenceViolationError(f"evidence path is reused: {log_path}")
                used_paths.add(log_path)
        receipt_result = validate_s3_evidence_receipt(
            receipt_document, expected_identity_binding, repository_binding,
            expected_family if expected_family is not None else actual_family,
            file_bytes_by_path, blake3_by_path,
        )
        if expected_family is None and actual_family not in {"p2_check", "p2_metadata"}:
            raise S3EvidenceBlockedError(f"unsupported P2 receipt family: {actual_family}")
        if actual_family in {record["family"] for record in receipt_records
                             if record["scope"] == scope and record["gate"] == gate_name}:
            raise S3EvidenceBlockedError(f"duplicate receipt family for {scope}: {actual_family}")
        receipt_ids.add(receipt_id)
        return {
            "gate": gate_name, "scope": scope, "family": actual_family,
            "receipt": receipt_result["receipt"], "logs": receipt_result["logs"],
        }

    for gate_name, reference, expected_family, scope, repository_binding in gate_references:
        try:
            receipt_records.append(process_reference(
                gate_name, reference, expected_family, scope, repository_binding,
            ))
        except S3EvidenceViolationError as error:
            gate_observations[gate_name].append(("FAIL", str(error), scope))
        except S3EvidenceBlockedError as error:
            gate_observations[gate_name].append(("BLOCKED", str(error), scope))
    unreferenced_paths = set(file_bytes_by_path) - used_paths
    if unreferenced_paths:
        reason = f"evidence file bundle contains unreferenced paths: {sorted(unreferenced_paths)}"
        for gate_name in GLOBAL_GATE_NAMES:
            gate_observations[gate_name].append(("BLOCKED", reason, "production_workspace"))
        for repository_name in expected_repository_bindings:
            gate_observations["p2_offline_qualification"].append(("BLOCKED", reason, repository_name))

    for repository_name in expected_repository_bindings:
        p2_families = {record["family"] for record in receipt_records
                       if record["gate"] == "p2_offline_qualification" and record["scope"] == repository_name}
        if p2_families != {"p2_check", "p2_metadata"}:
            gate_observations["p2_offline_qualification"].append((
                "BLOCKED", f"P2 must have distinct metadata/check evidence: {repository_name}", repository_name,
            ))

    global_status = {}
    for gate_name in GLOBAL_GATE_NAMES:
        records = [record for record in receipt_records if record["gate"] == gate_name]
        try:
            gate_status, reason = _evaluate_gate_receipts(
                gate_name, records, source_test_inventory, milestone, expected_identity_binding
            )
        except S3EvidenceViolationError as error:
            gate_status, reason = "FAIL", str(error)
        except S3EvidenceBlockedError as error:
            gate_status, reason = "BLOCKED", str(error)
        gate_status, reason = _combine_gate_observations(
            gate_status, reason,
            [value for value in gate_observations[gate_name] if value[2] == "production_workspace"],
        )
        global_status[gate_name] = {"status": gate_status, "reason": reason}
    repository_results = {}
    tokens = {}
    identity_digest = hashlib.sha256(json.dumps(
        expected_identity_binding, sort_keys=True, separators=(",", ":")
    ).encode()).hexdigest()
    for repository_name in expected_repository_bindings:
        p2_records = [record for record in receipt_records
                      if record["gate"] == "p2_offline_qualification" and record["scope"] == repository_name]
        try:
            p2_status, p2_reason = _evaluate_gate_receipts(
                "p2_offline_qualification", p2_records, source_test_inventory, milestone,
                expected_identity_binding, expected_repository_bindings[repository_name],
            )
        except S3EvidenceViolationError as error:
            p2_status, p2_reason = "FAIL", str(error)
        except S3EvidenceBlockedError as error:
            p2_status, p2_reason = "BLOCKED", str(error)
        p2_status, p2_reason = _combine_gate_observations(
            p2_status, p2_reason,
            [value for value in gate_observations["p2_offline_qualification"] if value[2] == repository_name],
        )
        statuses = {**global_status, "p2_offline_qualification": {"status": p2_status, "reason": p2_reason}}
        repository_results[repository_name] = statuses
        if all(value["status"] == "PASS" for value in statuses.values()):
            tokens[repository_name] = S3EvidenceQualificationToken(run_identifier, repository_name, identity_digest)
    return {
        "repository_results": repository_results,
        "tokens_by_repository": tokens,
        "identity_digest": identity_digest,
    }


def _evaluate_gate_receipts(gate_name, records, source_test_inventory, milestone,
                            expected_identity_binding, repository_binding=None):
    if not records:
        return "BLOCKED", "no receipt is retained for this prerequisite"
    observations = []
    by_family = {record["family"]: record for record in records}
    for record in records:
        receipt = record["receipt"]
        status = "PASS"
        reason = "receipt command and output satisfy the gate contract"
        timed_out = receipt["timed_out"]
        try:
            if not timed_out and receipt["exit_status"] not in {None, 0}:
                raise S3EvidenceViolationError("receipt command exited nonzero")
            if not timed_out and receipt["exit_status"] is None:
                raise S3EvidenceBlockedError("receipt command timed out or lacks completion status")
            if record["family"] == "fixture_suite":
                parse_fixture_test_output(record["logs"]["stdout"], record["logs"]["stderr"],
                                          source_test_inventory, milestone)
            elif record["family"] == "fmt":
                if record["logs"]["stdout"].strip() or record["logs"]["stderr"].strip():
                    raise S3EvidenceViolationError("fmt --check emitted output despite a claimed successful exit")
            elif record["family"] == "clippy":
                if re.search(r"\b(?:warning|error)(?:\[[^\]]+\])?\s*:",
                             record["logs"]["stderr"], re.IGNORECASE):
                    raise S3EvidenceViolationError("clippy stderr contains warning/error diagnostics")
                messages = _cargo_json_messages(record["logs"]["stdout"], fail_on_diagnostic=True)
                _validate_build_finished(messages, "clippy")
                diagnostics = []
                for message in messages:
                    if message.get("reason") != "compiler-message":
                        continue
                    diagnostic = message.get("message")
                    if not isinstance(diagnostic, Mapping) or not isinstance(diagnostic.get("level"), str):
                        raise S3EvidenceBlockedError("clippy compiler-message is malformed")
                    diagnostics.append(diagnostic["level"])
                if any(level in {"warning", "error"} for level in diagnostics):
                    raise S3EvidenceViolationError("clippy output contains warning/error diagnostics")
            elif record["family"] == "dependency_audit":
                _validate_dependency_tree_output(record["logs"]["stdout"])
            elif record["family"] == "release_build":
                _validate_release_build_output(record["logs"]["stdout"], expected_identity_binding)
            elif record["family"] == "p2_metadata":
                _validate_p2_metadata_output(record["logs"]["stdout"], repository_binding)
            elif record["family"] == "p2_check":
                metadata_record = by_family.get("p2_metadata")
                if metadata_record is None:
                    raise S3EvidenceBlockedError("P2 metadata receipt is required to validate check artifacts")
                _validate_p2_check_output(record["logs"]["stdout"], metadata_record["logs"]["stdout"],
                                          repository_binding)
        except S3EvidenceViolationError as error:
            observations.append(("FAIL", str(error)))
        except S3EvidenceBlockedError as error:
            observations.append(("BLOCKED", str(error)))
        else:
            observations.append(("BLOCKED", "receipt command timed out") if timed_out else (status, reason))
    return _combine_gate_observations("PASS", "all retained receipts satisfy the gate contract", observations)


def _combine_gate_observations(status, reason, observations):
    all_observations = [(status, reason)] + [(item[0], item[1]) for item in observations]
    final_status = "FAIL" if any(item[0] == "FAIL" for item in all_observations) else (
        "BLOCKED" if any(item[0] == "BLOCKED" for item in all_observations) else "PASS"
    )
    reasons = [item[1] for item in all_observations if item[1]]
    return final_status, "; ".join(dict.fromkeys(reasons))


def _validate_dependency_tree_output(stdout_text):
    if not isinstance(stdout_text, str):
        raise S3EvidenceBlockedError("dependency tree output is not text")
    rows = [line.strip() for line in stdout_text.splitlines() if line.strip()]
    if not rows:
        raise S3EvidenceBlockedError("dependency tree output is empty")
    package_names = []
    for row in rows:
        match = re.fullmatch(r"([A-Za-z0-9_-]+) v([^\s()]+)(?: \((.+)\))?", row)
        if not match:
            raise S3EvidenceBlockedError("dependency tree output does not match parseable {p} rows")
        package_names.append(match.group(1).lower().replace("_", "-"))
    if package_names[0] != "parseltongue-cli-dispatcher-binary":
        raise S3EvidenceViolationError("dependency tree root is not the production CLI package")
    forbidden_roots = (
        "ide", "hir", "base-db", "salsa", "ra-ap-ide", "ra-ap-hir", "ra-ap-base-db", "ra-ap-salsa",
        "oracle-semantic-export-harness", "proc-macro-srv", "proc-macro-server",
        "ra-ap-proc-macro-srv", "ra-ap-proc-macro-server",
    )
    found = [name for name in package_names
             if any(name == root or name.startswith(root + "-") for root in forbidden_roots)]
    if found:
        raise S3EvidenceViolationError(f"forbidden runtime dependency rows: {sorted(set(found))}")


def _cargo_json_messages(stdout_text, fail_on_diagnostic=False):
    if not isinstance(stdout_text, str):
        raise S3EvidenceBlockedError("Cargo JSON output is not text")
    messages = []
    malformed_reason = None
    observed_failure = False
    for line in stdout_text.splitlines():
        if not line.strip():
            continue
        try:
            value = parse_s3_evidence_json(line, "Cargo JSON message")
        except (S3EvidenceBlockedError, S3EvidenceViolationError) as error:
            malformed_reason = malformed_reason or str(error)
            continue
        if not isinstance(value, Mapping):
            malformed_reason = malformed_reason or "Cargo JSON message is not an object"
            continue
        if value.get("reason") == "build-finished" and value.get("success") is False:
            observed_failure = True
        if fail_on_diagnostic and value.get("reason") == "compiler-message":
            diagnostic = value.get("message")
            if not isinstance(diagnostic, Mapping) or not isinstance(diagnostic.get("level"), str):
                malformed_reason = malformed_reason or "Cargo compiler-message is malformed"
            elif diagnostic["level"] in {"warning", "error"}:
                observed_failure = True
        messages.append(value)
    if observed_failure:
        raise S3EvidenceViolationError("Cargo build-finished reports failure")
    if malformed_reason:
        raise S3EvidenceBlockedError(malformed_reason)
    if not messages:
        raise S3EvidenceBlockedError("Cargo JSON output is empty")
    return messages


def _validate_build_finished(messages, command_name):
    finished = [message for message in messages if message.get("reason") == "build-finished"]
    if len(finished) != 1:
        raise S3EvidenceBlockedError(f"{command_name} JSON lacks one build-finished message")
    success = finished[0].get("success")
    if not isinstance(success, bool):
        raise S3EvidenceBlockedError(f"{command_name} build-finished success field is missing or malformed")
    if not success:
        raise S3EvidenceViolationError(f"{command_name} build-finished reports failure")


def _validate_release_build_output(stdout_text, expected_identity_binding):
    if not isinstance(expected_identity_binding, Mapping):
        raise S3EvidenceBlockedError("runtime identity binding is unavailable")
    messages = _cargo_json_messages(stdout_text)
    _validate_build_finished(messages, "release build")
    artifacts = [message for message in messages if message.get("reason") == "compiler-artifact"]
    configuration = expected_identity_binding.get("configuration", {})
    if not isinstance(configuration, Mapping):
        raise S3EvidenceBlockedError("production Cargo configuration identity is unavailable")
    expected_manifest = configuration.get("production_manifest")
    expected_package_name = configuration.get("production_package_name")
    expected_package_version = configuration.get("production_package_version")
    expected_bin_source = configuration.get("production_bin_source")
    expected_binary = expected_identity_binding.get("mapper")
    if (not isinstance(expected_manifest, str) or not PurePosixPath(expected_manifest).is_absolute()
            or not isinstance(expected_package_name, str) or not expected_package_name
            or not isinstance(expected_package_version, str) or not expected_package_version
            or not isinstance(expected_bin_source, str) or not PurePosixPath(expected_bin_source).is_absolute()
            or not isinstance(expected_binary, Mapping)
            or not isinstance(expected_binary.get("canonical_path"), str)):
        raise S3EvidenceBlockedError("production release package/version/bin source identity is unavailable")
    matches = []
    for artifact in artifacts:
        if not isinstance(artifact, Mapping):
            raise S3EvidenceBlockedError("release compiler-artifact message is malformed")
        package_id = artifact.get("package_id")
        manifest_path = artifact.get("manifest_path")
        target = artifact.get("target")
        profile = artifact.get("profile")
        if (not isinstance(package_id, str) or not isinstance(manifest_path, str)
                or not isinstance(target, Mapping) or not isinstance(profile, Mapping)):
            raise S3EvidenceBlockedError("release compiler-artifact fields are missing or malformed")
        target_name = target.get("name")
        target_kind = target.get("kind")
        target_source = target.get("src_path")
        if (not isinstance(target_name, str) or not isinstance(target_kind, list)
                or any(not isinstance(kind, str) for kind in target_kind)
                or not isinstance(target_source, str) or not PurePosixPath(target_source).is_absolute()):
            raise S3EvidenceBlockedError("release target identity is missing or malformed")
        if ("test" not in profile or not isinstance(profile.get("test"), bool)
                or not isinstance(profile.get("opt_level"), str)):
            raise S3EvidenceBlockedError("release build profile is missing or malformed")
        candidate = (
            manifest_path == expected_manifest or target_name == "parseltongue" or "bin" in target_kind
            or f"#{expected_package_name}@{expected_package_version}" in package_id
        )
        if not candidate:
            continue
        locator = re.fullmatch(r"(?:path\+)?file://(?P<directory>/[^#]*)#(?P<fragment>[^#]+)", package_id)
        if locator is None:
            raise S3EvidenceBlockedError("release package locator is not a supported Cargo file URL package ID")
        package_directory = unquote(locator.group("directory"))
        fragment = locator.group("fragment")
        if "@" in fragment:
            locator_name, locator_version = fragment.rsplit("@", 1)
        else:
            locator_name = PurePosixPath(package_directory).name
            locator_version = fragment
        if (package_directory != str(PurePosixPath(expected_manifest).parent)
                or locator_name != expected_package_name
                or locator_version != expected_package_version):
            raise S3EvidenceViolationError("release Cargo package locator differs from frozen production identity")
        if manifest_path != expected_manifest:
            raise S3EvidenceViolationError("release artifact manifest differs from frozen production manifest")
        if target_name != "parseltongue" or "bin" not in target_kind:
            raise S3EvidenceViolationError("release artifact is not the production parseltongue binary target")
        if target_source != expected_bin_source:
            raise S3EvidenceViolationError("release binary source differs from frozen production target")
        matches.append(artifact)
    if len(matches) != 1:
        if not matches:
            raise S3EvidenceBlockedError("release output lacks the frozen production binary artifact")
        raise S3EvidenceViolationError("release output duplicates the frozen production binary artifact")
    executable = matches[0].get("executable")
    profile = matches[0].get("profile", {})
    if not isinstance(executable, str) or not PurePosixPath(executable).is_absolute():
        raise S3EvidenceBlockedError("release executable path is missing or malformed")
    if profile.get("test") is not False or profile.get("opt_level") == "0":
        raise S3EvidenceViolationError("release artifact is not a non-test optimized binary")
    binary_path = PurePosixPath(executable)
    if str(binary_path) != expected_binary.get("canonical_path"):
        raise S3EvidenceViolationError("release executable path differs from the frozen mapper binary")


def _validate_p2_metadata_output(stdout_text, repository_binding):
    if not isinstance(repository_binding, Mapping):
        raise S3EvidenceBlockedError("P2 repository binding is unavailable")
    if repository_binding.get("p2_policy_supported") is not True:
        raise S3EvidenceBlockedError(
            repository_binding.get("p2_blocked_reason") or "P2 unit/target policy cannot be established"
        )
    metadata = parse_s3_evidence_json(stdout_text, "Cargo metadata")
    if not isinstance(metadata, Mapping) or not isinstance(metadata.get("workspace_members"), list):
        raise S3EvidenceBlockedError("Cargo metadata lacks workspace_members")
    if not isinstance(metadata.get("packages"), list):
        raise S3EvidenceBlockedError("Cargo metadata lacks package target inventory")
    workspace_members = metadata["workspace_members"]
    if any(not isinstance(member, str) for member in workspace_members):
        raise S3EvidenceBlockedError("Cargo metadata workspace member IDs are malformed")
    if not workspace_members or len(workspace_members) != len(set(workspace_members)):
        raise S3EvidenceBlockedError("Cargo metadata workspace_members is empty or duplicated")
    expected_members = repository_binding.get("workspace_members")
    if not isinstance(expected_members, list) or any(not isinstance(path, str) for path in expected_members):
        raise S3EvidenceBlockedError("pinned source workspace-member manifest inventory is unavailable")
    packages = {}
    for package in metadata["packages"]:
        if (not isinstance(package, Mapping) or not isinstance(package.get("id"), str)
                or not isinstance(package.get("manifest_path"), str)
                or not PurePosixPath(package.get("manifest_path", "")).is_absolute()
                or not isinstance(package.get("edition"), str)
                or not isinstance(package.get("targets"), list)):
            raise S3EvidenceBlockedError("Cargo metadata package record is malformed")
        if package["id"] in packages:
            raise S3EvidenceBlockedError("Cargo metadata package IDs are duplicated")
        for target in package["targets"]:
            if (not isinstance(target, Mapping) or not isinstance(target.get("name"), str)
                    or not isinstance(target.get("kind"), list)
                    or not target["kind"] or any(not isinstance(kind, str) for kind in target["kind"])):
                raise S3EvidenceBlockedError("Cargo metadata target record is malformed")
        packages[package["id"]] = package
    member_packages = [packages.get(member) for member in workspace_members]
    if any(package is None for package in member_packages):
        raise S3EvidenceBlockedError("Cargo metadata workspace member has no package record")
    actual_members = sorted(package.get("manifest_path") for package in member_packages)
    if actual_members != sorted(expected_members):
        raise S3EvidenceViolationError("metadata workspace member manifest differs from frozen Cargo.toml inventory")
    selected = []
    for member in workspace_members:
        package = packages.get(member)
        for target in package.get("targets", []):
            kind = target.get("kind", [])
            known_kinds = {
                "lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro", "bin",
                "example", "test", "bench", "custom-build",
            }
            unknown_kinds = set(kind) - known_kinds
            if unknown_kinds:
                raise S3EvidenceBlockedError(
                    f"unsupported P2 workspace target kinds in {target.get('name')}: {sorted(unknown_kinds)}"
                )
            unsupported = set(kind) & {"dylib", "cdylib", "staticlib"}
            if unsupported:
                raise S3EvidenceBlockedError(
                    f"unsupported P2 workspace crate type in {target.get('name')}: {sorted(unsupported)}"
                )
            if any(value in {"lib", "rlib", "proc-macro"} for value in kind) or "bin" in kind:
                source_path = target.get("src_path")
                if not isinstance(source_path, str) or not PurePosixPath(source_path).is_absolute():
                    raise S3EvidenceBlockedError("Cargo metadata target src_path is missing or unsupported")
                selected.append({
                    "package_id": member, "manifest_path": package.get("manifest_path"),
                    "target_name": target.get("name"), "kind": sorted(kind),
                    "edition": package.get("edition"), "src_path": source_path,
                })
    if not selected:
        raise S3EvidenceBlockedError("P2 metadata selects zero workspace lib/bin targets")
    selected_identity_rows = [
        (target["package_id"], target["target_name"], tuple(target["kind"])) for target in selected
    ]
    if len(selected_identity_rows) != len(set(selected_identity_rows)):
        raise S3EvidenceBlockedError("P2 metadata target inventory contains duplicates")
    expected_units = repository_binding.get("selected_units")
    if (not isinstance(expected_units, list)
            or any(not isinstance(unit, Mapping) or not isinstance(unit.get("manifest_path"), str)
                   or not isinstance(unit.get("target_name"), str) or not isinstance(unit.get("kind"), list)
                   or not isinstance(unit.get("src_path"), str)
                   or not PurePosixPath(unit.get("src_path", "")).is_absolute()
                   or any(not isinstance(kind, str) for kind in unit.get("kind", []))
                   for unit in expected_units)):
        raise S3EvidenceBlockedError("pinned source selected-unit inventory is malformed")
    actual_units = sorted(
        ({key: value for key, value in target.items() if key != "package_id"}
         for target in selected),
        key=lambda unit: (unit["manifest_path"], unit["target_name"], tuple(unit["kind"])),
    )
    expected_by_key = {
        (unit["manifest_path"], unit["target_name"], tuple(unit["kind"])): unit
        for unit in expected_units
    }
    actual_by_key = {
        (unit["manifest_path"], unit["target_name"], tuple(unit["kind"])): unit
        for unit in actual_units
    }
    if set(actual_by_key) != set(expected_by_key):
        raise S3EvidenceViolationError("P2 selected lib/bin units differ from frozen source inventory")
    for key, actual in actual_by_key.items():
        if actual["src_path"] != expected_by_key[key]["src_path"]:
            raise S3EvidenceViolationError("Cargo metadata target src_path differs from frozen source inventory")
        if actual["edition"] != expected_by_key[key]["edition"]:
            raise S3EvidenceViolationError("Cargo metadata edition differs from frozen source inventory")
    resolve = metadata.get("resolve")
    nodes = resolve.get("nodes") if isinstance(resolve, Mapping) else None
    if not isinstance(nodes, list):
        raise S3EvidenceBlockedError("Cargo metadata lacks resolved default-feature contexts")
    feature_by_package = {}
    for node in nodes:
        if (not isinstance(node, Mapping) or not isinstance(node.get("id"), str)
                or not isinstance(node.get("features"), list)
                or any(not isinstance(feature, str) for feature in node["features"])):
            raise S3EvidenceBlockedError("Cargo metadata resolve node is malformed")
        if node["id"] in feature_by_package:
            raise S3EvidenceBlockedError("Cargo metadata resolve node is duplicated")
        feature_by_package[node["id"]] = node["features"]
    if any(not isinstance(feature_by_package.get(member), list) for member in workspace_members):
        raise S3EvidenceBlockedError("Cargo metadata cannot establish every workspace feature context")
    for target in selected:
        target["features"] = sorted(feature_by_package[target["package_id"]])
    return {"metadata": metadata, "selected_targets": selected}


def _validate_p2_check_output(stdout_text, metadata_text, repository_binding):
    metadata_state = _validate_p2_metadata_output(metadata_text, repository_binding)
    messages = _cargo_json_messages(stdout_text)
    _validate_build_finished(messages, "P2 check")
    artifact_map = {}
    workspace_ids = set(metadata_state["metadata"]["workspace_members"])
    selected = metadata_state["selected_targets"]
    selected_keys = {
        (target["package_id"], target["target_name"], tuple(target["kind"])): target
        for target in selected
    }
    for artifact in messages:
        if not isinstance(artifact, Mapping):
            raise S3EvidenceBlockedError("P2 compiler-artifact message is malformed")
        if artifact.get("reason") != "compiler-artifact":
            continue
        package_id = artifact.get("package_id")
        target = artifact.get("target")
        manifest_path = artifact.get("manifest_path")
        profile = artifact.get("profile")
        features = artifact.get("features")
        if (not isinstance(package_id, str) or not isinstance(manifest_path, str)
                or not PurePosixPath(manifest_path).is_absolute() or not isinstance(target, Mapping)
                or not isinstance(target.get("name"), str) or not isinstance(target.get("kind"), list)
                or not isinstance(target.get("src_path"), str)
                or not PurePosixPath(target.get("src_path", "")).is_absolute()
                or any(not isinstance(kind_value, str) for kind_value in target["kind"])
                or not isinstance(target.get("edition"), str)
                or not isinstance(profile, Mapping) or not isinstance(profile.get("test"), bool)
                or not isinstance(features, list) or any(not isinstance(feature, str) for feature in features)):
            raise S3EvidenceBlockedError("P2 compiler-artifact fields are missing or malformed")
        kind = tuple(sorted(target["kind"]))
        if package_id not in workspace_ids:
            continue
        if kind == ("custom-build",):
            continue
        if profile.get("test") is not False:
            raise S3EvidenceBlockedError("P2 artifact is a test/unsupported target")
        key = (package_id, target.get("name"), kind)
        if key not in selected_keys:
            raise S3EvidenceBlockedError("P2 compiled a workspace target outside selected lib/bin policy")
        expected = selected_keys[key]
        if manifest_path != expected["manifest_path"]:
            raise S3EvidenceViolationError("P2 compiler-artifact manifest differs from frozen source inventory")
        if target["src_path"] != expected["src_path"]:
            raise S3EvidenceViolationError("P2 compiler-artifact src_path differs from frozen source inventory")
        if target.get("edition") != expected["edition"]:
            raise S3EvidenceViolationError("P2 artifact edition differs from metadata unit edition")
        if sorted(features) != expected["features"]:
            raise S3EvidenceViolationError("P2 artifact feature context differs from Cargo metadata")
        if key in artifact_map:
            raise S3EvidenceViolationError("P2 compiler-artifact row is duplicated")
        artifact_map[key] = artifact
    missing = []
    for target in selected:
        key = (target["package_id"], target["target_name"], tuple(target["kind"]))
        if key not in artifact_map:
            missing.append(key)
    if missing:
        raise S3EvidenceBlockedError(f"P2 check artifact coverage is incomplete: {missing}")
    if not artifact_map:
        raise S3EvidenceBlockedError("P2 check has zero selected compiler artifacts")
    return {"selected_artifacts": sorted(artifact_map)}
