"""Run separate untraced S3 telemetry and process-access audits."""

import importlib.util
import json
import os
import re
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("process_auditor", ROOT / "tools/process-tree-access-auditor.py")
auditor = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = auditor
spec.loader.exec_module(auditor)


def build_corpus_measurement_commands(
    record,
    output,
    mapper_executable_path=None,
    final_stage_kind=None,
):
    output = Path(output)
    binary = str(mapper_executable_path or record.get("corpus_mapper_binary_path", ROOT / "target/release/parseltongue"))
    final_stage = final_stage_kind or record.get("corpus_mapping_final_stage", "name-resolution")
    common = [binary, "map-workspace-full-run", "--workspace-root-path-value",
              record["corpus_source_checkout_path"], "--worker-thread-count-value", "4",
              "--mapping-final-stage-kind", final_stage, "--workspace-store-file-path"]
    measurement = [sys.executable, str(ROOT / "tools/process-tree-access-auditor.py"),
                   "sample", "--memory-sample-output-path", str(output / "process-tree-memory-samples.json"),
                   "--", *common, str(output / "production-untraced-workspace-store.db")]
    audit = ["strace", "--seccomp-bpf", "-f", "-yy", "-s", "65535", "-e", "trace=%process,%network",
             "-o", str(output / "production-process-access-audit.txt"),
             *common, str(output / "production-audit-workspace-store.db")]
    reads = ["strace", "--seccomp-bpf", "-f", "-s", "65535", "-e", "trace=openat",
             "-P", str(Path(record["corpus_source_checkout_path"]) / "target"),
             "-o", str(output / "production-target-access-audit.txt"),
             *common, str(output / "production-target-workspace-store.db")]
    return measurement, audit, reads


def audit_recorded_process_events(text, binary):
    root = re.search(r'^([0-9]+) execve\(' + re.escape(json.dumps(str(binary))) + ',', text, re.M)
    return auditor.audit_process_trace_events(text, int(root[1]) if root else 0, binary)


def invoke_production_stage_command(command_values, output_directory, output_prefix,
                                    environment, before_after_callback=None):
    if before_after_callback is not None:
        before_after_callback("before", command_values)
    try:
        with (output_directory / f"{output_prefix}-output-log.txt").open("w") as stdout, \
                (output_directory / f"{output_prefix}-error-log.txt").open("w") as stderr:
            completed = subprocess.run(command_values, cwd=ROOT, env=environment,
                                       stdout=stdout, stderr=stderr, check=False)
    finally:
        if before_after_callback is not None:
            before_after_callback("after", command_values)
    return completed


def validate_publication_stage_markers(banner, final_stage_kind):
    if not isinstance(banner, dict):
        return {"pass": False, "reason": "coverage banner is unavailable", "required_stages": []}
    stage_values = banner.get("stage_publication_status_values")
    if not isinstance(stage_values, dict):
        return {"pass": False, "reason": "coverage banner is missing stage publication statuses", "required_stages": []}
    required_stages = ["crate-graph", "source-ingest", "syntax-spans"]
    if final_stage_kind == "name-resolution":
        required_stages.append("name-resolution")
    missing_stages = [stage for stage in required_stages if stage_values.get(stage) != "complete"]
    generation = banner.get("run_generation_digest_value")
    source_tiling = banner.get("source_tiling_identity_holds")
    source_records = banner.get("source_file_record_count")
    if missing_stages:
        reason = f"required stages are not complete: {missing_stages}"
    elif not isinstance(generation, str) or not generation:
        reason = "published generation digest is missing"
    elif source_tiling is not True:
        reason = "source byte tiling marker is false or missing"
    elif not isinstance(source_records, int) or source_records <= 0:
        reason = "source file record count is missing or empty"
    else:
        reason = None
    return {"pass": reason is None, "reason": reason, "required_stages": required_stages}


