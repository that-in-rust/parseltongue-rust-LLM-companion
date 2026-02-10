# PRD: Ingestion Coverage Report (v1.6.1)

**Feature**: Code Graph Coverage Report
**Version**: v1.6.1
**Status**: Draft
**Date**: 2026-02-08
**Author**: Parseltongue Team

---

## Table of Contents

1. [Problem Statement](#1-problem-statement)
2. [Why This Matters for LLM Agents](#2-why-this-matters-for-llm-agents)
3. [User Stories / Jobs-To-Be-Done](#3-user-stories--jobs-to-be-done)
4. [Prior Art and Research](#4-prior-art-and-research)
5. [Detailed Requirements](#5-detailed-requirements)
6. [Data Model](#6-data-model)
7. [API Design](#7-api-design)
8. [Integration with Existing Pipeline](#8-integration-with-existing-pipeline)
9. [Edge Cases](#9-edge-cases)
10. [Testing Strategy](#10-testing-strategy)
11. [Implementation Phases](#11-implementation-phases)
12. [Success Metrics](#12-success-metrics)
13. [Open Questions](#13-open-questions)

---

## 1. Problem Statement

### The Gap: "How Much of My Codebase Does Parseltongue Actually Understand?"

Parseltongue ingests codebases using tree-sitter parsers, extracts code entities (functions, classes, structs, modules) and dependency edges (Calls, Uses, Implements), and stores them in CozoDB. However, there is currently **no visibility into ingestion completeness**. Users and LLM agents have no way to know:

- What percentage of files were successfully parsed and entity-extracted?
- Which folders have high vs. low coverage?
- How many files were silently skipped (binary, unsupported, too large, parse errors)?
- How many total entities were extracted per folder, and at what density?
- Are there entire subtrees of the codebase that are invisible to the graph?

**This is not test coverage.** This is *parse coverage* -- a fundamentally different metric that measures what percentage of the codebase is represented in the code intelligence graph.

### Current State

Today, after `pt01-folder-to-cozodb-streamer` finishes, the user sees a summary like:

```
Parallel Streaming Summary:
Total files found: 847
Files processed: 312
Entities created: 2,841 (CODE only)
  - CODE entities: 2,841
  - TEST entities: 109 (excluded for optimal LLM context)
Errors encountered: 3
Duration: 1.24s
```

This is a flat, global summary. It tells you nothing about *where* coverage gaps exist. A codebase could have 95% coverage in `src/core/` but 0% in `src/generated/` or `vendor/` -- and the aggregate number hides this completely.

### The "Warm Blanket" Problem

As research from Codecov and SonarQube has shown, aggregate coverage metrics are misleading at scale. A fintech platform might show 86% overall coverage, yet critical paths remain blind spots. Two services each reporting 90% coverage might share a checkout flow at only 22% coverage. The same principle applies to parse coverage: **aggregate numbers mask folder-level gaps**.

### Real-World Scenario

An LLM agent queries Parseltongue's `/blast-radius-impact-analysis` for a function in `src/billing/`. If Parseltongue only parsed 40% of files in `src/billing/` (because the rest are generated protobuf files or vendored code), the blast radius result is fundamentally incomplete. Without a coverage report, the agent has no way to assess confidence in the result. It might plan a refactoring based on incomplete dependency data, leading to missed breakages.

---

## 2. Why This Matters for LLM Agents

### 2.1 Token Budget Optimization

Parseltongue's core value proposition is 99% token reduction (2-5K tokens vs 500K raw dumps). But this metric only holds for the *covered* portion of the codebase. An LLM agent using `/smart-context-token-budget` needs to know if the context it received represents 95% or 45% of the relevant code in a given folder. Coverage data enables the agent to decide:

- "Coverage is 98% in this module. I can trust the dependency graph."
- "Coverage is 35% in this module. I should request raw file contents as supplementary context."

### 2.2 Hallucination Prevention

Research on LLM code hallucinations (CodeHalu framework, arXiv:2409.20550) identifies four major categories: Mapping, Naming, Resource, and Logical hallucinations. When an LLM agent believes it has complete knowledge of a module's dependencies but is actually missing 60% of the files, it is prone to:

- **Resource Hallucination**: Referencing functions/types that exist in unparsed files as if they don't exist.
- **Mapping Hallucination**: Incorrectly mapping call relationships because intermediate call sites are in unparsed files.
- **Logical Hallucination**: Making incorrect refactoring suggestions because the dependency graph is incomplete.

A coverage report gives the LLM agent a **calibrated confidence signal**. The agent can annotate its responses with confidence levels derived from coverage data.

### 2.3 Confidence Scoring for Agent Chains

In multi-agent workflows (e.g., planning agent -> coding agent -> review agent), the planning agent needs to assess whether Parseltongue's graph is trustworthy for the target area. A coverage report enables:

```
[Planning Agent] Checking coverage for src/payments/...
  - 47 files found, 45 parsed, 2 skipped (binary)
  - Coverage: 95.7%
  - Entity density: 12.3 entities/file

[Planning Agent] DECISION: High-confidence graph. Proceeding with graph-based analysis.
```

Without this, agents must either blindly trust the graph (risky) or fall back to expensive full-file reads (defeats the purpose of Parseltongue).

### 2.4 Grounding and Retrieval-Augmented Generation

Token-level hallucination detection systems like HaluGate (vLLM, 2025) demonstrate that grounding LLM responses in verified data reduces hallucination rates significantly. Coverage metadata is a form of **meta-grounding** -- it tells the LLM not just "here are the entities" but "here is how complete this picture is." This is analogous to how Sourcegraph's SCIP/LSIF system falls back to search-based navigation when precise code intelligence indices are incomplete.

---

## 3. User Stories / Jobs-To-Be-Done

### JTBD-1: Developer Assessing Ingestion Quality

**When** I run `pt01-folder-to-cozodb-streamer` on my monorepo,
**I want to** see a per-folder breakdown of what was parsed vs. skipped,
**So that** I can identify which parts of my codebase are invisible to the graph and take corrective action (e.g., adding language support, excluding generated code).

### JTBD-2: LLM Agent Assessing Context Reliability

**When** an LLM agent queries `/blast-radius-impact-analysis` or `/smart-context-token-budget`,
**I want to** also check `/ingestion-coverage-folder-report` for the relevant folders,
**So that** I can include a confidence qualifier in my response (e.g., "Based on 94% code graph coverage of this module...").

### JTBD-3: DevOps Engineer Monitoring Ingestion Health

**When** I set up Parseltongue in CI/CD to ingest our codebase nightly,
**I want to** track coverage trends over time and set thresholds,
**So that** I can detect regressions (e.g., a new language or folder being added that Parseltongue doesn't parse).

### JTBD-4: Developer Debugging Missing Entities

**When** I search for a function I know exists but Parseltongue doesn't find it,
**I want to** check coverage for that file's folder,
**So that** I can determine if the file was skipped, failed to parse, or uses an unsupported language.

### JTBD-5: Platform Team Evaluating Language Support

**When** our codebase includes Kotlin, Scala, or SQL files alongside the 12 supported languages,
**I want to** see how many files are in unsupported languages,
**So that** I can prioritize language support requests or exclude those folders.

---

## 4. Prior Art and Research

### 4.1 SonarQube Quality Gates

SonarQube defines "quality gates" with thresholds for new code coverage, defect density, and vulnerability exposure. Changes must meet criteria before integration. Parseltongue can adopt a similar pattern: define minimum ingestion coverage thresholds per project. SonarQube also provides granular coverage reports that break down by lines and files -- the per-folder breakdown in this PRD mirrors that approach for parse coverage.

**Source**: [SonarQube Code Coverage (Sonar)](https://www.sonarsource.com/blog/sonarqube-code-coverage/)

### 4.2 Codecov Sunburst Visualization

Codecov's coverage sunburst is an interactive graph where the size of each slice represents tracked lines and color indicates coverage level. The hierarchical folder-based approach in this PRD draws inspiration from this model -- each folder is a "slice" with its own coverage percentage.

**Source**: [Codecov Graphs](https://docs.codecov.com/docs/graphs)

### 4.3 Sourcegraph SCIP/LSIF Indexing Completeness

Sourcegraph's precise code navigation relies on SCIP indices. When an index is not found for a file, Sourcegraph falls back to search-based navigation. This fallback behavior is analogous to what an LLM agent should do when Parseltongue's coverage is low for a folder -- fall back to raw file reads. Sourcegraph's indexing completeness is tracked per-repository and per-commit.

**Source**: [Sourcegraph Precise Code Navigation](https://sourcegraph.com/docs/code-search/code-navigation/precise_code_navigation)

### 4.4 CodeScene Temporal Analysis

CodeScene augments static analysis with temporal data, prioritizing based on business and delivery impact rather than just code-level findings. The coverage report in this PRD similarly provides actionable context -- not just "this folder has low coverage" but also entity density, skipped file reasons, and language breakdown.

**Source**: [CodeScene Plugin System](https://codescene.io/docs/integrations/plugins/codescene-plugins.html)

### 4.5 Tree-sitter Error Recovery Limitations

Tree-sitter provides error recovery by determining where errors start and end, returning a working syntax tree up to that point. However, `tree-sitter parse` does not have built-in aggregate coverage reporting. ERROR nodes can be present without surfacing to the user (GitHub Issue #4049). This means Parseltongue must track parse success/failure at the application level, not rely on tree-sitter to report it.

**Source**: [Tree-sitter Error Recovery (GitHub)](https://github.com/tree-sitter/tree-sitter/issues/224)

### 4.6 Completeness in Static Analysis (Academic)

Monniaux (2022) distinguishes between *soundness* (no false negatives) and *completeness* (ability to infer all true properties) in abstract interpretation. For Parseltongue's use case, completeness means: "Can we extract all entities and dependencies that actually exist?" A coverage report quantifies the gap between the theoretical complete graph and what was actually extracted.

**Source**: [Completeness in Static Analysis (arXiv:2211.09572)](https://arxiv.org/abs/2211.09572)

### 4.7 LLM Hallucination in Code Generation

Research shows that LLMs produce high-confidence hallucinations where entropy-based detection fails. For code graph applications, this means an LLM might confidently state "function X has no callers" when in reality the callers are in unparsed files. Coverage metadata enables calibration-based confidence scoring, addressing the blind spot that entropy measures miss.

**Source**: [LLM Hallucinations in Code Generation (arXiv:2409.20550)](https://arxiv.org/pdf/2409.20550)

---

## 5. Detailed Requirements

### 5.1 What to Track Per File

| Metric | Description | Type |
|--------|-------------|------|
| `file_path` | Relative path from project root | String |
| `language` | Detected language or "unsupported" | String |
| `status` | `parsed`, `skipped`, `failed`, `binary`, `too_large`, `unsupported_language`, `excluded` | Enum |
| `entities_extracted` | Number of CODE entities extracted from this file | Integer |
| `edges_extracted` | Number of dependency edges extracted from this file | Integer |
| `file_size_bytes` | File size in bytes | Integer |
| `error_message` | Parse error details if status is `failed` | String? |
| `parse_duration_us` | Microseconds spent parsing this file | Integer |

### 5.2 What to Aggregate Per Folder

| Metric | Description | Formula |
|--------|-------------|---------|
| `folder_path` | Relative folder path (level 1 or level 2) | - |
| `total_files` | All files found in this folder (recursive) | count(*) |
| `parsed_files` | Files that produced at least 1 entity | count(status == 'parsed') |
| `skipped_files` | Files excluded by pattern | count(status == 'excluded') |
| `failed_files` | Files that tree-sitter could not parse | count(status == 'failed') |
| `binary_files` | Files detected as binary (non-UTF8) | count(status == 'binary') |
| `unsupported_files` | Files with unrecognized extensions | count(status == 'unsupported_language') |
| `too_large_files` | Files exceeding max_file_size | count(status == 'too_large') |
| `total_entities` | Entities extracted from this folder | sum(entities_extracted) |
| `total_edges` | Edges extracted from this folder | sum(edges_extracted) |
| `coverage_pct` | Parse coverage percentage | parsed_files / (total_files - excluded_files) * 100 |
| `entity_density` | Entities per parsed file | total_entities / parsed_files |
| `languages` | Languages detected in this folder | distinct(language) |

### 5.3 Coverage Percentage Calculation

Coverage percentage should exclude files that were *intentionally* excluded (node_modules, .git, target/, etc.) from the denominator, because those are expected to be absent. The formula:

```
coverage_pct = parsed_files / (total_files - excluded_files) * 100
```

Where:
- `total_files` = all files found during directory walk
- `excluded_files` = files matching exclude patterns
- `parsed_files` = files where tree-sitter produced at least one entity

This ensures that a project with `node_modules/` containing 50,000 files does not report 0.5% coverage when the actual codebase is fully parsed.

### 5.4 Folder Depth Levels

Track coverage at two levels:

- **Level 1**: Direct children of the project root (e.g., `src/`, `crates/`, `lib/`, `tests/`)
- **Level 2**: Grandchildren of the project root (e.g., `src/core/`, `src/api/`, `crates/parseltongue-core/`)

Deeper levels are not tracked individually but roll up to their level-2 ancestor. This prevents excessive granularity while still providing actionable folder-level insights.

### 5.5 Global Summary

In addition to per-folder data, provide a global summary:

| Metric | Description |
|--------|-------------|
| `total_files_discovered` | All files found during walk |
| `total_files_parsed` | Files that produced entities |
| `total_files_excluded` | Files matching exclude patterns |
| `total_files_failed` | Files that failed to parse |
| `total_files_binary` | Binary files detected |
| `total_files_unsupported` | Unsupported language files |
| `total_files_too_large` | Files exceeding size limit |
| `global_coverage_pct` | Overall parse coverage |
| `total_entities_extracted` | Total CODE entities |
| `total_edges_extracted` | Total dependency edges |
| `ingestion_duration_ms` | Total ingestion time |
| `languages_detected` | List of all detected languages |
| `supported_languages` | List of languages Parseltongue supports |
| `unsupported_extensions` | List of file extensions not supported |

---

## 6. Data Model

### 6.1 New CozoDB Relation: `IngestionCoverageFiles`

Stores per-file ingestion results. Created during `pt01` ingestion and queryable by `pt08` HTTP server.

```
:create IngestionCoverageFiles {
    file_path: String
    =>
    folder_level1: String,
    folder_level2: String?,
    language: String,
    status: String,
    entities_extracted: Int,
    edges_extracted: Int,
    file_size_bytes: Int,
    error_message: String?,
    parse_duration_us: Int,
    ingestion_run_id: String
}
```

**Key**: `file_path` (unique per file per ingestion run)

**Design Note**: `ingestion_run_id` is a UUID or timestamp string that groups all files from a single `pt01` run. This enables tracking coverage across multiple ingestion runs and supporting incremental re-ingestion.

### 6.2 New CozoDB Relation: `IngestionCoverageFolders`

Pre-aggregated folder-level statistics. Computed at the end of ingestion and stored for fast HTTP queries.

```
:create IngestionCoverageFolders {
    folder_path: String,
    folder_depth: Int
    =>
    total_files: Int,
    parsed_files: Int,
    skipped_files: Int,
    failed_files: Int,
    binary_files: Int,
    unsupported_files: Int,
    too_large_files: Int,
    total_entities: Int,
    total_edges: Int,
    coverage_pct: Float,
    entity_density: Float,
    languages_json: String,
    ingestion_run_id: String
}
```

**Key**: Composite (`folder_path`, `folder_depth`)

### 6.3 New CozoDB Relation: `IngestionCoverageGlobal`

Single-row global summary for fast API response.

```
:create IngestionCoverageGlobal {
    ingestion_run_id: String
    =>
    total_files_discovered: Int,
    total_files_parsed: Int,
    total_files_excluded: Int,
    total_files_failed: Int,
    total_files_binary: Int,
    total_files_unsupported: Int,
    total_files_too_large: Int,
    global_coverage_pct: Float,
    total_entities_extracted: Int,
    total_edges_extracted: Int,
    ingestion_duration_ms: Int,
    languages_detected_json: String,
    unsupported_extensions_json: String,
    ingestion_timestamp: String
}
```

**Key**: `ingestion_run_id`

---

## 7. API Design

### 7.1 Primary Endpoint

**Endpoint**: `GET /ingestion-coverage-folder-report`

**4-Word Name**: `ingestion-coverage-folder-report`

**Handler Function**: `handle_ingestion_coverage_folder_report` (4 words)

**Handler File**: `ingestion_coverage_folder_handler.rs` (4 words)

### 7.2 Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `depth` | int | 2 | Folder depth (1 or 2) |
| `min_coverage` | float | 0.0 | Filter: show only folders below this coverage % |
| `sort` | string | "coverage_asc" | Sort order: `coverage_asc`, `coverage_desc`, `files_desc`, `entities_desc` |
| `folder` | string | (all) | Filter to specific folder prefix |

### 7.3 Example API Response

#### Request: `GET /ingestion-coverage-folder-report?depth=2&sort=coverage_asc`

```json
{
  "success": true,
  "endpoint": "/ingestion-coverage-folder-report",
  "data": {
    "global_summary": {
      "total_files_discovered": 847,
      "total_files_parsed": 312,
      "total_files_excluded": 489,
      "total_files_failed": 3,
      "total_files_binary": 28,
      "total_files_unsupported": 15,
      "total_files_too_large": 0,
      "global_coverage_pct": 87.15,
      "total_entities_extracted": 2841,
      "total_edges_extracted": 1203,
      "ingestion_duration_ms": 1240,
      "languages_detected": ["rust", "python", "javascript", "typescript", "go"],
      "unsupported_extensions": [".proto", ".graphql", ".yaml"],
      "ingestion_run_id": "20260208-143052",
      "ingestion_timestamp": "2026-02-08T14:30:52Z"
    },
    "folders": [
      {
        "folder_path": "src/generated/",
        "folder_depth": 2,
        "total_files": 45,
        "parsed_files": 12,
        "skipped_files": 0,
        "failed_files": 8,
        "binary_files": 0,
        "unsupported_files": 25,
        "too_large_files": 0,
        "total_entities": 34,
        "total_edges": 8,
        "coverage_pct": 26.67,
        "entity_density": 2.83,
        "languages": ["python", "javascript"],
        "coverage_assessment": "low"
      },
      {
        "folder_path": "src/api/",
        "folder_depth": 2,
        "total_files": 23,
        "parsed_files": 21,
        "skipped_files": 0,
        "failed_files": 1,
        "binary_files": 1,
        "unsupported_files": 0,
        "too_large_files": 0,
        "total_entities": 187,
        "total_edges": 92,
        "coverage_pct": 91.30,
        "entity_density": 8.90,
        "languages": ["typescript"],
        "coverage_assessment": "high"
      },
      {
        "folder_path": "crates/parseltongue-core/",
        "folder_depth": 2,
        "total_files": 31,
        "parsed_files": 31,
        "skipped_files": 0,
        "failed_files": 0,
        "binary_files": 0,
        "unsupported_files": 0,
        "too_large_files": 0,
        "total_entities": 412,
        "total_edges": 203,
        "coverage_pct": 100.00,
        "entity_density": 13.29,
        "languages": ["rust"],
        "coverage_assessment": "complete"
      }
    ],
    "coverage_assessment_thresholds": {
      "complete": "100%",
      "high": ">= 80%",
      "medium": ">= 50%",
      "low": "< 50%",
      "none": "0%"
    }
  },
  "tokens": 320,
  "llm_guidance": {
    "interpretation": "Folders with coverage_assessment 'low' or 'none' have significant blind spots. Dependency graph queries for entities in these folders may be incomplete. Consider supplementing with raw file reads.",
    "confidence_formula": "result_confidence = base_confidence * (coverage_pct / 100)",
    "recommended_action_by_coverage": {
      "complete": "Trust graph fully. No supplementary context needed.",
      "high": "Trust graph with minor caveat. Mention coverage in response.",
      "medium": "Use graph as starting point. Supplement with file reads for critical paths.",
      "low": "Do not rely on graph alone. Request raw file contents.",
      "none": "Graph has no data for this area. Use file-based analysis only."
    }
  }
}
```

### 7.4 Secondary Endpoint: Per-File Detail

**Endpoint**: `GET /ingestion-coverage-file-detail`

**4-Word Name**: `ingestion-coverage-file-detail`

**Query Parameters**:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `folder` | string | required | Folder prefix to list files for |
| `status` | string | (all) | Filter by status: `parsed`, `failed`, `binary`, `unsupported_language`, `too_large` |
| `limit` | int | 100 | Maximum files to return |

#### Example Request: `GET /ingestion-coverage-file-detail?folder=src/generated/&status=failed`

```json
{
  "success": true,
  "endpoint": "/ingestion-coverage-file-detail",
  "data": {
    "folder": "src/generated/",
    "files": [
      {
        "file_path": "src/generated/api_client.py",
        "language": "python",
        "status": "failed",
        "entities_extracted": 0,
        "edges_extracted": 0,
        "file_size_bytes": 45023,
        "error_message": "tree-sitter parse error: unexpected token at line 1203",
        "parse_duration_us": 342
      },
      {
        "file_path": "src/generated/models_v2.py",
        "language": "python",
        "status": "failed",
        "entities_extracted": 0,
        "edges_extracted": 0,
        "file_size_bytes": 128400,
        "error_message": "tree-sitter parse error: maximum recursion depth exceeded",
        "parse_duration_us": 5201
      }
    ],
    "total_matching": 8,
    "returned": 2
  },
  "tokens": 85
}
```

---

## 8. Integration with Existing Pipeline

### 8.1 Changes to `pt01-folder-to-cozodb-streamer`

#### 8.1.1 Schema Creation

During `FileStreamerImpl::new()`, after `db.create_schema()` and `db.create_dependency_edges_schema()`, add:

```rust
db.create_ingestion_coverage_schema().await?;
```

This creates all three new relations: `IngestionCoverageFiles`, `IngestionCoverageFolders`, `IngestionCoverageGlobal`.

#### 8.1.2 Per-File Tracking in `stream_file()`

The existing `stream_file()` method in `streamer.rs` already tracks success/failure and entities created. The modification is to **also** record each file's result into `IngestionCoverageFiles`:

```rust
// After processing each file (success or failure):
let coverage_record = FileCoverageRecord {
    file_path: relative_path_from_root,
    folder_level1: extract_level1_folder(&relative_path),
    folder_level2: extract_level2_folder(&relative_path),
    language: detected_language_or_unsupported,
    status: determine_file_status(parse_result, file_type),
    entities_extracted: entities_created,
    edges_extracted: edges_count,
    file_size_bytes: file_metadata.len(),
    error_message: error_if_any,
    parse_duration_us: parse_timer.elapsed_micros(),
    ingestion_run_id: self.run_id.clone(),
};
```

#### 8.1.3 Pre-Classification of All Files

Currently, `should_process_file()` silently skips non-matching files. For coverage tracking, the directory walk must record ALL files, not just processable ones. The flow becomes:

```
For each file in WalkDir:
  1. Record file exists (total_files++)
  2. Check exclude patterns -> if excluded, record status="excluded"
  3. Check if binary (non-UTF8 read fails) -> if binary, record status="binary"
  4. Check file size limit -> if too large, record status="too_large"
  5. Detect language -> if unsupported, record status="unsupported_language"
  6. Attempt parse -> if parse fails, record status="failed" with error
  7. Parse succeeds -> record status="parsed" with entity/edge counts
```

#### 8.1.4 Parallel Pipeline Integration (`stream_directory_with_parallel_rayon`)

The parallel pipeline in `stream_directory_with_parallel_rayon()` collects `Vec<Result<(FileResult, Vec<CodeEntity>, Vec<DependencyEdge>)>>`. The coverage record should be added to this tuple:

```rust
// Return type becomes:
Result<(FileResult, Vec<CodeEntity>, Vec<DependencyEdge>, FileCoverageRecord)>
```

After parallel processing, all `FileCoverageRecord` entries are batch-inserted into `IngestionCoverageFiles`.

#### 8.1.5 Post-Ingestion Aggregation

After all files are processed, compute folder-level and global aggregates using CozoDB Datalog queries:

```datalog
# Aggregate per folder (level 1)
?[folder, total, parsed, failed, binary, unsupported, too_large, entities, edges] :=
    *IngestionCoverageFiles{folder_level1: folder, status, entities_extracted, edges_extracted},
    total = count(file_path),
    parsed = count(file_path), status == 'parsed',
    ...
```

Or compute in Rust and batch-insert into `IngestionCoverageFolders` and `IngestionCoverageGlobal`.

**Recommended approach**: Compute in Rust for clarity and insert as batch. CozoDB aggregation queries with conditional counts are complex in Datalog.

### 8.2 Changes to `pt08-http-code-query-server`

#### 8.2.1 New Handler Module

Add `ingestion_coverage_folder_handler.rs` to `http_endpoint_handler_modules/`.

#### 8.2.2 Route Registration

In `route_definition_builder_module.rs`:

```rust
.route(
    "/ingestion-coverage-folder-report",
    get(ingestion_coverage_folder_handler::handle_ingestion_coverage_folder_report)
)
.route(
    "/ingestion-coverage-file-detail",
    get(ingestion_coverage_file_detail_handler::handle_ingestion_coverage_file_detail)
)
```

#### 8.2.3 Query Implementation

The handler queries the pre-aggregated `IngestionCoverageFolders` table (fast, <10ms) rather than computing aggregates on the fly from `IngestionCoverageFiles` (slow for large codebases).

### 8.3 Changes to `parseltongue-core`

#### 8.3.1 New Storage Methods

Add to `CozoDbStorage`:

```rust
/// Create ingestion coverage schema
///
/// # 4-Word Name: create_ingestion_coverage_schema
pub async fn create_ingestion_coverage_schema(&self) -> Result<()>;

/// Insert file coverage record
///
/// # 4-Word Name: insert_file_coverage_record
pub async fn insert_file_coverage_record(&self, record: &FileCoverageRecord) -> Result<()>;

/// Insert file coverage records in batch
///
/// # 4-Word Name: insert_coverage_records_batch
pub async fn insert_coverage_records_batch(&self, records: &[FileCoverageRecord]) -> Result<()>;

/// Insert folder coverage aggregate
///
/// # 4-Word Name: insert_folder_coverage_aggregate
pub async fn insert_folder_coverage_aggregate(&self, aggregate: &FolderCoverageAggregate) -> Result<()>;

/// Insert global coverage summary
///
/// # 4-Word Name: insert_global_coverage_summary
pub async fn insert_global_coverage_summary(&self, summary: &GlobalCoverageSummary) -> Result<()>;

/// Query folder coverage aggregates
///
/// # 4-Word Name: query_folder_coverage_aggregates
pub async fn query_folder_coverage_aggregates(
    &self,
    depth: Option<u32>,
    min_coverage: Option<f64>,
    folder_prefix: Option<&str>,
) -> Result<Vec<FolderCoverageAggregate>>;

/// Query file coverage records for a folder
///
/// # 4-Word Name: query_file_coverage_records
pub async fn query_file_coverage_records(
    &self,
    folder: &str,
    status_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<FileCoverageRecord>>;

/// Query global coverage summary
///
/// # 4-Word Name: query_global_coverage_summary
pub async fn query_global_coverage_summary(&self) -> Result<Option<GlobalCoverageSummary>>;
```

#### 8.3.2 New Domain Types

Add to a new file `parseltongue-core/src/coverage_types.rs` (or extend `entities.rs`):

```rust
/// File ingestion status
///
/// # 4-Word Name: FileIngestionStatusEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileIngestionStatus {
    Parsed,
    Failed,
    Binary,
    TooLarge,
    UnsupportedLanguage,
    Excluded,
}

/// Per-file coverage record
///
/// # 4-Word Name: FileCoverageRecordStruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverageRecord {
    pub file_path: String,
    pub folder_level1: String,
    pub folder_level2: Option<String>,
    pub language: String,
    pub status: FileIngestionStatus,
    pub entities_extracted: usize,
    pub edges_extracted: usize,
    pub file_size_bytes: u64,
    pub error_message: Option<String>,
    pub parse_duration_us: u64,
    pub ingestion_run_id: String,
}

/// Folder-level coverage aggregate
///
/// # 4-Word Name: FolderCoverageAggregateStruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCoverageAggregate {
    pub folder_path: String,
    pub folder_depth: u32,
    pub total_files: usize,
    pub parsed_files: usize,
    pub skipped_files: usize,
    pub failed_files: usize,
    pub binary_files: usize,
    pub unsupported_files: usize,
    pub too_large_files: usize,
    pub total_entities: usize,
    pub total_edges: usize,
    pub coverage_pct: f64,
    pub entity_density: f64,
    pub languages: Vec<String>,
    pub ingestion_run_id: String,
}

/// Coverage assessment level
///
/// # 4-Word Name: CoverageAssessmentLevelEnum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoverageAssessmentLevel {
    Complete, // 100%
    High,     // >= 80%
    Medium,   // >= 50%
    Low,      // < 50%
    None,     // 0%
}

impl CoverageAssessmentLevel {
    pub fn from_percentage(pct: f64) -> Self {
        match pct {
            p if p >= 100.0 => Self::Complete,
            p if p >= 80.0 => Self::High,
            p if p >= 50.0 => Self::Medium,
            p if p > 0.0 => Self::Low,
            _ => Self::None,
        }
    }
}

/// Global ingestion coverage summary
///
/// # 4-Word Name: GlobalCoverageSummaryStruct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalCoverageSummary {
    pub ingestion_run_id: String,
    pub total_files_discovered: usize,
    pub total_files_parsed: usize,
    pub total_files_excluded: usize,
    pub total_files_failed: usize,
    pub total_files_binary: usize,
    pub total_files_unsupported: usize,
    pub total_files_too_large: usize,
    pub global_coverage_pct: f64,
    pub total_entities_extracted: usize,
    pub total_edges_extracted: usize,
    pub ingestion_duration_ms: u64,
    pub languages_detected: Vec<String>,
    pub unsupported_extensions: Vec<String>,
    pub ingestion_timestamp: String,
}
```

---

## 9. Edge Cases

### 9.1 Binary Files

**Detection**: Attempt `read_to_string()` on the file. If it fails with a UTF-8 error, classify as `binary`. This matches the current behavior in `read_file_content()` which returns a `FileSystemError` for non-text files.

**Common examples**: `.png`, `.jpg`, `.wasm`, `.so`, `.dll`, `.exe`, `.zip`, `.tar.gz`, compiled `.pyc` files, `.class` files.

**Impact on coverage**: Binary files should be counted in `total_files` but excluded from the coverage denominator (they were never expected to be parseable). The adjusted formula:

```
coverage_pct = parsed_files / (total_files - excluded_files - binary_files) * 100
```

### 9.2 Generated Code

**Detection heuristics** (optional, future enhancement):
- File contains `// Code generated by` or `# Auto-generated` in first 5 lines
- File is in a folder named `generated/`, `gen/`, `proto/`, `__generated__/`
- File has a `.generated.ts` or `.g.cs` extension

**Current approach**: Do not auto-detect generated code. Users can exclude generated folders using `-e generated` flag. Coverage report shows these folders with low entity density as a signal.

### 9.3 Vendored Dependencies

**Examples**: `vendor/`, `third_party/`, `external/`, `deps/`

**Current handling**: The exclude patterns in `cli.rs` already include `node_modules` and `target`. Users can add vendor directories with `-e vendor`.

**Coverage report behavior**: If vendored directories are not excluded, they appear with their actual coverage. This is intentional -- it gives visibility into "we found 5,000 files in `vendor/` that weren't excluded."

### 9.4 node_modules and .git

**Already handled**: The `exclude_patterns` in `StreamerConfig::default()` include `target` and `node_modules`. The `is_under_git_subdirectory()` method skips nested `.git` directories.

**Coverage report behavior**: Files in excluded directories get status `excluded` and are subtracted from the coverage denominator.

### 9.5 Symlinks

**Current handling**: `WalkDir::new().follow_links(false)` is already used. Symlinked files are not followed. They should be recorded as `excluded` with a note.

### 9.6 Empty Files

**Handling**: A 0-byte file should be classified as `parsed` with 0 entities. Tree-sitter can parse empty files without error. This is not a coverage gap.

### 9.7 Mixed-Language Files

**Example**: `.html` files containing `<script>` tags, `.vue` single-file components.

**Current handling**: Parseltongue detects language by file extension only. A `.html` file is `unsupported_language`. A `.vue` file is also `unsupported_language`. This is a known limitation. The coverage report accurately reflects this gap.

### 9.8 Very Large Codebases (100K+ files)

**Concern**: Storing 100K+ `IngestionCoverageFiles` records and returning them all via API.

**Mitigation**:
- The file-level table is stored but rarely queried in full. The primary API endpoint returns folder aggregates (typically <100 rows).
- The file-detail endpoint has a `limit` parameter (default 100).
- Batch insertion uses the same `insert_coverage_records_batch()` pattern proven for entity insertion.
- CozoDB handles 100K+ rows efficiently with RocksDB backend.

### 9.9 Incremental Re-Ingestion

**Scenario**: File watcher triggers re-ingestion of a single file. Should coverage data be updated?

**Approach**: Coverage data is generated during full `pt01` ingestion runs. Incremental re-indexing (via the file watcher and `/incremental-reindex-file-update`) does NOT update coverage tables. This is because coverage is a point-in-time snapshot of the full codebase, and incremental updates would require re-aggregating folder-level data (expensive).

**Future enhancement**: Track a `last_full_scan_timestamp` and show staleness in the API response.

### 9.10 Multiple Ingestion Runs

**Scenario**: User runs `pt01` multiple times on the same database.

**Handling**: Each run gets a unique `ingestion_run_id`. The HTTP endpoint always returns the latest run's data. Old coverage data is overwritten (`:put` semantics in CozoDB).

---

## 10. Testing Strategy

### 10.1 Unit Tests (parseltongue-core)

| Test | Description |
|------|-------------|
| `test_coverage_pct_calculation_standard` | 10 files, 8 parsed, 2 excluded = 100% |
| `test_coverage_pct_excludes_binary` | 10 files, 3 binary, 7 parsed = 100% |
| `test_coverage_pct_with_failures` | 10 files, 7 parsed, 3 failed = 70% |
| `test_coverage_assessment_thresholds` | Verify complete/high/medium/low/none |
| `test_folder_level_extraction` | `src/core/utils.rs` -> level1=`src`, level2=`src/core` |
| `test_file_status_classification` | Each status variant round-trips through DB |

### 10.2 Integration Tests (pt01-folder-to-cozodb-streamer)

| Test | Description |
|------|-------------|
| `test_coverage_tracking_full_pipeline` | Ingest test fixtures, verify coverage records |
| `test_binary_file_detection_coverage` | Include a `.png` in test fixtures, verify `binary` status |
| `test_unsupported_language_tracking` | Include a `.proto` file, verify `unsupported_language` |
| `test_excluded_folder_not_in_denominator` | Exclude `vendor/`, verify coverage formula |
| `test_parallel_coverage_consistency` | Parallel and sequential produce identical coverage |
| `test_folder_aggregation_accuracy` | Verify folder aggregates match file-level data |

### 10.3 E2E Tests (pt08-http-code-query-server)

| Test | Description |
|------|-------------|
| `test_endpoint_returns_coverage` | GET `/ingestion-coverage-folder-report` returns valid JSON |
| `test_depth_filter_works` | `?depth=1` returns only level-1 folders |
| `test_min_coverage_filter` | `?min_coverage=50` excludes high-coverage folders |
| `test_folder_filter` | `?folder=src/` returns only src/ subfolders |
| `test_file_detail_endpoint` | GET `/ingestion-coverage-file-detail?folder=src/` |
| `test_empty_database_graceful` | Returns empty coverage with 0% when no ingestion ran |

---

## 11. Implementation Phases

### Phase 1: Domain Types and Storage (1-2 days)

1. Add `FileIngestionStatus`, `FileCoverageRecord`, `FolderCoverageAggregate`, `GlobalCoverageSummary` types to `parseltongue-core`
2. Add `create_ingestion_coverage_schema()`, `insert_coverage_records_batch()`, `insert_folder_coverage_aggregate()`, `insert_global_coverage_summary()` to `CozoDbStorage`
3. Add query methods: `query_folder_coverage_aggregates()`, `query_file_coverage_records()`, `query_global_coverage_summary()`
4. Unit tests for all storage methods

### Phase 2: Ingestion Pipeline Integration (2-3 days)

1. Modify `stream_file()` to record `FileCoverageRecord` for each file
2. Modify `stream_directory()` and `stream_directory_with_parallel_rayon()` to track ALL files (not just processable ones)
3. Add binary file detection at the `read_file_content()` stage
4. Add unsupported language detection before tree-sitter parsing
5. Add post-ingestion aggregation step
6. Integration tests with test fixtures

### Phase 3: HTTP Endpoint (1-2 days)

1. Create `ingestion_coverage_folder_handler.rs`
2. Create `ingestion_coverage_file_detail_handler.rs` (optional, lower priority)
3. Register routes in `route_definition_builder_module.rs`
4. Add response types with `llm_guidance` section
5. E2E tests

### Phase 4: CLI Output Enhancement (0.5 days)

1. Print per-folder coverage summary to console after ingestion (like the current "Streaming Summary" but with folder breakdown)
2. Add `--coverage-report` flag to `pt01` CLI for verbose coverage output

### Estimated Total: 5-8 days

---

## 12. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Coverage data available | 100% of `pt01` runs produce coverage data | Automated test |
| API response time | < 50ms for folder-level report | Load test |
| API response time | < 100ms for file-level detail (100 files) | Load test |
| Storage overhead | < 5% increase in database size | Measure on 10K-file codebase |
| Ingestion overhead | < 10% increase in ingestion time | Benchmark before/after |
| LLM token count | < 500 tokens for typical folder report | Count response tokens |
| Coverage accuracy | Coverage % matches manual count on test fixtures | Automated verification |
| Parallel consistency | Parallel and sequential produce identical coverage | Automated comparison |

---

## 13. Open Questions

### Q1: Should Coverage Include Test Files?

Currently, test files are parsed but TEST entities are excluded from the CodeGraph. Should coverage report count test files as "parsed" (since they are parsed by tree-sitter) or "excluded" (since their entities don't enter the graph)?

**Recommendation**: Count test files as "parsed" in coverage data. The coverage report measures *parse completeness*, not *entity storage completeness*. The fact that TEST entities are excluded from the graph is a separate concern visible via `/codebase-statistics-overview-summary`.

### Q2: Should We Track Parse Quality (Not Just Success/Failure)?

Tree-sitter's error recovery means a file can produce a syntax tree with ERROR nodes but still yield some entities. Should we distinguish between "clean parse" and "partial parse with errors"?

**Recommendation**: For v1.6.1, track only binary success/failure. Add a `parse_quality` field (clean/partial/failed) in a future version if needed.

### Q3: Should Coverage Data Be Available During Incremental Re-Indexing?

If the file watcher triggers a re-index of `src/main.rs`, should the coverage for `src/` be updated?

**Recommendation**: No, for v1.6.1. Coverage is a snapshot from the last full `pt01` run. Add a `coverage_staleness_seconds` field to the API response so agents can judge freshness.

### Q4: Should We Provide a Coverage Diff Between Runs?

E.g., "Coverage improved from 85% to 92% since last ingestion."

**Recommendation**: Defer to v1.7.0. Store `ingestion_run_id` to enable this in the future.

### Q5: Should the Endpoint Name Be Different?

Alternative 4-word endpoint names considered:

- `/ingestion-coverage-folder-report` (chosen -- clear, specific)
- `/parse-coverage-folder-breakdown`
- `/codebase-coverage-analysis-report`
- `/graph-coverage-folder-summary`

The chosen name emphasizes that this is about *ingestion* coverage (what pt01 did), not test coverage or graph coverage in the abstract.

---

## Appendix A: Endpoint Summary (After v1.6.1)

After this feature, Parseltongue will have **23 HTTP endpoints** (21 existing + 2 new):

| # | Endpoint | Category | New? |
|---|----------|----------|------|
| 1 | `/server-health-check-status` | Core | |
| 2 | `/codebase-statistics-overview-summary` | Core | |
| 3 | `/api-reference-documentation-help` | Core | |
| 4 | `/code-entities-list-all` | Entity | |
| 5 | `/code-entity-detail-view` | Entity | |
| 6 | `/code-entities-search-fuzzy` | Entity | |
| 7 | `/dependency-edges-list-all` | Graph | |
| 8 | `/reverse-callers-query-graph` | Graph | |
| 9 | `/forward-callees-query-graph` | Graph | |
| 10 | `/blast-radius-impact-analysis` | Analysis | |
| 11 | `/circular-dependency-detection-scan` | Analysis | |
| 12 | `/complexity-hotspots-ranking-view` | Analysis | |
| 13 | `/semantic-cluster-grouping-list` | Analysis | |
| 14 | `/smart-context-token-budget` | Context | |
| 15 | `/incremental-reindex-file-update` | Maintenance | |
| 16 | `/file-watcher-status-check` | Maintenance | |
| 17 | `/strongly-connected-components-analysis` | Graph v1.6.0 | |
| 18 | `/technical-debt-sqale-scoring` | Analysis v1.6.0 | |
| 19 | `/kcore-decomposition-layering-analysis` | Graph v1.6.0 | |
| 20 | `/centrality-measures-entity-ranking` | Graph v1.6.0 | |
| 21 | `/entropy-complexity-measurement-scores` | Analysis v1.6.0 | |
| 22 | `/coupling-cohesion-metrics-suite` | Analysis v1.6.0 | |
| 23 | `/leiden-community-detection-clusters` | Graph v1.6.0 | |
| 24 | `/ingestion-coverage-folder-report` | Coverage | **NEW** |
| 25 | `/ingestion-coverage-file-detail` | Coverage | **NEW** |

## Appendix B: Token Budget Analysis

Typical API response sizes for the coverage endpoint:

| Scenario | Folders | Estimated Tokens |
|----------|---------|-----------------|
| Small project (10 folders) | 10 | ~150 tokens |
| Medium project (50 folders) | 50 | ~400 tokens |
| Large project (200 folders) | 200 | ~1,200 tokens |
| Monorepo (500+ folders) | 500 | ~2,500 tokens |

Even for monorepos, the coverage report stays well within Parseltongue's 2-5K token target. The `llm_guidance` section adds ~50 tokens but provides critical decision-making context for agents.

## Appendix C: Research Sources

- [SonarQube Code Coverage (Sonar)](https://www.sonarsource.com/blog/sonarqube-code-coverage/)
- [Codecov Sunburst Graphs](https://docs.codecov.com/docs/graphs)
- [Sourcegraph SCIP Protocol](https://sourcegraph.com/blog/announcing-scip)
- [Sourcegraph Precise Code Navigation](https://sourcegraph.com/docs/code-search/code-navigation/precise_code_navigation)
- [CodeScene Plugin System](https://codescene.io/docs/integrations/plugins/codescene-plugins.html)
- [Tree-sitter Error Recovery (GitHub Issue #224)](https://github.com/tree-sitter/tree-sitter/issues/224)
- [Tree-sitter Parse Error Reporting (GitHub Issue #4049)](https://github.com/tree-sitter/tree-sitter/issues/4049)
- [Completeness in Static Analysis by Abstract Interpretation (arXiv:2211.09572)](https://arxiv.org/abs/2211.09572)
- [LLM Hallucinations in Code Generation (arXiv:2409.20550)](https://arxiv.org/pdf/2409.20550)
- [Token-Level Hallucination Detection: HaluGate (vLLM Blog)](https://blog.vllm.ai/2025/12/14/halugate.html)
- [CodeHalu: Code Hallucination Framework (AAAI)](https://ojs.aaai.org/index.php/AAAI/article/download/34717/36872)
- [Code Quality Metrics (Codacy Blog)](https://blog.codacy.com/code-quality-metrics)
- [Best Practices for Code Coverage (Graphite)](https://graphite.com/guides/code-coverage-best-practices)
