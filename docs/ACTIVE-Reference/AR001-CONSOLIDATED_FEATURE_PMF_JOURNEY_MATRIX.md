# Consolidated Feature PMF + Journey Matrix (Shreyas Doshi Lens)

Generated: 2026-02-25

Scope: all canonical features from `docs/CONSOLIDATED_FEATURE_POSSIBILITIES_DEDUP.md`
Total features scored: 278

PMF Interpretation: `90-100` must-have painkiller, `70-89` strong leverage, `50-69` useful but situational, `<50` optional/delight.

Score Basis: `source` = existing PMF found in research tables, `inferred` = Shreyas-style heuristic based on user journey criticality.

## FUJ-Compatible Journey Packs

| Journey | Purpose | Feature Count | Avg PMF | Core Pack (PMF>=85) |
|---|---|---:|---:|---|
| J1 Setup Journey | Install, configure, and connect gateway surfaces (MCP/HTTP/LSP/Tauri). | 14 | 82.0 | MCP Protocol Integration, MCP Server Core Implementation, HTTP API Coexistence Mode, LSP Integration (Client + Adapter), MCP JSON-RPC 2.0 Stdio Transport, Auto Port + Port File Lifecycle, Project Slug in URL Path, Route Prefix Nesting |
| J2 Ingest Journey | Parse code, enrich semantics, build stable graph state, and keep it fresh. | 40 | 69.6 | Data-Flow Tree-Sitter Queries, ISGL1 Stable Entity Key Format, ISGL1 v3 Exhaustive Graph Identity, Rust-Analyzer Enrichment Timeout Fallback, Rust-Analyzer Semantic Enrichment, Cross-Language Heuristic Detection (HTTP/FFI/WASM/PyO3/JNI/Ruby FFI), Tree-Sitter Grammar Updates (12 Languages), Concurrent Ingest Guard |
| J3 Query Journey | Find entities, trace dependencies, and retrieve optimal context quickly. | 32 | 75.8 | get_context MCP Tool, Smart Context Token Budget Endpoint, Budget Aware Query Planner, Tree-Sitter Query Pack, Incremental Query Streaming API, OpenAPI 3.0 Spec Generation, Token Count At Ingest, Top Callers/Callees Precompute |
| J4 Reasoning Journey | Run graph intelligence (SCC, centrality, complexity, debt, compliance). | 28 | 74.1 | SQALE Technical Debt Scoring, Tarjan SCC (Strongly Connected Components), Leiden Algorithm Clustering, Business-Context Technical Debt Scoring, Centrality Measures (PageRank/Betweenness), K-Core Decomposition Layering, Spectral Graph Partition, Technical Debt Quantification Scoring |
| J5 Refactor Journey | Convert analysis to execution plans, risk reduction, and migration decisions. | 14 | 72.9 | Progressive Root Cause Diagnosis, Change Impact Since Commit, Intelligent Refactoring Roadmap, Intelligent Refactoring Suggestions, Test Impact Prediction |
| J6 Scale Journey | Improve throughput, latency, cache efficiency, and runtime resilience. | 17 | 67.9 | Incremental Graph Update Performance, Parallel Graph Algorithm Execution |
| J7 Visualization Journey | Turn graph intelligence into explainable visuals and reports. | 26 | 57.5 | (none) |
| J8 Ecosystem Journey | Embed value inside IDE, CI/CD, cloud, and collaboration surfaces. | 24 | 56.4 | Kubernetes Operator, Docker Compose Deployment, VSCode Extension, JetBrains Plugin |
| J9 Governance Journey | Security, policy, and compliance-oriented code intelligence workflows. | 8 | 55.0 | (none) |
| J10 Experimental Journey | Delight/novel features with lower immediate practical pull. | 7 | 27.6 | (none) |
| J11 General Journey | Useful but uncategorized capabilities pending sharper product framing. | 68 | 61.4 | Multi-Crate Rust Analysis, SARIF Architecture Recovery, Semantic Module Boundary Detection, Entity Preview Signature Pointers, NPM Package Publishing Infrastructure, Hierarchical Module Boundary Detection, Monorepo Multi-Workspace Support, Git Blame Entity Ownership |

## Full Feature Scores By Journey