def audit_target_path_trace(trace_text, target_path, wrapper_return_code=None):
    target_root = Path(target_path).absolute()
    violations = []
    unfinished_processes = set()
    exit_statuses = []
    malformed_resume = False
    for line in trace_text.splitlines():
        process_match = re.match(r"^\s*(?:\[pid\s+)?(\d+)\]?\s+", line)
        process_id = process_match.group(1) if process_match else None
        if "<unfinished ...>" in line:
            if process_id is not None:
                unfinished_processes.add(process_id)
            else:
                unfinished_processes.add("unknown")
        elif " resumed>" in line:
            if process_id is None or process_id not in unfinished_processes:
                malformed_resume = True
                violations.append(f"orphan resumed target trace: {line}")
            else:
                unfinished_processes.remove(process_id)
        exit_match = re.match(
            r"^\s*(?:\[pid\s+)?(\d+)\]?\s+\+\+\+ exited with (\d+) \+\+\+\s*$", line
        )
        if exit_match is not None:
            exit_statuses.append(int(exit_match.group(2)))
        if "openat(" not in line:
            continue
        if "O_WRONLY" in line or " = -1 " in line:
            continue
        path_match = re.search(r'openat\([^,]+,\s*("(?:\\.|[^"\\])*")', line)
        if path_match is None:
            violations.append(line)
            continue
        try:
            opened_path = Path(json.loads(path_match.group(1)))
        except (TypeError, ValueError, json.JSONDecodeError):
            violations.append(line)
            continue
        if not opened_path.is_absolute():
            opened_path = Path.cwd() / opened_path
        opened_path = opened_path.absolute()
        within_target = opened_path == target_root or target_root in opened_path.parents
        if within_target and opened_path.name != ".rustc_info.json":
            violations.append(line)
    wrapper_exit_verified = (
        isinstance(wrapper_return_code, int) and not isinstance(wrapper_return_code, bool)
        and wrapper_return_code == 0
    )
    complete = (
        bool(exit_statuses) and all(status == 0 for status in exit_statuses)
        and wrapper_exit_verified and not unfinished_processes and not malformed_resume
    )
    if unfinished_processes:
        violations.extend(
            f"unfinished target trace for process {process_id}" for process_id in sorted(unfinished_processes)
        )
    return {
        "pass": complete and not violations,
        "complete": complete,
        "wrapper_exit_code": wrapper_return_code,
        "violations": violations,
        "filter_scope": "exact target path; not recursive beyond the supplied strace filter",
    }


