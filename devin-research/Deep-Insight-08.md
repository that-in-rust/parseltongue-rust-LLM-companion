# Deep Insight 08

*Every named thing carries a four-word hyphenated name that describes its contents.*

1. **hld-current-dependency-edges is the product.**
2. **It is read on every task and rebuilt only where the tree changed.**
   - One row = a public item depending on a public item: node01 (signature, file, folder, kind, span id) · node02 (same) · edge type · status · confidence [min, max] · evidence count · snapshot. No bodies, no private items — those are evidence behind a row, in edge-evidence-record-table and span-reference-text-table.
   - Raw code is not in the row. It lives once in span-reference-text-table, keyed by digest, where full-text search also lives; search starts there and lands in HLD rows by one join.
   - **Written only by extraction** — cpu-tool-hld-table-builder deriving from edge and span rows that the parser pass (cpu-tool-syntax-tree-walker) and the two remaining tiny models produced. Never by ext-llm-coding-agent, never by human-architect-intent-author.
   - **hld-future-dependency-edges** holds the intended design. Written only by ext-llm-coding-agent or human-architect-intent-author through cpu-tool-propose-future-rows, which validates and never authors. Same row shape, status `proposed`, plus creator and reason.
   - **hld-delta-dependency-view** is the gap: a full outer join of the two. Nobody writes it. States: to_build · to_remove · realized · unplanned.
   - **hld-file-level-rollup** and **hld-folder-level-rollup** are counters (weight, kinds, confidence range) updated by trigger on every HLD row — the folder map is never stale relative to the item map.
   - **agent-learned-notes-table** is the only place prose is allowed: what ext-llm-coding-agent learned about the map, keyed to span ids. Agent-written; never read by extraction.

## End-to-end workflow, with roles

### The journey in one line per step

```text
 0   codebase  →  files           the working tree is walked; every .rs file's bytes are stored unchanged as one row in Turso, with a digest;
                                  a file whose digest did not change is skipped
 1   file      →  spans           each file is parsed once (ra_ap_syntax); every module-level item becomes a span row: byte start, byte end, kind,
                                  name, visibility; a file with parse errors is re-cut by the recovery rule and its doubtful spans are marked grey
 2   span      →  fields + names  from the same tree: params, return, generics, bounds, struct fields, trait items, use trees, cfg/attrs, docs;
                                  names are split into tokens and indexed for full-text search
 3   span      →  body sites      from the same tree: every call, method call, construction, field read/write, .await, unsafe block, panic!/? inside
                                  a body becomes a pending L3 row pointing at its byte range — found, not yet resolved
 4   name      →  target          each site/field name is joined to span rows through use trees and paths: one match = extracted edge,
                                  several = candidate list, none = unresolved; the join rate is published
 5   candidates →  one target     the CPU resolver tries first; only still-ambiguous candidates go to the tiny binder; macro invocations at item
                                  position go to the macro guesser; no evidence = abstain, never guess
 6   edges     →  HLD             public → public edges are derived by collapsing private chains; file and folder roll-ups update by trigger
 7   tables    →  banner          coverage per level, parser-error count, grey-span count, join rate and unresolved backlog are published
 ─── first map done; everything below is the daily loop
 8   agent asks                   "where is X", "what breaks if X changes" — answered by search and graph walk over the tables
 9   agent/human writes intent    new node, new edge, retire edge → rows in hld-future-dependency-edges (validated, never auto-created from code)
 10  agent reads the gap          hld-delta-dependency-view lists to_build rows with file, folder, neighbours and signature
 11  agent writes code            exactly for those rows, in the working tree
 12  changed files re-mapped      steps 0–7 run again for the changed files only (digest says which)
 13  delta flips                  a future row whose fact now exists in hld-current becomes realized — by definition, nobody edits it
 14  human reviews                rows of intent against rows of fact
```


Decision (measured in the span-cutter experiment — 376/376 items byte-identical to the syn oracle on 125 real files; 789 k lines/s per core): one parse per file with `ra_ap_syntax` (pinned library, not the rust-analyzer server) does the cutting; models leave steps 1–3 entirely; the only model-shaped step left is binding (step 5), and even there pure name-join is measured first.