### J1 — Setup Journey
Install, configure, and connect gateway surfaces (MCP/HTTP/LSP/Tauri).

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| MCP Protocol Integration | 95 | source | MCP Protocol Integration | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v200-docs/Prep-V200-MCP-Protocol-Integration.md |
| MCP Server Core Implementation | 95 | source | MCP Server Core Implementation | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| HTTP API Coexistence Mode | 89 | inferred | HTTP API Coexistence Mode | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/ES-V200-attempt-01.md;docs/v200-docs/Prep-V200-MCP-Protocol-Integration.md |
| LSP Integration (Client + Adapter) | 89 | inferred | LSP Integration (Client + Adapter) | docs/v199-docs/CR-v173-04-oh-my-pi.md;docs/v200-docs/Prep-V200-Max-Adoption-Architecture-Strategy.md |
| MCP JSON-RPC 2.0 Stdio Transport | 89 | inferred | MCP JSON-RPC 2.0 Stdio Transport | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/v200-FUJ-Final-User-Journey.md;docs/v200-docs/Prep-V200-MCP-Protocol-Integration.md |
| Auto Port + Port File Lifecycle | 86 | inferred | Auto Port + Port File Lifecycle | docs/v200-docs/CR-cachebro-202601.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ES-V200-attempt-01.md |
| Project Slug in URL Path | 86 | inferred | Project Slug in URL Path | docs/v199-docs/PRD_v173.md; docs/v200-docs/ES-V200-Decision-log-01.md |
| Route Prefix Nesting | 86 | inferred | Route Prefix Nesting | docs/v199-docs/PRD_v173.md; docs/v200-docs/CR-codex-v200-implications.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-Hashing-Risks-v01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md; docs/v200-docs/ES-V200-attempt-01.md; docs/v200-docs/ZAI-PRD-contracts-01.md; docs/v200-docs/ZAI-PRD-contracts-02.md; docs/v200-docs/v210-backlog.md |
| Shutdown CLI Command | 86 | inferred | Shutdown CLI Command | docs/v199-docs/PRD_v173.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-Hashing-Risks-v01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ZAI-PRD-contracts-01.md; docs/v200-docs/ZAI-PRD-contracts-02.md; docs/v200-docs/v210-backlog.md |
| Slug-Aware Port File Naming | 86 | inferred | Slug-Aware Port File Naming | docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ES-V200-attempt-01.md |
| Tauri Instance Manager | 86 | inferred | Tauri Instance Manager | docs/v200-docs/v200-FUJ-Final-User-Journey.md;docs/v200-docs/ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md |
| Multi-Workspace Session Management | 62 | source | Multi-Workspace Session Management | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| CLI Flag: --stream | 58 | source | CLI Flag: --stream | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| API Key Authentication | 55 | source | API Key Authentication | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J2 — Ingest Journey
Parse code, enrich semantics, build stable graph state, and keep it fresh.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Data-Flow Tree-Sitter Queries | 94 | inferred | Data-Flow Tree-Sitter Queries | docs/v199-docs/PRD_v173.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-Hashing-Risks-v01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ZAI-PRD-contracts-01.md; docs/v200-docs/ZAI-PRD-contracts-02.md; docs/v200-docs/v210-backlog.md |
| ISGL1 Stable Entity Key Format | 94 | inferred | ISGL1 Stable Entity Key Format | docs/unclassified/ISGL1-v2-Stable-Entity-Identity.md;docs/A00_preserved_code_patterns/ISGL1-v2-key-generation-system.md;docs/v200-docs/Prep-V200-Key-Format-Design.md |
| ISGL1 v3 Exhaustive Graph Identity | 94 | inferred | ISGL1 v3 Exhaustive Graph Identity | docs/v199-docs/RESEARCH-isgl1v3-exhaustive-graph-identity.md;docs/v200-docs/Prep-V200-isgl1-ambiguity-risk-table.md |
| Rust-Analyzer Enrichment Timeout Fallback | 94 | inferred | Rust-Analyzer Enrichment Timeout Fallback | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/Prep-V200-Rust-Analyzer-API-Surface.md |
| Rust-Analyzer Semantic Enrichment | 94 | inferred | Rust-Analyzer Semantic Enrichment | docs/ACTIVE-Reference/THESIS-supermodel-competitive-analysis.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/v200-FUJ-Final-User-Journey.md |
| Cross-Language Heuristic Detection (HTTP/FFI/WASM/PyO3/JNI/Ruby FFI) | 93 | inferred | Cross-Language Heuristic Detection (HTTP/FFI/WASM/PyO3/JNI/Ruby FFI) | docs/v200-docs/Prep-V200-Cross-Language-Detection-Heuristics.md;docs/v200-docs/Research-Extraction-Tools-Per-Language.md |
| Tree-Sitter Grammar Updates (12 Languages) | 92 | source | Tree-Sitter Grammar Updates (12 Languages) | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Concurrent Ingest Guard | 90 | inferred | Concurrent Ingest Guard | docs/v200-docs/ES-V200-Decision-log-01.md |
| Cross-Language Boundary Pre-Filter | 90 | inferred | Cross-Language Boundary Pre-Filter | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/Prep-V200-Cross-Language-Detection-Heuristics.md |
| Cross-Language Edge Confidence Scoring | 90 | inferred | Cross-Language Edge Confidence Scoring | docs/v200-docs/v200-FUJ-Final-User-Journey.md;docs/v200-docs/ZAI-PRD-contracts-01.md |
| FactSet Protocol for Ecosystem Interop | 90 | inferred | FactSet Protocol for Ecosystem Interop | docs/v200-docs/ZAI-PRD-contracts-01.md;docs/v200-docs/Prep-V200-Max-Adoption-Architecture-Strategy.md |
| Ingestion Error Logging | 90 | inferred | Ingestion Error Logging | docs/pre175/v162-ingestion-error-logging-plan.md; docs/specs/ingestion-error-logging-spec.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| Slim Graph Snapshot Model | 90 | inferred | Slim Graph Snapshot Model | docs/pre175/THESIS-v173-slim-graph-address-model.md;docs/pre175/IMPLEMENTATION-v173-slim-snapshot-plan.md;docs/unclassified/TDD_v173_Slim_Snapshot_Tasks.md |
| Snapshot Guarded Endpoint Set | 90 | inferred | Snapshot Guarded Endpoint Set | docs/pre175/DECISION-v173-pt02-pt03-endpoint-selection.md;docs/pre175/IMPLEMENTATION-v173-slim-snapshot-plan.md |
| CozoDB Version Upgrade | 78 | source | CozoDB Version Upgrade | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Dart/Flutter Support | 78 | source | Dart/Flutter Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Kotlin Support | 75 | source | Kotlin Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/unclassified/CRITICAL-BUGS-ANALYSIS-v143.md |
| Scala Support | 72 | source | Scala Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Solidity Support | 72 | source | Solidity Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Elixir Support | 70 | source | Elixir Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| SQL Dialect Parsing | 70 | source | SQL Dialect Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Zig Support | 70 | source | Zig Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| CUDA/OpenCL Support | 68 | source | CUDA/OpenCL Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Haskell Support | 68 | source | Haskell Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Incremental LSP Enrichment | 68 | source | Incremental LSP Enrichment | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Temporal Graph Evolution Snapshots | 68 | source | Temporal Graph Evolution Snapshots | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Crystal Support | 65 | source | Crystal Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Nim Support | 65 | source | Nim Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| OCaml Support | 65 | source | OCaml Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Snapshot Comparison Tool | 62 | source | Snapshot Comparison Tool | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Minimum Spanning Tree | 55 | source | Minimum Spanning Tree | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Animated Architecture Evolution Video | 45 | source | Animated Architecture Evolution Video | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| PowerShell Support | 42 | source | PowerShell Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Julia Support | 40 | source | Julia Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Lua Support | 40 | source | Lua Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v199-docs/CR-v173-03.md; docs/v199-docs/Priortization-v173.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| Assembly (x86/ARM) | 35 | source | Assembly (x86/ARM) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Matlab Support | 35 | source | Matlab Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| COBOL Support | 32 | source | COBOL Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Verilog/VHDL Support | 32 | source | Verilog/VHDL Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Fortran Support | 30 | source | Fortran Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J3 — Query Journey
Find entities, trace dependencies, and retrieve optimal context quickly.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| get_context MCP Tool | 100 | inferred | get_context MCP Tool | docs/v200-docs/CR-codex-v200-implications.md; docs/v200-docs/ZAI-PRD-contracts-02.md |
| Smart Context Token Budget Endpoint | 97 | inferred | Smart Context Token Budget Endpoint | (no direct markdown filename match found) |
| Budget Aware Query Planner | 96 | source | Budget Aware Query Planner / Budget-Aware Code Review Context | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md; docs/unclassified/PRD-v3-backlog.md |
| Tree-Sitter Query Pack | 96 | inferred | Tree-Sitter Query Pack | docs/v200-docs/Prep-V200-Tree-Sitter-Query-Patterns.md;docs/v200-docs/ZAI-REUSABLE-Patterns-01.md |
| Incremental Query Streaming API | 92 | source | Incremental Query Streaming API / Server-Sent Events (SSE) Streaming | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| OpenAPI 3.0 Spec Generation | 92 | source | OpenAPI 3.0 Spec Generation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Token Count At Ingest | 92 | inferred | Token Count At Ingest | docs/v199-docs/PRD_v173.md; docs/v200-docs/CR-codex-v200-implications.md; docs/v200-docs/ES-V200-Decision-log-01.md; docs/v200-docs/ES-V200-Hashing-Risks-v01.md; docs/v200-docs/ES-V200-User-Journey-01.md; docs/v200-docs/ES-V200-attempt-01.md; docs/v200-docs/ZAI-PRD-contracts-01.md; docs/v200-docs/ZAI-PRD-contracts-02.md; docs/v200-docs/ZAI-REUSABLE-Patterns-01.md; docs/v200-docs/v210-backlog.md |
| Top Callers/Callees Precompute | 92 | inferred | Top Callers/Callees Precompute | docs/v200-docs/ES-V200-Decision-log-01.md |
| GraphQL API Layer | 91 | source | GraphQL API Layer | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Context-Aware Tech Debt Prioritization | 90 | source | Context-Aware Tech Debt Prioritization / Context-Aware Technical Debt Prioritization | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Prometheus Metrics Export | 90 | source | Prometheus Metrics Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Query Token Budget Estimator | 90 | source | Query Token Budget Estimator | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| WebSocket Real-Time Updates | 90 | source | WebSocket Real-Time Updates | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Stateful Query Pagination Bookmarks | 85 | source | Stateful Query Pagination Bookmarks | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Workspace Context Multiplexing | 82 | source | Workspace Context Multiplexing | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Token Estimation Accuracy Improvement | 75 | source | Token Estimation Accuracy Improvement | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| ISG Query Composition Pipeline | 72 | source | ISG Query Composition Pipeline | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Query Templates Library | 70 | source | Query Templates Library | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Semantic Code Search (Vector DB) | 67 | source | Semantic Code Search (Vector DB) / Semantic Search with Embeddings | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Dynamic Tool Discovery Endpoint | 65 | source | Dynamic Tool Discovery Endpoint | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Natural Language Query Interface | 65 | source | Natural Language Query Interface | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md; docs/v199-docs/CR-v173-03.md; docs/v199-docs/v173-CR-Shreyas-Feedback-01.md |
| REPL Interactive Query Shell | 65 | source | REPL Interactive Query Shell | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Batch Query Endpoint | 62 | source | Batch Query Endpoint | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Custom Query Workflow Builder | 62 | source | Custom Query Workflow Builder | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Graph Query DSL Endpoint | 62 | source | Graph Query DSL Endpoint | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Fuzzy Entity Key Completion | 60 | source | Fuzzy Entity Key Completion | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| GraphQL Schema Parsing | 60 | source | GraphQL Schema Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Path Finding (A* Search) | 58 | source | Path Finding (A* Search) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Fuzzy Diff Algorithm | 55 | source | Fuzzy Diff Algorithm | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| HTTP Response Headers (X-Estimated-Tokens) | 55 | source | HTTP Response Headers (X-Estimated-Tokens) | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Semantic Query Prompt Templates | 55 | source | Semantic Query Prompt Templates | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Documentation: Query Pipeline Examples | 42 | source | Documentation: Query Pipeline Examples | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |

