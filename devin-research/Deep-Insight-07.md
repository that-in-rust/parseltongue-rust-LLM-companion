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

## 8. The time-to-map equation

*The full form, as derived earlier in this research and carried here unchanged, then instantiated for the span-first pipeline. Every symbol is listed. Values marked "estimate" are envelopes from published throughput figures; the equation is built so each can be replaced by a measurement without changing shape.*

### 8.1 The equation

```text
 y  =  T_io  +  Σ over passes p of [ READ_p + WRITE_p ]  +  T_store  +  T_index  +  T_feed

 READ_p   =  N · (1 − h) · f_p · r_ctx(L) · 2 · P_act,p
             ─────────────────────────────────────────          prefill: every token the pass reads goes through
                       η_pre(P_act,p) · C_dev                    every active weight — compute-bound

 WRITE_p  =  N · (1 − h) · f_p · o_p · b_p · P_tot,p
             ──────────────────────────────────────────────────────────────────────────────
             min( η_dec · B_dev · batch_p ,  η_pre · C_dev / (2 · P_act,p) · b_p · P_tot,p )
                                                                decode (decoders only): every output token
                                                                re-reads all weights from memory, batched
                                                                until compute-bound; zero for encoders

 N_eff for the next pass  =  N · f_p · a_p                      a_p = abstention rate of pass p — what it hands upward
```

### 8.2 The variables

```text
 N          input tokens in the corpus              Linux-sized tree ≈ 400 M; a 3 M-line workspace ≈ 30 M; a crate ≈ 0.5 M
 h          cache / feed hit rate                    0 on the first map; 0.9+ on the second (content-addressed rows)
 f_p        routing fraction                        share of tokens pass p's model actually reads — THE lever
 P_act,p    active parameters per token             the compute cost; equals P_tot except for mixture-of-experts
 P_tot,p    total parameters                        memory and decode bandwidth
 b_p        bytes per weight                        int8 1 · int4 0.5 · fp16 2 — matters for WRITE and memory, barely for READ
 o_p        output ratio                            output tokens per input token; 0 for encoders, 0.1–0.3 for decoders
 L, r_ctx   context length and its multiplier       ≈ 1.0 at ≤ 4k tokens, ≈ 1.3 at 20k for transformers; 1 for state-space models
 C_dev      sustained compute, FLOP/s               estimate: M1 Air ~1.1e12 · M2 Air ~1.7e12 · M4 Pro mini ~1e13 · desktop GPU ~5e13
 B_dev      memory bandwidth, achieved              ~80 % of nominal: 68 · 100 · 120 · 273 GB/s across the Apple range
 η_pre/dec  utilisation                             tiny models cannot fill a GPU: ~1.0 at 7B · ~0.6 at 125M · ~0.2 at 2–5M unless batched and fused
 batch_p    decode batching                         1 … 32 parallel sequences; WRITE only
 k, a_p     number of passes and abstention rates   cascades shrink N per pass; a ≈ 0.02–0.20
 T_io       read and hash the tree                  ~5 min for 1.49 GB on four cores
 T_store    write rows                              seconds to minutes at 200–500k rows/s
 T_index    FTS and name indexes                    minutes
 T_feed     consult the public feed, if any         network-bound; zero off-line
```

### 8.3 Two facts the equation settles

- Quantisation does not speed up READ. Q4, Q8 and F16 prefill run within a few percent of each other on every chip measured, because prefill is compute-bound; quantisation acts on WRITE and on memory. An encoder cascade has no WRITE term, so for this pipeline int8 is a memory decision, not a time decision.
- Routing fraction and caching dominate. Halving P_act halves READ once; cutting f_p from 1.0 to 0.05 cuts it twenty times; h = 0.9 on the second map cuts everything by ten. After the first map, y is dominated by T_io + T_index, not by any model.

### 8.4 Instantiated for the span-first pipeline

Each stage is a pass; its f_p is what it reads. Encoders throughout, so WRITE = 0 everywhere.

```text
 stage   pass model            reads                               f_p       P_act      READ on an 8 GB Air (N = 400 M, h = 0)
 0       — (CPU tool)          whole tree, bytes only              —         —          T_io ≈ 5 min
 1       2–5 MB tagger         every token                         1.0       3 M        2.4e15 / (0.2–0.7 · 1.7e12) ≈ 35–100 min; batched and fused ≈ 20–25 min
 2       — (CPU tool)          span rows                           —         —          T_store + T_index ≈ 5–10 min
 3       100 MB clarifier      grey spans only                     0.02–0.05 100 M      1.6–4e15 / (0.6 · 1.7e12) ≈ 25–65 min
 4       30–100 MB field heads span text only (signatures + docs)  0.10–0.20 30–100 M   30M: ≈ 1–1.5 h · 100M: ≈ 3–5 h  (background; Mac mini ÷ 6)
 5       100 MB binder         ambiguous sites + candidates        ~0.01–0.03 100 M     ≈ 15–45 min for L2; L3 lazy, per region, seconds
 6       labeller              on demand                           —         —          not on the critical path

 first usable table (stages 0–2)      ≈ 25–40 min on an Air · ≈ 10–15 min on a Mac mini · minutes on a desktop GPU
 fully bound L1/L2 (through stage 5)  ≈ 2–3 h on an Air · ≈ 45 min on a Mac mini · ≈ 10 min on a desktop GPU
 second map (h ≈ 0.9)                 ≈ T_io + T_index ≈ 10–15 min regardless of hardware
 any real workspace (N ≤ 30 M)        every line above ÷ 13 — the whole pipeline inside 15 minutes on an Air
```

