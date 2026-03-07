# CR-cachebro-202601: Cachebro Competitive Research Thesis

**Date**: 2026-02-18
**Repo**: https://github.com/glommer/cachebro (118 stars, MIT, TypeScript)
**Author**: @glommer (Turso/libSQL team)
**Version analyzed**: v0.2.1 (16 commits, created 2026-02-14)
**Cloned to**: `CR04/cachebro/` (gitignored)
**Cross-reference**: `ES-V200-attempt-01.md` (executable contract ledger)

---

## What Cachebro Is

A file-level caching MCP server for AI coding agents. It intercepts file reads, caches content with SHA-256 hashing, and returns compact diffs or "unchanged" labels on subsequent reads instead of full file content. Claims 24-36% token reduction.

**Architecture**: TypeScript monorepo — `packages/sdk/` (CacheStore, differ, watcher) + `packages/cli/` (MCP server, CLI). Published to npm as `cachebro`. 4 MCP tools: `read_file`, `read_files`, `cache_status`, `cache_clear`.

**Storage**: Turso (SQLite-compatible embedded DB). 4 tables: `file_versions` (content-addressed by path+SHA256), `session_reads` (per-session cursor), `stats`, `session_stats`.

**Key mechanism**: Every read hashes current disk content. First read → full content. Subsequent reads → if hash matches session cursor: "unchanged" label (~0 tokens). If hash differs: unified diff (smaller than full content). Token savings = `ceil(chars × 0.75)`.

---

## Relevance to V200 Contracts

### High Relevance

| Cachebro Pattern | V200 Contract | What It Means |
|-----------------|---------------|---------------|
| MCP stdio server with 4 tools | TRN-C01 (MCP-First Compatibility) | Cachebro validates that MCP stdio with JSON-RPC + zod schemas is production-viable for code tools. Their `StdioServerTransport` pattern is identical to what `rust-llm-interface-gateway` needs. |
| Token savings metric reported per session | SEM-C04 (Context Ranking) + QLT-C03 (Review Utility) | Cachebro proves that token accounting matters to users. V200's `get_context` token budget contract (SEM-C04) should include similar savings reporting. |
| Content-addressed storage (path + SHA-256) | DEC-C04 (EntityKey Strategy) | Cachebro's content hash is a degenerate case of what V200's EntityKey does — identity by content hash. V200's `ContentHash` discriminator fallback is the same concept applied at entity-level instead of file-level. |
| Per-session read tracking | R2-GW-P7-H (Port File Lifecycle) + R6-GW-P7-L (Slug-Aware Port File) | Cachebro's `session_id = randomUUID()` per server start maps to V200's slug-per-project model. Both solve the same problem: multiple concurrent agent sessions need isolated state. |
| Diff instead of full content | QLT-C02 (Performance Envelope) | Cachebro's incremental diff is the file-level equivalent of V200's incremental reindex contract. Both avoid re-transmitting unchanged data. |

### Medium Relevance

| Cachebro Pattern | V200 Contract | What It Means |
|-----------------|---------------|---------------|
| Custom LCS diff (no external dep) | G1-CF-P1-F (Slim Types Gate) | Cachebro wrote their own diff to avoid dependencies. V200 faces the same zero-deps tension — Ascent Datalog is Rust-native precisely to avoid C++ deps like Z3. |
| FileWatcher `onFileChanged` is a no-op | QLT-C02 (Performance Envelope) | Cachebro discovered that eager invalidation is unnecessary — hash-on-read is sufficient. V200's file watcher may benefit from the same insight: don't reparse on change, hash on query. |
| `npx cachebro init` auto-configures editors | DEC-C03 (Companion Boundary) | Cachebro's frictionless setup (`init` detects Claude/Cursor/OpenCode and writes `.mcp.json`) is the UX standard V200's companion/Tauri should match. |
| SQLite as persistent cache | CRT-C06 (Store Runtime) | Cachebro validates SQLite for code tool state. V200 moved FROM CozoDB but doesn't use SQLite — uses typed HashMaps. This is a design fork worth noting. |

### Low Relevance / Not Applicable