### J4 — Reasoning Journey
Run graph intelligence (SCC, centrality, complexity, debt, compliance).

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| SQALE Technical Debt Scoring | 93 | source | SQALE Technical Debt Scoring | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/UserJourney20260202v1.md; docs/pre175/v160-development-journal.md; docs/unclassified/TDD_COMPREHENSIVE_TESTING_PLAN_V160.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| Tarjan SCC (Strongly Connected Components) | 93 | source | Tarjan SCC (Strongly Connected Components) / Tarjan's Strongly Connected Components | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/pre175/v165-TDD-PROGRESS.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| Leiden Algorithm Clustering | 91 | source | Leiden Algorithm Clustering | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Business-Context Technical Debt Scoring | 90 | source | Business-Context Technical Debt Scoring | docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/v199-docs/v173_backlog.md |
| Centrality Measures (PageRank/Betweenness) | 90 | source | Centrality Measures (PageRank/Betweenness) / Centrality Measures for Entity Importance | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| K-Core Decomposition Layering | 90 | source | K-Core Decomposition Layering | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/pre175/UserJourney20260202v1.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Spectral Graph Partition | 90 | source | Spectral Graph Partition / Spectral Graph Partition Decomposition | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Technical Debt Quantification Scoring | 90 | source | Technical Debt Quantification Scoring | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Information-Theoretic Entropy | 88 | source | Information-Theoretic Entropy / Information-Theoretic Entropy Complexity | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Context-Aware Complexity Scoring | 87 | source | Context-Aware Complexity Scoring | docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/v199-docs/v173_backlog.md |
| Coupling/Cohesion Metrics Suite (CK) | 85 | source | Coupling and Cohesion Metrics (CK Suite) / Coupling/Cohesion Metrics Suite (CK) | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Architecture Compliance Scanner | 74 | source | Architecture Compliance Scanner / Architecture Compliance Violation Scanner | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Git Churn Hotspot Correlation | 73 | source | Git Churn Hotspot Correlation | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Program Slicing Backward Forward | 72 | source | Program Slicing Backward Forward | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Code Clone Detection | 70 | source | Code Clone Detection / Code Clone Detection via AST Edit Distance | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_MERMAID_DIAGRAMS.md; docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Node2Vec Entity Embeddings | 70 | source | Node2Vec Entity Embeddings / Node2Vec Entity Embeddings CPU | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Temporal Architecture Drift Detection | 68 | source | Temporal Architecture Drift Detection | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Triangle Counting Cohesion | 68 | source | Triangle Counting Cohesion / Triangle Counting Cohesion Metrics | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Weisfeiler-Lehman Graph Kernel | 68 | source | Weisfeiler-Lehman Graph Kernel / Weisfeiler-Lehman Graph Kernel Similarity | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Anti-Pattern Detection Engine | 65 | source | Anti-Pattern Detection Engine | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Churn Hotspot Detection | 65 | source | Churn Hotspot Detection | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Cyclomatic Complexity Per Entity | 65 | source | Cyclomatic Complexity Per Entity | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Design Pattern Recognition | 63 | source | Design Pattern Recognition | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Defect Density Prediction | 62 | source | Defect Density Prediction | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Maintainability Index | 60 | source | Maintainability Index | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Halstead Complexity Metrics | 55 | source | Halstead Complexity Metrics | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| GDPR Compliance Scanner | 45 | source | GDPR Compliance Scanner | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Spectral Clustering | 45 | source | Spectral Clustering | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_MERMAID_DIAGRAMS.md; docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |

### J5 — Refactor Journey
Convert analysis to execution plans, risk reduction, and migration decisions.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Progressive Root Cause Diagnosis | 98 | source | Progressive Root Cause Diagnosis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Change Impact Since Commit | 92 | source | Change Impact Since Commit | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Intelligent Refactoring Roadmap | 92 | source | Intelligent Refactoring Roadmap / Intelligent Refactoring Roadmap Generation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Intelligent Refactoring Suggestions | 92 | source | Intelligent Refactoring Suggestions | docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/v199-docs/v173_backlog.md |
| Test Impact Prediction | 86 | source | Test Impact Prediction / Test Impact Prediction Selective Testing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Progressive Codebase Onboarding | 84 | source | Progressive Codebase Onboarding / Progressive Codebase Onboarding Assistant | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Dead Code Elimination Finder | 76 | source | Dead Code Elimination Candidate Finder / Dead Code Elimination Finder | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| API Contract Change Impact | 72 | source | API Contract Change Impact / API Contract Change Impact Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Microservice Extraction Candidates | 70 | source | Microservice Extraction Candidate Identification / Microservice Extraction Candidates | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| RefDiff Refactoring Detection History | 65 | source | RefDiff Refactoring Detection History | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Real-Time Refactoring Impact Preview | 58 | source | Real-Time Refactoring Impact Preview | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Documentation Gap Identifier | 52 | source | Documentation Gap Identifier | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Predictive Refactoring Suggestions | 45 | source | Predictive Refactoring Suggestions | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Pair Programming Workflow Suggester | 38 | source | Pair Programming Workflow Suggester | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |

