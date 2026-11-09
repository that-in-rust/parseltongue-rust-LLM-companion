# Deep Insight 07 — Span-first: the table before the graph

*Concise edition. The earlier long form (schema discussion, staged pipeline, AgentFS and the hands-on comparison of code-graph tools) is preserved verbatim as Part 9 of the collected Deep-Insight-01. This edition states the thesis, separates what is fixed from what is still moving, and gives the pipeline, the schema and the evidence in the fewest words that still carry them. Vertical sections only.*

## 0. The whole document in one picture

```text
 THESIS      save the code into a table first → tiny models cut it into PUBLIC SPANS and fill in what each span is
             → derive the graph afterwards → serve the agent bounded answers, every unknown stored as a record
             → no compiler, tree-sitter or Cargo at runtime; rust-analyzer/rustc are off-line TEACHER and JUDGE

 agent-LLM / orchestrator / human
        │  one CLI
        ▼
 PARSELTONGUE ─┬─ ENTITY 1  multi-level-code-map  (we build it)      ─┬─ ENTITY 2  multi-level-query-engine (agent uses it)
               │   CPU-tool-01 INGEST   walk · digest · file rows      │   CPU-tool-07 FIND     trie/FTS/BM25 → rows + span text
               │   CPU-tool-02 SPAN     → tinyLLM01 spanner 2–5 MB     │   CPU-tool-08 IMPACT   BFS over edges, depth/kind/conf
               │                        → tinyLLM02 clarifier ~100 MB  │   CPU-tool-09 PROPOSE  writes HLD_future rows, never a model
               │   CPU-tool-03 TEXT     copy span text · names · FTS   │   CPU-tool-10 STATUS   coverage banner, stale rows
               │   CPU-tool-04 FIELD    → tinyLLM03 fielder 30–100 MB  │
               │   CPU-tool-05 JOIN     SQL: use-paths × names → edges │   rule: a tiny model makes EVERY judgment;
               │   CPU-tool-06 BIND     → tinyLLM04 binder ~100 MB     │         a CPU tool only MOVES BYTES and runs SQL;
               │                                                       │         a model never touches the table
               ▼                                                       ▼
 ┌─────────────────────────── ONE TURSO FILE (SQLite-compatible, portable) ───────────────────────────┐
 │  reference   file · span (raw code lives HERE, once, by digest; FTS here) · name · field · edge     │
 │  HLD_current  extraction-only  = the as-built design (L0 crate · L1 folder/file · L2 public items)   │
 │  HLD_future   agent/human-only = the intended design                                                 │
 │  HLD_delta    a VIEW, nobody writes it = to_build · to_remove · realized · unplanned                 │
 │  statuses     extracted · abstained · inferred · proposed · realized · stale                         │
 │  levels       L1 = roll-up of L2+L3 on write (weight · kinds ∪ · cfg ∪ · conf [min,max]) · L3 DEFERRED  │
 └─────────────────────────────────────────────────────────────────────────────────────────────────────┘

 PIPELINE    0 INGEST(CPU) → 1 SPAN(tinyLLM01, every token) → 2 TEXT(CPU) → 3 CLARIFY(tinyLLM02, grey 2–5 %)
             → 4 FIELD(tinyLLM03, span text only 10–20 %) → 5 JOIN(CPU) → 6 BIND(tinyLLM04, ambiguous ~1–3 %)
 EQUATION    y = T_io + Σ_p [ N·(1−h)·f_p·r_ctx·2·P_act / (η·C_dev)  +  WRITE_p (0 for encoders) ] + T_store + T_index
             → routing fraction f_p and cache hit h dominate; quantisation barely touches READ
 TARGET      Linux-sized tree (400 M tokens) → L0–L2 map in 30 min on an 8 GB MacBook Air — the HARD target
             forces: tinyLLM01 ≤ 3 M params at η ≥ 0.7 · heavy models read ≤ 2–3 % inside the window · CPU ≤ 10 min · h ≥ 0.9 on re-map
 HLD LOOP    edit HLD_future rows → HLD_delta.to_build → agent writes exactly that code → re-extract → realized
 RUST RULE   items are order-free (fn, struct, impl, use anywhere in the file) → identity = digest, never line numbers;
             statements run top-down only inside bodies (L3); macro_rules! is the one position-sensitive item
 PRECEDENTS  AgentFS (files in Turso) · Codemogger (tree-sitter chunks in Turso) · code-graph-mcp (typed edges + confidence)
             — storage shape proven by others; nobody stores visibility, a roll-up rule, abstention, or a future
 OPEN        model call style · runtime · scheduler · agent interface · return shape · grey rule · L3 trigger · teacher loop
             · retire marker · rows vs events for HLD_future · boundary accuracy on broken code (the one risk)
```

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
- Levels: L0 crate (cargo metadata), L1 folder and file, L2 public-interface item, L3 body. L1 is a roll-up of L2 and L3 edges by the fixed rule (weight = count, kinds = union, cfg = union, confidence = [min, max]), maintained on write. L3 is deferred and incremental — pending rows counted from the first map, resolved in the background, immediately for edited files and on demand for queried regions — never skipped (§12.7).

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
                                L3: find sites in bodies → pending rows                               tinyLLM06 site tagger 2–5 MB
 QUERY    CPU-tool-07 FIND      trie / FTS / BM25 over span + field → ranked rows + span text        tinyLLM05 ranker (optional)
 QUERY    CPU-tool-08 IMPACT    BFS over edge with depth, kind filter, confidence range              —
 QUERY    CPU-tool-09 PROPOSE   validate and write edge(status=proposed, creator=agent)              — never a model
 QUERY    CPU-tool-10 STATUS    counts per status and level · coverage banner · stale rows           —