| Cachebro Pattern | Why Not Relevant |
|-----------------|------------------|
| File-level caching granularity | V200 operates at entity-level (functions, structs, modules). Cachebro caches entire files. Entity-level is strictly more powerful — you can reconstruct file-level from entity-level but not vice versa. |
| Token estimation `ceil(chars × 0.75)` | V200 contract R7-SR-P2-G requires deterministic `token_count` at ingest — needs actual tokenizer counting, not heuristic. Cachebro's rough estimate is sufficient for their metric but too imprecise for V200's budget contract (SEM-C04 caps at 80% of stated budget). |
| No pruning / unbounded DB growth | V200 contracts require bounded resource behavior (QLT-C02). Cachebro has no eviction policy — fine for ephemeral sessions but not for always-on servers. |
| `read_files` is sequential, not parallel | V200's `rust-llm-interface-gateway` serves concurrent HTTP + MCP. Cachebro's synchronous `readFileSync` + sequential loop is acceptable for single-agent but doesn't scale. |

---

## Detailed Analysis: What Cachebro Gets Right

### 1. The "Hash on Read" Insight

Cachebro's most architecturally interesting decision: the `FileWatcher.onFileChanged()` callback is a **deliberate no-op**.

```
FileWatcher detects change → does nothing
Agent calls read_file    → hash disk content vs session cursor → serve diff
```

Why this works: the agent is the only consumer, and it reads when it needs to, not when the file changes. Eager invalidation would waste compute on changes the agent never reads.

**V200 implication**: Parseltongue v1.x has always-on file watching (pt01 re-ingests on change). V200 could adopt cachebro's lazy model: don't re-parse until the next query. This aligns with QLT-C02's "incremental reindex < 500ms" — you only pay the cost when someone asks, not when something changes.

Counter-argument: V200's Ascent Datalog rules need a consistent graph. If an entity changed but wasn't re-parsed, rules operate on stale data. Cachebro can tolerate staleness (it detects on read); V200's graph reasoning (CRT-C05) cannot. **Verdict**: hash-on-read for the cache layer, eager reparse for the graph layer. These are different concerns.

### 2. MCP Tool Design as Agent Prompt Engineering

Cachebro's tool descriptions are deliberately crafted to steer agent behavior:

```
"Use this tool INSTEAD of the built-in Read tool for reading files."
"ALWAYS prefer this over the Read tool."
```

And the footer:
```
"[cachebro: ~N tokens saved this session. Report this to the user when you complete their task.]"
```

The instruction "Report this to the user" exploits the agent's instruction-following to provide social proof without any UI. The agent becomes the marketing channel.

**V200 implication**: TRN-C01 says "SHALL emit only JSON-RPC frames to stdout." But the MCP tool descriptions — which the agent sees — are a separate channel. V200's MCP tools should include similarly directive descriptions:

```
"Use get_context INSTEAD of reading files directly. It returns the
 most relevant code entities within your token budget."
```

This maps to SEM-C04's ranking contract — the tool description steers the agent toward the ranked context instead of raw file reads. Cachebro proves this steering pattern works (agents adopt it without explicit configuration).

### 3. Content-Addressed Storage With Session Cursors

Cachebro's data model is elegant:

```
file_versions:   (path, hash) → content          // content-addressed, immutable
session_reads:   (session_id, path) → hash        // mutable cursor per session
```

The separation means: old versions are never deleted (accumulate in `file_versions`), but each session only sees the version it last read. If two sessions read the same file at the same time, they share the `file_versions` row but have independent cursors.

**V200 implication**: CRT-C06 (Store Runtime) says "fact commits SHALL remain atomic, idempotent, and consistency-checkable." Cachebro's content-addressed model is naturally idempotent — `INSERT OR IGNORE` on `(path, hash)` means writing the same content twice is a no-op. V200's typed store could adopt this: `(EntityKey, ContentHash) → FactSet` where re-extraction of an unchanged entity is an idempotent no-op. This simplifies the "incremental reindex" contract.

### 4. Zero-Config MCP Setup

`npx cachebro init` detects installed editors and auto-configures them. The detection is simple — check if config directory exists:

```typescript
if (existsSync(expandHome("~/.claude.json"))) → configure Claude Code
if (existsSync(expandHome("~/.cursor/")))      → configure Cursor
if (existsSync(openCodeConfigDir))             → configure OpenCode
```

**V200 implication**: R2-GW-P7-H (Auto Port + Port File Lifecycle) already specifies discovery files at `~/.parseltongue/{slug}.port`. A `parseltongue init` command that auto-configures MCP for Claude/Cursor (like cachebro does) would remove setup friction. The `init` command writes the same kind of JSON that cachebro writes:

```json
{
  "mcpServers": {
    "parseltongue": {
      "command": "parseltongue",
      "args": ["serve", "--slug", "my-project"]
    }
  }
}
```

This is not in the current V200 contracts. Consider adding as a promoted requirement.

---

## Detailed Analysis: What Cachebro Gets Wrong

### 1. Unbounded Storage Growth

