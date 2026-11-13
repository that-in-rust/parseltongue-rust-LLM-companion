"""REQ-SOP-RUNNER-* contract tests for the Python qualification harness."""

from __future__ import annotations

import hashlib
import importlib.util
from io import StringIO
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
import unittest
from types import SimpleNamespace
from unittest.mock import patch


TOOLS_ROOT = Path(__file__).resolve().parent


def load_python_tools_module(module_name: str, source_path: Path):
    module_spec = importlib.util.spec_from_file_location(module_name, source_path)
    if module_spec is None or module_spec.loader is None:
        raise RuntimeError(f"cannot load {source_path}")
    module_value = importlib.util.module_from_spec(module_spec)
    sys.modules[module_name] = module_value
    module_spec.loader.exec_module(module_value)
    return module_value


runner = load_python_tools_module("sop_qualification_runner", TOOLS_ROOT / "hld-current-qualification-runner.py")
inventory = load_python_tools_module("coverage_inventory", TOOLS_ROOT / "corpus-index-coverage-inventory.py")
measurements = load_python_tools_module("production_measurements", TOOLS_ROOT / "corpus-production-stage-measurements.py")
auditor = load_python_tools_module("production_auditor", TOOLS_ROOT / "process-tree-access-auditor.py")


def create_committed_git_repository(root_path: Path, file_values: dict[str, str | bytes]) -> str:
    root_path.mkdir(parents=True, exist_ok=True)
    for relative_path, contents in file_values.items():
        file_path = root_path / relative_path
        file_path.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(contents, bytes):
            file_path.write_bytes(contents)
        else:
            file_path.write_text(contents)
    subprocess.run(["git", "init", "--quiet"], cwd=root_path, check=True, capture_output=True, text=True)
    subprocess.run(
        ["git", "add", "--", *file_values], cwd=root_path, check=True, capture_output=True, text=True
    )
    subprocess.run(
        ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
         "commit", "--quiet", "-m", "fixture baseline"],
        cwd=root_path, check=True, capture_output=True, text=True,
    )
    return subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=root_path, check=True, capture_output=True, text=True
    ).stdout.strip()


def create_coverage_command_runner(scope_text: str, missed_texts: list[str]):
    missed_query_count = 0

    def run_command(command_values, capture_output, check):
        nonlocal missed_query_count
        if "list_projects" in command_values:
            output = "project-fixture\n"
        elif "index_status" in command_values:
            output = json.dumps({
                "status": "ready", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
            })
        elif "get_graph_schema" in command_values:
            output = json.dumps({"node_labels": [{"label": "File"}], "edge_labels": []})
        elif "check_index_coverage" in command_values:
            output = scope_text
        elif "query_graph" in command_values:
            if missed_query_count >= len(missed_texts):
                raise AssertionError("coverage collector requested another page after its cursor cycle")
            output = missed_texts[missed_query_count]
            missed_query_count += 1
        else:
            raise AssertionError(f"unexpected indexer command: {command_values!r}")
        return subprocess.CompletedProcess(command_values, 0, output, "")

    return run_command, lambda: missed_query_count


def create_preflight_fixture(root_path: Path) -> dict[str, object]:
    project_root = root_path / "project"
    phase_root = project_root / "docs/phase-20261111"
    corpus_roots = {}
    corpus_records = []
    for repository_name in runner.DEFAULT_REPOSITORY_ORDER:
        corpus_root = root_path / "corpora" / repository_name
        commit = create_committed_git_repository(corpus_root, {
            "Cargo.toml": f"[package]\nname='{repository_name}'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": f"pub fn {repository_name.replace('-', '_')}() {{}}\n",
        })
        corpus_roots[repository_name] = corpus_root
        corpus_records.append({
            "corpus_repository_short_name": repository_name,
            "corpus_source_checkout_path": str(corpus_root),
            "corpus_source_commit_hash": commit,
            "corpus_source_upstream_hash": "upstream-rust-analyzer",
        })
    rust_analyzer_commit = next(
        record["corpus_source_commit_hash"] for record in corpus_records
        if record["corpus_repository_short_name"] == "rust-analyzer"
    )
    patch_bytes = b"fixture parser patch\n"
    patch_digest = hashlib.sha256(patch_bytes).hexdigest()
    parser_manifest = {
        "parser_source_commit_hash": rust_analyzer_commit,
        "parser_source_upstream_hash": "upstream-rust-analyzer",
        "parser_source_patch_digest": patch_digest,
        "parser_source_patch_path": "docs/phase-20261111/parser.patch",
        "parser_source_bundle_path": "docs/phase-20261111/parser.bundle",
    }
    oracle_manifest = {
        "oracle_source_commit_hash": rust_analyzer_commit,
        "oracle_source_upstream_hash": "upstream-rust-analyzer",
        "oracle_source_patch_path": "docs/phase-20261111/parser.patch",
    }
    project_files: dict[str, str | bytes] = {
        "Cargo.toml": (
            '[workspace]\nmembers = ["crates/parseltongue-cli-dispatcher-binary"]\n'
            '[workspace.package]\nversion = "0.1.0"\nedition = "2024"\n'
        ),
        "Cargo.lock": "",
        "crates/parseltongue-cli-dispatcher-binary/Cargo.toml": (
            '[package]\nname = "parseltongue-cli-dispatcher-binary"\nversion.workspace = true\n'
            'edition.workspace = true\n[[bin]]\nname = "parseltongue"\npath = "src/main.rs"\n'
        ),
        "crates/parseltongue-cli-dispatcher-binary/src/main.rs": "fn main() {}\n",
        "docs/phase-20261111/corpus-source-pin-manifest.json": json.dumps(corpus_records),
        "docs/phase-20261111/parser-source-pin-manifest.json": json.dumps(parser_manifest),
        "docs/phase-20261111/oracle-source-build-manifest.json": json.dumps(oracle_manifest),
        "docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md": "# SOP fixture\n",
        "docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md": "# SPEC fixture\n",
        "docs/phase-20261111/parser.patch": patch_bytes,
        "docs/phase-20261111/parser.bundle": b"fixture parser bundle\n",
        "tools/hld-current-qualification-runner.py": "# runner fixture\n",
        "tools/corpus-index-coverage-inventory.py": "# index fixture\n",
        "tools/corpus-production-stage-measurements.py": "# measurement fixture\n",
        "tools/process-tree-access-auditor.py": "# audit fixture\n",
    }
    create_committed_git_repository(project_root, project_files)
    binaries = {}
    for binary_name in ("mapper", "indexer", "strace"):
        binary_path = root_path / "bin" / binary_name
        binary_path.parent.mkdir(parents=True, exist_ok=True)
        binary_path.write_text(f"#!{sys.executable}\nprint('fixture {binary_name}')\n")
        binary_path.chmod(0o755)
        binaries[binary_name] = binary_path
    return {
        "project_root": project_root,
        "corpus_roots": corpus_roots,
        "corpus_manifest": project_root / "docs/phase-20261111/corpus-source-pin-manifest.json",
        "rust_analyzer_commit": rust_analyzer_commit,
        "patch_digest": patch_digest,
        "binaries": binaries,
    }


def create_identity_bound_s3_fixture(root_path: Path, fixture_failed: bool = False,
                                     malformed_sibling: bool = False, native_skip: bool = False):
    project_root = root_path / "identity-project"
    package_name = "parseltongue-cli-dispatcher-binary"
    production_directory = f"crates/{package_name}"
    test_names = sorted(runner.s3_evidence_intake.M0_TEST_NAMES | runner.s3_evidence_intake.M1_TEST_NAMES)
    fixture_rows = [f"test fixture::{name} ... {'FAILED' if fixture_failed and index == 0 else 'ok'}"
                    for index, name in enumerate(test_names)]
    passed = len(test_names) - int(fixture_failed)
    failed = int(fixture_failed)
    fixture_output = "\n".join([
        f"running {len(test_names)} tests", *fixture_rows,
        f"test result: {'FAILED' if fixture_failed else 'ok'}. {passed} passed; {failed} failed; "
        "0 ignored; 0 measured; 0 filtered out; finished in 0.01s",
    ])
    production_manifest = f"{production_directory}/Cargo.toml"
    production_source = f"{production_directory}/src/main.rs"
    parser_manifest = "docs/phase-20261111/parser-source-pin-manifest.json"
    parser_bundle = "docs/phase-20261111/parser.bundle"
    r1_patch = "docs/phase-20261111/parser.patch"
    corpus_manifest = "docs/phase-20261111/corpus-source-pin-manifest.json"
    config_path = ".cargo/config.toml"
    config_text = '[env]\nPARSELTONGUE_SKIP_NATIVE_TESTS = "1"\n' if native_skip else '[term]\ncolor = "never"\n'
    project_files: dict[str, str | bytes] = {
        "Cargo.toml": (
            '[workspace]\nmembers = ["crates/parseltongue-cli-dispatcher-binary"]\nresolver = "3"\n'
            '[workspace.package]\nversion = "0.1.0"\nedition = "2024"\n'
        ),
        "Cargo.lock": "",
        production_manifest: (
            '[package]\nname = "parseltongue-cli-dispatcher-binary"\nversion.workspace = true\n'
            'edition.workspace = true\n[[bin]]\nname = "parseltongue"\npath = "src/main.rs"\n'
        ),
        production_source: "fn main() {}\n",
        f"{production_directory}/tests/required.rs": "\n".join(
            f"#[test]\nfn {name}() {{}}" for name in test_names
        ) + "\n",
        parser_manifest: json.dumps({"parser_source_bundle_path": parser_bundle}),
        parser_bundle: b"synthetic parser bundle\n",
        r1_patch: b"synthetic frozen R1 patch\n",
        corpus_manifest: "[]\n",
        "docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md": "# synthetic SOP\n",
        "docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md": "# synthetic SPEC\n",
        config_path: config_text,
    }
    create_committed_git_repository(project_root, project_files)

    corpus_roots = {}
    corpus_records = {}
    for repository_name in runner.DEFAULT_REPOSITORY_ORDER:
        corpus_root = root_path / "corpora" / repository_name
        commit = create_committed_git_repository(corpus_root, {
            "Cargo.toml": (
                f'[package]\nname = "{repository_name}"\nversion = "0.1.0"\n'
                'edition.workspace = true\n[workspace]\n[workspace.package]\nedition = "2024"\n'
                '[lib]\npath = "src/lib.rs"\n'
            ),
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
            "src/main.rs": "fn main() {}\n",
        })
        corpus_roots[repository_name] = corpus_root.resolve()
        corpus_records[repository_name] = {"corpus_source_commit_hash": commit}

    binary_root = root_path / "toolchain-bin"
    binary_root.mkdir()
    executable_identities = []
    toolchain_versions = {}
    for tool_name in ("cargo", "rustc", "mapper"):
        binary_path = binary_root / ("parseltongue" if tool_name == "mapper" else tool_name)
        binary_path.write_text(f"#!/bin/sh\nprintf '{tool_name} harness\\n'\n")
        binary_path.chmod(0o755)
        identity = dict(runner.capture_frozen_file_identity(binary_path))
        identity["canonical_path"] = str(binary_path.resolve())
        identity["blake3"] = hashlib.sha256(b"harness-blake3:" + binary_path.read_bytes()).hexdigest()
        executable_identities.append(identity)
        if tool_name != "mapper":
            toolchain_versions[tool_name] = {
                "path": str(binary_path), "canonical_path": str(binary_path.resolve()),
                "version": f"{tool_name} 1.90.0", "return_code": 0,
                "file_identity": identity,
            }
    mapper_identity = executable_identities[-1]
    environment = {"PATH": str(binary_root), "CARGO_HOME": str(root_path / "cargo-home")}
    with patch.object(runner, "PROJECT_ROOT", project_root), patch.object(
        runner, "capture_s3_evidence_environment", return_value=environment,
    ):
        cargo_configuration_paths = runner.find_cargo_configuration_files(
            project_root, corpus_roots, environment,
        )
        present_configs = [path for path in cargo_configuration_paths if path.exists()]
        absent_configs = [path for path in cargo_configuration_paths if not path.exists()]
        manifest_paths = [
            project_root / "Cargo.toml", project_root / production_manifest,
            project_root / production_source,
            project_root / parser_manifest, project_root / parser_bundle, project_root / r1_patch,
            project_root / corpus_manifest, project_root / "docs/phase-20261111/SOP-phase-20261111-hld-current-e2e-test.md",
            project_root / "docs/phase-20261111/SPEC-phase-20261111-executable-requirements.md",
            *present_configs,
        ]
        optional_identities = {
            str(path.absolute()): runner.capture_optional_frozen_file_identity(path)
            for path in absent_configs
        }
        registry = runner.capture_identity_registry(
            project_root, manifest_paths, corpus_roots, (), executable_identities,
            optional_frozen_paths=absent_configs, expected_optional_files=optional_identities,
        )
        source_snapshots = {
            name: registry["source_snapshots"][str(root)] for name, root in corpus_roots.items()
        }
        arguments = SimpleNamespace(
            run_id="identity-bound-harness", milestone="M1", mode="execute",
            corpus_manifest=str(project_root / corpus_manifest), project_root=[],
        )
        run_state = runner.create_persistent_run_report(
            root_path / "reports", arguments.run_id, "M1", harness_only_flag=True,
        )
        pin_summary = {
            "r1_patch_path": str(project_root / r1_patch),
            "r1_patch_sha256": hashlib.sha256((project_root / r1_patch).read_bytes()).hexdigest(),
        }
        p2_configuration_reason = runner.cargo_configuration_override_reason(present_configs)
        expected_identity = runner.build_s3_expected_identity(
            arguments, run_state, mapper_identity, pin_summary, registry, toolchain_versions,
            cargo_configuration_paths, p2_configuration_reason,
        )
        expected_repositories = runner.build_s3_repository_bindings(
            corpus_records, corpus_roots, source_snapshots, registry["lock_files"],
            p2_configuration_reason,
        )
        source_inventory = runner.derive_production_source_test_inventory(project_root)

    evidence_root = root_path / "evidence"
    evidence_root.mkdir()

    def blake3_test_digest(content: bytes) -> str:
        return hashlib.sha256(b"harness-blake3:" + content).hexdigest()

    def store_file(relative_path: str, contents: str | bytes):
        content = contents.encode() if isinstance(contents, str) else contents
        target = evidence_root / relative_path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
        return {
            "path": relative_path, "size_bytes": len(content),
            "sha256": hashlib.sha256(content).hexdigest(), "blake3": blake3_test_digest(content),
        }

    cargo = toolchain_versions["cargo"]["path"]
    receipt_number = 0

    def add_receipt(family, scope, argv, stdout="", stderr="", repository_binding=None):
        nonlocal receipt_number
        receipt_number += 1
        receipt_id = f"s3-harness-receipt-{receipt_number}"
        stdout_path = f"logs/{receipt_id}.stdout"
        stderr_path = f"logs/{receipt_id}.stderr"
        stdout_ref = store_file(stdout_path, stdout)
        stderr_ref = store_file(stderr_path, stderr)
        receipt = {
            "schema_version": 1, "receipt_id": receipt_id, "family": family, "scope": scope,
            "identity_binding": expected_identity, "argv": argv,
            "cwd": (repository_binding["checkout_path"] if repository_binding
                    else expected_identity["configuration"]["project_root"]),
            "relevant_environment": expected_identity["relevant_environment"],
            "elapsed_seconds": 0.25, "timeout_seconds": 30.0,
            "completion_status": "completed", "timed_out": False, "exit_status": 0,
            "stdout": {**stdout_ref, "complete": True},
            "stderr": {**stderr_ref, "complete": True},
        }
        if repository_binding is not None:
            receipt["repository_binding"] = repository_binding
        return store_file(f"receipts/{receipt_id}.json", json.dumps(receipt, sort_keys=True))

    fixture_ref = add_receipt(
        "fixture_suite", "production_workspace", [cargo, "test", "--workspace"], fixture_output,
    )
    fmt_ref = add_receipt("fmt", "production_workspace", [cargo, "fmt", "--all", "--", "--check"])
    clippy_ref = (
        store_file("receipts/malformed-clippy.json", b'{"schema_version":') if malformed_sibling else
        add_receipt(
            "clippy", "production_workspace",
            [cargo, "clippy", "--message-format=json", "--workspace", "--all-targets", "--all-features",
             "--", "-D", "warnings"],
            json.dumps({"reason": "build-finished", "success": True}),
        )
    )
    tree_ref = add_receipt(
        "dependency_audit", "production_workspace",
        [cargo, "tree", "--package", package_name, "--edges", "normal", "--prefix", "none", "--format", "{p}",
         "--offline", "--locked"],
        f"{package_name} v0.1.0\nra_ap_syntax v0.0.0\n",
    )
    production_config = expected_identity["configuration"]
    production_manifest_path = str(project_root / production_manifest)
    production_source_path = str(project_root / production_source)
    release_output = "\n".join([
        json.dumps({
            "reason": "compiler-artifact",
            "package_id": f"path+file://{Path(production_manifest_path).parent}#0.1.0",
            "manifest_path": production_manifest_path,
            "target": {"name": "parseltongue", "kind": ["bin"],
                       "src_path": production_source_path},
            "executable": mapper_identity["canonical_path"],
            "profile": {"test": False, "opt_level": "3"},
        }),
        json.dumps({"reason": "build-finished", "success": True}),
    ])
    release_ref = add_receipt(
        "release_build", "production_workspace",
        [cargo, "build", "--release", "--package", package_name, "--bin", "parseltongue",
         "--message-format=json"], release_output,
    )
    global_prerequisites = {
        "fixture_suite": [fixture_ref], "build_evidence": {"fmt": fmt_ref, "clippy": clippy_ref},
        "dependency_audit": [tree_ref], "release_provenance": [release_ref],
    }
    repositories = {}
    for repository_name, binding in expected_repositories.items():
        workspace_manifest = binding["workspace_members"][0]
        package_dir = Path(workspace_manifest).parent
        package_id = f"path+file://{package_dir}#{repository_name}@0.1.0"
        edition = next(unit["edition"] for unit in binding["selected_units"])
        metadata = {
            "workspace_members": [package_id],
            "packages": [{
                "id": package_id, "name": repository_name, "edition": edition,
                "manifest_path": workspace_manifest,
                "targets": [{"name": unit["target_name"], "kind": unit["kind"],
                             "src_path": unit["src_path"]}
                            for unit in binding["selected_units"]],
            }],
            "resolve": {"nodes": [{"id": package_id, "features": []}]},
        }
        metadata_ref = add_receipt(
            "p2_metadata", repository_name,
            [cargo, "metadata", "--format-version", "1", "--offline", "--locked"],
            json.dumps(metadata), repository_binding=binding,
        )
        check_messages = [json.dumps({
            "reason": "compiler-artifact", "package_id": package_id, "features": [],
            "manifest_path": unit["manifest_path"],
            "target": {"name": unit["target_name"], "kind": unit["kind"], "edition": unit["edition"],
                       "src_path": unit["src_path"]},
            "profile": {"test": False},
        }) for unit in binding["selected_units"]]
        check_messages.append(json.dumps({"reason": "build-finished", "success": True}))
        check_ref = add_receipt(
            "p2_check", repository_name,
            [cargo, "check", "--workspace", "--lib", "--bins", "--offline", "--locked",
             "--message-format=json"], "\n".join(check_messages), repository_binding=binding,
        )
        repositories[repository_name] = {
            "repository_binding": binding, "p2_offline_qualification": [metadata_ref, check_ref],
        }
    manifest = {
        "schema_version": 1, "run_id": arguments.run_id, "milestone": "M1",
        "identity_binding": expected_identity, "global_prerequisites": global_prerequisites,
        "repositories": repositories,
    }
    manifest_path = root_path / "evidence" / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, sort_keys=True))
    return {
        "project_root": project_root, "corpus_roots": corpus_roots, "registry": registry,
        "expected_identity": expected_identity, "expected_repositories": expected_repositories,
        "source_inventory": source_inventory, "arguments": arguments, "run_state": run_state,
        "manifest_path": manifest_path, "mapper_identity": mapper_identity,
        "cargo_configuration_paths": cargo_configuration_paths,
        "configuration_reason": p2_configuration_reason,
        "blake3_test_digest": blake3_test_digest,
    }