```

**Contract.** Every CPU tool is rows in, rows out, idempotent, and resumable from the status column. Every model call is batch in, labels and confidence out. Three things follow: a model can be replaced by a deterministic stub with the same signature, so every stage is testable without a model; only CPU-tools 02, 04, 06 and optionally 07 ever need a GPU; and the table is the only shared state, so tools can run on different machines against the same file or against shards merged by digest.

## 10. Constraints and success criteria

**Success criterion.** A Linux-sized codebase — about 400 million tokens, 1.5 GB of source, the yardstick for "very large" — becomes a multi-level code map in 30 minutes **on an 8 GB MacBook Air**. The Air is the hard target, not a nice-to-have; the design must get there. "Map" means L0, L1 and L2 complete with every unresolved site stored as `abstained`, every L3 site written as a pending row so it is counted, a per-level coverage banner telling the truth, and L3 resolved in the background and on demand per region in seconds (§12.7). Faster hardware is a bonus, never the assumption.

**What the Air target forces.** Thirty minutes of Air GPU is ≈ 2.2e15 operations (plus ≈ 0.7e15 on CPU), against 400 M tokens. By the equation in §8 that budget is met only if all four hold:

```text
 a  tinyLLM01 reads everything at ≤ ~5 M ops/token  →  ≤ 2–3 M parameters, and η ≥ 0.7 through batching and fused kernels
    (3 M params × 400 M tokens = 2.4e15 ops ≈ 20 min at η 0.7; at η 0.2 it is 70 min and the target is missed)
 b  the heavy models — tinyLLM02 clarifier, 03 fielder, 04 binder — read ≤ 2–3 % of the text inside the window;
    everything else they do is background work after minute 30, flipping abstained → extracted while the agent works
 c  T_io + T_store + T_index ≤ 10 min: hash and insert 1.49 GB, write ~3 M span rows, build FTS and name indexes
 d  the second map is cache-bound: h ≥ 0.9 by content digest, so re-mapping costs T_io + T_index, not model time