```text
 step  what happens                                           who does it → writes                         inspiration (use)                              tiny LLM in this step · when            tiny-LLM estimate, 10 M lines, 8 GB Air · in the 30 min?
 ───── ───────────────────────────────────────────────────── ──────────────────────────────────────────── ────────────────────────────────────────────── ─────────────────────────────────────── ────────────────────────────────────────────────────────
 0     tree walked; file bytes stored VERBATIM (no CRLF/      cpu-tool-ingest-source-files                 agentfs: files as rows (○ pattern)             none                                    0 model time. CPU: read 350 MB + digests ≈ 1–5 min · IN
       whitespace/encoding change); digest per file;          → file-source-text-table                     turso / libsql (✔ as is)
       unchanged digests skipped                                                                           ignore walker, blake3 (✔ as is)
 1     ONE PARSE PER FILE: module-level items → byte range,   cpu-tool-syntax-tree-walker                  ra-ap-rustc_lexer (✔ as is)                    tiny-llm-span-clarifier · only on a     ≤ 2 % of 40k files broken → 1–2k grey spans × ~300 tokens
       kind, name, visibility, conf 1.0; nested items under   → span-reference-text-table                  ra_ap_parser + ra_ap_syntax (✔ as is, pinned)  file whose parse reported errors AND   ≈ 0.5 M tokens; 30–100 M model ≈ 1–2 min GPU / 5 min CPU ·
       a parent id; tree dropped per file. Parse errors →     → tiny-llm-span-clarifier (grey rows,        syn 2: oracle in tests, fallback (✔ as is)     the recovery rule left conf < 1;        BACKGROUND. First map ships them grey. Parse itself:
       indentation-recovery re-cut; those spans conf < 1       background)                                 rust-analyzer resync set (○ pattern)           never on a clean file                   ≈ 13 s/core measured, ≈ 1 min with row writing · IN
                                                                                                           our indentation recovery rule (✔ ours)
 2     same pass: params, returns, generics, bounds, struct    cpu-tool-syntax-tree-walker (no model)       rustdoc JSON (○ schema idea)                   none — fields come from the tree        0 model time. Rows: 1.5 M spans, 4 M fields, 7 M name
       fields, trait items, use trees, cfg/attrs, docs;        → field-signature-detail-table               SQLite FTS5 / Turso FTS (✔ as is)                                                      tokens; DB write + FTS ≈ 5–10 min (shared with 6) · IN
       name tokens for FTS                                     → name-token-index-table                     cargo-modules (○ pattern)
 3     same pass: body sites — calls, method calls,            cpu-tool-syntax-tree-walker (no model)       ra_ap_syntax ast::CallExpr / MethodCallExpr /  none — sites come from the tree         0 model time. 3.9 M call sites + 0.9 M field/await/
       constructions, field reads/writes, .await, unsafe,      → edge-evidence-record-table                 FieldExpr / AwaitExpr (✔ as is)                                                        unsafe/? rows written pending, same pass · IN
       panic!/? → PENDING L3 rows with byte ranges              (status pending)                            rust-code-analysis, cargo-geiger (○ pattern)
 4     names joined to spans via use trees + paths: unique →  cpu-tool-name-join-edges                     SCIP symbol strings (○ id format)              none — pure SQL; publishes the JOIN     0 model time. Indexed joins over 1.0 M L2 edges ≈ 2–5 min ·
       extracted, several → candidates, none → unresolved;     → edge-evidence-record-table                 stack-graphs (○ pattern)                       RATE, the number that sizes step 5      IN. Join rate is the one unmeasured input to every
       join rate published                                                                                 rust-analyzer nameres (offline teacher only)                                           estimate below.
 5     CPU resolver first (use trees, field/receiver types);   cpu-tool-bind-resolve-edges                  rust-analyzer resolution as labels             tiny-llm-edge-binder · only rows with   at 20 % ambiguous: L2 200k rows × 400 tokens ≈ 80 M tokens,
       only still-ambiguous candidates → binder; macro         → tiny-llm-edge-binder                       (✔ offline teacher)                            SEVERAL candidates after the CPU        100 M model ≈ 15–45 min GPU; L3 0.8 M rows ≈ 1.5 h by region.
       invocations at item position → guesser; abstain when    → tiny-llm-macro-guesser                     cargo-expand as labels (✔ offline teacher)     resolver fails                          Guesser: 5–15k macro calls ≈ 1–5 min.
       evidence is missing                                     → edge-evidence-record-table                 Qwen-Coder small / Cassini (○ teacher/base)    tiny-llm-macro-guesser · item-position  Both BACKGROUND after the first map; one model in RAM
                                                                                                                                                          macro call not in the known-macro table at a time (< 0.5 GB)
 6     public → public edges derived by collapsing private     cpu-tool-hld-table-builder                   cargo-modules graph, RustPräzi,                none — by derivation                    0 model time. 1.2 M HLD rows + roll-up triggers, inside
       chains; file/folder roll-ups by trigger                 → hld-current-dependency-edges,              cargo-semver-checks (○ pattern)                                                        the 5–10 min DB budget · IN
                                                               hld-file/folder-level-rollup                SQL recursive CTEs (✔ as is)
 7     coverage per level + parser-error count + grey-span     cpu-tool-status-coverage-banner              —                                              none                                    seconds · IN. Banner shows what the background models still
       count + join rate + unresolved backlog published        → per-level-coverage-banner                                                                                                        owe: grey spans, candidate rows, macro calls
 ───── first map: CPU critical path ≈ 10–20 min for 10 M lines (read 1–5 · parse 1 · DB/FTS 5–10 · join 2–5) → headroom for the ×3.3 Linux-sized tree; zero tiny-LLM time inside it.
       Then, in background: clarifier 1–5 min · guesser 1–5 min · binder L2 15–45 min · binder L3 ≈ 1.5 h by region. Only parser throughput and the oracle are measured; model numbers are estimates.
 8     agent asks: where is X · what breaks if X changes        ext-llm-coding-agent                         Aider repo-map (○ pattern)                     tiny-llm-result-ranker · optional, only 20–50 rows × 100 tokens ≈ 2k tokens per query → ms · n/a
       answered from the tables                                cpu-tool-find-search-query,                  codebase-memory-mcp (○ pattern; ✔ dev tool)    when find returns more rows than the
                                                               cpu-tool-impact-graph-walk                   Flowistry: which way to walk (○ pattern)       agent's budget
 9     intent written as rows: new node, new edge, retire edge ext-llm-coding-agent or human-architect-     spec-kit, Kiro: spec → code (○ pattern)        none                                    —
                                                               intent-author via cpu-tool-propose-future-   MCP: the one CLI face (✔ as is)
                                                               rows → hld-future-dependency-edges
 10    gap read: to_build rows, each with file, folder,        ext-llm-coding-agent (reads                  —                                              none                                    —
       neighbours, sig                                         hld-delta-dependency-view)
 11    code written for exactly those rows                     ext-llm-coding-agent → the working tree      Claude Code / Codex / Aider (✔ external)       none                                    —
 12    changed files re-run steps 0–7 for those files only     cpu tools (+ models on grey/candidate rows)  salsa (○ pattern; we use digests)              same three models, same triggers,       per changed file: parse ms · join ms · binder seconds
                                                               → all extraction tables                      agentfs change tracking (○ pattern)            only on the changed files
 13    matching rows appear in hld-current → delta flips to    hld-delta-dependency-view (by definition)    —                                              none                                    —
       realized
 14    review: rows of intent vs rows of fact                  human-architect-intent-author                —                                              none                                    human corrections become binder training rows
 ───── always in background: L3 bind + HLD re-derive per region, grey clarifier, macro guesser, re-map on edit; offline-teacher-rust-analyzer (rust-analyzer, rustc, cargo-expand)
       and offline-judge-heldout-evaluation (syn oracle, expect-test) train and score the tiny LLMs — never at runtime
```

