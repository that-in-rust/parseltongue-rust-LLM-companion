# Deep Insight 07 — Span-first: the table before the graph

*Concise edition. The earlier long form (schema discussion, staged pipeline, AgentFS and the hands-on comparison of code-graph tools) is preserved verbatim as Part 9 of the collected Deep-Insight-01. This edition states the thesis, separates what is fixed from what is still moving, and gives the pipeline, the schema and the evidence in the fewest words that still carry them. Vertical sections only.*

---

## 1. The answer

Save the code first, as rows in one Turso table, using a CPU tool that decides nothing. Then let tiny models cut it into spans of public items and fill in what each span is. Derive the dependency graph afterwards — by SQL joins where names are unique, by a small binder model where they are not, and by the agent reading the spans directly when that is cheaper than any edge. The span table is the product's floor; the graph is a view over it.

```text
 usual         text → model → edges → store → views
 span-first    text → table (CPU) → spans (tiny model) → fields (tiny model) → edges (SQL, then binder) → views
```

## 2. Fixed — decided, not up for re-opening

**Order of work**
- Files go into the database first. A CPU tool walks the tree, records path and content digest, and writes each file's text into a `file` table in Turso. No parsing, no judgment.
- Spans come second, as rows that point into those files. Public items are the unit; private items get light rows (kind, name, file, range — no text) so traversal to a public anchor still works.
- Edges come last and are derived. An edge is never the first artefact.

**Who decides what**
- The deterministic tool does five things only: tokenize, digest, copy, insert, join. Everything that requires judgment — where an item starts and ends, its kind, its visibility, its fields, what it binds to — is a model's output.
- rust-analyzer and rustc are off-line teacher and judge. tree-sitter is not in the runtime. No Cargo build loop at runtime.
- The tool never writes `proposed`. An LLM or human never writes `extracted`.

**Records**
- Identity is the content digest of the span text; the file is identified by path @ file digest.
- Statuses: `extracted · abstained · inferred · proposed · realized · stale`. A name join that is not unique yields `inferred` with a confidence, never `extracted`. A site the binder cannot resolve is stored as `abstained` with its span — a record, not a gap.
- Levels: L0 crate (cargo metadata), L1 folder and file, L2 public-interface item, L3 body. L1 is a roll-up of L2 edges by the fixed rule (weight = count, kinds = union, cfg = union, confidence = [min, max]). L3 is lazy, per region, when an agent asks.

**Storage**
- One SQLite-compatible embedded file. Turso is the first candidate: Rust, MIT, recursive CTEs, FTS, vector functions, MVCC writers so spanners can write while agents read. The schema must stay portable to stock SQLite.
- The working tree remains the source of file bytes for a developer's checkout; the database is never the only copy of their code. When Parseltongue runs an agent in a sandbox, the file layer may be AgentFS's schema so code, map and tool-call log share one file.

## 3. Moving — to be chosen or measured

**Model sizes and gating**
- Span tagger: 2–5 MB reads everything; 100 MB reads only grey spans (failed parse, low confidence, macro soup) — or reads everything on a desktop GPU. The gate threshold is a knob.
- Field tagger for node details: 5, 30 or 100 MB. It reads span text only (10–20 % of the corpus), so size is a time trade-off, not a feasibility one.
- Binder: ~100 MB encoder over retrieved candidates with an explicit abstain class. Whether to distil it from a Rust-adapted 4B teacher is open.

**Packaging**
- Relay of specialists (one tiny model per stage) · one encoder with many heads · table-first with models as background repair workers. The third is the working frame; the other two are packaging choices inside it.

**Unmeasured numbers** — all arithmetic from published throughput, none measured on real hardware yet
- Linux-sized tree (~400 M tokens): 100 MB model reading everything — Air ~30 h, Mac mini ~7 h, desktop GPU ~30 min. Gated to grey spans behind a 2–5 MB tagger — Air ~25 min, Mac mini ~8–15 min. Any real Rust workspace (≤ 3 M lines): under 15 minutes everywhere.
- Boundary accuracy on broken Rust, for a deterministic scanner, a 5 MB tagger and a 100 MB encoder, scored separately from clean code — the one-afternoon benchmark that has not been run.
- Binder precision at coverage on held-out repositories; 98 % precision is the target, coverage is the honest second number.
- Whether span digests survive rename/move histories; whether Turso beats stock SQLite and the graph-native options after a benchmark.