```

If (a) fails on real hardware, the fallback that keeps the criterion is a deterministic first cut for spans with tinyLLM01 demoted to the grey rows — a loss of the "model owns every judgment" purity at stage 1 only, recorded here so it is a conscious trade, not a drift.

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
- What a query returns to the agent-LLM by default: headlines first (one line per hit, expand on demand) · a token-budgeted page packed by the engine · raw rows as JSON · span text verbatim.
- Model sizes per stage and the grey-span gate threshold (§3).
- Whether the binder is distilled from a Rust-adapted 4B teacher.
- Turso versus stock SQLite versus a graph-native store, after a benchmark.
- Span identity under rename and move.
- Whether the file layer adopts AgentFS's schema when an agent runs sandboxed.
- The training and evaluation loop as a component: who runs rust-analyzer off-line to label spans, fields and bindings for tinyLLM01–04 and scores them on held-out repositories — a `parseltongue-teacher` inside this repo, or a separate one.
- What "grey" means precisely: the rule CPU-tool-02 uses to hand a span to tinyLLM02 — confidence below a threshold · unbalanced braces · a macro invocation inside the span · unknown kind · some combination.
- The L3 trigger: resolve bodies only when the agent asks (`impact --depth`, `find --body`), or also pre-warm regions in the background once L2 is bound.

## 12. HLD_current · HLD_future · HLD_delta

*The map, named for what it is. L0–L2 of the multi-level code map is a high-level design extracted from the code; the proposed records are the high-level design as intended. Giving the two tables those names, and deriving the gap between them as a third, gives humans and agents one language: rows.*

### 12.1 Three logical databases in one file

```text
 HLD_current   written only by extraction (CPU tools + tiny models)        the as-built design, true by construction
 HLD_future    written only by the agent-LLM or a human; never by the tool   the intended design
 HLD_delta     a VIEW over the two — nobody writes it                        the gap; it cannot be stale or disagree with the truth

 all three live in the one Turso file beside file · span · name · field · edge
```

A row in either HLD table is one edge at the signature level:

```text
 node01_signature · node01_file · node01_folder · node01_kind · node01_span_id
 node02_signature · node02_file · node02_folder · node02_kind · node02_span_id
 edge_type · status · confidence · cfg · creator · reason · snapshot
```

Raw code is not in the row. It lives once in the reference `span` table, keyed by digest, where the full-text index also lives. Search starts there — FTS over `span.raw_code` returns span ids; one join lands in the HLD rows with their folders and edges. An HLD row is the signature level by definition; the body is LLD and does not belong in it, and a span that appears in many rows must be stored once so a change touches one place.

Nodes with no edges still get a row (node02 null, edge_type `none` or `abstained`), so "no edges" means none rather than "not indexed".

### 12.2 HLD_delta as a view

```text
 full outer join of HLD_future and HLD_current on (node01_span_id, node02_span_id, edge_type)

 in future only                         → to_build      the work; a bounded task with a file, a folder, neighbours, a signature
 in current only, future says retire    → to_remove
 in both                                → realized      the proposed row's status flips when extraction lands a match
 in current only, no future opinion     → unplanned     structure nobody intended — the most interesting rows on a legacy tree
```

Because it is a view, the agent's loop is one query — `SELECT * FROM HLD_delta WHERE state = 'to_build' ORDER BY node01_folder` — and there is no job to forget and no second artefact to trust. At kernel scale a whole-tree read of the view is slow (tens of millions of edge rows); it is then materialised per folder or per snapshot into a cache table that is regenerable from the view, so it can be fast without ever being wrong.

### 12.3 Programming at the HLD level

```text
 today                                   with the three tables
 code is the source                      HLD_future is what you edit
 the design document drifts              HLD_current is extracted — always true
 "what should I write?" is open-ended    "make this to_build row real" has a location, neighbours and a signature
 review reads diffs of code              review reads rows of intent; the realized flip is the proof the code matches

 human / agent  ──writes rows──►  HLD_future
                                       │   HLD_delta: to_build
                                       ▼
                                agent writes code for exactly those rows
                                       │
                                re-extract (CPU tools + tiny models)
                                       ▼
                                HLD_current gains the matching rows  →  to_build becomes realized
