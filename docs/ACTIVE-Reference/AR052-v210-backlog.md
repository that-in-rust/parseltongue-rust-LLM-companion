# v210-backlog
Status: Active backlog
Purpose: Track work intentionally deferred out of V200.
## Decision Summary
- `rust-llm-context-packer` is moved out of V200 and parked in V210 backlog.
- Rationale: V200 priority is dependency-graph-grounded context adequacy, not token-minimization strategy work.
- Operating principle: if dependency graph + read-path delivers sufficient context quality, extra packing/ranking layers are not required in current scope.
## Deferred Crate: `rust-llm-context-packer`
Decision:
- Fully deferred from V200 pass execution and risk-tracking scope.
- No V200 implementation commitment for context packing contracts, token estimators, or progressive detail tiers.
Deferred capability cluster:
1. Relevance ranking and pack orchestration
2. Token estimation/calibration
3. Progressive detail tiers
4. Model-specific budget profiles
5. Context payload shaping policies
## Re-entry Conditions (V210+)
Bring this crate back only if one or more conditions become true:
1. Dependency-graph read-path alone is measurably insufficient for answer quality.
2. Large-codebase response latency/size creates concrete operational failures.
3. Product direction explicitly requires model-budget optimization as a first-class feature.
4. Test evidence shows deterministic context selection cannot be achieved via read-path-only orchestration.
## Notes
- This is a scope-control decision, not a rejection of the crate forever.
- Until re-entry criteria are met, effort should stay on graph fidelity, dependency reasoning, and stable read-path behavior.





Perfect. I interpreted “all items” as the numbered Build Order items (1–40) in docs/PRD_v173.md, and scored each against your current mindset (graph fidelity + read-path determinism first, no premature token-optimization).
|   # | Item                           | 1-liner description                                   | Include Score (1-100) | Include in V200 |
| --: | ------------------------------ | ----------------------------------------------------- | --------------------: | --------------- |
|   1 | Slim types                     | Keep entity model minimal, stable, and deterministic. |                    95 | Yes             |
|   2 | DB getter                      | Core storage read access abstraction.                 |                    92 | Yes             |
|   3 | Export + Import                | Reliable graph serialization/deserialization path.    |                    88 | Yes             |
|   4 | pt02 crate                     | Snapshot export pipeline for portable graph files.    |                    35 | No              |
|   5 | pt08 snapshot loader           | Load snapshot into server runtime mode.               |                    40 | No              |
|   6 | Endpoint guards                | Enforce mode/capability correctness at API boundary.  |                    80 | Yes             |
|   7 | Route prefix nesting           | Namespace API by mode for routing clarity.            |                    65 | Later           |
|   8 | Auto port + port file          | Automatic port lifecycle/discovery management.        |                    55 | Later           |
|   9 | POST /shutdown endpoint        | Controlled server stop contract.                      |                    50 | Later           |
|  10 | shutdown CLI command           | CLI wrapper for graceful shutdown flow.               |                    48 | No              |
|  11 | Filesystem source read         | Read live source for entity detail/read-path quality. |                    90 | Yes             |
|  12 | Unblock smart-context          | Re-enable token-budget endpoint behavior.             |                    30 | No              |
|  13 | CLI --db/--mem flags           | Dual-mode runtime selection via CLI flags.            |                    45 | No              |
|  14 | /mem/ file watching            | Incremental updates in memory-backed mode.            |                    72 | Yes             |
|  15 | pt09 MCP crate                 | Native MCP server process integration.                |                    42 | No              |
|  16 | MCP tool definitions           | Tool registry for MCP-callable operations.            |                    38 | No              |
|  17 | MCP resource providers         | MCP passive resource exposure layer.                  |                    34 | No              |
|  18 | MCP CLI subcommand             | CLI entrypoint for MCP runtime.                       |                    32 | No              |
|  19 | Coverage: exclude tests        | Prevent test-only noise from coverage metrics.        |                    88 | Yes             |
|  20 | Coverage: zero-entity tag      | Distinguish parsed-empty vs unparsed files.           |                    85 | Yes             |
|  21 | Coverage: path normalize       | Canonical path matching for accurate coverage.        |                    90 | Yes             |
|  22 | Coverage: error log tags       | Structured diagnostics categories for failures.       |                    78 | Yes             |
|  23 | Remove debug eprintln          | Remove ingestion noise from operational output.       |                    82 | Yes             |
|  24 | Remove watcher eprintln        | Remove watcher chatter and keep signal clean.         |                    80 | Yes             |
|  25 | XML-tagged responses           | Semantic grouping to improve LLM context clarity.     |                    76 | Yes             |
|  26 | Swap walkdir → ignore          | Respect `.gitignore` and reduce graph pollution.      |                    87 | Yes             |
|  27 | Project slug in URL            | Self-describing multi-project endpoint identity.      |                    52 | Later           |
|  28 | Slug in port file              | Project-aware port-file naming for discovery.         |                    45 | No              |
|  29 | Token count at ingest          | Store real token counts per entity.                   |                    28 | No              |
|  30 | Smart-context real tokens      | Budgeting by actual tokens instead of heuristic.      |                    22 | No              |
|  31 | README audit                   | Align docs with actual runtime behavior.              |                    58 | Later           |
|  32 | Testing journal                | Track validation evidence and regressions over time.  |                    64 | Later           |
|  33 | Taint types + enums            | Security taint domain model primitives.               |                    40 | No              |
|  34 | Source/sink registry           | Curated taint source/sink knowledge base.             |                    38 | No              |
|  35 | Data-flow tree-sitter queries  | Extract assignment/param/return flow edges.           |                    55 | Later           |
|  36 | CozoDB taint relations         | Persist taint graph relations in storage model.       |                    50 | Later           |
|  37 | pt01 taint extraction          | Ingestion-time taint classification/extraction.       |                    52 | Later           |
|  38 | Taint flow endpoint            | Query taint propagation paths from API.               |                    45 | No              |
|  39 | Source/sink discovery endpoint | Expose taint inventory and classification summary.    |                    43 | No              |
|  40 | MCP taint tools                | MCP wrappers for taint APIs.                          |                    20 | No              |


|   7 | Route prefix nesting           | Namespace API by mode for routing clarity.            |                
|   8 | Auto port + port file          | Automatic port lifecycle/discovery management.        |                  
|  10 | shutdown CLI command           | CLI wrapper for graceful shutdown flow.               |
|  25 | XML-tagged responses           | Semantic grouping to improve LLM context clarity.     |
|  27 | Project slug in URL            | Self-describing multi-project endpoint identity.      |
|  28 | Slug in port file              | Project-aware port-file naming for discovery.         |
|  29 | Token count at ingest          | Store real token counts per entity.                   |
|  35 | Data-flow tree-sitter queries  | Extract assignment/param/return flow edges.           |