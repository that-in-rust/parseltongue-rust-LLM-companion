"""Run untraced S3 telemetry separately from process and filtered-target audits."""
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
spec.loader.exec_module(auditor)


def build_corpus_measurement_commands(record, output):
    binary = str(ROOT / "target/release/parseltongue")
    common = [binary, "map-workspace-full-run", "--workspace-root-path-value",
              record["corpus_source_checkout_path"], "--worker-thread-count-value", "4",
              "--workspace-store-file-path"]
    measurement = [sys.executable, str(ROOT / "tools/process-tree-access-auditor.py"),
                   "sample", "--memory-sample-output-path", str(output / "process-tree-memory-samples.json"),
                   "--", *common, str(output / "production-untraced-workspace-store.db")]
    audit = ["strace", "--seccomp-bpf", "-f", "-s", "65535", "-e", "trace=execve,connect,clone,clone3,fork,vfork",
             "-o", str(output / "production-process-access-audit.txt"),
             *common, str(output / "production-audit-workspace-store.db")]
    reads = ["strace", "--seccomp-bpf", "-f", "-s", "65535", "-e", "trace=openat",
             "-P", str(Path(record["corpus_source_checkout_path"]) / "target"),
             "-o", str(output / "production-target-access-audit.txt"),
             *common, str(output / "production-target-workspace-store.db")]
    return measurement, audit, reads


def measure_corpus_production_stages(record):
    output = ROOT / record["corpus_evidence_directory_path"]
    if suffix := os.environ.get("PARSELTONGUE_CORPUS_RUN_DIRECTORY"):
        if Path(suffix).name != suffix or suffix in {".", ".."}:
            raise ValueError("run directory must be one relative path component")
        output = output / suffix
        output.mkdir(exist_ok=True)
    database = output / "production-untraced-workspace-store.db"
    memory = output / "process-tree-memory-samples.json"
    trace = output / "production-process-access-audit.txt"
    binary = ROOT / "target/release/parseltongue"
    command, audit_command, read_command = build_corpus_measurement_commands(record, output)
    environment = dict(os.environ, CARGO_TARGET_DIR=str(output / "cargo-discovery-cache"))
    with (output / "production-stage-output-log.txt").open("w") as stdout, (output / "production-stage-error-log.txt").open("w") as stderr:
        mapped = subprocess.run(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
    measurement = json.loads(memory.read_text())
    print(json.dumps({"corpus_repository_short_name": record["corpus_repository_short_name"],
                      "untraced_measurement_complete": True,
                      "elapsed_seconds": measurement["process_tree_elapsed_seconds"],
                      "exit_code": mapped.returncode}), flush=True)
    with (output / "production-audit-output-log.txt").open("w") as stdout, (output / "production-audit-error-log.txt").open("w") as stderr:
        audited = subprocess.run(audit_command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
    text = trace.read_text()
    root = re.search(r'^(\d+) execve\("[^"]*/parseltongue"', text, re.M)
    violations = auditor.audit_process_trace_events(text, int(root[1]) if root else 0)
    with (output / "production-target-output-log.txt").open("w") as stdout, (output / "production-target-error-log.txt").open("w") as stderr:
        reads = subprocess.run(read_command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
    target_violations = [line for line in (output / "production-target-access-audit.txt").read_text().splitlines()
                         if "openat(" in line and "O_WRONLY" not in line and ".rustc_info.json" not in line and " = -1 " not in line]
    (output / "production-process-audit-result.json").write_text(json.dumps({
        "process_audit_pass_flag": not violations, "process_audit_violation_rows": violations,
        "target_filtered_audit_pass": not target_violations, "target_filtered_violation_rows": target_violations,
        "target_filter_scope_value": "exact repo/target path; not recursive; source guards reject target descendants",
    }, indent=2) + "\n")
    status = subprocess.run([str(binary), "status-coverage-banner-show", "--json",
        "--workspace-store-file-path", str(database)], cwd=ROOT, text=True, capture_output=True, check=False)
    (output / "hld-current-coverage-banner.json").write_text(status.stdout)
    (output / "coverage-banner-error-log.txt").write_text(status.stderr)
    banner = json.loads(status.stdout) if status.returncode == 0 else None
    summary = {
        "corpus_repository_short_name": record["corpus_repository_short_name"],
        "production_mapper_exit_code": mapped.returncode,
        "production_status_exit_code": status.returncode,
        "production_process_audit_exit": audited.returncode,
        "production_target_audit_exit": reads.returncode,
        "production_target_audit_pass": not target_violations,
        "production_process_audit_pass": not violations,
        "production_audit_violation_count": len(violations),
        "production_total_elapsed_seconds": measurement["process_tree_elapsed_seconds"],
        "production_peak_rss_bytes": measurement["process_tree_peak_bytes"],
        "production_peak_rss_gib": measurement["process_tree_peak_bytes"] / 2**30,
        "production_coverage_banner_values": banner,
        "production_memory_budget_pass": 0 < measurement["process_tree_peak_bytes"] <= 2**31,
    }
    (output / "production-stage-measurement-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return summary


if __name__ == "__main__":
    records = json.loads((ROOT / "docs/phase-20261111/corpus-source-pin-manifest.json").read_text())
    if len(sys.argv) > 1:
        records = [record for record in records if record["corpus_repository_short_name"] in sys.argv[1:]]
    for record in records:
        print(json.dumps(measure_corpus_production_stages(record)), flush=True)