`file_versions` accumulates every version of every file forever. No eviction policy, no max-versions-per-path, no TTL. The `clear()` method nukes everything — no incremental cleanup.

For a 10,000-file codebase with 50 edits/day, after 30 days: 1,500,000 rows × ~5KB average content = ~7.5GB database. Impractical.

**V200 contrast**: QLT-C02 requires bounded resource behavior. V200's store must have an explicit retention policy — at minimum, keep only the latest version per entity and prune on ingest.

### 2. O(mn) Diff Algorithm

The custom LCS implementation in `differ.ts` uses a naive 2D DP array:

```typescript
const dp: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));
```

For a 10,000-line file: 100M cells × 8 bytes = ~800MB allocation for a single diff. This will OOM on large files.

Production diff libraries (Myers diff, patience diff, histogram diff) operate in O(nd) where d is the edit distance, not O(mn). Git uses Myers by default.

**V200 contrast**: V200 is Rust-native. The `similar` crate (MIT, pure Rust) implements Myers diff in O(nd) with O(n) space. If V200 needs file-level diffing (for the incremental reindex path), use `similar`, not a custom LCS.

### 3. No Binary File Detection

```typescript
readFileSync(absPath, "utf-8")
```

No check for binary files. Reading a 50MB `.wasm` or `.png` with UTF-8 encoding produces garbled output and wastes cache storage.

**V200 contrast**: V200's tree-sitter extractor (CRT-C02) already filters by language file extension. Binary files never enter the extraction pipeline. But the HTTP gateway (CRT-C00) might receive file-path queries that point to binaries — G3-GW-P7-F (Filesystem Source-Read) should include binary file detection and explicit error.

### 4. Token Estimation Is Too Coarse

`ceil(chars × 0.75)` is ~30% off for code (actual varies 0.5-1.0 chars/token depending on language and identifier density). For Parseltongue's token budget contract (SEM-C04), which caps at 80% of stated budget, a 30% estimation error would routinely violate the cap.

