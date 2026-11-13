#!/usr/bin/env python3
"""Persistent orchestration for HLD-current SOP M1 and M2 tool gates."""

from __future__ import annotations

import argparse
from collections.abc import Callable, Mapping, Sequence
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import signal
import shutil
import stat
import subprocess
import sys
import time
import tomllib
from typing import Any

sys.dont_write_bytecode = True


PROJECT_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_REPOSITORY_ORDER = ("serde", "rust-analyzer", "ripgrep", "tokio", "cargo-modules")
RUST_ANALYZER_COMMIT = "bf6b2099d6351c37039b01447a9bb64f9a54a586"
R1_PATCH_SHA256 = "89146042a1c1629ef67c730c0e180dec15264577f675de1c090dca2d87da6bfa"
SOURCE_SUFFIXES = {
    ".c", ".cc", ".cfg", ".conf", ".cpp", ".cs", ".css", ".diff", ".ex", ".exs",
    ".fs", ".fsx", ".go", ".h", ".hpp", ".hrl", ".hs", ".html", ".inc", ".ini",
    ".java", ".js", ".json", ".jsx", ".kt", ".kts", ".lock", ".lua", ".m", ".md",
    ".mm", ".mod", ".nix", ".patch", ".php", ".pl", ".proto", ".py", ".r", ".rb",
    ".rs", ".scala", ".sc", ".sh", ".sql", ".sum", ".swift", ".toml", ".ts", ".tsx",
    ".txt", ".vb", ".vue", ".xml", ".yaml", ".yml",
}
GENERATED_SOURCE_DIRECTORY_NAMES = {".git", "target"}
S3_COMPONENT_NAMES = ("mapping", "measurement", "process_access_audit", "publication")
S3_GATE_NAMES = ("fixture_suite", "build_evidence", "dependency_audit",
                 "p2_offline_qualification", "release_provenance")
S3_RELEVANT_ENVIRONMENT_NAMES = {
    "PATH", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN", "RUSTC", "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET",
    "CARGO_TARGET_DIR", "RUSTC_BOOTSTRAP", "CC", "CXX", "AR", "CFLAGS", "CXXFLAGS", "CPPFLAGS",
    "LDFLAGS", "PKG_CONFIG", "PKG_CONFIG_PATH", "PKG_CONFIG_LIBDIR",
}
MAX_COMMAND_EVENT_BYTES = 16_384
MAX_COMMAND_JOURNAL_BYTES = 8 * 1024 * 1024
MAX_COMMAND_EVENT_COUNT = 512
DEPENDENCY_TREE_TIMEOUT_SECONDS = 60.0
DEPENDENCY_TREE_CLEANUP_TIMEOUT_SECONDS = 5.0


class RunnerPreconditionBlockedError(RuntimeError):
    """A prerequisite or evidence item is absent or cannot be verified."""


class RunnerContractViolationError(RuntimeError):
    """Observed data or execution violates the qualification contract."""


class RunnerFrozenInputMutationError(RunnerContractViolationError):
    """A frozen input changed or could no longer be verified after preflight."""


def load_runner_support_module(module_name: str, module_path: Path):
    module_spec = importlib.util.spec_from_file_location(module_name, module_path)
    if module_spec is None or module_spec.loader is None:
        raise RunnerPreconditionBlockedError(f"cannot load support module {module_path}")
    module_value = importlib.util.module_from_spec(module_spec)
    sys.modules[module_name] = module_value
    module_spec.loader.exec_module(module_value)
    return module_value


coverage_inventory = load_runner_support_module(
    "hld_runner_coverage_inventory", PROJECT_ROOT / "tools/corpus-index-coverage-inventory.py"
)
production_measurements = load_runner_support_module(
    "hld_runner_production_measurements", PROJECT_ROOT / "tools/corpus-production-stage-measurements.py"
)
process_auditor = load_runner_support_module(
    "hld_runner_process_auditor", PROJECT_ROOT / "tools/process-tree-access-auditor.py"
)
s3_evidence_intake = load_runner_support_module(
    "hld_runner_s3_evidence_intake", PROJECT_ROOT / "tools/s3-prerequisite-evidence-intake.py"
)


def current_utc_timestamp_text() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def probe_toolchain_versions() -> Mapping[str, Mapping[str, Any]]:
    versions = {}
    for tool_name in ("rustc", "cargo"):
        executable_path = shutil.which(tool_name)
        if not executable_path:
            versions[tool_name] = {
                "path": None, "canonical_path": None,
                "version": "unavailable", "return_code": None, "file_identity": None,
            }
            continue
        file_identity = capture_frozen_file_identity(Path(executable_path))
        canonical_path = str(Path(executable_path).resolve(strict=True))
        try:
            result = subprocess.run([executable_path, "--version"], text=True,
                                    capture_output=True, check=False)
        except OSError as error:
            versions[tool_name] = {
                "path": executable_path, "canonical_path": canonical_path,
                "version": "unavailable", "return_code": None,
                "error": str(error), "file_identity": file_identity,
            }
            continue
        verify_frozen_file_identity(file_identity)
        version_text = result.stdout.strip() or result.stderr.strip() or "unavailable"
        versions[tool_name] = {
            "path": executable_path,
            "canonical_path": canonical_path,
            "version": version_text if result.returncode == 0 else "unavailable",
            "return_code": result.returncode,
            "error": result.stderr.strip() if result.returncode != 0 else None,
            "file_identity": file_identity,
        }
    return versions


def append_frozen_identity_report(run_state: Mapping[str, Any]) -> None:
    manifest = run_state["manifest"]
    identity = manifest["input_identity"]
    mapper = identity["mapper"]
    entries = [
        "## Frozen execution identities",
        f"- Mapper canonical executable: `{mapper['canonical_path']}`; SHA-256 `{mapper['sha256']}`; "
        f"BLAKE3 `{mapper.get('blake3', 'unavailable')}`.",
        f"- Input corpus manifest: `{identity['frozen_manifest_paths'][0]}`.",
        f"- BLAKE3 provider: `{identity['b3_provider']}`.",
    ]
    for input_path, file_identity in identity.get("frozen_file_identities", {}).items():
        entries.append(
            f"- Frozen input `{input_path}` SHA-256 `{file_identity['sha256']}` "
            f"(size `{file_identity['size_bytes']}` bytes)."
        )
    for tool_name in ("indexer", "strace"):
        tool_identity = identity[tool_name]
        canonical_path = tool_identity.get("canonical_path", tool_identity["path"])
        entries.append(
            f"- Frozen `{tool_name}` executable `{canonical_path}`; "
            f"SHA-256 `{tool_identity['sha256']}`."
        )
    for repository_name, source_snapshot in identity["repository_source_snapshots"].items():
        entries.append(
            f"- `{repository_name}` frozen source HEAD `{source_snapshot['head_commit']}`; "
            f"source-tree SHA-256 `{source_snapshot['source_tree_sha256']}`; "
            f"tracked inputs `{source_snapshot['tracked_source_file_count']}`."
        )
    for lock_path, lock_identity in identity["cargo_lock_files"].items():
        entries.append(f"- Cargo lock `{lock_path}` SHA-256 `{lock_identity['sha256']}`.")
    for tool_name, tool_identity in manifest["runtime_versions"].get("toolchain_versions", {}).items():
        entries.append(
            f"- Toolchain `{tool_name}` path `{tool_identity['path'] or 'unavailable'}`; "
            f"version `{tool_identity['version']}`."
        )
    for key in ("milestone", "mode", "indexer_path", "project_root_overrides"):
        entries.append(f"- Configuration `{key}`: `{manifest['configuration'].get(key)}`.")
    append_persistent_report_entry(run_state, "\n".join(entries))


def json_default_value(value: Any) -> str:
    if isinstance(value, Path):
        return str(value)
    raise TypeError(f"cannot encode {type(value).__name__} as JSON")


def write_json_atomically(output_path: Path, value: Mapping[str, Any]) -> None:
    temporary_path = output_path.with_name(output_path.name + ".tmp")
    temporary_path.write_text(json.dumps(value, indent=2, default=json_default_value) + "\n")
    os.replace(temporary_path, output_path)


def append_persistent_report_entry(run_state: Mapping[str, Any], entry_text: str) -> None:
    report_path = Path(run_state["report_path"])
    with report_path.open("a", encoding="utf-8") as report_file:
        report_file.write(entry_text.rstrip() + "\n")


def format_report_value(value: Any) -> str:
    if value is None:
        return "unavailable"
    if isinstance(value, (Mapping, list)):
        return json.dumps(value, sort_keys=True)
    return str(value)


def build_initial_repository_state() -> dict[str, Any]:
    return {
        "steps": {step_name: "NOT STARTED" for step_name in ("S1", "S2", "S3")},
        "s3_components": {component: "NOT STARTED" for component in S3_COMPONENT_NAMES},
        "s3_qualification_gates": {
            gate: {"status": "NOT STARTED", "reason": None}
            for gate in (
                "fixture_suite", "build_evidence", "dependency_audit",
                "p2_offline_qualification", "release_provenance",
            )
        },
        "first_failed_clause": None,
        "first_reason": None,
        "events": [],
    }


def calculate_overall_run_status(manifest_value: Mapping[str, Any]) -> str:
    if manifest_value.get("execution_identity") == "HARNESS ONLY":
        return "HARNESS ONLY"
    statuses = []
    for repository_state in manifest_value["repositories"].values():
        statuses.extend(repository_state["steps"].values())
    for evidence_kind in ("R4", "R5"):
        statuses.extend(
            result.get("status", "NOT STARTED")
            for result in manifest_value.get("external_evidence", {}).get(evidence_kind, {}).values()
        )
    statuses.extend(gate["status"] for gate in manifest_value["milestone_gates"].values())
    for status_field in (
        "preflight", "input_integrity_status", "artifact_digest_status", "finalization_status"
    ):
        status = manifest_value.get(status_field)
        if isinstance(status, Mapping):
            status = status.get("status")
        if status in {"PASS", "FAIL", "BLOCKED"}:
            statuses.append(status)
    if "FAIL" in statuses:
        return "FAIL"
    if "BLOCKED" in statuses:
        return "BLOCKED"
    if statuses and all(status == "PASS" for status in statuses):
        return "PASS"
    if any(status == "PASS" for status in statuses):
        return "IN PROGRESS"
    return "NOT STARTED"


def merge_s3_intake_error(current_error, new_error):
    if current_error is None:
        return new_error
    priority = {"PASS": 0, "BLOCKED": 1, "FAIL": 2}
    current_status = current_error[0] if isinstance(current_error, tuple) and current_error else "BLOCKED"
    new_status = new_error[0] if isinstance(new_error, tuple) and new_error else "BLOCKED"
    return new_error if priority.get(new_status, 1) > priority.get(current_status, 1) else current_error


def persist_run_manifest(run_state: Mapping[str, Any]) -> None:
    manifest_value = run_state["manifest"]
    manifest_value["overall_status"] = calculate_overall_run_status(manifest_value)
    write_json_atomically(Path(run_state["manifest_path"]), manifest_value)


def create_persistent_run_report(
    report_root: Path,
    run_identifier: str,
    milestone: str,
    repository_names: Sequence[str] = DEFAULT_REPOSITORY_ORDER,
    harness_only_flag: bool = False,
) -> Mapping[str, Any]:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}", run_identifier):
        raise ValueError("run identifier must be a safe, unique path component")
    if milestone not in {"M1", "M2"}:
        raise ValueError("milestone must be M1 or M2")
    if set(repository_names) != set(DEFAULT_REPOSITORY_ORDER) or len(repository_names) != len(DEFAULT_REPOSITORY_ORDER):
        raise ValueError("a persistent SOP report must include each of the five pinned repositories")
    report_root = Path(report_root).expanduser().absolute()
    report_root.mkdir(parents=True, exist_ok=True)
    run_directory = report_root / run_identifier
    run_directory.mkdir(exist_ok=False)
    report_path = run_directory / "hld-current-test-report.md"
    manifest_path = run_directory / "run-manifest.json"
    if report_path.exists() or manifest_path.exists():
        raise FileExistsError(f"refusing to overwrite files in {run_directory}")
    future_gate_names = ("S10",) if milestone == "M1" else ("S4", "S5")
    manifest_value: dict[str, Any] = {
        "run_id": run_identifier,
        "milestone": milestone,
        "created_at": current_utc_timestamp_text(),
        "mode": "UNSET",
        "execution_identity": "HARNESS ONLY" if harness_only_flag else "QUALIFICATION",
        "overall_status": "NOT STARTED",
        "preflight": {"status": "NOT STARTED", "reason": None},
        "input_integrity_status": "NOT STARTED",
        "artifact_digest_status": "NOT STARTED",
        "finalization_status": "NOT STARTED",
        "repositories": {name: build_initial_repository_state() for name in repository_names},
        "milestone_gates": {
            gate_name: {"status": "NOT STARTED", "reason": "separate evidence gate not yet evaluated"}
            for gate_name in future_gate_names
        },
        "external_evidence": {"R4": {}, "R5": {}},
        "events": [],
    }
    report_path.write_text(
        "# HLD-current SOP qualification run\n\n"
        f"- Run ID: `{run_identifier}`\n- Milestone: `{milestone}`\n"
        f"- Identity: `{manifest_value['execution_identity']}`\n"
        "- Overall status: `NOT STARTED`\n\n"
        "## Repository steps\n\n"
        "All five repositories begin NOT STARTED; no evidence is inferred from prior runs.\n\n"
    )
    write_json_atomically(manifest_path, manifest_value)
    return {
        "run_directory": run_directory,
        "report_path": report_path,
        "manifest_path": manifest_path,
        "manifest": manifest_value,
    }


def record_persistent_run_transition(
    run_state: Mapping[str, Any],
    repository_name: str,
    step_name: str,
    status_value: str,
    command_values: Sequence[str] = (),
    reason_text: str | None = None,
    clause_name: str | None = None,
    command_metadata: Sequence[Mapping[str, Any]] | None = None,
) -> None:
    if status_value not in {"NOT STARTED", "PASS", "BLOCKED", "FAIL"}:
        raise ValueError(f"unsupported step status: {status_value}")
    manifest_value = run_state["manifest"]
    if repository_name not in manifest_value["repositories"]:
        raise KeyError(f"unknown repository {repository_name}")
    repository_state = manifest_value["repositories"][repository_name]
    if step_name not in repository_state["steps"]:
        raise KeyError(f"unknown SOP step {step_name}")
    repository_state["steps"][step_name] = status_value
    event = {
        "timestamp": current_utc_timestamp_text(),
        "repository": repository_name,
        "step": step_name,
        "status": status_value,
        "commands": list(command_values),
        "reason": reason_text,
        "clause": clause_name,
    }
    if command_metadata is not None:
        event["command_metadata"] = [dict(value) for value in command_metadata]
    repository_state["events"].append(event)
    manifest_value["events"].append(event)
    if status_value in {"BLOCKED", "FAIL"} and repository_state["first_failed_clause"] is None:
        repository_state["first_failed_clause"] = clause_name or step_name
        repository_state["first_reason"] = reason_text or status_value
    rendered_reason = f" — {reason_text}" if reason_text else ""
    rendered_metadata = (
        f"\n  - Command metadata: `{json.dumps(list(command_metadata), sort_keys=True)}`"
        if command_metadata is not None else ""
    )
    append_persistent_report_entry(
        run_state,
        f"- [{event['timestamp']}] `{repository_name}` `{step_name}`: **{status_value}**{rendered_reason}\n"
        f"  - Clause: `{clause_name or 'not specified'}`\n"
        f"  - Commands: `{json.dumps(list(command_values))}`{rendered_metadata}",
    )
    persist_run_manifest(run_state)


def record_persistent_gate_transition(
    run_state: Mapping[str, Any], gate_name: str, status_value: str, reason_text: str
) -> None:
    manifest_value = run_state["manifest"]
    if gate_name not in manifest_value["milestone_gates"]:
        raise KeyError(f"gate {gate_name} is not applicable to {manifest_value['milestone']}")
    event = {"timestamp": current_utc_timestamp_text(), "gate": gate_name,
             "status": status_value, "reason": reason_text}
    manifest_value["milestone_gates"][gate_name] = {"status": status_value, "reason": reason_text}
    manifest_value["events"].append(event)
    append_persistent_report_entry(
        run_state, f"- [{event['timestamp']}] Gate `{gate_name}`: **{status_value}** — {reason_text}"
    )
    persist_run_manifest(run_state)


def record_preflight_outcome(
    run_state: Mapping[str, Any], status_value: str, reason_text: str, clause_name: str
) -> None:
    manifest_value = run_state["manifest"]
    manifest_value["preflight"] = {
        "status": status_value,
        "reason": reason_text,
        "clause": clause_name,
        "timestamp": current_utc_timestamp_text(),
    }
    if manifest_value.get("input_integrity_status") not in {"PASS", "FAIL", "BLOCKED"}:
        manifest_value["input_integrity_status"] = "BLOCKED"
        manifest_value["input_integrity_reason"] = "frozen inputs were not fully verified during preflight"
    append_persistent_report_entry(
        run_state,
        f"- Preflight: **{status_value}** — `{clause_name}` — {reason_text}. Repository steps remain NOT STARTED.",
    )
    persist_run_manifest(run_state)


def record_finalization_outcome(run_state: Mapping[str, Any], status: str, reason: str) -> None:
    run_state["manifest"]["finalization_status"] = status
    run_state["manifest"]["finalization_reason"] = reason
    append_persistent_report_entry(run_state, f"- Post-run finalization: **{status}** — {reason}.")
    persist_run_manifest(run_state)


def compute_sha256_file_digest(file_path: Path) -> str:
    digest = hashlib.sha256()
    with file_path.open("rb") as input_file:
        for content_chunk in iter(lambda: input_file.read(1024 * 1024), b""):
            digest.update(content_chunk)
    return digest.hexdigest()


def capture_frozen_file_identity(file_path: Path) -> Mapping[str, Any]:
    file_path = Path(file_path).absolute()
    try:
        file_stat = file_path.lstat()
    except FileNotFoundError as error:
        raise RunnerPreconditionBlockedError(f"required frozen file is missing: {file_path}") from error
    is_symlink = file_path.is_symlink()
    try:
        resolved_path = file_path.resolve(strict=True) if is_symlink else file_path
        resolved_stat = resolved_path.stat()
    except (OSError, RuntimeError) as error:
        raise RunnerPreconditionBlockedError(f"frozen symlink target is unavailable: {file_path}: {error}") from error
    if not resolved_path.is_file():
        raise RunnerPreconditionBlockedError(f"frozen input is not a regular file: {file_path}")
    content_digest = compute_sha256_file_digest(resolved_path)
    identity = {
        "path": str(file_path),
        "size_bytes": resolved_stat.st_size,
        "mode": file_stat.st_mode,
        "sha256": content_digest,
        "is_symlink": is_symlink,
        "symlink_target": os.readlink(file_path) if is_symlink else None,
    }
    if is_symlink:
        identity["symlink_target_sha256"] = content_digest
        identity["symlink_resolved_path"] = str(resolved_path)
    return identity


def verify_frozen_file_identity(file_identity: Mapping[str, Any]) -> None:
    file_path = Path(file_identity["path"]).absolute()
    current_path = Path(file_path.anchor)
    for component in file_path.parts[1:-1]:
        current_path /= component
        if current_path.is_symlink():
            raise RunnerFrozenInputMutationError(
                f"frozen file ancestor became a symlink: {current_path}"
            )
    try:
        current_identity = capture_frozen_file_identity(file_path)
    except RunnerPreconditionBlockedError as error:
        raise RunnerFrozenInputMutationError(
            f"frozen file can no longer be verified: {file_identity['path']}: {error}"
        ) from error
    expected_file_identity = {key: file_identity[key] for key in current_identity}
    if dict(current_identity) != expected_file_identity:
        raise RunnerFrozenInputMutationError(f"frozen file identity changed: {file_identity['path']}")


def capture_optional_frozen_file_identity(file_path: Path) -> Mapping[str, Any]:
    file_path = Path(file_path).absolute()
    if not file_path.exists() and not file_path.is_symlink():
        return {"path": str(file_path), "exists": False}
    return {"exists": True, **capture_frozen_file_identity(file_path)}


def verify_optional_frozen_file_identity(file_identity: Mapping[str, Any]) -> None:
    file_path = Path(file_identity["path"]).absolute()
    current_path = Path(file_path.anchor)
    for component in file_path.parts[1:-1]:
        current_path /= component
        if current_path.is_symlink():
            raise RunnerFrozenInputMutationError(
                f"optional frozen file ancestor became a symlink: {current_path}"
            )
    try:
        current = capture_optional_frozen_file_identity(file_path)
    except RunnerPreconditionBlockedError as error:
        raise RunnerFrozenInputMutationError(
            f"optional frozen file can no longer be verified: {file_identity['path']}: {error}"
        ) from error
    if dict(current) != dict(file_identity):
        raise RunnerFrozenInputMutationError(f"optional frozen file identity changed: {file_identity['path']}")


