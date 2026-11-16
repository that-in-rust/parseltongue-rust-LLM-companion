import hashlib
import json
from collections import Counter


RELATION_KINDS = ("calls", "uses_type", "implements", "reexports", "contains", "derives", "imports", "reads", "generates", "expands")
BUILTIN_PRIMITIVES = {"bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize",
                      "i8", "i16", "i32", "i64", "i128", "isize", "f16", "f32", "f64", "f128"}


def relation_counts(edges):
    counts = Counter(edge.get("relation") for edge in edges)
    return {relation: counts.get(relation, 0) for relation in RELATION_KINDS}


def verify_source_digests(source_files, expected_digests):
    for path, digest in expected_digests.items():
        if path not in source_files or hashlib.sha256(source_files[path]).hexdigest() != digest:
            raise ValueError(f"source pin/digest mismatch: {path}")


def serialize_dataset(value):
    return json.dumps(value, sort_keys=True, indent=2, ensure_ascii=False) + "\n"
