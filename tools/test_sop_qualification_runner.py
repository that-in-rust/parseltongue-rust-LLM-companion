"""REQ-SOP-RUNNER-* contract tests for the Python qualification harness."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
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
        "Cargo.lock": "",
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


def create_coverage_scope_page(generation: str = "generation-a", offset: int = 0, next_offset: int = 1,
                               has_more: bool = False, entries: list[str] | None = None,
                               total: int = 1, generation_matches: bool = True) -> dict:
    return {
        "indexed_at": generation,
        "requested_offset": offset,
        "metadata": {"recording_status": "complete", "generation_matches": generation_matches},
        "scopes": [{"entries": entries if entries is not None else ["tools/a.py"]}],
        "scope_total": total,
        "has_more": has_more,
        "next_offset": next_offset,
    }


def create_coverage_missed_page(generation: str = "generation-a", rows: list[str] | None = None,
                                has_more: bool = False, next_cursor: str | None = None,
                                total: int = 0) -> dict:
    return {
        "indexed_at": generation,
        "rows": rows if rows is not None else [],
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
        repository_state["s3_components"]["mapping"] = "PASS"
        result = runner.finalize_s3_component_qualification(run_state, "serde")
        self.assertEqual(result, "BLOCKED")
        self.assertEqual(repository_state["steps"]["S3"], "BLOCKED")
        self.assertEqual(repository_state["s3_qualification_gates"]["fixture_suite"]["status"], "BLOCKED")
        self.assertEqual(run_state["manifest"]["overall_status"], "BLOCKED")

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

    def test_empty_pages_cannot_loop(self):
        """REQ-SOP-RUNNER-010: empty pages with continuation cannot outlive declared totals."""
        scope_page = {
            "indexed_at": "generation-a",
            "metadata": {"recording_status": "complete", "generation_matches": True},
            "scope_total": 0, "scopes": [{"entries": []}],
            "has_more": True, "next_offset": 20,
        }
        missed_page = {"rows": [], "has_more": True, "next_cursor": "cursor-a", "total": 0}
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
            "indexed_at": "generation-a", "rows": [str(index)], "has_more": True,
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
            "indexed_at": "generation-stale", "rows": ["row"], "has_more": True,
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
            "scopes": [{"entries": ["entry-a"]}],
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
            "rows": [], "has_more": False, "total": 0, "index_generation": "generation-a",
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
        """REQ-SOP-RUNNER-004: S1/S2 artifacts stay separate from the fresh S3 directory."""
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
            return runner.finalize_s3_component_qualification(state, repository_name)

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