def run_read_only_git_command(project_root: Path, argument_values: Sequence[str]) -> str:
    command = ["git", "-C", str(project_root), *argument_values]
    completed = subprocess.run(command, text=True, capture_output=True, check=False)
    if completed.returncode != 0:
        raise RunnerPreconditionBlockedError(
            f"git could not verify {project_root}: {completed.stderr.strip()}"
        )
    return completed.stdout


def path_is_excluded_from_snapshot(file_path: Path, excluded_paths: Sequence[Path]) -> bool:
    absolute_path = file_path.absolute()
    return any(absolute_path == excluded.absolute() or excluded.absolute() in absolute_path.parents
               for excluded in excluded_paths)


def build_owned_run_exclusions(report_run_directory: Path, artifact_run_directory: Path) -> list[Path]:
    return [
        Path(report_run_directory).resolve(strict=True),
        Path(artifact_run_directory).resolve(strict=True),
    ]


def capture_frozen_source_snapshot(
    project_root: Path,
    excluded_paths: Sequence[Path] = (),
    permitted_lock_paths: Sequence[Path] = (),
) -> Mapping[str, Any]:
    project_root = Path(project_root).resolve(strict=True)
    excluded_paths = tuple(Path(path).absolute() for path in excluded_paths)
    permitted_lock_relative_paths = set()
    for lock_path in permitted_lock_paths:
        lock_path = Path(lock_path).absolute()
        expected_lock_path = project_root / "Cargo.lock"
        if lock_path != expected_lock_path or lock_path.is_symlink() or not lock_path.is_file():
            raise RunnerPreconditionBlockedError(
                f"permitted frozen lock must be the regular top-level Cargo.lock: {lock_path}"
            )
        permitted_lock_relative_paths.add("Cargo.lock")
    lock_is_tracked = "Cargo.lock" in run_read_only_git_command(
        project_root, ["ls-files", "--", "Cargo.lock"]
    ).splitlines()
    status_text = run_read_only_git_command(project_root, ["status", "--porcelain=v1", "--untracked-files=all"])
    filtered_status_lines = []
    for status_line in status_text.splitlines():
        relative_path = status_line[3:].split(" -> ")[-1] if len(status_line) >= 4 else ""
        permitted_untracked_lock = (
            relative_path == "Cargo.lock" and relative_path in permitted_lock_relative_paths and not lock_is_tracked
        )
        if not permitted_untracked_lock and not path_is_excluded_from_snapshot(
            project_root / relative_path, excluded_paths
        ):
            filtered_status_lines.append(status_line)
    if filtered_status_lines:
        raise RunnerPreconditionBlockedError(
            f"source checkout is not clean: {project_root}: {filtered_status_lines[:5]}"
        )
    head_commit = run_read_only_git_command(project_root, ["rev-parse", "HEAD"]).strip()
    tracked_entries = run_read_only_git_command(project_root, ["ls-files", "--stage", "-z"]).encode()
    parsed_entries = []
    for encoded_entry in tracked_entries.split(b"\0"):
        if not encoded_entry:
            continue
        try:
            metadata_bytes, path_bytes = encoded_entry.split(b"\t", 1)
            mode_text, object_id, _stage = metadata_bytes.decode().split(" ")
            relative_path = Path(os.fsdecode(path_bytes))
        except (ValueError, UnicodeDecodeError) as error:
            raise RunnerPreconditionBlockedError(f"tracked source index is malformed: {error}") from error
        absolute_path = project_root / relative_path
        if not path_is_excluded_from_snapshot(absolute_path, excluded_paths):
            parsed_entries.append((mode_text, object_id, relative_path, absolute_path))
    tracked_relative_paths = {relative_path.as_posix() for _, _, relative_path, _ in parsed_entries}
    tracked_records = []
    for mode_text, object_id, relative_path, absolute_path in parsed_entries:
        if mode_text == "160000":
            raise RunnerPreconditionBlockedError(
                f"source submodule closure cannot be frozen: {relative_path.as_posix()}"
            )
        identity = capture_frozen_file_identity(absolute_path)
        if identity["is_symlink"]:
            resolved_path = Path(identity["symlink_resolved_path"])
            try:
                resolved_relative_path = resolved_path.relative_to(project_root).as_posix()
            except ValueError as error:
                raise RunnerPreconditionBlockedError(
                    f"tracked symlink target escapes source checkout: {relative_path.as_posix()}"
                ) from error
            if resolved_relative_path not in tracked_relative_paths:
                raise RunnerPreconditionBlockedError(
                    f"tracked symlink target is not a tracked input: {relative_path.as_posix()}"
                )
        tracked_records.append({
            "path": relative_path.as_posix(), "git_mode": mode_text, "git_object": object_id,
            "sha256": identity["sha256"], "size_bytes": identity["size_bytes"],
            "is_symlink": identity["is_symlink"], "symlink_target": identity["symlink_target"],
            "symlink_target_sha256": identity.get("symlink_target_sha256"),
            "symlink_resolved_path": identity.get("symlink_resolved_path"),
        })
    untracked_text = run_read_only_git_command(project_root, ["ls-files", "--others", "--exclude-standard", "-z"])
    untracked_source_paths = []
    untracked_relative_paths = set()
    for relative in untracked_text.split("\0"):
        if not relative:
            continue
        relative_path = Path(relative)
        untracked_relative_paths.add(relative_path.as_posix())
        absolute_path = project_root / relative_path
        if path_is_excluded_from_snapshot(absolute_path, excluded_paths):
            continue
        if relative_path.as_posix() in permitted_lock_relative_paths:
            continue
        if relative_path.suffix.lower() in SOURCE_SUFFIXES or relative_path.name in {
            "Cargo.toml", "Cargo.lock", "Makefile", "Justfile",
        }:
            untracked_source_paths.append(relative_path.as_posix())
    if untracked_source_paths:
        raise RunnerPreconditionBlockedError(
            f"untracked source inputs found in {project_root}: {untracked_source_paths[:5]}"
        )
    ignored_text = run_read_only_git_command(
        project_root, ["ls-files", "--others", "--ignored", "--exclude-standard", "-z"]
    )
    ignored_source_paths = []
    ignored_relative_paths = set()
    for relative in ignored_text.split("\0"):
        if not relative:
            continue
        relative_path = Path(relative)
        ignored_relative_paths.add(relative_path.as_posix())
        if any(part in GENERATED_SOURCE_DIRECTORY_NAMES for part in relative_path.parts):
            continue
        if path_is_excluded_from_snapshot(project_root / relative_path, excluded_paths):
            continue
        if relative_path.as_posix() in permitted_lock_relative_paths:
            continue
        under_vendor_or_build = any(part in {"vendor", "build"} for part in relative_path.parts)
        if under_vendor_or_build or relative_path.suffix.lower() in SOURCE_SUFFIXES or relative_path.name in {
            "Cargo.toml", "Cargo.lock", "Makefile", "Justfile",
        }:
            ignored_source_paths.append(relative_path.as_posix())
    if ignored_source_paths:
        raise RunnerPreconditionBlockedError(
            f"ignored untracked source inputs found in {project_root}: {ignored_source_paths[:5]}"
        )
    tracked_records.sort(key=lambda row: row["path"])
    permitted_lock_records = {}
    for relative_path in sorted(permitted_lock_relative_paths):
        if relative_path in tracked_relative_paths:
            provenance = "tracked"
        elif relative_path in ignored_relative_paths:
            provenance = "ignored"
        elif relative_path in untracked_relative_paths:
            provenance = "untracked"
        else:
            raise RunnerPreconditionBlockedError(
                f"selected Cargo.lock Git provenance is unavailable: {project_root / relative_path}"
            )
        permitted_lock_records[relative_path] = {
            "provenance": provenance,
            "identity": capture_frozen_file_identity(project_root / relative_path),
        }
    tree_digest = hashlib.sha256(
        json.dumps(
            {"tracked_files": tracked_records, "permitted_lock_files": permitted_lock_records},
            sort_keys=True, separators=(",", ":"),
        ).encode()
    ).hexdigest()
    return {
        "root_path": str(project_root),
        "head_commit": head_commit,
        "tracked_source_file_count": len(tracked_records),
        "permitted_lock_files": permitted_lock_records,
        "source_tree_sha256": tree_digest,
        "clean": True,
    }


def validate_repository_pin_manifest(
    project_root: Path,
    corpus_manifest_path: Path | None = None,
    parser_manifest_path: Path | None = None,
    oracle_manifest_path: Path | None = None,
    patch_file_path: Path | None = None,
) -> Mapping[str, Any]:
    phase_root = Path(project_root) / "docs/phase-20261111"
    corpus_manifest_path = corpus_manifest_path or phase_root / "corpus-source-pin-manifest.json"
    parser_manifest_path = parser_manifest_path or phase_root / "parser-source-pin-manifest.json"
    oracle_manifest_path = oracle_manifest_path or phase_root / "oracle-source-build-manifest.json"
    try:
        corpus_records = json.loads(Path(corpus_manifest_path).read_text())
        parser_record = json.loads(Path(parser_manifest_path).read_text())
        oracle_record = json.loads(Path(oracle_manifest_path).read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"pin manifest is unavailable or malformed: {error}") from error
    if not isinstance(corpus_records, list):
        raise RunnerPreconditionBlockedError("corpus pin manifest must be a list")
    corpus_records_by_name = {}
    for record in corpus_records:
        if not isinstance(record, dict) or not isinstance(record.get("corpus_repository_short_name"), str):
            raise RunnerPreconditionBlockedError("corpus pin manifest contains a malformed entry")
        name = record["corpus_repository_short_name"]
        if name in corpus_records_by_name:
            raise RunnerContractViolationError(f"duplicate corpus pin for {name}")
        corpus_records_by_name[name] = record
    rust_analyzer_record = corpus_records_by_name.get("rust-analyzer")
    if not rust_analyzer_record:
        raise RunnerPreconditionBlockedError("rust-analyzer corpus pin is missing")
    corpus_commit = rust_analyzer_record.get("corpus_source_commit_hash")
    parser_commit = parser_record.get("parser_source_commit_hash")
    oracle_commit = oracle_record.get("oracle_source_commit_hash")
    if (corpus_commit != RUST_ANALYZER_COMMIT or parser_commit != RUST_ANALYZER_COMMIT
            or oracle_commit != RUST_ANALYZER_COMMIT):
        raise RunnerContractViolationError("R1 corpus/parser/oracle commit pins do not match the approved RA commit")
    if parser_record.get("parser_source_patch_digest") != R1_PATCH_SHA256:
        raise RunnerContractViolationError("parser pin manifest does not carry the R1 patch digest")
    corpus_upstream = rust_analyzer_record.get("corpus_source_upstream_hash")
    if (parser_record.get("parser_source_upstream_hash") != corpus_upstream
            or oracle_record.get("oracle_source_upstream_hash") != corpus_upstream):
        raise RunnerContractViolationError("R1 corpus/parser/oracle upstream pins do not agree")
    if oracle_record.get("oracle_source_patch_path") != parser_record.get("parser_source_patch_path"):
        raise RunnerContractViolationError("oracle and parser pins do not identify the same R1 patch")
    resolved_patch_path = Path(patch_file_path) if patch_file_path else Path(project_root) / parser_record.get(
        "parser_source_patch_path", ""
    )
    if not resolved_patch_path.is_file():
        raise RunnerPreconditionBlockedError("R1 parser patch file is missing")
    patch_sha256 = compute_sha256_file_digest(resolved_patch_path)
    if patch_sha256 != R1_PATCH_SHA256:
        raise RunnerContractViolationError("R1 parser patch bytes differ from the pinned SHA-256")
    return {
        "corpus_commit_by_repository": {
            name: record.get("corpus_source_commit_hash") for name, record in corpus_records_by_name.items()
        },
        "rust_analyzer_commit": corpus_commit,
        "parser_commit": parser_commit,
        "oracle_commit": oracle_commit,
        "r1_patch_path": str(resolved_patch_path.resolve()),
        "r1_patch_sha256": patch_sha256,
        "corpus_order_in_manifest": list(corpus_records_by_name),
    }


def validate_coverage_page_sequence(
    scope_pages: Sequence[Mapping[str, Any]],
    missed_pages: Sequence[Mapping[str, Any]],
) -> Mapping[str, Any]:
    try:
        scope_result = coverage_inventory.validate_scope_response_pages(scope_pages)
        bound_missed_pages = []
        for missed_page in missed_pages:
            page_value = dict(missed_page)
            page_value.setdefault("index_generation", page_value.get("indexed_at", scope_result["generation"]))
            bound_missed_pages.append(page_value)
        missed_result = coverage_inventory.validate_missed_response_pages(
            bound_missed_pages, scope_result["generation"]
        )
    except coverage_inventory.CoverageInventoryBlockedError as error:
        raise RunnerPreconditionBlockedError(str(error)) from error
    except (KeyError, TypeError, ValueError) as error:
        raise RunnerPreconditionBlockedError(f"coverage response is malformed: {error}") from error
    return {
        "generation": scope_result["generation"],
        "scope_entry_count": scope_result["entry_count"],
        "missed_file_count": missed_result["row_count"],
        "status": "PASS",
    }


def require_evidence_identity_match(
    evidence: Mapping[str, Any], expected_identity: Mapping[str, Any], evidence_name: str
) -> None:
    for identity_field in ("run_id", "repository_name", "mapper_sha256", "input_digest"):
        if identity_field not in expected_identity:
            continue
        if identity_field not in evidence:
            raise RunnerPreconditionBlockedError(f"{evidence_name} is missing identity field {identity_field}")
        if evidence[identity_field] != expected_identity[identity_field]:
            raise RunnerPreconditionBlockedError(f"{evidence_name} identity mismatch for {identity_field}")


def validate_blake3_hex_digest(digest_value: Any, evidence_label: str) -> None:
    if not isinstance(digest_value, str) or not re.fullmatch(r"[0-9a-f]{64}", digest_value):
        raise RunnerPreconditionBlockedError(f"{evidence_label} must be a lowercase 64-character BLAKE3 digest")


def validate_m2_duckdb_handoff(
    evidence: Mapping[str, Any], expected_identity: Mapping[str, Any]
) -> Mapping[str, Any]:
    if not isinstance(evidence, Mapping):
        raise RunnerPreconditionBlockedError("R4 evidence must be a JSON object")
    require_evidence_identity_match(evidence, expected_identity, "R4 evidence")
    if "published" not in evidence or "cancelled" not in evidence:
        raise RunnerPreconditionBlockedError("R4 publication/cancellation fields are missing")
    if evidence["published"] is not True or evidence["cancelled"] is not False:
        raise RunnerContractViolationError("R4 source generation is not published or is cancelled")
    source_generation = evidence.get("source_generation")
    duckdb_generation = evidence.get("duckdb_generation")
    if not isinstance(source_generation, str) or not source_generation or not isinstance(duckdb_generation, str) or not duckdb_generation:
        raise RunnerPreconditionBlockedError("R4 source generation identity is missing")
    if source_generation != duckdb_generation:
        raise RunnerContractViolationError("R4 DuckDB generation does not match the published source generation")
    tables = evidence.get("tables")
    if not isinstance(tables, list) or not tables:
        raise RunnerPreconditionBlockedError("R4 per-table copy evidence is missing")
    seen_tables = set()
    for table in tables:
        if not isinstance(table, Mapping):
            raise RunnerPreconditionBlockedError("R4 table record is malformed")
        name = table.get("table_name")
        source_count = table.get("source_row_count")
        duckdb_count = table.get("duckdb_row_count")
        source_digest = table.get("source_content_blake3")
        duckdb_digest = table.get("duckdb_content_blake3")
        if not isinstance(name, str) or not name or name in seen_tables:
            raise RunnerPreconditionBlockedError("R4 table names are missing or duplicated")
        seen_tables.add(name)
        if (not isinstance(source_count, int) or isinstance(source_count, bool) or source_count < 0
                or not isinstance(duckdb_count, int) or isinstance(duckdb_count, bool) or duckdb_count < 0):
            raise RunnerPreconditionBlockedError(f"R4 row counts are missing for table {name}")
        validate_blake3_hex_digest(source_digest, f"R4 source digest for {name}")
        validate_blake3_hex_digest(duckdb_digest, f"R4 DuckDB digest for {name}")
        if source_count != duckdb_count or source_digest != duckdb_digest:
            raise RunnerContractViolationError(f"R4 copied table counts or digests differ for {name}")
    settings = evidence.get("duckdb_settings")
    if not isinstance(settings, Mapping):
        raise RunnerPreconditionBlockedError("effective DuckDB settings are missing")
    required_settings = (
        "autoload_known_extensions", "autoinstall_known_extensions", "enable_external_access",
        "memory_limit", "temp_directory",
    )
    missing_settings = [setting for setting in required_settings if setting not in settings]
    if missing_settings:
        raise RunnerPreconditionBlockedError(f"effective DuckDB settings are missing: {missing_settings}")
    for setting in ("autoload_known_extensions", "autoinstall_known_extensions", "enable_external_access"):
        if settings[setting] is not False:
            raise RunnerContractViolationError(f"DuckDB setting {setting} must be false")
    if not isinstance(settings["memory_limit"], str) or not settings["memory_limit"].strip():
        raise RunnerPreconditionBlockedError("effective DuckDB memory_limit was not captured")
    temp_directory = settings["temp_directory"]
    artifact_root = expected_identity.get("artifact_root")
    if not isinstance(temp_directory, str) or not temp_directory or not artifact_root:
        raise RunnerPreconditionBlockedError("run-local DuckDB temp directory is not bound to an artifact root")
    temp_path = Path(temp_directory).resolve()
    artifact_path = Path(artifact_root).resolve()
    if temp_path != artifact_path and artifact_path not in temp_path.parents:
        raise RunnerContractViolationError("DuckDB temp directory is outside the run-local artifact root")
    if not temp_path.is_dir():
        raise RunnerPreconditionBlockedError("run-local DuckDB temp directory does not exist")
    return {
        "status": "PASS",
        "source_generation": source_generation,
        "table_count": len(tables),
        "effective_memory_limit": settings["memory_limit"],
        "memory_acceptance_status": "DEFERRED",
        "temp_directory": str(temp_path),
    }


def validate_m2_path_partition(
    evidence: Mapping[str, Any], expected_identity: Mapping[str, Any]
) -> Mapping[str, Any]:
    if not isinstance(evidence, Mapping):
        raise RunnerPreconditionBlockedError("R5 evidence must be a JSON object")
    require_evidence_identity_match(evidence, expected_identity, "R5 evidence")
    rows = evidence.get("rows")
    raw_count = evidence.get("raw_oracle_row_count")
    if (not isinstance(rows, list) or not isinstance(raw_count, int)
            or isinstance(raw_count, bool) or raw_count < 0):
        raise RunnerPreconditionBlockedError("R5 raw rows or denominator is missing")
    if any(not isinstance(row, Mapping) for row in rows):
        raise RunnerPreconditionBlockedError("R5 classified row is malformed")
    row_ids = [row.get("oracle_row_id") for row in rows]
    if any(not isinstance(row_id, str) or not row_id for row_id in row_ids):
        raise RunnerPreconditionBlockedError("R5 oracle row identity is missing or malformed")
    if len(set(row_ids)) != len(row_ids):
        raise RunnerContractViolationError("R5 oracle row is duplicated across partition classes")
    if len(rows) != raw_count:
        raise RunnerContractViolationError("R5 raw oracle count does not equal classified row count")
    primary_digest = evidence.get("primary_export_blake3")
    validate_blake3_hex_digest(primary_digest, "R5 primary export digest")
    if expected_identity.get("primary_export_blake3") is not None and primary_digest != expected_identity["primary_export_blake3"]:
        raise RunnerPreconditionBlockedError("R5 evidence is bound to a different primary export")
    classes = evidence.get("partition_classes")
    if not isinstance(classes, list) or not classes:
        raise RunnerPreconditionBlockedError("R5 exhaustive partition classes are missing")
    class_names = set()
    declared_total = 0
    for partition_class in classes:
        if not isinstance(partition_class, Mapping):
            raise RunnerPreconditionBlockedError("R5 partition class is malformed")
        class_name = partition_class.get("class_name")
        class_count = partition_class.get("row_count")
        if not isinstance(class_name, str) or not class_name or class_name in class_names:
            raise RunnerPreconditionBlockedError("R5 class names are missing or duplicated")
        if not isinstance(class_count, int) or isinstance(class_count, bool) or class_count < 0:
            raise RunnerPreconditionBlockedError(f"R5 row count is missing for {class_name}")
        if not isinstance(partition_class.get("reason"), str) or not partition_class["reason"].strip():
            raise RunnerPreconditionBlockedError(f"R5 reason is missing for {class_name}")
        if not isinstance(partition_class.get("stage_ownership"), str) or not partition_class["stage_ownership"].strip():
            raise RunnerPreconditionBlockedError(f"R5 stage ownership is missing for {class_name}")
        class_names.add(class_name)
        declared_total += class_count
    observed_counts = {class_name: 0 for class_name in class_names}
    for row in rows:
        class_name = row.get("class_name")
        if not isinstance(class_name, str) or not class_name:
            raise RunnerPreconditionBlockedError("R5 row partition class is missing or malformed")
        if class_name not in class_names:
            raise RunnerContractViolationError("R5 row belongs to no declared partition class")
        observed_counts[class_name] += 1
    declared_counts = {row["class_name"]: row["row_count"] for row in classes}
    if observed_counts != declared_counts or declared_total != raw_count:
        raise RunnerContractViolationError("R5 exhaustive disjoint class counts do not equal the raw denominator")
    return {
        "status": "PASS",
        "raw_oracle_row_count": raw_count,
        "partition_class_count": len(classes),
        "partition_counts": observed_counts,
    }


