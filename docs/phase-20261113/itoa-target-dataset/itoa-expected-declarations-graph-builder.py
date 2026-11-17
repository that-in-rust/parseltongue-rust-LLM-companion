#!/usr/bin/env python3
"""REQ-RUST-066.0: the expected graph of a run that reads the declarations of core.

Reads the frozen expected graph (scope excluded) and the frozen targets of the method
calls into core, and writes the expected graph of a run with scope declarations. Neither
input is changed. Never a production input.
"""
import argparse
import collections
import hashlib
import importlib.util
import json
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
RULE_SPEC = importlib.util.spec_from_file_location("itoa_expected_graph_rules", HERE / "itoa-expected-graph-rules.py")
RULES = importlib.util.module_from_spec(RULE_SPEC)
RULE_SPEC.loader.exec_module(RULES)

GRAPH_FREEZE_NAME = "itoa-expected-graph-freeze.json"
TARGET_FREEZE_NAME = "itoa-expected-core-target-freeze.json"
NODE_NAME, EDGE_NAME = "itoa-expected-graph-nodes.json", "itoa-expected-graph-edges.json"
TARGET_NAME = "itoa-expected-core-member-targets.json"
OUTPUT_PREFIX = "itoa-expected-declarations-graph-"
# M3: how a run names the target, by the kind of the owner in the targets file.
RECORD_BY_OWNER_KIND = {"inherent_impl": "core_inherent_member",
                        "trait_declaration_impl_not_named": "core_trait_declared_member"}
# M4: labels that are known and not merged, each with the requirement that keeps it so.
# None since REQ-RUST-073.0 (2026-11-18): `*mut T::cast` is merged like the other 132.
UNMERGED_LABELS = {}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read_frozen_inputs(directory):
    """M1: the three input files, each checked against its freeze record."""
    data = {name: (directory / name).read_bytes() for name in (NODE_NAME, EDGE_NAME, TARGET_NAME)}
    graph = json.loads((directory / GRAPH_FREEZE_NAME).read_text())["frozen_file_sha256"]
    target = json.loads((directory / TARGET_FREEZE_NAME).read_text())["frozen_file_sha256"]
    wanted = {NODE_NAME: graph[NODE_NAME], EDGE_NAME: graph[EDGE_NAME], TARGET_NAME: target[TARGET_NAME]}
    differing = sorted(name for name in data if digest(data[name]) != wanted[name])
    if differing:
        raise ValueError(f"not the frozen file: {', '.join(differing)}")
    return data


def format_site_span(site, exact):
    return (f"{site['file']} {exact['first_token_kind']}:{exact['first_token_start']}"
            f"..{exact['last_token_kind']}:{exact['last_token_end']}")


def index_target_rows(rows):
    """M2: the targets rows by source label and site span; two rows of one key fail."""
    keys = [(row["site"]["source"], row["site"]["site_span_key"]) for row in rows]
    repeated = sorted(key for key, count in collections.Counter(keys).items() if count > 1)
    if repeated:
        raise ValueError(f"two targets rows for one site: {repeated}")
    return dict(zip(keys, rows))


def split_edge_sites(edge):
    """One record per site of a grouped row, with the three lists of the row kept together."""
    return [dict(site=site, exact=exact, evidence=evidence) for site, exact, evidence in
            zip(edge["sites"], edge["site_exact_ranges"], edge["source_evidence_sites"])]


def merge_call_site(edge, part, targets):
    """M3, M4: the row head and the site of one call without a target, after the merge."""
    key = (edge["source"], format_site_span(part["site"], part["exact"]))
    if key not in targets:
        raise ValueError(f"no targets row for the site {key}")
    row = targets[key]
    kept = UNMERGED_LABELS.get(row["target_label"])
    record = kept["record"] if kept else RECORD_BY_OWNER_KIND[row["target_owner_kind"]]
    head = {**{name: edge[name] for name in ("source", "source_kind", "relation", "generated_by", "via_use_node")},
            "flags": [], "callee_name": edge["callee_name"]}
    known = {"crate": row["target_crate_name"], "owner_kind": row["target_owner_kind"],
             "declaration": row["core"]["declaration"], "assumptions": row["assumptions"]}
    if kept:
        head.update(target=edge["target"], target_kind="unresolved", resolution_status="unresolved",
                    unresolved_reason=edge["unresolved_reason"], unresolved_why=kept["why"],
                    known_target_label=row["target_label"], unmerged_requirement=kept["requirement"])
    else:
        head.update(target=row["target_label"], target_kind="external", core_target=known)
    return key, head, {**part, "site": {**part["site"], "record": record}}


def group_merged_sites(merged):
    """M6: one row per source, relation and target, sites in source order."""
    groups = collections.defaultdict(list)
    for _, head, part in merged:
        groups[(head["source"], head["relation"], head["target"])].append((head, part))
    def build(found):
        parts = sorted((part for _, part in found), key=lambda part: part["evidence"]["byte_range"])
        evidence = [part["evidence"] for part in parts]
        return {**found[0][0], "sites": [part["site"] for part in parts],
                "site_exact_ranges": [part["exact"] for part in parts], "source_evidence_sites": evidence,
                "source_site_evidence_status": "EXACT", "evidence_ref_kind": "SYMBOLIC_SOURCE_SITE_REF_NOT_CANONICAL",
                "evidence_key": "|".join(f"{site['source_path']}:{site['line']}:{site['byte_range']}" for site in evidence)}
    return [build(found) for found in groups.values()]


