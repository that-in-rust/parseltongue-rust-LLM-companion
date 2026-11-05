"""Read the published graph without a compiler, database or model."""

import argparse
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent


def find_incident_graph_edges(graph, node, direction, role=None):
    if not any(n["id"] == node for n in graph["nodes"]):
        raise ValueError(f"unknown graph node: {node}")
    key = {"in": "target", "out": "source"}[direction]
    return [
        e
        for e in graph["edges"]
        if e[key] == node and (role is None or e["role"] == role)
    ]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("direction", choices=["in", "out"])
    parser.add_argument("node")
    parser.add_argument("--role")
    parser.add_argument("--profile", choices=["default", "serde"], default="default")
    args = parser.parse_args()
    graph = json.loads((HERE / f"graph-{args.profile}.json").read_text())
    try:
        rows = find_incident_graph_edges(graph, args.node, args.direction, args.role)
    except ValueError as err:
        parser.error(str(err))
    for e in rows:
        s = e["span"]
        print(
            f"{e['source']} --{e['role']}--> {e['target']}  {s['file']}:{s['begin'][0]}"
        )
