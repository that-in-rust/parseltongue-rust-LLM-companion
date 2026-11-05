"""Pin unmodified releases and capture offline oracle output, never model input."""

import gzip
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

CRATES = {
    "indexmap-2.12.0": "6717a8d2a5a929a1a2eb43a12812498ed141a0bcfb7e8f7844fbdbe4303bba9f",
    "hashbrown-0.16.0": "5419bdc4f6a9207fbeba6d11b604d481addf78ecd10c11ad51e76c2f6482748d",
    "aho-corasick-1.1.3": "8e60d3430d3a69478ad0993f19238d2df97c507009a52b3c10addcd7f6bcb916",
    "regex-automata-0.4.13": "5276caf25ac86c8d810222b3dbb938e512c55c6831a10f3e6ed1c93b84041f1c",
    "futures-channel-0.3.31": "2dff15bf788c671c1934e366d07e30c1814a8ef514e1af724a602e8a2fbe1b10",
}


def prepare_real_crate_sources(work, releases=CRATES):
    checksums = {}
    for release in releases:
        name, version = release.rsplit("-", 1)
        registry = Path.home() / ".cargo/registry"
        archives = sorted(registry.glob(f"cache/*/{release}.crate"))
        if not archives:
            subprocess.run(["cargo", "info", f"{name}@{version}"], check=True)
            archives = sorted(registry.glob(f"cache/*/{release}.crate"))
        # Extract from the actual release archive rather than a mutable clone.
        archive = archives[0]
        checksums[release] = hashlib.sha256(archive.read_bytes()).hexdigest()
        if checksums[release] != releases[release]:
            raise ValueError(f"Unexpected archive checksum: {release}")
        subprocess.run(["tar", "-xzf", archive, "-C", work], check=True)
        crate = work / release
        dump = work / (release + ".lsif.gz")
        with (
            gzip.open(dump, "wb", compresslevel=1) as dst,
            subprocess.Popen(
                ["rust-analyzer", "lsif", "--exclude-vendored-libraries", "."],
                cwd=crate,
                stdout=subprocess.PIPE,
            ) as proc,
        ):
            shutil.copyfileobj(proc.stdout, dst)
            if proc.wait():
                raise RuntimeError(f"rust-analyzer failed on {release}")
    (work / "archives.json").write_text(json.dumps(checksums, indent=2) + "\n")


if __name__ == "__main__":
    work = Path(sys.argv[1]).resolve()
    work.mkdir(parents=True, exist_ok=True)
    prepare_real_crate_sources(work)
