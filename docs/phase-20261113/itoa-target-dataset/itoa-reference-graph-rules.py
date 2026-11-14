import hashlib
import json
from collections import Counter, defaultdict


FORBIDDEN_HOPS = {"contains", "imports"}
COLLAPSIBLE_RELATIONS = {"calls", "uses_type", "implements", "reexports", "derives", "reads"}
RELATION_KINDS = ("calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads")
ELIGIBLE_LOCAL_ONLY = "ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC"
CANDIDATE_STATUS = "LLM_PARSED_DRAFT"
BUILTIN_PRIMITIVES = {"bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize",
                      "i8", "i16", "i32", "i64", "i128", "isize", "f16", "f32", "f64", "f128"}
OUTPUTS = ["hld-current-interface-nodes", "hld-current-dependency-edges", "hld-edge-evidence-paths",
           "hld-file-level-rollup", "hld-folder-level-rollup"]


def _node_blockers(row):
    flags = set(row.get("flags", []))
    blockers = []
    visibility = row.get("effective_visibility")
    if visibility not in {"public", "restricted_non_private", "private"}:
        blockers.append("UNKNOWN_VISIBILITY")
    if row.get("generated_by"):
        blockers.extend(("OD-2_MACRO_OWNERSHIP_UNDECIDED", "MACRO_FILE_ASSIGNMENT_UNRESOLVED"))
    if row.get("source_evidence") and row["source_evidence"].get("range_status") != "EXACT_ANCHOR":
        blockers.append("SOURCE_SITE_RANGE_UNVERIFIED")
    if any(token in row.get("label", "") for token in (" as Default>::", " as Clone>::")):
        blockers.append("EXTERNAL_TRAIT_VISIBILITY_UNVERIFIED")
    if any(flag.startswith("OD-2") for flag in flags):
        blockers.append("OD-2_MACRO_OWNERSHIP_UNDECIDED")
    return sorted(set(blockers))


def _edge_blockers(row, node_blockers):
    blockers = []
    flags = set(row.get("flags", []))
    relation = row.get("relation")
    resolution = row.get("resolution_status")
    if relation not in RELATION_KINDS:
        blockers.append("UNKNOWN_RELATION_KIND")
    if any(flag.startswith("OD-7") for flag in flags):
        blockers.append("OD-7_OWNERSHIP_UNDECIDED")
    if any(flag.startswith("OD-8") for flag in flags):
        blockers.append("OD-8_NON_FUNCTION_RULE_UNDECIDED")
    if row.get("generated_by"):
        blockers.extend(("OD-2_MACRO_OWNERSHIP_UNDECIDED", "MACRO_FILE_ASSIGNMENT_UNRESOLVED"))
    if resolution and resolution.lower() not in {"extracted", "resolved"}:
        blockers.append("RESOLUTION_NOT_EXTRACTED")
    if row.get("target_kind") == "external":
        blockers.append("EXTERNAL_TARGET_IDENTITY_UNVERIFIED")
    if row.get("target_kind") == "builtin":
        if row.get("target") not in BUILTIN_PRIMITIVES:
            blockers.append("UNKNOWN_BUILTIN_IDENTITY")
        elif not row.get("sites") and not row.get("evidence_key"):
            blockers.append("BUILTIN_SOURCE_EVIDENCE_MISSING")
    if row.get("source_kind") != "node":
        blockers.append("SOURCE_ITEM_IDENTITY_UNVERIFIED")
    concrete_flags = ("concrete_impl", "concrete_receiver", "core_return_type", "chosen_from_argument_type")
    if any(any(flag_token in flag for flag_token in concrete_flags) for flag in flags):
        blockers.append("INFERRED_CORE_DEPENDENT_CALL_UNDECIDED")
    for endpoint_field, kind_field in (("source", "source_kind"), ("target", "target_kind")):
        endpoint = row.get(endpoint_field)
        if row.get(kind_field) == "node":
            if not endpoint or endpoint not in node_blockers:
                blockers.append("DANGLING_NODE_ENDPOINT")
            else:
                blockers.extend(node_blockers[endpoint])
    if not row.get("sites") and not row.get("evidence_key"):
        blockers.append("SOURCE_SITE_EVIDENCE_UNVERIFIED")
    if row.get("source_site_evidence_status") and row["source_site_evidence_status"] != "EXACT":
        blockers.append("SOURCE_SITE_RANGE_UNVERIFIED")
    return sorted(set(blockers))