```

Gaps are visible before any code exists: a future node with no incoming edge is dead on arrival; a future edge from `gateway` to `db` breaks an agreed layering; both are rows, found by SQL, with nothing written.

### 12.4 Scope note

Earlier in this research the verification product — a diff command, an as-built map, a conformance report, a CI gate — was cut as unnecessary complication. HLD_delta is a deliberate, smaller reversal: a derived view, no workflow of its own, no report, no gate. The reason it earns its place is language — three named tables let a human and an agent talk about the same rows.

### 12.5 Open in this section

- Deletion: "this edge should stop existing" needs a `retire` marker in HLD_future that becomes realized when extraction stops seeing the row.
- Whether HLD_future is written as whole rows or as add/retire events with a reason (the event form gives "why" and "when" for free and makes retire natural; a view can present it as rows either way).
- The one-hour test: write the view, hand-author ten HLD_future rows for `cargo-modules`, and check that to_build / to_remove / realized / unplanned come out as a person would classify them. No model needed.

### 12.6 Building HLD_current, step by step

*A derivation from the base tables; every step is SQL a CPU tool runs. Inputs: `file`, `span`, `field` (stage 4), `edge` (stage 5 join and stage 6 binder; L2- and L3-sourced).*

**Step 1 — the node set: every public span, once.** `hld_node` = spans with visibility `pub` or `pub(crate)` whose kind is an item (members — fields, variants — fold into their parent), joined to `file` for path and folder. `pub(crate)` is kept with its visibility column: it is HLD inside the crate and filtered out across crates.

**Step 2 — L2-sourced edges from signatures.** Each `field` row that names a type (param_type, return_type, field_type, implements, bound, reexports) is joined through the file's `use` scope to a span by path. A unique match is `extracted`; several matches are `candidate` rows for CPU-tool-06 and tinyLLM04, which turn them into `inferred` or `abstained`.

**Step 3 — L3-sourced edges, collapsed to public anchors.** Raw call, construct and may_call rows exist between any spans. HLD wants public → public, so a recursive walk starts at a public item, passes through private items (depth ≤ 6), and stops at the first public target: depth 1 is `calls`, deeper is `calls_via` with the chain as evidence and the minimum confidence along it. Private → private calls never enter HLD on their own.

**Step 4 — HLD_current: one row per (public item, public item, edge kind).** Join `edge` to `hld_node` twice and aggregate: status by precedence (any extracted → extracted; else any inferred → inferred; else abstained), confidence as [min, max], `evidence_count` = number of sites, `evidence_spans` as the list, cfg as the union, snapshot as the latest. The signature text from `field` rows is kept in the row so it reads as a sentence.

**Step 5 — nodes with no edges still get a row** (`edge_type = none`), and every abstained site with an unknown target gets a row with node02 null — so "no edges" means none and "unknown" is counted.

**Step 6 — the L1 roll-ups, maintained on write.** `hld_file (src_file, dst_file, weight, kinds, conf_min, conf_max, status)` and `hld_folder` likewise, filled once by GROUP BY and then kept current by an AFTER INSERT/DELETE trigger on `hld_current` that upserts weight ± 1, kinds ∪, confidence min/max. Every L3 edge that lands is in the folder map immediately.

**Step 7 — incremental maintenance.** A changed file deletes its span, field and edge rows, re-runs stages 1–6 for that file only, deletes the HLD rows touching its spans and re-inserts them; triggers fix the roll-ups. A binder result updates one `edge` row and re-aggregates one HLD row. Every row carries the tree digest it was built from; `stale` is a comparison, not a flag someone sets.

### 12.7 L3 is deferred, not lazy — and how it is built

**The correction.** Earlier sections said "L3 lazy". That is right for the first map on the Air and wrong as a steady state, because roughly half of HLD edges (the `calls` kinds) are read from bodies: until L3 is bound for a file, that file's HLD rows show only signature-sourced edges, and file → file dependencies that exist only through calls (glob imports, trait methods) are missing from L1 altogether. So:

```text
 first map (30-minute criterion)   L0–L2 complete · every L3 site written as a row with status = pending, so it is COUNTED
 after minute 30                   CPU-tool-06 drains L3 in the background: the folders the agent touched first, then by in-degree
 on every edit                     the changed file's L3 is resolved immediately — one file, seconds
 on every agent query              the region asked about is resolved on demand, ahead of the queue
 on every L3 row written           the roll-up updates incrementally (step 6) — the folder map reflects it within milliseconds
 the banner                        per level: "L1 93 % · L2 98 % · L3 41 % · climbing"; `impact` on a function is declared a
                                   lower bound until L3 for its callers' files is done
