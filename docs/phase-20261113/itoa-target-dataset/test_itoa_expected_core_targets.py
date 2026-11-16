import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


HERE = Path(__file__).parent
BUILDER_PATH = HERE / "itoa-expected-core-target-builder.py"
TARGET_PATH = HERE / "itoa-expected-core-member-targets.json"
BUILDER_SPEC = importlib.util.spec_from_file_location("itoa_expected_core_target_builder", BUILDER_PATH)
BUILDER = importlib.util.module_from_spec(BUILDER_SPEC)
BUILDER_SPEC.loader.exec_module(BUILDER)


def read_json(path):
    return json.loads(path.read_text())


def list_citations(value):
    """Every {file, line, text} record anywhere inside a row."""
    if isinstance(value, dict):
        own = [value] if {"file", "line", "text"} <= value.keys() else []
        return own + [found for child in value.values() for found in list_citations(child)]
    if isinstance(value, list):
        return [found for child in value for found in list_citations(child)]
    return []


class ExpectedCoreTargetTests(unittest.TestCase):
    """REQ-RUST-058.0: the expected target of every method call into core, written from source."""

    @classmethod
    def setUpClass(cls):
        cls.document = read_json(TARGET_PATH)
        cls.rows = cls.document["rows"]
        cls.library = BUILDER.find_library_root(None)
        cls.core_files = BUILDER.read_core_files(cls.library)
        cls.itoa_files = {path: (BUILDER.VENDORED_SOURCE / path).read_bytes()
                          for path in cls.document["source_file_sha256"]}

    def test_core_target_site_coverage(self):
        """T1: one row for each calls site the frozen graph lists without a target, and no other row."""
        frozen = BUILDER.list_untargeted_call_sites(read_json(HERE / "itoa-expected-graph-edges.json"))
        key = lambda site: (site["source"], site["file"], tuple(site["byte_range"]), site["method_name"])
        self.assertEqual(133, len(frozen))
        self.assertEqual(sorted(map(key, frozen)), sorted(key(row["site"]) for row in self.rows))
        self.assertEqual(len(self.rows), len({key(row["site"]) for row in self.rows}))

    def test_core_target_label_or_reason(self):
        """T2: a row gives a target label, or no_target with a reason; never both, never neither."""
        for row in self.rows:
            labelled = row["expected"] == "target" and bool(row.get("target_label"))
            refused = row["expected"] == "no_target" and bool(row.get("no_target_reason"))
            self.assertNotEqual(labelled, refused, row["site"])
            self.assertFalse(labelled and "no_target_reason" in row, row["site"])

    def test_core_target_label_shape(self):
        """T3: the label is the owner as the cited core line writes it, then the method name."""
        for row in (row for row in self.rows if row["expected"] == "target"):
            owner = BUILDER.read_owner_label(row["core"]["owner"]["text"])
            self.assertEqual(f"{owner}::{row['site']['method_name']}", row["target_label"])
            self.assertIn(f"fn {row['site']['method_name']}", row["core"]["declaration"]["text"])
            self.assertEqual("trait" in row["core"]["owner"]["text"].split(), row["target_owner_kind"] == "trait")
            self.assertIn(row["lookup_step"]["step_kind"], BUILDER.STEP_KINDS)
            self.assertIn(row["lookup_step"]["self_adjustment"], BUILDER.SELF_ADJUSTMENTS)

    def test_core_declaration_citation_text(self):
        """T4: every cited line of itoa and of core holds the cited text in the bytes read."""
        files = {**self.core_files, **self.itoa_files}
        lines = {path: data.decode().split("\n") for path, data in files.items()}
        citations = [citation for row in self.rows for citation in list_citations(row)]
        self.assertGreater(len(citations), 3 * len(self.rows))
        for citation in citations:
            self.assertIn(citation["text"], lines[citation["file"]][citation["line"] - 1], citation)
        for row in self.rows:
            site = row["site"]
            start, end = site["byte_range"]
            self.assertEqual(site["anchor_text"], self.itoa_files[site["file"]][start:end].decode())

    def test_core_target_assumption_names(self):
        """T5: assumptions come from the vocabulary; the calls that need one carry it."""
        kinds = lambda row: [assumption["assumption_kind"] for assumption in row["assumptions"]]
        for row in self.rows:
            self.assertLessEqual(set(kinds(row)), set(BUILDER.ASSUMPTION_KINDS), row["site"])
        buffer_len = [row for row in self.rows if row["site"]["method_name"] == "len" and row["receiver_text"] == "buf"]
        self.assertEqual(5, len(buffer_len))
        for row in buffer_len:
            self.assertEqual(["prelude_trait_impl_unrefuted"], kinds(row))
            self.assertEqual("[T]::len", row["target_label"])
        in_macro = [row for row in self.rows if "declared_in_macro" in row["core"]]
        self.assertEqual(10, len(in_macro))
        for row in in_macro:
            self.assertEqual(["core_macro_body_declaration"], kinds(row))
        carrying = [row for row in self.rows if row["assumptions"]]
        self.assertEqual(15, len(carrying))

    def test_core_name_scope_counts(self):
        """T4: the name-scope notes that say 'declared only by' rest on these counts of `fn NAME` in core."""
        counts = {name: len(found) for name, found in self.document["name_declarations_in_core"].items()}
        self.assertEqual({"try_into": 2, "rev": 1, "expect": 2, "unwrap": 2, "ilog10": 3},
                         {name: counts[name] for name in ("try_into", "rev", "expect", "unwrap", "ilog10")})
        for name, found in self.document["name_declarations_in_core"].items():
            self.assertEqual(found, BUILDER.list_name_declarations(self.core_files, name))

    def test_core_source_digest_recorded(self):
        """T6: the recorded compiler version and core digest are those of the files read."""
        recorded = self.document["core_source"]
        self.assertEqual(BUILDER.read_compiler_version(), recorded["compiler_version_lines"])
        self.assertEqual(BUILDER.digest_core_files(self.core_files), recorded["listing_sha256"])
        self.assertEqual(len(self.core_files), recorded["file_count"])
        shell = subprocess.run("find core/src -name '*.rs' | LC_ALL=C sort | xargs sha256sum | sha256sum",
                               shell=True, cwd=self.library, check=True, capture_output=True, text=True)
        self.assertEqual(recorded["listing_sha256"], shell.stdout.split()[0])

    def test_core_target_builder_reproduces(self):
        """The builder writes the same bytes again, and the file is not a production input."""
        with tempfile.TemporaryDirectory() as output:
            subprocess.run([sys.executable, str(BUILDER_PATH), "--output-directory", output], check=True, capture_output=True)
            self.assertEqual(TARGET_PATH.read_bytes(), (Path(output) / TARGET_PATH.name).read_bytes())
        self.assertFalse(self.document["production_input"])
        crates = BUILDER.ROOT / "crates"
        named = [path for path in crates.rglob("*.rs") if "/tests/" not in path.as_posix()
                 and TARGET_PATH.name in path.read_text(errors="ignore")]
        self.assertEqual([], named)

    def test_core_citation_error_stops(self):
        """A citation that the bytes do not hold stops the builder with the line named."""
        read = BUILDER.make_line_reader({"a.rs": b"fn one() {}\n"})
        with self.assertRaises(BUILDER.CitationError) as raised:
            BUILDER.cite(read, "a.rs", 1, "fn two")
        self.assertIn("a.rs:1", str(raised.exception))


if __name__ == "__main__":
    unittest.main()