def classify_candidates(nodes, edges):
    node_rows = []
    node_blockers = {}
    for source in nodes:
        row = dict(source)
        row["candidate_status"] = CANDIDATE_STATUS
        blockers = _node_blockers(row)
        row["blockers"] = blockers
        row["eligibility_status"] = ELIGIBLE_LOCAL_ONLY if not blockers else "BLOCKED"
        node_blockers[row.get("label")] = blockers
        node_rows.append(row)
    edge_rows = []
    for source in edges:
        row = dict(source)
        row["candidate_status"] = CANDIDATE_STATUS
        blockers = _edge_blockers(row, node_blockers)
        row["blockers"] = blockers
        row["eligibility_status"] = ELIGIBLE_LOCAL_ONLY if not blockers else "BLOCKED"
        edge_rows.append(row)
    return {
        "nodes": node_rows,
        "edges": edge_rows,
        "eligible_nodes": [row for row in node_rows if row["eligibility_status"] == ELIGIBLE_LOCAL_ONLY],
        "eligible_edges": [row for row in edge_rows if row["eligibility_status"] == ELIGIBLE_LOCAL_ONLY],
        "blocked_nodes": [row for row in node_rows if row["eligibility_status"] == "BLOCKED"],
        "blocked_edges": [row for row in edge_rows if row["eligibility_status"] == "BLOCKED"],
    }


def classify_diagnostic_site(site):
    row = dict(site)
    verified = bool(row.get("reason_verified"))
    row["reason_claim_status"] = "VERIFIED_SOURCE_FACT" if verified else "NOT_ASSERTED"
    row["eligibility_status"] = "BLOCKED" if not verified else "DIAGNOSTIC_NOT_EDGE"
    return row


def relation_counts(edges):
    counts = Counter(edge.get("relation") for edge in edges)
    return {relation: counts.get(relation, 0) for relation in RELATION_KINDS}


def derive_interface(nodes, edges):
    selected = {node["label"] for node in nodes if node.get("interface_selected")}
    outgoing = defaultdict(list)
    for edge in edges:
        if edge.get("eligibility_status", ELIGIBLE_LOCAL_ONLY) == ELIGIBLE_LOCAL_ONLY:
            outgoing[edge["source"]].append(edge)
    grouped = {}

    def walk(source, current, visited, evidence):
        for edge in outgoing[current]:
            relation = edge["relation"]
            target = edge["target"]
            if relation in FORBIDDEN_HOPS:
                if not evidence and (target in selected or edge.get("target_kind") in {"external", "builtin"}):
                    key = (source, relation, target, "direct")
                    grouped.setdefault(key, set()).add((edge.get("evidence_key", target),))
                continue
            path = evidence + (edge.get("evidence_key", target),)
            terminal = target in selected or edge.get("target_kind") in {"external", "builtin"}
            if terminal:
                via = "collapsed" if evidence else "direct"
                key = (source, relation, target, via)
                grouped.setdefault(key, set()).add(path)
            elif relation in COLLAPSIBLE_RELATIONS and target not in visited:
                walk(source, target, visited | {target}, path)

    for source in sorted(selected):
        walk(source, source, {source}, ())
    rows = []
    for key in sorted(grouped):
        source, relation, target, via = key
        paths = [list(path) for path in sorted(grouped[key])]
        rows.append({"source": source, "relation": relation, "target": target, "via": via,
                     "path_count": len(paths), "evidence_paths": paths,
                     "edge_identity": (source, relation, target, via)})
    return {"nodes": sorted(selected), "edges": rows}


