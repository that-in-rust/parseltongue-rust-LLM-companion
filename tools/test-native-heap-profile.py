import importlib.util
import unittest
from pathlib import Path

SPEC = importlib.util.spec_from_file_location(
    "native_heap_profile", Path(__file__).with_name("summarize-native-heap-profile.py")
)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def profile_fixture(points=None):
    return {
        "dhatFileVersion": 2,
        "mode": "rust-heap",
        "tu": "µs",
        "tg": 15,
        "te": 20,
        "ftbl": ["allocator", "binding clone", "import round"],
        "pps": [] if points is None else points,
    }


class NativeHeapProfileContracts(unittest.TestCase):
    def test_global_peak_not_independent_point_maxima(self):
        profile = profile_fixture([
            {"gb": index, "mb": 100 - index, "fs": [0, 1, 2]}
            for index in range(13)
        ])
        summary = MODULE.summarize_native_heap_profile(profile)
        self.assertEqual(summary["global_peak_bytes"], 78)
        self.assertEqual(summary["global_peak_time_us"], 15)
        self.assertEqual(summary["profile_duration_us"], 20)
        self.assertEqual(
            [row["live_bytes"] for row in summary["top_allocation_stacks"]],
            list(range(12, 2, -1)),
        )
        self.assertEqual(
            summary["top_allocation_stacks"][0]["frames"],
            ["allocator", "binding clone", "import round"],
        )

    def test_empty_profile(self):
        summary = MODULE.summarize_native_heap_profile(profile_fixture())
        self.assertEqual(summary["global_peak_bytes"], 0)
        self.assertEqual(summary["top_allocation_stacks"], [])

    def test_reject_wrong_mode(self):
        profile = profile_fixture()
        profile["mode"] = "ad-hoc"
        with self.assertRaises(ValueError):
            MODULE.summarize_native_heap_profile(profile)

    def test_reject_negative_byte_counts(self):
        with self.assertRaises(ValueError):
            MODULE.summarize_native_heap_profile(
                profile_fixture([{"gb": -1, "fs": [0]}])
            )

    def test_reject_unknown_frame_reference(self):
        with self.assertRaises(ValueError):
            MODULE.summarize_native_heap_profile(
                profile_fixture([{"gb": 5, "fs": [3]}])
            )

    def test_reject_invalid_limit(self):
        with self.assertRaises(ValueError):
            MODULE.summarize_native_heap_profile(profile_fixture(), 0)


if __name__ == "__main__":
    unittest.main()
