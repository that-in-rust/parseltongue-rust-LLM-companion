---
name: dependency-map-cli-tools
description: Build a rough codebase map and dependency graph without Parseltongue or a database. Use this when you need fast pointer-first context (file:start:end), lightweight import/include edges, and a pragmatic architecture overview from CLI tools.
---

# Dependency Map CLI Tools

## Overview

Use this skill to produce a lightweight map of a codebase using standard CLI tools (`rg`, optional `ctags`, optional `ast-grep`, optional `dot`).

Primary output is pointer-first metadata (`file:start:end`) and rough dependency edges, not full semantic indexing.

## When To Use

Use this skill when:
- You need quick architecture orientation on a new repository.
- You do not want to run or maintain a persistent index database.
- You need "good enough" dependency mapping for LLM context selection.
- You want to return targeted spans to an LLM instead of dumping whole files.

Do not use this as the only source for compiler-level correctness across dynamic dispatch, macros, reflection, or generated code.

## Workflow

1. Generate rough map artifacts.
2. Read `summary.md` and identify likely hubs.
3. Pick top candidate symbols from `symbols.tsv`.
4. Expand local neighborhood using `internal_file_edges.tsv`.
5. Read only required spans using `read_code_span_pointer.sh`.
6. Return pointer-first context to the LLM.

## Quick Start

Run from repository root:

```bash
bash docs/skills/dependency-map-cli-tools/scripts/build_rough_codebase_map.sh . .code-map
```

Then inspect:

```bash
cat .code-map/summary.md
head -n 30 .code-map/symbols.tsv
head -n 30 .code-map/internal_file_edges.tsv
```

Read exact span:

```bash
bash docs/skills/dependency-map-cli-tools/scripts/read_code_span_pointer.sh src/main.rs:120:168
```

## Artifacts

`build_rough_codebase_map.sh` writes:
- `summary.md`: counts, top fan-in/fan-out, tool availability
- `files.tsv`: code file inventory + line counts
- `symbols.tsv`: symbol and line range (ctags when available, regex fallback)
- `import_edges.tsv`: lexical import/include edges
- `internal_file_edges.tsv`: resolved file-to-file edges (best effort)
- `external_ref_edges.tsv`: unresolved refs (likely third-party or alias)
- `dependency_graph.mmd`: Mermaid graph (edge-capped)
- `dependency_graph.dot`: DOT graph
- `dependency_graph.svg`: generated when Graphviz `dot` exists
- `tooling.tsv`: actual tool availability detected in this run

## Tool Strategy

- `rg` is required and acts as lexical baseline.
- `ctags` improves symbol span quality (`start_line` + `end_line`).
- `ast-grep` can be used manually for high-noise areas needing structural search.
- `dot` is optional visualization output.

If `ctags` is unavailable, the script falls back to language-specific regex symbol extraction.

## LLM Response Contract

When answering with this skill, return:
- Top 3 candidate entities with pointers (`file:start:end`).
- Why each candidate is relevant (query match + graph relation).
- Minimal surrounding evidence read from those spans.
- What is uncertain and what extra read is required.

Use this format:

```text
Candidate 1
- Pointer: path/to/file.ext:10:42
- Why: keyword match + high fan-in from related files
- Local edges: callers/importers from X, Y
- Confidence: high|medium|low
```

## Practical Limits

- Import/include edges are lexical and can miss runtime wiring.
- End lines are strongest when `ctags` is installed.
- Cross-language call graph precision requires LSP/SCIP-class infrastructure.

When precision is insufficient, switch to language-server queries or compiler-aware indexers.

## Resources

- Scripts:
  - `scripts/build_rough_codebase_map.sh`
  - `scripts/read_code_span_pointer.sh`
- Research context:
  - `references/internet-precedents.md`