```

Information also travels down and sideways: an L2 `abstained` edge (which impl of `RetryPolicy`?) is often resolved by an L3 construction site (`FixedBackoff::new()` passed into `Client`) — L3 rows may flip L2 rows to `inferred` with the L3 row as evidence; and an L3 `may_call` with three candidates narrows when another site on the same receiver resolves — the `evidence_count` refinement of §15.

**HLD is not L3.** HLD is the public-item graph (L0–L2). L2 and L3 are the two places its edges are *read from*; L3 rows themselves — the call at line 212, the field read, the await — are evidence behind an HLD edge, not HLD rows.

**How L3 is built, and which model.** Two jobs; only the second needs the heavy model.

```text
 job 1  FIND THE SITES in bodies          calls f( · a::b::f( · .f( · constructs Foo { } / Foo::new( · field .x · .await · ? · unsafe {
        tinyLLM06 "site tagger", 2–5 MB — the spanner's class. Bodies are 80–90 % of all tokens, so this is the one L3 pass that
        reads everything and it must be tiny: ~20 min in the background on an Air (§8). A tokenizer alone cannot separate a macro
        argument, a closure call, a string literal and a method call on broken code; the tagger does, with confidence.

 job 2  RESOLVE EACH SITE to a public target
        2a  deterministic (CPU-tool-05 JOIN)     path calls a::b::f(  → unique span by path
                                                 Foo::new( / Foo { }   → unique type by name
                                                 self.field.f(         → field_type from `field` → impl → f
                                                 let x = Foo::new(); x.f()   → local binding in the same body, one-pass scope
                                                 expected to clear 60–70 % of all sites with no model
        2b  tinyLLM04 BINDER (~100 MB) on the rest: receiver of unknown type · trait method with several impls · generic T.
            Input: the site, the enclosing signature, the let-bindings above it, k candidate signatures from `name`.
            Output: one candidate or abstain. For a 10 M-line tree ≈ 1.2–2.5 M sites × ~400 tokens ≈ 0.5–1 B tokens →
            Air: days in the background · Mac mini: ~a day · desktop GPU: 1–2 h — hence deferred, by region, on demand first.
        2c  teacher: rust-analyzer labels every site on compiling snapshots off-line; tinyLLM04 and tinyLLM06 are trained on
            those and scored on held-out repositories; a Rust-adapted 4B teacher adds labels on broken code.
```

The same cascade as L2 — tiny tagger over everything, SQL for the majority, the ~100 MB binder only where the join cannot type the receiver — with a much larger queue. The structure table in §9 gains one row: CPU-tool-06 BIND also calls tinyLLM06 site tagger (2–5 MB) for job 1.

## 13. Order in a Rust file — what moves, what does not

*Why a function can sit below its caller, why a `use` can sit at the bottom, and what that means for identity and the join.*

### 13.1 Items are order-independent; statements are not

```text
 ITEMS at module level                                     order does NOT matter
   fn · struct · enum · trait · impl · const · static       a fn may call one defined 500 lines below
   type · mod · use · extern crate                          an impl may precede its struct
                                                            a `use` anywhere in the file is in scope for the whole module
 STATEMENTS inside a body                                   order matters — evaluated top to bottom
   let · expressions · nested fn/use/struct in a block       a `let` precedes its use; a block-local `use` applies to that block only
 the one exception: macro_rules!                            textual — must be defined ABOVE its first use in the same file
                                                            (#[macro_use] on the module, or 2018-style paths, make it item-like again)
```

Nothing at module level runs. A Rust file is a set of declarations resolved over the whole module at once; the only thing that executes top to bottom is a function body, starting at `main`. `static` and `const` initialisers are evaluated at compile time; there is no module-initialisation code.

### 13.2 What this fixes in the map

- **Identity is the digest, never the line.** Moving a `pub fn` above another, or a `use` to the bottom, changes nothing in HLD_current — same signature, same edges, same span id; only `start_line` moves. Line numbers are display data.
- **The join may treat every `use` in a file as visible to every item in it**, regardless of position. CPU-tool-05 needs no ordering logic at L2. Block-local `use` inside a body is L3 and scoped to its block — the one case where position matters for binding.
- **The roll-up is order-free.** A file's dependencies are the union of its items' edges; there is no "what the file does first". Two `impl` blocks for one type in different files merge — the map unions them.
- **The spanner must know the macro exception.** A `macro_rules!` used above its definition in the same file is a real error; a model that treats macros as order-free would miss it. The macro's definition line is the one span where position is semantic.

### 13.3 Three ways to be surprised anyway

- `mod foo;` only declares the file; items inside `foo` still need `pub` and a `use` to be visible, wherever the `mod` line sits.
- Two `use` lines bringing the same name into one scope are an error, not an override — there is no shadowing by position at module level.
- Glob imports (`use x::*`) resolve against whatever `x` exports at compile time; the join cannot enumerate them without the target's span rows — a `use *` edge is `inferred` until the target file is indexed.

## 14. Row-count envelope, measured

*How many rows each table holds for a 10 M-line Rust tree, from densities measured on six Rust trees (2026-11-09), scaled. Low / medium / high are the minimum, middle and maximum of five trees; cargo-semver-checks is shown but excluded from the ranges as fixture-heavy (99 public types per 1,000 lines). Counts are grep proxies over non-blank, non-comment lines; good to about ±2× at the ends, which is what an envelope is for.*

### 14.1 Measured densities, per 1,000 code lines

```text
 tree                      lines   fn   pub fn  pub types  impl  use   mod   macro  doc   .method(  path(   .field  unsafe{  files
 cargo-modules             4k      55   22      14         11    36    9     0      29    242       197     125     0        9.7
 flowistry                 3.9k    33   16      10         9     52    6     0.5    70    308       140     115     0.8      8.8
 rust-code-analysis        25k     40   11      4          9     19    2.5   0.8    39    105       111     68      0.2      3.2
 rust-analyzer             435k    76   17      8.5        18    15    2.6   2.7    30    215       186     69      0.8      2.0
 linux/rust                104k    73   26      10         38    17    3.8   3.5    334   189       312     114     23       4.0
 cargo-semver-checks (x)   29k     97   55      99         21    13    2.5   1.7    44    99        197     41      0.3      23
```

Two densities swing everything: `impl` blocks (9 per 1,000 lines in rust-analyzer, 38 in the kernel's Rust — trait-heavy code roughly doubles the public span count) and call sites (210–610 per 1,000 lines — the reason L3 is lazy).

### 14.2 Rows for a 10 M-line tree

```text
 table / level                                        low      medium   high     driver
 file                                                 20k      40k      100k     2–10 files per 1k lines
 span — public items (fn, types, impl, const, macro)  250k     480k     820k     pub fn 11–26 · pub types 4–14 · impl 9–38
 span — public members (fields, variants, assoc)      120k     400k     700k     3–5 per public type (assumed, not measured)
 span — private light rows                            320k     590k     800k     fn − pub fn, private types
 span total                                           0.7 M    1.5 M    2.3 M
 name (tokens of name + path)                         2 M      7 M      14 M     3–6 tokens per span
 field (stage-4 output)                               1 M      4 M      9 M      3–6 fields per public span
 L1 evidence (use + mod rows)                         150k     200k     450k     use 13–36 · mod 2.5–9
 L1 file → file edges (distinct pairs)                80k      120k     250k     ~60 % of use lines intra-tree and distinct
 L1 folder → folder edges                             5k       10k      20k
 L2 item → item edges                                 0.4 M    1.0 M    2.0 M    2–4 type refs per pub fn + implements + reexports
 HLD_current rows (L2 edges + edgeless nodes)         0.5 M    1.2 M    2.3 M
 L3 call sites (method + path)                        2.1 M    3.9 M    6.1 M    210–610 per 1k lines
 L3 field reads / writes                              0.4 M    0.7 M    1.25 M
 L3 await · unsafe · ? · panics                       0.1 M    0.15 M   0.4 M    kernel-style code: unsafe 23 per 1k
 L3 total, if ever fully resolved                     2.6 M    4.8 M    7.8 M    10–15 % expected to sit as abstained
```

### 14.3 Bytes

```text
 file text                       ≈ 350 MB      10 M lines × ~35 bytes; the same at every estimate
 span text + doc                 0.1 / 0.3 / 0.5 GB     ≈ 300 bytes per public span including doc; kernel-style doc density doubles it
 name + field                    0.1 / 0.4 / 0.9 GB
 L2 edges + HLD tables           0.1 / 0.2 / 0.4 GB
 FTS over span text              ≈ 1–1.5 × span text
 one file, L0–L2                 ≈ 0.8 / 1.4 / 2.4 GB
 L3 fully eager, additional      + 0.3 / 0.5 / 0.9 GB
```

Scale: the Linux-sized yardstick (33 M lines) is ×3.3; rust-analyzer itself (435k lines) is ÷23 — about 20k–100k HLD rows and a database under 100 MB.

### 14.4 What the numbers settle

- HLD_current at 0.5–2.3 M rows is comfortable for recursive CTEs on a laptop; the three-table HLD design (§12) does not need a graph engine at this size.
- L3 at 3–8 M call sites is the volume of the map and the reason it is resolved lazily; eager L3 adds roughly a third to the database and most of the model time.
- The file text (350 MB) is a fixed cost that dwarfs nothing else — storing it in the same file (§2) is affordable; the span text is a fraction of it.
- The widest uncertainty is members per public type and fields per span, both assumed; the first measurement to take when the spanner exists.

### 14.5 Caveats

Grep proxies: a `fn` inside a macro body or string literal is counted; calls under `#[cfg(test)]` are included; comment lines are excluded but doc lines are counted separately; path-call counts include function definitions (a few percent). The five-tree range is narrow for fn and use density and wide for impl and doc density; the envelope reflects that.

## 15. Interesting ideas from elsewhere

*A section for ideas worth stealing from neighbouring systems, each with what it is, what transfers, and what does not. Entries are dated; nothing here is a decision.*

### 15.1 Hindsight — agent memory that learns (2026-11-09)

**What it is.** vectorize-io/hindsight: MIT, Python, created 2025-10, 60k stars, active; paper arXiv 2512.12818. An agent memory system whose point is that agents should *learn*, not just recall conversation. Storage is PostgreSQL + pgvector (or an embedded `pg0`); it needs an LLM for `retain`. Its vocabulary:

```text
 world facts     facts about the world                              "the stove gets hot"
 experiences     the agent's own                                    "I touched it and it hurt"
 observations    consolidated, evidence-backed beliefs formed from many memories — each keeps its quotes and a proof
                 count, and is REFINED rather than overwritten when new evidence arrives
 mental models   a standing answer to a question about a bank, rewritten in the background; reading one is a database
                 read — no retrieval, no LLM call at query time
 knowledge pages mental models presented as a wiki: living markdown the bank writes about itself
 retain / recall / reflect
                 recall runs four strategies in parallel — semantic, BM25, graph (entity/temporal/causal links),
                 temporal — merged by reciprocal rank fusion and a cross-encoder reranker, trimmed to a token budget
 decision models small typed-answer models that replace LLM calls for the judgments inside recall, consolidation and
                 reflect: hs-decide-1-flash 0.8B (Apache-2.0) · hs-rerank-1 114M (MIT) · hs-embed-1 112M (MIT)
 banks           isolated stores with disposition traits; memory defense scans every retain for secrets and PII
```

**What transfers to Parseltongue.**

- *A fourth logical store: the agent's experience of the map.* HLD_current is world fact (extracted), HLD_future is intent, HLD_delta is the gap. None of them holds what the agent *learned* while working: that the `auth` in `gateway/middleware` was the one that mattered, that an `abstained` call site turned out to resolve to `RetryPolicy::next_delay`, that a `to_build` row took three attempts because of a lifetime. Hindsight's world-vs-experience split says this belongs in its own store, written only by the agent, keyed to our span digests and HLD rows — never mixed into extraction. Call it MEMORY; it is the only table where prose is allowed.
- *Observations are our `inferred` edges, done properly.* An observation keeps its evidence quotes and a proof count and is refined, not overwritten. Our binder emits one `inferred` edge with one confidence; the better record is one edge with a list of evidence spans and a count that grows as more sites agree, so a second map does not restart from zero. Add `evidence_count` beside `evidence_span`.
- *Mental models are the labeller tier with the right execution model.* "What is this folder for?" is a standing question whose answer is rewritten in the background when the folder's rows change, and read as a plain row at query time. That is exactly how the optional tinyLLM05 label column should behave — never computed on the agent's critical path.
- *Four-way recall with RRF is FIND.* Our trie / trigram / BM25 plan plus graph proximity (edges from the hits) plus the snapshot dimension (temporal) merged by reciprocal rank fusion, then trimmed to a token budget — the budgeted-page option from §11, independently arrived at.
- *Decision models validate the tiny-model thesis.* A production memory system replaced its in-loop LLM judgments with 0.1–0.8B typed-answer models and published them. Their 114M reranker and 112M embedder sit exactly at our 100 MB tier; hs-rerank-1 (MIT) is a concrete candidate base for tinyLLM05 after code post-training.
- *Memory defense for the public feed.* Scan spans for secrets before they leave the machine; the pattern list is a solved problem to copy.

**What does not transfer.** The Postgres server and the LLM-in-the-loop `retain` — the map must build with no big model and no server. The biomimetic framing adds nothing. Entity extraction by LLM is what our deterministic tools and tiny models replace for code; it remains fine for the agent's own prose memories, which are the agent's business, not the map's.

**Two ways to use it, both open.**
```text
 borrow the schema   add MEMORY (agent-written, span-keyed, prose allowed) · evidence_count on edges · background-written
                     labels · RRF in FIND — all inside the one Turso file, no dependency
 run it beside       the agent keeps a Hindsight bank whose memories cite Parseltongue span ids and HLD rows; Parseltongue
                     stays map-only; the two meet at the digest. Costs a server and an LLM key on the agent's side only.
```

## 16. Span cutter — TDD log (STUB → RED → GREEN → REFACTOR)

*Rubber-duck recheck of the span claims in §9 and the chat ("a lexer cuts ~99 %, a tiny model only the grey rows"), done as a test-first experiment instead of an assertion. Oracle: `syn::parse_file` item spans (proc-macro2 span-locations) on real files; broken-code cases are hand-written fixtures plus mutated real files. Checkpoints appended after each phase.*

**Question under test.** Does a CPU lexer with brace matching and rust-analyzer's ITEM_RECOVERY_SET resync cut module-level item spans (a) exactly on code that parses, (b) usefully on broken code — and how large is the residue that would need a model?

**Facts read from source (2026-11-09).** rustc `SpanData { lo: BytePos(u32), hi: BytePos(u32), ctxt, parent }`; rust-analyzer `TextRange` over a lossless rowan tree, `LineCol { line, col: UTF-8 byte offset }` derived by a line index; `ITEM_RECOVERY_SET = { fn struct enum impl trait const async unsafe extern static let mod pub crate use macro ; }` with `error_block` and `err_and_bump` on unmatched delimiters. Decision taken for Parseltongue: span identity = (file digest, start byte, end byte, text digest); line/col derived for display only.

### Checkpoint 0 — STUB
- Crate `spancut` (local, `~/spanbench/spancut`): `cut_item_spans(&str) -> Vec<Span {kind, name, vis, start, end, confidence}>`, `line_col(&str, usize)`; bodies `todo!()`; `cargo check` green.
- Oracle binary `truth` (`~/spanbench/truth`): prints syn item kind, name, visibility, start/end line:col for any parseable file.
- Tests written (RED next), four-word names: `test_cut_two_items_one_line` · `test_cut_brace_inside_string` · `test_attach_attrs_docs_above` · `test_resync_after_missing_brace` · `test_grey_half_typed_tail` · `test_macro_rules_braces_not_block` · `test_nested_items_not_module_level` · `test_cfg_duplicate_names_two_spans` · `test_visibility_variants_classified` · `test_line_col_derived_only`.
- Decision: byte offsets in the API; attributes and doc comments belong to the item span; confidence < 0.5 marks a grey span (unterminated, resynced, or truncated).
- Next: run tests (must all fail on `todo!()`), then GREEN with the minimum lexer; then the real-file comparison against `truth` on rust-analyzer, cargo-modules and linux/rust, then mutation runs (delete `}`, truncate, inject half item) with the failing snippets recorded here.
- Blocker/question: none. A first Python prototype of the cutter mis-handled `;` termination (emitted an ERROR span for the semicolon) — recorded as the first real bug class: terminator handling, not brace matching.

### Checkpoint 1 — RED
- `cargo test` on the stub: `test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` — every test panics on `todo!()`, as required; none passes without an implementation.
- Status: 10 tests failing (RED) · 0 passing · implementation pending.
- Next: GREEN — the minimum lexer (strings, chars vs lifetimes, raw strings, nested block comments, attributes/docs attached to the following item, brace matching, `;` termination, ITEM_RECOVERY_SET resync with confidence 0.3 on grey spans), then the oracle comparison on real files.
