# Parseltongue Diff Visualization System: Unified PRD & Architecture

> **Version**: 1.0.0 (2026-01-23)
> **Status**: IMPLEMENTED - All phases complete
> **Mantra**: "DIFF IS THE PRODUCT"

---

## 1. Executive Summary

### 1.1 Product Vision

Parseltongue transforms from a static code analysis tool into a **change impact analysis platform**. When AI agents edit code, developers need to understand:

1. **What changed?** (entities added, removed, modified, relocated)
2. **What's affected?** (blast radius - dependent code that may need attention)
3. **How significant?** (visualization showing impact magnitude)

### 1.2 Key Value Propositions

| Metric | Value |
|--------|-------|
| Token reduction | 99% (2-5K tokens vs 500K raw) |
| Query speed | 31x faster than grep |
| False positive rate | <1% (key normalization prevents line-shift noise) |

---

## 2. Architecture Overview

### 2.1 System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    PARSELTONGUE SYSTEM                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐    ┌──────────────────┐                  │
│  │ pt01-folder-to-  │    │ pt08-http-code-  │                  │
│  │ cozodb-streamer  │    │ query-server     │                  │
│  │                  │    │                  │                  │
│  │ Ingest codebase  │    │ HTTP REST API    │                  │
│  │ → CozoDB         │    │ 16 endpoints     │                  │
│  └────────┬─────────┘    └────────┬─────────┘                  │
│           │                       │                             │
│           ▼                       ▼                             │
│  ┌────────────────────────────────────────────────┐            │
│  │              parseltongue-core                  │            │
│  │                                                 │            │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────┐ │            │
│  │  │  Storage    │  │  Entities   │  │  Diff   │ │            │
│  │  │  (CozoDB)   │  │  (Types)    │  │  Module │ │            │
│  │  └─────────────┘  └─────────────┘  └─────────┘ │            │
│  └────────────────────────────────────────────────┘            │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  parseltongue (CLI)                       │  │
│  │                                                           │  │
│  │  Commands: pt01, pt08, diff                              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Data Flow

```
[Codebase v1] → pt01 → [base.db] ─┐
                                   ├─→ diff → [DiffResult] → visualization
[Codebase v2] → pt01 → [live.db] ─┘
```

---

## 3. Diff Module Architecture

### 3.1 Core Types (types.rs)

| Type | Purpose |
|------|---------|
| `NormalizedEntityKeyData` | Parsed entity key with stable identity |
| `LineRangeData` | Start/end line range |
| `EntityChangeTypeClassification` | Added/Removed/Modified/Relocated/Unchanged |
| `DiffResultDataPayload` | Complete diff result (summary + changes) |
| `BlastRadiusResultPayload` | Impact analysis result |
| `VisualizationGraphDataPayload` | Three.js-ready visualization data |

### 3.2 Core Traits (traits.rs)

| Trait | Responsibility |
|-------|----------------|
| `KeyNormalizerTrait` | Extract stable identity from entity keys |
| `EntityDifferTrait` | Compare entity/edge sets, produce diff |
| `BlastRadiusCalculatorTrait` | BFS traversal with cycle detection |
| `DiffVisualizationTransformerTrait` | Transform diff → visualization format |

### 3.3 Key Design Decision: ADR-001 (Key Normalization)

**Problem**: Entity keys include line numbers that shift when code moves.

```
Before: rust:fn:helper:__path:20-30
After:  rust:fn:helper:__path:24-34   // 4 lines added above
```

**Solution**: Extract stable identity by stripping line numbers.

```
Full Key:    rust:fn:helper:__path:20-30
Stable ID:   rust:fn:helper:__path
```

**Implementation**: `KeyNormalizerTrait::extract_stable_identity_from_key()`

---

## 4. API Specification

### 4.1 CLI Command

```bash
parseltongue diff --base rocksdb:base.db --live rocksdb:live.db [--json] [--max-hops N]
```

| Argument | Description | Default |
|----------|-------------|---------|
| `--base` | Path to base/before database | Required |
| `--live` | Path to live/after database | Required |
| `--json` | Output as JSON | false (human-readable) |
| `--max-hops` | Blast radius depth | 2 |