### J6 — Scale Journey
Improve throughput, latency, cache efficiency, and runtime resilience.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Incremental Graph Update Performance | 88 | source | Incremental Graph Update Performance | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Parallel Graph Algorithm Execution | 86 | source | Parallel Graph Algorithm Execution | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Graph Compression Sparse Storage | 82 | source | Graph Compression Sparse Storage | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| PostgreSQL Backend | 78 | source | PostgreSQL Backend | docs/pre154/v152-ASYNC-PIPELINE-ARCHITECTURE-DIAGRAMS.md; docs/pre154/v152-ASYNC-PIPELINE-RUBBER-DUCK-ANALYSIS.md; docs/pre154/v152-DATABASE-ALTERNATIVES-RESEARCH.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/THESIS-v173-storage-architecture.md |
| Session Hot Path Cache | 78 | source | Session Hot Path Cache | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Async Pipeline Architecture | 75 | source | Async Pipeline Architecture | docs/pre154/v152-ASYNC-PIPELINE-ARCHITECTURE-DIAGRAMS.md; docs/pre154/v152-ASYNC-PIPELINE-SUMMARY.md; docs/pre154/v152-OPTION3-ASYNC-PIPELINE-RESEARCH.md; docs/pre154/v152-README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Random Walk Probability Impact | 72 | source | Random Walk Probability Impact | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| SQLite WAL Backend | 72 | source | SQLite WAL Backend | docs/pre154/v152-ASYNC-PIPELINE-SUMMARY.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Rayon Parallel Parsing | 70 | source | Rayon Parallel Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/SPEC-v169-sled-on-windows.md; docs/pre175/v154-PRD-RAYON-SLED-PARALLEL-INGESTION.md; docs/pre175/v154-TDD-PROGRESS.md; docs/pre175/v154-rayon-sled-parallel-ingestion.md |
| Benchmark Suite | 60 | source | Benchmark Suite | docs/pre154/TDD-PROGRESS-v1.4.9-dependency-patterns.md; docs/pre154/TDD-PROGRESS-v1.5.0-batch-insertion.md; docs/pre154/TDD-SPEC-v1.5.0-ingestion-performance.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v200-docs/Prep-V200-LLM-Context-Optimization-Research.md |
| Incremental Re-Index Queue | 60 | source | Incremental Re-Index Queue | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Query Result Caching | 60 | source | Query Result Caching | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Load Testing Tools | 58 | source | Load Testing Tools | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Query Performance Profiler | 58 | source | Query Performance Profiler | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Rate Limiting | 58 | source | Rate Limiting | docs/CR-droid-factory-20260219/factory-droid/CR-factory-droid-control-flow-202601.md; docs/PRD-research-20260131v1/PRD-v142-Parseltongue.md; docs/pre154/v152-ASYNC-PIPELINE-SUMMARY.md; docs/pre154/v152-README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| Background Job Scheduler | 55 | source | Background Job Scheduler | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| HTTP Response Headers (X-Cache-Status) | 45 | source | HTTP Response Headers (X-Cache-Status) | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |

### J7 — Visualization Journey
Turn graph intelligence into explainable visuals and reports.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Blast Radius Visualization | 82 | source | Blast Radius Visualization / Blast Radius Visualization Change Planning | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md; docs/v200-docs/ES-V200-User-Journey-01.md |
| Mermaid Diagram Generation | 82 | source | Mermaid Diagram Generation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| D3.js Force-Directed Graph | 80 | source | D3.js Force-Directed Graph | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Dependency Health Dashboard | 78 | source | Dependency Health Dashboard / Dependency Health Dashboard Generation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Dependency Structure Matrix Visualization | 78 | source | Dependency Structure Matrix Visualization | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| DSM (Dependency Structure Matrix) | 78 | source | DSM (Dependency Structure Matrix) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Graphviz DOT Export | 75 | source | Graphviz DOT Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| UMAP 2D Code Layout | 75 | source | UMAP 2D Code Layout / UMAP 2D Code Layout Projection | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Ingestion Coverage Diagnostics Report | 68 | inferred | Ingestion Coverage Diagnostics Report | (no direct markdown filename match found) |
| HTML Static Site Export | 62 | source | HTML Static Site Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| CLI Flag: --format (json/ndjson/tsv/csv) | 60 | source | CLI Flag: --format (json/ndjson/tsv/csv) | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Markdown Technical Documentation | 60 | source | Markdown Technical Documentation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| JSON Lines Bulk Export | 58 | source | JSON Lines Bulk Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| PDF Architecture Report Generator | 58 | source | PDF Architecture Report Generator | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| CSV/Excel Data Export | 55 | source | CSV/Excel Data Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Grafana Dashboard Templates | 48 | source | Grafana Dashboard Templates | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Code Ownership Heatmap | 45 | source | Code Ownership Heatmap | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Interactive Timeline Visualization | 45 | source | Interactive Timeline Visualization | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| 3D Graph Visualization | 42 | source | 3D Graph Visualization | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Cypher Query Export (Neo4j) | 42 | source | Cypher Query Export (Neo4j) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Code Age Heatmap | 40 | source | Code Age Heatmap | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Heatmap Overlays | 40 | source | Heatmap Overlays | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Team Collaboration Dashboard | 40 | source | Team Collaboration Dashboard | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Parquet Export | 38 | source | Parquet Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v199-docs/Priortization-v173.md |
| XML Export | 35 | source | XML Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v200-docs/ES-V200-User-Journey-Addendum-Tauri-CLI-Philosophy.md |
| RDF/Turtle Export | 32 | source | RDF/Turtle Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J8 — Ecosystem Journey
Embed value inside IDE, CI/CD, cloud, and collaboration surfaces.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Kubernetes Operator | 93 | source | Kubernetes Operator | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Docker Compose Deployment | 90 | source | Docker Compose Deployment | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| VSCode Extension | 88 | source | VSCode Extension | docs/pre154/UserJourney20260202v1.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/UserJourney20260202v1.md; docs/v199-docs/unstructured-CR-context-20260214.md |
| JetBrains Plugin | 85 | source | JetBrains Plugin | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| GitHub Action | 80 | source | GitHub Action | docs/pre154/RELEASE-CHECKLIST-v1.5.0.md; docs/pre154/RELEASE-PROGRESS-v1.5.0.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/unclassified/RELEASE_CHECKLISTS_COMPILATION.md; docs/v199-docs/Priortization-v173.md; docs/ACTIVE-Reference/THESIS-supermodel-competitive-analysis.md; docs/v199-docs/unstructured-CR-context-20260214.md; docs/v200-docs/CR-factory-droid-202601.md; docs/v200-docs/Prep-V200-Competitive-Deep-Dive.md; docs/v200-docs/Prep-V200-Max-Adoption-Architecture-Strategy.md |
| GitLab CI Integration | 75 | source | GitLab CI Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Vim/Neovim Plugin | 72 | source | Vim/Neovim Plugin | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Terraform/HCL Parsing | 60 | source | Terraform/HCL Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Dockerfile Analysis | 58 | source | Dockerfile Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Ansible Playbook Analysis | 55 | source | Ansible Playbook Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| AWS Lambda Deployment | 48 | source | AWS Lambda Deployment | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Jira Issue Linking | 48 | source | Jira Issue Linking | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Slack/Discord Bot Integration | 48 | source | Slack/Discord Bot Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Google Cloud Run Support | 46 | source | Google Cloud Run Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Azure Functions Integration | 45 | source | Azure Functions Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Helm Chart | 44 | source | Helm Chart | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Browser Extension | 42 | source | Browser Extension | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Notion Database Sync | 42 | source | Notion Database Sync | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Terraform Module | 42 | source | Terraform Module | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Ansible Playbook | 40 | source | Ansible Playbook | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Confluence Export | 40 | source | Confluence Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Mobile App (iOS/Android) | 40 | source | Mobile App (iOS/Android) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Nomad Job Spec | 38 | source | Nomad Job Spec | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Email Digest Reports | 35 | source | Email Digest Reports | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J9 — Governance Journey
Security, policy, and compliance-oriented code intelligence workflows.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| PII/Secret Detection | 68 | source | PII/Secret Detection | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Dependency Vulnerability Database | 65 | source | Dependency Vulnerability Database | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| License Compliance Scanner | 62 | source | License Compliance Scanner | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| SBOM (Software Bill of Materials) | 60 | source | SBOM (Software Bill of Materials) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Security Vulnerability Path Tracer | 55 | source | Security Vulnerability Path Tracer | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| OWASP Top 10 Scanner | 48 | source | OWASP Top 10 Scanner | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| CERT C Secure Coding | 42 | source | CERT C Secure Coding | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| MISRA C Compliance Checker | 40 | source | MISRA C Compliance Checker | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J10 — Experimental Journey
Delight/novel features with lower immediate practical pull.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Achievement Badges | 32 | source | Achievement Badges | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Voice Query Interface | 30 | source | Voice Query Interface | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| VR Codebase Exploration | 30 | source | VR Codebase Exploration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| AR Code Overlay (HoloLens) | 28 | source | AR Code Overlay (HoloLens) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Sonification (Graph to Sound) | 28 | source | Sonification (Graph to Sound) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Haptic Feedback (Complexity Vibration) | 25 | source | Haptic Feedback (Complexity Vibration) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Smell-o-Vision Code Smells | 20 | source | Smell-o-Vision Code Smells | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |

### J11 — General Journey
Useful but uncategorized capabilities pending sharper product framing.

| Feature | PMF | Basis | Aliases Merged | Filenames |
|---|---:|---|---|---|
| Multi-Crate Rust Analysis | 94 | source | Multi-Crate Rust Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| SARIF Architecture Recovery | 94 | source | SARIF Architecture Recovery / SARIF Architecture Recovery Integration | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Semantic Module Boundary Detection | 94 | source | Semantic Module Boundary Detection / Semantic-Guided Module Boundary Detection | docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md; docs/v199-docs/v173-pt04-bidirectional-workflow.md; docs/v199-docs/v173_backlog.md |
| Entity Preview Signature Pointers | 93 | source | Entity Preview Signature Pointers | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/pre175/v160-development-journal.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| NPM Package Publishing Infrastructure | 93 | source | NPM Package Publishing Infrastructure | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Hierarchical Module Boundary Detection | 91 | source | Hierarchical Module Boundary Detection | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Monorepo Multi-Workspace Support | 91 | source | Monorepo Multi-Workspace Support | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Git Blame Entity Ownership | 90 | source | Git Blame Entity Ownership | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Unix Piping Output Format | 90 | source | Unix Piping Output Format / Unix Piping Output Format (NDJSON) | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/README.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Layered Architecture Compliance Verification | 89 | source | Layered Architecture Compliance Verification | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Iterative Circular Dependency Classification | 88 | source | Iterative Circular Dependency Classification | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Semantic Cycle Classification | 88 | source | Semantic Cycle Classification | docs/PRD-research-20260131v1/PARSELTONGUE_V2_BIDIRECTIONAL_LLM_ENHANCEMENT.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/v199-docs/v173_backlog.md |
| Label Propagation Enhanced Clustering | 82 | source | Label Propagation Enhanced Clustering | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Pre-Commit Hook | 78 | source | Pre-Commit Hook | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Approximate Algorithms Massive Graphs | 75 | source | Approximate Algorithms Massive Graphs | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Emacs Package | 68 | source | Emacs Package | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Entity Versioning History | 68 | source | Entity Versioning History | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Jupyter Notebook Parsing | 68 | source | Jupyter Notebook Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Subgraph Export Local Execution | 68 | source | Subgraph Export Local Execution | docs/PRD-research-20260131v1/00_EXECUTIVE_SUMMARY.md; docs/PRD-research-20260131v1/03_V17_V19_ARIZE_PATTERNS_EXTRACTED.md; docs/PRD-research-20260131v1/04_VISUAL_ROADMAP_V14_TO_V19.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/QUICK_REFERENCE_7_PRDS.md; docs/PRD-research-20260131v1/README.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Cross-Language Dependency Analysis | 65 | source | Cross-Language Dependency Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Entity Content Hash Caching | 65 | source | Entity Content Hash Caching | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Graph Subgraph Extraction | 65 | source | Graph Subgraph Extraction | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Integration Test Suite | 65 | source | Integration Test Suite | docs/pre154/v151-primary-PRD.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Interactive Force Layout Graph | 65 | source | Interactive Force Layout Graph | docs/PRD-research-20260131v1/PARSELTONGUE_V2_FEATURES_RESEARCH_BACKED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| SigHashLookup CozoDB Table | 65 | source | SigHashLookup CozoDB Table | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v3.md; docs/unclassified/PRD-candidates-20260131v4.md |
| Config File Support (TOML/YAML) | 62 | source | Config File Support (TOML/YAML) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Graph Diff Algorithm | 62 | source | Graph Diff Algorithm | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Protobuf Schema Analysis | 62 | source | Protobuf Schema Analysis | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Ascent Runtime Max Hops Fact | 60 | inferred | Ascent Runtime Max Hops Fact | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/Prep-V200-Datalog-Ascent-Rule-Patterns.md |
| Database Migration Utilities | 60 | source | Database Migration Utilities | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Dependency Graph Export Endpoints | 60 | inferred | Dependency Graph Export Endpoints | docs/pre175/v162-dependency-graph-export-plan.md |
| Graph Union/Intersection | 60 | source | Graph Union/Intersection | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Ignore Patterns (.ptignore) | 60 | source | Ignore Patterns (.ptignore) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Lazy Datalog Fact Loading | 60 | inferred | Lazy Datalog Fact Loading | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/Prep-V200-Datalog-Ascent-Rule-Patterns.md |
| Live Source Read (Gate G3) | 60 | inferred | Live Source Read (Gate G3) | docs/v200-docs/ES-V200-attempt-01.md;docs/v200-docs/ES-V200-Decision-log-01.md |
| Uncertain Edge Handling Rules | 60 | inferred | Uncertain Edge Handling Rules | docs/v200-docs/ES-V200-Decision-log-01.md;docs/v200-docs/v200-FUJ-Final-User-Journey.md |
| XML-Tagged Response Categories | 60 | inferred | XML-Tagged Response Categories | docs/v200-docs/ES-V200-Decision-log-01.md |
| Custom Entity Type Definitions | 58 | source | Custom Entity Type Definitions | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Hierarchical Memory Persistence Layer | 58 | source | Hierarchical Memory Persistence Layer | docs/PRD-research-20260131v1/02_V16_PRD_IDEAS_EXTRACTED.md; docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Multi-Database Federation | 58 | source | Multi-Database Federation | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Mutation Testing Integration | 58 | source | Mutation Testing Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Workspace Tagging & Metadata | 58 | source | Workspace Tagging & Metadata | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Chaos Testing Framework | 55 | source | Chaos Testing Framework | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Debug Logging Levels | 55 | source | Debug Logging Levels | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Language-Specific Config | 55 | source | Language-Specific Config | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| YAML/TOML Config Parsing | 55 | source | YAML/TOML Config Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Health Check Ping Endpoint | 52 | source | Health Check Ping Endpoint | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Code Quality Leaderboard | 50 | source | Code Quality Leaderboard / Developer Leaderboard | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md; docs/unclassified/PRD-candidates-20260131v2.md |
| Bash/Shell Script Parsing | 48 | source | Bash/Shell Script Parsing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| LLM-Generated Code Summaries | 48 | source | LLM-Generated Code Summaries | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Louvain Community Detection | 48 | source | Louvain Community Detection | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| OpenTelemetry Tracing | 48 | source | OpenTelemetry Tracing | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/v199-docs/unstructured-CR-context-20260214.md; docs/v199-docs/v173-CR-Shreyas-Feedback-01.md; docs/v200-docs/Prep-V200-Compiled-Research-Best-Ideas.md; docs/v200-docs/ZAI-PRD-contracts-01.md |
| Datadog Integration | 45 | source | Datadog Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Linear Integration | 45 | source | Linear Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| PlantUML Export | 45 | source | PlantUML Export | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Auto-Fix Simple Code Smells | 42 | source | Auto-Fix Simple Code Smells | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Bus Factor Calculator | 42 | source | Bus Factor Calculator | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Girvan-Newman Algorithm | 42 | source | Girvan-Newman Algorithm | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Hierarchical Agglomerative Clustering | 42 | source | Hierarchical Agglomerative Clustering | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| New Relic Integration | 42 | source | New Relic Integration | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| DBSCAN Clustering | 40 | source | DBSCAN Clustering | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Documentation: Unix Piping Examples | 40 | source | Documentation: Unix Piping Examples | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |
| Markov Clustering (MCL) | 40 | source | Markov Clustering (MCL) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Code Review Assignment AI | 38 | source | Code Review Assignment AI | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Comment Quality Scoring | 38 | source | Comment Quality Scoring | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| R Support | 38 | source | R Support | docs/CONSOLIDATED_FEATURE_POSSIBILITIES_DEDUP.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/PRD-research-20260131v1/PRD-v142-Parseltongue.md; docs/pre154/v151-primary-PRD.md; docs/pre154/v152-DATABASE-ALTERNATIVES-RESEARCH.md; docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md; docs/pre175/v156-PRD-final.md; docs/v199-docs/CR-v173-01.md; docs/v199-docs/CR-v173-03.md; docs/v199-docs/PRD_v173.md; docs/ACTIVE-Reference/THESIS-supermodel-competitive-analysis.md; docs/v199-docs/THESIS-taint-analysis-for-parseltongue.md; docs/v199-docs/unstructured-CR-context-20260214.md; docs/v200-docs/Prep-V200-Competitive-Deep-Dive.md; docs/v200-docs/Prep-V200-Compiled-Research-Best-Ideas.md; docs/v200-docs/Prep-V200-MCP-Protocol-Integration.md; docs/v200-docs/Prep-V200-Rust-Analyzer-API-Surface.md; docs/v200-docs/Research-Extraction-Tools-Per-Language.md |
| Truck Number (Conway's Law) | 38 | source | Truck Number (Conway's Law) | docs/pre175/FINAL_FEATURE_EXTRACTION_MASTER_TABLE.md |
| Four-Word Naming Enforcement Tool | 35 | source | Four-Word Naming Enforcement Tool | docs/PRD-research-20260131v1/PRD-candidates-20260130.md; docs/PRD-research-20260131v1/PRD-candidates-20260131.md; docs/pre175/MASTER_FEATURE_EXTRACTION_TABLE.md |