## 4. Pipeline

Each stage reads rows and writes rows or columns; every stage is resumable and separately scorable against the compiler on held-out repositories.

```text
 0  CONTEXT   cargo metadata · walk tree · file digests · file text → `file` table        CPU tool
 1  SPANS     start, end, kind, name, visibility over all text                             2–5 MB tagger
 2  TEXT      copy span text + doc into columns · FTS index                                CPU tool
 3  CLARIFY   grey spans: fix boundary, kind, visibility; split or merge                   100 MB encoder
 4  NODES     signature parts, generics + bounds, flags, derives, cfg, impl trait/self      30–100 MB field heads
 5  EDGES     SQL joins on names → exact or candidate; binder on ambiguous → inferred/abstained   CPU join + 100 MB binder
 6  LABELS    one-line role per node (opinion layer, optional)                             labeller, on demand
```

## 5. Schema

```text
 file   id · path · digest · crate · text · snapshot
 span   id (= digest of text) · file_id · kind · name · path · visibility · start_line · end_line
        text · doc · cfg · creator · confidence · snapshot
 name   span_id · token                      (sub-tokens of name and path, for joins and BM25)
 field  span_id · key · value · creator · confidence     (stage-4 output; generics, bounds, flags, impl trait…)
 edge   src · dst · kind · status · confidence · evidence_span · snapshot   (materialised lazily)
```

## 6. Evidence from what exists

**AgentFS** — tursodatabase/agentfs, MIT, by Turso co-founder Pekka Enberg (turso.tech/blog/agentfs; agentfs-fuse; agentfs-just-bash). An agent's files, tool calls and key-value state in one SQLite/Turso file: `fs_inode`, `fs_dentry`, `fs_data` (4 KB chunks), `tool_calls`; FUSE/NFS mount; a shell that reads from the database. It stores bytes, not structure. It is the precedent for stage 0 — code saved into Turso first — and the file layer to adopt when sandboxing an agent.

**Codemogger** — glommer/codemogger, MIT, TypeScript, by Turso co-founder Glauber Costa (created 2026-02, 349 stars). A code-indexing library and MCP server: walks the tree, chunks each file with tree-sitter (WASM) into top-level definitions (functions, structs, classes, impl blocks; items over 150 lines split), embeds each chunk locally with all-MiniLM-L6-v2 (int8), and stores chunks + embeddings + FTS in one Turso `.db`. Incremental by SHA-256. Keyword search reported 25–370× faster than ripgrep on four codebases (Turso, TypeScript, …) on an M2 Air with 8 GB; indexing time ~97 % embedding. This is the closest precedent for DI-07's stage 1–2: definitions as rows in Turso with text search over them. It differs in three ways that define Parseltongue's gap: chunks are for retrieval, not records (no visibility, no fields, no edges, no statuses); the cutter is tree-sitter, not a model, so broken code is whatever the grammar recovers; and there is no graph, roll-up or proposed layer.

**Three code-graph MCP tools, run on `cargo-modules` (87 Rust files)** — mcp-codebase-index (regex; 9 file edges against 146 `use` lines), CodeGraphContext (tree-sitter → Kuzu; confidence labels on calls but 0 of 19 trait impls, visibility null, no file-level edges), code-graph-mcp (Rust, tree-sitter → SQLite; typed nodes and edges, `extracted | inferred | ambiguous` on every edge, blake3 hashes, FTS5, file and folder views, 0.14 s). The storage shape is proven by others. Nobody stores visibility, defines a roll-up rule, keeps abstention as a record, holds proposed structure, works on broken code, or puts a model in the resolver.

**Every other codebase tool** keeps files on disk and stores only derived records. AgentFS and Codemogger are the two that put code or code chunks into Turso — both by Turso's founders. Parseltongue takes the middle path: span text and fields in the database, the tree as byte source, file bytes in the database only for sandboxed agents.

## 7. Non-goals and the one risk

Not in scope: a verification or diff product; tool-authored future structure; replacing compiler diagnostics or rename; eager L3 for a whole tree; claiming any number above as measured.

The one risk: boundary accuracy on broken code. If the taggers cannot cut half-written items reliably, every stage above them inherits the error. It is also the cheapest thing to test — take compiling commits, break them, score the three cutters separately on clean and broken input.
