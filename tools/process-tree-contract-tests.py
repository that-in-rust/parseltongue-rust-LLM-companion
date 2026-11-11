import importlib.util
import sys
import tempfile
import json
import unittest
from unittest.mock import patch
from pathlib import Path

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("auditor", Path(__file__).with_name("process-tree-access-auditor.py"))
auditor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(auditor)
driver_spec = importlib.util.spec_from_file_location("driver", Path(__file__).with_name("corpus-production-stage-measurements.py"))
driver = importlib.util.module_from_spec(driver_spec)
driver_spec.loader.exec_module(driver)


class ProcessTreeContractTests(unittest.TestCase):
    def test_tree_walk_stays_local(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            records = {10: (1, 10, {10: [20, 99], 11: [30]}),
                       20: (10, 20, {20: [40]}), 30: (10, 30, {30: []}),
                       40: (20, 40, {40: [10]}), 50: (1, 100000, {50: []})}
            for pid, (parent, rss, tasks) in records.items():
                process = root / str(pid)
                process.mkdir()
                (process / "status").write_text(f"Tgid:\t{pid}\nPPid:\t{parent}\nVmRSS:\t{rss} kB\n")
                for tid, children in tasks.items():
                    task = process / "task" / str(tid)
                    task.mkdir(parents=True)
                    (task / "children").write_text(" ".join(map(str, children)))
            original = Path.read_text
            def read_scoped_process_files(path, *args, **kwargs):
                self.assertTrue(path.is_relative_to(root))
                self.assertNotEqual(path, root / "50" / "status")
                return original(path, *args, **kwargs)
            with patch.object(Path, "read_text", autospec=True, side_effect=read_scoped_process_files):
                self.assertEqual(auditor.read_process_tree_residency(10, root), 100 * 1024)
                self.assertEqual(auditor.read_process_tree_residency(99, root), 0)
            with patch.object(Path, "read_text", side_effect=PermissionError("denied")):
                with self.assertRaises(PermissionError):
                    auditor.read_process_tree_residency(10, root)

    def test_measurement_audit_runs_separated(self):
        record = {"corpus_source_checkout_path": "/source/repo"}
        measurement, audit, reads = driver.build_corpus_measurement_commands(record, Path("/evidence"))
        self.assertNotIn("strace", measurement)
        self.assertIn("sample", measurement)
        self.assertEqual(audit[0], "strace")
        self.assertNotIn("sample", audit)
        self.assertNotIn("openat", audit[audit.index("-e") + 1])
        self.assertIn("execve", audit[audit.index("-e") + 1])
        self.assertIn("connect", audit[audit.index("-e") + 1])
        self.assertNotEqual(measurement[measurement.index("--workspace-store-file-path") + 1],
                            audit[audit.index("--workspace-store-file-path") + 1])
        self.assertIn("-P", reads)
        self.assertIn("/source/repo/target", reads)
    def test_crate_graph_offline_only(self):
        trace = '''100 execve("/bin/parseltongue", ["parseltongue", "map-workspace-full-run"], []) = 0
100 clone(child_stack=NULL, flags=SIGCHLD) = 101
101 execve("/bin/cargo", ["cargo", "metadata", "--offline"], []) = 0
101 clone(child_stack=NULL, flags=SIGCHLD) = 102
102 execve("/bin/rustc", ["rustc", "-vV"], []) = 0
101 clone(child_stack=NULL, flags=SIGCHLD) = 103
103 execve("/bin/rustc", ["rustc", "-", "--crate-name", "___", "--print=cfg", "--print=sysroot"], []) = 0
'''
        self.assertEqual(auditor.audit_process_trace_events(trace, 100), [])
        for bad in [
            '103 execve("/bin/rustc", ["rustc", "real.rs"], []) = 0',
            '103 execve("/bin/rustc", ["rustc", "-", "--crate-name", "___", "--print=cfg", "-o", "output"], []) = 0',
            '103 execve("/bin/build-script-build", ["build-script-build"], []) = 0',
            '103 execve("/bin/curl", ["curl", "https://example.com"], []) = 0',
            '103 connect(4, {sa_family=AF_INET, sin_port=htons(443)}, 16) = 0',
            '103 execve("/bin/rustc", ["rustc", "-", "--crate-name", "___", "--crate-type", "bin", "--print=cfg"], []) = 0',
        ]:
            with self.subTest(bad=bad):
                self.assertTrue(auditor.audit_process_trace_events(trace + bad + "\n", 100))
        direct = '100 clone(child_stack=NULL, flags=SIGCHLD) = 104\n104 execve("/bin/rustc", ["rustc", "-vV"], []) = 0\n'
        self.assertTrue(auditor.audit_process_trace_events(trace + direct, 100))
        self.assertTrue(auditor.audit_process_trace_events("", 100))
        probe = ["-", "--crate-name", "___", "--print=file-names"]
        for selector in ["bin", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"]:
            probe.extend(["--crate-type", selector])
        probe.extend(["--print=sysroot", "--print=split-debuginfo", "--print=crate-name", "--print=cfg", "-Wwarnings"])
        self.assertTrue(auditor.check_rustc_discovery_arguments(probe))
        for option in ["--emit=link", "--out-dir=/tmp/output", "--print=cfg=/tmp/output", "-o", "real.rs"]:
            self.assertFalse(auditor.check_rustc_discovery_arguments(probe + [option]))

    def test_process_tree_memory_sampling(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "samples.json"
            self.assertEqual(auditor.sample_process_tree_memory([sys.executable, "-c", "import time; data=bytearray(2**20); time.sleep(.05)"], str(output)), 0)
            result = json.loads(output.read_text())
            self.assertGreater(result["process_tree_peak_bytes"], 0)
            self.assertGreater(result["process_tree_elapsed_seconds"], 0)
            self.assertTrue(result["process_memory_sample_values"])
            self.assertEqual(auditor.sample_process_tree_memory([sys.executable, "-c", "raise SystemExit(7)"], str(output)), 7)
            with self.assertRaises(ValueError):
                auditor.sample_process_tree_memory([], str(output))

    def test_build_script_never_executed(self):
        trace = '''100 execve("/bin/parseltongue", ["parseltongue"], []) = 0
100 clone(child_stack=NULL, flags=SIGCHLD) = 101
101 openat(AT_FDCWD, "/workspace/target/debug/build/a/output", O_RDONLY) = 3
'''
        self.assertTrue(auditor.audit_process_trace_events(trace, 100))


if __name__ == "__main__":
    unittest.main()
