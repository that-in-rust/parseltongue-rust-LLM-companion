"""Follow rust-analyzer LSIF range -> resultSet -> definitionResult -> item."""
import json
import subprocess
from collections import defaultdict
from pathlib import Path
from urllib.parse import unquote, urlparse

from corpus import identify_definition_query_positions


def collect_rust_analyzer_labels(root, manifest):
    files, ranges, owner, nxt, defs, items = {}, {}, {}, {}, {}, defaultdict(list)
    with subprocess.Popen(["rust-analyzer", "lsif", "--exclude-vendored-libraries", "."],
                          cwd=root, stdout=subprocess.PIPE, text=True) as proc:
        for line in proc.stdout:
            row = json.loads(line)
            tag = row["label"]
            if tag == "document":
                files[row["id"]] = Path(unquote(urlparse(row["uri"]).path)).resolve()
            elif tag == "range":
                ranges[row["id"]] = row["start"]["line"], row["start"]["character"]
            elif tag == "contains" and row["outV"] in files:
                owner.update((rid, row["outV"]) for rid in row["inVs"])
            elif tag == "next":
                nxt[row["outV"]] = row["inV"]
            elif tag == "textDocument/definition":
                defs[row["outV"]] = row["inV"]
            elif tag == "item":
                items[row["outV"]].extend(row["inVs"])
        if proc.wait():
            raise RuntimeError("rust-analyzer lsif failed")
    locations = {(files[doc], ranges[rid]): rid for rid, doc in owner.items() if rid in ranges}
    for row in manifest:
        path = (root / "src" / (row["case"] + ".rs")).resolve()
        symbols, query = identify_definition_query_positions(path)
        v = locations.get((path, query))
        while v is not None and v not in defs:
            v = nxt.get(v)
        destinations = items.get(defs.get(v), [])
        targets = {symbols[ranges[t]] for t in destinations if files.get(owner.get(t)) == path and ranges.get(t) in symbols}
        if destinations and (len(destinations) != 1 or len(targets) != 1):
            raise RuntimeError(f"unexpected oracle location: {row['case']} {destinations}")
        row["ra_target"] = next(iter(targets), None)
        if (row["ra_target"] is None) != (row["split"] == "missing" or row["kind"] == "missing_prefix"):
            raise RuntimeError(f"unexpected oracle coverage: {row['case']}: {row['ra_target']}")