def measure_corpus_production_stages(
    record,
    output_directory_path=None,
    mapper_executable_path=None,
    before_after_callback=None,
    final_stage_kind=None,
):
    explicit_output = output_directory_path is not None
    output = Path(output_directory_path) if explicit_output else ROOT / record["corpus_evidence_directory_path"]
    if explicit_output and output.exists() and any(output.iterdir()):
        raise FileExistsError(f"refusing to reuse production artifacts in {output}")
    output.mkdir(parents=True, exist_ok=True)
    database = output / "production-untraced-workspace-store.db"
    memory = output / "process-tree-memory-samples.json"
    trace = output / "production-process-access-audit.txt"
    binary = Path(mapper_executable_path or record.get("corpus_mapper_binary_path", ROOT / "target/release/parseltongue"))
    command, audit_command, read_command = build_corpus_measurement_commands(
        record, output, binary, final_stage_kind
    )
    environment = dict(os.environ, CARGO_TARGET_DIR=str(output / "cargo-discovery-cache"))
    summary = {
        "corpus_repository_short_name": record["corpus_repository_short_name"],
        "production_artifact_directory": str(output),
        "production_mapper_status": "NOT STARTED",
        "production_mapper_exit_code": None,
        "production_measurement_status": "NOT STARTED",
        "production_process_audit_status": "NOT STARTED",
        "production_process_audit_exit": None,
        "production_target_audit_status": "NOT STARTED",
        "production_target_audit_exit": None,
        "production_status_command_status": "NOT STARTED",
        "production_status_exit_code": None,
        "production_publication_status": "NOT STARTED",
        "production_process_audit_pass": None,
        "production_target_audit_pass": None,
        "production_trace_nonempty": None,
        "production_process_trace_complete": None,
        "production_target_trace_nonempty": None,
        "production_target_trace_complete": None,
        "production_target_audit_recursive": False,
        "production_total_elapsed_seconds": None,
        "production_peak_rss_bytes": None,
        "production_peak_rss_sampling_interval_ms": None,
        "production_peak_rss_sampling_exact": False,
        "production_peak_rss_sampling_limitation": "nominal 10-ms sampling can miss short-lived children and exact peaks",
        "production_memory_acceptance_status": "DEFERRED",
        "production_memory_sample_count": 0,
        "production_memory_sample_error": None,
        "production_coverage_banner_values": None,
        "publication_stage_marker_result": None,
    }
    summary_path = output / "production-stage-measurement-summary.json"

    def persist_measurement_summary():
        summary_path.write_text(json.dumps(summary, indent=2) + "\n")

    persist_measurement_summary()

    mapped = invoke_production_stage_command(command, output, "production-stage", environment, before_after_callback)
    summary["production_mapper_exit_code"] = mapped.returncode
    summary["production_mapper_status"] = "PASS" if mapped.returncode == 0 else "FAIL"
    measurement = None
    measurement_error = None
    try:
        measurement = json.loads(memory.read_text())
    except (OSError, json.JSONDecodeError) as error:
        measurement_error = f"memory sample output missing or malformed: {error}"
    samples = measurement.get("process_memory_sample_values") if isinstance(measurement, dict) else None
    peak_bytes = measurement.get("process_tree_peak_bytes") if isinstance(measurement, dict) else None
    summary.update({
        "production_measurement_status": (
            "PASS" if isinstance(samples, list) and samples and isinstance(peak_bytes, int)
            and isinstance(measurement.get("process_tree_elapsed_seconds"), (int, float)) else "BLOCKED"
        ),
        "production_total_elapsed_seconds": (
            measurement.get("process_tree_elapsed_seconds") if isinstance(measurement, dict) else None
        ),
        "production_peak_rss_bytes": peak_bytes,
        "production_peak_rss_sampling_interval_ms": (
            measurement.get("sample_interval_millisecond_value") if isinstance(measurement, dict) else None
        ),
        "production_memory_sample_count": len(samples) if isinstance(samples, list) else 0,
        "production_memory_sample_error": measurement_error,
    })
    persist_measurement_summary()
    if mapped.returncode != 0:
        return summary

    audited = invoke_production_stage_command(audit_command, output, "production-audit", environment, before_after_callback)
    try:
        trace_text = trace.read_text()
    except OSError:
        trace_text = ""
    complete_process_events, pending_process_events = auditor.read_complete_trace_events(trace_text)
    process_trace_complete = bool(complete_process_events) and not pending_process_events
    violations = audit_recorded_process_events(trace_text, binary) if trace_text else ["process access trace missing"]
    if not process_trace_complete:
        violations.append("process access trace incomplete")

    incomplete_trace_markers = ("incomplete trace", "unfinished trace", "trace missing")
    observed_violations = [
        value for value in violations
        if not any(marker in value for marker in incomplete_trace_markers)
    ]
    if audited.returncode != 0 or observed_violations:
        process_audit_status = "FAIL"
    elif not process_trace_complete:
        process_audit_status = "BLOCKED"
    else:
        process_audit_status = "PASS"
    summary.update({
        "production_process_audit_status": process_audit_status,
        "production_process_audit_exit": audited.returncode,
        "production_process_audit_pass": process_audit_status == "PASS",
        "production_trace_nonempty": bool(trace_text.strip()),
        "production_process_trace_complete": process_trace_complete,
    })
    process_audit = {
        "process_audit_status": process_audit_status,
        "process_audit_pass_flag": process_audit_status == "PASS",
        "process_audit_exit_code": audited.returncode,
        "process_audit_violation_rows": violations,
        "process_trace_complete": process_trace_complete,
        "target_audit_status": "NOT STARTED",
        "target_filtered_audit_pass": None,
        "target_filtered_violation_rows": [],
        "target_trace_complete": None,
        "target_filter_scope_value": "exact repo/target path only; not recursive",
        "target_filter_complete_recursive_audit": False,
        "target_trace_nonempty": None,
    }
    process_audit_path = output / "production-process-audit-result.json"
    process_audit_path.write_text(json.dumps(process_audit, indent=2) + "\n")
    if process_audit_status != "PASS":
        persist_measurement_summary()
        return summary

    reads = invoke_production_stage_command(read_command, output, "production-target", environment, before_after_callback)
    target_trace_path = output / "production-target-access-audit.txt"
    try:
        target_trace_text = target_trace_path.read_text()
        target_trace_available = bool(target_trace_text.strip())
    except OSError:
        target_trace_text = ""
        target_trace_available = False
    target_audit = audit_target_path_trace(
        target_trace_text, Path(record["corpus_source_checkout_path"]) / "target",
        wrapper_return_code=reads.returncode,
    )
    target_violations = target_audit["violations"]
    if reads.returncode != 0 or target_violations:
        target_audit_status = "FAIL"
    elif not target_audit["complete"]:
        target_audit_status = "BLOCKED"
    else:
        target_audit_status = "PASS"
    summary.update({
        "production_target_audit_status": target_audit_status,
        "production_target_audit_exit": reads.returncode,
        "production_target_audit_pass": target_audit_status == "PASS",
        "production_target_trace_nonempty": target_trace_available,
        "production_target_trace_complete": target_audit["complete"],
    })
    process_audit.update({
        "target_audit_status": target_audit_status,
        "target_filtered_audit_pass": target_audit_status == "PASS",
        "target_filtered_violation_rows": target_violations,
        "target_trace_complete": target_audit["complete"],
        "target_trace_nonempty": target_trace_available,
    })
    process_audit_path.write_text(json.dumps(process_audit, indent=2) + "\n")
    if target_audit_status != "PASS":
        persist_measurement_summary()
        return summary

    status_command = [str(binary), "status-coverage-banner-show", "--json",
                      "--workspace-store-file-path", str(database)]
    if before_after_callback is not None:
        before_after_callback("before", status_command)
    try:
        status = subprocess.run(status_command, cwd=ROOT, text=True, capture_output=True, check=False)
    finally:
        if before_after_callback is not None:
            before_after_callback("after", status_command)
    status_path = output / "hld-current-coverage-banner.json"
    status_path.write_text(status.stdout)
    (output / "coverage-banner-error-log.txt").write_text(status.stderr)
    try:
        banner = json.loads(status.stdout) if status.returncode == 0 else None
    except json.JSONDecodeError:
        banner = None
    marker_result = validate_publication_stage_markers(banner, final_stage_kind or record.get(
        "corpus_mapping_final_stage", "name-resolution"
    ))
    if status.returncode != 0:
        publication_status = "FAIL"
    elif marker_result.get("pass") is True:
        publication_status = "PASS"
    elif any(token in str(marker_result.get("reason", "")).lower()
             for token in ("missing", "unavailable")):
        publication_status = "BLOCKED"
    else:
        publication_status = "FAIL"

    summary.update({
        "production_status_command_status": "PASS" if status.returncode == 0 else "FAIL",
        "production_status_exit_code": status.returncode,
        "production_publication_status": publication_status,
        "production_coverage_banner_values": banner,
        "publication_stage_marker_result": marker_result,
    })
    samples = measurement.get("process_memory_sample_values") if isinstance(measurement, dict) else None
    peak_bytes = measurement.get("process_tree_peak_bytes") if isinstance(measurement, dict) else None
    summary.update({
        "production_process_audit_violation_count": len(violations),
        "production_trace_nonempty": bool(trace_text.strip()),
        "production_target_audit_recursive": False,
        "production_peak_rss_sampling_exact": False,
        "production_peak_rss_sampling_limitation": "nominal 10-ms sampling can miss short-lived children and exact peaks",
        "production_memory_acceptance_status": "DEFERRED",
        "production_artifact_directory": str(output),
    })
    summary["production_memory_sample_count"] = len(samples) if isinstance(samples, list) else 0
    summary["production_memory_sample_error"] = measurement_error
    persist_measurement_summary()
    return summary


def main():
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("repository_names", nargs="*")
    parser.add_argument("--output-directory", type=Path)
    parser.add_argument("--mapper-executable", type=Path)
    parser.add_argument("--final-stage-kind", choices=("syntax-spans", "name-resolution"))
    arguments = parser.parse_args()
    records = json.loads((ROOT / "docs/phase-20261111/corpus-source-pin-manifest.json").read_text())
    if arguments.repository_names:
        records = [record for record in records if record["corpus_repository_short_name"] in arguments.repository_names]
    if len(records) > 1 and arguments.output_directory:
        parser.error("--output-directory can be used with one repository at a time")
    for record in records:
        summary = measure_corpus_production_stages(
            record,
            arguments.output_directory,
            arguments.mapper_executable,
            final_stage_kind=arguments.final_stage_kind,
        )
        print(json.dumps(summary), flush=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
