# RESEARCH: v1.7.3 — Serialization Format Analysis for pt02/pt03

**Date**: 2026-02-12
**Context**: Evaluating serialization formats for pt02 (binary snapshot) and pt03 (human-readable export) as Windows-compatible alternatives to pt01's CozoDB/RocksDB pipeline.

---

## Why This Research Exists

pt01 uses CozoDB with RocksDB backend. Six attempts (v1.6.7–v1.7.2) to make persistent storage work on Windows all failed:

| Version | Approach | Result |
|---------|----------|--------|
| v1.6.7 | RocksDB OPTIONS tuning | 75MB write stall (Defender scans SST files) |
| v1.6.8 | Windows chunked batch inserts | No effect |
| v1.6.9 | Sled backend | Abandoned project, data loss |
| v1.7.0 | SQLite backend | 12KB empty database |
| v1.7.1 | Sequential SQLite inserts | Still 12KB empty |
| v1.7.2 | mem→SQLite backup | Requires holding full codebase in RAM during ingestion AND serving |

**v1.7.2 was deleted** because the mem→SQLite approach requires CozoDB's `mem` backend to hold ALL entities + edges in RAM, which crashes on large codebases (>50K entities on 8GB machines).

**New approach**: pt02/pt03 serialize parsed data directly to files. pt08 loads subset of data needed per endpoint.

---

## Question 1: Can We Use Protocol Buffers (Protobuf)?

### Available Rust Crates

| Crate | Maintainer | Purpose |
|-------|-----------|---------|
| `prost` 0.14 | tokio-rs | Protobuf code gen from `.proto` files |
| `prost-build` 0.14 | tokio-rs | Build-time `.proto` compilation |
| `tonic` 0.12+ | tokio-rs | gRPC framework (overkill for file serialization) |

### Verdict: Not Recommended

Protobuf **cannot** use existing `#[derive(Serialize, Deserialize)]` structs. It requires:

| Component | Lines of Code |
|-----------|:------------:|
| `.proto` schema file (redefine all structs) | ~100 |
| `build.rs` configuration | ~20 |
| `From<CodeEntity> for proto::CodeEntity` conversion layer | ~200-400 |
| Serialization/deserialization calls | ~10 |
| **Total extra code** | **~330-530** |