### Legend for the inspiration column

```text
 ✔ as is           a crate or tool we take unchanged. At runtime this is always a LIBRARY linked into the Parseltongue binary
                   (ra_ap_syntax, ra-ap-rustc_lexer, turso, blake3, ignore, SQLite FTS5) — never a server, build loop or compiler.
                   Example: ra_ap_syntax parses the file; we call it, we do not re-implement it.
 ✔ as is, pinned   same, but version-locked; bumped only when the syn-oracle test suite stays green (ra_ap_* releases weekly, 0.0.x API).
 ✔ offline teacher a tool we RUN, never ship: it produces labels or ground truth for training/evaluation on our machines.
                   Example: rust-analyzer resolves a call site so the binder has a correct answer to learn from.
 ✔ ours            Parseltongue code, written and tested in this repo. Example: the indentation-recovery rule from the span-cutter experiment.
 ✔ external        the thing on the other side of the CLI; we talk to it, we do not contain it. Example: Claude Code / Codex as ext-llm-coding-agent.
 ✔ dev tool        used by humans/agents while developing Parseltongue; not part of the shipped runtime. Example: codebase-memory-mcp.
 ○ pattern         an IDEA we copy, no code dependency. We read how the project does it and write our own version.
                   Example: AgentFS stores files as rows in SQLite → our file-source-text-table does the same; no AgentFS crate is linked.
 ○ id format       a naming/encoding convention we adopt for compatibility. Example: SCIP symbol strings as the shape of our item ids.
 ○ schema idea     field names and shapes borrowed from a spec. Example: rustdoc JSON tells us which fields a public item has.
 —                 nothing external; plain Parseltongue logic or a by-definition view.
```

