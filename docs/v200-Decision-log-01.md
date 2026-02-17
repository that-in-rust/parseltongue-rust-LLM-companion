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
