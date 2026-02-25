# V210-Codemogger-Fuzzy-Candidate-Use-Case
Status: Reference draft
Date: 2026-02-25
Scope: V210 exploration (non-canonical, optional)

## Intent
Use `codemogger` as a fuzzy-search middle layer for natural-language retrieval, while preserving Parseltongue canonical graph truth as final authority.

## Why this exists
Some user prompts are fuzzy ("where is auth lifecycle", "payment retries flow") and do not map directly to one entity name.
`codemogger` can quickly return likely code regions.
Parseltongue then validates, maps, and reasons over canonical EntityKeys.

## Proposed Query Flow
```text
User/LLM fuzzy query
  -> codemogger semantic or hybrid search (top-K chunks)
  -> candidate spans (file + start_line + end_line)
  -> map spans to Parseltongue canonical EntityKey
  -> run graph algorithms in Parseltongue (blast radius, SCC, boundaries, etc.)
  -> answer from canonical graph
```

## Hard Boundary Rule
`codemogger` output is evidence, not truth.

Promotion to canonical facts is allowed only when:
1. Span maps unambiguously to a single canonical EntityKey
2. Mapping passes schema and confidence gates
3. Conflict checks do not fail

If mapping fails, keep result as `external_evidence` only.

## Where this fits in Big-Rock sequence
- Big-Rock-01 (ingestion truth loop): optional accelerator, not source of truth
- Big-Rock-02 (graph algorithms): candidate funnel to improve query recall

## Patterns worth stealing now (without adopting whole system)
1. Incremental hash + stale cleanup loop
   - Reference: `CR09/codemogger/src/db/store.ts`
2. Large-node AST splitting for chunk quality
   - Reference: `CR09/codemogger/src/chunk/treesitter.ts`
3. Query preprocessing + hybrid rank fusion
   - Reference: `CR09/codemogger/src/search/query.ts`
   - Reference: `CR09/codemogger/src/search/rank.ts`
4. MCP ergonomics (`index`, `search`, `reindex`)
   - Reference: `CR09/codemogger/src/mcp.ts`

## Non-goals for V210 use
1. Replacing Parseltongue canonical ingestion
2. Replacing canonical EntityKey model
3. Promoting fuzzy evidence directly into graph without validation
