import json
import subprocess
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
BINARY = HERE.parent / "target/release/learned-binding"
SOURCE = """pub mod metal { pub struct Pin; }
pub mod wood { pub struct Pin; }
use self::metal::Pin as Chosen;
use self::wood::Pin as Decoy;
pub fn make() -> Chosen { loop {} }
"""


class BindingTest(unittest.TestCase):
    def test_change_binding_without_reordering(self):
        for name, code in [("metal", SOURCE), ("wood", SOURCE.replace("Chosen;", "Temp;").replace("Decoy;", "Chosen;").replace("Temp;", "Decoy;"))]:
            p = subprocess.run([BINARY, "predict", HERE / "weights.json", "-"], input=code,
                               text=True, capture_output=True)
            self.assertEqual(p.returncode, 0, p.stderr)
            self.assertEqual(json.loads(p.stdout)["target"], name + "::Pin")

    def test_no_binding_is_not_a_target(self):
        p = subprocess.run([BINARY, "predict", HERE / "weights.json", "-"],
                           input=SOURCE.replace("-> Chosen", "-> Missing"), text=True, capture_output=True)
        self.assertEqual(p.returncode, 0, p.stderr)
        self.assertIsNone(json.loads(p.stdout)["target"])

    def test_weights_actually_determine_binding(self):
        weights = json.loads((HERE / "weights.json").read_text())
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / "weights.json"
            for control, target in [({}, None), ({k:-v for k,v in weights.items()}, "wood::Pin")]:
                path.write_text(json.dumps(control))
                p = subprocess.run([BINARY, "predict", path, "-"], input=SOURCE, text=True, capture_output=True)
                self.assertEqual(p.returncode, 0, p.stderr)
                self.assertEqual(json.loads(p.stdout)["target"], target)


if __name__ == "__main__":
    unittest.main()