## Global Top 50 By PMF

| Rank | Feature | PMF | Journey | Basis |
|---:|---|---:|---|---|
| 1 | get_context MCP Tool | 100 | J3 Query Journey | inferred |
| 2 | Progressive Root Cause Diagnosis | 98 | J5 Refactor Journey | source |
| 3 | Smart Context Token Budget Endpoint | 97 | J3 Query Journey | inferred |
| 4 | Budget Aware Query Planner | 96 | J3 Query Journey | source |
| 5 | Tree-Sitter Query Pack | 96 | J3 Query Journey | inferred |
| 6 | MCP Protocol Integration | 95 | J1 Setup Journey | source |
| 7 | MCP Server Core Implementation | 95 | J1 Setup Journey | source |
| 8 | Data-Flow Tree-Sitter Queries | 94 | J2 Ingest Journey | inferred |
| 9 | ISGL1 Stable Entity Key Format | 94 | J2 Ingest Journey | inferred |
| 10 | ISGL1 v3 Exhaustive Graph Identity | 94 | J2 Ingest Journey | inferred |
| 11 | Multi-Crate Rust Analysis | 94 | J11 General Journey | source |
| 12 | Rust-Analyzer Enrichment Timeout Fallback | 94 | J2 Ingest Journey | inferred |
| 13 | Rust-Analyzer Semantic Enrichment | 94 | J2 Ingest Journey | inferred |
| 14 | SARIF Architecture Recovery | 94 | J11 General Journey | source |
| 15 | Semantic Module Boundary Detection | 94 | J11 General Journey | source |
| 16 | Cross-Language Heuristic Detection (HTTP/FFI/WASM/PyO3/JNI/Ruby FFI) | 93 | J2 Ingest Journey | inferred |
| 17 | Entity Preview Signature Pointers | 93 | J11 General Journey | source |
| 18 | Kubernetes Operator | 93 | J8 Ecosystem Journey | source |
| 19 | NPM Package Publishing Infrastructure | 93 | J11 General Journey | source |
| 20 | SQALE Technical Debt Scoring | 93 | J4 Reasoning Journey | source |
| 21 | Tarjan SCC (Strongly Connected Components) | 93 | J4 Reasoning Journey | source |
| 22 | Change Impact Since Commit | 92 | J5 Refactor Journey | source |
| 23 | Incremental Query Streaming API | 92 | J3 Query Journey | source |
| 24 | Intelligent Refactoring Roadmap | 92 | J5 Refactor Journey | source |
| 25 | Intelligent Refactoring Suggestions | 92 | J5 Refactor Journey | source |
| 26 | OpenAPI 3.0 Spec Generation | 92 | J3 Query Journey | source |
| 27 | Token Count At Ingest | 92 | J3 Query Journey | inferred |
| 28 | Top Callers/Callees Precompute | 92 | J3 Query Journey | inferred |
| 29 | Tree-Sitter Grammar Updates (12 Languages) | 92 | J2 Ingest Journey | source |
| 30 | GraphQL API Layer | 91 | J3 Query Journey | source |
| 31 | Hierarchical Module Boundary Detection | 91 | J11 General Journey | source |
| 32 | Leiden Algorithm Clustering | 91 | J4 Reasoning Journey | source |
| 33 | Monorepo Multi-Workspace Support | 91 | J11 General Journey | source |
| 34 | Business-Context Technical Debt Scoring | 90 | J4 Reasoning Journey | source |
| 35 | Centrality Measures (PageRank/Betweenness) | 90 | J4 Reasoning Journey | source |
| 36 | Concurrent Ingest Guard | 90 | J2 Ingest Journey | inferred |
| 37 | Context-Aware Tech Debt Prioritization | 90 | J3 Query Journey | source |
| 38 | Cross-Language Boundary Pre-Filter | 90 | J2 Ingest Journey | inferred |
| 39 | Cross-Language Edge Confidence Scoring | 90 | J2 Ingest Journey | inferred |
| 40 | Docker Compose Deployment | 90 | J8 Ecosystem Journey | source |
| 41 | FactSet Protocol for Ecosystem Interop | 90 | J2 Ingest Journey | inferred |
| 42 | Git Blame Entity Ownership | 90 | J11 General Journey | source |
| 43 | Ingestion Error Logging | 90 | J2 Ingest Journey | inferred |
| 44 | K-Core Decomposition Layering | 90 | J4 Reasoning Journey | source |
| 45 | Prometheus Metrics Export | 90 | J3 Query Journey | source |
| 46 | Query Token Budget Estimator | 90 | J3 Query Journey | source |
| 47 | Slim Graph Snapshot Model | 90 | J2 Ingest Journey | inferred |
| 48 | Snapshot Guarded Endpoint Set | 90 | J2 Ingest Journey | inferred |
| 49 | Spectral Graph Partition | 90 | J4 Reasoning Journey | source |
| 50 | Technical Debt Quantification Scoring | 90 | J4 Reasoning Journey | source |
