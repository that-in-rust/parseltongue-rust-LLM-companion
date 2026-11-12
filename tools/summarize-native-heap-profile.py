#!/usr/bin/env python3
"""Report allocation stacks live at the DHAT global heap peak."""

import argparse
import json
from pathlib import Path


def summarize_native_heap_profile(profile, limit=10):
    if type(limit) is not int or limit < 1:
        raise ValueError("limit must be a positive integer")
    if (not isinstance(profile, dict)
            or profile.get("dhatFileVersion") != 2
            or profile.get("mode") != "rust-heap"
            or profile.get("tu") != "µs"):
        raise ValueError("expected a version-2 Rust heap profile in microseconds")
    frames = profile.get("ftbl")
    points = profile.get("pps")
    if not isinstance(frames, list) or not all(isinstance(f, str) for f in frames):
        raise ValueError("invalid profile frame dictionary")
    if not isinstance(points, list):
        raise ValueError("invalid allocation points")
    for key in ("tg", "te"):
        if type(profile.get(key)) is not int or profile[key] < 0:
            raise ValueError(f"invalid profile time: {key}")

    def allocation_stack(point):
        if not isinstance(point, dict) or type(point.get("gb")) is not int or point["gb"] < 0:
            raise ValueError("invalid global-peak byte count")
        references = point.get("fs")
        if not isinstance(references, list) or not all(
            type(frame) is int and 0 <= frame < len(frames) for frame in references
        ):
            raise ValueError("invalid profile frame reference")
        return {
            "live_bytes": point["gb"],
            "frames": [frames[frame] for frame in references],
        }

    stacks = [allocation_stack(point) for point in points]
    return {
        "global_peak_bytes": sum(stack["live_bytes"] for stack in stacks),
        "global_peak_time_us": profile["tg"],
        "profile_duration_us": profile["te"],
        "top_allocation_stacks": sorted(
            (stack for stack in stacks if stack["live_bytes"]),
            key=lambda stack: (-stack["live_bytes"], stack["frames"]),
        )[:limit],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", type=Path)
    parser.add_argument("--limit", type=int, default=10)
    args = parser.parse_args()
    print(json.dumps(summarize_native_heap_profile(
        json.loads(args.profile.read_text()), args.limit
    ), indent=2))


if __name__ == "__main__":
    main()
