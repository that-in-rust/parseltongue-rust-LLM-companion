"""Synthetic contracts for S3 prerequisite evidence validation."""

from __future__ import annotations

import importlib.util
import hashlib
import json
from pathlib import Path
import sys
import unittest


TOOLS_ROOT = Path(__file__).resolve().parent
MODULE_SPEC = importlib.util.spec_from_file_location(
    "s3_prerequisite_evidence_intake", TOOLS_ROOT / "s3-prerequisite-evidence-intake.py"
)
if MODULE_SPEC is None or MODULE_SPEC.loader is None:
    raise RuntimeError("cannot load S3 prerequisite evidence intake")
intake = importlib.util.module_from_spec(MODULE_SPEC)
sys.modules[MODULE_SPEC.name] = intake
MODULE_SPEC.loader.exec_module(intake)


def synthetic_blake3_digest(content):
    return hashlib.sha256(b"synthetic-blake3:" + content).hexdigest()


def synthetic_fixture_output(inventory, extra_rows=()):
    rows = [
        f"test fixture::{test['name']} ... "
        f"ignored, {test['ignore_reason']}" if test.get("ignore_reason") is not None
        else f"test fixture::{test['name']} ... ok"
        for test in inventory["tests"]
    ]
    rows.extend(extra_rows)
    passed = sum(row.endswith(" ... ok") for row in rows)
    ignored = sum(" ... ignored," in row for row in rows)
    failed = sum(row.endswith(" ... FAILED") for row in rows)
    return "\n".join([
        f"running {len(rows)} tests", *rows,
        f"test result: {'FAILED' if failed else 'ok'}. {passed} passed; {failed} failed; {ignored} ignored; "
        f"0 measured; 0 filtered out; finished in 0.01s",
    ])


class SyntheticEvidenceBundle:
    def __init__(self, milestone="M1", child=False, manual=False):
        self.expected_identity = {
            "run_id": "synthetic-run", "milestone": milestone,
            "mapper": {
                "canonical_path": "/external/parseltongue", "sha256": "a" * 64, "blake3": "b" * 64,
            },
            "runner_source_snapshot": {"source_tree_sha256": "c" * 64},
            "production_cargo_lock": {"sha256": "d" * 64},
            "parser_pin_bundle": {"sha256": "e" * 64}, "r1_patch": {"sha256": "f" * 64},
            "toolchain_versions": {"rustc": {"version": "rustc 1.90.0"},
                                   "cargo": {"version": "cargo 1.90.0", "path": "cargo"}},
            "configuration": {
                "project_root": "/production", "production_manifest": "/production/Cargo.toml",
                "production_package_name": "parseltongue-cli-dispatcher-binary",
                "production_package_version": "0.1.0",
                "production_bin_source": "/production/crates/parseltongue-cli-dispatcher-binary/src/main.rs",
            },
            "relevant_environment": {},
        }
        self.expected_repositories = {}
        self.file_bytes = {}
        self.blake3_by_path = {}
        self.receipt_counter = 0
        self.source_inventory = {
            "tests": [
                {"name": name, "path": "crates/fixture-tests.rs", "ignore_reason": None}
                for name in sorted(intake.M0_TEST_NAMES | intake.M1_TEST_NAMES
                                   | (intake.M2_TEST_NAMES if milestone == "M2" else set()))
            ],
            "ambiguous_reason": None,
        }
        if manual:
            self.source_inventory["tests"].append({
                "name": "test_profile_json_bulk_batches",
                "path": "crates/parseltongue-turso-store-adapter/src/stage-table-store-backend.rs",
                "ignore_reason": "manual JSON bulk insert profile",
            })
        if child:
            child_path = "crates/parseltongue-cli-dispatcher-binary/tests/workspace-cli-contract-tests.rs"
            self.source_inventory["tests"].extend([
                {"name": name, "path": child_path, "ignore_reason": None}
                for name in intake.CHILD_TEST_NAMES
            ])
            self.source_inventory["tests"].append({
                "name": intake.CHILD_IGNORED_TEST[0], "path": child_path,
                "ignore_reason": intake.CHILD_IGNORED_TEST[1],
            })
        self.manifest = None
        self._build()

    def _add_raw_file(self, path_value, content):
        content_bytes = content if isinstance(content, bytes) else content.encode("utf-8")
        self.file_bytes[path_value] = content_bytes
        self.blake3_by_path[path_value] = synthetic_blake3_digest(content_bytes)
        return content_bytes

    def _reference(self, path_value, content):
        content_bytes = self._add_raw_file(path_value, content)
        return {
            "path": path_value, "size_bytes": len(content_bytes),
            "sha256": hashlib.sha256(content_bytes).hexdigest(),
            "blake3": self.blake3_by_path[path_value],
        }

    def _receipt(self, family, scope, argv, stdout="", stderr="", repository_binding=None):
        self.receipt_counter += 1
        receipt_id = f"synthetic-receipt-{self.receipt_counter}"
        stdout_path = f"logs/{receipt_id}.stdout"
        stderr_path = f"logs/{receipt_id}.stderr"
        stdout_bytes = self._add_raw_file(stdout_path, stdout)
        stderr_bytes = self._add_raw_file(stderr_path, stderr)
        receipt = {
            "schema_version": 1, "receipt_id": receipt_id, "family": family, "scope": scope,
            "identity_binding": self.expected_identity, "argv": argv,
            "cwd": repository_binding["checkout_path"] if repository_binding else "/production",
            "relevant_environment": self.expected_identity["relevant_environment"],
            "elapsed_seconds": 0.5, "timeout_seconds": 30.0,
            "completion_status": "completed", "timed_out": False, "exit_status": 0,
            "stdout": {**self._descriptor(stdout_path, stdout_bytes), "complete": True},
            "stderr": {**self._descriptor(stderr_path, stderr_bytes), "complete": True},
        }
        if repository_binding is not None:
            receipt["repository_binding"] = repository_binding
        receipt_path = f"receipts/{receipt_id}.json"
        receipt_bytes = json.dumps(receipt, sort_keys=True, separators=(",", ":")).encode()
        return self._reference(receipt_path, receipt_bytes)

    def _descriptor(self, path_value, content_bytes):
        return {
            "path": path_value, "size_bytes": len(content_bytes),
            "sha256": hashlib.sha256(content_bytes).hexdigest(),
            "blake3": self.blake3_by_path[path_value],
        }

    def _build(self):
        fixture_output = synthetic_fixture_output(self.source_inventory)
        fixture = self._receipt("fixture_suite", "production_workspace", ["cargo", "test", "--workspace"], fixture_output)
        fmt = self._receipt("fmt", "production_workspace", ["cargo", "fmt", "--all", "--", "--check"])
        clippy = self._receipt(
            "clippy", "production_workspace",
            ["cargo", "clippy", "--message-format=json", "--workspace", "--all-targets", "--all-features",
             "--", "-D", "warnings"],
            json.dumps({"reason": "build-finished", "success": True}),
        )
        tree = self._receipt(
            "dependency_audit", "production_workspace",
            ["cargo", "tree", "--package", "parseltongue-cli-dispatcher-binary", "--edges", "normal",
             "--prefix", "none", "--format", "{p}", "--offline", "--locked"],
            "parseltongue-cli-dispatcher-binary v0.1.0\nparser-pin v1.0.0\n",
        )
        release = self._receipt(
            "release_build", "production_workspace",
            ["cargo", "build", "--release", "--package", "parseltongue-cli-dispatcher-binary",
             "--bin", "parseltongue", "--message-format=json"],
            "\n".join([
                json.dumps({
                    "reason": "compiler-artifact",
                    "package_id": "path+file:///production#parseltongue-cli-dispatcher-binary@0.1.0",
                    "manifest_path": "/production/Cargo.toml",
                    "target": {
                        "name": "parseltongue", "kind": ["bin"],
                        "src_path": self.expected_identity["configuration"]["production_bin_source"],
                    },
                    "executable": self.expected_identity["mapper"]["canonical_path"],
                    "profile": {"test": False, "opt_level": "3"},
                }),
                json.dumps({"reason": "build-finished", "success": True}),
            ]),
        )
        self.manifest = {
            "schema_version": 1, "run_id": "synthetic-run", "milestone": self.expected_identity["milestone"],
            "identity_binding": self.expected_identity,
            "global_prerequisites": {
                "fixture_suite": [fixture], "build_evidence": {"fmt": fmt, "clippy": clippy},
                "dependency_audit": [tree], "release_provenance": [release],
            },
            "repositories": {},
        }
        for repository_name in intake.DEFAULT_REPOSITORY_ORDER:
            package_id = f"{repository_name} 0.1.0 (path+file:///corpus/{repository_name})"
            repository_binding = {
                "pinned_name": repository_name, "pinned_commit": "1" * 40,
                "checkout_path": f"/corpus/{repository_name}",
                "source_snapshot": {"source_tree_sha256": "2" * 64},
                "lock_identity": {"sha256": "3" * 64},
                "selected_policy": {
                    "targets": ["lib", "bin"], "features": "default",
                    "per_unit_cfg": "compiler-default", "target": "compiler-default",
                },
                "workspace_members": [f"/corpus/{repository_name}/Cargo.toml"],
                "selected_units": [
                    {
                        "manifest_path": f"/corpus/{repository_name}/Cargo.toml",
                        "target_name": f"{repository_name.replace('-', '_')}_lib",
                        "kind": ["lib"], "edition": "2021",
                        "src_path": f"/corpus/{repository_name}/src/lib.rs",
                    },
                    {
                        "manifest_path": f"/corpus/{repository_name}/Cargo.toml",
                        "target_name": f"{repository_name.replace('-', '_')}_bin",
                        "kind": ["bin"], "edition": "2021",
                        "src_path": f"/corpus/{repository_name}/src/main.rs",
                    },
                ],
                "p2_policy_supported": True, "p2_blocked_reason": None,
            }
            self.expected_repositories[repository_name] = repository_binding
            metadata = {
                "workspace_members": [package_id],
                "packages": [{
                    "id": package_id, "name": repository_name, "edition": "2021",
                    "manifest_path": f"/corpus/{repository_name}/Cargo.toml",
                    "targets": [
                        {"name": f"{repository_name.replace('-', '_')}_lib", "kind": ["lib"],
                         "src_path": f"/corpus/{repository_name}/src/lib.rs"},
                        {"name": f"{repository_name.replace('-', '_')}_bin", "kind": ["bin"],
                         "src_path": f"/corpus/{repository_name}/src/main.rs"},
                    ],
                }],
                "resolve": {"nodes": [{"id": package_id, "features": []}]},
            }
            metadata_ref = self._receipt(
                "p2_metadata", repository_name,
                ["cargo", "metadata", "--format-version", "1", "--offline", "--locked"],
                json.dumps(metadata), repository_binding=repository_binding,
            )
            check_messages = []
            for target in metadata["packages"][0]["targets"]:
                check_messages.append(json.dumps({
                    "reason": "compiler-artifact", "package_id": package_id, "features": [],
                    "manifest_path": f"/corpus/{repository_name}/Cargo.toml",
                    "target": {"name": target["name"], "kind": target["kind"], "edition": "2021",
                               "src_path": target["src_path"]},
                    "profile": {"test": False},
                }))
            check_messages.append(json.dumps({"reason": "build-finished", "success": True}))
            check_ref = self._receipt(
                "p2_check", repository_name,
                ["cargo", "check", "--workspace", "--lib", "--bins", "--offline", "--locked",
                 "--message-format=json"],
                "\n".join(check_messages), repository_binding=repository_binding,
            )
            self.manifest["repositories"][repository_name] = {
                "repository_binding": repository_binding,
                "p2_offline_qualification": [metadata_ref, check_ref],
            }

    def refresh_receipt(self, receipt_path, mutate):
        receipt = json.loads(self.file_bytes[receipt_path])
        mutate(receipt)
        receipt_bytes = json.dumps(receipt, sort_keys=True, separators=(",", ":")).encode()
        self.file_bytes[receipt_path] = receipt_bytes
        self.blake3_by_path[receipt_path] = synthetic_blake3_digest(receipt_bytes)
        for reference in iter_manifest_references(self.manifest):
            if reference["path"] == receipt_path:
                reference.update({
                    "size_bytes": len(receipt_bytes), "sha256": hashlib.sha256(receipt_bytes).hexdigest(),
                    "blake3": self.blake3_by_path[receipt_path],
                })
                return receipt
        raise AssertionError(receipt_path)

    def refresh_log(self, receipt_path, stream_name, output_text):
        receipt = json.loads(self.file_bytes[receipt_path])
        descriptor = receipt[stream_name]
        content = output_text.encode()
        self.file_bytes[descriptor["path"]] = content
        self.blake3_by_path[descriptor["path"]] = synthetic_blake3_digest(content)
        receipt[stream_name] = {**self._descriptor(descriptor["path"], content), "complete": True}
        self.refresh_receipt(receipt_path, lambda value: value.update(receipt))

    def receipt_paths(self, family, scope="production_workspace"):
        return [
            reference["path"] for reference in iter_manifest_references(self.manifest)
            if (lambda receipt: receipt["family"] == family and receipt["scope"] == scope)(
                json.loads(self.file_bytes[reference["path"]])
            )
        ]