def resolve_canonical_executable_path(executable_value: str | Path) -> Path:
    candidate = Path(executable_value).expanduser()
    if candidate.parent == Path("."):
        discovered = shutil.which(str(executable_value))
        if discovered:
            candidate = Path(discovered)
    try:
        return candidate.resolve(strict=True)
    except FileNotFoundError as error:
        raise RunnerPreconditionBlockedError(f"executable is missing: {executable_value}") from error


def validate_canonical_runner_executable(executable_path: Path) -> Mapping[str, Any]:
    canonical_path = resolve_canonical_executable_path(executable_path)
    if not canonical_path.is_file() or not os.access(canonical_path, os.X_OK):
        raise RunnerPreconditionBlockedError(f"canonical mapper is not an executable file: {canonical_path}")
    identity = dict(capture_frozen_file_identity(canonical_path))
    identity["canonical_path"] = str(canonical_path)
    identity["sha256"] = compute_sha256_file_digest(canonical_path)
    return identity


def find_external_b3_digest_provider() -> str | None:
    try:
        import blake3  # type: ignore[import-not-found]

        return f"python-blake3:{getattr(blake3, '__version__', 'available')}"
    except ImportError:
        provider_path = shutil.which("b3sum")
        return str(Path(provider_path).resolve()) if provider_path else None


def require_external_b3_digest_provider(harness_only_flag: bool = False) -> str | None:
    provider = find_external_b3_digest_provider()
    if provider is None and not harness_only_flag:
        raise RunnerPreconditionBlockedError("large artifact BLAKE3 provider is unavailable")
    return provider


def calculate_b3_file_digest(
    file_path: Path,
    provider: str,
    canonical_executable_identity: Mapping[str, Any] | None = None,
    identity_guard: Callable[[str, Sequence[str]], None] | None = None,
    provider_identity: Mapping[str, Any] | None = None,
) -> str:
    if canonical_executable_identity is not None:
        verify_frozen_file_identity(canonical_executable_identity)
    if provider_identity is not None:
        verify_frozen_file_identity(provider_identity)
    if identity_guard is not None:
        identity_guard("before", [provider, str(file_path)])
    if provider.startswith("python-blake3:"):
        try:
            import blake3
        except ImportError as error:
            raise RunnerPreconditionBlockedError("Python BLAKE3 provider disappeared during the run") from error
        digest = blake3.blake3()
        with file_path.open("rb") as input_file:
            for content_chunk in iter(lambda: input_file.read(1024 * 1024), b""):
                digest.update(content_chunk)
        digest_value = digest.hexdigest()
    elif canonical_executable_identity is not None:
        result = run_guarded_child_process(
            [provider, str(file_path)], canonical_executable_identity, identity_guard
        )
        if result["return_code"] != 0:
            raise RunnerPreconditionBlockedError(f"b3sum failed for {file_path}: {result['stderr'].strip()}")
        digest_value = result["stdout"].split()[0] if result["stdout"].split() else ""
    else:
        completed = subprocess.run([provider, str(file_path)], text=True, capture_output=True, check=False)
        if completed.returncode != 0:
            raise RunnerPreconditionBlockedError(f"b3sum failed for {file_path}: {completed.stderr.strip()}")
        digest_value = completed.stdout.split()[0] if completed.stdout.split() else ""
    if identity_guard is not None:
        identity_guard("after", [provider, str(file_path)])
    if canonical_executable_identity is not None:
        verify_frozen_file_identity(canonical_executable_identity)
    if provider_identity is not None:
        verify_frozen_file_identity(provider_identity)
    validate_blake3_hex_digest(digest_value, f"BLAKE3 digest for {file_path}")
    return digest_value


def classify_memory_acceptance_measurement(
    sampled_peak_bytes: int,
    all_five_s5_complete_flag: bool = False,
) -> Mapping[str, Any]:
    if not isinstance(sampled_peak_bytes, int) or sampled_peak_bytes < 0:
        raise ValueError("sampled RSS must be a non-negative integer")
    if not all_five_s5_complete_flag:
        acceptance_status = "DEFERRED"
    else:
        acceptance_status = "PASS" if sampled_peak_bytes <= 500_000_000 else "FAIL"
    return {
        "sampled_peak_bytes": sampled_peak_bytes,
        "acceptance_status": acceptance_status,
        "acceptance_gate_enabled": all_five_s5_complete_flag,
        "target_bytes": 500_000_000,
        "measurement_only_until_all_five_s5": not all_five_s5_complete_flag,
    }


def audit_production_process_trace(
    trace_source_text: str,
    canonical_executable_path: Path,
) -> Mapping[str, Any]:
    if not trace_source_text.strip():
        return {"status": "BLOCKED", "reason": "process trace is empty", "violations": []}
    complete_events, pending_events = process_auditor.read_complete_trace_events(trace_source_text)
    if not complete_events or pending_events:
        return {"status": "BLOCKED", "reason": "process trace is incomplete", "violations": []}
    canonical_path = str(Path(canonical_executable_path).resolve())
    root_pattern = re.compile(r"^(?:\[pid\s+)?(\d+)\]?\s+execve\(" + re.escape(json.dumps(canonical_path)) + r",")
    root_match = next((root_pattern.match(line) for line in trace_source_text.splitlines()
                       if root_pattern.match(line)), None)
    if root_match is None:
        return {"status": "BLOCKED", "reason": "canonical mapper root exec is missing from trace", "violations": []}
    violations = process_auditor.audit_process_trace_events(
        trace_source_text, int(root_match[1]), Path(canonical_path)
    )
    if any("incomplete trace" in violation or "unfinished trace" in violation for violation in violations):
        return {"status": "BLOCKED", "reason": "process trace did not record complete execution", "violations": violations}
    return {
        "status": "FAIL" if violations else "PASS",
        "reason": "forbidden process behavior observed" if violations else None,
        "violations": violations,
    }


def run_guarded_child_process(
    command_values: Sequence[str],
    canonical_executable_identity: Mapping[str, Any],
    identity_guard: Callable[[str, Sequence[str]], None] | None = None,
    harness_only_flag: bool = False,
) -> Mapping[str, Any]:
    if not command_values or not all(isinstance(value, str) for value in command_values):
        raise ValueError("subprocess command must be a nonempty sequence of strings")
    executable_identity = dict(canonical_executable_identity)
    command_list = list(command_values)
    verify_frozen_file_identity(executable_identity)
    if identity_guard is not None:
        identity_guard("before", command_list)
    try:
        completed = subprocess.run(command_list, capture_output=True, check=False)
    finally:
        if identity_guard is not None:
            identity_guard("after", command_list)
        verify_frozen_file_identity(executable_identity)
    stdout_bytes = completed.stdout if isinstance(completed.stdout, bytes) else completed.stdout.encode()
    stderr_bytes = completed.stderr if isinstance(completed.stderr, bytes) else completed.stderr.encode()
    return {
        "command": command_list,
        "return_code": completed.returncode,
        "stdout": stdout_bytes.decode("utf-8", errors="replace"),
        "stderr": stderr_bytes.decode("utf-8", errors="replace"),
        "stdout_bytes": stdout_bytes,
        "stderr_bytes": stderr_bytes,
        "qualification_status": "HARNESS ONLY" if harness_only_flag else "COMMAND COMPLETED",
    }


def collect_dependency_tree_reference(
    evidence_inputs: Mapping[str, Any], expected_identity: Mapping[str, Any],
    registry: Mapping[str, Any], identity_guard: Callable[[str, Sequence[str]], None],
    artifact_run_directory: Path, b3_provider: str | None, b3_tool_identity=None,
):
    manifest_document = evidence_inputs.get("manifest_document")
    if not isinstance(manifest_document, Mapping):
        return None
    global_prerequisites = manifest_document.get("global_prerequisites")
    dependency_references = (
        global_prerequisites.get("dependency_audit") if isinstance(global_prerequisites, Mapping) else None
    )
    if not isinstance(dependency_references, list) or not dependency_references:
        return None
    configuration = expected_identity.get("configuration")
    toolchain_versions = expected_identity.get("toolchain_versions")
    cargo_record = toolchain_versions.get("cargo") if isinstance(toolchain_versions, Mapping) else None
    cargo_identity = expected_identity.get("toolchain_executables", {}).get("cargo")
    if (not isinstance(configuration, Mapping) or configuration.get("p2_configuration_blocked_reason")
            or configuration.get("unsupported_prerequisite_gates")
            or not isinstance(cargo_record, Mapping) or cargo_record.get("version") in {None, "", "unavailable"}
            or not isinstance(cargo_record.get("path"), str) or not isinstance(cargo_identity, Mapping)
            or not isinstance(b3_provider, str)):
        return None
    package_name = configuration.get("production_package_name")
    project_root = configuration.get("project_root")
    if not isinstance(package_name, str) or not isinstance(project_root, str):
        return None
    command = [
        cargo_record["path"], "tree", "--package", package_name, "--edges", "normal", "--prefix", "none",
        "--format", "{p}", "--offline", "--locked",
    ]
    cwd = Path(project_root).resolve(strict=True)
    if cwd != PROJECT_ROOT.resolve():
        return None
    artifact_run_directory = Path(artifact_run_directory)
    stdout_path = artifact_run_directory / "s3-dependency-tree.stdout"
    stderr_path = artifact_run_directory / "s3-dependency-tree.stderr"
    observation_path = artifact_run_directory / "s3-dependency-tree-observation.json"
    observation = {
        "argv": command, "cwd": str(cwd),
        "relevant_environment": dict(expected_identity.get("relevant_environment", {})),
        "timeout_seconds": DEPENDENCY_TREE_TIMEOUT_SECONDS,
        "timed_out": False, "completion_status": "not_started", "exit_status": None,
        "streams": {}, "forbidden_runtime_rows": [],
    }
    process = None
    guard_error = None
    command_error = None
    mutation_error = None
    observation_error = None
    output_collision_error = None
    timed_out = False
    reaped = False
    attempted = False
    stream_records = {}
    stream_capture_errors = {}
    forbidden_runtime_rows = []
    hash_error = None
    output_identities = registry.setdefault("runner_owned_output_identities", {})
    collection_started = time.monotonic()
    command_started = None
    command_finished = None
    stdout_path.parent.mkdir(parents=True, exist_ok=True)

    def verify_open_output_path(output_file, output_path, expected_device=None, expected_inode=None):
        descriptor_stat = os.fstat(output_file.fileno())
        try:
            path_stat = output_path.lstat()
        except OSError as error:
            raise RunnerFrozenInputMutationError(
                f"runner-owned dependency {output_path.name} path changed during capture"
            ) from error
        if (not stat.S_ISREG(path_stat.st_mode) or path_stat.st_dev != descriptor_stat.st_dev
                or path_stat.st_ino != descriptor_stat.st_ino
                or (expected_device is not None and descriptor_stat.st_dev != expected_device)
                or (expected_inode is not None and descriptor_stat.st_ino != expected_inode)):
            raise RunnerFrozenInputMutationError(
                f"runner-owned dependency {output_path.name} path no longer names its open output file"
            )
        return descriptor_stat.st_dev, descriptor_stat.st_ino

    def capture_stream(name, output_file, output_path):
        nonlocal forbidden_runtime_rows, hash_error, mutation_error
        complete = process is not None and reaped and not timed_out
        opened_device = None
        opened_inode = None
        raw_bytes = None
        try:
            output_file.flush()
            os.fsync(output_file.fileno())
            output_file.seek(0)
            raw_bytes = output_file.read()
            sha256 = hashlib.sha256(raw_bytes).hexdigest()
            observation["streams"][name] = {
                "path": str(output_path), "size_bytes": len(raw_bytes), "sha256": sha256,
                "complete": complete,
            }
            if name == "stdout":
                forbidden_runtime_rows = sorted(set(
                    s3_evidence_intake._forbidden_dependency_tree_names(raw_bytes)
                ))
                observation["forbidden_runtime_rows"] = forbidden_runtime_rows
            opened_device, opened_inode = verify_open_output_path(output_file, output_path)
            verify_open_output_path(output_file, output_path, opened_device, opened_inode)
            file_identity = capture_frozen_file_identity(output_path)
            verify_open_output_path(output_file, output_path, opened_device, opened_inode)
            if (file_identity["path"] != str(output_path.resolve(strict=True))
                    or file_identity["size_bytes"] != len(raw_bytes)
                    or file_identity["sha256"] != sha256 or file_identity["is_symlink"] is not False):
                raise RunnerFrozenInputMutationError(
                    f"runner-owned dependency {name} changed during open-file capture"
                )
            output_identities[file_identity["path"]] = dict(file_identity)
            blake3 = calculate_b3_file_digest(
                output_path, b3_provider, file_identity, identity_guard, b3_tool_identity,
            )
            verify_open_output_path(output_file, output_path, opened_device, opened_inode)
            stream_record = {
                "path": file_identity["path"], "size_bytes": len(raw_bytes), "sha256": sha256,
                "blake3": blake3, "complete": complete, "raw_bytes": raw_bytes,
                "file_identity": file_identity,
            }
            stream_records[name] = stream_record
            observation["streams"][name]["blake3"] = blake3
        except RunnerFrozenInputMutationError as error:
            mutation_error = mutation_error or error
            stream_capture_errors[name] = str(error)
        except (OSError, RunnerPreconditionBlockedError, RunnerContractViolationError) as error:
            hash_error = hash_error or str(error)
            stream_capture_errors[name] = str(error)
            if opened_device is not None:
                try:
                    verify_open_output_path(output_file, output_path, opened_device, opened_inode)
                except RunnerFrozenInputMutationError as mutation:
                    mutation_error = mutation_error or mutation
                    stream_capture_errors[name] = str(mutation)
        if raw_bytes is None:
            observation["streams"].setdefault(name, {
                "path": str(output_path), "size_bytes": None, "sha256": None, "complete": False,
            })

    try:
        with stdout_path.open("x+b") as stdout_file, stderr_path.open("x+b") as stderr_file:
            try:
                verify_frozen_file_identity(cargo_identity)
                child_environment = os.environ.copy()
                if capture_s3_evidence_environment() != observation["relevant_environment"]:
                    raise RunnerFrozenInputMutationError(
                        "relevant environment changed before S3 dependency tree collection"
                    )
                identity_guard("before S3 dependency tree collection", command)
                attempted = True
                command_started = time.monotonic()
                try:
                    try:
                        process = subprocess.Popen(
                            command, cwd=str(cwd), env=child_environment, stdin=subprocess.DEVNULL,
                            stdout=stdout_file, stderr=stderr_file, start_new_session=True,
                        )
                    except (OSError, subprocess.SubprocessError) as error:
                        command_error = str(error)
                    if process is not None:
                        try:
                            process.wait(timeout=DEPENDENCY_TREE_TIMEOUT_SECONDS)
                            reaped = True
                        except subprocess.TimeoutExpired:
                            try:
                                os.killpg(process.pid, signal.SIGKILL)
                            except ProcessLookupError:
                                pass
                            except OSError as error:
                                command_error = command_error or f"cannot kill dependency process group: {error}"
                            timed_out = True
                            try:
                                process.wait(timeout=DEPENDENCY_TREE_CLEANUP_TIMEOUT_SECONDS)
                                reaped = True
                            except subprocess.TimeoutExpired:
                                reaped = False
                                try:
                                    os.killpg(process.pid, signal.SIGKILL)
                                except OSError:
                                    pass
                        except (OSError, subprocess.SubprocessError) as error:
                            command_error = command_error or str(error)
                finally:
                    command_finished = time.monotonic()
            except RunnerFrozenInputMutationError as error:
                guard_error = error
            except (OSError, subprocess.SubprocessError) as error:
                command_error = str(error)
            capture_stream("stdout", stdout_file, stdout_path)
            capture_stream("stderr", stderr_file, stderr_path)
    except FileExistsError as error:
        output_collision_error = RunnerPreconditionBlockedError(
            f"refusing to reuse dependency tree output: {error}"
        )
    except (OSError, subprocess.SubprocessError) as error:
        command_error = command_error or str(error)
    finally:
        elapsed_seconds = (
            command_finished - command_started
            if command_started is not None and command_finished is not None else None
        )
        if attempted:
            try:
                identity_guard("after S3 dependency tree collection", command)
            except Exception as error:
                guard_error = guard_error or error
        collection_elapsed_seconds = time.monotonic() - collection_started
        observation.update({
            "elapsed_seconds": elapsed_seconds,
            "collection_elapsed_seconds": collection_elapsed_seconds,
            "timed_out": timed_out,
            "completion_status": (
                "timed_out" if timed_out else "completed" if process is not None and reaped else
                "guard_failed" if guard_error is not None else "command_failed" if command_error else "not_started"
            ),
            "exit_status": process.returncode if process is not None else None,
            "process_reaped": reaped,
            "command_error": command_error,
            "guard_error": str(guard_error) if guard_error else None,
            "stream_capture_errors": stream_capture_errors,
            "forbidden_runtime_rows": forbidden_runtime_rows,
            "streams": {name: {key: value for key, value in record.items() if key != "raw_bytes"}
                        for name, record in stream_records.items()} | {
                            name: value for name, value in observation["streams"].items()
                            if name not in stream_records
                        },
        })
        try:
            write_json_atomically(observation_path, observation)
        except OSError as error:
            observation_error = str(error)
    if mutation_error is not None:
        raise mutation_error
    if forbidden_runtime_rows:
        raise RunnerContractViolationError(f"forbidden runtime dependency rows: {forbidden_runtime_rows}")
    if guard_error is not None:
        raise guard_error
    if output_collision_error is not None:
        raise output_collision_error
    if (process is None or not reaped or hash_error is not None or command_error is not None
            or observation_error is not None
            or set(stream_records) != {"stdout", "stderr"}):
        return None
    identity_digest = hashlib.sha256(json.dumps(
        expected_identity, sort_keys=True, separators=(",", ":"),
    ).encode()).hexdigest()
    try:
        return s3_evidence_intake._mint_runner_owned_dependency_tree_reference(
            identity_digest=identity_digest, argv=command, cwd=str(cwd),
            relevant_environment=observation["relevant_environment"],
            elapsed_seconds=elapsed_seconds, timeout_seconds=DEPENDENCY_TREE_TIMEOUT_SECONDS,
            timed_out=timed_out, completion_status=observation["completion_status"],
            exit_status=observation["exit_status"], stdout=stream_records["stdout"],
            stderr=stream_records["stderr"],
        )
    except s3_evidence_intake.S3EvidenceBlockedError:
        return None


def run_ordered_repository_sequence(
    repository_names: Sequence[str],
    repository_action: Callable[[str], str],
) -> Mapping[str, Any]:
    if len(set(repository_names)) != len(repository_names):
        raise ValueError("repository names must be unique")
    ordered_names = [name for name in DEFAULT_REPOSITORY_ORDER if name in repository_names]
    ordered_names.extend(name for name in repository_names if name not in DEFAULT_REPOSITORY_ORDER)
    repository_statuses = {name: "NOT STARTED" for name in ordered_names}
    attempted_repositories = []
    for repository_name in ordered_names:
        attempted_repositories.append(repository_name)
        status = repository_action(repository_name)
        if status not in {"PASS", "BLOCKED", "FAIL", "COMPONENTS COMPLETE"}:
            raise ValueError(f"repository action returned unsupported status {status!r}")
        repository_statuses[repository_name] = status
        if status not in {"PASS", "COMPONENTS COMPLETE"}:
            break
    return {
        "repository_statuses": repository_statuses,
        "attempted_repositories": attempted_repositories,
        "overall_status": (
            "COMPONENTS COMPLETE"
            if all(status in {"PASS", "COMPONENTS COMPLETE"} for status in repository_statuses.values())
            and any(status == "COMPONENTS COMPLETE" for status in repository_statuses.values())
            else "PASS" if all(status == "PASS" for status in repository_statuses.values())
            else (repository_statuses[attempted_repositories[-1]]
                  if attempted_repositories else "NOT STARTED")
        ),
    }


def build_runner_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--milestone", required=True, choices=("M1", "M2"))
    parser.add_argument("--mode", choices=("plan", "execute"), default="plan")
    parser.add_argument("--mapper-binary", type=Path)
    parser.add_argument("--corpus-manifest", type=Path,
                        default=PROJECT_ROOT / "docs/phase-20261111/corpus-source-pin-manifest.json")
    parser.add_argument("--report-root", type=Path, default=PROJECT_ROOT / "hld-current-test-runs")
    parser.add_argument("--artifact-root", type=Path, default=Path.home() / "hld-current-test-artifacts")
    parser.add_argument("--indexer-path", default="codebase-memory-mcp")
    parser.add_argument("--project-root", action="append", default=[], metavar="NAME=PATH")
    parser.add_argument("--harness-only", action="store_true")
    parser.add_argument("--r4-evidence", type=Path)
    parser.add_argument("--r5-evidence", type=Path)
    parser.add_argument("--s3-prerequisite-evidence", type=Path)
    return parser


