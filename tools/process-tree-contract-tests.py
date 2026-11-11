import importlib.util
import sys
import unittest
from pathlib import Path

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("auditor", Path(__file__).with_name("process-tree-access-auditor.py"))
auditor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(auditor)


class ProcessTreeContractTests(unittest.TestCase):
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

    def test_build_script_never_executed(self):
        trace = '''100 execve("/bin/parseltongue", ["parseltongue"], []) = 0
100 clone(child_stack=NULL, flags=SIGCHLD) = 101
101 openat(AT_FDCWD, "/workspace/target/debug/build/a/output", O_RDONLY) = 3
'''
        self.assertTrue(auditor.audit_process_trace_events(trace, 100))


if __name__ == "__main__":
    unittest.main()
