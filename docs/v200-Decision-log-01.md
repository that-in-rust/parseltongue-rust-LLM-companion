# v200-Decision-log-01
Status: Active
Purpose: Record binding scope and architecture decisions for V200.

## 2026-02-17 — Defer `rust-llm-context-packer` to V210
Decision:
- Remove `rust-llm-context-packer` from active V200 scope.
- Track it only in `docs/v210-backlog.md` with explicit re-entry criteria.

Why this decision was made:
1. V200 objective is dependency-graph-grounded context adequacy, not token-minimization strategy.
2. Context-packing is an optimization layer that can mask weak retrieval/fidelity in core graph and read-path contracts.
3. Deferring this crate reduces coupling, pass count, and contract surface so high-risk foundations converge faster.
4. This preserves optionality: packing is still available later, but only when evidence shows read-path-only orchestration is insufficient.

Decision principle:
- Prove graph fidelity and read-path determinism first; add context packing only after measured insufficiency and explicit product need.

Immediate scope impact:
- V200 active scope is 8 crates (context-packer excluded).
- Pass ledger/probe sequencing remains contiguous with no context-packer pass in V200.

## 2026-02-17 — Promote lifecycle + companion readiness bundle to V200 requirements
Decision:
- Promote the following `PRD_v173` ideas to explicit V200 pre-PRD requirements:
  - `#7` route prefix nesting
  - `#8` auto port + port file lifecycle
  - `#10` shutdown CLI command
  - `#25` XML-tagged response categories
  - `#27` project slug in URL path
  - `#28` slug-aware port file naming
  - `#29` token count at ingest
  - `#35` data-flow tree-sitter queries (`assign` / `param` / `return`)

Scope/architecture stance:
- These are requirement upgrades, not topology changes.
- V200 remains the same 8-crate core dependency graph.
- Tauri app work is treated as a companion consumer of stable gateway contracts (external app track), not as a new core crate.

Crate mapping:
- `rust-llm-interface-gateway`: `#7`, `#8`, `#10`, `#25`, `#27`, `#28`
- `rust-llm-store-runtime`: `#29`
- `rust-llm-tree-extractor`: `#35`

Decision principle:
- Lifecycle contracts and context clarity are first-class product requirements when they improve deterministic operation, discoverability, and LLM usability without introducing new core crates.