def create_coverage_scope_page(generation: str = "generation-a", offset: int = 0, next_offset: int = 1,
                               has_more: bool = False, entries: list[str] | None = None,
                               total: int = 1, generation_matches: bool = True) -> dict:
    raw_entries = entries if entries is not None else ["tools/a.py"]
    scope_entries = [
        dict(entry) if isinstance(entry, dict) else {
            "path": entry, "kind": "not_indexed_dir", "detail": "excluded subtree",
        }
        for entry in raw_entries
    ]
    return {
        "indexed_at": generation,
        "requested_offset": offset,
        "metadata": {"recording_status": "complete", "generation_matches": generation_matches},
        "scopes": [{"entries": scope_entries}],
        "scope_total": total,
        "has_more": has_more,
        "next_offset": next_offset,
    }


def create_coverage_missed_page(generation: str = "generation-a", rows: list[str] | None = None,
                                has_more: bool = False, next_cursor: str | None = None,
                                total: int = 0) -> dict:
    row_values = [list(row) if isinstance(row, (list, tuple)) else [row]
                  for row in (rows if rows is not None else [])]
    return {
        "columns": ["f.file_path"],
        "returned": len(row_values),
        "total_relation": "eq",
        "indexed_at": generation,
        "rows": row_values,
        "truncated": has_more,
        "truncation_reason": "page_limit" if has_more else None,
        "has_more": has_more,
        "next_cursor": next_cursor,
        "total": total,
    }