def merge_edge_rows(edges, target_rows):
    """M2 to M6: the edge rows of the expected graph with the targets written in."""
    targets = index_target_rows(target_rows)
    open_calls = [edge for edge in edges if edge["relation"] == "calls" and edge["target_kind"] == "unresolved"]
    merged = [merge_call_site(edge, part, targets) for edge in open_calls for part in split_edge_sites(edge)]
    unused = sorted(set(targets) - {key for key, _, _ in merged})
    if unused:
        raise ValueError(f"targets rows that no site uses: {unused}")
    kept = [edge for edge in edges if edge not in open_calls]
    return sorted(kept + group_merged_sites(merged), key=lambda row: (row["source"], row["relation"], row["target"]))


def list_merged_nodes(nodes, edges):
    """M5: the nodes of the expected graph and one node per merged label."""
    labels = sorted({edge["target"] for edge in edges if "core_target" in edge})
    clash = sorted(set(labels) & {node["label"] for node in nodes})
    if clash:
        raise ValueError(f"a merged label is already a node: {clash}")
    template = next(node for node in nodes if node["node_category"] == "external_name")
    added = [{**template, "label": label, "external_name_kind": "core_declared_member"} for label in labels]
    return sorted(nodes + added, key=lambda row: (row["file"], row["start_line"], row["label"]))


def count_sites(rows):
    return sum(len(row["sites"]) for row in rows)


def build_output_files(data):
    """The text of each file written, by name, from the bytes of the three inputs."""
    graph_nodes, graph_edges, targets = (json.loads(data[name]) for name in (NODE_NAME, EDGE_NAME, TARGET_NAME))
    edges = merge_edge_rows(graph_edges["rows"], targets["rows"])
    nodes = list_merged_nodes(graph_nodes["rows"], edges)
    open_rows = [row for row in edges if row["target_kind"] == "unresolved"]
    counts = {"nodes": len(nodes), "nodes_added": len(nodes) - len(graph_nodes["rows"]),
              "edge_rows_grouped": len(edges), "edge_sites": count_sites(edges),
              "edge_sites_merged": count_sites([row for row in edges if "core_target" in row]),
              "edge_sites_without_target": count_sites(open_rows),
              "edge_sites_known_not_merged": count_sites([row for row in open_rows if "known_target_label" in row])}
    if (counts["edge_sites"], counts["nodes_added"]) != (count_sites(graph_edges["rows"]),
                                                        len({row["target"] for row in edges if "core_target" in row})):
        raise ValueError(f"M6: the merge changed the number of sites or added a wrong number of nodes: {counts}")
    header = {name: graph_nodes[name] for name in ("provenance", "source_coverage", "identity_status",
                                                   "production_input", "source_commit", "source_pin_file")}
    header.update(artifact_status="FROZEN_EXPECTED_GRAPH", sysroot_source_scope_kind="declarations",
                  requirement="REQ-RUST-066.0", merged_from=[NODE_NAME, EDGE_NAME, TARGET_NAME],
                  core_source=targets["core_source"], unmerged_labels=UNMERGED_LABELS)
    summary = {**header, "counts": counts, "relation_kind_counts": RULES.relation_counts(edges),
               "target_kind_counts": dict(sorted(collections.Counter(row["target_kind"] for row in edges).items()))}
    documents = {"nodes.json": {**header, "row_count": len(nodes), "rows": nodes},
                 "edges.json": {**header, "row_count": len(edges),
                                "relation_kind_counts": RULES.relation_counts(edges), "rows": edges},
                 "summary.json": summary}
    return {OUTPUT_PREFIX + name: RULES.serialize_dataset(value) for name, value in documents.items()}, counts


def build_freeze_record(data, files, counts):
    """M7: the digests of the inputs, of the builder and of each file written."""
    builders = (pathlib.Path(__file__).resolve(), HERE / "itoa-expected-graph-rules.py")
    return {"input_file_sha256": {name: digest(data[name]) for name in sorted(data)},
            "frozen_file_sha256": {name: digest(text.encode()) for name, text in sorted(files.items())},
            "builder_file_sha256": {path.name: digest(path.read_bytes()) for path in builders},
            "counts": counts, "unmerged_labels": UNMERGED_LABELS}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input-directory", default=str(HERE))
    parser.add_argument("--output-directory", default=str(HERE))
    parser.add_argument("--write-freeze-manifest", action="store_true",
                        help="write the freeze record (M7); a later change then needs a source citation and a new freeze")
    args = parser.parse_args()
    data = read_frozen_inputs(pathlib.Path(args.input_directory).resolve())
    files, counts = build_output_files(data)
    output = pathlib.Path(args.output_directory).resolve()
    for name, text in files.items():
        (output / name).write_text(text)
    if args.write_freeze_manifest:
        (output / (OUTPUT_PREFIX + "freeze.json")).write_text(
            RULES.serialize_dataset(build_freeze_record(data, files, counts)))
    print(json.dumps(counts, indent=2))


if __name__ == "__main__":
    main()