### 4.2 HTTP Endpoint

**Route**: `POST /diff-analysis-compare-snapshots`

**Request Body**:
```json
{
  "base_db": "rocksdb:path/to/base.db",
  "live_db": "rocksdb:path/to/live.db"
}
```

**Query Parameters**:
- `max_hops` (optional, default: 2, max: 10)

**Response**:
```json
{
  "diff": {
    "summary": {
      "total_before_count": 100,
      "total_after_count": 105,
      "added_entity_count": 7,
      "removed_entity_count": 2,
      "modified_entity_count": 3,
      "unchanged_entity_count": 93,
      "relocated_entity_count": 0
    },
    "entity_changes": [...],
    "edge_changes": [...]
  },
  "blast_radius": {
    "origin_entity": "...",
    "affected_by_distance": {"1": [...], "2": [...]},
    "total_affected_count": 15,
    "max_depth_reached": 2
  },
  "visualization": {
    "nodes": [...],
    "edges": [...],
    "diff_summary": {...},
    "max_blast_radius_depth": 2
  },
  "token_estimate": 1234
}
```

---

## 5. Implementation Status

### 5.1 Phase Completion

| Phase | Component | Status | Tests |
|-------|-----------|--------|-------|
| 1 | KeyNormalizerTrait | ✅ GREEN | 12 |
| 2 | EntityDifferTrait | ✅ GREEN | 7 |
| 3 | BlastRadiusCalculatorTrait | ✅ GREEN | 7 |
| 4 | DiffVisualizationTransformerTrait | ✅ GREEN | 6 |
| 5 | Performance Tests | ✅ GREEN | 3 |
| 6 | CLI `parseltongue diff` | ✅ GREEN | 5 |
| 7 | HTTP Endpoint | ✅ GREEN | 48 |

**Total Tests**: 244+ passing

### 5.2 File Locations

**Core Diff Module** (`crates/parseltongue-core/src/diff/`):
- `mod.rs` - Module exports
- `types.rs` - Data types
- `traits.rs` - Trait definitions
- `key_normalizer_impl.rs` - Key normalization
- `entity_differ_impl.rs` - Entity/edge diffing
- `blast_radius_calculator_impl.rs` - BFS impact analysis
- `visualization_transformer_impl.rs` - Visualization transform

**CLI** (`crates/parseltongue/src/`):
- `main.rs` - CLI entry point
- `commands/diff_command_execution_module.rs` - Diff command

**HTTP** (`crates/pt08-http-code-query-server/src/http_endpoint_handler_modules/`):
- `diff_analysis_compare_handler.rs` - HTTP endpoint

---

## 6. Naming Convention: 4-Word Rule

All functions follow the pattern: `verb_constraint_target_qualifier()`

**Examples**:
- `extract_stable_identity_from_key()`
- `compute_entity_diff_result()`
- `handle_diff_analysis_compare_snapshots()`

---

## 7. Related Documents

| Document | Location | Purpose |
|----------|----------|---------|
| Data Structures | `docs/prd/04_DATA_STRUCTURES.md` | Type definitions |
| Interface Definitions | `docs/prd/07_RUST_INTERFACE_DEFINITIONS.md` | Trait specs |
| ADR-001 | `docs/prd/ADR_001_KEY_NORMALIZATION.md` | Key normalization decision |
| CLI Specs | `docs/specs/EXECUTABLE_SPECS_diff_command.md` | CLI behavior specs |
| HTTP Specs | `docs/specs/REQ-HTTP-DIFF-ANALYSIS-COMPARE-SNAPSHOTS.md` | Endpoint specs |
| TDD Plan | `docs/tdd/TDD-plan-20260123000800.md` | Test-driven development plan |

---

## 8. Change Log

| Date | Version | Changes |
|------|---------|---------|
| 2026-01-23 | 1.0.0 | Initial unified PRD-ARCH document |
| 2026-01-23 | 1.0.0 | All 7 phases implemented and tested |
