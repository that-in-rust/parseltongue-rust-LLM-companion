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
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
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
MAX_COMMAND_EVENT_BYTES = 16_384
MAX_COMMAND_JOURNAL_BYTES = 8 * 1024 * 1024
MAX_COMMAND_EVENT_COUNT = 512


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


def current_utc_timestamp_text() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def probe_toolchain_versions() -> Mapping[str, Mapping[str, Any]]:
    versions = {}
    for tool_name in ("rustc", "cargo"):
        executable_path = shutil.which(tool_name)
        if not executable_path:
            versions[tool_name] = {
                "path": None, "canonical_path": None,
                "version": "unavailable", "return_code": None,
            }
            continue
        canonical_path = str(Path(executable_path).resolve())
        try:
            result = subprocess.run([executable_path, "--version"], text=True,
                                    capture_output=True, check=False)
        except OSError as error:
            versions[tool_name] = {
                "path": executable_path, "canonical_path": canonical_path,
                "version": "unavailable", "return_code": None,
                "error": str(error),
            }
            continue
        version_text = result.stdout.strip() or result.stderr.strip() or "unavailable"
        versions[tool_name] = {
            "path": executable_path,
            "canonical_path": canonical_path,
            "version": version_text if result.returncode == 0 else "unavailable",
            "return_code": result.returncode,
            "error": result.stderr.strip() if result.returncode != 0 else None,
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
    if manifest_value.get("execution_identity") == "HARNESS ONLY" and "harness_probe" in manifest_value:
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
    try:
        current_identity = capture_frozen_file_identity(Path(file_identity["path"]))
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
    try:
        current = capture_optional_frozen_file_identity(Path(file_identity["path"]))
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


def validate_run_output_paths(arguments: argparse.Namespace) -> Mapping[str, Path]:
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,127}", arguments.run_id):
        raise RunnerPreconditionBlockedError("run identifier must be a safe, unique path component")
    output_paths = validate_output_root_paths(
        arguments.report_root,
        arguments.artifact_root,
        PROJECT_ROOT,
        resolve_output_source_roots(arguments),
    )
    arguments.report_root = output_paths["report_root"]
    arguments.artifact_root = output_paths["artifact_root"]
    return output_paths


def capture_identity_registry(
    project_root: Path,
    manifest_paths: Sequence[Path],
    corpus_roots: Mapping[str, Path],
    excluded_paths: Sequence[Path],
    executable_identities: Sequence[Mapping[str, Any]],
) -> Mapping[str, Any]:
    frozen_files = {}
    for file_path in manifest_paths:
        file_identity = capture_frozen_file_identity(file_path)
        frozen_files[file_identity["path"]] = file_identity
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
    return {
        "frozen_files": frozen_files,
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


def make_identity_boundary_callback(registry: Mapping[str, Any]):
    def verify_boundary(phase_name: str, command_values: Sequence[str]) -> None:
        verify_identity_registry(registry)
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
    run_state: Mapping[str, Any], repository_name: str
) -> str:
    repository_state = run_state["manifest"]["repositories"][repository_name]
    components = repository_state["s3_components"]
    missing_components = [name for name, status in components.items() if status != "PASS"]
    gate_reasons = {
        "fixture_suite": "SPEC §4 negative/stale/collision/cancellation fixture suite was not supplied or run",
        "build_evidence": "current-run production build evidence was not supplied or identity-bound",
        "dependency_audit": "production cargo tree/dependency audit evidence was not supplied",
        "p2_offline_qualification": "selected workspace offline cargo check P2 evidence was not supplied",
        "release_provenance": "current-run release build/source/executable provenance was not supplied",
    }
    if any(status == "FAIL" for status in components.values()):
        step_status = "FAIL"
        continuation_status = "FAIL"
        reason = f"S3 component contract failed: {missing_components}"
    elif missing_components:
        step_status = "BLOCKED"
        continuation_status = "BLOCKED"
        reason = f"S3 components are incomplete: {missing_components}"
    else:
        step_status = "BLOCKED"
        continuation_status = "COMPONENTS COMPLETE"
        reason = "mapping components passed; full S3 remains BLOCKED on independent prerequisites"
    for gate_name, gate_reason in gate_reasons.items():
        repository_state["s3_qualification_gates"][gate_name] = {
            "status": "BLOCKED", "reason": gate_reason,
        }
    record_s3_stage_transition(
        run_state, repository_name, step_status, (), reason,
        "S3.full-qualification-prerequisites-missing" if step_status == "BLOCKED"
        else "S3.component-contract-failure",
    )
    gate_summary = "; ".join(f"{name}: BLOCKED ({value})" for name, value in gate_reasons.items())
    append_persistent_report_entry(
        run_state,
        f"- S3 `{repository_name}` components `{json.dumps(components, sort_keys=True)}`; "
        f"qualification **{step_status}** — {reason}. Missing gates: {gate_summary}.",
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
    registry = capture_identity_registry(
        PROJECT_ROOT,
        manifest_paths,
        corpus_roots,
        excluded_paths,
        [mapper_identity, indexer_identity, strace_identity, *([b3_tool_identity] if b3_tool_identity else [])],
    )
    if registry["frozen_files"].get(corpus_manifest_identity["path"]) != corpus_manifest_identity:
        raise RunnerContractViolationError("corpus pin manifest changed while freezing execution inputs")
    for repository_name, expected_snapshot in source_snapshots.items():
        if registry["source_snapshots"][str(corpus_roots[repository_name])] != expected_snapshot:
            raise RunnerContractViolationError(f"source changed while freezing {repository_name}")
    if registry["source_snapshots"][str(PROJECT_ROOT.resolve())] != root_snapshot:
        raise RunnerContractViolationError("runner source changed while freezing repository inputs")
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
        "toolchain_versions": probe_toolchain_versions(),
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
        "identity_guard": make_identity_boundary_callback(registry),
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
            record_persistent_run_transition(
                run_state, repository_name, "S1", "PASS", current_commands,
                f"fresh full generation {metadata_result['generation']} has matching status/schema metadata",
                "S1.fresh-full-index-and-metadata",
                command_metadata=s1_command_metadata,
            )
            metadata_complete = True
            current_step = "S2"
            current_commands = []

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
        return finalize_s3_component_qualification(run_state, repository_name)
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
        finalize_unimplemented_milestone_gates(run_state, execution_context, arguments)
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