**Why it doesn't fit Parseltongue**:
1. All entity types already derive serde — protobuf can't consume them directly
2. `LanguageSpecificSignature` uses `#[serde(tag = "language")]` tagged enum — complex to map to protobuf oneof
3. Only Rust reads these files (pt08) — cross-language interop is unnecessary
4. Schema evolution (protobuf's strength) can be achieved more cheaply with MessagePack

---

## Question 2: All Viable Serialization Formats

### Serde-Compatible (drop-in, ~10 lines integration)

#### Bincode 1.3.3
- **Crate**: `bincode = "1.3.3"`
- **File size**: ~500KB (17% of JSON)
- **Speed**: Fastest serde-compatible format (~40ns serialize, ~60ns deserialize per struct)
- **Cross-language**: None (Rust-only)
- **Schema evolution**: None — ANY struct field change breaks all existing files
- **WARNING**: Development ceased due to doxxing of maintainers. v3.0.0 on crates.io is a tombstone. Community fork `bincode-next` exists but uncertain future.

#### MessagePack (rmp-serde) — RECOMMENDED
- **Crate**: `rmp-serde = "1.3"`
- **File size**: ~450-500KB (15-17% of JSON)
- **Speed**: ~1.5-2x slower than bincode (still microseconds for 500KB)
- **Cross-language**: Excellent — 50+ languages (Python `msgpack`, JS `msgpack-lite`, Go, Java, etc.)
- **Schema evolution**: Self-describing format. New fields with `#[serde(default)]` don't break old files
- **Status**: Actively maintained, healthy contributor base

#### CBOR (ciborium)
- **Crate**: `ciborium = "0.2"`
- **File size**: ~800KB-1.2MB (27-40% of JSON)
- **Speed**: Slower than bincode/msgpack
- **Cross-language**: Excellent (IETF RFC 8949)
- **Verdict**: No advantage over MessagePack. Larger files, slower.

#### Postcard
- **Crate**: `postcard = "1.1"`
- **File size**: ~350KB (12% of JSON) — smallest serde-compatible format
- **Speed**: ~1.5x slower than bincode
- **Cross-language**: None (Rust-only)
- **Schema evolution**: None
- **Verdict**: Good for embedded/no_std. No benefit over MessagePack for this use case.

#### JSON (serde_json) — RECOMMENDED for human-readable
- **Crate**: `serde_json` (already a dependency)
- **File size**: ~3MB (baseline)
- **Speed**: Slowest (text parsing overhead)
- **Cross-language**: Universal
- **Human readable**: Yes — inspectable with any text editor, `jq`, etc.

#### RON (Rusty Object Notation)
- **Crate**: `ron = "0.11"`
- **File size**: ~3.5-4MB (larger than JSON)
- **Cross-language**: Rust-only
- **PROBLEM**: Known issues with `#[serde(tag = ...)]` — directly affects `LanguageSpecificSignature`
- **Verdict**: Not suitable.

#### Bitcode
- **Crate**: `bitcode = "0.6"`
- **File size**: ~300KB (absolute smallest)
- **PROBLEM**: Serde path is ~10x slower than native `Encode`/`Decode` traits. Would need separate derives.
- **Verdict**: Not worth the complexity.

### Non-Serde (require schema files, conversion layers, 300-550 lines)

| Format | Crate | Integration Lines | Verdict |
|--------|-------|:-----------------:|---------|
| Protobuf (prost) | `prost = "0.14"` | ~330-530 | Not justified — cross-lang unnecessary |
| FlatBuffers | `flatbuffers = "24.12"` | ~300-500 | Zero-copy reads, but heavy integration |
| Cap'n Proto | `capnp = "0.20"` | ~300-500 | Heavy integration, less mature in Rust |

### Alternative Storage (not simple serialization)

#### SQLite (rusqlite) — without CozoDB wrapper
- **Crate**: `rusqlite = "0.32"`
- **File size**: ~1.5-2.5MB
- **Integration**: ~100-150 lines (CREATE TABLE + INSERT + SELECT)
- **Advantage**: Random access queries without loading everything. Atomic writes.
- **Problem**: Nested structs must be stored as JSON blobs. More code than binary formats.

#### Parquet/Arrow (columnar)
- **Crate**: `parquet = "53"` + `serde_arrow = "0.12"`
- **File size**: ~400-600KB (great compression)
- **Problem**: Wrong paradigm for graph data. Individual entity lookup is slow. Tagged enums need flattening.
- **Verdict**: Overkill. Wrong tool for the job.

#### rkyv (zero-copy deserialization)
- **Crate**: `rkyv = "0.8"`
- **Speed**: 21ns deserialization (effectively instant — pointer cast)
- **Integration**: ~50-100 lines. Requires separate `rkyv::Archive/Serialize/Deserialize` derives (not serde).
- **Advantage**: Can combine with `memmap2` for memory-mapped access without loading entire file.
- **Best for**: Future optimization if codebases exceed 50K entities.

---

## Question 3: Memory Crash Risk for Large Codebases

### Per-Entity RAM Usage

| Component | RAM per unit | Notes |
|-----------|:----------:|-------|
| `CodeEntity` | ~1-3 KB | Varies by source code body size |
| `DependencyEdge` | ~200-300 bytes | Two ISGL1 keys + edge type |
| CozoDB `mem` overhead per entity | ~300-500 bytes | BTreeMap internal nodes |

### RAM Projections by Codebase Scale

| Scale | Entities | Edges | Data RAM | CozoDB Overhead | **Total** | 8GB Safe? | 16GB Safe? |
|-------|----------|-------|----------|----------------|-----------|:---------:|:----------:|
| **Parseltongue self** | 1,600 | 10K | ~6 MB | ~5-10 MB | **~15-20 MB** | Yes | Yes |
| **Medium project** | 10,000 | 50K | ~25 MB | ~20-30 MB | **~50-80 MB** | Yes | Yes |
| **Large monorepo** | 50,000 | 100K | ~125 MB | ~50-100 MB | **~200-350 MB** | Tight | Yes |
| **Massive** (Linux kernel) | 200,000 | 500K | ~525 MB | ~200-400 MB | **~800 MB-1.2 GB** | No | Yes |
| **Extreme** | 500,000 | 2M | ~1.5 GB | ~500 MB-1 GB | **~2-3 GB** | No | Tight |

### Additional RAM During Query Execution

Graph algorithms (SCC, PageRank, k-core, Leiden, CK metrics) **double** peak RAM by building `AdjacencyListGraphRepresentation` + intermediate computation state.

**Practical ceilings**:
- **8GB machine**: ~50,000-60,000 entities (with graph algorithms)
- **16GB machine**: ~150,000-200,000 entities (with graph algorithms)
- **Without graph algorithms** (LIGHT endpoints only): 2-3x higher ceilings

### Memory-Mapped Alternatives (Future)

| Approach | Crate | How it works |
|----------|-------|-------------|
| `rkyv` + `memmap2` | `rkyv = "0.8"`, `memmap2` | Serialize to file, memory-map it, access fields without loading everything |
| `mmap-sync` (Cloudflare) | `mmap-sync` | Zero-copy via rkyv + mmap. Supports up to 549 GB. |

These are future optimizations for when codebases exceed 50K entities.

---

## Comprehensive Comparison Table

Scale: 1 = worst, 5 = best.

| Format | File Size | Read Speed | Write Speed | Cross-Lang | Schema Safety | Serde Compat | Human Readable | **Total /35** |
|--------|:---------:|:----------:|:-----------:|:----------:|:------------:|:------------:|:--------------:|:------------:|
| **MessagePack** | 5 | 4 | 4 | 5 | 3 | 5 | 1 | **27** |
| **JSON** | 1 | 2 | 2 | 5 | 3 | 5 | 5 | **23** |
| Bincode 1.3.3 | 4 | 5 | 5 | 1 | 1 | 5 | 1 | **22** |
| SQLite (raw) | 2 | 3 | 3 | 5 | 4 | 3 | 3 | **23** |
| Protobuf | 3 | 3 | 3 | 5 | 5 | 1 | 1 | **21** |
| CBOR | 3 | 3 | 3 | 5 | 3 | 5 | 1 | **23** |
| Postcard | 5 | 4 | 4 | 1 | 1 | 5 | 1 | **21** |
| FlatBuffers | 3 | 5 | 3 | 5 | 4 | 1 | 1 | **22** |
| rkyv | 3 | 5 | 5 | 1 | 1 | 1 | 1 | **17** |
| Parquet | 4 | 3 | 3 | 4 | 3 | 3 | 1 | **21** |

---

## Recommendation: 2-Format Strategy

### Format 1: MessagePack (`.ptgraph`) — Primary Binary Format

**Why MessagePack over bincode:**

1. **Schema evolution**: MessagePack is self-describing. Adding fields with `#[serde(default)]` doesn't break old files. Bincode breaks silently on any struct change.
2. **Actively maintained**: bincode development ceased (maintainers doxxed). `rmp-serde` is healthy.
3. **Cross-language**: If someone wants to read `.ptgraph` from Python/JS, MessagePack has mature libraries everywhere.
4. **Comparable performance**: Within 5% file size of bincode. Speed difference is microseconds on 500KB files.

**Integration**:
```rust
// Cargo.toml: rmp-serde = "1.3"

// Serialize (~5 lines)
let bytes = rmp_serde::to_vec(&snapshot)?;
std::fs::write("output.ptgraph", &bytes)?;

// Deserialize (~5 lines)
let bytes = std::fs::read("output.ptgraph")?;
let snapshot: GraphSnapshot = rmp_serde::from_slice(&bytes)?;
```

### Format 2: JSON (`.json`) — Human-Readable Export

Already a dependency. Zero new code needed beyond `serde_json::to_string_pretty()`.

### Future Format 3: rkyv + memmap2 — For Codebases >50K Entities

Add `rkyv` derives alongside serde derives on all entity types. Enables memory-mapped zero-copy access where pt08 serves endpoints without loading everything into RAM. Not needed for v1.7.3.

---

## What NOT to Use (and Why)

| Format | Reason to Reject |
|--------|-----------------|
| Protobuf | Requires .proto schemas + conversion layer (+300-550 lines). Cross-lang unnecessary. |
| FlatBuffers | Requires .fbs schemas + flatc codegen. "Un-Rust-y." |
| Cap'n Proto | Heavy integration. Less mature in Rust. |
| RON | Known issues with `#[serde(tag)]` — breaks LanguageSpecificSignature |
| TOML | Cannot represent nested Vec<CodeEntity> with enum variants |
| Bitcode | Serde path 10x slower than native. Needs separate derives. |
| Bincode | Development ceased. Zero schema evolution. |
| Parquet | Wrong paradigm for graph data. Individual entity lookup slow. |

---

## References

- [rust_serialization_benchmark](https://github.com/djkoloski/rust_serialization_benchmark) — Comprehensive benchmarks (2026-01-12)
- [Curtis Lowder, "Sufficient Serialization" (2025)](https://curtislowder.com/blog/2025-08-10-sufficient-serialization/)
- [rkyv documentation](https://rkyv.org/)
- [Cloudflare mmap-sync](https://github.com/cloudflare/mmap-sync)
- [bincode crates.io](https://crates.io/crates/bincode) — Maintenance status
- [rmp-serde crates.io](https://crates.io/crates/rmp-serde)
