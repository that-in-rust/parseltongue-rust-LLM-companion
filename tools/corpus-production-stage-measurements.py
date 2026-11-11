"""Run the five production maps under strace and sampled mapper-tree RSS."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("process_auditor", ROOT / "tools/process-tree-access-auditor.py")
auditor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(auditor)


def measure_corpus_production_stages(record):
    output = ROOT / record["corpus_evidence_directory_path"]
    database = output / "hld-current-workspace-store.db"
    memory = output / "process-tree-memory-samples.json"
    trace = output / "production-process-access-audit.txt"
    binary = ROOT / "target/release/parseltongue"
    command = [
        "strace", "-f", "-s", "65535", "-e", "trace=process,openat,connect,sendto,sendmsg",
        "-o", str(trace), sys.executable, str(ROOT / "tools/process-tree-access-auditor.py"),
        "sample", "--memory-sample-output-path", str(memory), "--",
        str(binary), "map-workspace-full-run",
        "--workspace-root-path-value", record["corpus_source_checkout_path"],
        "--workspace-store-file-path", str(database), "--worker-thread-count-value", "4",
    ]
    environment = dict(os.environ, CARGO_TARGET_DIR=str(output / "cargo-discovery-cache"))
    with (output / "production-stage-output-log.txt").open("w") as stdout, (output / "production-stage-error-log.txt").open("w") as stderr:
        mapped = subprocess.run(command, cwd=ROOT, env=environment, stdout=stdout, stderr=stderr, check=False)
    measurement = json.loads(memory.read_text())
    violations = auditor.audit_process_trace_events(trace.read_text(), measurement["mapper_process_identifier_value"])
    (output / "production-process-audit-result.json").write_text(json.dumps({
        "process_audit_pass_flag": not violations, "process_audit_violation_rows": violations,
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