def compare_semantic_rows(expected, actual, canonical=True):
    if not canonical:
        return {"status": "BLOCKED", "missing": [], "extra": [], "reason": "canonical identities unavailable"}
    if any("semantic_identity" not in row for row in [*expected, *actual]):
        return {"status": "BLOCKED", "missing": [], "extra": [], "reason": "exact identity unavailable"}
    expected_keys = Counter(tuple(row["semantic_identity"]) for row in expected)
    actual_keys = Counter(tuple(row["semantic_identity"]) for row in actual)
    missing = sorted((key, count - actual_keys.get(key, 0)) for key, count in expected_keys.items()
                     if count > actual_keys.get(key, 0))
    extra = sorted((key, count - expected_keys.get(key, 0)) for key, count in actual_keys.items()
                   if count > expected_keys.get(key, 0))
    return {"status": "PASS" if not missing and not extra else "FAIL", "missing": missing, "extra": extra}


def rollup_unique_edges(edges, known_files):
    known = set(known_files)
    unique = defaultdict(set)
    blocked_files = set()
    for edge in edges:
        blockers = set(edge.get("blockers", []))
        macro_file = edge.get("macro_invocation_file")
        if "MACRO_FILE_ASSIGNMENT_UNRESOLVED" in blockers and macro_file in known:
            blocked_files.add(macro_file)
        if edge.get("eligibility_status", ELIGIBLE_LOCAL_ONLY) != ELIGIBLE_LOCAL_ONLY:
            continue
        source_file = edge.get("source_file")
        if source_file not in known:
            continue
        identity = edge.get("edge_identity", (edge.get("source"), edge.get("target"), edge.get("relation"), edge.get("via")))
        unique[source_file].add(json.dumps(identity, sort_keys=True, default=str))
    counts = {path: len(unique.get(path, set())) for path in sorted(known) if path in unique}
    return {"counts": counts, "blocked_files": sorted(blocked_files)}


def group_expected_rollup_edges(expected_edges, source_edges, nodes, known_files):
    node_by_label = {row["label"]: row for row in nodes}
    builtin_targets = {row.get("target") for row in source_edges if row.get("target_kind") == "builtin"}
    known = set(known_files)
    file_groups, folder_groups = defaultdict(set), defaultdict(set)
    excluded = []
    for row in expected_edges:
        source = node_by_label.get(row["source"])
        target = node_by_label.get(row["target"])
        reason = None
        if target is None and row["target"] in builtin_targets:
            reason = "BUILTIN_BUCKET_MAPPING_NOT_IMPLEMENTED"
        elif source is None:
            reason = "SOURCE_NODE_MAPPING_MISSING"
        elif target is None:
            reason = "TARGET_NODE_MAPPING_MISSING"
        elif source.get("file") not in known:
            reason = "SOURCE_FILE_MAPPING_MISSING"
        elif target.get("file") not in known:
            reason = "TARGET_FILE_MAPPING_MISSING"
        identity = (row["source"], row["relation"], row["target"], row["via"])
        if reason:
            excluded.append({"source": row["source"], "relation": row["relation"],
                             "target": row["target"], "via": row["via"], "reason": reason})
            continue
        source_file, target_file = source["file"], target["file"]
        file_groups[(source_file, target_file, row["relation"], row["via"])].add(identity)
        source_folder = source_file.rsplit("/", 1)[0] if "/" in source_file else "."
        target_folder = target_file.rsplit("/", 1)[0] if "/" in target_file else "."
        folder_groups[(source_folder, target_folder, row["relation"], row["via"])].add(identity)
    excluded.sort(key=lambda row: (row["source"], row["relation"], row["target"], row["via"], row["reason"]))
    return {"file_groups": file_groups, "folder_groups": folder_groups,
            "excluded_rollup_edges": excluded}


def verify_source_digests(source_files, expected_digests):
    for path, digest in expected_digests.items():
        if path not in source_files or hashlib.sha256(source_files[path]).hexdigest() != digest:
            raise ValueError(f"source pin/digest mismatch: {path}")


def serialize_dataset(value):
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"


def contract_manifest_statuses(contracts):
    return {row["table_name"]: "BLOCKED/NOT_IMPLEMENTED" if row.get("status") == "PROPOSED_NOT_IMPLEMENTED"
            else row.get("status", "BLOCKED/NOT_IMPLEMENTED") for row in contracts}