The one number that would change the table most is η at tiny sizes: a 3 M model at η = 0.2 is as slow as a 15 M model at η = 1.0. Batching and fused kernels are therefore worth more than any architectural choice for stage 1.

## 9. Structure

Parseltongue is one CLI with two entities inside it. The first is the thing we build; the second is the thing the agent uses.

```text
 parseltongue
 ├── multi-level-code-map        the entity we create — our key job
 │     one table set in Turso: file · span · name · field · edge
 │     levels L0 crate · L1 folder, file · L2 public-interface item · L3 body (lazy)
 │     statuses extracted · abstained · inferred · proposed · realized · stale
 └── multi-level-query-engine    the entity the agent-LLM talks to
       find · impact · status · propose — bounded, confidence-bearing answers over the map
```

**Dispatcher.** One binary, one verb per call. It routes a verb to exactly one CPU tool. CPU tools never call each other; they communicate only through the table.

**CPU tools and the tiny models they call.** The unit of work is a CPU tool. A tiny model is something a CPU tool calls, like a function: a batch of texts in, a batch of labels with confidence out, nothing else. A model never touches the table.

```text
 entity   CPU tool              what it does                                                        calls
 MAP      CPU-tool-01 INGEST    walk tree · digest · write file rows                                 —
 MAP      CPU-tool-02 SPAN      read file text in chunks → span rows (start, end, kind, name, vis)   tinyLLM01 spanner    2–5 MB
                                low-confidence rows → rewrite them                                   tinyLLM02 clarifier  ~100 MB
 MAP      CPU-tool-03 TEXT      copy span text and doc into columns · name tokens · FTS              —
 MAP      CPU-tool-04 FIELD     read span text → field rows (generics, bounds, flags, cfg, impl…)    tinyLLM03 fielder    30–100 MB
 MAP      CPU-tool-05 JOIN      SQL: use-paths × names → unique = edge(extracted) · else candidates  —
 MAP      CPU-tool-06 BIND      candidate lists → edge(inferred | abstained)                         tinyLLM04 binder     ~100 MB
 QUERY    CPU-tool-07 FIND      trie / FTS / BM25 over span + field → ranked rows + span text        tinyLLM05 ranker (optional)
 QUERY    CPU-tool-08 IMPACT    BFS over edge with depth, kind filter, confidence range              —
 QUERY    CPU-tool-09 PROPOSE   validate and write edge(status=proposed, creator=agent)              — never a model
 QUERY    CPU-tool-10 STATUS    counts per status and level · coverage banner · stale rows           —
```

**Contract.** Every CPU tool is rows in, rows out, idempotent, and resumable from the status column. Every model call is batch in, labels and confidence out. Three things follow: a model can be replaced by a deterministic stub with the same signature, so every stage is testable without a model; only CPU-tools 02, 04, 06 and optionally 07 ever need a GPU; and the table is the only shared state, so tools can run on different machines against the same file or against shards merged by digest.

## 10. Constraints and success criteria

**Success criterion.** A Linux-sized codebase — about 400 million tokens, 1.5 GB of source, used as the yardstick for "very large" — becomes a multi-level code map in 30 minutes. "Map" here means L0, L1 and L2 complete with every unresolved site stored as `abstained`, the coverage banner telling the truth, and L3 available lazily per region in seconds. Measured on the hardware named in the run; the Mac mini and desktop GPU meet it by the equation in §8, the 8 GB Air does not yet.

**Fixed constraints.**
- Code is saved first, by a CPU tool, into a Turso table; nothing reads the working tree after that.
- Every judgment is a tiny model's output; the CPU tool only moves bytes and runs SQL.
- No tree-sitter, rust-analyzer or Cargo build at runtime. rust-analyzer and rustc are off-line teacher and judge.
- One SQLite-compatible file; the schema stays portable to stock SQLite.
- The tool never writes `proposed`; an LLM or human never writes `extracted`.
- Public items are the unit; private items are light rows.
- Fully open: Apache/MIT tool, open weights, open training data, reproducible evaluation.
- Claims are precision with coverage printed beside it; no number is called measured until it is.

## 11. Open questions

Decisions deliberately deferred. Each is independent of the structure above.

- How a CPU tool calls its model: linked into the process (ONNX/safetensors, one binary) · sidecar executable over stdin/stdout · both.
- Where models run: inside the CLI call · background worker processes draining queues · a warm local server.
- Who schedules the stages: the CLI inline · a daemon · the agent calling each verb.
- How the agent talks to Parseltongue: subcommands with JSON output · MCP · both.
- What a query returns: raw rows · a token-budgeted page · span text verbatim.
- Model sizes per stage and the grey-span gate threshold (§3).
- Whether the binder is distilled from a Rust-adapted 4B teacher.
- Turso versus stock SQLite versus a graph-native store, after a benchmark.
- Span identity under rename and move.
- Whether the file layer adopts AgentFS's schema when an agent runs sandboxed.
