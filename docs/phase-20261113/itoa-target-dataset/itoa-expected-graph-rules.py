import hashlib
import json
from collections import Counter


RELATION_KINDS = ("calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads")
ELIGIBLE_LOCAL_ONLY = "ELIGIBLE_LOCAL_ONLY_DIAGNOSTIC"
CANDIDATE_STATUS = "LLM_PARSED_DRAFT"
BUILTIN_PRIMITIVES = {"bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize",
                      "i8", "i16", "i32", "i64", "i128", "isize", "f16", "f32", "f64", "f128"}


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


def verify_source_digests(source_files, expected_digests):
    for path, digest in expected_digests.items():
        if path not in source_files or hashlib.sha256(source_files[path]).hexdigest() != digest:
            raise ValueError(f"source pin/digest mismatch: {path}")


def serialize_dataset(value):
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