def iter_manifest_references(value):
    if isinstance(value, dict):
        if set(value) == intake.RECEIPT_REFERENCE_KEYS:
            yield value
        else:
            for child in value.values():
                yield from iter_manifest_references(child)
    elif isinstance(value, list):
        for child in value:
            yield from iter_manifest_references(child)


class S3PrerequisiteEvidenceIntakeTests(unittest.TestCase):
    baseline_tree_bytes = b"parseltongue-cli-dispatcher-binary v0.1.0\nparser-pin v1.0.0\n"

    def test_json_rejects_duplicate_keys(self):
        """REQ-SOP-RUNNER-022: strict JSON rejects duplicate object keys."""
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake.parse_s3_evidence_json('{"schema_version":1,"schema_version":1}', "manifest")

    def test_json_rejects_nonfinite_numbers(self):
        """REQ-SOP-RUNNER-022: strict JSON rejects nonfinite numeric values."""
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake.parse_s3_evidence_json('{"elapsed_seconds":NaN}', "receipt")

    def test_receipt_rejects_boolean_version(self):
        """REQ-SOP-RUNNER-022: schema version rejects bool despite Python integer subtype."""
        receipt = {"schema_version": True}
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake.validate_s3_evidence_receipt(receipt, {}, expected_family="fixture_suite")

    def test_receipt_rejects_boolean_elapsed(self):
        """REQ-SOP-RUNNER-022: elapsed and timeout values must be finite non-bool numbers."""
        receipt = {"schema_version": 1, "elapsed_seconds": True}
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake.validate_s3_evidence_receipt(receipt, {}, expected_family="fixture_suite")

    def test_bad_reference_shape_blocks(self):
        """REQ-SOP-RUNNER-022: malformed sizes and digests are incomplete evidence."""
        valid = {"path": "receipts/a.json", "size_bytes": 1,
                 "sha256": "a" * 64, "blake3": "b" * 64}
        for key, value in (("size_bytes", True), ("size_bytes", "1"), ("sha256", "bad"),
                           ("blake3", "g" * 64)):
            with self.subTest(key=key, value=value), self.assertRaises(intake.S3EvidenceBlockedError):
                intake.validate_s3_evidence_reference({**valid, key: value})

    def test_missing_receipt_schema_blocks(self):
        """REQ-SOP-RUNNER-022: absent or unsupported receipt schema is BLOCKED."""
        for schema_value in (None, 2):
            with self.subTest(schema_value=schema_value):
                bundle = SyntheticEvidenceBundle()
                receipt_path = bundle.receipt_paths("fmt")[0]
                def remove_or_replace_schema(receipt):
                    if schema_value is None:
                        receipt.pop("schema_version")
                    else:
                        receipt["schema_version"] = schema_value
                bundle.refresh_receipt(receipt_path, remove_or_replace_schema)
                result = self.build_bundle_result(bundle)
                self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED")

    def test_missing_receipt_exit_blocks(self):
        """REQ-SOP-RUNNER-022: malformed or incomplete exit fields are BLOCKED."""
        mutations = (
            lambda receipt: receipt.pop("exit_status"),
            lambda receipt: receipt.update({"exit_status": True}),
            lambda receipt: receipt.update({"completion_status": []}),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                bundle = SyntheticEvidenceBundle()
                bundle.refresh_receipt(bundle.receipt_paths("fmt")[0], mutation)
                result = self.build_bundle_result(bundle)
                self.assertEqual(
                    result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED",
                )

    def test_nested_receipt_identity_fails(self):
        """REQ-SOP-RUNNER-022: nested bool/string identity changes are strict FAIL mismatches."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fmt")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["identity_binding"]["mapper"].update({
            "sha256": True,
        }))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "FAIL")

    def test_receipt_rejects_timeout_success(self):
        """REQ-SOP-RUNNER-022: timed-out receipts cannot claim successful completion."""
        receipt = {
            "schema_version": 1, "elapsed_seconds": 2.0, "timeout_seconds": 1.0,
            "completion_status": "timed_out", "timed_out": True, "exit_status": 0,
        }
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake.validate_s3_evidence_receipt(receipt, {}, expected_family="fixture_suite")

    def test_manifest_rejects_duplicate_references(self):
        """REQ-SOP-RUNNER-022: receipts cannot be shared or path-aliased across gates."""
        reference = {"path": "receipts/shared.json", "size_bytes": 1,
                     "sha256": "a" * 64, "blake3": "b" * 64}
        manifest = {
            "schema_version": 1, "run_id": "fixture-run", "milestone": "M1",
            "identity_binding": {},
            "global_prerequisites": {
                "fixture_suite": [reference],
                "build_evidence": {"fmt": reference, "clippy": reference},
                "dependency_audit": [reference], "release_provenance": [reference],
            },
            "repositories": {},
        }
        with self.assertRaises(intake.S3EvidenceViolationError):
            intake.validate_s3_prerequisite_evidence(
                manifest, {}, {}, {}, {}, {"tests": [], "ambiguous_reason": None}, "M1"
            )

    def test_manifest_schema_version_blocks(self):
        """REQ-SOP-RUNNER-022: absent, boolean, or unsupported manifest schema is BLOCKED."""
        for schema in (None, True, 2):
            with self.subTest(schema=schema):
                bundle = SyntheticEvidenceBundle()
                if schema is None:
                    bundle.manifest.pop("schema_version")
                else:
                    bundle.manifest["schema_version"] = schema
                with self.assertRaises(intake.S3EvidenceBlockedError):
                    self.build_bundle_result(bundle)

    def make_dependency_reference(
        self, bundle, stdout_bytes=None, stderr_bytes=b"", *, timed_out=False,
        exit_status=0, identity_digest=None, argv=None, stdout_complete=True, stderr_complete=True,
    ):
        stdout_bytes = self.baseline_tree_bytes if stdout_bytes is None else stdout_bytes
        expected_identity = bundle.expected_identity
        package_name = expected_identity["configuration"]["production_package_name"]
        cargo_path = expected_identity["toolchain_versions"]["cargo"]["path"]
        identity_digest = identity_digest or hashlib.sha256(json.dumps(
            expected_identity, sort_keys=True, separators=(",", ":"),
        ).encode()).hexdigest()

        def stream(name, content, complete):
            path = f"/runner-owned/{bundle.receipt_counter}-{name}"
            return {
                "path": path, "raw_bytes": content, "size_bytes": len(content),
                "sha256": hashlib.sha256(content).hexdigest(), "blake3": synthetic_blake3_digest(content),
                "complete": complete,
                "file_identity": {
                    "path": path, "size_bytes": len(content),
                    "sha256": hashlib.sha256(content).hexdigest(), "is_symlink": False,
                },
            }

        return intake._mint_runner_owned_dependency_tree_reference(
            identity_digest=identity_digest,
            argv=argv or [cargo_path, "tree", "--package", package_name, "--edges", "normal",
                          "--prefix", "none", "--format", "{p}", "--offline", "--locked"],
            cwd=expected_identity["configuration"]["project_root"],
            relevant_environment=expected_identity["relevant_environment"],
            elapsed_seconds=0.5, timeout_seconds=60.0, timed_out=timed_out,
            completion_status="timed_out" if timed_out else "completed", exit_status=exit_status,
            stdout=stream("stdout", stdout_bytes, stdout_complete),
            stderr=stream("stderr", stderr_bytes, stderr_complete),
        )

    def build_bundle_result(self, bundle, dependency_reference=None):
        if dependency_reference is None:
            dependency_reference = self.make_dependency_reference(bundle)
        return intake.validate_s3_prerequisite_evidence(
            bundle.manifest, bundle.file_bytes, bundle.blake3_by_path,
            bundle.expected_identity, bundle.expected_repositories,
            bundle.source_inventory, bundle.expected_identity["milestone"],
            dependency_reference=dependency_reference,
        )

    def test_valid_bundle_returns_tokens(self):
        """REQ-SOP-RUNNER-022: synthetic complete evidence validates without running Cargo."""
        bundle = SyntheticEvidenceBundle()
        result = self.build_bundle_result(bundle)
        self.assertEqual(set(result["tokens_by_repository"]), set(intake.DEFAULT_REPOSITORY_ORDER))
        for repository_result in result["repository_results"].values():
            self.assertEqual({gate["status"] for gate in repository_result.values()}, {"PASS"})

    def test_identity_mismatch_fails_bundle(self):
        """REQ-SOP-RUNNER-022: mapper/source/lock/pin/toolchain/env mismatch cannot qualify."""
        for identity_key in ("mapper", "runner_source_snapshot", "production_cargo_lock",
                             "parser_pin_bundle", "r1_patch", "toolchain_versions", "relevant_environment",
                             "configuration"):
            with self.subTest(identity_key=identity_key):
                bundle = SyntheticEvidenceBundle()
                bundle.manifest["identity_binding"] = {
                    **bundle.expected_identity, identity_key: {"tampered": True},
                }
                with self.assertRaises(intake.S3EvidenceViolationError):
                    self.build_bundle_result(bundle)

    def test_hash_mismatch_fails_bundle(self):
        """REQ-SOP-RUNNER-022: declared receipt/raw-log digest mismatch is FAIL, not missing evidence."""
        bundle = SyntheticEvidenceBundle()
        receipt = json.loads(bundle.file_bytes[bundle.receipt_paths("fmt")[0]])
        stdout_path = receipt["stdout"]["path"]
        bundle.file_bytes[stdout_path] += b"tampered"
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "FAIL")

    def test_missing_log_blocks_bundle(self):
        """REQ-SOP-RUNNER-022: missing raw stdout/stderr remains BLOCKED at intake."""
        bundle = SyntheticEvidenceBundle()
        receipt = json.loads(bundle.file_bytes[bundle.receipt_paths("fmt")[0]])
        missing_path = receipt["stderr"]["path"]
        bundle.file_bytes.pop(missing_path)
        bundle.blake3_by_path.pop(missing_path)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED")

    def test_truncated_log_blocks_bundle(self):
        """REQ-SOP-RUNNER-022: a receipt cannot pass with incomplete stream descriptors."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fmt")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["stdout"].update({"complete": False}))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED")

    def test_fixture_filter_blocks_bundle(self):
        """REQ-SOP-RUNNER-022: positional fixture filters are unsupported and BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].append("filtered-test"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_fixture_release_profile_passes(self):
        """REQ-SOP-RUNNER-022: explicit standard release profile preserves the unfiltered fixture argv."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].insert(2, "--release"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "PASS")
        receipt = json.loads(bundle.file_bytes[receipt_path])
        self.assertEqual(receipt["argv"][1:], ["test", "--release", "--workspace"])

    def test_clippy_release_profile_passes(self):
        """REQ-SOP-RUNNER-022: explicit standard release profile preserves strict clippy argv."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("clippy")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].insert(2, "--release"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "PASS")
        receipt = json.loads(bundle.file_bytes[receipt_path])
        self.assertEqual(receipt["argv"][1:], [
            "clippy", "--release", "--message-format=json", "--workspace", "--all-targets",
            "--all-features", "--", "-D", "warnings",
        ])

    def test_release_profile_misuse_blocks(self):
        """REQ-SOP-RUNNER-022: package, skip, custom-profile and unknown flags stay rejected."""
        def package_filter(argv):
            argv[2:2] = ["--release", "--package", "parseltongue"]

        def custom_profile(argv):
            argv[2:2] = ["--release", "--profile", "release"]

        def unknown_option(argv):
            argv[2:2] = ["--release", "--unknown-option"]

        def skipped_fixture(argv):
            argv.insert(2, "--release")
            argv.extend(["--", "--skip", "test_fixture"])

        def duplicate_release(argv):
            argv[2:2] = ["--release", "--release"]

        def release_after_separator(argv):
            argv.extend(["--", "--release"])

        def release_before_subcommand(argv):
            argv.insert(1, "--release")

        mutations = (
            ("fixture_suite", package_filter), ("fixture_suite", custom_profile),
            ("fixture_suite", unknown_option), ("fixture_suite", skipped_fixture),
            ("fixture_suite", duplicate_release), ("fixture_suite", release_after_separator),
            ("fixture_suite", release_before_subcommand),
            ("clippy", package_filter), ("clippy", custom_profile), ("clippy", unknown_option),
            ("clippy", duplicate_release), ("clippy", release_after_separator),
            ("clippy", release_before_subcommand),
        )
        for family, mutate in mutations:
            with self.subTest(family=family, mutate=mutate.__name__):
                bundle = SyntheticEvidenceBundle()
                receipt_path = bundle.receipt_paths(family)[0]
                bundle.refresh_receipt(receipt_path, lambda receipt: mutate(receipt["argv"]))
                if mutate.__name__ in {"package_filter", "custom_profile", "unknown_option", "skipped_fixture"}:
                    receipt = json.loads(bundle.file_bytes[receipt_path])
                    release_position = receipt["argv"].index("--release")
                    separator_position = (
                        receipt["argv"].index("--") if "--" in receipt["argv"] else len(receipt["argv"])
                    )
                    self.assertEqual(receipt["argv"].count("--release"), 1)
                    self.assertGreater(release_position, 1)
                    self.assertLess(release_position, separator_position)
                result = self.build_bundle_result(bundle)
                gate_name = "fixture_suite" if family == "fixture_suite" else "build_evidence"
                self.assertEqual(result["repository_results"]["serde"][gate_name]["status"], "BLOCKED")

        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("clippy")[0]
        def release_environment_mismatch(receipt):
            receipt["argv"].insert(2, "--release")
            receipt["relevant_environment"].update({"CARGO_PROFILE_RELEASE_DEBUG": "0"})

        bundle.refresh_receipt(receipt_path, release_environment_mismatch)
        receipt = json.loads(bundle.file_bytes[receipt_path])
        self.assertIn("--release", receipt["argv"])
        result = self.build_bundle_result(bundle)
        self.assertNotEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "PASS")

    def test_native_skip_blocks_bundle(self):
        """REQ-SOP-RUNNER-022: native-skip environment overrides cannot qualify fixture tests."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["relevant_environment"].update({
            "PARSELTONGUE_SKIP_NATIVE_TESTS": "1",
        }))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")

    def test_forbidden_tree_fails_gate(self):
        """REQ-SOP-RUNNER-022: actual forbidden runtime dependency rows fail the tree gate."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(receipt_path, "stdout", "parseltongue-cli-dispatcher-binary v0.1.0\nra_ap_hir v0.1.0\n")
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_analysis_tree_dependencies_fail(self):
        """REQ-SOP-RUNNER-022: ide/hir/base-db/salsa runtime families are forbidden."""
        for name in ("ide", "hir", "base_db", "salsa", "ra_ap_ide", "ra_ap_hir", "ra_ap_salsa"):
            with self.subTest(name=name), self.assertRaises(intake.S3EvidenceViolationError):
                intake._validate_dependency_tree_output(
                    f"parseltongue-cli-dispatcher-binary v0.1.0\n{name} v1.0.0\n",
                    "parseltongue-cli-dispatcher-binary",
                    {"configuration": {"production_package_version": "0.1.0"}},
                )

    def test_pinned_parser_tree_passes(self):
        """REQ-SOP-RUNNER-022: pinned parser/syntax/mbe/tt/span crates remain permitted."""
        intake._validate_dependency_tree_output(
            "parseltongue-cli-dispatcher-binary v0.1.0\n"
            "ra_ap_syntax v1.0.0\nra_ap_mbe v1.0.0\nra_ap_tt v1.0.0\nra_ap_span v1.0.0\n",
            "parseltongue-cli-dispatcher-binary",
            {"configuration": {"production_package_version": "0.1.0"}},
        )

    def test_release_mismatch_fails_gate(self):
        """REQ-SOP-RUNNER-022: release artifact must match frozen production binary identity."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("release_build")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[stdout_path].decode().splitlines()]
        messages[0]["executable"] = "/external/unrelated-binary"
        bundle.refresh_log(receipt_path, "stdout", "\n".join(json.dumps(message) for message in messages))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["release_provenance"]["status"], "FAIL")

    def test_p2_target_override_blocks(self):
        """REQ-SOP-RUNNER-022: explicit P2 target overrides are BLOCKED in v1."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_check", "serde")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].extend(["--target", "x86_64-unknown-linux-gnu"]))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED")

    def test_p2_feature_override_blocks(self):
        """REQ-SOP-RUNNER-022: P2 cannot use all-features or no-default-features selectors."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_check", "serde")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].append("--all-features"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED")

    def test_manual_ignore_passes_bundle(self):
        """REQ-SOP-RUNNER-022: exact source-backed manual-profile ignores are allowed."""
        bundle = SyntheticEvidenceBundle(manual=True)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "PASS")

    def test_unknown_ignore_blocks_fixture(self):
        """REQ-SOP-RUNNER-022: unknown ignored tests cannot qualify the fixture suite."""
        bundle = SyntheticEvidenceBundle()
        bundle.source_inventory["tests"].append({
            "name": "test_unknown_ignored_profile", "path": "crates/fixture.rs",
            "ignore_reason": "local shortcut",
        })
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_log(receipt_path, "stdout", synthetic_fixture_output(bundle.source_inventory))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_child_absent_ignore_blocks(self):
        """REQ-SOP-RUNNER-022: a claimed child ignore absent from frozen source is BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        extra = "test fixture::test_child_fixture_waits_release ... ignored, subprocess fixture only"
        bundle.refresh_log(receipt_path, "stdout", synthetic_fixture_output(bundle.source_inventory, [extra]))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_child_exception_active_passes(self):
        """REQ-SOP-RUNNER-022: exact conditional child ignore requires and accepts three active checks."""
        bundle = SyntheticEvidenceBundle(child=True)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "PASS")

    def test_failed_fixture_sets_fail(self):
        """REQ-SOP-RUNNER-022: a failed libtest row is FAIL even when receipt exit is zero."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        tests = bundle.source_inventory["tests"]
        rows = [f"test fixture::{test['name']} ... ok" for test in tests]
        rows[-1] = rows[-1][:-2] + "FAILED"
        output = "\n".join([
            f"running {len(rows)} tests", *rows,
            f"test result: FAILED. {len(rows)-1} passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s",
        ])
        bundle.refresh_log(receipt_path, "stdout", output)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")

    def test_nonzero_then_missing_fails(self):
        """REQ-SOP-RUNNER-022: later missing receipt cannot mask an observed nonzero exit."""
        bundle = SyntheticEvidenceBundle()
        first_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(first_path, lambda receipt: receipt.update({"exit_status": 17}))
        missing = bundle._reference("receipts/missing-later.json", b"{}").copy()
        bundle.file_bytes.pop(missing["path"])
        bundle.blake3_by_path.pop(missing["path"])
        bundle.manifest["global_prerequisites"]["fixture_suite"].append(missing)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")
        self.assertNotIn("serde", result["tokens_by_repository"])

    def test_fixture_failure_beats_truncated(self):
        """REQ-SOP-RUNNER-022: failed fixture output dominates an incomplete sibling."""
        bundle = SyntheticEvidenceBundle()
        first_path = bundle.receipt_paths("fixture_suite")[0]
        rows = [f"test fixture::{test['name']} ... ok" for test in bundle.source_inventory["tests"]]
        rows[-1] = rows[-1][:-2] + "FAILED"
        output = "\n".join([
            f"running {len(rows)} tests", *rows,
            f"test result: FAILED. {len(rows)-1} passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s",
        ])
        bundle.refresh_log(first_path, "stdout", output)
        second = bundle._receipt("fixture_suite", "production_workspace", ["cargo", "test", "--workspace"], output)
        second_path = second["path"]
        bundle.manifest["global_prerequisites"]["fixture_suite"].append(second)
        bundle.refresh_receipt(second_path, lambda item: item["stdout"].update({"complete": False}))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")
        self.assertNotIn("serde", result["tokens_by_repository"])

    def test_forbidden_tree_lacks_p2(self):
        """REQ-SOP-RUNNER-022: forbidden runtime dependency survives missing repository P2 evidence."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", "parseltongue-cli-dispatcher-binary v0.1.0\nsalsa v1.0.0\n")
        p2_path = bundle.receipt_paths("p2_check", "serde")[0]
        p2_receipt = json.loads(bundle.file_bytes[p2_path])
        for stream_name in ("stdout", "stderr"):
            log_path = p2_receipt[stream_name]["path"]
            bundle.file_bytes.pop(log_path)
            bundle.blake3_by_path.pop(log_path)
        bundle.file_bytes.pop(p2_path)
        bundle.blake3_by_path.pop(p2_path)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")
        self.assertEqual(result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED")
        self.assertNotIn("serde", result["tokens_by_repository"])

    def test_missing_then_nonzero_fails(self):
        """REQ-SOP-RUNNER-022: failure priority is independent of receipt order."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt.update({"exit_status": 7}))
        missing = bundle._reference("receipts/missing-earlier.json", b"{}").copy()
        bundle.file_bytes.pop(missing["path"])
        bundle.blake3_by_path.pop(missing["path"])
        bundle.manifest["global_prerequisites"]["fixture_suite"].insert(0, missing)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")
        self.assertNotIn("serde", result["tokens_by_repository"])

    def test_m2_missing_crosswalk_blocks(self):
        """REQ-SOP-RUNNER-022: missing exact M2 crosswalk names stay required and BLOCKED."""
        bundle = SyntheticEvidenceBundle(milestone="M2")
        missing_name = "test_oracle_disagreements_keep_reasons"
        bundle.source_inventory["tests"] = [test for test in bundle.source_inventory["tests"]
                                              if test["name"] != missing_name]
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_log(receipt_path, "stdout", synthetic_fixture_output(bundle.source_inventory))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_source_macro_ambiguity_blocks(self):
        """REQ-SOP-RUNNER-022: ambiguous macro-generated source inventory cannot claim exhaustive tests."""
        bundle = SyntheticEvidenceBundle()
        bundle.source_inventory["ambiguous_reason"] = "generated test inventory is ambiguous"
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_duplicate_receipt_id_fails(self):
        """REQ-SOP-RUNNER-022: receipt identifiers are globally unique across prerequisite gates."""
        bundle = SyntheticEvidenceBundle()
        fmt_path, clippy_path = bundle.receipt_paths("fmt")[0], bundle.receipt_paths("clippy")[0]
        clippy = json.loads(bundle.file_bytes[clippy_path])
        fmt = json.loads(bundle.file_bytes[fmt_path])
        bundle.refresh_receipt(clippy_path, lambda receipt: receipt.update({"receipt_id": fmt["receipt_id"]}))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "FAIL")

    def test_duplicate_receipt_contents_fail(self):
        """REQ-SOP-RUNNER-022: byte-identical receipts cannot be replayed under another path."""
        bundle = SyntheticEvidenceBundle()
        fmt_path, clippy_path = bundle.receipt_paths("fmt")[0], bundle.receipt_paths("clippy")[0]
        fmt_bytes = bundle.file_bytes[fmt_path]
        bundle.file_bytes[clippy_path] = fmt_bytes
        bundle.blake3_by_path[clippy_path] = synthetic_blake3_digest(fmt_bytes)
        reference = next(value for value in iter_manifest_references(bundle.manifest)
                         if value["path"] == clippy_path)
        reference.update({"size_bytes": len(fmt_bytes), "sha256": hashlib.sha256(fmt_bytes).hexdigest(),
                          "blake3": synthetic_blake3_digest(fmt_bytes)})
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "FAIL")

    def test_p2_feature_context_blocks(self):
        """REQ-SOP-RUNNER-022.P: unsupported P2 unit feature context remains BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        metadata_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        receipt = json.loads(bundle.file_bytes[metadata_path])
        stdout_path = receipt["stdout"]["path"]
        metadata = json.loads(bundle.file_bytes[stdout_path])
        metadata["resolve"]["nodes"][0]["features"] = ["feature-not-in-check"]
        bundle.refresh_log(metadata_path, "stdout", json.dumps(metadata))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED")

    def test_tree_reference_required_blocks(self):
        """REQ-SOP-RUNNER-022.D: supplied dependency rows alone cannot issue qualification tokens."""
        bundle = SyntheticEvidenceBundle()
        result = intake.validate_s3_prerequisite_evidence(
            bundle.manifest, bundle.file_bytes, bundle.blake3_by_path,
            bundle.expected_identity, bundle.expected_repositories,
            bundle.source_inventory, bundle.expected_identity["milestone"],
        )
        self.assertEqual(
            result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED",
        )
        self.assertEqual(result["tokens_by_repository"], {})

    def test_tree_root_only_fails(self):
        """REQ-SOP-RUNNER-022.D: root-only submitted rows fail exact current-tree comparison."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", "parseltongue-cli-dispatcher-binary v0.1.0\n")
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_tree_root_version_fails(self):
        """REQ-SOP-RUNNER-022.D: a non-frozen production root version is FAIL."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", self.baseline_tree_bytes.replace(b"v0.1.0", b"v9.9.9").decode())
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_root_malformed_fails(self):
        """REQ-SOP-RUNNER-022.D: wrong frozen root remains FAIL before a malformed suffix and absent reference."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", "wrong-root v9.9.9\nmalformed tree suffix\n")
        result = intake.validate_s3_prerequisite_evidence(
            bundle.manifest, bundle.file_bytes, bundle.blake3_by_path,
            bundle.expected_identity, bundle.expected_repositories,
            bundle.source_inventory, bundle.expected_identity["milestone"],
        )
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_dependency_legacy_receipt_passes(self):
        """REQ-SOP-RUNNER-022.D: schema-v1 tree receipts may omit optional offline/locked flags."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_receipt(tree_path, lambda receipt: receipt.update(argv=receipt["argv"][:-2]))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "PASS")

    def test_dependency_unknown_flag_blocks(self):
        """REQ-SOP-RUNNER-022.D: unknown supplied tree flags are blocked while runner argv stays separate."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_receipt(tree_path, lambda receipt: receipt["argv"].append("--unknown"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_tree_suffix_omission_fails(self):
        """REQ-SOP-RUNNER-022.D: omission of a suffix row fails against the independent reference."""
        bundle = SyntheticEvidenceBundle()
        reference_rows = self.baseline_tree_bytes + b"unrelated-pin v3.0.0\n"
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", reference_rows.decode())
        reference = self.make_dependency_reference(bundle, reference_rows)
        bundle.refresh_log(tree_path, "stdout", self.baseline_tree_bytes.decode())
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_interior_omission_fails(self):
        """REQ-SOP-RUNNER-022.D: an interior omitted row fails without sorting or deduplication."""
        bundle = SyntheticEvidenceBundle()
        reference_rows = b"parseltongue-cli-dispatcher-binary v0.1.0\na-pin v1.0.0\nb-pin v1.0.0\nc-pin v1.0.0\n"
        supplied_rows = b"parseltongue-cli-dispatcher-binary v0.1.0\na-pin v1.0.0\nc-pin v1.0.0\n"
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", reference_rows.decode())
        reference = self.make_dependency_reference(bundle, reference_rows)
        bundle.refresh_log(tree_path, "stdout", supplied_rows.decode())
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_order_mismatch_fails(self):
        """REQ-SOP-RUNNER-022.D: inserted duplicate or reordered rows fail exact comparison."""
        swapped_reference = (
            b"parseltongue-cli-dispatcher-binary v0.1.0\nfirst-pin v1.0.0\nsecond-pin v2.0.0\n"
        )
        swapped_rows = (
            b"parseltongue-cli-dispatcher-binary v0.1.0\nsecond-pin v2.0.0\nfirst-pin v1.0.0\n"
        )
        for changed_rows, reference_rows in (
            (self.baseline_tree_bytes + b"parser-pin v1.0.0\n", self.baseline_tree_bytes),
            (swapped_rows, swapped_reference),
        ):
            with self.subTest(changed_rows=changed_rows):
                bundle = SyntheticEvidenceBundle()
                tree_path = bundle.receipt_paths("dependency_audit")[0]
                bundle.refresh_log(tree_path, "stdout", reference_rows.decode())
                reference = self.make_dependency_reference(bundle, reference_rows)
                bundle.refresh_log(tree_path, "stdout", changed_rows.decode())
                result = self.build_bundle_result(bundle, reference)
                self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_reference_incomplete_blocks(self):
        """REQ-SOP-RUNNER-022.D: incomplete retained internal streams cannot qualify with exit zero."""
        bundle = SyntheticEvidenceBundle()
        reference = self.make_dependency_reference(bundle, stdout_complete=False)
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_tree_duplicate_reference_passes(self):
        """REQ-SOP-RUNNER-022.D: valid repeated rows pass only when exactly present in the reference."""
        bundle = SyntheticEvidenceBundle()
        duplicate_rows = self.baseline_tree_bytes + b"parser-pin v1.0.0\n"
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", duplicate_rows.decode())
        result = self.build_bundle_result(bundle, self.make_dependency_reference(bundle, duplicate_rows))
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "PASS")

    def test_tree_crlf_only_passes(self):
        """REQ-SOP-RUNNER-022.D: CRLF-to-LF normalization is the only row normalization."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", self.baseline_tree_bytes.replace(b"\n", b"\r\n").decode())
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "PASS")

    def test_tree_whitespace_mismatch_fails(self):
        """REQ-SOP-RUNNER-022.D: whitespace changes are not normalized in ordered tree rows."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.refresh_log(tree_path, "stdout", (b" " + self.baseline_tree_bytes).decode())
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_forged_reference_blocks(self):
        """REQ-SOP-RUNNER-022.D: serialized dictionaries cannot forge the internal reference capability."""
        bundle = SyntheticEvidenceBundle()
        for forged in ({"runner_owned": True}, True, "serialized-reference"):
            with self.subTest(forged=forged):
                result = self.build_bundle_result(bundle, forged)
                self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED")
                self.assertEqual(result["tokens_by_repository"], {})

    def test_tree_stale_reference_blocks(self):
        """REQ-SOP-RUNNER-022.D: a reference bound to another identity cannot qualify this run."""
        bundle = SyntheticEvidenceBundle()
        stale_rows = self.baseline_tree_bytes + b"stale-extra v9.0.0\n"
        reference = self.make_dependency_reference(
            bundle, stale_rows, identity_digest="f" * 64,
        )
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED")

    def test_tree_timeout_output_blocks(self):
        """REQ-SOP-RUNNER-022.D: a timed-out retained reference remains BLOCKED without forbidden rows."""
        bundle = SyntheticEvidenceBundle()
        reference = self.make_dependency_reference(bundle, timed_out=True, exit_status=124)
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "BLOCKED")

    def test_tree_nonzero_reference_fails(self):
        """REQ-SOP-RUNNER-022.D: a completed internal tree command with nonzero exit is FAIL."""
        bundle = SyntheticEvidenceBundle()
        reference = self.make_dependency_reference(bundle, exit_status=7)
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_partial_forbidden_fails(self):
        """REQ-SOP-RUNNER-022.D: a forbidden row in a timed-out partial reference is FAIL."""
        bundle = SyntheticEvidenceBundle()
        partial = b"parseltongue-cli-dispatcher-binary v0.1.0\nra_ap_ide v0.0.0\n"
        reference = self.make_dependency_reference(bundle, partial, timed_out=True, exit_status=124)
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_incomplete_forbidden_fails(self):
        """REQ-SOP-RUNNER-022.D: forbidden partial supplied rows outrank malformed sibling evidence."""
        bundle = SyntheticEvidenceBundle()
        tree_path = bundle.receipt_paths("dependency_audit")[0]
        bundle.manifest["global_prerequisites"]["build_evidence"]["clippy"] = bundle._reference(
            "receipts/malformed-clippy.json", b'{"schema_version":',
        )
        bundle.refresh_log(tree_path, "stdout", "malformed sibling\nra_ap_ide v0.0.0\n")
        bundle.refresh_receipt(tree_path, lambda receipt: receipt["stdout"].update({"complete": False}))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_tree_reference_mutation_fails(self):
        """REQ-SOP-RUNNER-022.D: changed retained bytes cannot preserve reference authority."""
        bundle = SyntheticEvidenceBundle()
        reference = self.make_dependency_reference(bundle)
        object.__setattr__(reference.stdout, "raw_bytes", b"mutated")
        result = self.build_bundle_result(bundle, reference)
        self.assertEqual(result["repository_results"]["serde"]["dependency_audit"]["status"], "FAIL")

    def test_async_source_inventory_matches(self):
        """REQ-SOP-RUNNER-022: async test functions are included in frozen source inventory."""
        inventory = intake.derive_source_fixture_inventory({
            "crates/async.rs": "#[tokio::test]\nasync fn test_async_fixture_runs() {}\n",
        })
        self.assertEqual(inventory["ambiguous_reason"], None)
        self.assertEqual([test["name"] for test in inventory["tests"]], ["test_async_fixture_runs"])

    def test_detached_test_attribute_blocks(self):
        """REQ-SOP-RUNNER-022: test attributes separated from functions are ambiguous and BLOCKED."""
        inventory = intake.derive_source_fixture_inventory({
            "crates/detached.rs": "#[test]\n// separated by comment\nfn test_detached_fixture_runs() {}\n",
        })
        self.assertIsNotNone(inventory["ambiguous_reason"])

    def test_wrong_runtime_run_fails(self):
        """REQ-SOP-RUNNER-022: a manifest bound to another run is FAIL."""
        bundle = SyntheticEvidenceBundle()
        bundle.manifest["run_id"] = "different-run"
        with self.assertRaises(intake.S3EvidenceViolationError):
            self.build_bundle_result(bundle)

    def test_wrong_pinned_repository_fails(self):
        """REQ-SOP-RUNNER-022: repository pin identity mismatches are FAIL."""
        bundle = SyntheticEvidenceBundle()
        bundle.manifest["repositories"]["serde"]["repository_binding"]["pinned_commit"] = "2" * 40
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_selected_unit_policy_fails(self):
        """REQ-SOP-RUNNER-022: explicit repository feature-policy identity mismatch is FAIL."""
        bundle = SyntheticEvidenceBundle()
        bundle.manifest["repositories"]["serde"]["repository_binding"]["selected_policy"]["features"] = "all"
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_command_target_selector_blocks(self):
        """REQ-SOP-RUNNER-022: selective clippy and non-PATH Cargo commands are BLOCKED."""
        mutations = (
            lambda receipt: receipt["argv"].extend(["--package", "selected"]),
            lambda receipt: receipt["argv"].__setitem__(0, "/tmp/cargo"),
        )
        for mutation in mutations:
            with self.subTest(mutation=mutation):
                bundle = SyntheticEvidenceBundle()
                bundle.refresh_receipt(bundle.receipt_paths("clippy")[0], mutation)
                result = self.build_bundle_result(bundle)
                self.assertEqual(
                    result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED",
                )

    def test_fixture_section_counts_block(self):
        """REQ-SOP-RUNNER-022: per-section passed/failed/ignored counts match rows."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        passed = len(bundle.source_inventory["tests"])
        output = bundle.file_bytes[stdout_path].decode().replace(
            f"{passed} passed; 0 failed; 0 ignored",
            f"{passed - 1} passed; 0 failed; 1 ignored",
        )
        bundle.refresh_log(receipt_path, "stdout", output)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_fixture_measured_rows_block(self):
        """REQ-SOP-RUNNER-022: filtered or measured libtest sections are BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        output = bundle.file_bytes[stdout_path].decode().replace(
            "0 measured; 0 filtered out", "1 measured; 0 filtered out",
        )
        bundle.refresh_log(receipt_path, "stdout", output)
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")

    def test_clippy_warning_stderr_fails(self):
        """REQ-SOP-RUNNER-022: clippy warning/error diagnostics on stderr are FAIL."""
        bundle = SyntheticEvidenceBundle()
        bundle.refresh_log(bundle.receipt_paths("clippy")[0], "stderr", "warning: denied lint\n")
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "FAIL")

    def test_tree_full_rows_block(self):
        """REQ-SOP-RUNNER-022: dependency tree rows must parse completely."""
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_dependency_tree_output(
                b"parseltongue-cli-dispatcher-binary v0.1.0\nparser-pin v1.0.0 trailing garbage\n",
                "parseltongue-cli-dispatcher-binary",
                {"configuration": {"production_package_version": "0.1.0"}},
            )

    def test_tree_server_families_fail(self):
        """REQ-SOP-RUNNER-022: oracle harness and proc-macro server rows are forbidden."""
        for name in ("oracle-semantic-export-harness", "proc-macro-srv", "proc_macro_srv",
                     "ra_ap_proc_macro_srv", "proc-macro-server"):
            with self.subTest(name=name), self.assertRaises(intake.S3EvidenceViolationError):
                intake._validate_dependency_tree_output(
                    f"parseltongue-cli-dispatcher-binary v0.1.0\n{name} v1.0.0\n".encode(),
                    "parseltongue-cli-dispatcher-binary",
                    {"configuration": {"production_package_version": "0.1.0"}},
                )

    def test_release_missing_artifact_blocks(self):
        """REQ-SOP-RUNNER-022: successful release without a binary artifact is BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        output = json.dumps({"reason": "build-finished", "success": True})
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_release_build_output(output, bundle.expected_identity)

    def test_release_identity_context_blocks(self):
        """REQ-SOP-RUNNER-022: release validation blocks without package/version/source context."""
        bundle = SyntheticEvidenceBundle()
        receipt = json.loads(bundle.file_bytes[bundle.receipt_paths("release_build")[0]])
        stdout_path = receipt["stdout"]["path"]
        output = bundle.file_bytes[stdout_path].decode()
        expected = {**bundle.expected_identity, "configuration": {
            key: value for key, value in bundle.expected_identity["configuration"].items()
            if key != "production_package_version"
        }}
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_release_build_output(output, expected)

    def test_release_package_locator_fails(self):
        """REQ-SOP-RUNNER-022: a same-name Cargo package locator must match production directory/version."""
        for replacement in (
            "path+file:///unrelated#parseltongue-cli-dispatcher-binary@0.1.0",
            "path+file:///production#parseltongue-cli-dispatcher-binary@9.9.9",
        ):
            with self.subTest(replacement=replacement):
                bundle = SyntheticEvidenceBundle()
                receipt_path = bundle.receipt_paths("release_build")[0]
                receipt = json.loads(bundle.file_bytes[receipt_path])
                stdout_path = receipt["stdout"]["path"]
                messages = [json.loads(line) for line in bundle.file_bytes[stdout_path].decode().splitlines()]
                messages[0]["package_id"] = replacement
                bundle.refresh_log(receipt_path, "stdout", "\n".join(json.dumps(value) for value in messages))
                result = self.build_bundle_result(bundle)
                self.assertEqual(
                    result["repository_results"]["serde"]["release_provenance"]["status"], "FAIL",
                )

    def test_release_version_only_pkgid(self):
        """REQ-SOP-RUNNER-022: Cargo path package IDs may infer name from directory."""
        bundle = SyntheticEvidenceBundle()
        expected = json.loads(json.dumps(bundle.expected_identity))
        package_directory = "/production/parseltongue-cli-dispatcher-binary"
        expected["configuration"]["production_manifest"] = f"{package_directory}/Cargo.toml"
        expected["configuration"]["production_bin_source"] = f"{package_directory}/src/main.rs"
        receipt_path = bundle.receipt_paths("release_build")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[stdout_path].decode().splitlines()]
        messages[0]["manifest_path"] = expected["configuration"]["production_manifest"]
        messages[0]["target"]["src_path"] = expected["configuration"]["production_bin_source"]
        for scheme in ("path+file://", "file://"):
            with self.subTest(scheme=scheme):
                messages[0]["package_id"] = f"{scheme}{package_directory}#0.1.0"
                intake._validate_release_build_output(
                    "\n".join(json.dumps(value) for value in messages), expected,
                )

    def test_timeout_fixture_receipt_blocks(self):
        """REQ-SOP-RUNNER-022: timeout exit 124 without complete test results is BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt.update({
            "timed_out": True, "completion_status": "timed_out", "exit_status": 124,
            "elapsed_seconds": 30.1,
        }))
        bundle.refresh_log(
            receipt_path, "stdout",
            "running 2 tests\ntest fixture::test_crate_graph_offline_only ... ok\n",
        )
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "BLOCKED")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_timeout_fixture_failure_fails(self):
        """REQ-SOP-RUNNER-022: an explicit failed fixture row remains FAIL after timeout."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fixture_suite")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt.update({
            "timed_out": True, "completion_status": "timed_out", "exit_status": 124,
            "elapsed_seconds": 30.1,
        }))
        output = synthetic_fixture_output(bundle.source_inventory)
        lines = output.splitlines()
        row_index = next(index for index, line in enumerate(lines) if line.endswith(" ... ok"))
        lines[row_index] = lines[row_index][:-2] + "FAILED"
        summary_index = next(index for index, line in enumerate(lines) if line.startswith("test result:"))
        summary = lines[summary_index].replace("test result: ok.", "test result: FAILED.")
        summary = summary.replace(" passed; 0 failed;", " passed; 1 failed;")
        lines[summary_index] = summary
        bundle.refresh_log(receipt_path, "stdout", "\n".join(lines))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["fixture_suite"]["status"], "FAIL")

    def test_unknown_target_override_blocks(self):
        """REQ-SOP-RUNNER-022: unknown CARGO_TARGET overrides block rather than imply cfg parity."""
        bundle = SyntheticEvidenceBundle()
        bundle.expected_identity["relevant_environment"]["CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER"] = "cc"
        bundle.manifest["identity_binding"] = bundle.expected_identity
        for reference in iter_manifest_references(bundle.manifest):
            bundle.refresh_receipt(reference["path"], lambda receipt: (
                receipt.update({"identity_binding": bundle.expected_identity,
                                "relevant_environment": bundle.expected_identity["relevant_environment"]})
            ))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["tokens_by_repository"], {})
        self.assertTrue(all(
            gates[gate_name]["status"] == "BLOCKED"
            for gates in result["repository_results"].values()
            for gate_name in ("fixture_suite", "build_evidence", "dependency_audit", "release_provenance")
        ))

    def test_wrong_dispatched_cargo_blocks(self):
        """REQ-SOP-RUNNER-022: receipts cannot substitute an arbitrary cargo executable path."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("fmt")[0]
        bundle.refresh_receipt(receipt_path, lambda receipt: receipt["argv"].__setitem__(0, "/tmp/other/cargo"))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED")
        self.assertEqual(result["tokens_by_repository"], {})

    def test_release_binary_source_fails(self):
        """REQ-SOP-RUNNER-022: release target source must match the frozen production bin source."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("release_build")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[stdout_path].decode().splitlines()]
        messages[0]["target"]["src_path"] = "/unrelated/src/main.rs"
        bundle.refresh_log(receipt_path, "stdout", "\n".join(json.dumps(value) for value in messages))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["release_provenance"]["status"], "FAIL")

    def test_release_malformed_target_blocks(self):
        """REQ-SOP-RUNNER-022: malformed nested Cargo target objects are typed BLOCKED outcomes."""
        bundle = SyntheticEvidenceBundle()
        receipt = json.loads(bundle.file_bytes[bundle.receipt_paths("release_build")[0]])
        stdout_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[stdout_path].decode().splitlines()]
        messages[0]["target"] = None
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_release_build_output(
                "\n".join(json.dumps(value) for value in messages), bundle.expected_identity,
            )

    def test_release_success_beats_truncated(self):
        """REQ-SOP-RUNNER-022: explicit release failure beats a later malformed JSON line."""
        bundle = SyntheticEvidenceBundle()
        output = '{"reason":"build-finished","success":false}\n{"reason":'
        with self.assertRaises(intake.S3EvidenceViolationError):
            intake._validate_release_build_output(output, bundle.expected_identity)

    def test_release_missing_success_blocks(self):
        """REQ-SOP-RUNNER-022: a build-finished message without boolean success is BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_release_build_output(
                '{"reason":"build-finished"}', bundle.expected_identity,
            )

    def test_p2_missing_finish_blocks(self):
        """REQ-SOP-RUNNER-022: missing P2 build-finished is incomplete and BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_check", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        stdout_path = receipt["stdout"]["path"]
        lines = bundle.file_bytes[stdout_path].decode().splitlines()
        bundle.refresh_log(receipt_path, "stdout", "\n".join(lines[:-1]))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED",
        )

    def test_p2_failure_beats_truncated(self):
        """REQ-SOP-RUNNER-022: explicit P2 build failure beats a later malformed JSON line."""
        bundle = SyntheticEvidenceBundle()
        metadata_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        metadata_receipt = json.loads(bundle.file_bytes[metadata_path])
        metadata = bundle.file_bytes[metadata_receipt["stdout"]["path"]].decode()
        output = '{"reason":"build-finished","success":false}\n{"reason":'
        with self.assertRaises(intake.S3EvidenceViolationError):
            intake._validate_p2_check_output(
                output, metadata, bundle.expected_repositories["serde"],
            )

    def test_p2_metadata_source_fails(self):
        """REQ-SOP-RUNNER-022: a Cargo metadata target source mismatch is FAIL."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        metadata_path = receipt["stdout"]["path"]
        metadata = json.loads(bundle.file_bytes[metadata_path])
        metadata["packages"][0]["targets"][0]["src_path"] = "/wrong/lib.rs"
        bundle.refresh_log(receipt_path, "stdout", json.dumps(metadata))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_p2_metadata_manifest_fails(self):
        """REQ-SOP-RUNNER-022: Cargo metadata from a different manifest is FAIL."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        metadata_path = receipt["stdout"]["path"]
        metadata = json.loads(bundle.file_bytes[metadata_path])
        metadata["packages"][0]["manifest_path"] = "/wrong/Cargo.toml"
        bundle.refresh_log(receipt_path, "stdout", json.dumps(metadata))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_p2_artifact_source_fails(self):
        """REQ-SOP-RUNNER-022: a P2 compiler artifact source mismatch is FAIL."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_check", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        output_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[output_path].decode().splitlines()]
        messages[0]["target"]["src_path"] = "/wrong/lib.rs"
        bundle.refresh_log(receipt_path, "stdout", "\n".join(json.dumps(value) for value in messages))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_p2_artifact_manifest_fails(self):
        """REQ-SOP-RUNNER-022: a P2 compiler artifact manifest mismatch is FAIL."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_check", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        output_path = receipt["stdout"]["path"]
        messages = [json.loads(line) for line in bundle.file_bytes[output_path].decode().splitlines()]
        messages[0]["manifest_path"] = "/wrong/Cargo.toml"
        bundle.refresh_log(receipt_path, "stdout", "\n".join(json.dumps(value) for value in messages))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "FAIL",
        )

    def test_p2_missing_source_blocks(self):
        """REQ-SOP-RUNNER-022: missing Cargo target source paths are BLOCKED."""
        bundle = SyntheticEvidenceBundle()
        receipt_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        receipt = json.loads(bundle.file_bytes[receipt_path])
        metadata_path = receipt["stdout"]["path"]
        metadata = json.loads(bundle.file_bytes[metadata_path])
        metadata["packages"][0]["targets"][0].pop("src_path")
        bundle.refresh_log(receipt_path, "stdout", json.dumps(metadata))
        result = self.build_bundle_result(bundle)
        self.assertEqual(
            result["repository_results"]["serde"]["p2_offline_qualification"]["status"], "BLOCKED",
        )

    def test_metadata_nested_malformed_blocks(self):
        """REQ-SOP-RUNNER-022: malformed package/target Cargo JSON is BLOCKED, never uncaught."""
        bundle = SyntheticEvidenceBundle()
        metadata_path = bundle.receipt_paths("p2_metadata", "serde")[0]
        receipt = json.loads(bundle.file_bytes[metadata_path])
        stdout_path = receipt["stdout"]["path"]
        metadata = json.loads(bundle.file_bytes[stdout_path])
        metadata["packages"][0]["id"] = []
        with self.assertRaises(intake.S3EvidenceBlockedError):
            intake._validate_p2_metadata_output(
                json.dumps(metadata), bundle.expected_repositories["serde"],
            )

    def test_malformed_receipt_id_blocks(self):
        """REQ-SOP-RUNNER-022: malformed nested receipt identifiers are typed BLOCKED outcomes."""
        bundle = SyntheticEvidenceBundle()
        bundle.refresh_receipt(bundle.receipt_paths("fmt")[0], lambda receipt: receipt.update({"receipt_id": []}))
        result = self.build_bundle_result(bundle)
        self.assertEqual(result["repository_results"]["serde"]["build_evidence"]["status"], "BLOCKED")


if __name__ == "__main__":
    unittest.main()