def parse_project_root_overrides(override_values: Sequence[str]) -> Mapping[str, Path]:
    overrides = {}
    for override_value in override_values:
        name, separator, root_value = override_value.partition("=")
        if not separator or name not in DEFAULT_REPOSITORY_ORDER or not root_value or name in overrides:
            raise RunnerPreconditionBlockedError("--project-root must use unique NAME=PATH values for pinned repositories")
        overrides[name] = Path(root_value).expanduser().resolve(strict=True)
    return overrides


def parse_pin_manifest_records(manifest_path: Path) -> Mapping[str, Mapping[str, Any]]:
    try:
        records = json.loads(manifest_path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"corpus pin manifest is unavailable: {error}") from error
    if not isinstance(records, list):
        raise RunnerPreconditionBlockedError("corpus pin manifest must be a JSON list")
    result = {}
    for record in records:
        if not isinstance(record, dict):
            raise RunnerPreconditionBlockedError("corpus pin record is malformed")
        name = record.get("corpus_repository_short_name")
        if not isinstance(name, str) or name in result:
            raise RunnerContractViolationError("corpus pin names are missing or duplicated")
        result[name] = record
    if set(result) != set(DEFAULT_REPOSITORY_ORDER):
        raise RunnerPreconditionBlockedError("corpus pin manifest must contain exactly the five supported repositories")
    return result


def canonicalize_output_root_path(path_value: Path) -> Path:
    candidate = Path(path_value).expanduser()
    if not candidate.is_absolute():
        candidate = Path.cwd() / candidate
    current_path = Path(candidate.anchor)
    for component in candidate.parts[1:]:
        if component == "..":
            current_path = current_path.parent
            continue
        if component in {"", "."}:
            continue
        next_path = current_path / component
        if next_path.is_symlink():
            raise RunnerPreconditionBlockedError(f"output root contains a symlink component: {path_value}")
        current_path = next_path
    try:
        return candidate.resolve(strict=False)
    except (OSError, RuntimeError) as error:
        raise RunnerPreconditionBlockedError(f"output root cannot be canonicalized: {path_value}: {error}") from error


def source_output_paths_overlap(first_path: Path, second_path: Path) -> bool:
    return first_path == second_path or first_path in second_path.parents or second_path in first_path.parents


def validate_output_root_paths(
    report_root: Path,
    artifact_root: Path,
    project_root: Path,
    corpus_roots: Mapping[str, Path],
) -> Mapping[str, Path]:
    project_path = Path(project_root).resolve(strict=True)
    corpus_paths = {name: Path(path).resolve(strict=False) for name, path in corpus_roots.items()}
    report_path = canonicalize_output_root_path(report_root)
    artifact_path = canonicalize_output_root_path(artifact_root)
    default_report_path = project_path / "hld-current-test-runs"
    source_paths = [project_path, *corpus_paths.values()]
    if report_path == default_report_path:
        overlapping_corpora = [
            name for name, corpus_path in corpus_paths.items()
            if source_output_paths_overlap(report_path, corpus_path)
        ]
        if overlapping_corpora:
            raise RunnerPreconditionBlockedError(
                f"default report root overlaps pinned corpus sources: {overlapping_corpora}"
            )
    else:
        overlapping_sources = [
            str(source_path) for source_path in source_paths
            if source_output_paths_overlap(report_path, source_path)
        ]
        if overlapping_sources:
            raise RunnerPreconditionBlockedError(
                f"report root must be external or exactly {default_report_path}; overlaps {overlapping_sources}"
            )
    overlapping_artifact_sources = [
        str(source_path) for source_path in source_paths
        if source_output_paths_overlap(artifact_path, source_path)
    ]
    if overlapping_artifact_sources:
        raise RunnerPreconditionBlockedError(
            f"artifact root must be outside project and corpus sources; overlaps {overlapping_artifact_sources}"
        )
    if source_output_paths_overlap(report_path, artifact_path):
        raise RunnerPreconditionBlockedError("report and artifact roots must not overlap")
    return {"report_root": report_path, "artifact_root": artifact_path}


def resolve_output_source_roots(arguments: argparse.Namespace) -> Mapping[str, Path]:
    records = parse_pin_manifest_records(Path(arguments.corpus_manifest))
    overrides = parse_project_root_overrides(arguments.project_root)
    source_roots = {}
    for repository_name, record in records.items():
        root_value = overrides.get(repository_name, record.get("corpus_source_checkout_path"))
        if not isinstance(root_value, (Path, str)) or not str(root_value):
            raise RunnerPreconditionBlockedError(
                f"pinned source checkout path is missing for {repository_name}"
            )
        try:
            source_roots[repository_name] = Path(root_value).expanduser().resolve(strict=False)
        except (OSError, RuntimeError) as error:
            raise RunnerPreconditionBlockedError(
                f"pinned source checkout cannot be canonicalized for {repository_name}: {error}"
            ) from error
    return source_roots


def canonicalize_non_symlink_path(path_value: Path, allow_missing: bool) -> Path:
    path = Path(path_value).expanduser()
    if not path.is_absolute():
        path = Path.cwd() / path
    if ".." in path.parts:
        raise RunnerPreconditionBlockedError(f"path aliases containing '..' are unsupported: {path}")
    current_path = Path(path.anchor)
    for component in path.parts[1:]:
        current_path /= component
        if current_path.is_symlink():
            raise RunnerPreconditionBlockedError(f"symlink path components are unsupported: {current_path}")
        if current_path.exists() and current_path != path and not current_path.is_dir():
            raise RunnerPreconditionBlockedError(f"path component is not a directory: {current_path}")
    if not allow_missing and not path.exists():
        raise RunnerPreconditionBlockedError(f"path does not exist: {path}")
    return path.resolve(strict=not allow_missing)


def validate_s3_evidence_path(arguments, output_paths, source_roots) -> None:
    evidence_value = getattr(arguments, "s3_prerequisite_evidence", None)
    run_id = arguments.run_id
    output_directories = [output_paths["report_root"] / run_id, output_paths["artifact_root"] / run_id]
    source_paths = [PROJECT_ROOT, *source_roots.values()]
    if evidence_value is not None:
        evidence_path = canonicalize_non_symlink_path(Path(evidence_value), allow_missing=True)
        for source_path in source_paths:
            if source_output_paths_overlap(evidence_path, Path(source_path).resolve()):
                raise RunnerPreconditionBlockedError(
                    f"S3 prerequisite evidence must be external to source roots: {evidence_path} overlaps {source_path}"
                )
        for output_directory in output_directories:
            if source_output_paths_overlap(evidence_path, output_directory):
                raise RunnerPreconditionBlockedError(
                    f"S3 evidence cannot overlap a newly owned output directory: {evidence_path}"
                )
        arguments.s3_prerequisite_evidence = evidence_path
    target_directory_value = os.environ.get("CARGO_TARGET_DIR")
    if target_directory_value:
        target_directory = canonicalize_non_symlink_path(Path(target_directory_value), allow_missing=True)
        for source_path in source_paths:
            if source_output_paths_overlap(target_directory, Path(source_path).resolve()):
                raise RunnerPreconditionBlockedError(
                    f"CARGO_TARGET_DIR must be external to source roots: {target_directory} overlaps {source_path}"
                )
        for output_directory in output_directories:
            if source_output_paths_overlap(target_directory, output_directory):
                raise RunnerPreconditionBlockedError("CARGO_TARGET_DIR overlaps a newly owned output directory")


def validate_run_output_paths(arguments: argparse.Namespace) -> Mapping[str, Path]:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}", arguments.run_id):
        raise RunnerPreconditionBlockedError("run identifier must be a safe, unique path component")
    source_roots = resolve_output_source_roots(arguments)
    output_paths = validate_output_root_paths(
        arguments.report_root,
        arguments.artifact_root,
        PROJECT_ROOT,
        source_roots,
    )
    validate_s3_evidence_path(arguments, output_paths, source_roots)
    arguments.report_root = output_paths["report_root"]
    arguments.artifact_root = output_paths["artifact_root"]
    return output_paths


def capture_identity_registry(
    project_root: Path,
    manifest_paths: Sequence[Path],
    corpus_roots: Mapping[str, Path],
    excluded_paths: Sequence[Path],
    executable_identities: Sequence[Mapping[str, Any]],
    extra_frozen_paths: Sequence[Path] = (),
    optional_frozen_paths: Sequence[Path] = (),
    expected_frozen_files: Mapping[str, Mapping[str, Any]] | None = None,
    expected_optional_files: Mapping[str, Mapping[str, Any]] | None = None,
) -> Mapping[str, Any]:
    frozen_files = {}
    for file_path in manifest_paths:
        file_identity = capture_frozen_file_identity(file_path)
        expected_identity = (expected_frozen_files or {}).get(file_identity["path"])
        if expected_identity is not None and dict(file_identity) != dict(expected_identity):
            raise RunnerFrozenInputMutationError(f"frozen input changed while freezing: {file_identity['path']}")
        frozen_files[file_identity["path"]] = dict(expected_identity or file_identity)
    lock_files = {str((project_root / "Cargo.lock").absolute()): capture_optional_frozen_file_identity(project_root / "Cargo.lock")}
    for corpus_root in corpus_roots.values():
        lock_path = corpus_root / "Cargo.lock"
        lock_files[str(lock_path.absolute())] = capture_optional_frozen_file_identity(lock_path)
    source_roots = {str(project_root.resolve()): Path(project_root).resolve()}
    permitted_lock_paths_by_root = {}
    for repository_name, root in corpus_roots.items():
        root_path = root.resolve()
        root_key = str(root_path)
        source_roots[root_key] = root_path
        if repository_name in {"serde", "tokio"}:
            permitted_lock_paths_by_root[root_key] = [root_path / "Cargo.lock"]
    source_snapshots = {
        root_key: capture_frozen_source_snapshot(
            root_path, excluded_paths, permitted_lock_paths_by_root.get(root_key, ())
        )
        for root_key, root_path in source_roots.items()
    }
    for executable_identity in executable_identities:
        frozen_files[executable_identity["path"]] = dict(executable_identity)
    for file_path in extra_frozen_paths:
        file_identity = capture_frozen_file_identity(file_path)
        expected_identity = (expected_frozen_files or {}).get(file_identity["path"])
        if expected_identity is not None and file_identity != expected_identity:
            raise RunnerFrozenInputMutationError(f"evidence changed while freezing: {file_identity['path']}")
        frozen_files[file_identity["path"]] = dict(expected_identity or file_identity)
    optional_files = {}
    for file_path in optional_frozen_paths:
        file_key = str(Path(file_path).absolute())
        current_identity = capture_optional_frozen_file_identity(Path(file_path))
        expected_identity = (expected_optional_files or {}).get(file_key)
        if expected_identity is not None and dict(current_identity) != dict(expected_identity):
            raise RunnerFrozenInputMutationError(f"optional frozen input changed while freezing: {file_key}")
        optional_files[file_key] = dict(expected_identity or current_identity)
    return {
        "frozen_files": frozen_files,
        "optional_files": optional_files,
        "lock_files": lock_files,
        "source_snapshots": source_snapshots,
        "permitted_lock_paths_by_root": {
            root_key: [str(path) for path in paths]
            for root_key, paths in permitted_lock_paths_by_root.items()
        },
        "excluded_paths": [str(Path(path).absolute()) for path in excluded_paths],
    }


def verify_identity_registry(registry: Mapping[str, Any]) -> None:
    for identity in registry["frozen_files"].values():
        verify_frozen_file_identity(identity)
    for identity in registry["lock_files"].values():
        verify_optional_frozen_file_identity(identity)
    for identity in registry.get("optional_files", {}).values():
        verify_optional_frozen_file_identity(identity)
    for identity in registry.get("runner_owned_output_identities", {}).values():
        verify_frozen_file_identity(identity)
    excluded_paths = [Path(path) for path in registry["excluded_paths"]]
    permitted_lock_paths_by_root = registry.get("permitted_lock_paths_by_root", {})
    for root_path, expected_snapshot in registry["source_snapshots"].items():
        try:
            current_snapshot = capture_frozen_source_snapshot(
                Path(root_path), excluded_paths, permitted_lock_paths_by_root.get(root_path, ())
            )
        except RunnerPreconditionBlockedError as error:
            raise RunnerFrozenInputMutationError(
                f"frozen source snapshot can no longer be verified: {root_path}: {error}"
            ) from error
        if current_snapshot != expected_snapshot:
            raise RunnerFrozenInputMutationError(f"frozen source snapshot changed: {root_path}")


def capture_s3_evidence_environment() -> Mapping[str, str]:
    environment = {}
    for name, value in os.environ.items():
        if name not in S3_RELEVANT_ENVIRONMENT_NAMES and not name.startswith(
            ("PARSELTONGUE_", "CARGO_PROFILE_", "CARGO_TARGET_", "CC_", "CFLAGS_", "CXXFLAGS_",
             "AR_", "PKG_CONFIG_")
        ):
            continue
        if re.search(r"(?:TOKEN|SECRET|PASSWORD|CREDENTIAL|AUTH|PRIVATE_KEY)", name, re.IGNORECASE):
            continue
        environment[name] = value
    return dict(sorted(environment.items()))


def find_cargo_configuration_files(project_root, corpus_roots, environment):
    candidate_paths = set()
    roots = [Path(project_root), *[Path(root) for root in corpus_roots.values()]]
    for root in roots:
        current = root.resolve()
        while True:
            candidate_paths.update(current / ".cargo" / name for name in ("config.toml", "config"))
            if current.parent == current:
                break
            current = current.parent
    cargo_home = environment.get("CARGO_HOME")
    if cargo_home is not None:
        cargo_home_path = Path(cargo_home).expanduser()
        if not cargo_home_path.is_absolute():
            cargo_home_path = Path.cwd() / cargo_home_path
    else:
        cargo_home_path = Path.home() / ".cargo"
    candidate_paths.update(cargo_home_path / name for name in ("config.toml", "config"))
    return sorted(candidate_paths, key=str)


def cargo_configuration_override_reason(configuration_paths):
    for config_path in configuration_paths:
        if not Path(config_path).exists():
            continue
        try:
            document = tomllib.loads(Path(config_path).read_text())
        except (OSError, tomllib.TOMLDecodeError) as error:
            return f"Cargo config cannot be parsed: {config_path}: {error}"
        build = document.get("build", {})
        if not isinstance(build, Mapping):
            return f"Cargo config build table is not an object: {config_path}"
        forbidden_build_keys = {
            "target", "target-dir", "rustflags", "rustc", "rustc-wrapper", "rustc-workspace-wrapper",
        }
        if forbidden_build_keys.intersection(build):
            return f"Cargo config changes target/compiler policy: {config_path}"
        target_tables = document.get("target", {})
        if isinstance(target_tables, Mapping):
            for target_config in target_tables.values():
                if isinstance(target_config, Mapping) and {"rustflags", "linker"}.intersection(target_config):
                    return f"Cargo target config changes cfg/compiler policy: {config_path}"
        configured_environment = document.get("env", {})
        if isinstance(configured_environment, Mapping) and any(
            re.search(r"(?:SKIP|FILTER|IGNORE|NO_NATIVE)", str(name), re.IGNORECASE)
            for name in configured_environment
        ):
            return f"Cargo config injects an unsupported test-skip environment: {config_path}"
        if isinstance(configured_environment, Mapping) and any(
            str(name).startswith(("CARGO_TARGET_", "CARGO_BUILD_TARGET", "RUSTFLAGS", "RUSTC"))
            for name in configured_environment
        ):
            return f"Cargo config injects target/cfg/compiler environment: {config_path}"
    return None


def s3_configuration_blocked_gates(environment, configuration_reason):
    blocked = {}
    native_skip = any(
        re.search(r"(?:SKIP|FILTER|IGNORE|NO_NATIVE)", name, re.IGNORECASE) and value
        for name, value in environment.items()
    )
    target_or_compiler_override = any(environment.get(name) for name in (
        "CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC",
        "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER",
    )) or any(name.startswith("CARGO_TARGET_") and name != "CARGO_TARGET_DIR" and value
             for name, value in environment.items()) or any(
                 name.startswith("CARGO_PROFILE_") and value for name, value in environment.items()
             )
    if native_skip:
        blocked["fixture_suite"] = "native test skip/filter environment is unsupported"
    if target_or_compiler_override:
        for gate in ("fixture_suite", "build_evidence", "release_provenance", "p2_offline_qualification"):
            blocked[gate] = "unsupported Cargo target/compiler configuration affects this prerequisite"
    if configuration_reason:
        if "test-skip" in configuration_reason or "native test skip" in configuration_reason:
            blocked["fixture_suite"] = configuration_reason
        elif "target/compiler" in configuration_reason or "target/cfg/compiler" in configuration_reason:
            for gate in ("fixture_suite", "build_evidence", "release_provenance", "p2_offline_qualification"):
                blocked[gate] = configuration_reason
        else:
            for gate in S3_GATE_NAMES:
                blocked[gate] = configuration_reason
    return blocked


def read_workspace_unit_inventory(repository_root):
    root = Path(repository_root).resolve(strict=True)
    root_manifest_path = root / "Cargo.toml"
    try:
        root_manifest = tomllib.loads(root_manifest_path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"workspace manifest cannot be parsed: {root_manifest_path}: {error}") from error
    workspace = root_manifest.get("workspace", {})
    members = workspace.get("members") if isinstance(workspace, Mapping) else None
    excluded = workspace.get("exclude", []) if isinstance(workspace, Mapping) else []
    member_paths = {root_manifest_path.resolve()} if "package" in root_manifest else set()
    if members is not None:
        if not isinstance(members, list) or not members:
            raise RunnerPreconditionBlockedError(f"workspace member patterns are unsupported: {root_manifest_path}")
        for pattern in members:
            matches = list(root.glob(pattern))
            if not matches:
                raise RunnerPreconditionBlockedError(f"workspace member pattern matched nothing: {pattern}")
            for match in matches:
                manifest_path = (match / "Cargo.toml") if match.is_dir() else match
                if manifest_path.name == "Cargo.toml" and manifest_path.is_file():
                    member_paths.add(manifest_path.resolve())
    for pattern in excluded:
        for match in root.glob(pattern):
            excluded_path = (match / "Cargo.toml") if match.is_dir() else match
            member_paths.discard(excluded_path.resolve())
    if not member_paths:
        raise RunnerPreconditionBlockedError(f"workspace has no Cargo package manifests: {root_manifest_path}")
    workspace_package = workspace.get("package", {}) if isinstance(workspace, Mapping) else {}
    selected_units = []
    for manifest_path in sorted(member_paths, key=str):
        if not manifest_path.is_relative_to(root):
            raise RunnerPreconditionBlockedError(f"workspace member escapes pinned source root: {manifest_path}")
        try:
            manifest = tomllib.loads(manifest_path.read_text())
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise RunnerPreconditionBlockedError(f"member manifest cannot be parsed: {manifest_path}: {error}") from error
        package = manifest.get("package")
        if not isinstance(package, Mapping) or not isinstance(package.get("name"), str):
            raise RunnerPreconditionBlockedError(f"workspace member has no package identity: {manifest_path}")
        package_name = package["name"]
        edition_value = package.get("edition", workspace_package.get("edition"))
        if isinstance(edition_value, Mapping) and edition_value.get("workspace") is True:
            edition = workspace_package.get("edition")
        else:
            edition = edition_value
        if not isinstance(edition, str):
            raise RunnerPreconditionBlockedError(f"workspace package edition is not explicit: {manifest_path}")
        lib = manifest.get("lib", {})
        if not isinstance(lib, Mapping):
            raise RunnerPreconditionBlockedError(f"lib target table is invalid: {manifest_path}")
        lib_relative_path = lib.get("path", "src/lib.rs")
        if not isinstance(lib_relative_path, str) or not lib_relative_path:
            raise RunnerPreconditionBlockedError(f"lib target source path is unsupported: {manifest_path}")
        lib_path = manifest_path.parent / lib_relative_path
        if "lib" in manifest or (package.get("autolib", True) and lib_path.is_file()):
            lib_source_path = canonicalize_non_symlink_path(lib_path, allow_missing=False)
            if not lib_source_path.is_relative_to(root):
                raise RunnerPreconditionBlockedError(f"lib target source escapes pinned checkout: {lib_path}")
            crate_types = lib.get("crate-type", ["lib"])
            if isinstance(crate_types, str):
                crate_types = [crate_types]
            if (not isinstance(crate_types, list) or not crate_types
                    or any(not isinstance(crate_type, str) or not crate_type for crate_type in crate_types)):
                raise RunnerPreconditionBlockedError(f"lib crate types are unsupported: {manifest_path}")
            proc_macro = lib.get("proc-macro", False)
            if not isinstance(proc_macro, bool):
                raise RunnerPreconditionBlockedError(f"lib proc-macro policy is malformed: {manifest_path}")
            unsupported_types = set(crate_types) - {"lib", "rlib", "proc-macro"}
            if unsupported_types:
                raise RunnerPreconditionBlockedError(
                    f"P2 does not support lib crate types {sorted(unsupported_types)}: {manifest_path}"
                )
            if proc_macro or "proc-macro" in crate_types:
                if set(crate_types) - {"lib", "proc-macro"}:
                    raise RunnerPreconditionBlockedError(f"proc-macro crate types are unsupported: {manifest_path}")
                kind = ["proc-macro"]
            else:
                kind = sorted(set(crate_types))
            selected_units.append({
                "manifest_path": str(manifest_path), "target_name": lib.get("name", package_name.replace("-", "_")),
                "kind": kind, "edition": edition, "src_path": str(lib_source_path),
            })
        explicit_bins = manifest.get("bin", [])
        if isinstance(explicit_bins, Mapping):
            explicit_bins = [explicit_bins]
        if not isinstance(explicit_bins, list):
            raise RunnerPreconditionBlockedError(f"bin targets are invalid: {manifest_path}")
        bin_targets = {}
        for binary in explicit_bins:
            if not isinstance(binary, Mapping):
                raise RunnerPreconditionBlockedError(f"bin target is invalid: {manifest_path}")
            name = binary.get("name")
            target_path = binary.get("path", f"src/bin/{name}.rs" if name else "")
            if not isinstance(name, str) or not isinstance(target_path, str) or not target_path:
                raise RunnerPreconditionBlockedError(f"bin target identity is incomplete: {manifest_path}")
            bin_targets[name] = target_path
        if package.get("autobins", True):
            main_path = manifest_path.parent / "src/main.rs"
            if main_path.is_file():
                bin_targets.setdefault(package_name, "src/main.rs")
            bin_root = manifest_path.parent / "src/bin"
            if bin_root.is_dir():
                for target_path in bin_root.glob("*.rs"):
                    bin_targets.setdefault(target_path.stem, str(target_path.relative_to(manifest_path.parent)))
                for target_path in bin_root.glob("*/main.rs"):
                    bin_targets.setdefault(target_path.parent.name, str(target_path.relative_to(manifest_path.parent)))
        explicit_bin_names = {binary.get("name") for binary in explicit_bins if isinstance(binary, Mapping)}
        for name, relative_target_path in bin_targets.items():
            target_file = manifest_path.parent / relative_target_path
            if name in explicit_bin_names or target_file.is_file():
                source_path = canonicalize_non_symlink_path(target_file, allow_missing=False)
                if not source_path.is_relative_to(root):
                    raise RunnerPreconditionBlockedError(f"bin target source escapes pinned checkout: {target_file}")
                selected_units.append({
                    "manifest_path": str(manifest_path), "target_name": name,
                    "kind": ["bin"], "edition": edition, "src_path": str(source_path),
                })
    if not selected_units:
        raise RunnerPreconditionBlockedError(f"workspace has zero statically selected lib/bin targets: {root}")
    selected_units.sort(key=lambda unit: (unit["manifest_path"], unit["target_name"], tuple(unit["kind"])))
    return {
        "workspace_members": [str(path) for path in sorted(member_paths, key=str)],
        "selected_units": selected_units,
    }


