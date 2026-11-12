#!/usr/bin/env python3
"""Report allocation stacks live at the DHAT global heap peak."""

import argparse
import json
from pathlib import Path


def summarize_native_heap_profile(profile, limit=10):
    raise NotImplementedError("native heap summary not implemented")


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
