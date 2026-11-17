import collections
import hashlib
import importlib.util
import json
import unittest
from pathlib import Path


HERE = Path(__file__).parent
BUILDER_PATH = HERE / "itoa-expected-declarations-graph-builder.py"
BUILDER_SPEC = importlib.util.spec_from_file_location("itoa_expected_declarations_graph_builder", BUILDER_PATH)
BUILDER = importlib.util.module_from_spec(BUILDER_SPEC)
BUILDER_SPEC.loader.exec_module(BUILDER)
PREFIX = BUILDER.OUTPUT_PREFIX


def read_json(name):
    return json.loads((HERE / name).read_text())


class ExpectedDeclarationsGraphTests(unittest.TestCase):
    """REQ-RUST-066.0: the expected graph of a run that reads the declarations of core."""

    @classmethod
    def setUpClass(cls):
        cls.data = BUILDER.read_frozen_inputs(HERE)
        cls.nodes = read_json(PREFIX + "nodes.json")["rows"]
        cls.edges = read_json(PREFIX + "edges.json")["rows"]
        cls.freeze = read_json(PREFIX + "freeze.json")

    def test_rebuild_equals_frozen_files(self):
        """M1, M7: a rebuild from the frozen inputs writes the frozen bytes."""
        files, counts = BUILDER.build_output_files(self.data)
        self.assertEqual({name: (HERE / name).read_text() for name in files}, files)
        self.assertEqual(BUILDER.build_freeze_record(self.data, files, counts), self.freeze)

    def test_inputs_stay_frozen_files(self):
        """M1: the freeze record names the three inputs by the digest their own freeze records hold."""
        graph = read_json(BUILDER.GRAPH_FREEZE_NAME)["frozen_file_sha256"]
        target = read_json(BUILDER.TARGET_FREEZE_NAME)["frozen_file_sha256"]
        wanted = {name: {**graph, **target}[name] for name in self.data}
        self.assertEqual(wanted, self.freeze["input_file_sha256"])
        self.assertEqual({name: hashlib.sha256((HERE / name).read_bytes()).hexdigest()
                          for name in self.freeze["frozen_file_sha256"]}, self.freeze["frozen_file_sha256"])

    def test_missing_target_row_fails(self):
        """M2: a site without a targets row, and a targets row without a site, fail the build."""
        edges = json.loads(self.data[BUILDER.EDGE_NAME])["rows"]
        targets = json.loads(self.data[BUILDER.TARGET_NAME])["rows"]
        with self.assertRaisesRegex(ValueError, "no targets row for the site"):
            BUILDER.merge_edge_rows(edges, targets[1:])
        moved = {**targets[0], "site": {**targets[0]["site"], "source": "no such item"}}
        with self.assertRaisesRegex(ValueError, "no targets row for the site"):
            BUILDER.merge_edge_rows(edges, [moved] + targets[1:])
        with self.assertRaisesRegex(ValueError, "two targets rows for one site"):
            BUILDER.merge_edge_rows(edges, targets + targets[:1])

    def test_merged_site_record_counts(self):
        """M3, M4, M6: 686 sites; 132 merged by the owner kind of their row; the cast is not."""
        sites = [(row, site) for row in self.edges for site in row["sites"]]
        self.assertEqual(686, len(sites))
        records = collections.Counter(site["record"] for row, site in sites if "core_target" in row)
        self.assertEqual({"core_inherent_member": 118, "core_trait_declared_member": 14}, dict(records))
        open_rows = [row for row in self.edges if row["target_kind"] == "unresolved"]
        self.assertEqual([("unresolved .cast", "*mut T::cast", "REQ-RUST-062.0 C7", 1)],
                         [(row["target"], row["known_target_label"], row["unmerged_requirement"], len(row["sites"]))
                          for row in open_rows])
        self.assertEqual(["undetermined/inherent_member_unproven"], [site["record"] for site in open_rows[0]["sites"]])
        self.assertTrue(all(len(row["sites"]) == len(row["site_exact_ranges"]) == len(row["source_evidence_sites"])
                            for row in self.edges if row["site_exact_ranges"] is not None))

    def test_merged_label_node_rows(self):
        """M5: 153 nodes of the expected graph and one core_declared_member node per merged label."""
        added = [node for node in self.nodes if node.get("external_name_kind") == "core_declared_member"]
        self.assertEqual((172, 19), (len(self.nodes), len(added)))
        self.assertEqual({row["target"] for row in self.edges if "core_target" in row}, {node["label"] for node in added})
        self.assertNotIn("*mut T::cast", {node["label"] for node in self.nodes})
        self.assertTrue(all((node["node_category"], node["file"], node["item_kind"], node["owner_kind"])
                            == ("external_name", "", None, None) for node in added))
        base = json.loads(self.data[BUILDER.NODE_NAME])["rows"]
        self.assertEqual(base, [node for node in self.nodes if node not in added])


if __name__ == "__main__":
    unittest.main()