class SopQualificationRunnerContractTests(unittest.TestCase):
    def setUp(self):
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.root_path = Path(self.temporary_directory.name)

    def tearDown(self):
        self.temporary_directory.cleanup()

    def make_dependency_collector_identity(self):
        cargo_path = self.root_path / "frozen-cargo"
        cargo_path.write_text("harness only\n")
        cargo_identity = runner.capture_frozen_file_identity(cargo_path)
        return {
            "configuration": {
                "project_root": str(runner.PROJECT_ROOT.resolve()),
                "production_package_name": "parseltongue-cli-dispatcher-binary",
                "p2_configuration_blocked_reason": None, "unsupported_prerequisite_gates": {},
            },
            "toolchain_versions": {"cargo": {"path": str(cargo_path), "version": "cargo harness"}},
            "toolchain_executables": {"cargo": cargo_identity},
            "relevant_environment": runner.capture_s3_evidence_environment(),
        }

    def make_dependency_collector_evidence(self):
        return {"manifest_document": {
            "global_prerequisites": {"dependency_audit": [{"path": "receipts/dependency-tree.json"}]},
        }}

    def grant_synthetic_s3_token(self, context, run_state, repository_name):
        binding = {"run_id": run_state["manifest"]["run_id"]}
        identity_digest = hashlib.sha256(
            json.dumps(binding, sort_keys=True, separators=(",", ":")).encode()
        ).hexdigest()
        context["s3_expected_identity"] = binding
        run_state["manifest"]["repositories"][repository_name]["s3_qualification_gates"] = {
            name: {"status": "PASS", "reason": "synthetic harness"} for name in runner.S3_GATE_NAMES
        }
        context["s3_evidence"] = {
            "repository_results": {repository_name: {
                name: {"status": "PASS", "reason": "synthetic harness"} for name in runner.S3_GATE_NAMES
            }},
            "intake_status": {"status": "PASS", "reason": "synthetic harness"},
            "tokens_by_repository": {repository_name: runner.s3_evidence_intake.S3EvidenceQualificationToken(
                run_state["manifest"]["run_id"], repository_name, identity_digest,
            )},
        }

    def prepare_identity_bound_evidence(self, fixture):
        def digest_evidence(path, *_args, **_kwargs):
            return fixture["blake3_test_digest"](Path(path).read_bytes())

        with patch.object(runner, "calculate_b3_file_digest", side_effect=digest_evidence):
            evidence_inputs = runner.read_s3_evidence_inputs(
                fixture["manifest_path"], "python-blake3:harness-only", None,
                forbidden_roots=(fixture["project_root"], *fixture["corpus_roots"].values(),
                                 self.root_path / "reports", self.root_path / "artifacts"),
            )
        expected_identity = fixture["expected_identity"]
        package_name = expected_identity["configuration"]["production_package_name"]
        stdout_bytes = f"{package_name} v0.1.0\nra_ap_syntax v0.0.0\n".encode()

        def stream(name, content):
            path = f"/harness-only/{name}"
            return {
                "path": path, "raw_bytes": content, "size_bytes": len(content),
                "sha256": hashlib.sha256(content).hexdigest(),
                "blake3": fixture["blake3_test_digest"](content),
                "complete": True,
                "file_identity": {"path": path, "size_bytes": len(content),
                                  "sha256": hashlib.sha256(content).hexdigest(), "is_symlink": False},
            }

        cargo_path = expected_identity["toolchain_versions"]["cargo"]["path"]
        dependency_reference = runner.s3_evidence_intake._mint_runner_owned_dependency_tree_reference(
            identity_digest=hashlib.sha256(json.dumps(
                expected_identity, sort_keys=True, separators=(",", ":"),
            ).encode()).hexdigest(),
            argv=[cargo_path, "tree", "--package", package_name, "--edges", "normal", "--prefix", "none",
                  "--format", "{p}", "--offline", "--locked"],
            cwd=expected_identity["configuration"]["project_root"],
            relevant_environment=expected_identity["relevant_environment"],
            elapsed_seconds=0.1, timeout_seconds=60.0, timed_out=False,
            completion_status="completed", exit_status=0,
            stdout=stream("stdout", stdout_bytes), stderr=stream("stderr", b""),
        )
        return runner.prepare_s3_evidence_qualification(
            fixture["arguments"], fixture["run_state"], evidence_inputs,
            expected_identity, fixture["expected_repositories"], fixture["source_inventory"],
            dependency_reference,
        )

    def test_s3_identity_bound_harness(self):
        """REQ-SOP-RUNNER-022: real builder/evidence path validates tokens but keeps harness S3 unqualified."""
        fixture = create_identity_bound_s3_fixture(self.root_path)
        identity = fixture["expected_identity"]
        self.assertEqual(identity["configuration"]["production_package_name"],
                         "parseltongue-cli-dispatcher-binary")
        self.assertEqual(identity["configuration"]["production_package_version"], "0.1.0")
        self.assertEqual(identity["configuration"]["production_manifest_identity"]["path"],
                         identity["configuration"]["production_manifest"])
        self.assertEqual(
            identity["configuration"]["production_bin_source"],
            str(fixture["project_root"] / "crates/parseltongue-cli-dispatcher-binary/src/main.rs"),
        )
        self.assertEqual(identity["configuration"]["production_bin_source_identity"]["path"],
                         identity["configuration"]["production_bin_source"])
        self.assertTrue(any(not value["exists"] for value in identity["configuration"]["cargo_config_file_identities"]))
        self.assertEqual(identity["toolchain_versions"]["cargo"]["path"],
                         identity["toolchain_executables"]["cargo"]["path"])
        self.assertEqual(identity["toolchain_versions"]["rustc"]["path"],
                         identity["toolchain_executables"]["rustc"]["path"])
        runner.verify_identity_registry(fixture["registry"])
        evidence = self.prepare_identity_bound_evidence(fixture)
        runner.verify_identity_registry(fixture["registry"])
        self.assertEqual(evidence["intake_status"]["status"], "PASS")
        repository_results = evidence["repository_results"]["serde"]
        self.assertTrue(all(value["status"] == "PASS" for value in repository_results.values()))
        token = evidence["tokens_by_repository"]["serde"]
        self.assertEqual(token.run_identifier, fixture["arguments"].run_id)
        self.assertEqual(token.repository_name, "serde")
        self.assertEqual(token.identity_digest, hashlib.sha256(json.dumps(
            identity, sort_keys=True, separators=(",", ":"),
        ).encode()).hexdigest())
        repository = fixture["run_state"]["manifest"]["repositories"]["serde"]
        repository["s3_components"].update({name: "PASS" for name in runner.S3_COMPONENT_NAMES})
        repository["s3_qualification_gates"] = repository_results
        result = runner.finalize_s3_component_qualification(
            fixture["run_state"], "serde", token, identity,
        )
        self.assertEqual(result, "BLOCKED")
        self.assertEqual(repository["steps"]["S3"], "BLOCKED")
        self.assertEqual(fixture["run_state"]["manifest"]["overall_status"], "HARNESS ONLY")

    def test_dependency_collector_token_harness(self):
        """REQ-SOP-RUNNER-022.D: HARNESS ONLY collector output binds helper tokens to fixed argv."""
        fixture = create_identity_bound_s3_fixture(self.root_path)
        expected_identity = fixture["expected_identity"]
        evidence_root = fixture["manifest_path"].parent
        manifest_document = json.loads(fixture["manifest_path"].read_text())
        file_bytes = {}
        blake3_by_path = {}
        for path in evidence_root.rglob("*"):
            if not path.is_file() or path == fixture["manifest_path"]:
                continue
            relative_path = path.relative_to(evidence_root).as_posix()
            content = path.read_bytes()
            file_bytes[relative_path] = content
            blake3_by_path[relative_path] = fixture["blake3_test_digest"](content)
        evidence_inputs = {
            "manifest_document": manifest_document,
            "file_bytes_by_path": file_bytes,
            "blake3_by_path": blake3_by_path,
        }
        expected_stdout = b"parseltongue-cli-dispatcher-binary v0.1.0\nra_ap_syntax v0.0.0\n"
        constructed_commands = []
        guard_phases = []

        class FakeProcess:
            pid = os.getpid()
            returncode = None

            def wait(self, timeout=None):
                self.returncode = 0
                return 0

        def fake_popen(command, **kwargs):
            constructed_commands.append(list(command))
            kwargs["stdout"].write(expected_stdout)
            kwargs["stderr"].write(b"")
            return FakeProcess()

        def fake_b3sum(command, **kwargs):
            return SimpleNamespace(returncode=0, stdout=("a" * 64 + "  output\n").encode(), stderr=b"")

        def identity_guard(phase_name, command):
            guard_phases.append((phase_name, list(command)))

        artifact_root = self.root_path / "artifacts" / "tree-collector"
        with patch.object(runner, "PROJECT_ROOT", fixture["project_root"]), \
             patch.dict(os.environ, expected_identity["relevant_environment"], clear=True), \
             patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum):
            self.assertIsNone(runner.collect_dependency_tree_reference(
                {}, expected_identity, fixture["registry"], identity_guard, artifact_root,
                "/fixture/b3sum", None,
            ))
            self.assertEqual(constructed_commands, [])
            unsupported_identity = json.loads(json.dumps(expected_identity))
            unsupported_identity["configuration"]["p2_configuration_blocked_reason"] = "unsupported harness context"
            self.assertIsNone(runner.collect_dependency_tree_reference(
                evidence_inputs, unsupported_identity, fixture["registry"], identity_guard,
                artifact_root / "unsupported", "/fixture/b3sum", None,
            ))
            self.assertEqual(constructed_commands, [])
            reference = runner.collect_dependency_tree_reference(
                evidence_inputs, expected_identity, fixture["registry"], identity_guard,
                artifact_root, "/fixture/b3sum", None,
            )
            self.assertIsInstance(reference, runner.s3_evidence_intake.RunnerOwnedDependencyTreeReference)
            fixed_command = [
                expected_identity["toolchain_versions"]["cargo"]["path"], "tree", "--package",
                expected_identity["configuration"]["production_package_name"], "--edges", "normal",
                "--prefix", "none", "--format", "{p}", "--offline", "--locked",
            ]
            self.assertEqual(constructed_commands[0], fixed_command)
            result = runner.prepare_s3_evidence_qualification(
                fixture["arguments"], fixture["run_state"], evidence_inputs, expected_identity,
                fixture["expected_repositories"], fixture["source_inventory"], reference,
            )
            self.assertTrue(result["tokens_by_repository"])
            self.assertTrue(all(
                gates["dependency_audit"]["status"] == "PASS"
                for gates in result["repository_results"].values()
            ))
            receipt_path = manifest_document["global_prerequisites"]["dependency_audit"][0]["path"]
            malicious_receipt = json.loads(file_bytes[receipt_path])
            malicious_receipt["argv"].append("--malicious")
            malicious_inputs = {**evidence_inputs, "file_bytes_by_path": dict(file_bytes)}
            malicious_inputs["file_bytes_by_path"][receipt_path] = json.dumps(malicious_receipt).encode()
            second_reference = runner.collect_dependency_tree_reference(
                malicious_inputs, expected_identity, fixture["registry"], identity_guard,
                artifact_root / "second", "/fixture/b3sum", None,
            )
            self.assertIsNotNone(second_reference)
            self.assertEqual(constructed_commands[1], fixed_command)
            Path(second_reference.stdout.path).write_bytes(b"mutated retained output")
            with self.assertRaises(runner.RunnerFrozenInputMutationError):
                runner.verify_identity_registry(fixture["registry"])
        self.assertTrue(any(name == "before S3 dependency tree collection" for name, _ in guard_phases))
        self.assertTrue(any(name == "after S3 dependency tree collection" for name, _ in guard_phases))

    def test_collector_slow_guard_timing(self):
        """REQ-SOP-RUNNER-022.D: slow identity guards do not inflate command timeout elapsed time."""
        fixture = create_identity_bound_s3_fixture(self.root_path)
        expected_identity = fixture["expected_identity"]
        package_name = expected_identity["configuration"]["production_package_name"]
        expected_stdout = f"{package_name} v0.1.0\nra_ap_syntax v0.0.0\n".encode()
        evidence_inputs = {"manifest_document": {"global_prerequisites": {"dependency_audit": [{}]}}}
        constructed_commands = []
        guard_phases = []
        timeout_seconds = 0.05

        class FakeProcess:
            pid = os.getpid()
            returncode = None

            def wait(self, timeout=None):
                self.returncode = 0
                return 0

        def fake_popen(command, **kwargs):
            constructed_commands.append(list(command))
            kwargs["stdout"].write(expected_stdout)
            kwargs["stderr"].write(b"")
            return FakeProcess()

        def fake_blake3(file_path, *_args, **_kwargs):
            return fixture["blake3_test_digest"](Path(file_path).read_bytes())

        def slow_identity_guard(phase_name, command):
            guard_phases.append(phase_name)
            if phase_name in {"before S3 dependency tree collection", "after S3 dependency tree collection"}:
                time.sleep(0.06)

        artifact_directory = self.root_path / "artifacts" / "slow-guard-tree"
        with patch.object(runner, "PROJECT_ROOT", fixture["project_root"]), \
             patch.dict(os.environ, expected_identity["relevant_environment"], clear=True), \
             patch.object(runner, "DEPENDENCY_TREE_TIMEOUT_SECONDS", timeout_seconds), \
             patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner, "calculate_b3_file_digest", side_effect=fake_blake3):
            reference = runner.collect_dependency_tree_reference(
                evidence_inputs, expected_identity, fixture["registry"], slow_identity_guard,
                artifact_directory, "/fixture/b3sum", None,
            )
            self.assertIsInstance(reference, runner.s3_evidence_intake.RunnerOwnedDependencyTreeReference)
            gate_status, gate_reason = runner.s3_evidence_intake._evaluate_dependency_audit_records(
                [{"receipt": {"timed_out": False, "exit_status": 0},
                  "log_bytes": {"stdout": expected_stdout}}],
                expected_identity, reference,
            )
        observation = json.loads((artifact_directory / "s3-dependency-tree-observation.json").read_text())
        self.assertEqual(len(constructed_commands), 1)
        self.assertTrue({"before S3 dependency tree collection", "after S3 dependency tree collection"}
                        .issubset(guard_phases))
        self.assertFalse(observation["timed_out"])
        self.assertLessEqual(observation["elapsed_seconds"], timeout_seconds)
        self.assertGreater(observation["collection_elapsed_seconds"], timeout_seconds)
        self.assertEqual(gate_status, "PASS", gate_reason)

    def test_dependency_tree_timeout_reaps(self):
        """REQ-SOP-RUNNER-022.D: HARNESS ONLY timeout kills and bounds process-group cleanup."""
        cargo_path = self.root_path / "frozen-cargo"
        cargo_path.write_text("harness only\n")
        cargo_identity = runner.capture_frozen_file_identity(cargo_path)
        expected_identity = {
            "configuration": {
                "project_root": str(runner.PROJECT_ROOT.resolve()),
                "production_package_name": "parseltongue-cli-dispatcher-binary",
                "p2_configuration_blocked_reason": None, "unsupported_prerequisite_gates": {},
            },
            "toolchain_versions": {"cargo": {"path": str(cargo_path), "version": "cargo harness"}},
            "toolchain_executables": {"cargo": cargo_identity},
            "relevant_environment": runner.capture_s3_evidence_environment(),
        }
        partial_stdout = b"parseltongue-cli-dispatcher-binary v0.1.0\n"
        wait_timeouts = []

        class TimedOutProcess:
            pid = 45678
            returncode = None

            def wait(self, timeout=None):
                wait_timeouts.append(timeout)
                if len(wait_timeouts) == 1:
                    raise subprocess.TimeoutExpired("cargo tree", timeout)
                self.returncode = -9
                return -9

        def fake_popen(_command, **kwargs):
            kwargs["stdout"].write(partial_stdout)
            kwargs["stderr"].write(b"partial diagnostic")
            return TimedOutProcess()

        def fake_b3sum(_command, **_kwargs):
            return SimpleNamespace(returncode=0, stdout=("b" * 64 + "  output\n").encode(), stderr=b"")

        with patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum), \
             patch.object(runner.os, "killpg") as kill_process_group:
            reference = runner.collect_dependency_tree_reference(
                self.make_dependency_collector_evidence(), expected_identity, {}, lambda *_args: None,
                self.root_path / "artifacts" / "timeout", "/fixture/b3sum", None,
            )
        self.assertTrue(reference.timed_out)
        self.assertEqual(reference.completion_status, "timed_out")
        self.assertEqual(reference.read_stream("stdout"), partial_stdout)
        self.assertEqual(wait_timeouts, [60.0, runner.DEPENDENCY_TREE_CLEANUP_TIMEOUT_SECONDS])
        kill_process_group.assert_called_once_with(45678, runner.signal.SIGKILL)

    def test_collector_stream_race_fails(self):
        """REQ-SOP-RUNNER-022.D: output replacement between read and identity capture is FAIL."""
        expected_identity = self.make_dependency_collector_identity()
        artifact_root = self.root_path / "artifacts" / "race"
        stdout_path = artifact_root / "s3-dependency-tree.stdout"
        expected_stdout = b"parseltongue-cli-dispatcher-binary v0.1.0\nparser-pin v1.0.0\n"
        original_capture = runner.capture_frozen_file_identity
        replaced = False

        class CompletedProcess:
            pid = os.getpid()
            returncode = 0

            def wait(self, timeout=None):
                return 0

        def fake_popen(_command, **kwargs):
            kwargs["stdout"].write(expected_stdout)
            kwargs["stderr"].write(b"")
            return CompletedProcess()

        def fake_b3sum(_command, **_kwargs):
            return SimpleNamespace(returncode=0, stdout=("c" * 64 + "  output\n").encode(), stderr=b"")

        def replace_before_identity_capture(path):
            nonlocal replaced
            if Path(path) == stdout_path and not replaced:
                replacement_path = stdout_path.with_name("replacement-output")
                replacement_path.write_bytes(b"replacement after retained read\n")
                os.replace(replacement_path, stdout_path)
                replaced = True
            return original_capture(path)

        with patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum), \
             patch.object(runner, "capture_frozen_file_identity", side_effect=replace_before_identity_capture):
            with self.assertRaises(runner.RunnerFrozenInputMutationError):
                runner.collect_dependency_tree_reference(
                    self.make_dependency_collector_evidence(), expected_identity, {}, lambda *_args: None,
                    artifact_root, "/fixture/b3sum", None,
                )
        observation = json.loads((artifact_root / "s3-dependency-tree-observation.json").read_text())
        self.assertIn("stdout", observation["stream_capture_errors"])

    def test_dependency_source_mutation_fails(self):
        """REQ-SOP-RUNNER-022.D: mocked Cargo input mutation is caught by the real registry guard."""
        fixture = create_identity_bound_s3_fixture(self.root_path)
        expected_identity = fixture["expected_identity"]
        expected_environment = dict(expected_identity["relevant_environment"])
        expected_environment["PATH"] = os.pathsep.join((expected_environment["PATH"], os.environ["PATH"]))
        expected_identity["relevant_environment"] = expected_environment
        frozen_path = next(
            Path(identity["path"])
            for identity in fixture["registry"]["frozen_files"].values()
            if Path(identity["path"]) == fixture["project_root"] / "Cargo.toml"
        )
        original_bytes = frozen_path.read_bytes()
        phases = []
        cargo_commands = []
        stdout_bytes = b"parseltongue-cli-dispatcher-binary v0.1.0\nparser-pin v1.0.0\n"

        class CompletedProcess:
            pid = os.getpid()
            returncode = 0

            def wait(self, timeout=None):
                return 0

        original_popen = runner.subprocess.Popen

        def fake_popen(command, **kwargs):
            if len(command) < 2 or command[1] != "tree":
                return original_popen(command, **kwargs)
            cargo_commands.append(list(command))
            kwargs["stdout"].write(stdout_bytes)
            kwargs["stderr"].write(b"")
            frozen_path.write_bytes(original_bytes + b"\nsource mutation during collection\n")
            return CompletedProcess()

        def fake_b3sum(command, **_kwargs):
            if command[0] != "/fixture/b3sum":
                return original_run(command, **_kwargs)
            path = Path(command[-1])
            digest = fixture["blake3_test_digest"](path.read_bytes())
            return SimpleNamespace(returncode=0, stdout=f"{digest}  {path}\n".encode(), stderr=b"")

        identity_guard = runner.make_identity_boundary_callback(
            fixture["registry"], expected_identity["relevant_environment"], fixture["mapper_identity"],
            "/fixture/b3sum",
        )

        def recording_guard(phase_name, command):
            phases.append(phase_name)
            identity_guard(phase_name, command)

        artifact_root = self.root_path / "artifacts" / "source-mutation"
        original_run = runner.subprocess.run
        with patch.object(runner, "PROJECT_ROOT", fixture["project_root"]), \
             patch.dict(os.environ, expected_environment, clear=True), \
             patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum):
            with self.assertRaises(runner.RunnerFrozenInputMutationError):
                runner.collect_dependency_tree_reference(
                    self.make_dependency_collector_evidence(), expected_identity, fixture["registry"], recording_guard,
                    artifact_root, "/fixture/b3sum", None,
                )
        observation = json.loads((artifact_root / "s3-dependency-tree-observation.json").read_text())
        self.assertTrue(observation["guard_error"])
        self.assertIn("after S3 dependency tree collection", phases)
        self.assertEqual(len(cargo_commands), 1)

    def test_unreaped_timeout_forbidden_fails(self):
        """REQ-SOP-RUNNER-022.D: forbidden retained rows FAIL when timed-out cleanup cannot reap."""
        expected_identity = self.make_dependency_collector_identity()
        artifact_root = self.root_path / "artifacts" / "unreaped"
        partial_stdout = b"parseltongue-cli-dispatcher-binary v0.1.0\nra_ap_ide v0.0.0\n"
        wait_timeouts = []
        phases = []

        class UnreapedProcess:
            pid = 45679
            returncode = None

            def wait(self, timeout=None):
                wait_timeouts.append(timeout)
                raise subprocess.TimeoutExpired("cargo tree", timeout)

        def fake_popen(_command, **kwargs):
            kwargs["stdout"].write(partial_stdout)
            kwargs["stderr"].write(b"partial error\n")
            return UnreapedProcess()

        def fake_b3sum(_command, **_kwargs):
            return SimpleNamespace(returncode=0, stdout=("d" * 64 + "  output\n").encode(), stderr=b"")

        with patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum), \
             patch.object(runner.os, "killpg") as kill_process_group:
            with self.assertRaises(runner.RunnerContractViolationError):
                runner.collect_dependency_tree_reference(
                    self.make_dependency_collector_evidence(), expected_identity, {},
                    lambda phase, _command: phases.append(phase), artifact_root, "/fixture/b3sum", None,
                )
        observation = json.loads((artifact_root / "s3-dependency-tree-observation.json").read_text())
        self.assertFalse(observation["process_reaped"])
        self.assertEqual(observation["forbidden_runtime_rows"], ["ra-ap-ide"])
        self.assertEqual(wait_timeouts, [60.0, runner.DEPENDENCY_TREE_CLEANUP_TIMEOUT_SECONDS])
        self.assertEqual(kill_process_group.call_count, 2)
        self.assertIn("after S3 dependency tree collection", phases)

    def test_collector_hash_failure_forbidden(self):
        """REQ-SOP-RUNNER-022.D: hash failure cannot discard an observed forbidden partial row."""
        expected_identity = self.make_dependency_collector_identity()
        artifact_root = self.root_path / "artifacts" / "hash-failure"
        partial_stdout = b"parseltongue-cli-dispatcher-binary v0.1.0\nra_ap_ide v0.0.0\n"

        class CompletedProcess:
            pid = os.getpid()
            returncode = 7

            def wait(self, timeout=None):
                return 7

        def fake_popen(_command, **kwargs):
            kwargs["stdout"].write(partial_stdout)
            kwargs["stderr"].write(b"hash unavailable\n")
            return CompletedProcess()

        with patch.object(runner.subprocess, "Popen", side_effect=fake_popen), \
             patch.object(runner.subprocess, "run", side_effect=runner.RunnerPreconditionBlockedError("hash unavailable")):
            with self.assertRaises(runner.RunnerContractViolationError):
                runner.collect_dependency_tree_reference(
                    self.make_dependency_collector_evidence(), expected_identity, {}, lambda *_args: None,
                    artifact_root, "/fixture/b3sum", None,
                )
        observation = json.loads((artifact_root / "s3-dependency-tree-observation.json").read_text())
        self.assertEqual(observation["forbidden_runtime_rows"], ["ra-ap-ide"])

    def test_collector_spawn_failure_guards(self):
        """REQ-SOP-RUNNER-022.D: post-collection guard runs even when process spawn fails."""
        expected_identity = self.make_dependency_collector_identity()
        artifact_root = self.root_path / "artifacts" / "spawn-failure"
        phases = []

        def failed_popen(_command, **_kwargs):
            raise FileNotFoundError("harness cargo spawn failure")

        def fake_b3sum(_command, **_kwargs):
            return SimpleNamespace(returncode=0, stdout=("e" * 64 + "  output\n").encode(), stderr=b"")

        with patch.object(runner.subprocess, "Popen", side_effect=failed_popen), \
             patch.object(runner.subprocess, "run", side_effect=fake_b3sum):
            result = runner.collect_dependency_tree_reference(
                self.make_dependency_collector_evidence(), expected_identity, {},
                lambda phase, _command: phases.append(phase), artifact_root, "/fixture/b3sum", None,
            )
        self.assertIsNone(result)
        self.assertIn("after S3 dependency tree collection", phases)

    def test_s3_intake_failure_priority(self):
        """REQ-SOP-RUNNER-022: a malformed sibling cannot mask a validated failed fixture receipt."""
        fixture = create_identity_bound_s3_fixture(
            self.root_path, fixture_failed=True, malformed_sibling=True,
        )
        evidence = self.prepare_identity_bound_evidence(fixture)
        self.assertEqual(evidence["intake_status"]["status"], "BLOCKED")
        self.assertEqual(
            evidence["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL",
        )
        self.assertEqual(evidence["tokens_by_repository"], {})
        self.assertEqual(
            fixture["run_state"]["manifest"]["s3_prerequisite_intake"]["status"], "BLOCKED",
        )

    def test_s3_identity_failure_stops(self):
        """REQ-SOP-RUNNER-022: wrong manifest identity fails serde before mapping."""
        fixture = create_identity_bound_s3_fixture(self.root_path)
        manifest_document = json.loads(fixture["manifest_path"].read_text())
        manifest_document["identity_binding"]["run_id"] = "wrong-run"
        fixture["manifest_path"].write_text(json.dumps(manifest_document))
        evidence = self.prepare_identity_bound_evidence(fixture)
        self.assertEqual(evidence["qualification_status"], "FAIL")
        self.assertEqual(evidence["validation_error"][0], "FAIL")
        self.assertTrue(all(
            item["status"] == "FAIL" for item in evidence["repository_results"]["serde"].values()
        ))

        repository_name = "serde"
        source_root = fixture["corpus_roots"][repository_name]
        artifact_root = self.root_path / "artifacts" / "identity-failure"
        artifact_root.mkdir(parents=True)
        coverage_summary = {
            "scope_literal_sop_pass": True, "scope_coverage_entry_count": 1,
            "missed_source_file_count": 0, "index_generation_timestamp_value": "generation-a",
            "index_graph_node_count": 1, "index_graph_edge_count": 1,
            "index_partial_file_count": 0, "index_unusable_file_count": 0,
            "scope_truncated_informational": False, "response_digest_manifest_path": "digest.json",
        }

        def collect_coverage(*positional, **_keywords):
            positional[6]({"generation": "generation-a"})
            return dict(coverage_summary)

        execution_context = {
            "artifact_run_directory": artifact_root,
            "corpus_roots": {repository_name: source_root},
            "corpus_records": {repository_name: {
                "corpus_repository_short_name": repository_name,
                "corpus_source_checkout_path": str(source_root),
            }},
            "source_snapshots": {repository_name: {
                "head_commit": "fixture-head", "source_tree_sha256": "a" * 64,
            }},
            "indexer_path": Path("/fixture/indexer"),
            "mapper_identity": {"canonical_path": "/fixture/mapper", "sha256": "a" * 64},
            "identity_guard": lambda *_args: None,
            "b3_provider": "python-blake3:harness-only", "b3_tool_identity": None,
            "s3_evidence": evidence, "s3_expected_identity": fixture["expected_identity"],
        }
        with patch.object(runner, "run_fresh_index_stage", return_value={
            "command": ["/fixture/indexer", "cli", "index_repository", "--repo-path", str(source_root),
                        "--mode", "full"],
            "result": {"status": "indexed", "indexed_at": "generation-a"},
        }), patch.object(runner.coverage_inventory, "collect_index_coverage_inventory",
                         side_effect=collect_coverage), \
             patch.object(runner.production_measurements, "measure_corpus_production_stages") as measure_mapper:
            status = runner.execute_repository_sop_stages(
                repository_name, fixture["run_state"], execution_context,
                SimpleNamespace(milestone="M1"),
            )
        repository = fixture["run_state"]["manifest"]["repositories"][repository_name]
        self.assertEqual(status, "FAIL")
        self.assertEqual(repository["steps"]["S3"], "FAIL")
        self.assertFalse((artifact_root / repository_name / "production").exists())
        measure_mapper.assert_not_called()
        self.assertEqual(fixture["run_state"]["manifest"]["repositories"]["tokio"]["steps"]["S1"],
                         "NOT STARTED")

    def test_s3_intake_error_priority(self):
        """REQ-SOP-RUNNER-022: unavailable toolchain cannot replace an observed intake FAIL."""
        current = ("FAIL", "trusted manifest identity mismatch")
        self.assertEqual(
            runner.merge_s3_intake_error(current, ("BLOCKED", "cargo or rustc unavailable")), current,
        )

    def test_s3_native_skip_blocks(self):
        """REQ-SOP-RUNNER-022: Cargo-configured native skips block fixture evidence and tokens."""
        fixture = create_identity_bound_s3_fixture(self.root_path, native_skip=True)
        evidence = self.prepare_identity_bound_evidence(fixture)
        self.assertEqual(evidence["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")
        self.assertEqual(evidence["tokens_by_repository"], {})

    def test_workspace_inherited_proc_macro(self):
        """REQ-SOP-RUNNER-022: inherited edition and proc-macro targets are inventoried from Cargo TOML."""
        root = self.root_path / "inherited-proc-macro"
        (root / "src").mkdir(parents=True)
        (root / "Cargo.toml").write_text(
            '[package]\nname="fixture-macro"\nversion="0.1.0"\nedition.workspace=true\n'
            '[workspace]\n[workspace.package]\nedition="2024"\n[lib]\nproc-macro=true\n'
        )
        (root / "src/lib.rs").write_text("extern crate proc_macro;\n")
        inventory = runner.read_workspace_unit_inventory(root)
        self.assertEqual(inventory["selected_units"], [{
            "manifest_path": str((root / "Cargo.toml").resolve()),
            "target_name": "fixture_macro", "kind": ["proc-macro"], "edition": "2024",
            "src_path": str((root / "src/lib.rs").resolve()),
        }])

    def test_workspace_custom_crate_blocks(self):
        """REQ-SOP-RUNNER-022: unsupported Cargo crate types block P2 instead of disappearing."""
        root = self.root_path / "unsupported-crate-type"
        (root / "src").mkdir(parents=True)
        (root / "Cargo.toml").write_text(
            '[package]\nname="fixture-lib"\nversion="0.1.0"\nedition="2021"\n'
            '[lib]\ncrate-type=["cdylib"]\n'
        )
        (root / "src/lib.rs").write_text("pub fn fixture() {}\n")
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.read_workspace_unit_inventory(root)

    def test_workspace_external_source_blocks(self):
        """REQ-SOP-RUNNER-022: Cargo targets outside the pinned checkout are unsupported."""
        root = self.root_path / "external-target"
        (root / "src").mkdir(parents=True)
        outside_source = self.root_path / "external.rs"
        outside_source.write_text("pub fn external() {}\n")
        (root / "Cargo.toml").write_text(
            f'[package]\nname="external-target"\nversion="0.1.0"\nedition="2021"\n'
            f'[lib]\npath="{outside_source}"\n'
        )
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.read_workspace_unit_inventory(root)

    def build_output_validation_fixture(self):
        project_root = self.root_path / "output-project"
        project_root.mkdir()
        corpus_roots = {}
        manifest_records = []
        for repository_name in runner.DEFAULT_REPOSITORY_ORDER:
            corpus_root = self.root_path / "output-corpora" / repository_name
            corpus_root.mkdir(parents=True)
            corpus_roots[repository_name] = corpus_root
            manifest_records.append({
                "corpus_repository_short_name": repository_name,
                "corpus_source_checkout_path": str(corpus_root),
            })
        manifest_path = self.root_path / "output-corpus-manifest.json"
        manifest_path.write_text(json.dumps(manifest_records))
        return project_root, corpus_roots, manifest_path

    def run_output_validation_plan(self, project_root, manifest_path, report_root, artifact_root, run_id):
        argument_values = [
            "--run-id", run_id, "--milestone", "M1", "--mode", "plan",
            "--corpus-manifest", str(manifest_path), "--report-root", str(report_root),
        ]
        if artifact_root is not None:
            argument_values.extend(["--artifact-root", str(artifact_root)])
        arguments = runner.build_runner_argument_parser().parse_args(argument_values)
        with patch.object(runner, "PROJECT_ROOT", project_root):
            return runner.run_persistent_qualification_plan(arguments)

    def build_corpus_lock_fixture(self, repository_name, ignored):
        source_root = self.root_path / f"lock-{repository_name}"
        file_values = {
            "Cargo.toml": f"[package]\nname='{repository_name}'\nversion='0.1.0'\n",
            "src/lib.rs": "pub fn fixture() {}\n",
        }
        if ignored:
            file_values[".gitignore"] = "Cargo.lock\n"
        create_committed_git_repository(source_root, file_values)
        lock_path = source_root / "Cargo.lock"
        lock_path.write_bytes(f"# {repository_name} frozen lock\n".encode())
        return source_root, lock_path

    def build_selected_lock_fixture(self):
        project_root = self.root_path / "lock-project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='runner-fixture'\nversion='0.1.0'\n",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        serde_root, serde_lock = self.build_corpus_lock_fixture("serde", True)
        tokio_root, tokio_lock = self.build_corpus_lock_fixture("tokio", False)
        return project_root, serde_root, serde_lock, tokio_root, tokio_lock

    def build_complete_publication_banner(self):
        return {
            "run_generation_digest_value": "published-generation-sha256",
            "source_tiling_identity_holds": True,
            "source_file_record_count": 1,
            "stage_publication_status_values": {
                "crate-graph": "complete",
                "source-ingest": "complete",
                "syntax-spans": "complete",
                "name-resolution": "complete",
            },
        }

    def test_output_checkout_root_blocks(self):
        """REQ-SOP-RUNNER-015: reject checkout-root report output before persistence."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        run_id = "checkout-root-output"
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, project_root, self.root_path / "artifacts", run_id
            )
        self.assertFalse((project_root / run_id).exists())

    def test_output_source_ancestor_blocks(self):
        """REQ-SOP-RUNNER-015: reject report roots that are ancestors of project sources."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        report_root = project_root.parent
        run_id = "project-ancestor-output"
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_root, self.root_path / "artifacts", run_id
            )
        self.assertFalse((report_root / run_id).exists())

    def test_output_source_subtree_blocks(self):
        """REQ-SOP-RUNNER-015: reject artifact roots within project source subtrees."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        report_root = self.root_path / "safe-reports"
        artifact_root = project_root / "src"
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_root, artifact_root, "source-subtree-output"
            )
        self.assertFalse((report_root / "source-subtree-output").exists())

    def test_corpus_ancestor_root_blocks(self):
        """REQ-SOP-RUNNER-015: reject artifact roots that are corpus ancestors."""
        project_root, corpus_roots, manifest_path = self.build_output_validation_fixture()
        artifact_root = corpus_roots["serde"].parent
        report_root = self.root_path / "safe-reports"
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_root, artifact_root, "corpus-ancestor-output"
            )
        self.assertFalse((report_root / "corpus-ancestor-output").exists())

    def test_corpus_root_output_blocks(self):
        """REQ-SOP-RUNNER-015: reject report output at a pinned corpus root."""
        project_root, corpus_roots, manifest_path = self.build_output_validation_fixture()
        report_root = corpus_roots["serde"]
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_root, self.root_path / "artifacts", "corpus-root-output"
            )
        self.assertFalse((report_root / "corpus-root-output").exists())

    def test_output_symlink_bypass_blocks(self):
        """REQ-SOP-RUNNER-015: reject symlink aliases into source trees."""
        project_root, corpus_roots, manifest_path = self.build_output_validation_fixture()
        output_alias = self.root_path / "artifact-source-alias"
        output_alias.symlink_to(corpus_roots["serde"], target_is_directory=True)
        report_root = self.root_path / "safe-reports"
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_root, output_alias, "symlink-source-output"
            )
        self.assertFalse((report_root / "symlink-source-output").exists())

    def test_output_symlink_alias_blocks(self):
        """REQ-SOP-RUNNER-015: reject symlink-equivalent external report roots."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        external_target = self.root_path / "external-report-target"
        external_target.mkdir()
        report_alias = self.root_path / "external-report-alias"
        report_alias.symlink_to(external_target, target_is_directory=True)
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            self.run_output_validation_plan(
                project_root, manifest_path, report_alias, self.root_path / "artifacts", "symlink-alias-output"
            )
        self.assertFalse((external_target / "symlink-alias-output").exists())

    def test_output_safe_paths_validate(self):
        """REQ-SOP-RUNNER-015: accept the exact default report root and external roots."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        default_report_root = project_root / "hld-current-test-runs"
        default_result = self.run_output_validation_plan(
            project_root, manifest_path, default_report_root, None, "valid-default-output"
        )
        self.assertEqual(default_result["overall_status"], "NOT STARTED")
        self.assertTrue((default_report_root / "valid-default-output").is_dir())
        external_result = self.run_output_validation_plan(
            project_root, manifest_path, self.root_path / "external-reports",
            self.root_path / "external-artifacts", "valid-external-output",
        )
        self.assertEqual(external_result["overall_status"], "NOT STARTED")

    def test_outputs_block_before_writes(self):
        """REQ-SOP-RUNNER-015: report unsafe output configuration to stderr without writes."""
        project_root, _, manifest_path = self.build_output_validation_fixture()
        artifact_root = self.root_path / "unsafe-artifacts"
        stderr = StringIO()
        argument_values = [
            "--run-id", "unsafe-before-write", "--milestone", "M1", "--mode", "plan",
            "--corpus-manifest", str(manifest_path), "--report-root", str(project_root),
            "--artifact-root", str(artifact_root),
        ]
        with patch.object(runner, "PROJECT_ROOT", project_root), patch.object(runner.sys, "stderr", stderr):
            return_code = runner.main(argument_values)
        self.assertEqual(return_code, 2)
        self.assertEqual(json.loads(stderr.getvalue())["overall_status"], "BLOCKED")
        self.assertFalse((project_root / "unsafe-before-write").exists())
        self.assertFalse(artifact_root.exists())

    def test_output_exclusions_detect_mutations(self):
        """REQ-SOP-RUNNER-015: only owned run directories are excluded from source snapshots."""
        source_root = self.root_path / "snapshot-output-project"
        create_committed_git_repository(source_root, {
            ".gitignore": "hld-current-test-runs/\n",
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        report_run_directory = source_root / "hld-current-test-runs" / "owned-run"
        artifact_run_directory = self.root_path / "artifact-runs" / "owned-run"
        report_run_directory.mkdir(parents=True)
        artifact_run_directory.mkdir(parents=True)
        (report_run_directory / "hld-current-test-report.md").write_text("owned report\n")
        exclusions = runner.build_owned_run_exclusions(report_run_directory, artifact_run_directory)
        self.assertEqual(exclusions, [report_run_directory.resolve(), artifact_run_directory.resolve()])
        registry = runner.capture_identity_registry(source_root, [], {}, exclusions, [])
        unexpected_source = source_root / "hld-current-test-runs" / "unowned-run" / "src" / "rogue.rs"
        unexpected_source.parent.mkdir(parents=True)
        unexpected_source.write_text("pub fn rogue() {}\n")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_selected_locks_are_frozen(self):
        """REQ-SOP-RUNNER-016: selected ignored and untracked Cargo.lock bytes enter identity evidence."""
        project_root, serde_root, serde_lock, tokio_root, tokio_lock = self.build_selected_lock_fixture()
        registry = runner.capture_identity_registry(
            project_root, [], {"serde": serde_root, "tokio": tokio_root}, [], []
        )
        expected_provenance = ((serde_root, serde_lock, "ignored"), (tokio_root, tokio_lock, "untracked"))
        for source_root, lock_path, provenance in expected_provenance:
            snapshot = registry["source_snapshots"][str(source_root.resolve())]
            lock_evidence = snapshot["permitted_lock_files"]["Cargo.lock"]
            self.assertEqual(lock_evidence["provenance"], provenance)
            self.assertEqual(lock_evidence["identity"]["sha256"], hashlib.sha256(lock_path.read_bytes()).hexdigest())
            self.assertEqual(registry["lock_files"][str(lock_path.absolute())]["sha256"],
                             hashlib.sha256(lock_path.read_bytes()).hexdigest())

    def test_selected_lock_modification_fails(self):
        """REQ-SOP-RUNNER-016: selected ignored lock mutation fails frozen identity checks."""
        project_root, serde_root, serde_lock, tokio_root, _ = self.build_selected_lock_fixture()
        registry = runner.capture_identity_registry(
            project_root, [], {"serde": serde_root, "tokio": tokio_root}, [], []
        )
        serde_lock.write_bytes(b"changed selected lock\n")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_selected_lock_deletion_fails(self):
        """REQ-SOP-RUNNER-016: selected untracked lock deletion fails frozen identity checks."""
        project_root, serde_root, _, tokio_root, tokio_lock = self.build_selected_lock_fixture()
        registry = runner.capture_identity_registry(
            project_root, [], {"serde": serde_root, "tokio": tokio_root}, [], []
        )
        tokio_lock.unlink()
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_unselected_lock_stays_blocked(self):
        """REQ-SOP-RUNNER-016: ignored Cargo.lock files outside the explicit allowlist stay blocked."""
        project_root = self.root_path / "unselected-lock-project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='runner-fixture'\nversion='0.1.0'\n",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        ripgrep_root, _ = self.build_corpus_lock_fixture("ripgrep", True)
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.capture_identity_registry(project_root, [], {"ripgrep": ripgrep_root}, [], [])

    def test_publication_map_accepts_complete(self):
        """REQ-SOP-RUNNER-017: complete publication statuses pass with a generation digest."""
        result = measurements.validate_publication_stage_markers(
            self.build_complete_publication_banner(), "name-resolution"
        )
        self.assertTrue(result["pass"], result["reason"])

    def test_publication_map_requires_stages(self):
        """REQ-SOP-RUNNER-017: incomplete published stage values cannot pass."""
        banner = self.build_complete_publication_banner()
        banner["stage_publication_status_values"]["syntax-spans"] = "staged"
        result = measurements.validate_publication_stage_markers(banner, "source-ingest")
        self.assertFalse(result["pass"])

    def test_publication_map_requires_generation(self):
        """REQ-SOP-RUNNER-017: publication completion requires its published generation identity."""
        banner = self.build_complete_publication_banner()
        banner.pop("run_generation_digest_value")
        result = measurements.validate_publication_stage_markers(banner, "source-ingest")
        self.assertFalse(result["pass"])

    def test_static_status_cannot_publish(self):
        """REQ-SOP-RUNNER-017: static implementation completion and vacuous tiling do not qualify publication."""
        banner = {
            "run_generation_digest_value": "published-generation-sha256",
            "source_tiling_identity_holds": True,
            "source_file_record_count": 3,
            "stage_implementation_status_values": {
                "crate-graph": "complete",
                "source-ingest": "complete",
                "syntax-spans": "complete",
                "name-resolution": "complete",
            },
        }
        result = measurements.validate_publication_stage_markers(banner, "name-resolution")
        self.assertFalse(result["pass"])
        self.assertIn("missing", result["reason"])

    def test_report_initializes_repo_states(self):
        """REQ-SOP-RUNNER-001: initialize five repositories as NOT STARTED."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "fixture-run", "M1")
        manifest = json.loads(Path(run_state["manifest_path"]).read_text())
        self.assertEqual(manifest["repositories"]["serde"]["steps"]["S1"], "NOT STARTED")
        self.assertEqual(manifest["repositories"]["serde"]["s3_components"]["mapping"], "NOT STARTED")
        self.assertEqual(len(manifest["repositories"]), 5)

    def test_s3_component_never_qualifies(self):
        """REQ-SOP-RUNNER-009: successful mapping components leave full S3 BLOCKED."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "s3-component", "M1")
        repository_state = run_state["manifest"]["repositories"]["serde"]
        repository_state["s3_components"].update({component: "PASS" for component in runner.S3_COMPONENT_NAMES})
        result = runner.finalize_s3_component_qualification(run_state, "serde")
        self.assertEqual(result, "BLOCKED")
        self.assertEqual(repository_state["steps"]["S3"], "BLOCKED")
        self.assertEqual(repository_state["s3_qualification_gates"]["fixture_suite"]["status"], "NOT STARTED")
        self.assertEqual(run_state["manifest"]["overall_status"], "BLOCKED")

    def test_s3_component_cannot_promote(self):
        """REQ-SOP-RUNNER-022: component-only summaries cannot promote supplied prerequisite PASS values."""
        state = runner.create_persistent_run_report(self.root_path / "runs", "component-only", "M1")
        repository = state["manifest"]["repositories"]["serde"]
        repository["s3_components"] = {name: "PASS" for name in runner.S3_COMPONENT_NAMES}
        repository["s3_qualification_gates"] = {name: {"status": "PASS", "reason": None}
                                                 for name in runner.S3_GATE_NAMES}
        result = runner.finalize_s3_component_qualification(
            state, "serde", qualification_token=None, expected_identity_binding={"run_id": "component-only"},
        )
        self.assertEqual(result, "BLOCKED")
        self.assertEqual(repository["steps"]["S3"], "BLOCKED")
        self.assertEqual(repository["s3_qualification_gates"]["fixture_suite"]["status"], "PASS")

    def test_s3_fail_precedes_blocked(self):
        """REQ-SOP-RUNNER-022: any prerequisite FAIL takes priority over BLOCKED gates."""
        state = runner.create_persistent_run_report(self.root_path / "runs", "failed-gate", "M1")
        repository = state["manifest"]["repositories"]["serde"]
        repository["s3_components"] = {name: "PASS" for name in runner.S3_COMPONENT_NAMES}
        repository["s3_qualification_gates"] = {
            name: {"status": "FAIL" if name == "fixture_suite" else "BLOCKED", "reason": "synthetic"}
            for name in runner.S3_GATE_NAMES
        }
        result = runner.finalize_s3_component_qualification(
            state, "serde", qualification_token=None, expected_identity_binding={"run_id": "failed-gate"},
        )
        self.assertEqual(result, "FAIL")
        self.assertEqual(repository["steps"]["S3"], "FAIL")

    def test_s3_prerequisite_blocks_mapper(self):
        """REQ-SOP-RUNNER-022: public PASS summaries without an internal token stop before output or mapping."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "intake-blocked", "M1")
        artifact_run_directory = self.root_path / "artifacts" / "intake-blocked"
        artifact_run_directory.mkdir(parents=True)
        source_root = self.root_path / "source"
        source_root.mkdir()
        missing_inputs = runner.read_s3_evidence_inputs(None, "python-blake3:test", None)
        expected_identity = {"run_id": "intake-blocked", "milestone": "M1"}
        s3_evidence = runner.prepare_s3_evidence_qualification(
            SimpleNamespace(run_id="intake-blocked", milestone="M1", s3_prerequisite_evidence=None),
            run_state, missing_inputs, expected_identity, {}, {"tests": [], "ambiguous_reason": None},
        )
        s3_evidence["repository_results"]["serde"] = {
            name: {"status": "PASS", "reason": "forged public summary"} for name in runner.S3_GATE_NAMES
        }
        s3_evidence["tokens_by_repository"] = {}
        gate_results = s3_evidence["repository_results"]["serde"]
        execution_context = {
            "artifact_run_directory": artifact_run_directory,
            "corpus_roots": {"serde": source_root},
            "corpus_records": {"serde": {"corpus_repository_short_name": "serde",
                                            "corpus_source_checkout_path": str(source_root)}},
            "source_snapshots": {"serde": {"head_commit": "fixture-head", "source_tree_sha256": "a" * 64}},
            "indexer_path": Path("/fixture/indexer"),
            "mapper_identity": {"canonical_path": "/fixture/mapper", "sha256": "b" * 64},
            "identity_guard": lambda *_args: None,
            "b3_provider": "python-blake3:fixture", "b3_tool_identity": None,
            "s3_evidence": s3_evidence,
            "s3_expected_identity": expected_identity,
        }
        coverage_summary = {
            "scope_literal_sop_pass": True, "scope_coverage_entry_count": 1,
            "missed_source_file_count": 0, "index_generation_timestamp_value": "generation-a",
            "index_graph_node_count": 1, "index_graph_edge_count": 1,
            "index_partial_file_count": 0, "index_unusable_file_count": 0,
            "scope_truncated_informational": False, "response_digest_manifest_path": "digest.json",
        }

        def collect_coverage(*positional, **_keywords):
            positional[6]({"generation": "generation-a"})
            return dict(coverage_summary)

        with patch.object(runner, "run_fresh_index_stage", return_value={
            "command": ["/fixture/indexer", "cli", "index_repository", "--repo-path", str(source_root), "--mode", "full"],
            "result": {"status": "indexed", "indexed_at": "generation-a"},
        }), patch.object(runner.coverage_inventory, "collect_index_coverage_inventory",
                         side_effect=collect_coverage), \
             patch.object(runner.production_measurements, "measure_corpus_production_stages") as measure_mapper:
            status = runner.execute_repository_sop_stages(
                "serde", run_state, execution_context,
                SimpleNamespace(run_id="intake-blocked", milestone="M1", s3_prerequisite_evidence=None),
            )

        self.assertEqual(status, "BLOCKED")
        self.assertEqual(run_state["manifest"]["repositories"]["serde"]["steps"]["S2"], "PASS")
        self.assertEqual(run_state["manifest"]["repositories"]["serde"]["steps"]["S3"], "BLOCKED")
        self.assertEqual(
            {name: gate["status"] for name, gate in
             run_state["manifest"]["repositories"]["serde"]["s3_qualification_gates"].items()},
            {name: "PASS" for name in runner.S3_GATE_NAMES},
        )
        self.assertFalse((artifact_run_directory / "serde" / "production").exists())
        measure_mapper.assert_not_called()
        self.assertEqual(run_state["manifest"]["repositories"]["rust-analyzer"]["steps"]["S1"], "NOT STARTED")

    def create_evidence_registry_fixture(self, fixture_name):
        project_root = self.root_path / fixture_name / "project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        evidence_path = self.root_path / fixture_name / "external-evidence.json"
        evidence_path.parent.mkdir(parents=True, exist_ok=True)
        evidence_path.write_text("initial receipt bytes")
        registry = runner.capture_identity_registry(
            project_root, (), {}, (), (), extra_frozen_paths=(evidence_path,),
        )
        return evidence_path, registry

    def build_s3_manifest_reference(self, relative_path, content=b"member"):
        return {
            "path": relative_path,
            "size_bytes": len(content),
            "sha256": hashlib.sha256(content).hexdigest(),
            "blake3": "0" * 64,
        }

    def build_s3_manifest_file(self, manifest_path, references):
        manifest_path.write_text(json.dumps({
            "global_prerequisites": {"fixture_suite": references},
            "repositories": {},
        }))
        return manifest_path

    def test_intake_mutated_bytes_fail(self):
        """REQ-SOP-RUNNER-022: changed manifest, receipt, or log bytes fail intake."""
        original_read_bytes = Path.read_bytes
        for target_name in ("manifest", "receipt", "stdout"):
            with self.subTest(target_name=target_name):
                case_root = self.root_path / f"mutated-{target_name}"
                case_root.mkdir()
                manifest_path = case_root / "manifest.json"
                receipt_path = case_root / "receipts" / "receipt.json"
                stdout_path = case_root / "logs" / "stdout.log"
                stderr_path = case_root / "logs" / "stderr.log"
                receipt_path.parent.mkdir()
                stdout_path.parent.mkdir()
                stdout_path.write_bytes(b"stdout")
                stderr_path.write_bytes(b"stderr")
                receipt_bytes = json.dumps({
                    "stdout": {"path": "logs/stdout.log"},
                    "stderr": {"path": "logs/stderr.log"},
                }).encode()
                receipt_path.write_bytes(receipt_bytes)
                reference = self.build_s3_manifest_reference("receipts/receipt.json", receipt_bytes)
                self.build_s3_manifest_file(manifest_path, [reference])
                target_path = {
                    "manifest": manifest_path,
                    "receipt": receipt_path,
                    "stdout": stdout_path,
                }[target_name]

                def alter_read_bytes(path):
                    content = original_read_bytes(path)
                    return content + b" altered" if path == target_path else content

                with patch.object(Path, "read_bytes", alter_read_bytes), patch.object(
                    runner, "calculate_b3_file_digest", return_value="0" * 64,
                ):
                    result = runner.read_s3_evidence_inputs(manifest_path, "python-blake3:test", None)
                self.assertEqual(result["intake_error"][0], "FAIL")

    def test_parent_symlink_replacement_fails(self):
        """REQ-SOP-RUNNER-022: replacing a frozen file parent with same-byte symlink fails."""
        project_root = self.root_path / "parent-link-project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        parent_path = self.root_path / "evidence-parent"
        parent_path.mkdir()
        evidence_path = parent_path / "receipt.json"
        evidence_path.write_bytes(b"same evidence bytes")
        registry = runner.capture_identity_registry(
            project_root, (), {}, (), (), extra_frozen_paths=(evidence_path,),
        )
        real_parent = self.root_path / "evidence-parent-real"
        parent_path.rename(real_parent)
        parent_path.symlink_to(real_parent, target_is_directory=True)
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_external_member_overlap_blocks(self):
        """REQ-SOP-RUNNER-022: external manifest members inside source/output roots block."""
        project_root = self.root_path / "project"
        project_root.mkdir()
        source_path = project_root / "src.rs"
        source_path.write_bytes(b"source member")
        artifact_root = self.root_path / "artifacts"
        artifact_root.mkdir()
        artifact_path = artifact_root / "artifact.log"
        artifact_path.write_bytes(b"artifact member")
        manifest_path = self.root_path / "external-manifest.json"
        references = [
            self.build_s3_manifest_reference("project/src.rs", source_path.read_bytes()),
            self.build_s3_manifest_reference("artifacts/artifact.log", artifact_path.read_bytes()),
        ]
        self.build_s3_manifest_file(manifest_path, references)
        result = runner.read_s3_evidence_inputs(
            manifest_path, "python-blake3:test", None, forbidden_roots=(project_root, artifact_root),
        )
        self.assertEqual(result["intake_error"][0], "BLOCKED")
        self.assertIn("overlaps a source/output root", result["intake_error"][1])

    def test_config_appears_pre_freeze(self):
        """REQ-SOP-RUNNER-022: absent Cargo config appearing before registry is rejected."""
        project_root = self.root_path / "config-project-before"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        config_path = self.root_path / "cargo-home-before" / ".cargo" / "config.toml"
        initial_identity = runner.capture_optional_frozen_file_identity(config_path)
        config_path.parent.mkdir(parents=True)
        config_path.write_text("[build]\ntarget-dir='late'\n")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.capture_identity_registry(
                project_root, (), {}, (), (), optional_frozen_paths=(config_path,),
                expected_optional_files={str(config_path.absolute()): initial_identity},
            )

    def test_config_appears_post_freeze(self):
        """REQ-SOP-RUNNER-022: absent Cargo config appearing after registry is rejected."""
        project_root = self.root_path / "config-project-after"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        config_path = self.root_path / "cargo-home-after" / ".cargo" / "config.toml"
        initial_identity = runner.capture_optional_frozen_file_identity(config_path)
        registry = runner.capture_identity_registry(
            project_root, (), {}, (), (), optional_frozen_paths=(config_path,),
            expected_optional_files={str(config_path.absolute()): initial_identity},
        )
        config_path.parent.mkdir(parents=True)
        config_path.write_text("[build]\ntarget-dir='late'\n")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_receipt_appears_pre_freeze(self):
        """REQ-SOP-RUNNER-022: missing receipt appearing before registry is rejected."""
        project_root = self.root_path / "receipt-project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        manifest_path = self.root_path / "receipt-manifest.json"
        reference = self.build_s3_manifest_reference("missing-receipt.json")
        self.build_s3_manifest_file(manifest_path, [reference])
        evidence_inputs = runner.read_s3_evidence_inputs(manifest_path, "python-blake3:test", None)
        receipt_path = manifest_path.parent / reference["path"]
        self.assertFalse(evidence_inputs["optional_identities"][str(receipt_path.absolute())]["exists"])
        receipt_path.write_bytes(b"receipt appeared")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.capture_identity_registry(
                project_root, (), {}, (), (), extra_frozen_paths=evidence_inputs["frozen_paths"],
                optional_frozen_paths=evidence_inputs["optional_paths"],
                expected_frozen_files=evidence_inputs["frozen_identities"],
                expected_optional_files=evidence_inputs["optional_identities"],
            )

    def test_intake_fail_over_blocks(self):
        """REQ-SOP-RUNNER-022: later intake FAIL overrides earlier unsafe-path BLOCKED."""
        manifest_path = self.root_path / "intake-priority.json"
        directory_path = self.root_path / "member-directory"
        directory_path.mkdir()
        references = [
            self.build_s3_manifest_reference("../blocked-first.json"),
            self.build_s3_manifest_reference("member-directory"),
        ]
        self.build_s3_manifest_file(manifest_path, references)
        result = runner.read_s3_evidence_inputs(manifest_path, "python-blake3:test", None)
        self.assertEqual(result["intake_error"][0], "FAIL")
        self.assertIn("not a regular non-symlink file", result["intake_error"][1])

    def test_frozen_evidence_mutation_fails(self):
        """REQ-SOP-RUNNER-022: post-freeze receipt mutation fails identity verification."""
        evidence_path, registry = self.create_evidence_registry_fixture("evidence-mutation")
        evidence_path.write_text("changed receipt bytes")
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_frozen_evidence_deletion_fails(self):
        """REQ-SOP-RUNNER-022: post-freeze receipt deletion fails identity verification."""
        evidence_path, registry = self.create_evidence_registry_fixture("evidence-deletion")
        evidence_path.unlink()
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_evidence_symlink_retarget_fails(self):
        """REQ-SOP-RUNNER-022: post-freeze symlink retargeting fails identity verification."""
        project_root = self.root_path / "evidence-symlink" / "project"
        create_committed_git_repository(project_root, {
            "Cargo.toml": "[package]\nname='registry-fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        first_target = self.root_path / "evidence-symlink" / "first.json"
        second_target = self.root_path / "evidence-symlink" / "second.json"
        first_target.write_text("first")
        second_target.write_text("second")
        evidence_path = self.root_path / "evidence-symlink" / "receipt.json"
        evidence_path.symlink_to(first_target)
        registry = runner.capture_identity_registry(
            project_root, (), {}, (), (), extra_frozen_paths=(evidence_path,),
        )
        evidence_path.unlink()
        evidence_path.symlink_to(second_target)
        with self.assertRaises(runner.RunnerFrozenInputMutationError):
            runner.verify_identity_registry(registry)

    def test_preflight_failure_affects_overall(self):
        """REQ-SOP-RUNNER-011: terminal preflight BLOCKED contributes to overall status."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "preflight-status", "M1")
        run_state["manifest"]["preflight"]["status"] = "BLOCKED"
        self.assertEqual(runner.calculate_overall_run_status(run_state["manifest"]), "BLOCKED")

    def test_artifact_digest_affects_overall(self):
        """REQ-SOP-RUNNER-011: final artifact digest failures affect overall status."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "digest-status", "M1")
        run_state["manifest"]["artifact_digest_status"] = "FAIL"
        self.assertEqual(runner.calculate_overall_run_status(run_state["manifest"]), "FAIL")
        self.assertIn("NOT STARTED", Path(run_state["report_path"]).read_text())

    def test_existing_run_refuses_overwrite(self):
        """REQ-SOP-RUNNER-001: run directories are exclusive and never resumed."""
        runner.create_persistent_run_report(self.root_path / "runs", "same-run", "M1")
        with self.assertRaises(FileExistsError):
            runner.create_persistent_run_report(self.root_path / "runs", "same-run", "M1")

    def test_error_persists_repo_states(self):
        """REQ-SOP-RUNNER-005: persist one failure and leave later repositories untouched."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "failure-run", "M1")
        runner.record_persistent_run_transition(
            run_state, "serde", "S1", "FAIL", ["indexer", "index"], "observed output violation", "S1.generation"
        )
        manifest = json.loads(Path(run_state["manifest_path"]).read_text())
        self.assertEqual(manifest["repositories"]["serde"]["steps"]["S1"], "FAIL")
        self.assertEqual(manifest["repositories"]["ripgrep"]["steps"]["S1"], "NOT STARTED")
        self.assertIn("S1.generation", Path(run_state["report_path"]).read_text())

    def test_m1_s10_stays_unstarted(self):
        """REQ-SOP-RUNNER-005: M1 S10 stays independent of S1-S3."""
        state = runner.create_persistent_run_report(self.root_path / "runs", "m1-gates", "M1")
        manifest = json.loads(Path(state["manifest_path"]).read_text())
        self.assertEqual(manifest["milestone_gates"]["S10"]["status"], "NOT STARTED")

    def test_m2_exports_remain_blocked(self):
        """REQ-SOP-RUNNER-005: M2 S4/S5 require validated external evidence."""
        state = runner.create_persistent_run_report(self.root_path / "runs", "m2-gates", "M2")
        manifest = json.loads(Path(state["manifest_path"]).read_text())
        self.assertIn(manifest["milestone_gates"]["S4"]["status"], {"NOT STARTED", "BLOCKED"})
        self.assertIn(manifest["milestone_gates"]["S5"]["status"], {"NOT STARTED", "BLOCKED"})

    def test_missing_executable_blocks_preflight(self):
        """REQ-SOP-RUNNER-002: real execution requires one existing executable path."""
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_canonical_runner_executable(self.root_path / "missing-mapper")

    def test_binary_mutation_fails_boundary(self):
        """REQ-SOP-RUNNER-002: detect canonical executable bytes changing after freeze."""
        executable = self.root_path / "mapper"
        executable.write_bytes(b"first executable bytes")
        executable.chmod(0o755)
        frozen = runner.validate_canonical_runner_executable(executable)
        executable.write_bytes(b"changed executable bytes")
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.verify_frozen_file_identity(frozen)

    def test_lock_mutation_fails_boundary(self):
        """REQ-SOP-RUNNER-002: detect Cargo.lock content changes at a stage boundary."""
        lock_file = self.root_path / "Cargo.lock"
        lock_file.write_bytes(b"lock-v1")
        frozen = runner.capture_frozen_file_identity(lock_file)
        lock_file.write_bytes(b"lock-v2")
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.verify_frozen_file_identity(frozen)

    def test_cli_imports_avoid_bytecode(self):
        """REQ-SOP-RUNNER-002: default CLI imports do not create source-tree bytecode."""
        project_root = self.root_path / "cli-project"
        tools_directory = project_root / "tools"
        tools_directory.mkdir(parents=True)
        support_files = (
            "hld-current-qualification-runner.py",
            "s3-prerequisite-evidence-intake.py",
            "corpus-index-coverage-inventory.py",
            "corpus-production-stage-measurements.py",
            "process-tree-access-auditor.py",
        )
        for filename in support_files:
            shutil.copy2(TOOLS_ROOT / filename, tools_directory / filename)

        environment = dict(os.environ)
        environment.pop("PYTHONDONTWRITEBYTECODE", None)
        environment.pop("PYTHONPYCACHEPREFIX", None)
        result = subprocess.run(
            [sys.executable, str(tools_directory / support_files[0]), "--help"],
            cwd=project_root, env=environment, capture_output=True, text=True, check=False,
        )
        pyc_paths = sorted(path.relative_to(project_root).as_posix()
                           for path in project_root.rglob("*.pyc"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("usage:", result.stdout.lower())
        self.assertEqual(pyc_paths, [])

    def test_source_mutation_fails_boundary(self):
        """REQ-SOP-RUNNER-002: detect tracked source bytes changing at a stage boundary."""
        source_file = self.root_path / "src.rs"
        source_file.write_bytes(b"fn source() {}\n")
        frozen = runner.capture_frozen_file_identity(source_file)
        source_file.write_bytes(b"fn changed() {}\n")
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.verify_frozen_file_identity(frozen)

    def test_vendor_build_sources_block(self):
        """REQ-SOP-RUNNER-013: ignored vendor and build source closure stays BLOCKED."""
        source_root = self.root_path / "vendor-source"
        create_committed_git_repository(source_root, {
            ".gitignore": "vendor/\nbuild/\n",
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        vendor_source = source_root / "vendor/dependency/src/lib.rs"
        vendor_binary = source_root / "vendor/dependency/lib/archive.bin"
        build_source = source_root / "build/generated.rs"
        vendor_source.parent.mkdir(parents=True)
        vendor_binary.parent.mkdir(parents=True)
        build_source.parent.mkdir(parents=True)
        vendor_source.write_text("pub fn vendor_input() {}\n")
        vendor_binary.write_bytes(b"vendor dependency bytes")
        build_source.write_text("pub fn build_input() {}\n")
        with self.assertRaises(runner.RunnerPreconditionBlockedError) as context:
            runner.capture_frozen_source_snapshot(source_root)
        self.assertIn("vendor/dependency/src/lib.rs", str(context.exception))
        self.assertIn("vendor/dependency/lib/archive.bin", str(context.exception))
        self.assertIn("build/generated.rs", str(context.exception))

    def test_tracked_binary_changes_snapshot(self):
        """REQ-SOP-RUNNER-013: every tracked byte participates in the source digest."""
        source_root = self.root_path / "tracked-binary-source"
        create_committed_git_repository(source_root, {
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
            "assets/fixture.bin": b"fixture-bytes-one",
        })
        before_snapshot = runner.capture_frozen_source_snapshot(source_root)
        binary_path = source_root / "assets/fixture.bin"
        binary_path.write_bytes(b"fixture-bytes-two")
        subprocess.run(["git", "add", "--", "assets/fixture.bin"], cwd=source_root, check=True,
                       capture_output=True, text=True)
        subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             "commit", "--quiet", "-m", "update binary fixture"],
            cwd=source_root, check=True, capture_output=True, text=True,
        )
        after_snapshot = runner.capture_frozen_source_snapshot(source_root)
        self.assertNotEqual(before_snapshot["source_tree_sha256"], after_snapshot["source_tree_sha256"])

    def test_dirty_snapshot_fails_boundary(self):
        """REQ-SOP-RUNNER-011: post-freeze dirty Git state is an integrity FAIL."""
        source_root = self.root_path / "dirty-source"
        create_committed_git_repository(source_root, {
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        registry = runner.capture_identity_registry(source_root, [], {}, [], [])
        (source_root / "src/lib.rs").write_text("pub fn changed() {}\n")
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.verify_identity_registry(registry)

    def test_symlink_target_change_fails(self):
        """REQ-SOP-RUNNER-013: tracked symlink targets are bound to their content bytes."""
        source_root = self.root_path / "symlink-source"
        create_committed_git_repository(source_root, {
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
            "src/target.rs": "pub fn original() {}\n",
        })
        link_path = source_root / "src/external.rs"
        link_path.symlink_to("target.rs")
        subprocess.run(["git", "add", "--", "src/external.rs"], cwd=source_root, check=True,
                       capture_output=True, text=True)
        subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             "commit", "--quiet", "-m", "add external symlink"],
            cwd=source_root, check=True, capture_output=True, text=True,
        )
        registry = runner.capture_identity_registry(source_root, [], {}, [], [])
        (source_root / "src/target.rs").write_text("pub fn changed() {}\n")
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.verify_identity_registry(registry)

    def test_directory_symlink_blocks_snapshot(self):
        """REQ-SOP-RUNNER-013: directory symlinks with unresolved closure stay BLOCKED."""
        source_root = self.root_path / "directory-link-source"
        external_directory = self.root_path / "external-directory"
        external_directory.mkdir()
        (external_directory / "lib.rs").write_text("pub fn external() {}\n")
        create_committed_git_repository(source_root, {
            "Cargo.toml": "[package]\nname='fixture'\nversion='0.1.0'\n",
            "Cargo.lock": "",
            "src/lib.rs": "pub fn fixture() {}\n",
        })
        (source_root / "vendor-link").symlink_to(external_directory, target_is_directory=True)
        subprocess.run(["git", "add", "--", "vendor-link"], cwd=source_root, check=True,
                       capture_output=True, text=True)
        subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
             "commit", "--quiet", "-m", "add directory symlink"],
            cwd=source_root, check=True, capture_output=True, text=True,
        )
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.capture_frozen_source_snapshot(source_root)

    def test_pin_mismatch_blocks_parser(self):
        """REQ-SOP-RUNNER-002: R1 parser/oracle/corpus pins and patch digest must agree."""
        project = self.root_path / "project"
        phase = project / "docs/phase-20261111"
        tools = project / "tools"
        phase.mkdir(parents=True)
        tools.mkdir()
        patch_file = tools / "parser.patch"
        patch_file.write_bytes(b"pinned patch")
        digest = hashlib.sha256(patch_file.read_bytes()).hexdigest()
        (phase / "corpus.json").write_text(json.dumps([{
            "corpus_repository_short_name": "rust-analyzer",
            "corpus_source_commit_hash": "commit-a",
        }]))
        (phase / "parser.json").write_text(json.dumps({
            "parser_source_commit_hash": "commit-a",
            "parser_source_patch_path": "tools/parser.patch",
            "parser_source_patch_digest": digest,
        }))
        (phase / "oracle.json").write_text(json.dumps({"oracle_source_commit_hash": "commit-a"}))
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.validate_repository_pin_manifest(
                project, phase / "corpus.json", phase / "parser.json", phase / "oracle.json", patch_file
            )

    def test_stale_generation_blocks_scope(self):
        """REQ-SOP-RUNNER-003: stale coverage generations are BLOCKED, not incomplete PASS."""
        page = create_coverage_scope_page(generation_matches=False)
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_coverage_page_sequence([page], [create_coverage_missed_page()])

    def test_scope_cursor_cycle_blocks(self):
        """REQ-SOP-RUNNER-003: repeated scope offsets cannot complete pagination."""
        pages = [
            create_coverage_scope_page(offset=0, next_offset=10, has_more=True, entries=["a"]),
            create_coverage_scope_page(offset=10, next_offset=10, has_more=True, entries=["b"]),
        ]
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_coverage_page_sequence(pages, [create_coverage_missed_page()])

    def test_missed_cursor_cycle_blocks(self):
        """REQ-SOP-RUNNER-003: a repeated missed-File cursor cannot complete coverage."""
        scopes = [create_coverage_scope_page()]
        missed = [
            create_coverage_missed_page(rows=["a"], has_more=True, next_cursor="cursor-a", total=2),
            create_coverage_missed_page(rows=["b"], has_more=True, next_cursor="cursor-a", total=2),
        ]
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_coverage_page_sequence(scopes, missed)

    def test_malformed_coverage_page_blocks(self):
        """REQ-SOP-RUNNER-003: malformed or absent coverage fields remain BLOCKED."""
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_coverage_page_sequence([{}], [create_coverage_missed_page()])

    def test_complete_coverage_pages_validate(self):
        """REQ-SOP-RUNNER-003: complete scope and empty missed-File pages validate."""
        result = runner.validate_coverage_page_sequence(
            [create_coverage_scope_page()], [create_coverage_missed_page(total=0)]
        )
        self.assertEqual(result["scope_entry_count"], 1)
        self.assertEqual(result["missed_file_count"], 0)

    def test_empty_missed_pages_validate(self):
        """REQ-SOP-RUNNER-003: an empty exhaustive missed-File result is valid evidence."""
        result = runner.validate_coverage_page_sequence(
            [create_coverage_scope_page()], [create_coverage_missed_page(rows=[], total=0)]
        )
        self.assertEqual(result["status"], "PASS")

    def test_scope_total_mismatch_blocks(self):
        """REQ-SOP-RUNNER-003: accumulated scope rows must equal each stable total."""
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_coverage_page_sequence(
                [create_coverage_scope_page(total=2)], [create_coverage_missed_page(total=0)]
            )

    def test_boolean_scope_total_blocks(self):
        """REQ-SOP-RUNNER-003: boolean scope totals are malformed pagination data."""
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_scope_response_pages([create_coverage_scope_page(total=True)])

    def test_boolean_missed_total_blocks(self):
        """REQ-SOP-RUNNER-003: boolean missed totals are malformed pagination data."""
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_missed_response_pages(
                [create_coverage_missed_page(total=True)], "generation-a"
            )

    def test_scope_duplicate_identity_blocks(self):
        """REQ-SOP-RUNNER-019: duplicate scope paths cannot satisfy complete pagination."""
        first_paths = [f"src/{index}.rs" for index in range(20)]
        first_paths[0] = "src/lib.rs"
        pages = [
            create_coverage_scope_page(offset=0, next_offset=20, has_more=True,
                                       entries=first_paths, total=21),
            create_coverage_scope_page(offset=20, has_more=False,
                                       entries=["src/lib.rs"], total=21),
        ]
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_scope_response_pages(pages, expected_generation="generation-a")

    def test_scope_identity_missing_blocks(self):
        """REQ-SOP-RUNNER-019: scope entries without a path identity are BLOCKED."""
        page = create_coverage_scope_page(
            entries=[{"kind": "not_indexed_file", "detail": "missing"}], total=1,
        )
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_scope_response_pages([page], expected_generation="generation-a")

    def test_scope_content_conflict_blocks(self):
        """REQ-SOP-RUNNER-019: repeated scope paths with conflicting content are BLOCKED."""
        first_entries = [
            {"path": f"src/{index}.rs", "kind": "file", "detail": "indexed"}
            for index in range(20)
        ]
        first_entries[0]["path"] = "src/lib.rs"
        pages = [
            create_coverage_scope_page(
                offset=0, next_offset=20, has_more=True, entries=first_entries, total=21,
            ),
            create_coverage_scope_page(
                offset=20, has_more=False,
                entries=[{"path": "src/lib.rs", "kind": "not_indexed_file", "detail": "missing"}], total=21,
            ),
        ]
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_scope_response_pages(pages, expected_generation="generation-a")

    def test_missed_duplicate_identity_blocks(self):
        """REQ-SOP-RUNNER-019: duplicate missed File paths cannot satisfy pagination totals."""
        first_paths = [f"src/{index}.rs" for index in range(200)]
        pages = [
            create_coverage_missed_page(rows=first_paths, has_more=True,
                                        next_cursor="cursor-a", total=201),
            create_coverage_missed_page(rows=[first_paths[0]], total=201),
        ]
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_missed_response_pages(pages, "generation-a")

    def test_duplicate_page_stops_collection(self):
        """REQ-SOP-RUNNER-019: live duplicate missed paths stop before another query."""
        report_directory = self.root_path / "reports" / "serde"
        artifact_directory = self.root_path / "artifacts" / "serde"
        report_directory.mkdir(parents=True)
        artifact_directory.mkdir(parents=True)
        (artifact_directory / "codebase-index-result-log.txt").write_text(json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        }))
        scope_text = json.dumps({
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0, "scopes": [{"entries": []}], "has_more": False,
        })
        missed_texts = [json.dumps({
            "columns": ["f.file_path"], "rows": [["src/lib.rs"]], "returned": 1,
            "total_relation": "eq", "has_more": True, "next_cursor": cursor, "total": 201,
        }) for cursor in ("cursor-a", "cursor-b")]
        fake_run, missed_query_count = create_coverage_command_runner(scope_text, missed_texts)
        with patch.object(inventory.subprocess, "run", side_effect=fake_run):
            with self.assertRaises(inventory.CoverageInventoryBlockedError):
                inventory.collect_index_coverage_inventory(
                    "serde", report_directory, "fixture-indexer", "project-fixture", "generation-a",
                    raw_response_directory_path=artifact_directory,
                    digest_file_callback=lambda _path: "b" * 64,
                )
        self.assertEqual(missed_query_count(), 2)
        self.assertEqual(
            (artifact_directory / "codebase-missed-source-page-1.json").read_text(), missed_texts[1]
        )

    def test_missed_identity_missing_blocks(self):
        """REQ-SOP-RUNNER-019: a missed File row without a path identity is BLOCKED."""
        page = create_coverage_missed_page(rows=[None], total=1)
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_missed_response_pages([page], "generation-a")

    def test_empty_pages_cannot_loop(self):
        """REQ-SOP-RUNNER-010: empty pages with continuation cannot outlive declared totals."""
        scope_page = {
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0, "scopes": [{"entries": []}],
            "has_more": True, "next_offset": 20,
        }
        missed_page = {
            "columns": ["f.file_path"], "rows": [], "returned": 0,
            "total_relation": "eq", "has_more": True,
            "next_cursor": "cursor-a", "total": 0,
        }
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_scope_response_pages(
                [{"response": scope_page, "boundary_offset": 0}],
                require_complete=False, expected_generation="generation-a",
            )
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.validate_missed_response_pages(
                [{"response": missed_page, "boundary_cursor": None}],
                "generation-a", require_complete=False,
            )

    def test_missed_cycle_stops_collection(self):
        """REQ-SOP-RUNNER-010: live A-B-A missed cursors stop before another subprocess."""
        report_directory = self.root_path / "reports" / "serde"
        artifact_directory = self.root_path / "artifacts" / "serde"
        report_directory.mkdir(parents=True)
        artifact_directory.mkdir(parents=True)
        (artifact_directory / "codebase-index-result-log.txt").write_text(json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        }))
        scope_text = json.dumps({
            "indexed_at": "generation-a",
            "index_generation": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0,
            "scopes": [{"entries": []}],
            "has_more": False,
        })
        missed_texts = [json.dumps({
            "indexed_at": "generation-a", "columns": ["f.file_path"],
            "rows": [[str(index)]], "returned": 1, "total_relation": "eq", "has_more": True,
            "next_cursor": cursor, "total": 1000,
        }) for index, cursor in enumerate(("cursor-a", "cursor-b", "cursor-a"))]
        fake_run, missed_query_count = create_coverage_command_runner(scope_text, missed_texts)
        with patch.object(inventory.subprocess, "run", side_effect=fake_run):
            with self.assertRaises(inventory.CoverageInventoryBlockedError):
                inventory.collect_index_coverage_inventory(
                    "serde", report_directory, "fixture-indexer", "project-fixture", "generation-a",
                    raw_response_directory_path=artifact_directory,
                    digest_file_callback=lambda _path: "b" * 64,
                )
        self.assertEqual(missed_query_count(), 3)

    def test_stale_missed_generation_stops(self):
        """REQ-SOP-RUNNER-010: stale native missed-page generations stop immediately."""
        report_directory = self.root_path / "reports" / "serde"
        artifact_directory = self.root_path / "artifacts" / "serde"
        report_directory.mkdir(parents=True)
        artifact_directory.mkdir(parents=True)
        (artifact_directory / "codebase-index-result-log.txt").write_text(json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        }))
        scope_text = json.dumps({
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0,
            "scopes": [{"entries": []}],
            "has_more": False,
        })
        stale_page = json.dumps({
            "indexed_at": "generation-stale", "columns": ["f.file_path"],
            "rows": [["row"]], "returned": 1, "total_relation": "eq", "has_more": True,
            "next_cursor": "cursor-next", "total": 10,
        })
        fake_run, missed_query_count = create_coverage_command_runner(scope_text, [stale_page])
        with patch.object(inventory.subprocess, "run", side_effect=fake_run):
            with self.assertRaises(inventory.CoverageInventoryBlockedError):
                inventory.collect_index_coverage_inventory(
                    "serde", report_directory, "fixture-indexer", "project-fixture", "generation-a",
                    raw_response_directory_path=artifact_directory,
                    digest_file_callback=lambda _path: "b" * 64,
                )
        self.assertEqual(missed_query_count(), 1)

    def test_boolean_scope_stops_collection(self):
        """REQ-SOP-RUNNER-010: boolean continuation offsets stop before another query."""
        report_directory = self.root_path / "reports" / "serde"
        artifact_directory = self.root_path / "artifacts" / "serde"
        report_directory.mkdir(parents=True)
        artifact_directory.mkdir(parents=True)
        (artifact_directory / "codebase-index-result-log.txt").write_text(json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        }))
        scope_calls = []
        scope_response_text = json.dumps({
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 2,
            "scopes": [{"entries": [{"path": "entry-a", "kind": "file", "detail": "indexed"}]}],
            "has_more": True,
            "next_offset": True,
        })

        def fake_run(command_values, capture_output, check):
            if "list_projects" in command_values:
                output = "project-fixture\n"
            elif "index_status" in command_values:
                output = json.dumps({
                    "status": "ready", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
                })
            elif "get_graph_schema" in command_values:
                output = json.dumps({"node_labels": [{"label": "File"}], "edge_labels": []})
            elif "check_index_coverage" in command_values:
                scope_calls.append(command_values)
                output = scope_response_text
            else:
                raise AssertionError(f"unexpected query before malformed offset rejection: {command_values!r}")
            return subprocess.CompletedProcess(command_values, 0, output, "")

        with patch.object(inventory.subprocess, "run", side_effect=fake_run):
            with self.assertRaises(inventory.CoverageInventoryBlockedError):
                inventory.collect_index_coverage_inventory(
                    "serde", report_directory, "fixture-indexer", "project-fixture", "generation-a",
                    raw_response_directory_path=artifact_directory,
                    digest_file_callback=lambda _path: "b" * 64,
                )
        self.assertEqual(len(scope_calls), 1)
        self.assertEqual(
            (artifact_directory / "codebase-index-coverage-page-0.json").read_text(), scope_response_text
        )

    def test_raw_pages_stay_external(self):
        """REQ-SOP-RUNNER-010: raw coverage bytes stay external with BLAKE3 locators."""
        report_directory = self.root_path / "reports" / "serde"
        artifact_directory = self.root_path / "artifacts" / "serde"
        report_directory.mkdir(parents=True)
        artifact_directory.mkdir(parents=True)
        (artifact_directory / "codebase-index-result-log.txt").write_text(json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        }))
        scope_text = json.dumps({
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0,
            "scopes": [{"entries": []}],
            "has_more": False,
        }, separators=(",", ":"))
        missed_text = json.dumps({
            "columns": ["f.file_path"], "rows": [], "returned": 0,
            "total_relation": "eq", "has_more": False,
            "total": 0, "index_generation": "generation-a",
        }, separators=(",", ":"))
        fake_run, _ = create_coverage_command_runner(scope_text, [missed_text])
        with patch.object(inventory.subprocess, "run", side_effect=fake_run):
            summary = inventory.collect_index_coverage_inventory(
                "serde", report_directory, "fixture-indexer", "project-fixture", "generation-a",
                raw_response_directory_path=artifact_directory,
                digest_file_callback=lambda _path: "b" * 64,
            )
        raw_missed_page = artifact_directory / "codebase-missed-source-page-0.json"
        self.assertEqual(raw_missed_page.read_text(), missed_text)
        raw_scope_page = artifact_directory / "codebase-index-coverage-page-0.json"
        self.assertEqual(raw_scope_page.read_text(), scope_text)
        self.assertEqual(json.loads(raw_missed_page.read_text())["index_generation"], "generation-a")
        digest_manifest = json.loads((report_directory / "coverage-response-digest-manifest.json").read_text())
        self.assertEqual(digest_manifest["algorithm"], "blake3")
        missed_digest_entry = next(
            entry for entry in digest_manifest["files"]
            if entry["path"].endswith("codebase-missed-source-page-0.json")
        )
        self.assertEqual(missed_digest_entry["size_bytes"], len(missed_text))
        self.assertEqual(summary["index_generation_timestamp_value"], "generation-a")

    def test_invalid_digest_blocks_manifest(self):
        """REQ-SOP-RUNNER-010: malformed BLAKE3 output cannot be stored as evidence."""
        report_directory = self.root_path / "reports"
        artifact_directory = self.root_path / "artifacts"
        report_directory.mkdir()
        artifact_directory.mkdir()
        response_path = artifact_directory / "response.json"
        response_path.write_bytes(b"{}\n")
        with self.assertRaises(inventory.CoverageInventoryBlockedError):
            inventory.write_response_digest_manifest(
                report_directory, artifact_directory, [response_path], lambda _path: "z" * 64, {}
            )

    def test_serde_executes_before_ripgrep(self):
        """REQ-SOP-RUNNER-003: corpus execution order starts with serde."""
        attempts = []
        result = runner.run_ordered_repository_sequence(
            ["ripgrep", "tokio", "serde"], lambda name: attempts.append(name) or "PASS"
        )
        self.assertEqual(attempts[0], "serde")
        self.assertEqual(result["attempted_repositories"], attempts)

    def test_serde_failure_stops_corpus(self):
        """REQ-SOP-RUNNER-005: serde failure leaves every later repository NOT STARTED."""
        attempts = []
        result = runner.run_ordered_repository_sequence(
            ["ripgrep", "serde", "tokio", "cargo-modules"],
            lambda name: attempts.append(name) or ("FAIL" if name == "serde" else "PASS"),
        )
        self.assertEqual(attempts, ["serde"])
        self.assertEqual(result["repository_statuses"]["ripgrep"], "NOT STARTED")

    def test_component_completion_continues_corpus(self):
        """REQ-SOP-RUNNER-009: component completion continues without claiming full PASS."""
        attempted = []
        result = runner.run_ordered_repository_sequence(
            runner.DEFAULT_REPOSITORY_ORDER,
            lambda name: attempted.append(name) or "COMPONENTS COMPLETE",
        )
        self.assertEqual(attempted, list(runner.DEFAULT_REPOSITORY_ORDER))
        self.assertEqual(result["overall_status"], "COMPONENTS COMPLETE")

    def test_missing_duckdb_settings_blocks(self):
        """REQ-SOP-RUNNER-006: absent effective DuckDB settings are BLOCKED."""
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_m2_duckdb_handoff({"published": True}, {"run_id": "run-a"})

    def test_handoff_digest_mismatch_fails(self):
        """REQ-SOP-RUNNER-006: copied table digest disagreement is an observed contract FAIL."""
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "source_generation": "generation-a", "duckdb_generation": "generation-a",
            "published": True, "cancelled": False,
            "tables": [{"table_name": "spans", "source_row_count": 3, "duckdb_row_count": 3,
                        "source_content_blake3": "a" * 64, "duckdb_content_blake3": "b" * 64}],
            "duckdb_settings": {"autoload_known_extensions": False, "autoinstall_known_extensions": False,
                                "enable_external_access": False, "memory_limit": "1.2 GiB",
                                "temp_directory": str(self.root_path / "run-a" / "tmp")},
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
                    "artifact_root": str(self.root_path / "run-a")}
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.validate_m2_duckdb_handoff(evidence, expected)

    def test_negative_handoff_count_blocks(self):
        """REQ-SOP-RUNNER-006: negative table counts are malformed R4 evidence."""
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "source_generation": "generation-a", "duckdb_generation": "generation-a",
            "published": True, "cancelled": False,
            "tables": [{"table_name": "spans", "source_row_count": -1, "duckdb_row_count": -1,
                        "source_content_blake3": "a" * 64, "duckdb_content_blake3": "a" * 64}],
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a"}
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_m2_duckdb_handoff(evidence, expected)

    def test_valid_handoff_records_settings(self):
        """REQ-SOP-RUNNER-006: valid R4 evidence records settings without a RAM threshold."""
        temporary_path = self.root_path / "run-a" / "tmp"
        temporary_path.mkdir(parents=True)
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "source_generation": "generation-a", "duckdb_generation": "generation-a",
            "published": True, "cancelled": False,
            "tables": [{"table_name": "spans", "source_row_count": 1, "duckdb_row_count": 1,
                        "source_content_blake3": "a" * 64, "duckdb_content_blake3": "a" * 64}],
            "duckdb_settings": {"autoload_known_extensions": False, "autoinstall_known_extensions": False,
                                "enable_external_access": False, "memory_limit": "1.2 GiB",
                                "temp_directory": str(temporary_path)},
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
                    "artifact_root": str(self.root_path / "run-a")}
        result = runner.validate_m2_duckdb_handoff(evidence, expected)
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["memory_acceptance_status"], "DEFERRED")

    def test_m2_component_stays_blocked(self):
        """REQ-SOP-RUNNER-012: self-asserted M2 validator success is not corpus qualification."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "m2-components", "M2")
        evidence_root = {"repositories": {name: {} for name in runner.DEFAULT_REPOSITORY_ORDER}}
        arguments = SimpleNamespace(
            milestone="M2", r4_evidence=self.root_path / "r4.json", r5_evidence=self.root_path / "r5.json"
        )
        execution_context = {
            "mapper_identity": {"sha256": "a" * 64},
            "source_snapshots": {name: {"source_tree_sha256": "b" * 64}
                                 for name in runner.DEFAULT_REPOSITORY_ORDER},
            "artifact_run_directory": self.root_path / "artifacts/m2-components",
        }
        with patch.object(runner, "read_structured_evidence_file", return_value=evidence_root), \
                patch.object(runner, "validate_m2_duckdb_handoff", return_value={"status": "PASS"}), \
                patch.object(runner, "validate_m2_path_partition", return_value={"status": "PASS"}):
            runner.validate_external_m2_evidence(run_state, arguments, execution_context)
        for repository_name in runner.DEFAULT_REPOSITORY_ORDER:
            for evidence_kind in ("R4", "R5"):
                result = run_state["manifest"]["external_evidence"][evidence_kind][repository_name]
                self.assertEqual(result["validator_component_status"], "PASS")
                self.assertEqual(result["status"], "BLOCKED")
                self.assertTrue(result["blocked_gates"])

    def test_m2_validator_fail_aggregates(self):
        """REQ-SOP-RUNNER-012: observed M2 validator failures survive corpus aggregation."""
        run_state = runner.create_persistent_run_report(self.root_path / "runs", "m2-failures", "M2")
        evidence_root = {"repositories": {name: {"fixture": True} for name in runner.DEFAULT_REPOSITORY_ORDER}}
        arguments = SimpleNamespace(
            milestone="M2", r4_evidence=self.root_path / "r4.json", r5_evidence=self.root_path / "r5.json"
        )
        execution_context = {
            "mapper_identity": {"sha256": "a" * 64},
            "source_snapshots": {name: {"source_tree_sha256": "b" * 64}
                                 for name in runner.DEFAULT_REPOSITORY_ORDER},
            "artifact_run_directory": self.root_path / "artifacts/m2-failures",
        }
        for repository_name, state in run_state["manifest"]["repositories"].items():
            state["steps"]["S1"] = "PASS"
            state["steps"]["S2"] = "PASS"
            state["s3_components"] = {component: "PASS" for component in state["s3_components"]}

        def validate_r4(_evidence, identity):
            if identity["repository_name"] == "serde":
                raise runner.RunnerContractViolationError("R4 source/count digest mismatch")
            return {"status": "PASS"}

        def validate_r5(_evidence, identity):
            if identity["repository_name"] == "rust-analyzer":
                raise runner.RunnerContractViolationError("R5 duplicate partition rows")
            return {"status": "PASS"}

        with patch.object(runner, "read_structured_evidence_file", return_value=evidence_root), \
                patch.object(runner, "validate_m2_duckdb_handoff", side_effect=validate_r4), \
                patch.object(runner, "validate_m2_path_partition", side_effect=validate_r5):
            runner.finalize_unimplemented_milestone_gates(run_state, execution_context, arguments)

        manifest = run_state["manifest"]
        self.assertEqual(manifest["external_evidence"]["R4"]["serde"]["validator_component_status"], "FAIL")
        self.assertEqual(manifest["external_evidence"]["R4"]["serde"]["status"], "FAIL")
        self.assertEqual(manifest["external_evidence"]["R5"]["rust-analyzer"]["validator_component_status"], "FAIL")
        self.assertEqual(manifest["external_evidence"]["R5"]["rust-analyzer"]["status"], "FAIL")
        self.assertEqual(manifest["external_evidence"]["R4"]["ripgrep"]["status"], "BLOCKED")
        self.assertEqual(manifest["external_evidence"]["R5"]["ripgrep"]["status"], "BLOCKED")
        self.assertEqual(manifest["milestone_gates"]["S4"]["status"], "FAIL")
        self.assertEqual(manifest["milestone_gates"]["S5"]["status"], "FAIL")
        self.assertEqual(manifest["overall_status"], "FAIL")

    def test_partition_rejects_duplicate_rows(self):
        """REQ-SOP-RUNNER-006: every raw oracle row appears in exactly one class."""
        evidence = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
                    "primary_export_blake3": "a" * 64, "raw_oracle_row_count": 2,
                    "rows": [{"oracle_row_id": "row-a", "path_lookup_scope_kind": "module-scope", "reason": "module"},
                             {"oracle_row_id": "row-a", "path_lookup_scope_kind": "type-relative", "reason": "type"}]}
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
                    "primary_export_blake3": "a" * 64}
        with self.assertRaises(runner.RunnerContractViolationError):
            runner.validate_m2_path_partition(evidence, expected)

    def test_partition_identity_mismatch_blocks(self):
        """REQ-SOP-RUNNER-006: M2 evidence cannot be reused across frozen inputs."""
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_m2_path_partition({"run_id": "other"}, {"run_id": "run-a"})

    def test_malformed_row_identity_blocks(self):
        """REQ-SOP-RUNNER-006: missing raw row identities keep R5 evidence BLOCKED."""
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "primary_export_blake3": "a" * 64, "raw_oracle_row_count": 1,
            "rows": [{"oracle_row_id": None, "class_name": "module-scope"}],
            "partition_classes": [{"class_name": "module-scope", "row_count": 1,
                                   "reason": "module lookup", "stage_ownership": "export"}],
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a"}
        with self.assertRaises(runner.RunnerPreconditionBlockedError):
            runner.validate_m2_path_partition(evidence, expected)

    def test_valid_partition_counts_rows(self):
        """REQ-SOP-RUNNER-006: exhaustive R5 classes count and own every raw row."""
        export_digest = "a" * 64
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "primary_export_blake3": export_digest, "raw_oracle_row_count": 2,
            "rows": [{"oracle_row_id": "row-a", "class_name": "module-scope"},
                     {"oracle_row_id": "row-b", "class_name": "type-relative"}],
            "partition_classes": [
                {"class_name": "module-scope", "row_count": 1, "reason": "module lookup", "stage_ownership": "export"},
                {"class_name": "type-relative", "row_count": 1, "reason": "type lookup", "stage_ownership": "export"},
            ],
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
                    "primary_export_blake3": export_digest}
        result = runner.validate_m2_path_partition(evidence, expected)
        self.assertEqual(result["raw_oracle_row_count"], 2)
        self.assertEqual(result["partition_class_count"], 2)

    def test_empty_partition_checks_denominator(self):
        """REQ-SOP-RUNNER-006: a valid zero-row denominator has one explicit empty class."""
        evidence = {
            "run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a",
            "primary_export_blake3": "a" * 64, "raw_oracle_row_count": 0, "rows": [],
            "partition_classes": [{"class_name": "no-rows", "row_count": 0,
                                   "reason": "empty raw export", "stage_ownership": "export"}],
        }
        expected = {"run_id": "run-a", "repository_name": "serde", "mapper_sha256": "sha-a"}
        self.assertEqual(runner.validate_m2_path_partition(evidence, expected)["raw_oracle_row_count"], 0)

    def test_large_rss_never_fails(self):
        """REQ-SOP-RUNNER-007: RSS above 500 MB is measurement-only before all-five S5."""
        result = runner.classify_memory_acceptance_measurement(900_000_000, False)
        self.assertEqual(result["acceptance_status"], "DEFERRED")
        self.assertEqual(result["sampled_peak_bytes"], 900_000_000)

    def test_memory_target_remains_deferred(self):
        """REQ-SOP-RUNNER-007: no interim DuckDB or tree RSS acceptance threshold is imposed."""
        result = runner.classify_memory_acceptance_measurement(2_000_000_000, False)
        self.assertNotEqual(result["acceptance_status"], "FAIL")

    def test_real_run_requires_blake3(self):
        """REQ-SOP-RUNNER-008: a real run blocks if no true BLAKE3 provider exists."""
        with patch.object(runner, "find_external_b3_digest_provider", return_value=None):
            with self.assertRaises(runner.RunnerPreconditionBlockedError):
                runner.require_external_b3_digest_provider(False)

    def test_fake_execution_is_harness(self):
        """REQ-SOP-RUNNER-008: a fake mapper subprocess can never produce qualification PASS."""
        fake_executable = self.root_path / "fake-mapper"
        fake_executable.write_text(f"#!{sys.executable}\nimport sys\nprint('harness only')\n")
        fake_executable.chmod(0o755)
        identity = runner.validate_canonical_runner_executable(fake_executable)
        result = runner.run_guarded_child_process(
            [str(fake_executable)], identity, harness_only_flag=True
        )
        self.assertEqual(result["return_code"], 0)
        self.assertEqual(result["qualification_status"], "HARNESS ONLY")

    def test_forbidden_network_trace_fails(self):
        """REQ-SOP-RUNNER-004: forbidden network or build execution is an observed FAIL."""
        trace = (
            '100 execve("/frozen/parseltongue", ["parseltongue"], []) = 0\n'
            '100 connect(3, {sa_family=AF_INET, sin_port=htons(443)}, 16) = 0\n'
            '101 execve("/usr/bin/cargo", ["cargo", "build"], []) = 0\n'
        )
        result = runner.audit_production_process_trace(trace, Path("/frozen/parseltongue"))
        self.assertEqual(result["status"], "FAIL")
        self.assertTrue(result["violations"])

    def test_incomplete_trace_blocks_measurement(self):
        """REQ-SOP-RUNNER-004: missing root/unfinished trace evidence cannot pass S3."""
        result = runner.audit_production_process_trace("", Path("/frozen/parseltongue"))
        self.assertEqual(result["status"], "BLOCKED")

    def test_clean_trace_validates_root(self):
        """REQ-SOP-RUNNER-004: a complete trace rooted at the frozen mapper can validate."""
        trace = (
            '123 execve("/frozen/parseltongue", ["/frozen/parseltongue"], []) = 0\n'
            "123 +++ exited with 0 +++\n"
        )
        result = runner.audit_production_process_trace(trace, Path("/frozen/parseltongue"))
        self.assertEqual(result["status"], "PASS")

    def test_unfinished_process_trace_blocks(self):
        """REQ-SOP-RUNNER-004: unfinished process traces cannot establish a complete audit."""
        trace = '123 execve("/frozen/parseltongue", ["/frozen/parseltongue"], [] <unfinished ...>\n'
        result = runner.audit_production_process_trace(trace, Path("/frozen/parseltongue"))
        self.assertEqual(result["status"], "BLOCKED")

    def test_measurement_uses_frozen_path(self):
        """REQ-SOP-RUNNER-004: telemetry and audits share one explicit mapper identity."""
        source_root = self.root_path / "serde"
        output_root = self.root_path / "artifacts"
        record = {"corpus_repository_short_name": "serde", "corpus_source_checkout_path": str(source_root)}
        commands = measurements.build_corpus_measurement_commands(
            record, output_root, Path("/frozen/parseltongue"), "syntax-spans"
        )
        for command in commands:
            self.assertIn("/frozen/parseltongue", command)
            self.assertIn("syntax-spans", command)
        stores = [next(arg for arg in command if arg.endswith("workspace-store.db")) for command in commands]
        self.assertEqual(len(set(stores)), 3)

    def test_trace_captures_syscall_families(self):
        """REQ-SOP-RUNNER-018: process and network syscall families are traced together."""
        record = {"corpus_source_checkout_path": str(self.root_path / "serde")}
        _, audit_command, _ = measurements.build_corpus_measurement_commands(
            record, self.root_path / "artifacts", Path("/frozen/parseltongue"), "syntax-spans"
        )
        self.assertIn("trace=%process,%network", audit_command)
        self.assertIn("-yy", audit_command)

    def test_execveat_spawn_is_forbidden(self):
        """REQ-SOP-RUNNER-018: successful execveat descendants obey the spawn allowlist."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 clone(child_stack=NULL, flags=SIGCHLD) = 101\n'
            '101 execveat(AT_FDCWD, "/usr/bin/cargo", ["cargo", "build"], 0) = 0\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertTrue(any("forbidden exec /usr/bin/cargo" in row for row in violations))

    def test_internet_send_families_fail(self):
        """REQ-SOP-RUNNER-018: IPv4 and IPv6 send syscall families fail the audit."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 sendto(3, "x", 1, 0, {sa_family=AF_INET, sin_port=htons(53)}, 16) = 1\n'
            '100 sendmsg(3, {msg_name={sa_family=AF_INET6, sin6_port=htons(53)}}, 0) = 1\n'
            '100 sendmmsg(3, [{msg_hdr={msg_name={sa_family=AF_INET, sin_port=htons(53)}}}], 1, 0) = 1\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertEqual(sum("forbidden network" in row for row in violations), 3)

    def test_unaddressed_sends_require_proof(self):
        """REQ-SOP-RUNNER-018: addressless sends fail without known local socket evidence."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 sendto(91, "x", 1, 0, NULL, 0) = 1\n'
            '100 sendmsg(92, {msg_name=NULL, msg_iov=[{iov_base="x", iov_len=1}]}, 0) = 1\n'
            '100 sendmmsg(93, [{msg_hdr={msg_name=NULL}}], 1, 0) = 1\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertEqual(sum("unproven network send" in row for row in violations), 3)

    def test_socketpair_local_send_passes(self):
        """REQ-SOP-RUNNER-018: known inherited AF_UNIX socketpair sends remain allowed."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 socketpair(AF_UNIX, SOCK_STREAM, 0, [3, 4]) = 0\n'
            '100 clone(child_stack=NULL, flags=SIGCHLD) = 101\n'
            '101 sendto(3<UNIX:[1234]>, "x", 1, 0, NULL, 0) = 1\n'
            '101 sendmsg(4<UNIX:[1235]>, {msg_name=NULL}, 0) = 1\n'
        )
        self.assertEqual(auditor.audit_process_trace_events(trace, 100, binary), [])

    def test_annotated_unix_fd_allowed(self):
        """REQ-SOP-RUNNER-018: strace UNIX socket descriptors prove local addressless sends."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 sendto(4<UNIX-STREAM:[1234]>, "x", 1, 0, NULL, 0) = 1\n'
        )
        self.assertEqual(auditor.audit_process_trace_events(trace, 100, binary), [])

    def test_stale_socket_proof_rejected(self):
        """REQ-SOP-RUNNER-018: only per-event UNIX evidence allows addressless sends."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 socketpair(AF_UNIX, SOCK_STREAM, 0, [3, 4]) = 0\n'
            '100 sendto(3, "x", 1, 0, NULL, 0) = 1\n'
            '100 sendto(4<UNIX-STREAM:[1234]>, "x", 1, 0, NULL, 0) = 1\n'
            '100 close(4) = 0\n'
            '100 sendto(4, "x", 1, 0, NULL, 0) = 1\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertEqual(sum("unproven network send" in row for row in violations), 2)

    def test_socketpair_inode_fd_untrusted(self):
        """REQ-SOP-RUNNER-018: socketpair annotation inode digits never prove descriptor identity."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 socketpair(AF_UNIX, SOCK_STREAM, 0, '
            '[3<UNIX-STREAM:[1234]>, 4<UNIX-STREAM:[5678]>]) = 0\n'
            '100 sendto(1234, "x", 1, 0, NULL, 0) = 1\n'
            '100 sendto(5678, "x", 1, 0, NULL, 0) = 1\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertEqual(sum("unproven network send" in row for row in violations), 2)

    def test_execveat_relative_path_blocks(self):
        """REQ-SOP-RUNNER-018: relative execveat identities cannot satisfy the allowlist."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 execveat(AT_FDCWD, "cargo", ["cargo", "metadata", "--offline"], 0) = 0\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertTrue(any("unresolved execveat path" in row for row in violations))

    def test_failed_exec_payload_blocks(self):
        """REQ-SOP-RUNNER-018: undecodable failed exec attempts remain explicit violations."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 execve("/usr/bin/cargo", [not-valid], []) = -1 ENOENT (No such file or directory)\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertTrue(any("undecodable exec argv" in row for row in violations))

    def test_failed_forbidden_exec_blocks(self):
        """REQ-SOP-RUNNER-018: forbidden exec attempts fail even when execve returns an error."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 execve("/bin/sh", ["sh", "-c", "true"], []) = -1 ENOENT (No such file or directory)\n'
        )
        violations = auditor.audit_process_trace_events(trace, 100, binary)
        self.assertTrue(any("forbidden exec attempt /bin/sh" in row for row in violations))

    def test_unix_sends_remain_allowed(self):
        """REQ-SOP-RUNNER-018: local AF_UNIX socket traffic remains allowed."""
        binary = "/frozen/parseltongue"
        trace = (
            f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
            '100 connect(3, {sa_family=AF_UNIX, sun_path="/run/local.sock"}, 24) = 0\n'
            '100 sendmsg(3, {msg_name={sa_family=AF_UNIX, sun_path="/run/local.sock"}}, 0) = 1\n'
        )
        self.assertEqual(auditor.audit_process_trace_events(trace, 100, binary), [])

    def test_measurement_guard_wraps_commands(self):
        """REQ-SOP-RUNNER-004: subprocess probes invoke both identity-boundary callbacks."""
        executable = self.root_path / "fake-command"
        executable.write_text(f"#!{sys.executable}\nprint('ok')\n")
        executable.chmod(0o755)
        identity = runner.validate_canonical_runner_executable(executable)
        phases = []
        result = runner.run_guarded_child_process(
            [str(executable)], identity,
            lambda phase, command: phases.append((phase, list(command))),
            harness_only_flag=True,
        )
        self.assertEqual([phase for phase, _ in phases], ["before", "after"])
        self.assertEqual(result["return_code"], 0)

    def test_mapper_failure_skips_audits(self):
        """REQ-SOP-RUNNER-020: mapper failure skips all later audit and publication commands."""
        output = self.root_path / "mapper-failure-artifacts"
        record = {
            "corpus_repository_short_name": "serde",
            "corpus_source_checkout_path": str(self.root_path / "serde"),
        }
        commands = []

        def run_command(command_values, **_kwargs):
            commands.append(list(command_values))
            return subprocess.CompletedProcess(command_values, 1, "mapper failed", "")

        with patch.object(measurements.subprocess, "run", side_effect=run_command):
            summary = measurements.measure_corpus_production_stages(
                record, output, Path("/frozen/parseltongue"), final_stage_kind="syntax-spans"
            )
        self.assertEqual(summary.get("production_mapper_status"), "FAIL")
        self.assertEqual(summary.get("production_process_audit_status"), "NOT STARTED")
        self.assertEqual(summary.get("production_target_audit_status"), "NOT STARTED")
        self.assertEqual(summary.get("production_status_command_status"), "NOT STARTED")
        self.assertEqual(summary.get("production_total_elapsed_seconds"), None)
        self.assertIsNone(summary.get("production_coverage_banner_values"))
        self.assertEqual(len(commands), 1)
        persisted = json.loads((output / "production-stage-measurement-summary.json").read_text())
        self.assertEqual(persisted["production_process_audit_status"], "NOT STARTED")

    def test_process_failure_skips_status(self):
        """REQ-SOP-RUNNER-020: process audit failure skips target and publication commands."""
        output = self.root_path / "process-failure-artifacts"
        binary = Path("/frozen/parseltongue")
        record = {
            "corpus_repository_short_name": "serde",
            "corpus_source_checkout_path": str(self.root_path / "serde"),
        }
        commands = []

        def run_command(command_values, **_kwargs):
            commands.append(list(command_values))
            if len(commands) == 1:
                sample_path = Path(command_values[command_values.index("--memory-sample-output-path") + 1])
                sample_path.write_text(json.dumps({
                    "process_tree_elapsed_seconds": 1.25,
                    "process_tree_peak_bytes": 2048,
                    "sample_interval_millisecond_value": 10,
                    "process_memory_sample_values": [{"elapsed_seconds": 0.01, "rss_bytes": 2048}],
                }))
            elif len(commands) == 2:
                trace_path = Path(command_values[command_values.index("-o") + 1])
                trace_path.write_text(
                    f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
                    '100 socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP) = 3\n'
                )
            return subprocess.CompletedProcess(command_values, 0, "", "")

        with patch.object(measurements.subprocess, "run", side_effect=run_command):
            summary = measurements.measure_corpus_production_stages(
                record, output, binary, final_stage_kind="syntax-spans"
            )
        self.assertEqual(summary.get("production_process_audit_status"), "FAIL")
        self.assertEqual(summary.get("production_target_audit_status"), "NOT STARTED")
        self.assertEqual(summary.get("production_status_command_status"), "NOT STARTED")
        self.assertIsNone(summary.get("production_coverage_banner_values"))
        self.assertEqual(len(commands), 2)
        persisted = json.loads((output / "production-stage-measurement-summary.json").read_text())
        self.assertEqual(persisted["production_target_audit_status"], "NOT STARTED")

    def test_mapper_failure_retains_samples(self):
        """REQ-SOP-RUNNER-020: failed mapper runs retain genuine completed telemetry."""
        output = self.root_path / "mapper-failure-samples"
        record = {
            "corpus_repository_short_name": "serde",
            "corpus_source_checkout_path": str(self.root_path / "serde"),
        }
        commands = []
        sample = {
            "process_tree_elapsed_seconds": 2.5,
            "process_tree_peak_bytes": 4096,
            "sample_interval_millisecond_value": 10,
            "process_memory_sample_values": [{"elapsed_seconds": 0.01, "rss_bytes": 4096}],
        }

        def run_command(command_values, **_kwargs):
            commands.append(list(command_values))
            sample_path = Path(command_values[command_values.index("--memory-sample-output-path") + 1])
            sample_path.write_text(json.dumps(sample))
            return subprocess.CompletedProcess(command_values, 1, "mapper failed", "")

        with patch.object(measurements.subprocess, "run", side_effect=run_command):
            summary = measurements.measure_corpus_production_stages(
                record, output, Path("/frozen/parseltongue"), final_stage_kind="syntax-spans"
            )
        self.assertEqual(summary["production_mapper_status"], "FAIL")
        self.assertEqual(summary["production_total_elapsed_seconds"], 2.5)
        self.assertEqual(summary["production_peak_rss_bytes"], 4096)
        self.assertEqual(summary["production_memory_sample_count"], 1)
        self.assertEqual(summary["production_process_audit_status"], "NOT STARTED")
        self.assertEqual(len(commands), 1)

    def test_target_failure_skips_status(self):
        """REQ-SOP-RUNNER-020: failed target audit skips publication status retrieval."""
        output = self.root_path / "target-failure-artifacts"
        binary = Path("/frozen/parseltongue")
        source_root = self.root_path / "serde"
        record = {
            "corpus_repository_short_name": "serde",
            "corpus_source_checkout_path": str(source_root),
        }
        commands = []

        def run_command(command_values, **_kwargs):
            commands.append(list(command_values))
            if len(commands) == 1:
                sample_path = Path(command_values[command_values.index("--memory-sample-output-path") + 1])
                sample_path.write_text(json.dumps({
                    "process_tree_elapsed_seconds": 1.25,
                    "process_tree_peak_bytes": 2048,
                    "sample_interval_millisecond_value": 10,
                    "process_memory_sample_values": [{"elapsed_seconds": 0.01, "rss_bytes": 2048}],
                }))
            elif len(commands) == 2:
                trace_path = Path(command_values[command_values.index("-o") + 1])
                trace_path.write_text(
                    f'100 execve("{binary}", ["parseltongue"], []) = 0\n'
                    '100 +++ exited with 0 +++\n'
                )
            elif len(commands) == 3:
                trace_path = Path(command_values[command_values.index("-o") + 1])
                trace_path.write_text(
                    f'100 openat(AT_FDCWD, "{source_root / "target"}", O_RDONLY) = 3\n'
                    '100 +++ exited with 0 +++\n'
                )
            return subprocess.CompletedProcess(command_values, 0, "", "")

        with patch.object(measurements.subprocess, "run", side_effect=run_command):
            summary = measurements.measure_corpus_production_stages(
                record, output, binary, final_stage_kind="syntax-spans"
            )
        self.assertEqual(summary["production_target_audit_status"], "FAIL")
        self.assertEqual(summary["production_status_command_status"], "NOT STARTED")
        self.assertIsNone(summary["production_status_exit_code"])
        self.assertIsNone(summary["production_coverage_banner_values"])
        self.assertEqual(len(commands), 3)

    def test_rustc_suffix_path_fails(self):
        """REQ-SOP-RUNNER-014: only an exact rustc-info basename is target-exempt."""
        trace = ('123 openat(AT_FDCWD, "/repo/target/not.rustc_info.json", O_RDONLY) = 3\n'
                 '123 +++ exited with 0 +++\n')
        result = measurements.audit_target_path_trace(
            trace, Path("/repo/target"), wrapper_return_code=0
        )
        self.assertFalse(result["pass"])
        self.assertEqual(len(result["violations"]), 1)

    def test_exact_rustc_basename_allowed(self):
        """REQ-SOP-RUNNER-014: exact .rustc_info.json basename remains the sole exemption."""
        trace = ('123 openat(AT_FDCWD, "/repo/target/.rustc_info.json", O_RDONLY) = 3\n'
                 '123 +++ exited with 0 +++\n')
        result = measurements.audit_target_path_trace(
            trace, Path("/repo/target"), wrapper_return_code=0
        )
        self.assertTrue(result["pass"])

    def test_unfinished_target_trace_blocks(self):
        """REQ-SOP-RUNNER-014: unfinished target access traces cannot pass the audit."""
        trace = ('123 openat(AT_FDCWD, "/repo/target/source.rs", O_RDONLY <unfinished ...>\n'
                 '123 +++ exited with 0 +++\n')
        result = measurements.audit_target_path_trace(
            trace, Path("/repo/target"), wrapper_return_code=0
        )
        self.assertFalse(result["complete"])
        self.assertTrue(result["violations"])

    def test_exit_only_target_passes(self):
        """REQ-SOP-RUNNER-014: clean wrapper and traced-process exits prove empty access."""
        result = measurements.audit_target_path_trace(
            "123 +++ exited with 0 +++\n", Path("/repo/target"), wrapper_return_code=0
        )
        self.assertTrue(result["complete"])
        self.assertTrue(result["pass"])
        self.assertEqual(result["violations"], [])

    def test_exit_only_requires_wrapper(self):
        """REQ-SOP-RUNNER-014: exit-only target evidence requires a verified wrapper status."""
        result = measurements.audit_target_path_trace(
            "123 +++ exited with 0 +++\n", Path("/repo/target")
        )
        self.assertFalse(result["complete"])
        self.assertFalse(result["pass"])

    def test_empty_target_trace_blocks(self):
        """REQ-SOP-RUNNER-014: an empty target trace is not evidence of no access."""
        result = measurements.audit_target_path_trace(
            "", Path("/repo/target"), wrapper_return_code=0
        )
        self.assertFalse(result["complete"])
        self.assertFalse(result["pass"])

    def test_truncated_target_trace_blocks(self):
        """REQ-SOP-RUNNER-014: target syscall traces require a terminal process exit."""
        result = measurements.audit_target_path_trace(
            '123 openat(AT_FDCWD, "/repo/target/.rustc_info.json", O_RDONLY) = 3\n',
            Path("/repo/target"), wrapper_return_code=0,
        )
        self.assertFalse(result["complete"])
        self.assertFalse(result["pass"])

    def test_orphan_resumed_target_blocks(self):
        """REQ-SOP-RUNNER-014: an orphan resumed syscall prevents trace completeness."""
        trace = "123 <... openat resumed>) = 3\n123 +++ exited with 0 +++\n"
        result = measurements.audit_target_path_trace(
            trace, Path("/repo/target"), wrapper_return_code=0
        )
        self.assertFalse(result["complete"])
        self.assertTrue(result["violations"])

    def test_toolchain_versions_are_recorded(self):
        """REQ-SOP-RUNNER-009: rustc and cargo versions record explicit unavailability."""
        with patch.object(runner.shutil, "which", return_value=None):
            versions = runner.probe_toolchain_versions()
        self.assertEqual(versions["rustc"]["version"], "unavailable")
        self.assertEqual(versions["cargo"]["version"], "unavailable")

    def test_rustup_shims_preserve_invocation(self):
        """REQ-SOP-RUNNER-009: cargo and rustc shims retain their PATH invocation names."""
        binary_directory = self.root_path / "rustup-bin"
        binary_directory.mkdir()
        shim_path = binary_directory / "rustup"
        shim_path.write_text(
            "#!/bin/sh\n"
            "invoked=${0##*/}\n"
            "printf '%s shim-version\\n' \"$invoked\"\n"
        )
        shim_path.chmod(0o755)
        cargo_path = binary_directory / "cargo"
        rustc_path = binary_directory / "rustc"
        cargo_path.symlink_to(shim_path.name)
        rustc_path.symlink_to(shim_path.name)

        with patch.dict(os.environ, {"PATH": str(binary_directory)}):
            versions = runner.probe_toolchain_versions()

        for tool_name, invocation_path in (("cargo", cargo_path), ("rustc", rustc_path)):
            self.assertEqual(versions[tool_name]["version"], f"{tool_name} shim-version")
            self.assertEqual(versions[tool_name]["path"], str(invocation_path))
            self.assertEqual(versions[tool_name]["canonical_path"], str(shim_path.resolve()))
            self.assertEqual(versions[tool_name]["file_identity"]["path"], str(invocation_path))

    def test_planning_creates_truthful_report(self):
        """REQ-SOP-RUNNER-001: planning persists statuses without mapper or oracle execution."""
        arguments = runner.build_runner_argument_parser().parse_args([
            "--run-id", "planning-run", "--milestone", "M1", "--mode", "plan",
            "--report-root", str(self.root_path / "runs"),
        ])
        result = runner.run_persistent_qualification_plan(arguments)
        self.assertEqual(result["overall_status"], "NOT STARTED")
        manifest = json.loads(Path(result["manifest_path"]).read_text())
        self.assertEqual(manifest["repositories"]["serde"]["steps"]["S1"], "NOT STARTED")

    def test_preflight_failure_persists_report(self):
        """REQ-SOP-RUNNER-005: missing mapper preflight persists BLOCKED without execution."""
        arguments = runner.build_runner_argument_parser().parse_args([
            "--run-id", "blocked-run", "--milestone", "M1", "--mode", "execute",
            "--mapper-binary", str(self.root_path / "missing-mapper"),
            "--report-root", str(self.root_path / "runs"),
            "--artifact-root", str(self.root_path / "artifacts"),
        ])
        result = runner.run_persistent_qualification_plan(arguments)
        manifest = json.loads(Path(result["manifest_path"]).read_text())
        self.assertEqual(result["overall_status"], "BLOCKED")
        self.assertEqual(manifest["preflight"]["status"], "BLOCKED")
        self.assertEqual(manifest["repositories"]["serde"]["steps"]["S1"], "NOT STARTED")
        self.assertIn("missing-mapper", Path(result["report_path"]).read_text())

    def test_artifact_exception_persists_report(self):
        """REQ-SOP-RUNNER-005: unexpected artifact setup exceptions persist as BLOCKED."""
        artifact_root_file = self.root_path / "artifact-root-file"
        artifact_root_file.write_text("not a directory")
        arguments = runner.build_runner_argument_parser().parse_args([
            "--run-id", "artifact-error-run", "--milestone", "M1", "--mode", "execute",
            "--mapper-binary", str(self.root_path / "missing-mapper"),
            "--report-root", str(self.root_path / "runs"),
            "--artifact-root", str(artifact_root_file),
        ])
        result = runner.run_persistent_qualification_plan(arguments)
        manifest = json.loads(Path(result["manifest_path"]).read_text())
        self.assertEqual(manifest["preflight"]["status"], "BLOCKED")

    def test_repository_s3_directory_isolated(self):
        """REQ-SOP-RUNNER-004 and REQ-SOP-RUNNER-021: S1/S2 metadata preserves executed commands."""
        repository_name = "serde"
        report_root = self.root_path / "reports"
        artifact_run_directory = self.root_path / "artifacts" / "s3-layout"
        source_root = self.root_path / "corpora" / repository_name
        source_root.mkdir(parents=True)
        run_state = runner.create_persistent_run_report(
            report_root, "s3-layout", "M1", harness_only_flag=True
        )
        run_state["manifest"]["harness_probe"] = {"qualification_status": "HARNESS ONLY"}
        self.assertEqual(run_state["manifest"]["execution_identity"], "HARNESS ONLY")
        artifact_run_directory.mkdir(parents=True)
        project_identifier = runner.derive_indexer_project_identifier(source_root)
        scope_pages = [
            create_coverage_scope_page(
                offset=0, next_offset=20, has_more=True,
                entries=[f"tools/source-{index}.py" for index in range(20)], total=21,
            ),
            create_coverage_scope_page(
                offset=20, next_offset=40, has_more=False, entries=["tools/source-20.py"], total=21,
            ),
        ]
        missed_pages = [
            create_coverage_missed_page(
                rows=[f"src/file-{index}.rs" for index in range(200)],
                has_more=True, next_cursor="cursor-a", total=201,
            ),
            create_coverage_missed_page(rows=["src/file-last.rs"], total=201),
        ]
        scope_calls = 0
        missed_calls = 0

        def fake_indexer_run(command_values, capture_output, check):
            nonlocal scope_calls, missed_calls
            if "list_projects" in command_values:
                output = project_identifier + "\n"
            elif "index_status" in command_values:
                output = json.dumps({
                    "status": "ready", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
                })
            elif "get_graph_schema" in command_values:
                output = json.dumps({"node_labels": [{"label": "File"}], "edge_labels": []})
            elif "check_index_coverage" in command_values:
                output = json.dumps(scope_pages[scope_calls])
                scope_calls += 1
            elif "query_graph" in command_values:
                output = json.dumps(missed_pages[missed_calls])
                missed_calls += 1
            else:
                raise AssertionError(f"unexpected harness indexer command: {command_values!r}")
            return subprocess.CompletedProcess(command_values, 0, output, "")

        index_result = json.dumps({
            "status": "indexed", "indexed_at": "generation-a", "nodes": 1, "edges": 1,
        })
        production_invocations = []

        def stop_after_real_output_guard(command, output_directory, output_prefix, _environment, _callback):
            production_invocations.append((output_prefix, Path(output_directory), list(command)))
            raise RuntimeError("harness stop after production directory check")

        execution_context = {
            "artifact_run_directory": artifact_run_directory,
            "corpus_roots": {repository_name: source_root},
            "corpus_records": {repository_name: {
                "corpus_repository_short_name": repository_name,
                "corpus_source_checkout_path": str(source_root),
            }},
            "source_snapshots": {repository_name: {
                "head_commit": "fixture-head", "source_tree_sha256": "b" * 64,
            }},
            "indexer_path": Path("/fixture/indexer"),
            "mapper_identity": {"canonical_path": "/fixture/mapper", "sha256": "a" * 64},
            "identity_guard": lambda *_arguments: None,
            "b3_provider": "python-blake3:harness-fixture",
            "b3_tool_identity": None,
        }
        self.grant_synthetic_s3_token(execution_context, run_state, repository_name)

        with patch.object(runner, "run_guarded_child_process", return_value={
                 "return_code": 0, "stdout": index_result, "stdout_bytes": index_result.encode(), "stderr": "",
             }), patch.object(inventory.subprocess, "run", side_effect=fake_indexer_run), \
                patch.object(runner, "calculate_b3_file_digest", return_value="a" * 64), \
                patch.object(runner.production_measurements, "invoke_production_stage_command",
                             side_effect=stop_after_real_output_guard):
            result = runner.execute_repository_sop_stages(
                repository_name, run_state, execution_context, SimpleNamespace(milestone="M1")
            )

        self.assertEqual(result, "BLOCKED")
        self.assertEqual(run_state["manifest"]["overall_status"], "HARNESS ONLY")
        self.assertEqual(run_state["manifest"]["repositories"][repository_name]["steps"]["S1"], "PASS")
        self.assertEqual(run_state["manifest"]["repositories"][repository_name]["steps"]["S2"], "PASS")
        self.assertEqual(
            len(production_invocations), 1,
            run_state["manifest"]["repositories"][repository_name]["first_reason"],
        )
        self.assertEqual(production_invocations[0][0], "production-stage")
        expected_production_directory = artifact_run_directory / repository_name / "production"
        self.assertEqual(production_invocations[0][1], expected_production_directory)
        self.assertIn(str(expected_production_directory / "process-tree-memory-samples.json"),
                      production_invocations[0][2])
        self.assertEqual(scope_calls, 2)
        self.assertEqual(missed_calls, 2)

        command_log_path = artifact_run_directory / repository_name / "commands.jsonl"
        command_events = [json.loads(line) for line in command_log_path.read_text().splitlines()]
        index_events = [event for event in command_events if "index_repository" in event["command"]]
        scope_events = [event for event in command_events
                        if "check_index_coverage" in event["command"]]
        missed_events = [event for event in command_events if "query_graph" in event["command"]]
        self.assertEqual([event["phase"] for event in index_events], ["before", "after"])
        self.assertEqual(index_events[0]["command"], [
            "/fixture/indexer", "cli", "index_repository", "--repo-path", str(source_root), "--mode", "full",
        ])
        self.assertEqual(index_events[1]["outcome"]["return_code"], 0)
        self.assertEqual(index_events[0]["outcome"]["stage"], "S1")
        self.assertEqual(scope_events[0]["outcome"]["stage"], "S2")
        self.assertEqual(len(scope_events), 4)
        self.assertEqual(len(missed_events), 4)
        self.assertTrue(all(event["command"][event["command"].index("--project") + 1] == project_identifier
                            for event in scope_events + missed_events))
        self.assertIn("20", scope_events[2]["command"])
        self.assertEqual(missed_events[2]["command"][missed_events[2]["command"].index("--cursor") + 1],
                         "cursor-a")
        self.assertEqual([event["phase"] for event in scope_events],
                         ["before", "after", "before", "after"])
        self.assertEqual(scope_events[1]["outcome"]["return_code"], 0)
        self.assertNotIn("stdout", scope_events[1]["outcome"])
        s1_transition = next(
            event for event in run_state["manifest"]["repositories"][repository_name]["events"]
            if event["step"] == "S1" and event["status"] == "PASS"
        )
        s1_metadata = s1_transition["command_metadata"]
        expected_s1_argv = [
            ["/fixture/indexer", "cli", "index_repository", "--repo-path", str(source_root), "--mode", "full"],
            ["/fixture/indexer", "cli", "list_projects"],
            ["/fixture/indexer", "cli", "index_status", "--project", project_identifier,
             "--diagnostics", "full", "--format", "json"],
            ["/fixture/indexer", "cli", "get_graph_schema", "--project", project_identifier,
             "--format", "json", "--diagnostics", "full", "--limit", "100", "--offset", "0"],
        ]
        self.assertEqual([event["argv"] for event in s1_metadata], expected_s1_argv)
        self.assertTrue(all(event["cwd"] == str(Path.cwd()) for event in s1_metadata))
        self.assertEqual([event["exit_result"]["return_code"] for event in s1_metadata], [0, 0, 0, 0])
        self.assertFalse(any(
            flag in event["argv"] for event in s1_metadata for flag in ("--manifest-path", "--corpus-root")
        ))
        s2_transition = next(
            event for event in run_state["manifest"]["repositories"][repository_name]["events"]
            if event["step"] == "S2" and event["status"] == "PASS"
        )
        s2_metadata = s2_transition["command_metadata"]
        expected_s2_events = [
            event for event in command_events
            if event["phase"] == "after" and event["outcome"].get("stage") == "S2"
        ]
        self.assertEqual([event["argv"] for event in s2_metadata],
                         [event["command"] for event in expected_s2_events])
        self.assertTrue(all(event["cwd"] == str(Path.cwd()) for event in s2_metadata))
        self.assertEqual([event["exit_result"]["return_code"] for event in s2_metadata],
                         [event["outcome"]["return_code"] for event in expected_s2_events])
        self.assertTrue(any("--scope-offset" in event["argv"] for event in s2_metadata))
        self.assertTrue(any("--cursor" in event["argv"] for event in s2_metadata))

    def test_runner_persists_skipped_states(self):
        """REQ-SOP-RUNNER-020: runner manifest keeps failed mapper and skipped commands distinct."""
        repository_name = "serde"
        source_root = self.root_path / "corpora" / repository_name
        source_root.mkdir(parents=True)
        report_root = self.root_path / "reports"
        artifact_run_directory = self.root_path / "artifacts" / "mapper-failure"
        artifact_run_directory.mkdir(parents=True)
        run_state = runner.create_persistent_run_report(
            report_root, "mapper-failure", "M1", harness_only_flag=True
        )
        generation = "generation-a"
        coverage_summary = {
            "scope_literal_sop_pass": True,
            "scope_coverage_entry_count": 0,
            "missed_source_file_count": 0,
            "index_generation_timestamp_value": generation,
            "index_graph_node_count": 1,
            "index_graph_edge_count": 1,
            "index_partial_file_count": 0,
            "index_unusable_file_count": 0,
            "scope_truncated_informational": False,
            "response_digest_manifest_path": "/fixture/coverage-digests.json",
        }

        def collect_coverage(_name, _report, _indexer, _project, _generation, _guard,
                             on_metadata_complete, _raw, _digest, **_kwargs):
            on_metadata_complete({"generation": generation})
            return dict(coverage_summary)

        measurement_summary = {
            "production_mapper_exit_code": 1,
            "production_mapper_status": "FAIL",
            "production_measurement_status": "FAIL",
            "production_process_audit_status": None,
            "production_process_audit_exit": None,
            "production_target_audit_status": None,
            "production_target_audit_exit": None,
            "production_status_command_status": None,
            "production_status_exit_code": None,
            "production_publication_status": None,
            "production_memory_sample_count": 0,
            "production_peak_rss_bytes": None,
            "production_total_elapsed_seconds": None,
            "production_memory_acceptance_status": "DEFERRED",
            "production_coverage_banner_values": None,
            "publication_stage_marker_result": None,
            "production_artifact_directory": str(artifact_run_directory / repository_name / "production"),
        }
        execution_context = {
            "artifact_run_directory": artifact_run_directory,
            "corpus_roots": {repository_name: source_root},
            "corpus_records": {repository_name: {
                "corpus_repository_short_name": repository_name,
                "corpus_source_checkout_path": str(source_root),
            }},
            "source_snapshots": {repository_name: {
                "head_commit": "fixture-head", "source_tree_sha256": "b" * 64,
            }},
            "indexer_path": Path("/fixture/indexer"),
            "mapper_identity": {"canonical_path": "/fixture/mapper", "sha256": "a" * 64},
            "identity_guard": lambda *_arguments: None,
            "b3_provider": "python-blake3:harness-fixture",
            "b3_tool_identity": None,
        }
        self.grant_synthetic_s3_token(execution_context, run_state, repository_name)
        with patch.object(runner, "run_fresh_index_stage", return_value={
                 "command": ["/fixture/indexer", "cli", "index_repository", "--repo-path", str(source_root),
                             "--mode", "full"],
                 "result": {"status": "indexed", "indexed_at": generation, "nodes": 1, "edges": 1},
             }), patch.object(runner.coverage_inventory, "collect_index_coverage_inventory",
                              side_effect=collect_coverage), \
                patch.object(runner, "calculate_b3_file_digest", return_value="a" * 64), \
                patch.object(runner.production_measurements, "measure_corpus_production_stages",
                             return_value=measurement_summary):
            result = runner.execute_repository_sop_stages(
                repository_name, run_state, execution_context, SimpleNamespace(milestone="M1")
            )

        self.assertEqual(result, "FAIL")
        repository = run_state["manifest"]["repositories"][repository_name]
        self.assertEqual(repository["steps"]["S3"], "FAIL")
        self.assertEqual(repository["s3_measurements"]["production_mapper_status"], "FAIL")
        self.assertIsNone(repository["s3_measurements"]["production_process_audit_status"])
        self.assertIsNone(repository["s3_measurements"]["production_target_audit_status"])
        self.assertIsNone(repository["s3_measurements"]["production_status_command_status"])
        self.assertEqual(repository["s3_components"]["process_access_audit"], "NOT STARTED")
        self.assertEqual(repository["s3_components"]["publication"], "NOT STARTED")

    def test_run_writes_artifact_manifest(self):
        """REQ-SOP-RUNNER-011: orchestration finalizes evidence after frozen-input mutation."""
        fixture = create_preflight_fixture(self.root_path / "fixture")
        report_root = self.root_path / "reports"
        artifact_root = self.root_path / "artifacts"
        arguments = runner.build_runner_argument_parser().parse_args([
            "--run-id", "artifact-integration", "--milestone", "M1", "--mode", "execute",
            "--mapper-binary", str(fixture["binaries"]["mapper"]),
            "--corpus-manifest", str(fixture["corpus_manifest"]),
            "--report-root", str(report_root), "--artifact-root", str(artifact_root),
            "--indexer-path", str(fixture["binaries"]["indexer"]),
        ])
        run_state = runner.create_persistent_run_report(report_root, "artifact-integration", "M1")

        def record_component_success(repository_name, state, context, _arguments):
            self.grant_synthetic_s3_token(context, state, repository_name)
            runner.record_persistent_run_transition(state, repository_name, "S1", "PASS")
            runner.record_persistent_run_transition(state, repository_name, "S2", "PASS")
            repository_state = state["manifest"]["repositories"][repository_name]
            repository_state["s3_components"].update({
                component: "PASS" for component in repository_state["s3_components"]
            })
            repository_state["s3_measurements"] = {
                "generation": f"generation-{repository_name}",
                "production_total_elapsed_seconds": 0.5,
                "production_peak_rss_bytes": 900_000_000,
            }
            artifact_directory = Path(context["artifact_run_directory"]) / repository_name
            artifact_directory.mkdir(parents=True, exist_ok=True)
            (artifact_directory / "fixture-evidence.txt").write_text(repository_name)
            runner.append_persistent_report_entry(
                state, f"- `{repository_name}` generation `generation-{repository_name}`, sampled wall time `0.5s`, "
                "whole-tree RSS `900000000` bytes."
            )
            return runner.finalize_s3_component_qualification(
                state, repository_name, context["s3_evidence"]["tokens_by_repository"][repository_name],
                context["s3_expected_identity"],
            )

        def invalidated_identity_guard(*_arguments):
            raise runner.RunnerContractViolationError("frozen source changed after mapping")

        observed_contexts = []

        def execute_fake_stage(repository_name, state, context, _arguments):
            observed_contexts.append(context)
            result = record_component_success(repository_name, state, context, _arguments)
            if repository_name == runner.DEFAULT_REPOSITORY_ORDER[-1]:
                context["identity_guard"] = invalidated_identity_guard
                state["manifest"]["input_integrity_status"] = "FAIL"
            return result

        fake_blake3 = SimpleNamespace(__file__=None)
        with patch.object(runner, "PROJECT_ROOT", fixture["project_root"]), \
                patch.object(runner, "RUST_ANALYZER_COMMIT", fixture["rust_analyzer_commit"]), \
                patch.object(runner, "R1_PATCH_SHA256", fixture["patch_digest"]), \
                patch.object(runner, "require_external_b3_digest_provider",
                             return_value="python-blake3:fixture"), \
                patch.object(runner.shutil, "which",
                             side_effect=lambda name: str(fixture["binaries"]["strace"]) if name == "strace" else None), \
                patch.dict(sys.modules, {"blake3": fake_blake3}), \
                patch.object(runner, "calculate_b3_file_digest", return_value="a" * 64), \
                patch.object(runner, "run_guarded_child_process", return_value={
                    "return_code": 0, "stdout": "fixture tool version", "stderr": "",
                }), \
                patch.object(runner, "execute_repository_sop_stages", side_effect=execute_fake_stage):
            result = runner.execute_qualification_run(arguments, run_state)

        manifest_path = artifact_root / "artifact-integration" / "artifact-blake3-manifest.json"
        self.assertTrue(manifest_path.is_file())
        self.assertTrue(observed_contexts, {
            "preflight": run_state["manifest"]["preflight"],
            "finalization": run_state["manifest"].get("finalization_status"),
            "report": Path(run_state["report_path"]).read_text(),
        })
        self.assertIn("b3_tool_identity", observed_contexts[-1])
        self.assertEqual(
            observed_contexts[-1]["b3_tool_identity"],
            run_state["manifest"]["input_identity"]["b3_tool_identity"],
        )
        self.assertEqual(result["repository_execution_order"], list(runner.DEFAULT_REPOSITORY_ORDER))
        self.assertEqual(run_state["manifest"]["artifact_digest_status"], "PASS")
        self.assertEqual(run_state["manifest"]["input_integrity_status"], "FAIL")
        self.assertEqual(run_state["manifest"]["overall_status"], "FAIL")
        report_text = Path(run_state["report_path"]).read_text()
        self.assertIn(str(fixture["binaries"]["mapper"]), report_text)
        self.assertIn("whole-tree RSS", report_text)
        self.assertIn("generation-serde", report_text)
        self.assertIn(run_state["manifest"]["input_identity"]["repository_source_snapshots"]["serde"][
            "source_tree_sha256"], report_text)
        self.assertIn("rustc", report_text)
        self.assertIn("unavailable", report_text)
        self.assertIn("artifact-blake3-manifest", report_text)

    def test_persisted_harness_stays_harness(self):
        """REQ-SOP-RUNNER-001: persistent fake execution remains HARNESS ONLY, never PASS."""
        fake_executable = self.root_path / "fake-mapper"
        fake_executable.write_text(f"#!{sys.executable}\nprint('harness only')\n")
        fake_executable.chmod(0o755)
        arguments = runner.build_runner_argument_parser().parse_args([
            "--run-id", "harness-run", "--milestone", "M1", "--mode", "execute", "--harness-only",
            "--mapper-binary", str(fake_executable),
            "--report-root", str(self.root_path / "runs"),
            "--artifact-root", str(self.root_path / "artifacts"),
        ])
        result = runner.run_persistent_qualification_plan(arguments)
        manifest = json.loads(Path(result["manifest_path"]).read_text())
        self.assertEqual(result["overall_status"], "HARNESS ONLY")
        self.assertEqual(manifest["repositories"]["serde"]["steps"]["S1"], "NOT STARTED")


if __name__ == "__main__":
    unittest.main()