def collect_manifest_s3_references(manifest_document):
    references = []

    def visit(value):
        if isinstance(value, Mapping):
            if {"path", "size_bytes", "sha256", "blake3"}.issubset(value):
                references.append(value)
            else:
                for nested_value in value.values():
                    visit(nested_value)
        elif isinstance(value, list):
            for nested_value in value:
                visit(nested_value)

    if isinstance(manifest_document, Mapping):
        visit(manifest_document.get("global_prerequisites"))
        visit(manifest_document.get("repositories"))
    return references


def derive_production_source_test_inventory(project_root):
    tracked_paths = run_read_only_git_command(project_root, ["ls-files", "-z", "--", "crates"]).split("\0")
    source_contents = {}
    ambiguous_reasons = []
    for relative_path in tracked_paths:
        if not relative_path or Path(relative_path).suffix != ".rs":
            continue
        source_path = Path(project_root) / relative_path
        try:
            source_contents[relative_path] = source_path.read_text()
        except (OSError, UnicodeError) as error:
            ambiguous_reasons.append(f"frozen Rust source cannot be read: {relative_path}: {error}")
    inventory = s3_evidence_intake.derive_source_fixture_inventory(source_contents)
    if ambiguous_reasons:
        inventory["ambiguous_reason"] = "; ".join(
            filter(None, [inventory.get("ambiguous_reason"), *ambiguous_reasons])
        )
    return inventory


def read_s3_evidence_inputs(evidence_manifest_path, b3_provider, b3_tool_identity, forbidden_roots=()):
    input_state = {
        "manifest_document": None, "file_bytes_by_path": {}, "blake3_by_path": {},
        "frozen_paths": [], "optional_paths": [], "frozen_identities": {},
        "optional_identities": {}, "intake_error": None,
    }

    def record_intake_error(status, reason):
        input_state["intake_error"] = merge_s3_intake_error(
            input_state["intake_error"], (status, str(reason)),
        )

    def record_optional_path(path_value):
        identity = capture_optional_frozen_file_identity(Path(path_value))
        path_key = identity["path"]
        if identity["exists"]:
            missing_identity = {"path": path_key, "exists": False}
            input_state["optional_identities"].setdefault(path_key, missing_identity)
            input_state["optional_paths"].append(Path(path_key))
            raise RunnerFrozenInputMutationError(f"missing evidence appeared during intake: {path_key}")
        previous_identity = input_state["optional_identities"].get(path_key)
        if previous_identity is not None and dict(previous_identity) != dict(identity):
            raise RunnerFrozenInputMutationError(f"optional evidence changed during intake: {path_key}")
        input_state["optional_identities"][path_key] = identity
        input_state["optional_paths"].append(Path(path_key))

    if evidence_manifest_path is None:
        record_intake_error("BLOCKED", "S3 prerequisite manifest was not supplied")
        return input_state
    manifest_path = Path(evidence_manifest_path)
    try:
        manifest_canonical = canonicalize_non_symlink_path(manifest_path, allow_missing=True)
    except RunnerPreconditionBlockedError as error:
        record_intake_error("BLOCKED", error)
        return input_state
    if not manifest_canonical.exists():
        try:
            record_optional_path(manifest_canonical)
        except (RunnerPreconditionBlockedError, RunnerContractViolationError) as error:
            status = "FAIL" if isinstance(error, RunnerContractViolationError) else "BLOCKED"
            record_intake_error(status, error)
        record_intake_error("BLOCKED", f"S3 prerequisite manifest is missing: {manifest_canonical}")
        return input_state
    if manifest_canonical.is_symlink() or not manifest_canonical.is_file():
        record_intake_error("FAIL", f"S3 prerequisite manifest is not a regular non-symlink file: {manifest_canonical}")
        return input_state
    for forbidden_root in forbidden_roots:
        root = Path(forbidden_root).resolve()
        if manifest_canonical.is_relative_to(root) or root.is_relative_to(manifest_canonical):
            record_intake_error("BLOCKED", "S3 evidence must be external to source and output roots")
            return input_state
    try:
        manifest_identity_before = capture_frozen_file_identity(manifest_canonical)
        input_state["frozen_paths"].append(manifest_canonical)
        input_state["frozen_identities"][manifest_identity_before["path"]] = manifest_identity_before
        manifest_bytes = manifest_canonical.read_bytes()
        if hashlib.sha256(manifest_bytes).hexdigest() != manifest_identity_before["sha256"]:
            raise RunnerFrozenInputMutationError("manifest bytes changed during intake")
        verify_frozen_file_identity(manifest_identity_before)
        input_state["manifest_document"] = s3_evidence_intake.parse_s3_evidence_json(
            manifest_bytes.decode("utf-8"), "S3 prerequisite manifest"
        )
    except (OSError, UnicodeError, RunnerPreconditionBlockedError, RunnerContractViolationError,
            s3_evidence_intake.S3EvidenceBlockedError,
            s3_evidence_intake.S3EvidenceViolationError) as error:
        status = "FAIL" if isinstance(error, (RunnerContractViolationError,
                                               s3_evidence_intake.S3EvidenceViolationError)) else "BLOCKED"
        record_intake_error(status, error)
        return input_state
    evidence_root = manifest_path.parent.resolve()

    def read_member(relative_path):
        if not isinstance(relative_path, str) or not relative_path or "\\" in relative_path:
            raise RunnerPreconditionBlockedError("evidence member path must be relative POSIX text")
        member = PurePosixPath(relative_path)
        if member.is_absolute() or any(part in {"", ".", ".."} for part in relative_path.split("/")):
            raise RunnerPreconditionBlockedError(f"unsafe evidence member path: {relative_path}")
        path = evidence_root.joinpath(*member.parts)
        canonical_path = canonicalize_non_symlink_path(path, allow_missing=True)
        if not canonical_path.is_relative_to(evidence_root):
            raise RunnerPreconditionBlockedError(f"evidence member escapes manifest directory: {relative_path}")
        path_key = str(canonical_path.absolute())
        if canonical_path.exists() and path_key in input_state["optional_identities"]:
            if not input_state["optional_identities"][path_key]["exists"]:
                raise RunnerFrozenInputMutationError(f"missing evidence appeared during intake: {relative_path}")
        if not canonical_path.exists():
            if path_key in input_state["frozen_identities"]:
                raise RunnerFrozenInputMutationError(f"evidence disappeared during intake: {relative_path}")
            record_optional_path(canonical_path)
            return None
        if canonical_path.is_symlink() or not canonical_path.is_file():
            raise RunnerContractViolationError(f"evidence member is not a regular non-symlink file: {relative_path}")
        for forbidden_root in forbidden_roots:
            root = Path(forbidden_root).resolve()
            if canonical_path.is_relative_to(root) or root.is_relative_to(canonical_path):
                raise RunnerPreconditionBlockedError("S3 evidence member overlaps a source/output root")
        file_identity = capture_frozen_file_identity(canonical_path)
        initial_identity = input_state["frozen_identities"].get(file_identity["path"])
        if initial_identity is not None and dict(initial_identity) != dict(file_identity):
            raise RunnerFrozenInputMutationError(f"evidence changed during intake: {relative_path}")
        input_state["frozen_paths"].append(canonical_path)
        input_state["frozen_identities"][file_identity["path"]] = initial_identity or file_identity
        file_bytes = canonical_path.read_bytes()
        if hashlib.sha256(file_bytes).hexdigest() != file_identity["sha256"]:
            raise RunnerFrozenInputMutationError(f"evidence bytes changed during intake: {relative_path}")
        verify_frozen_file_identity(file_identity)
        digest = calculate_b3_file_digest(
            canonical_path, b3_provider, file_identity, provider_identity=b3_tool_identity,
        )
        input_state["file_bytes_by_path"][relative_path] = file_bytes
        input_state["blake3_by_path"][relative_path] = digest
        return file_bytes

    for reference in collect_manifest_s3_references(input_state["manifest_document"]):
        relative_path = reference.get("path") if isinstance(reference, Mapping) else None
        try:
            receipt_bytes = read_member(relative_path)
            if receipt_bytes is None:
                continue
            receipt_document = s3_evidence_intake.parse_s3_evidence_json(
                receipt_bytes.decode("utf-8"), str(relative_path)
            )
            if not isinstance(receipt_document, Mapping):
                continue
            for stream_name in ("stdout", "stderr"):
                descriptor = receipt_document.get(stream_name)
                if isinstance(descriptor, Mapping) and isinstance(descriptor.get("path"), str):
                    read_member(descriptor["path"])
        except (OSError, UnicodeError, RunnerPreconditionBlockedError,
                RunnerContractViolationError, s3_evidence_intake.S3EvidenceBlockedError,
                s3_evidence_intake.S3EvidenceViolationError) as error:
            status = "FAIL" if isinstance(error, (RunnerContractViolationError,
                                                   s3_evidence_intake.S3EvidenceViolationError)) else "BLOCKED"
            record_intake_error(status, error)
    input_state["frozen_paths"] = list(dict.fromkeys(input_state["frozen_paths"]))
    input_state["optional_paths"] = list(dict.fromkeys(input_state["optional_paths"]))
    return input_state


def build_s3_repository_bindings(corpus_records, corpus_roots, source_snapshots,
                                 lock_files, p2_configuration_reason):
    bindings = {}
    for repository_name in DEFAULT_REPOSITORY_ORDER:
        root = corpus_roots[repository_name]
        try:
            workspace_inventory = read_workspace_unit_inventory(root)
            p2_reason = p2_configuration_reason
        except RunnerPreconditionBlockedError as error:
            workspace_inventory = {"workspace_members": [], "selected_units": []}
            p2_reason = str(error)
        bindings[repository_name] = {
            "pinned_name": repository_name,
            "pinned_commit": corpus_records[repository_name]["corpus_source_commit_hash"],
            "checkout_path": str(root),
            "source_snapshot": source_snapshots[repository_name],
            "lock_identity": lock_files[str((root / "Cargo.lock").absolute())],
            "selected_policy": {
                "workspace_selector": "--workspace", "target_selectors": ["--lib", "--bins"],
                "features": "default", "per_unit_cfg": "compiler-default", "target": "compiler-default",
            },
            **workspace_inventory,
            "p2_policy_supported": p2_reason is None,
            "p2_blocked_reason": p2_reason,
        }
    return bindings