Reading rule for the inspiration column: anything ✔ at runtime is a *library* (parser, lexer, DB, FTS, hashing, walker) — never a server, a build loop, or a compiler. Everything that needs a compiler or a whole-program database is ✔ only in the offline lanes.

### Names changed by this revision (first draft → this version)

```text
 new       cpu-tool-syntax-tree-walker        one ra_ap_syntax pass per file → span, field, name-token and pending L3 rows (steps 1–3)
 new       tiny-llm-macro-guesser      macro invocation at item position → which item(s) it defines; status inferred
 retired   tiny-llm-span-boundary-tagger    boundaries come from the parser (376/376 on real files)
 retired   tiny-llm-field-detail-tagger     fields come from the tree
 retired   tiny-llm-body-site-tagger        sites come from the tree
 retired   cpu-tool-span-cutting-driver       folded into the walker
 retired   cpu-tool-text-copy-indexer         folded into the walker
 retired   cpu-tool-field-extraction-driver   folded into the walker
 kept      tiny-llm-span-clarifier     now only for spans the recovery rule marks conf < 1
 kept      tiny-llm-edge-binder   now gated behind the measured join rate
```

### When a tiny LLM is called — and when it is not

```text
 name                     called when                                   input                              output                      if it abstains                  share of work
 ──────────────────────── ───────────────────────────────────────────── ────────────────────────────────── ─────────────────────────── ─────────────────────────────── ─────────────────
 tiny-llm-span-clarifier  a file's parse reported errors AND the         the grey span's bytes plus the     confirmed/moved end byte,   span stays grey (conf ≤ 0.3);   ≈ 1 span per
                          recovery rule produced a span with conf < 1    lines around it                    kind, name, visibility,     its edges stay abstained;       broken file;
                          (step 1). Never on a file that parses clean.                                      new confidence              counted in the banner           0.01–0.5 % of spans
 tiny-llm-edge-binder     a site/field name joined to SEVERAL candidate  the site's span text, the          one target span id + conf,  row stays pending with its      measured next:
                          spans and the CPU resolver (use trees, field   candidate signatures, the          or abstain                  candidate list; never a         1 − join rate
                          types, receiver types) could not pick one      enclosing item's use tree                                      guessed edge
                          (step 5). Never on unique joins.
 tiny-llm-macro-guesser   a macro invocation sits at item position       the invocation bytes + the         zero or more proposed       span stays kind macro_call;     ≈ share of
                          (`foo! { … }` / `foo!(…);` at module or impl   macro_rules!/proc-macro span       items (kind, name, vis)     no items claimed; counted       item-position
                          level) and is not in the known-macro table     if found by name                   with status inferred        in the banner                   macro calls
 tiny-llm-result-ranker   optional; only when `find` returns more rows   query + candidate rows             reordered rows              CPU BM25 order is used          0 at first map
                          than the agent's budget
 NOT called              any span, field, name token, use tree or body site on a file that parses clean — the parser is exact there (376/376 measured);
                         any unique name join; any HLD derivation or roll-up; anything in the future/delta tables
```

