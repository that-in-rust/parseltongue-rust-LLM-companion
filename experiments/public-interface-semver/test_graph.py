import json
import unittest
from pathlib import Path

from query import find_incident_graph_edges

HERE = Path(__file__).resolve().parent


class PublicInterfaceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.graph = json.loads((HERE / "graph-default.json").read_text())
        cls.nodes = {n["id"]: n for n in cls.graph["nodes"]}

    def test_directed_signature_queries(self):
        out = find_incident_graph_edges(
            self.graph, "semver::VersionReq::matches", "out", "parameter"
        )
        self.assertEqual(
            [(e["slot"], e["target"]) for e in out], [("version", "semver::Version")]
        )
        incoming = find_incident_graph_edges(
            self.graph, "semver::Error", "in", "returns"
        )
        self.assertEqual(
            {e["source"] for e in incoming},
            {
                f"semver::{x}"
                for x in (
                    "Version::parse",
                    "VersionReq::parse",
                    "Comparator::parse",
                    "Prerelease::new",
                    "BuildMetadata::new",
                )
            },
        )

    def test_public_visibility_boundaries(self):
        exported = find_incident_graph_edges(self.graph, "semver", "out", "reexports")
        self.assertEqual([e["target"] for e in exported], ["semver::Error"])
        self.assertEqual(self.nodes["semver::Error"]["span"]["file"], "src/parse.rs")
        self.assertEqual(exported[0]["span"]["file"], "src/lib.rs")
        self.assertFalse(
            any(
                k.endswith("::ErrorKind") or k.endswith("::Identifier")
                for k in self.nodes
            )
        )

    def test_nested_local_targets(self):
        field = "semver::VersionReq::comparators"
        self.assertEqual(self.nodes[field]["type"], "alloc::vec::Vec<Comparator>")
        targets = {
            e["target"]
            for e in find_incident_graph_edges(self.graph, field, "out", "field_type")
        }
        self.assertEqual(targets, {"alloc::vec::Vec", "semver::Comparator"})
        refs = find_incident_graph_edges(
            self.graph, "semver::Version::cmp_precedence", "out", "parameter"
        )
        self.assertEqual(
            [(e["slot"], e["target"]) for e in refs], [("other", "semver::Version")]
        )

    def test_feature_specific_delta(self):
        def edge_keys(graph):
            return {
                (e["source"], e["role"], e["target"], e.get("slot"))
                for e in graph["edges"]
            }

        base = edge_keys(self.graph)
        opt = edge_keys(json.loads((HERE / "graph-serde.json").read_text()))
        self.assertFalse(base - opt)
        self.assertEqual(
            opt - base,
            {
                ("semver::" + name, "implements", "serde_core::" + trait, None)
                for name in ("Version", "VersionReq", "Comparator")
                for trait in ("ser::Serialize", "de::Deserialize")
            },
        )

    def test_all_evidence_anchors(self):
        for e in self.graph["edges"]:
            self.assertIn(e["source"], self.nodes)
            self.assertIn(e["target"], self.nodes)
        for item in self.graph["nodes"] + self.graph["edges"]:
            if "span" not in item:
                continue
            span = item["span"]
            src = (HERE / "work/semver" / span["file"]).read_text().splitlines()
            self.assertTrue(1 <= span["begin"][0] <= span["end"][0] <= len(src))