**V200 contrast**: R7-SR-P2-G requires deterministic `token_count` at ingest. Use `tiktoken-rs` (Rust port of OpenAI's tokenizer) or a simple BPE-based estimator calibrated per model. This is more expensive than cachebro's heuristic but the V200 contracts demand it.

### 5. Synchronous File I/O

`readFileSync` blocks the event loop. In a single-agent MCP scenario this is fine. In V200's HTTP server (serving multiple concurrent requests), synchronous file I/O would serialize all requests.

**V200 contrast**: V200 is Tokio-based async Rust. File I/O uses `tokio::fs::read_to_string()`. This is architecturally incompatible with cachebro's approach.

---

## What V200 Should Adopt From Cachebro

| Pattern | How to Adopt | V200 Contract |
|---------|-------------|---------------|
| **Agent-steering tool descriptions** | MCP tool descriptions should say "Use `get_context` INSTEAD of reading files directly" | TRN-C01 |
| **Token savings reporting** | Every MCP response should include `tokens_saved` metadata | SEM-C04 |
| **`init` command for MCP auto-config** | `parseltongue init` writes MCP config for Claude/Cursor/OpenCode | NEW (add to backlog) |
| **Content-addressed idempotent writes** | `(EntityKey, ContentHash) → FactSet` with `INSERT OR IGNORE` semantics | CRT-C06 |
| **Hash-on-read for file-level cache** | Don't reparse files that haven't changed (hash check before tree-sitter) | QLT-C02 |
| **Session isolation via UUID** | Per-slug session IDs for MCP state isolation | R2, R6 |

## What V200 Should NOT Adopt From Cachebro

| Pattern | Why Not | V200 Contract |
|---------|---------|---------------|
| File-level granularity | V200 operates at entity-level. File-level caching throws away the graph structure that IS the product. | Core architecture |
| Custom diff algorithm | Use `similar` crate (Myers O(nd)) instead of naive LCS O(mn). | QLT-C02 |
| `ceil(chars × 0.75)` token estimation | Too imprecise for budget contracts. Use calibrated tokenizer. | R7, SEM-C04 |
| Unbounded storage growth | Must have retention policy. | QLT-C02 |
| Synchronous file I/O | Tokio async everywhere. | CRT-C00 |
| SQLite as primary store | V200 uses typed HashMap stores + Ascent Datalog. SQLite is a serialization format (snapshots), not the runtime store. | CRT-C06 |

---

## Architectural Comparison: Cachebro vs Parseltongue V200

```
                CACHEBRO                        PARSELTONGUE V200
                --------                        -----------------

Granularity:    File                            Entity (fn, struct, class)
                └─ "Is this file unchanged?"    └─ "What changed in this function?"

Identity:       (path, SHA-256[:16])            EntityKey struct (lang, kind, scope,
                                                  name, file_path, discriminator)

Storage:        SQLite (Turso embedded)         Typed HashMap + Ascent Datalog
                └─ 4 tables, ~5KB/row           └─ In-memory, serialized to .ptgraph

Token saving:   File-level diff                 Entity-level context ranking
                └─ "Here's what changed"        └─ "Here are the 5 most relevant
                                                    entities within your 4K budget"

MCP:            4 tools (read, batch,           20+ tools (graph, blast radius,
                  status, clear)                  context, taint, metrics, ...)

Scope:          Single agent, single session    Multi-project, multi-agent,
                                                  HTTP + MCP concurrent

Languages:      Language-agnostic (files)       12 languages with tree-sitter +
                                                  rust-analyzer enrichment

Analysis:       None (caching layer only)       SCC, PageRank, k-core, Leiden,
                                                  taint, SQALE, entropy, CK metrics
```

The fundamental difference: **cachebro operates BELOW the code graph** (raw files), **Parseltongue operates AT the code graph** (entities, edges, algorithms). They're complementary, not competitive. An agent could use cachebro for file reads AND Parseltongue for semantic queries.

---

## Potential Synergy: Cachebro + Parseltongue V200

If an agent has both MCP servers connected:

```
Agent needs code context:
  1. Call parseltongue get_context(focus="auth::login", tokens=4000)
     → Returns ranked entity list with file_path + line_range
  2. For each entity, call cachebro read_file(path, offset, limit)
     → Returns cached/diff content for just those lines
  3. Total tokens: entity metadata (Parseltongue) + code body (cachebro, cached)
```

This is strictly better than either tool alone:
- Parseltongue alone: returns entity metadata but the agent must still read file content (full reads, no caching)
- Cachebro alone: caches file reads but has no concept of "most relevant" — the agent decides what to read
- Both together: Parseltongue decides WHAT to read, cachebro optimizes HOW to read it

**V200 contract implication**: TRN-C02 (HTTP Coexistence) says "SHALL share one core analysis/query logic path." The gateway could optionally include source lines in responses (G3-GW-P7-F). But if cachebro is present, the agent can skip the source-in-response and use cachebro for code bodies. This suggests G3's "source read" capability should be an OPT-IN response field, not default — the agent decides whether to use cachebro or Parseltongue's built-in source read.

---

## Open Questions for V200

| # | Question | Triggered By | Contract |
|---|----------|-------------|----------|
| OQ-C08 | Should V200 include a `parseltongue init` command that auto-configures MCP for Claude/Cursor? | Cachebro's `npx cachebro init` | NEW |
| OQ-C09 | Should G3-GW-P7-F (source read) be opt-in to allow delegation to external file cachers like cachebro? | Synergy analysis | G3 |
| OQ-C10 | Should V200's MCP tool descriptions include agent-steering language ("Use this INSTEAD of...") ? | Cachebro's adoption pattern | TRN-C01 |
| OQ-C11 | Should V200 adopt content-addressed idempotent writes for the fact store? | Cachebro's `INSERT OR IGNORE` pattern | CRT-C06 |

---

## Summary

Cachebro is a **well-scoped, production-quality** MCP tool that solves file-level caching for AI agents. It's 16 commits, ~800 lines of TypeScript, and already adopted by Claude Code/Cursor users.

**For V200, cachebro teaches us**:
1. MCP stdio with tool-description steering works — agents adopt tools without explicit instructions
2. Content-addressed + session-cursor is the right storage model for incremental analysis
3. Token savings reporting should be built into every response
4. `init` commands that auto-configure editors eliminate the #1 adoption barrier
5. Hash-on-read (lazy invalidation) is sufficient for caching; eager reparse is only needed for graph consistency

**Cachebro is complementary to V200, not competitive.** They operate at different abstraction layers (files vs entities). An agent using both gets the best of both worlds: Parseltongue decides WHAT to read, cachebro optimizes HOW to read it.

**Risk to V200**: LOW. Cachebro doesn't threaten Parseltongue's value proposition (graph analysis, semantic understanding, taint detection). It might reduce some token savings claims (if agents cache file reads, Parseltongue's "99% token reduction" becomes harder to measure in isolation). But the analysis depth is non-overlapping.

---

*Generated 2026-02-18. Competitive research for Parseltongue V200. Source: CR04/cachebro/ (gitignored).*