def build_s3_expected_identity(arguments, run_state, mapper_identity, pin_summary,
                               registry, toolchain_versions, cargo_configuration_paths,
                               p2_configuration_reason, evidence_environment=None):
    frozen_files = registry["frozen_files"]
    parser_manifest_path = str((PROJECT_ROOT / "docs/phase-20261111/parser-source-pin-manifest.json").resolve())
    parser_manifest = json.loads(Path(parser_manifest_path).read_text())
    parser_bundle_path = str((PROJECT_ROOT / parser_manifest["parser_source_bundle_path"]).resolve())
    r1_patch_path = str(Path(pin_summary["r1_patch_path"]).resolve())
    production_manifest_path = (PROJECT_ROOT / "crates/parseltongue-cli-dispatcher-binary/Cargo.toml").resolve(strict=True)
    try:
        production_document = tomllib.loads(production_manifest_path.read_text())
        root_document = tomllib.loads((PROJECT_ROOT / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"production Cargo identity cannot be parsed: {error}") from error
    production_package = production_document.get("package")
    if not isinstance(production_package, Mapping) or not isinstance(production_package.get("name"), str):
        raise RunnerPreconditionBlockedError("production Cargo package name is unavailable")
    package_version = production_package.get("version")
    if isinstance(package_version, Mapping) and package_version.get("workspace") is True:
        workspace_package = root_document.get("workspace", {}).get("package", {})
        package_version = workspace_package.get("version") if isinstance(workspace_package, Mapping) else None
    if not isinstance(package_version, str) or not package_version:
        raise RunnerPreconditionBlockedError("production Cargo package version is unavailable")
    production_bin_targets = production_document.get("bin")
    if not isinstance(production_bin_targets, list):
        raise RunnerPreconditionBlockedError("production [[bin]] target inventory is unavailable")
    production_bins = [target for target in production_bin_targets
                       if isinstance(target, Mapping) and target.get("name") == "parseltongue"]
    if len(production_bins) != 1 or not isinstance(production_bins[0].get("path"), str):
        raise RunnerPreconditionBlockedError("production parseltongue bin source is unavailable or ambiguous")
    production_bin_source = (production_manifest_path.parent / production_bins[0]["path"]).resolve(strict=True)
    frozen_production_manifest = frozen_files.get(str(production_manifest_path))
    frozen_production_bin_source = frozen_files.get(str(production_bin_source))
    frozen_root_manifest = frozen_files.get(str((PROJECT_ROOT / "Cargo.toml").resolve()))
    if frozen_production_manifest is None or frozen_production_bin_source is None or frozen_root_manifest is None:
        raise RunnerPreconditionBlockedError("production package identity files were not frozen in the registry")
    config_file_identities = []
    for config_path in cargo_configuration_paths:
        config_key = str(Path(config_path).absolute())
        if config_key in frozen_files:
            config_file_identities.append({"exists": True, **dict(frozen_files[config_key])})
        elif config_key in registry.get("optional_files", {}):
            config_file_identities.append(dict(registry["optional_files"][config_key]))
        else:
            raise RunnerPreconditionBlockedError(f"Cargo config identity was not frozen: {config_key}")
    toolchain_executables = {}
    for tool_name, version_record in toolchain_versions.items():
        identity = version_record.get("file_identity") if isinstance(version_record, Mapping) else None
        if isinstance(identity, Mapping) and identity.get("path") in frozen_files:
            toolchain_executables[tool_name] = dict(frozen_files[identity["path"]])
        else:
            toolchain_executables[tool_name] = None
    environment = (
        dict(evidence_environment) if evidence_environment is not None
        else capture_s3_evidence_environment()
    )
    unsupported_gates = s3_configuration_blocked_gates(environment, p2_configuration_reason)
    return {
        "run_id": arguments.run_id,
        "milestone": arguments.milestone,
        "mapper": {
            "canonical_path": mapper_identity["canonical_path"],
            "sha256": mapper_identity["sha256"], "blake3": mapper_identity["blake3"],
        },
        "runner_source_snapshot": registry["source_snapshots"][str(PROJECT_ROOT.resolve())],
        "production_cargo_lock": registry["lock_files"][str((PROJECT_ROOT / "Cargo.lock").absolute())],
        "parser_pin_bundle": {
            "pin_manifest": frozen_files[parser_manifest_path],
            "bundle": frozen_files[parser_bundle_path],
        },
        "r1_patch": frozen_files[r1_patch_path],
        "toolchain_versions": toolchain_versions,
        "toolchain_executables": toolchain_executables,
        "relevant_environment": environment,
        "configuration": {
            "project_root": str(PROJECT_ROOT.resolve()),
            "production_manifest": str(production_manifest_path),
            "production_manifest_identity": frozen_production_manifest,
            "workspace_manifest_identity": frozen_root_manifest,
            "production_package_name": production_package["name"],
            "production_package_version": package_version,
            "production_bin_source": str(production_bin_source),
            "production_bin_source_identity": frozen_production_bin_source,
            "production_package": production_package["name"],
            "production_binary": "parseltongue",
            "selected_p2_policy": {
                "workspace_selector": "--workspace", "target_selectors": ["--lib", "--bins"],
                "features": "default", "per_unit_cfg": "compiler-default", "target": "compiler-default",
            },
            "cargo_config_file_identities": config_file_identities,
            "p2_configuration_blocked_reason": p2_configuration_reason,
            "unsupported_prerequisite_gates": unsupported_gates,
            "mode": arguments.mode,
            "corpus_manifest": str(Path(arguments.corpus_manifest).resolve()),
            "project_root_overrides": [str(value) for value in arguments.project_root],
        },
    }


def prepare_s3_evidence_qualification(arguments, run_state, evidence_inputs,
                                      expected_identity, expected_repository_bindings,
                                      source_test_inventory, dependency_reference=None):
    intake_error = evidence_inputs.get("intake_error")
    intake_status, intake_reason = intake_error if intake_error is not None else ("PASS", None)
    validation_error = None
    validation_result = {"repository_results": {}, "tokens_by_repository": {}}
    if isinstance(evidence_inputs.get("manifest_document"), Mapping):
        try:
            validation_result = s3_evidence_intake.validate_s3_prerequisite_evidence(
                evidence_inputs["manifest_document"], evidence_inputs.get("file_bytes_by_path", {}),
                evidence_inputs.get("blake3_by_path", {}), expected_identity, expected_repository_bindings,
                source_test_inventory, arguments.milestone, dependency_reference=dependency_reference,
            )
        except s3_evidence_intake.S3EvidenceViolationError as error:
            validation_error = ("FAIL", str(error))
        except s3_evidence_intake.S3EvidenceBlockedError as error:
            validation_error = ("BLOCKED", str(error))
    repository_results = {
        repository_name: validation_result["repository_results"].get(repository_name,
            {gate: {"status": validation_error[0] if validation_error else "BLOCKED",
                    "reason": validation_error[1] if validation_error else
                    "S3 prerequisite evidence was unavailable"} for gate in S3_GATE_NAMES})
        for repository_name in DEFAULT_REPOSITORY_ORDER
    }
    unsupported_gates = expected_identity.get("configuration", {}).get("unsupported_prerequisite_gates", {})
    for repository_result in repository_results.values():
        for gate_name, gate_reason in unsupported_gates.items():
            if gate_name in repository_result and repository_result[gate_name].get("status") != "FAIL":
                repository_result[gate_name] = {"status": "BLOCKED", "reason": gate_reason}
    tokens = dict(validation_result.get("tokens_by_repository", {}))
    for repository_name, gate_results in repository_results.items():
        if intake_error is not None or any(value.get("status") != "PASS" for value in gate_results.values()):
            tokens.pop(repository_name, None)
    qualification_status = "PASS"
    status_priority = {"PASS": 0, "BLOCKED": 1, "FAIL": 2}
    statuses = [intake_status]
    if validation_error:
        statuses.append(validation_error[0])
    statuses.extend(value.get("status", "BLOCKED") for results in repository_results.values()
                    for value in results.values())
    qualification_status = max(statuses, key=lambda value: status_priority.get(value, 1))
    result = {
        **validation_result, "manifest_document": evidence_inputs.get("manifest_document"),
        "repository_results": repository_results, "identity_binding": expected_identity,
        "tokens_by_repository": tokens,
        "intake_status": {"status": intake_status, "reason": intake_reason},
        "qualification_status": qualification_status,
        "validation_error": validation_error,
    }
    run_state["manifest"]["s3_prerequisite_intake"] = {
        "manifest_path": str(getattr(arguments, "s3_prerequisite_evidence", None))
        if getattr(arguments, "s3_prerequisite_evidence", None) else None,
        "status": intake_status, "reason": intake_reason,
        "qualification_status": qualification_status, "validation_error": validation_error,
        "repository_results": repository_results,
    }
    return result


def make_identity_boundary_callback(registry: Mapping[str, Any], expected_environment=None,
                                   mapper_identity=None, b3_provider=None, b3_tool_identity=None):
    def verify_boundary(phase_name: str, command_values: Sequence[str]) -> None:
        verify_identity_registry(registry)
        if expected_environment is not None and capture_s3_evidence_environment() != expected_environment:
            raise RunnerFrozenInputMutationError(
                f"relevant environment changed at frozen identity boundary: {phase_name}"
            )
        if phase_name in {
            "before S3 prerequisite evidence validation", "after S3 prerequisite evidence validation",
            "before S3 prerequisite gate", "after S3 prerequisite gate",
            "before S3 dependency tree collection", "after S3 dependency tree collection",
        } and mapper_identity is not None and b3_provider is not None:
            current_mapper_digest = calculate_b3_file_digest(
                Path(mapper_identity["canonical_path"]), b3_provider, mapper_identity,
                provider_identity=b3_tool_identity,
            )
            if current_mapper_digest != mapper_identity.get("blake3"):
                raise RunnerFrozenInputMutationError("frozen mapper BLAKE3 identity changed across S3 gates")
    return verify_boundary


def save_command_output_file(output_path: Path, output_text: str) -> None:
    if output_path.exists():
        raise FileExistsError(f"refusing to overwrite fresh command output {output_path}")
    output_path.write_text(output_text)


def derive_indexer_project_identifier(project_root: Path) -> str:
    return str(Path(project_root).resolve()).strip("/").replace("/", "-")


def record_s3_stage_transition(
    run_state: Mapping[str, Any], repository_name: str, status: str,
    command_values: Sequence[str], reason: str | None = None, clause: str | None = None,
) -> None:
    record_persistent_run_transition(run_state, repository_name, "S3", status,
                                     command_values, reason, clause)


def finalize_s3_component_qualification(
    run_state: Mapping[str, Any], repository_name: str,
    qualification_token=None, expected_identity_binding=None,
) -> str:
    repository_state = run_state["manifest"]["repositories"][repository_name]
    components = repository_state["s3_components"]
    missing_components = [name for name in S3_COMPONENT_NAMES if components.get(name) != "PASS"]
    unexpected_components = sorted(set(components) - set(S3_COMPONENT_NAMES))
    gates = repository_state["s3_qualification_gates"]
    gate_statuses = {name: gates.get(name, {}).get("status", "NOT STARTED") for name in S3_GATE_NAMES}
    token_is_valid = False
    if isinstance(qualification_token, s3_evidence_intake.S3EvidenceQualificationToken):
        expected_digest = hashlib.sha256(json.dumps(
            expected_identity_binding, sort_keys=True, separators=(",", ":"),
        ).encode()).hexdigest()
        token_is_valid = (
            qualification_token.run_identifier == run_state["manifest"].get("run_id")
            and qualification_token.repository_name == repository_name
            and qualification_token.identity_digest == expected_digest
        )
    if any(status == "FAIL" for status in components.values()) or unexpected_components or "FAIL" in gate_statuses.values():
        step_status = "FAIL"
        continuation_status = "FAIL"
        reason = (
            f"S3 component/prerequisite contract failed: components={missing_components}; "
            f"unexpected_components={unexpected_components}; gates={gate_statuses}"
        )
    elif run_state["manifest"].get("execution_identity") == "HARNESS ONLY" or missing_components or \
         "PASS" not in {gate_statuses.get(name) for name in S3_GATE_NAMES} or any(
        gate_statuses[name] != "PASS" for name in S3_GATE_NAMES
    ) or not token_is_valid:
        step_status = "BLOCKED"
        continuation_status = "BLOCKED"
        reason = (
            f"S3 requires four mapping components and five identity-bound prerequisites; "
            f"harness_only={run_state['manifest'].get('execution_identity') == 'HARNESS ONLY'}; "
            f"missing_components={missing_components}; gates={gate_statuses}; internal_token_valid={token_is_valid}"
        )
    else:
        step_status = "PASS"
        continuation_status = "PASS"
        reason = "all four mapping components and all five identity-bound prerequisites passed"
    record_s3_stage_transition(
        run_state, repository_name, step_status, (), reason,
        "S3.full-qualification-complete" if step_status == "PASS"
        else "S3.full-qualification-prerequisites-missing" if step_status == "BLOCKED"
        else "S3.component-or-prerequisite-contract-failure",
    )
    gate_summary = "; ".join(
        f"{name}: {gates.get(name, {}).get('status', 'NOT STARTED')} "
        f"({gates.get(name, {}).get('reason', 'no validation result')})"
        for name in S3_GATE_NAMES
    )
    append_persistent_report_entry(
        run_state,
        f"- S3 `{repository_name}` components `{json.dumps(components, sort_keys=True)}`; "
        f"qualification **{step_status}** — {reason}. Prerequisites: {gate_summary}.",
    )
    persist_run_manifest(run_state)
    return continuation_status


def validate_s3_measurement_result(summary: Mapping[str, Any], repository_name: str) -> None:
    if not isinstance(summary, Mapping):
        raise RunnerPreconditionBlockedError("S3 measurement summary is malformed")
    mapper_exit_code = summary.get("production_mapper_exit_code")
    mapper_status = summary.get("production_mapper_status")
    if mapper_status == "FAIL" or mapper_exit_code is not None and mapper_exit_code != 0:
        raise RunnerContractViolationError("untraced mapper measurement subprocess did not succeed")
    if mapper_status in {"BLOCKED", "NOT STARTED"} or mapper_exit_code is None:
        raise RunnerPreconditionBlockedError("untraced mapper measurement was not completed")
    if summary.get("production_memory_sample_error"):
        raise RunnerPreconditionBlockedError(summary["production_memory_sample_error"])
    if summary.get("production_memory_sample_count", 0) <= 0 or not isinstance(summary.get("production_peak_rss_bytes"), int):
        raise RunnerPreconditionBlockedError("whole-tree RSS sampling produced no usable measurements")
    process_audit_status = summary.get("production_process_audit_status")
    process_audit_exit = summary.get("production_process_audit_exit")
    if process_audit_status == "FAIL":
        raise RunnerContractViolationError("process access audit observed forbidden behavior")
    if process_audit_status in {"BLOCKED", "NOT STARTED"} or process_audit_exit is None:
        raise RunnerPreconditionBlockedError(
            f"process access audit is {(process_audit_status or 'NOT STARTED').lower()}"
        )
    if process_audit_exit != 0:
        raise RunnerContractViolationError("process access audit subprocess did not succeed")
    target_audit_status = summary.get("production_target_audit_status")
    target_audit_exit = summary.get("production_target_audit_exit")
    if target_audit_status == "FAIL":
        raise RunnerContractViolationError("filtered target audit observed forbidden access")
    if target_audit_status in {"BLOCKED", "NOT STARTED"} or target_audit_exit is None:
        raise RunnerPreconditionBlockedError(
            f"filtered target audit is {(target_audit_status or 'NOT STARTED').lower()}"
        )
    if target_audit_exit != 0:
        raise RunnerContractViolationError("filtered target audit subprocess did not succeed")
    publication_status = summary.get("production_publication_status")
    status_exit_code = summary.get("production_status_exit_code")
    if publication_status == "FAIL":
        raise RunnerContractViolationError("publication status command or stage validation failed")
    if publication_status in {"BLOCKED", "NOT STARTED"} or status_exit_code is None:
        raise RunnerPreconditionBlockedError(
            f"publication status is {(publication_status or 'NOT STARTED').lower()}"
        )
    if status_exit_code != 0:
        raise RunnerContractViolationError("published coverage status command did not succeed")
    if (not summary.get("production_trace_nonempty") or not summary.get("production_target_trace_nonempty")):
        raise RunnerPreconditionBlockedError("one or more process trace files are empty")
    if (summary.get("production_process_trace_complete") is not True
            or summary.get("production_target_trace_complete") is not True):
        raise RunnerPreconditionBlockedError("process or target-path trace is unfinished or incomplete")
    if summary.get("production_memory_acceptance_status") != "DEFERRED":
        raise RunnerContractViolationError("S3 must measure RSS without a premature memory acceptance gate")
    artifact_directory = Path(summary.get("production_artifact_directory", ""))
    process_audit_path = artifact_directory / "production-process-audit-result.json"
    try:
        process_audit = json.loads(process_audit_path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"process audit result is unavailable: {error}") from error
    if not process_audit.get("process_audit_pass_flag"):
        if any("incomplete trace" in value or "unfinished trace" in value
               for value in process_audit.get("process_audit_violation_rows", [])):
            raise RunnerPreconditionBlockedError("process access trace is incomplete")
        raise RunnerContractViolationError(
            f"forbidden process behavior observed for {repository_name}: "
            f"{process_audit.get('process_audit_violation_rows', [])}"
        )
    if not process_audit.get("target_filtered_audit_pass"):
        raise RunnerContractViolationError(
            f"exact-target audit reported access rows for {repository_name}: "
            f"{process_audit.get('target_filtered_violation_rows', [])}"
        )
    marker_result = summary.get("publication_stage_marker_result")
    if not isinstance(marker_result, Mapping):
        raise RunnerPreconditionBlockedError("publication stage marker evidence is missing")
    if not marker_result.get("pass"):
        reason = marker_result.get("reason") or "required stage marker did not pass"
        if "unavailable" in reason or "missing" in reason:
            raise RunnerPreconditionBlockedError(reason)
        raise RunnerContractViolationError(reason)


def read_structured_evidence_file(evidence_path: Path, evidence_name: str) -> Mapping[str, Any]:
    try:
        evidence = json.loads(Path(evidence_path).read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise RunnerPreconditionBlockedError(f"{evidence_name} file is missing or malformed: {error}") from error
    if not isinstance(evidence, Mapping):
        raise RunnerPreconditionBlockedError(f"{evidence_name} root must be a JSON object")

    def reject_command_fields(value: Any) -> None:
        if isinstance(value, Mapping):
            for key, nested_value in value.items():
                if str(key).lower() in {"command", "commands", "shell", "shell_command"}:
                    raise RunnerPreconditionBlockedError(f"{evidence_name} cannot contain arbitrary command fields")
                reject_command_fields(nested_value)
        elif isinstance(value, list):
            for nested_value in value:
                reject_command_fields(nested_value)

    reject_command_fields(evidence)
    return evidence


def write_artifact_blake3_manifest(
    artifact_run_directory: Path,
    provider: str,
    provider_identity: Mapping[str, Any] | None = None,
) -> Mapping[str, Any]:
    file_entries = []
    manifest_path = artifact_run_directory / "artifact-blake3-manifest.json"
    for file_path in sorted(path for path in artifact_run_directory.rglob("*") if path.is_file() and path != manifest_path):
        file_entries.append({
            "path": file_path.relative_to(artifact_run_directory).as_posix(),
            "size_bytes": file_path.stat().st_size,
            "blake3": calculate_b3_file_digest(file_path, provider, provider_identity=provider_identity),
        })
    manifest = {"algorithm": "blake3", "provider": provider, "files": file_entries}
    write_json_atomically(manifest_path, manifest)
    return manifest


def finalize_artifact_digest_best_effort(
    run_state: Mapping[str, Any], artifact_run_directory: Path
) -> None:
    manifest_value = run_state["manifest"]
    if manifest_value.get("execution_identity") == "HARNESS ONLY":
        return
    if not artifact_run_directory.is_dir():
        reason = "artifact directory was unavailable; no artifact digest could be written"
        manifest_value["artifact_digest_status"] = "BLOCKED"
        manifest_value["artifact_digest_reason"] = reason
        append_persistent_report_entry(run_state, f"- Artifact digest integrity: **BLOCKED** — {reason}.")
        return
    input_identity = manifest_value.get("input_identity", {})
    provider = input_identity.get("b3_provider")
    if not isinstance(provider, str) or not provider:
        reason = "frozen BLAKE3 provider identity was unavailable; preserved artifacts were not digestible"
        manifest_value["artifact_digest_status"] = "BLOCKED"
        manifest_value["artifact_digest_reason"] = reason
        append_persistent_report_entry(run_state, f"- Artifact digest integrity: **BLOCKED** — {reason}.")
        return
    try:
        artifact_manifest = write_artifact_blake3_manifest(
            artifact_run_directory, provider, input_identity.get("b3_tool_identity")
        )
    except Exception as error:
        reason = f"best-effort BLAKE3 digesting failed: {type(error).__name__}: {error}"
        manifest_value["artifact_digest_status"] = "BLOCKED"
        manifest_value["artifact_digest_reason"] = reason
        append_persistent_report_entry(run_state, f"- Artifact digest integrity: **BLOCKED** — {reason}.")
        return
    manifest_path = artifact_run_directory / "artifact-blake3-manifest.json"
    manifest_value["artifact_blake3_manifest"] = {
        "path": str(manifest_path),
        "file_count": len(artifact_manifest["files"]),
        "artifact_root": str(artifact_run_directory),
        "files": artifact_manifest["files"],
    }
    manifest_value["artifact_digest_status"] = "PASS"
    manifest_value["artifact_digest_reason"] = None
    append_persistent_report_entry(
        run_state,
        f"- Artifact digest integrity: **PASS** — manifest `{manifest_path}`; "
        f"artifact root `{artifact_run_directory}`; files `{len(artifact_manifest['files'])}`. "
        "Input-integrity status is reported separately.",
    )


def prepare_execution_context(
    arguments: argparse.Namespace,
    run_state: Mapping[str, Any],
    artifact_run_directory: Path,
) -> Mapping[str, Any]:
    corpus_manifest_identity = capture_frozen_file_identity(Path(arguments.corpus_manifest))
    if arguments.mapper_binary is None:
        raise RunnerPreconditionBlockedError("--mapper-binary is required in execute mode")
    mapper_identity = validate_canonical_runner_executable(arguments.mapper_binary)
    b3_provider = require_external_b3_digest_provider(False)
    b3_tool_identity = None
    if not b3_provider.startswith("python-blake3:"):
        b3_tool_identity = capture_frozen_file_identity(Path(b3_provider))
    indexer_path = resolve_canonical_executable_path(arguments.indexer_path)
    if not os.access(indexer_path, os.X_OK):
        raise RunnerPreconditionBlockedError(f"indexer is not executable: {indexer_path}")
    indexer_identity = dict(capture_frozen_file_identity(indexer_path))
    mapper_identity = dict(mapper_identity)
    mapper_identity["blake3"] = calculate_b3_file_digest(
        Path(mapper_identity["canonical_path"]), b3_provider, mapper_identity,
        provider_identity=b3_tool_identity,
    )
    strace_path_value = shutil.which("strace")
    if not strace_path_value:
        raise RunnerPreconditionBlockedError("strace is required for S3 process tracing")
    strace_path = Path(strace_path_value).resolve()
    strace_identity = capture_frozen_file_identity(strace_path)
    corpus_records = parse_pin_manifest_records(Path(arguments.corpus_manifest))
    root_overrides = parse_project_root_overrides(arguments.project_root)
    corpus_roots = {}
    for repository_name, record in corpus_records.items():
        root_value = root_overrides.get(repository_name, Path(record["corpus_source_checkout_path"]))
        try:
            corpus_roots[repository_name] = Path(root_value).resolve(strict=True)
        except FileNotFoundError as error:
            raise RunnerPreconditionBlockedError(f"pinned source checkout is missing for {repository_name}") from error
    required_lock_paths = [PROJECT_ROOT / "Cargo.lock"] + [root / "Cargo.lock" for root in corpus_roots.values()]
    missing_locks = [str(path) for path in required_lock_paths if not path.is_file()]
    if missing_locks:
        raise RunnerPreconditionBlockedError(f"Cargo.lock is missing before offline metadata: {missing_locks}")
    pin_summary = validate_repository_pin_manifest(
        PROJECT_ROOT,
        Path(arguments.corpus_manifest),
    )
    expected_commits = pin_summary["corpus_commit_by_repository"]
    if set(expected_commits) != set(DEFAULT_REPOSITORY_ORDER):
        raise RunnerPreconditionBlockedError("corpus manifest does not include exactly five pinned repositories")
    excluded_paths = build_owned_run_exclusions(run_state["run_directory"], artifact_run_directory)
    source_snapshots = {}
    for repository_name, source_root in corpus_roots.items():
        permitted_locks = [source_root / "Cargo.lock"] if repository_name in {"serde", "tokio"} else []
        snapshot = capture_frozen_source_snapshot(source_root, excluded_paths, permitted_locks)
        if snapshot["head_commit"] != expected_commits[repository_name]:
            raise RunnerContractViolationError(f"{repository_name} source HEAD differs from its pinned commit")
        source_snapshots[repository_name] = snapshot
    root_snapshot = capture_frozen_source_snapshot(PROJECT_ROOT, excluded_paths)
    if root_snapshot["head_commit"] != run_read_only_git_command(PROJECT_ROOT, ["rev-parse", "HEAD"]).strip():
        raise RunnerContractViolationError("runner repository HEAD changed while freezing source inputs")

    manifest_paths = [
        PROJECT_ROOT / "Cargo.toml",
        PROJECT_ROOT / "crates/parseltongue-cli-dispatcher-binary/Cargo.toml",
        PROJECT_ROOT / "crates/parseltongue-cli-dispatcher-binary/src/main.rs",
        Path(arguments.corpus_manifest),
        PROJECT_ROOT / "docs/phase-20261111/parser-source-pin-manifest.json",
        PROJECT_ROOT / "docs/phase-20261111/oracle-source-build-manifest.json",
        PROJECT_ROOT / "docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md",
        PROJECT_ROOT / "docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md",
        Path(pin_summary["r1_patch_path"]),
        PROJECT_ROOT / "tools/hld-current-qualification-runner.py",
        PROJECT_ROOT / "tools/corpus-index-coverage-inventory.py",
        PROJECT_ROOT / "tools/corpus-production-stage-measurements.py",
        PROJECT_ROOT / "tools/process-tree-access-auditor.py",
    ]
    for evidence_path in (arguments.r4_evidence, arguments.r5_evidence):
        if evidence_path is not None:
            resolved_evidence_path = Path(evidence_path).resolve(strict=True)
            if not resolved_evidence_path.is_file():
                raise RunnerPreconditionBlockedError(
                    f"structured evidence file is not a file: {resolved_evidence_path}"
                )
            manifest_paths.append(resolved_evidence_path)
    parser_manifest = json.loads((PROJECT_ROOT / "docs/phase-20261111/parser-source-pin-manifest.json").read_text())
    parser_bundle_path = PROJECT_ROOT / parser_manifest["parser_source_bundle_path"]
    if parser_bundle_path.is_file():
        manifest_paths.append(parser_bundle_path)
    if b3_provider.startswith("python-blake3:"):
        try:
            import blake3
        except ImportError as error:
            raise RunnerPreconditionBlockedError("Python BLAKE3 provider disappeared during preflight") from error
        if getattr(blake3, "__file__", None):
            manifest_paths.append(Path(blake3.__file__).resolve())
    elif b3_tool_identity is not None:
        manifest_paths.append(Path(b3_tool_identity["path"]))
    evidence_environment = capture_s3_evidence_environment()
    toolchain_versions = probe_toolchain_versions()
    toolchain_executable_identities = [
        value["file_identity"] for value in toolchain_versions.values()
        if isinstance(value, Mapping) and isinstance(value.get("file_identity"), Mapping)
    ]
    p2_configuration_reason = None
    if any(evidence_environment.get(name) for name in (
        "CARGO_BUILD_TARGET", "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER", "RUSTC",
    )) or any(name.startswith("CARGO_TARGET_") and name != "CARGO_TARGET_DIR" and value
              for name, value in evidence_environment.items()) or any(
                  name.startswith("CARGO_PROFILE_") and value for name, value in evidence_environment.items()
              ):
        p2_configuration_reason = "environment supplies unsupported P2 target/cfg/compiler wrapper overrides"
    elif any(re.search(r"(?:SKIP|FILTER|IGNORE|NO_NATIVE)", name, re.IGNORECASE) and value
             for name, value in evidence_environment.items()):
        p2_configuration_reason = "environment supplies an unsupported native test skip/filter override"
    cargo_configuration_paths = find_cargo_configuration_files(PROJECT_ROOT, corpus_roots, evidence_environment)
    cargo_configuration_candidate_paths = list(cargo_configuration_paths)
    cargo_configuration_initial_identities = {}
    absent_cargo_configuration_identities = {}
    present_cargo_configuration_paths = []
    for config_path in cargo_configuration_paths:
        try:
            config_identity = capture_optional_frozen_file_identity(config_path)
        except RunnerPreconditionBlockedError as error:
            p2_configuration_reason = p2_configuration_reason or str(error)
            continue
        if config_identity["exists"]:
            present_cargo_configuration_paths.append(config_path)
            cargo_configuration_initial_identities[config_identity["path"]] = {
                key: value for key, value in config_identity.items() if key != "exists"
            }
        else:
            absent_cargo_configuration_identities[config_identity["path"]] = config_identity
    p2_configuration_reason = p2_configuration_reason or cargo_configuration_override_reason(cargo_configuration_paths)
    manifest_paths.extend(present_cargo_configuration_paths)
    evidence_inputs = read_s3_evidence_inputs(
        getattr(arguments, "s3_prerequisite_evidence", None), b3_provider, b3_tool_identity,
        forbidden_roots=[PROJECT_ROOT, *corpus_roots.values(), Path(run_state["run_directory"]),
                         artifact_run_directory],
    )
    evidence_inputs["optional_paths"].extend(
        Path(path) for path in absent_cargo_configuration_identities
    )
    evidence_inputs["optional_identities"].update(absent_cargo_configuration_identities)
    expected_frozen_files = {
        **cargo_configuration_initial_identities,
        **evidence_inputs["frozen_identities"],
    }
    registry = capture_identity_registry(
        PROJECT_ROOT,
        manifest_paths,
        corpus_roots,
        excluded_paths,
        [mapper_identity, indexer_identity, strace_identity, *toolchain_executable_identities,
         *([b3_tool_identity] if b3_tool_identity else [])],
        extra_frozen_paths=evidence_inputs["frozen_paths"],
        optional_frozen_paths=evidence_inputs["optional_paths"],
        expected_frozen_files=expected_frozen_files,
        expected_optional_files=evidence_inputs["optional_identities"],
    )
    if registry["frozen_files"].get(corpus_manifest_identity["path"]) != corpus_manifest_identity:
        raise RunnerContractViolationError("corpus pin manifest changed while freezing execution inputs")
    for repository_name, expected_snapshot in source_snapshots.items():
        if registry["source_snapshots"][str(corpus_roots[repository_name])] != expected_snapshot:
            raise RunnerContractViolationError(f"source changed while freezing {repository_name}")
    if registry["source_snapshots"][str(PROJECT_ROOT.resolve())] != root_snapshot:
        raise RunnerContractViolationError("runner source changed while freezing repository inputs")
    expected_identity = build_s3_expected_identity(
        arguments, run_state, mapper_identity, pin_summary, registry, toolchain_versions,
        cargo_configuration_candidate_paths, p2_configuration_reason, evidence_environment,
    )
    expected_repository_bindings = build_s3_repository_bindings(
        corpus_records, corpus_roots, source_snapshots, registry["lock_files"], p2_configuration_reason,
    )
    source_test_inventory = derive_production_source_test_inventory(PROJECT_ROOT)
    if any(value.get("version") in {None, "", "unavailable"} or not isinstance(value.get("file_identity"), Mapping)
           for value in toolchain_versions.values() if isinstance(value, Mapping)):
        evidence_inputs["intake_error"] = merge_s3_intake_error(
            evidence_inputs.get("intake_error"),
            ("BLOCKED", "current rustc/cargo version could not be established for S3 evidence binding"),
        )
    identity_guard = make_identity_boundary_callback(
        registry, evidence_environment, mapper_identity, b3_provider, b3_tool_identity,
    )
    identity_guard("before S3 prerequisite evidence validation", [])
    dependency_reference = collect_dependency_tree_reference(
        evidence_inputs, expected_identity, registry, identity_guard,
        artifact_run_directory, b3_provider, b3_tool_identity,
    )
    s3_evidence = prepare_s3_evidence_qualification(
        arguments, run_state, evidence_inputs, expected_identity,
        expected_repository_bindings, source_test_inventory, dependency_reference,
    )
    identity_guard("after S3 prerequisite evidence validation", [])
    manifest_value = run_state["manifest"]
    manifest_value["input_identity"] = {
        "mapper": mapper_identity,
        "indexer": indexer_identity,
        "strace": strace_identity,
        "r1_patch_sha256": pin_summary["r1_patch_sha256"],
        "repository_source_snapshots": source_snapshots,
        "runner_source_snapshot": root_snapshot,
        "cargo_lock_files": registry["lock_files"],
        "frozen_manifest_paths": list(registry["frozen_files"]),
        "frozen_file_identities": registry["frozen_files"],
        "b3_provider": b3_provider,
        "b3_tool_identity": b3_tool_identity,
        "historical_corpus_check_exit_codes_used_as_qualification": False,
    }
    manifest_value["runtime_versions"] = {
        "python": sys.version,
        "platform": platform.platform(),
        "machine": platform.machine(),
        "mapper_identity": f"sha256:{mapper_identity['sha256']}",
        "toolchain_versions": toolchain_versions,
    }
    manifest_value["configuration"] = {
        "milestone": arguments.milestone,
        "mode": arguments.mode,
        "corpus_manifest": str(Path(arguments.corpus_manifest).resolve()),
        "report_root": str(Path(arguments.report_root).expanduser().absolute()),
        "artifact_root": str(Path(arguments.artifact_root).expanduser().absolute()),
        "project_root_overrides": [str(value) for value in arguments.project_root],
        "indexer_path": str(indexer_path),
    }
    manifest_value["artifact_run_directory"] = str(artifact_run_directory)
    manifest_value["input_integrity_status"] = "PASS"
    append_frozen_identity_report(run_state)
    persist_run_manifest(run_state)
    return {
        "mapper_identity": mapper_identity,
        "b3_tool_identity": b3_tool_identity,
        "indexer_path": indexer_path,
        "indexer_identity": indexer_identity,
        "strace_path": strace_path,
        "b3_provider": b3_provider,
        "pin_summary": pin_summary,
        "corpus_records": corpus_records,
        "corpus_roots": corpus_roots,
        "source_snapshots": source_snapshots,
        "registry": registry,
        "identity_guard": identity_guard,
        "s3_evidence": s3_evidence,
        "s3_expected_identity": expected_identity,
        "s3_repository_bindings": expected_repository_bindings,
        "s3_source_test_inventory": source_test_inventory,
        "artifact_run_directory": artifact_run_directory,
        "report_run_directory": Path(run_state["run_directory"]),
    }


def run_fresh_index_stage(
    repository_name: str,
    source_root: Path,
    indexer_path: Path,
    mapper_identity: Mapping[str, Any],
    identity_guard: Callable[[str, Sequence[str]], None],
    artifact_repository_directory: Path,
    command_event_callback: Callable[[str, Sequence[str], Mapping[str, Any]], None] | None = None,
) -> Mapping[str, Any]:
    command = [str(indexer_path), "cli", "index_repository", "--repo-path", str(source_root), "--mode", "full"]
    if command_event_callback is not None:
        command_event_callback("before", command, {"stage": "S1", "status": "STARTED"})
    try:
        result = run_guarded_child_process(command, mapper_identity, identity_guard)
    except Exception as error:
        if command_event_callback is not None:
            command_event_callback("after", command, {
                "stage": "S1", "status": "ERROR", "exception_type": type(error).__name__,
            })
        raise
    if command_event_callback is not None:
        stdout_bytes = result.get("stdout_bytes")
        stderr_bytes = result.get("stderr_bytes")
        if stdout_bytes is None:
            stdout_bytes = str(result.get("stdout", "")).encode("utf-8")
        if stderr_bytes is None:
            stderr_bytes = str(result.get("stderr", "")).encode("utf-8")
        command_event_callback("after", command, {
            "stage": "S1",
            "status": "COMPLETED" if result.get("return_code") == 0 else "FAILED",
            "return_code": result.get("return_code"),
            "stdout_bytes": len(stdout_bytes),
            "stderr_bytes": len(stderr_bytes),
        })
    response_path = artifact_repository_directory / "codebase-index-result-log.txt"
    response_path.parent.mkdir(parents=True, exist_ok=True)
    if response_path.exists():
        raise FileExistsError(f"refusing to overwrite fresh command output {response_path}")
    response_bytes = result.get("stdout_bytes", result["stdout"].encode())
    response_path.write_bytes(response_bytes)
    if result["return_code"] != 0:
        raise RunnerContractViolationError(
            f"fresh full index command failed for {repository_name}: {result['stderr'].strip()}"
        )
    try:
        index_result = json.loads(result["stdout"])
    except json.JSONDecodeError as error:
        raise RunnerPreconditionBlockedError(f"fresh index output is malformed for {repository_name}") from error
    if not isinstance(index_result, Mapping):
        raise RunnerPreconditionBlockedError("fresh index output must be a JSON object")
    if index_result.get("status") != "indexed":
        raise RunnerContractViolationError(f"fresh full index did not succeed for {repository_name}")
    return {"command": command, "result": index_result}


def create_external_command_event_recorder(command_journal_path: Path):
    command_journal_path = Path(command_journal_path)
    command_journal_path.parent.mkdir(parents=True, exist_ok=True)
    if command_journal_path.exists():
        raise RunnerPreconditionBlockedError(f"refusing to reuse command journal {command_journal_path}")
    event_count = 0
    events = []

    def record_command_event(phase, command_values, outcome):
        nonlocal event_count
        if phase not in {"before", "after"}:
            raise RunnerContractViolationError(f"invalid command-journal phase: {phase}")
        event = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "phase": phase,
            "command": [str(value) for value in command_values],
            "cwd": str(Path.cwd()),
            "outcome": dict(outcome),
        }
        serialized = json.dumps(event, sort_keys=True, separators=(",", ":")) + "\n"
        serialized_bytes = serialized.encode("utf-8")
        if len(serialized_bytes) > MAX_COMMAND_EVENT_BYTES:
            raise RunnerPreconditionBlockedError("exact command event exceeds the bounded record size")
        current_size = command_journal_path.stat().st_size if command_journal_path.exists() else 0
        if (event_count >= MAX_COMMAND_EVENT_COUNT
                or current_size + len(serialized_bytes) > MAX_COMMAND_JOURNAL_BYTES):
            raise RunnerPreconditionBlockedError("S1/S2 command journal exceeded its bounded capture")
        with command_journal_path.open("a", encoding="utf-8") as journal:
            journal.write(serialized)
        event_count += 1
        events.append(event)

    record_command_event.events = events
    return record_command_event


def collect_transition_command_metadata(command_event_callback, step_name):
    command_events = getattr(command_event_callback, "events", ())
    return [
        {
            "argv": list(event["command"]),
            "cwd": event["cwd"],
            "exit_result": {
                "return_code": event["outcome"].get("return_code"),
                "status": event["outcome"].get("status"),
                "exception_type": event["outcome"].get("exception_type"),
            },
        }
        for event in command_events
        if event["phase"] == "after" and event.get("outcome", {}).get("stage") == step_name
    ]


def execute_repository_sop_stages(
    repository_name: str,
    run_state: Mapping[str, Any],
    execution_context: Mapping[str, Any],
    arguments: argparse.Namespace,
) -> str:
    execution_context["identity_guard"]("before S1", [])
    report_repository_directory = Path(run_state["run_directory"]) / repository_name
    report_repository_directory.mkdir(parents=True, exist_ok=False)
    artifact_repository_directory = Path(execution_context["artifact_run_directory"]) / repository_name
    artifact_repository_directory.mkdir(parents=True, exist_ok=False)
    command_journal_path = artifact_repository_directory / "commands.jsonl"
    command_event_callback = create_external_command_event_recorder(command_journal_path)
    source_root = execution_context["corpus_roots"][repository_name]
    current_step = "S1"
    current_commands: list[str] = []
    metadata_complete = False

    def transition_command_evidence():
        command_metadata = collect_transition_command_metadata(command_event_callback, current_step)
        command_values = [json.dumps(value["argv"]) for value in command_metadata] or current_commands
        return command_values, command_metadata or None

    try:
        current_commands = [json.dumps([
            str(execution_context["indexer_path"]), "cli", "index_repository",
            "--repo-path", str(source_root), "--mode", "full",
        ])]
        index_result = run_fresh_index_stage(
            repository_name,
            source_root,
            execution_context["indexer_path"],
            execution_context["mapper_identity"],
            execution_context["identity_guard"],
            artifact_repository_directory,
            command_event_callback,
        )
        current_commands = index_result["command"]
        source_snapshot = execution_context["source_snapshots"][repository_name]
        (report_repository_directory / "index-input-binding.json").write_text(json.dumps({
            "repository_name": repository_name,
            "source_head_commit": source_snapshot["head_commit"],
            "source_tree_sha256": source_snapshot["source_tree_sha256"],
            "index_result_status": index_result["result"].get("status"),
        }, indent=2) + "\n")
        current_step = "S1"
        current_commands = [json.dumps(index_result["command"])]

        def record_index_metadata_complete(metadata_result):
            nonlocal current_step, current_commands, metadata_complete
            s1_command_metadata = collect_transition_command_metadata(command_event_callback, "S1")
            current_commands = [json.dumps(value["argv"]) for value in s1_command_metadata]
            execution_context["identity_guard"]("after S1", [])
            record_persistent_run_transition(
                run_state, repository_name, "S1", "PASS", current_commands,
                f"fresh full generation {metadata_result['generation']} has matching status/schema metadata",
                "S1.fresh-full-index-and-metadata",
                command_metadata=s1_command_metadata,
            )
            metadata_complete = True
            current_step = "S2"
            current_commands = []
            execution_context["identity_guard"]("before S2", [])

        def record_s2_command_event(phase, command_values, outcome):
            command_stage = "S2" if metadata_complete else "S1"
            command_event_callback(phase, command_values, {"stage": command_stage, **outcome})

        coverage_summary = coverage_inventory.collect_index_coverage_inventory(
            repository_name,
            report_repository_directory,
            str(execution_context["indexer_path"]),
            derive_indexer_project_identifier(source_root),
            index_result["result"].get("indexed_at"),
            execution_context["identity_guard"],
            record_index_metadata_complete,
            artifact_repository_directory,
            lambda file_path: calculate_b3_file_digest(
                file_path,
                execution_context["b3_provider"],
                execution_context["mapper_identity"],
                execution_context["identity_guard"],
                execution_context["b3_tool_identity"],
            ),
            command_event_callback=record_s2_command_event,
        )
        if coverage_summary.get("scope_literal_sop_pass") is not True:
            raise RunnerPreconditionBlockedError("S2 full pagination result did not pass strict validation")
        execution_context["identity_guard"]("after S2", [])
        run_state["manifest"]["repositories"][repository_name]["coverage_summary"] = coverage_summary
        coverage_summary["commands_jsonl_path"] = str(command_journal_path.resolve())
        run_state["manifest"]["repositories"][repository_name]["coverage_summary"] = coverage_summary
        s2_command_metadata = collect_transition_command_metadata(command_event_callback, "S2")
        current_commands = [json.dumps(value["argv"]) for value in s2_command_metadata]
        record_persistent_run_transition(
            run_state, repository_name, "S2", "PASS", current_commands,
            f"scope entries={coverage_summary['scope_coverage_entry_count']}; "
            f"missed File rows={coverage_summary['missed_source_file_count']}; "
            f"generation={coverage_summary['index_generation_timestamp_value']}",
            "S2.complete-stable-pagination",
            command_metadata=s2_command_metadata,
        )
        append_persistent_report_entry(
            run_state,
            f"- `{repository_name}` index generation `{coverage_summary['index_generation_timestamp_value']}`; "
            f"nodes `{coverage_summary['index_graph_node_count']}`, edges `{coverage_summary['index_graph_edge_count']}`; "
            f"partial files `{format_report_value(coverage_summary.get('index_partial_file_count'))}`, "
            f"unusable files `{format_report_value(coverage_summary.get('index_unusable_file_count'))}`; "
            f"scope entries `{coverage_summary['scope_coverage_entry_count']}`, "
            f"missed File rows `{coverage_summary['missed_source_file_count']}`; "
            f"scope truncated informational `{coverage_summary['scope_truncated_informational']}`; "
            f"raw response digest manifest `{coverage_summary['response_digest_manifest_path']}` "
            f"(BLAKE3; artifact root `{artifact_repository_directory}`).",
        )
        append_persistent_report_entry(
            run_state,
            f"- `{repository_name}` exact S1/S2 argv and before/after outcomes: "
            f"external commands JSONL `{command_journal_path.resolve()}` (BLAKE3-bound in final artifacts).",
        )

        current_step = "S3"
        execution_context["identity_guard"]("before S3 prerequisite gate", [])
        evidence_context = execution_context.get("s3_evidence", {})
        evidence_results = evidence_context.get("repository_results", {}).get(repository_name, {})
        repository_state = run_state["manifest"]["repositories"][repository_name]
        repository_state["s3_qualification_gates"] = {
            gate_name: dict(evidence_results.get(gate_name, {
                "status": "BLOCKED", "reason": "S3 prerequisite evidence was not validated for this repository",
            }))
            for gate_name in S3_GATE_NAMES
        }
        failing_gate = next((name for name in S3_GATE_NAMES
                             if repository_state["s3_qualification_gates"][name]["status"] == "FAIL"), None)
        incomplete_gates = [name for name in S3_GATE_NAMES
                            if repository_state["s3_qualification_gates"][name]["status"] != "PASS"]
        intake_status = evidence_context.get("intake_status", {}).get("status", "BLOCKED")
        qualification_token = evidence_context.get("tokens_by_repository", {}).get(repository_name)
        expected_identity = execution_context.get("s3_expected_identity", evidence_context.get("identity_binding"))
        token_is_valid = False
        if isinstance(qualification_token, s3_evidence_intake.S3EvidenceQualificationToken) and isinstance(
            expected_identity, Mapping
        ):
            expected_digest = hashlib.sha256(json.dumps(
                expected_identity, sort_keys=True, separators=(",", ":"),
            ).encode()).hexdigest()
            token_is_valid = (
                qualification_token.run_identifier == run_state["manifest"].get("run_id")
                and qualification_token.repository_name == repository_name
                and qualification_token.identity_digest == expected_digest
            )
        if failing_gate is not None or intake_status == "FAIL":
            gate_status = "FAIL"
        elif incomplete_gates or intake_status != "PASS" or not token_is_valid:
            gate_status = "BLOCKED"
        else:
            gate_status = "PASS"
        execution_context["identity_guard"]("after S3 prerequisite gate", [])
        if gate_status != "PASS":
            gate_reason = (
                f"S3 prerequisite {failing_gate} failed"
                if failing_gate is not None else
                "S3 evidence intake failed" if intake_status == "FAIL" else
                f"S3 prerequisites are incomplete: {incomplete_gates}; "
                f"intake={intake_status}; internal_token_valid={token_is_valid}"
            )
            record_s3_stage_transition(
                run_state, repository_name, gate_status, (), gate_reason,
                "S3.prerequisite-evidence-failed" if failing_gate else "S3.prerequisite-evidence-missing",
            )
            append_persistent_report_entry(
                run_state,
                f"- S3 `{repository_name}` prerequisite results "
                f"`{json.dumps(repository_state['s3_qualification_gates'], sort_keys=True)}`; "
                f"qualification **{gate_status}** — stopped before mapper/output creation.",
            )
            persist_run_manifest(run_state)
            return gate_status
        final_stage = "syntax-spans" if arguments.milestone == "M1" else "name-resolution"
        production_output_directory = artifact_repository_directory / "production"
        production_output_directory.mkdir(parents=False, exist_ok=False)
        measurement_record = dict(execution_context["corpus_records"][repository_name])
        measurement_record["corpus_source_checkout_path"] = str(source_root)
        measurement_commands = production_measurements.build_corpus_measurement_commands(
            measurement_record,
            production_output_directory,
            Path(execution_context["mapper_identity"]["canonical_path"]),
            final_stage,
        )
        current_commands = [json.dumps(command) for command in measurement_commands]
        current_commands.append(json.dumps([
            execution_context["mapper_identity"]["canonical_path"], "status-coverage-banner-show", "--json",
            "--workspace-store-file-path", str(production_output_directory / "production-untraced-workspace-store.db"),
        ]))
        measurement_summary = production_measurements.measure_corpus_production_stages(
            measurement_record,
            production_output_directory,
            Path(execution_context["mapper_identity"]["canonical_path"]),
            execution_context["identity_guard"],
            final_stage,
        )
        production_summary_path = report_repository_directory / "production-stage-measurement-summary.json"
        if production_summary_path.exists():
            raise RunnerPreconditionBlockedError(f"refusing to overwrite production summary {production_summary_path}")
        production_summary_path.write_text(json.dumps(measurement_summary, indent=2) + "\n")
        run_state["manifest"]["repositories"][repository_name]["s3_measurements"] = measurement_summary
        repository_components = run_state["manifest"]["repositories"][repository_name]["s3_components"]
        mapper_status = measurement_summary.get("production_mapper_status")
        mapper_exit = measurement_summary.get("production_mapper_exit_code")
        if mapper_status in {"PASS", "FAIL", "BLOCKED", "NOT STARTED"}:
            repository_components["mapping"] = mapper_status
        elif mapper_exit is None:
            repository_components["mapping"] = "NOT STARTED"
        else:
            repository_components["mapping"] = "PASS" if mapper_exit == 0 else "FAIL"
        measurement_status = measurement_summary.get("production_measurement_status")
        if measurement_status in {"PASS", "FAIL", "BLOCKED", "NOT STARTED"}:
            repository_components["measurement"] = measurement_status
        elif mapper_exit is None:
            repository_components["measurement"] = "NOT STARTED"
        elif (measurement_summary.get("production_memory_sample_count", 0) > 0
              and isinstance(measurement_summary.get("production_peak_rss_bytes"), int)
              and isinstance(measurement_summary.get("production_total_elapsed_seconds"), (int, float))):
            repository_components["measurement"] = "PASS"
        else:
            repository_components["measurement"] = "BLOCKED"
        process_audit_status = measurement_summary.get("production_process_audit_status")
        target_audit_status = measurement_summary.get("production_target_audit_status")
        process_audit_exit = measurement_summary.get("production_process_audit_exit")
        target_audit_exit = measurement_summary.get("production_target_audit_exit")
        if process_audit_status in {"FAIL", "BLOCKED"}:
            repository_components["process_access_audit"] = process_audit_status
        elif target_audit_status in {"FAIL", "BLOCKED"}:
            repository_components["process_access_audit"] = target_audit_status
        elif (process_audit_status == "NOT STARTED" or target_audit_status == "NOT STARTED"
              or process_audit_status is None and process_audit_exit is None
              or target_audit_status is None and target_audit_exit is None):
            repository_components["process_access_audit"] = "NOT STARTED"
        elif process_audit_exit is not None and process_audit_exit != 0 or (
            target_audit_exit is not None and target_audit_exit != 0
        ):
            repository_components["process_access_audit"] = "FAIL"
        elif (process_audit_status == "PASS" and target_audit_status == "PASS"
              and process_audit_exit == 0 and target_audit_exit == 0):
            repository_components["process_access_audit"] = "PASS"
        elif process_audit_exit is None or target_audit_exit is None:
            repository_components["process_access_audit"] = "BLOCKED"
        elif (not measurement_summary.get("production_trace_nonempty")
                or not measurement_summary.get("production_process_trace_complete")
                or not measurement_summary.get("production_target_trace_nonempty")
                or not measurement_summary.get("production_target_trace_complete")):
            repository_components["process_access_audit"] = "BLOCKED"
        elif (measurement_summary.get("production_process_audit_pass") is False
              or measurement_summary.get("production_target_audit_pass") is False):
            repository_components["process_access_audit"] = "FAIL"
        else:
            repository_components["process_access_audit"] = "PASS"
        marker_result = measurement_summary.get("publication_stage_marker_result")
        publication_status = measurement_summary.get("production_publication_status")
        if publication_status in {"PASS", "BLOCKED", "FAIL", "NOT STARTED"}:
            repository_components["publication"] = publication_status
        elif measurement_summary.get("production_status_exit_code") is None:
            repository_components["publication"] = "NOT STARTED"
        elif measurement_summary.get("production_status_exit_code") != 0:
            repository_components["publication"] = "FAIL"
        elif isinstance(marker_result, Mapping) and marker_result.get("pass") is True:
            repository_components["publication"] = "PASS"
        elif isinstance(marker_result, Mapping) and any(
            token in str(marker_result.get("reason", "")).lower() for token in ("missing", "unavailable")
        ):
            repository_components["publication"] = "BLOCKED"
        else:
            repository_components["publication"] = "FAIL"
        persist_run_manifest(run_state)
        validate_s3_measurement_result(measurement_summary, repository_name)
        append_persistent_report_entry(
            run_state,
            f"- `{repository_name}` sampled wall `{format_report_value(measurement_summary.get('production_total_elapsed_seconds'))}` s, "
            f"whole-tree RSS `{format_report_value(measurement_summary.get('production_peak_rss_bytes'))}` bytes; "
            f"published generation `{format_report_value((measurement_summary.get('production_coverage_banner_values') or {}).get('run_generation_digest_value'))}`; "
            f"stage counts `{format_report_value((measurement_summary.get('production_coverage_banner_values') or {}).get('stage_counts'))}`. "
            "RSS is nominal 10-ms sampled, not an exact peak; target filter is exact-path-only, not recursive; "
            "memory acceptance remains DEFERRED.",
        )
        execution_context["identity_guard"]("after S3", [])
        return finalize_s3_component_qualification(
            run_state, repository_name,
            execution_context.get("s3_evidence", {}).get("tokens_by_repository", {}).get(repository_name),
            execution_context.get("s3_expected_identity"),
        )
    except RunnerFrozenInputMutationError as error:
        run_state["manifest"]["input_integrity_status"] = "FAIL"
        run_state["manifest"]["input_integrity_reason"] = str(error)
        transition_commands, transition_metadata = transition_command_evidence()
        record_persistent_run_transition(
            run_state, repository_name, current_step, "FAIL", transition_commands, str(error),
            f"{current_step}.frozen-input-mutation", command_metadata=transition_metadata,
        )
        return "FAIL"
    except RunnerContractViolationError as error:
        transition_commands, transition_metadata = transition_command_evidence()
        record_persistent_run_transition(
            run_state, repository_name, current_step, "FAIL", transition_commands, str(error),
            f"{current_step}.observed-contract-violation", command_metadata=transition_metadata,
        )
        return "FAIL"
    except (RunnerPreconditionBlockedError, coverage_inventory.CoverageInventoryBlockedError) as error:
        if current_step == "S1" and run_state["manifest"]["repositories"][repository_name]["steps"]["S1"] == "PASS":
            current_step = "S2"
        transition_commands, transition_metadata = transition_command_evidence()
        record_persistent_run_transition(
            run_state, repository_name, current_step, "BLOCKED", transition_commands, str(error),
            f"{current_step}.missing-or-incomplete-evidence", command_metadata=transition_metadata,
        )
        return "BLOCKED"
    except KeyboardInterrupt:
        transition_commands, transition_metadata = transition_command_evidence()
        record_persistent_run_transition(
            run_state, repository_name, current_step, "BLOCKED", transition_commands,
            "execution interrupted before the current stage completed", f"{current_step}.interrupted",
            command_metadata=transition_metadata,
        )
        raise
    except Exception as error:
        transition_commands, transition_metadata = transition_command_evidence()
        record_persistent_run_transition(
            run_state, repository_name, current_step, "BLOCKED", transition_commands,
            f"unexpected {type(error).__name__}: {error}", f"{current_step}.runner-exception",
            command_metadata=transition_metadata,
        )
        return "BLOCKED"


def validate_external_m2_evidence(
    run_state: Mapping[str, Any], arguments: argparse.Namespace, execution_context: Mapping[str, Any]
) -> None:
    if arguments.milestone != "M2":
        return
    evidence_by_type = {}
    evidence_load_errors = {}
    for evidence_kind, evidence_path in (("R4", arguments.r4_evidence), ("R5", arguments.r5_evidence)):
        if evidence_path is None:
            evidence_by_type[evidence_kind] = None
        else:
            try:
                evidence_by_type[evidence_kind] = read_structured_evidence_file(evidence_path, evidence_kind)
            except RunnerPreconditionBlockedError as error:
                evidence_by_type[evidence_kind] = None
                evidence_load_errors[evidence_kind] = str(error)
    for repository_name in DEFAULT_REPOSITORY_ORDER:
        expected_identity = {
            "run_id": run_state["manifest"]["run_id"],
            "repository_name": repository_name,
            "mapper_sha256": execution_context["mapper_identity"]["sha256"],
            "input_digest": execution_context["source_snapshots"][repository_name]["source_tree_sha256"],
            "artifact_root": str(Path(execution_context["artifact_run_directory"]) / repository_name),
        }
        for evidence_kind, validator in (("R4", validate_m2_duckdb_handoff), ("R5", validate_m2_path_partition)):
            root_evidence = evidence_by_type[evidence_kind]
            repositories = root_evidence.get("repositories") if isinstance(root_evidence, Mapping) else None
            evidence = repositories.get(repository_name) if isinstance(repositories, Mapping) else None
            if not isinstance(repositories, Mapping):
                result = {
                    "status": "BLOCKED",
                    "validator_component_status": "BLOCKED",
                    "reason": evidence_load_errors.get(
                        evidence_kind, f"{evidence_kind} evidence has no repositories object"
                    ),
                }
            elif evidence is None:
                result = {
                    "status": "BLOCKED",
                    "validator_component_status": "BLOCKED",
                    "reason": evidence_load_errors.get(
                        evidence_kind, f"{evidence_kind} evidence not supplied for {repository_name}"
                    ),
                }
            else:
                try:
                    validator_result = validator(evidence, expected_identity)
                except RunnerPreconditionBlockedError as error:
                    validator_result = {"status": "BLOCKED", "reason": str(error)}
                except RunnerContractViolationError as error:
                    validator_result = {"status": "FAIL", "reason": str(error)}
                corpus_status = "FAIL" if validator_result["status"] == "FAIL" else "BLOCKED"
                result = {
                    "status": corpus_status,
                    "validator_component_status": validator_result["status"],
                    "validator_component_reason": validator_result.get("reason"),
                    "validator_component_result": validator_result,
                    "reason": (
                        f"validator component {validator_result['status']}; authoritative {evidence_kind} "
                        "remains BLOCKED pending independent exported-table/primary-JSONL verification, "
                        "complete required classes, generation binding, and analysis-order evidence"
                    ),
                    "blocked_gates": [
                        "independent_artifact_verification", "complete_required_classes",
                        "generation_binding", "analysis_order",
                    ],
                }
            run_state["manifest"]["external_evidence"][evidence_kind][repository_name] = result
            repository_state = run_state["manifest"]["repositories"][repository_name]
            if result["validator_component_status"] != "PASS" and repository_state["first_failed_clause"] is None:
                repository_state["first_failed_clause"] = f"{evidence_kind}.validator-component"
                repository_state["first_reason"] = result.get("validator_component_reason") or result["reason"]
            append_persistent_report_entry(
                run_state,
                f"- {evidence_kind} `{repository_name}` validator component "
                f"**{result['validator_component_status']}**; component details "
                f"`{json.dumps(result.get('validator_component_result', {}), sort_keys=True)}`; "
                f"corpus gate **{result['status']}** — {result['reason']}.",
            )
    persist_run_manifest(run_state)


def finalize_unimplemented_milestone_gates(
    run_state: Mapping[str, Any], execution_context: Mapping[str, Any], arguments: argparse.Namespace
) -> None:
    repositories = run_state["manifest"]["repositories"]
    all_components_complete = all(
        state["steps"]["S1"] == "PASS" and state["steps"]["S2"] == "PASS"
        and all(status == "PASS" for status in state["s3_components"].values())
        for state in repositories.values()
    )
    if arguments.milestone == "M1":
        record_persistent_gate_transition(
            run_state, "S10", "BLOCKED",
            "S10 evidence is a separate handoff; mapping/audit components and full S3 do not establish S10.",
        )
    else:
        if all_components_complete:
            validate_external_m2_evidence(run_state, arguments, execution_context)
        else:
            for evidence_kind in ("R4", "R5"):
                for repository_name in DEFAULT_REPOSITORY_ORDER:
                    run_state["manifest"]["external_evidence"][evidence_kind][repository_name] = {
                        "status": "BLOCKED",
                        "validator_component_status": "BLOCKED",
                        "reason": "S1-S3 components did not complete for all five repositories",
                    }
        r4_components = {
            name: item.get("validator_component_status", "BLOCKED")
            for name, item in run_state["manifest"]["external_evidence"]["R4"].items()
        }
        r5_components = {
            name: item.get("validator_component_status", "BLOCKED")
            for name, item in run_state["manifest"]["external_evidence"]["R5"].items()
        }
        s4_status = "FAIL" if "FAIL" in r4_components.values() else "BLOCKED"
        s5_status = "FAIL" if "FAIL" in r5_components.values() else "BLOCKED"
        s4_reason = (
            "R4 validator observed a contract violation, so S4 is FAIL; independent authoritative table "
            "verification, complete required classes, generation binding, and analysis-order proof remain unverified; "
            f"R4 validator components: {r4_components}."
            if s4_status == "FAIL" else
            "S4 remains BLOCKED pending independent authoritative exported-table verification, complete required "
            "classes, mapper-generation binding, and proof the handoff preceded analysis; "
            f"R4 validator components: {r4_components}."
        )
        s5_reason = (
            "R5 validator observed a contract violation, so S5 is FAIL; authoritative primary-JSONL verification "
            "and the all-five gate remain unverified; "
            f"R5 validator components: {r5_components}."
            if s5_status == "FAIL" else
            "S5 remains BLOCKED pending authoritative primary-JSONL verification and the all-five gate; "
            f"R5 validator components: {r5_components}."
        )
        record_persistent_gate_transition(
            run_state, "S4", s4_status, s4_reason,
        )
        record_persistent_gate_transition(
            run_state, "S5", s5_status, s5_reason,
        )


def execute_qualification_run(
    arguments: argparse.Namespace, run_state: Mapping[str, Any]
) -> Mapping[str, Any]:
    artifact_run_directory = Path(arguments.artifact_root).expanduser().absolute() / arguments.run_id
    try:
        artifact_run_directory.mkdir(parents=True, exist_ok=False)
        run_state["manifest"]["artifact_run_directory"] = str(artifact_run_directory)
        persist_run_manifest(run_state)
        if arguments.harness_only:
            if arguments.mapper_binary is None:
                raise RunnerPreconditionBlockedError("--mapper-binary is required for a harness-only subprocess smoke")
            harness_identity = validate_canonical_runner_executable(arguments.mapper_binary)
            harness_result = run_guarded_child_process(
                [harness_identity["canonical_path"], "--version"], harness_identity, harness_only_flag=True
            )
            run_state["manifest"]["harness_probe"] = {
                key: value for key, value in harness_result.items()
                if key not in {"stdout_bytes", "stderr_bytes"}
            }
            run_state["manifest"]["overall_status"] = "HARNESS ONLY"
            append_persistent_report_entry(
                run_state, "- Harness-only subprocess probe completed; no qualification step was run or passed."
            )
            persist_run_manifest(run_state)
            return {"overall_status": "HARNESS ONLY", "run_directory": str(run_state["run_directory"]),
                    "manifest_path": str(run_state["manifest_path"]), "report_path": str(run_state["report_path"])}
        execution_context = prepare_execution_context(arguments, run_state, artifact_run_directory)
        run_state["manifest"]["preflight"]["status"] = "PASS"
        run_state["manifest"]["preflight"]["reason"] = "all frozen inputs and required tools are present"
        run_state["manifest"]["preflight"]["timestamp"] = current_utc_timestamp_text()
        append_persistent_report_entry(run_state, "- Preflight: **PASS** — frozen identity and prerequisites recorded.")
        persist_run_manifest(run_state)

        indexer_version = run_guarded_child_process(
            [str(execution_context["indexer_path"]), "--version"],
            execution_context["mapper_identity"], execution_context["identity_guard"],
        )
        strace_version = run_guarded_child_process(
            [str(execution_context["strace_path"]), "--version"],
            execution_context["mapper_identity"], execution_context["identity_guard"],
        )
        if indexer_version["return_code"] != 0 or strace_version["return_code"] != 0:
            raise RunnerPreconditionBlockedError("required tool version probe did not complete successfully")
        run_state["manifest"]["runtime_versions"].update({
            "indexer": indexer_version["stdout"].strip() or indexer_version["stderr"].splitlines()[0:1],
            "strace": strace_version["stdout"].splitlines()[0:1] or strace_version["stderr"].splitlines()[0:1],
            "indexer_version_command_status": indexer_version["return_code"],
            "strace_version_command_status": strace_version["return_code"],
        })
        append_persistent_report_entry(
            run_state,
            f"- Runtime tools: Python `{sys.version.splitlines()[0]}`; indexer "
            f"`{run_state['manifest']['runtime_versions']['indexer']}`; strace "
            f"`{run_state['manifest']['runtime_versions']['strace']}`; rustc/cargo versions "
            "are recorded as available or unavailable in frozen execution identities.",
        )
        persist_run_manifest(run_state)
        ordered_result = run_ordered_repository_sequence(
            DEFAULT_REPOSITORY_ORDER,
            lambda repository_name: execute_repository_sop_stages(
                repository_name, run_state, execution_context, arguments
            ),
        )
        run_state["manifest"]["repository_execution_order"] = ordered_result["attempted_repositories"]
        execution_context["identity_guard"]("before finalization", [])
        finalize_unimplemented_milestone_gates(run_state, execution_context, arguments)
        execution_context["identity_guard"]("after finalization", [])
        record_finalization_outcome(run_state, "PASS", "reports and separate milestone gates were finalized")
    except RunnerFrozenInputMutationError as error:
        if run_state["manifest"]["preflight"].get("status") == "PASS":
            run_state["manifest"]["input_integrity_status"] = "FAIL"
            run_state["manifest"]["input_integrity_reason"] = str(error)
            record_finalization_outcome(run_state, "FAIL", str(error))
        else:
            run_state["manifest"]["input_integrity_status"] = "FAIL"
            record_preflight_outcome(run_state, "FAIL", str(error), "input-integrity.frozen-input-mutation")
    except RunnerContractViolationError as error:
        if run_state["manifest"]["preflight"].get("status") == "PASS":
            record_finalization_outcome(run_state, "FAIL", str(error))
        else:
            record_preflight_outcome(run_state, "FAIL", str(error), "preflight.contract-violation")
    except RunnerPreconditionBlockedError as error:
        if run_state["manifest"]["preflight"].get("status") == "PASS":
            record_finalization_outcome(run_state, "BLOCKED", str(error))
        else:
            record_preflight_outcome(run_state, "BLOCKED", str(error), "preflight.missing-prerequisite")
    except KeyboardInterrupt:
        if run_state["manifest"]["preflight"].get("status") == "PASS":
            record_finalization_outcome(run_state, "BLOCKED", "execution interrupted")
        else:
            record_preflight_outcome(run_state, "BLOCKED", "execution interrupted", "preflight.interrupted")
        raise
    except Exception as error:
        reason = f"unexpected {type(error).__name__}: {error}"
        if run_state["manifest"]["preflight"].get("status") == "PASS":
            record_finalization_outcome(run_state, "BLOCKED", reason)
        else:
            record_preflight_outcome(run_state, "BLOCKED", reason, "preflight.runner-exception")
    finally:
        finalize_artifact_digest_best_effort(run_state, artifact_run_directory)
        persist_run_manifest(run_state)
    return {
        "overall_status": run_state["manifest"]["overall_status"],
        "run_directory": str(run_state["run_directory"]),
        "manifest_path": str(run_state["manifest_path"]),
        "report_path": str(run_state["report_path"]),
        "repository_execution_order": run_state["manifest"].get("repository_execution_order", []),
    }


def run_persistent_qualification_plan(arguments: argparse.Namespace) -> Mapping[str, Any]:
    validate_run_output_paths(arguments)
    run_state = create_persistent_run_report(
        arguments.report_root,
        arguments.run_id,
        arguments.milestone,
        DEFAULT_REPOSITORY_ORDER,
        arguments.harness_only,
    )
    run_state["manifest"]["mode"] = arguments.mode.upper()
    persist_run_manifest(run_state)
    if arguments.mode == "plan":
        append_persistent_report_entry(
            run_state,
            "- Planning only: mapper/oracle/indexer were not invoked; all qualification steps remain NOT STARTED.",
        )
        persist_run_manifest(run_state)
        return {
            "overall_status": "NOT STARTED",
            "run_directory": str(run_state["run_directory"]),
            "manifest_path": str(run_state["manifest_path"]),
            "report_path": str(run_state["report_path"]),
        }
    return execute_qualification_run(arguments, run_state)


def main(argument_values: Sequence[str] | None = None) -> int:
    arguments = build_runner_argument_parser().parse_args(argument_values)
    try:
        result = run_persistent_qualification_plan(arguments)
    except KeyboardInterrupt:
        print(json.dumps({"overall_status": "BLOCKED", "reason": "interrupted"}))
        return 130
    except (RunnerPreconditionBlockedError, RunnerContractViolationError) as error:
        print(json.dumps({"overall_status": "BLOCKED", "reason": str(error)}), file=sys.stderr)
        return 2
    print(json.dumps(result, indent=2, default=json_default_value))
    if result["overall_status"] == "FAIL":
        return 1
    if result["overall_status"] == "BLOCKED":
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