### Training data, processing time and the 30-minute / 8 GB Air budget, per tiny LLM

Numbers below are for a 10 M-line tree, medium estimate (40k files · 1.5 M spans · 1.0 M L2 edges · 3.9 M L3 call sites); the Linux-sized yardstick (33 M lines) is ×3.3. **M** = measured in the span-cutter experiment; everything else is an estimate from the cost rule (encoder cost ≈ 2 · params · tokens; Air ≈ 1.1–1.7e12 FLOP/s GPU, ≈ 4e11 FLOP/s CPU). Training is always offline on a desktop GPU or rented box — never on the Air, never at runtime.

```text
 name                     training INPUT dataset (what the model sees)           training OUTPUT / label set (who produces it)          how much data · time to build it               runtime volume on the 10 M-line tree · time on the Air          where it sits in the 30 min
 ──────────────────────── ────────────────────────────────────────────────────── ────────────────────────────────────────────────────── ────────────────────────────────────────────── ─────────────────────────────────────────────────────────────── ─────────────────────────────
 tiny-llm-span-clarifier  grey span bytes + ~20 lines of context, taken from     the TRUE item boundaries, kind, name, visibility of     synthetic: 1 M-line parseable corpus ×         ≤ 2 % of 40k files have parse errors ≈ 800 files × 1–2 grey     NOT in the 30 min. First map
                          DELIBERATELY BROKEN copies of parseable files          the same file BEFORE it was broken — from syn /         5 mutation kinds × several sites per file      spans ≈ 1–2k spans × ~300 tokens ≈ 0.5 M tokens.                 writes them grey (conf ≤ 0.3);
                          (delete `}`, truncate, half-typed item,     ra_ap_syntax on the unbroken file. Fully automatic,    → 0.5–1 M labelled examples; generation        30–100 M model → ~1e14 FLOP → 1–2 min GPU / ~5 min CPU.          clarifier runs in background
                          delete `;`, stray `{`) + the recovery rule's own       no human, no compiler run. (M: the oracle exists        ≈ minutes (parser at 789 klines/s, M).         Per broken file: milliseconds.                                     and flips rows to extracted.
                          grey span as the "question"                            today, 376/376.)                                       Training: ~3e8 tokens × 3 epochs on a 30–100 M
                                                                                                                                        encoder ≈ 2–6e17 FLOP ≈ 1–3 h desktop GPU.
 tiny-llm-edge-binder     a call/field site's text + the enclosing item's use    the ONE span the name resolves to — from the           real: every ambiguous site in the open-source  L2: 1.0 M edges × (1 − join rate). At 20 % ambiguous (to be      NOT in the 30 min. First map
                          tree + the 2–8 candidate spans' signatures, from REAL  offline teacher (rust-analyzer go-to-definition /      corpus (this VM: 3,370 files, 1 M lines →      measured, step 4) ≈ 200k rows × ~400 tokens ≈ 80 M tokens;        publishes unique joins
                          files in the corpus, only where name-join returned     rustc HIR), run once per corpus on a desktop. Human    ≈ 400k call sites, ~80k ambiguous). Teacher    100 M model → 1.6e16 FLOP ≈ 15–45 min on the Air GPU.            (extracted) and candidate
                          several candidates                                     correction rows from step 14 are added over time.      pass: RA load + per-site resolve ≈ 1 h per     L3: 3.9 M sites, ~0.8 M ambiguous ≈ 320 M tokens → 6e16 ≈ 1.5 h  lists; binder drains L2 then
                                                                                                                                        1 M lines (estimate). Training: 100 M encoder, background; per region on demand: seconds.                       L3 in the background (the
                                                                                                                                        ~1.6e8 tokens × 3 epochs ≈ 3e17 ≈ 2 h GPU.                                                                      "L3 drain").
 tiny-llm-macro-guesser   an item-position macro invocation's bytes + the        the items the macro actually produces (kind, name,     real: top ~1,000 crates.io crates + kernel     1–3 % of 480k public spans are item-position macro calls ≈        NOT in the 30 min. First map
                          macro_rules!/proc-macro definition's bytes when it is  visibility, impl-of) — from cargo-expand (offline       Rust; cargo-expand needs a BUILD → 1–5 min     5–15k × ~500 tokens ≈ 2.5–7.5 M tokens; 100 M model → ~1e15      stores the span as kind
                          in the tree, from real crates                          teacher) on crates that compile, diffed against the    per crate, 1–3 CPU-days total, parallel;       FLOP ≈ 1 min GPU / ~5 min CPU. Known-macro table (hand-written    macro_call; guesser adds
                                                                                 un-expanded tree. Automatic for compiling crates.      ≈ 50–200k examples. Training ≈ 1 h GPU.        for std/kernel macros) removes most of these before the model.    inferred items in background.
 tiny-llm-result-ranker   a `find` query + the 20–50 candidate rows (names,       the row the agent actually used — from                 starts empty; grows with use; thousands of     per query: ~20 rows × 100 tokens ≈ 2k tokens → milliseconds.      not part of the map at all;
                          signatures, doc first line, BM25 score)                agent-learned-notes-table and step-14 review clicks.   examples before it is worth training.                                                                           per-query, optional.
```

How the 30 minutes are spent, given the above (10 M lines; Linux-sized ×3.3 still fits):

```text
 inside the 30 min — CPU only, zero tiny-LLM time on the critical path
   step 0  read 350 MB + digests                                ≈ 1–5 min   (disk-bound)
   step 1–3 one ra_ap_syntax pass, rows out                      ≈ 1 min    (M: 789 klines/s/core → 13 s/core; 4 cores ≈ 5 s parse, the rest is row writing)
   step 2/6 DB write + FTS + roll-up triggers                    ≈ 5–10 min (1.5 M spans, 7 M name tokens, 1.0 M edges ≈ 1 GB)
   step 4  name-join SQL                                         ≈ 2–5 min  (indexed joins over use trees × names)
   step 7  banner                                                seconds
   total                                                         ≈ 10–20 min → headroom for the ×3.3 Linux-sized tree
 after the 30 min — one model loaded at a time (≤ 100 MB int8 → < 0.5 GB RAM; parser and DB stay under ~2 GB; fits 8 GB)
   clarifier   ≈ 1–5 min      macro guesser ≈ 1–5 min      binder L2 ≈ 15–45 min      binder L3 ≈ 1.5 h, by region on demand
 second map (edit loop)   only changed files re-enter; per file: parse ms, join ms, binder seconds
```

What changed versus the first budget: the 2–5 MB "every token" tagger (20–25 min of the 30) is gone — the parser does that work in ~1 minute — so the critical path is now disk + database, and every tiny LLM is a background worker whose cost is proportional to the *ambiguous* fraction, not to the tree.

Rules that hold for all four: text in, labels + confidence out; never write a table (a CPU tool writes the row with the label); always allowed to abstain; every output carries status `inferred` so hld-delta-dependency-view and the human reviewer can see which rows came from a model.

Name note: the four-word rule forces two-word descriptors; "grey" and "candidate" moved from the names into the "called when" column.

Roles in one line each:
- **ext-llm-coding-agent** — reads the product, writes intent and code; never writes facts.
- **human-architect-intent-author** — writes intent, reviews rows; never writes facts.
- **CPU tools** — move bytes, run the parser, run SQL, derive tables; decide nothing.
- **tiny models** — make the judgments grammar cannot: grey boundaries, macro-defined items, ambiguous binding; see text, return labels; never touch a table.
- **offline teacher and judge** — rust-analyzer/rustc/cargo-expand label, syn-oracle scores; absent at runtime.

### Three journeys through the workflow

**Journey A — the ordinary case (≈ 98 % of items): `pub fn send` calls `RetryPolicy::next_delay` through a private helper.** Measured path.
```text
 0  http/client.rs → bytes, digest d1
 1  tree → span #41 [bytes 1180..2630) fn send pub conf 1.0; span #44 fn backoff priv (parent: module)
 2  fields: send(req: Request) -> Response; use crate::retry::RetryPolicy; doc "Sends one request…"
 3  sites: send → backoff() [byte 1402]; backoff → self.policy.next_delay() [byte 2210]      status pending
 4  Request, Response, RetryPolicy: one span each in the tree → extracted, conf 1.0
    next_delay: TWO spans named next_delay (RetryPolicy::, Backoff::) → candidates
 5  binder sees receiver type RetryPolicy from the use + field → RetryPolicy::next_delay, inferred 0.9
 6  send → RetryPolicy::next_delay  may_call  inferred [0.9]  evidence: 2 L3 rows through private backoff
 7  banner: L2 100 %, L3 1 of 2 sites bound at first map, rest draining
 8  agent: "what breaks if RetryPolicy changes?" → walk → send, and its callers
 9  agent writes future row: send → Metrics::record (calls, proposed)       → delta: to_build
 11 agent edits send's body; 12 client.rs d1→d2, steps 1–6 for that file only; 13 realized; 14 human sees intent = fact
```

**Journey B — the broken file: a `}` deleted while the agent is mid-edit.** Measured path (oracle and mutation runs).
```text
 0  d3 stored verbatim (the file is broken; bytes don't care)
 1  parse reports 1 error; the ra_ap tree alone would nest every later item inside the open fn → recovery re-cut fires:
    `pub fn g` first on its line at column 0 ends the open item → fn f [..]  conf 0.3 (grey); g, S clean conf 1.0
    grey row queued → tiny-llm-span-clarifier later confirms/adjusts f's end
 2–3 fields and sites for g and S exact; f's sites written but flagged from a grey span
 4–6 edges from g and S flow normally; HLD rows involving f carry conf ≤ 0.3 and status abstained
 7  banner shows 1 grey span in this file; nothing is lost, nothing is silently wrong
 12 next save fixes the brace → d4 → f re-cut clean → its rows flip to extracted
```

**Journey C — what a parser cannot see: an item made by a macro, and a name with no unique target.** Design for the unmeasured slice.
```text
 1  kernel/platform.rs: `kernel::impl_device_context_deref!(unsafe { Device });` at item position → span kind macro_call conf 0.9
    (a parser knows it is a macro call; it does not know it DEFINES a Deref impl)
 5  tiny-llm-macro-guesser reads the span + the macro's own span (found by name) → proposes "impl Deref for Device", inferred 0.7
 3–4 elsewhere, site `ctx.lock()` → candidates: Mutex::lock, SpinLock::lock, RwLock::lock → no receiver type visible at the site
 5  binder: use-tree (`use crate::sync::SpinLock`) + field decl `ctx: SpinLock<…>` → SpinLock::lock inferred 0.85;
    if evidence is missing it ABSTAINS → row stays pending, counted in the banner, never guessed silently
 6  HLD gains Device → Deref (inferred) and Driver::probe → SpinLock::lock (inferred); both carry their confidence into the delta
 14 the human sees exactly which rows are inferred and can confirm or correct — that correction becomes training data for the judge
```

Open question carried forward (replaces "span boundaries on broken code"): **how good is pure name-join (step 4) before any model?** That is the next measurement.
