# Deep Insight 08

*Names follow the four-word rule in AGENTS.md.*

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


Decision (evidence: DI-07 §16): one parse per file with `ra_ap_syntax` (pinned library, not the rust-analyzer server) does the cutting; models leave steps 1–3 entirely; the only model-shaped step left is binding (step 5), and even there pure name-join is measured first.

```text
 step  what happens                                                        who does it → writes                                 inspiration crates / repos                           use
 ───── ──────────────────────────────────────────────────────────────────── ──────────────────────────────────────────────────── ──────────────────────────────────────────────────── ───────────
 0     tree walked, bytes stored VERBATIM (no CRLF/whitespace/encoding      cpu-tool-ingest-source-files                         tursodatabase/agentfs (files as rows)                ○ pattern
       change), digest per file; unchanged digests skipped                  → file-source-text-table                             turso / libsql crate (embedded DB)                   ✔ as is
                                                                                                                                 ignore crate (ripgrep's .gitignore walker)           ✔ as is
                                                                                                                                 blake3 crate (file + span digests)                   ✔ as is
 1     ONE PARSE PER FILE: module-level items → byte range, kind, name,     cpu-tool-syntax-tree-walker                          ra-ap-rustc_lexer (= rustc's own lexer)              ✔ as is
       visibility, conf 1.0; nested items kept under a parent id; tree      → span-reference-text-table                          ra_ap_parser + ra_ap_syntax (lossless tree, ranges)  ✔ as is, pinned
       dropped after the file. Files whose parse reports errors →           → tiny-llm-span-clarifier (grey rows only,    syn 2 (oracle in tests; fallback parser)             ✔ as is
       indentation-recovery re-cut; those spans conf < 1 = grey               background)                                        rust-analyzer ITEM_RECOVERY_SET / error_block        ○ pattern
                                                                                                                                 experiments/spancut (indentation recovery rule)      ✔ ours
 2     same pass: params, returns, generics, bounds, struct fields, trait   cpu-tool-syntax-tree-walker (no model)               rustdoc JSON (which fields a public item has)        ○ schema idea
       items, use trees, cfg/attrs, docs; name tokens for FTS               → field-signature-detail-table                       SQLite FTS5 / Turso FTS                              ✔ as is
                                                                            → name-token-index-table                             cargo-modules (what counts as a module item)         ○ pattern
 3     same pass: body sites — calls, method calls, constructions, field    cpu-tool-syntax-tree-walker (no model)               ra_ap_syntax ast::CallExpr / MethodCallExpr /        ✔ as is
       reads/writes, .await, unsafe, panic!/? — written as PENDING L3 rows  → edge-evidence-record-table (status pending)        FieldExpr / AwaitExpr / BlockExpr(unsafe)
       pointing at span + byte range; nothing resolved yet                                                                      rust-code-analysis (Mozilla; per-fn metrics)         ○ pattern
                                                                                                                                 cargo-geiger (unsafe counting)                       ○ pattern
 4     names joined to spans via use trees + paths: unique → extracted,     cpu-tool-name-join-edges                             SCIP (Sourcegraph symbol naming scheme)              ○ id format
       several → candidates, none → unresolved; JOIN RATE published —       → edge-evidence-record-table                         github/stack-graphs (name resolution w/o compiler)   ○ pattern
       the number that decides how much binding is needed                                                                       rust-analyzer hir-def nameres                        ○ offline teacher only
 5     only candidate rows go to the binder (CPU resolver first); macro     cpu-tool-bind-resolve-edges                          rust-analyzer (labels to train the binder)           ✔ offline teacher
       invocations at item position go to the guesser; background after    → tiny-llm-edge-binder                   cargo-expand (macro truth for offline labels)        ✔ offline teacher
       the first map; abstain when evidence is missing                     → tiny-llm-macro-guesser                      Qwen2.5-Coder small / Cassini-class encoders         ○ teacher / base
                                                                            → edge-evidence-record-table
 6     public→public edges collapsed through private chains → HLD rows;     cpu-tool-hld-table-builder                           cargo-modules dependency graph                       ○ pattern
       roll-ups by trigger                                                  → hld-current-dependency-edges,                      RustPräzi (ecosystem call graph)                     ○ pattern
                                                                              hld-file/folder-level-rollup                       cargo-semver-checks (what "public" means)            ○ pattern
                                                                                                                                 SQL recursive CTEs (Turso/SQLite)                    ✔ as is
 7     coverage per level + parser-error count + grey-span count +          cpu-tool-status-coverage-banner                      —                                                    ours
       join rate + unresolved/binding backlog published                     → per-level-coverage-banner
 ─────   first map: parse ≈ 1 min for 10 M lines (measured 789 klines/s/core, DI-07 §16 cp 8); DB write and binding are the long poles;
       budget stays ≤ 30 min on an 8 GB Air but is no longer spent reading tokens
 8     agent asks: where is X · what breaks if X changes                    ext-llm-coding-agent                                 Aider repo-map (ranked context for an LLM)           ○ pattern
       answered from the tables                                             cpu-tool-find-search-query, cpu-tool-impact-graph-   DeusData/codebase-memory-mcp (graph + MCP)           ○ pattern; ✔ dev tool
                                                                            walk                                                 Flowistry (information flow: which way to walk)      ○ pattern
 9     intent written as rows: new node, new edge, retire edge              ext-llm-coding-agent or human-architect-intent-      github/spec-kit, AWS Kiro (spec → code)              ○ pattern
                                                                            author via cpu-tool-propose-future-rows              MCP (the one CLI/protocol face to the agent)         ✔ as is
                                                                            → hld-future-dependency-edges
 10    gap read: to_build rows, each with file, folder, neighbours, sig     ext-llm-coding-agent (reads hld-delta-dependency-    —                                                    —
                                                                            view)
 11    code written for exactly those rows                                  ext-llm-coding-agent → the working tree              Claude Code / Codex / Aider                          ✔ external
 12    changed files re-run steps 0–7 for those files only                  cpu tools (+ models on grey/candidate rows)          salsa (rust-analyzer's incremental engine)           ○ pattern (we use digests)
                                                                            → all extraction tables                              agentfs change tracking                              ○ pattern
 13    matching rows appear in hld-current → delta flips to realized        hld-delta-dependency-view (by definition)            —                                                    —
 14    review: rows of intent vs rows of fact                               human-architect-intent-author                        —                                                    —
 ───── always in background: L3 bind + HLD re-derive per region, grey clarifier, macro guesser, re-map on edit;
       offline-teacher-rust-analyzer (rust-analyzer, rustc, cargo-expand) and offline-judge-heldout-evaluation (syn oracle, expect-test) train and
       score the two surviving tiny models — never at runtime
```

### Legend for the inspiration column

```text
 ✔ as is           a crate or tool we take unchanged. At runtime this is always a LIBRARY linked into the Parseltongue binary
                   (ra_ap_syntax, ra-ap-rustc_lexer, turso, blake3, ignore, SQLite FTS5) — never a server, build loop or compiler.
                   Example: ra_ap_syntax parses the file; we call it, we do not re-implement it.
 ✔ as is, pinned   same, but version-locked; bumped only when the §16 oracle suite stays green (ra_ap_* releases weekly, 0.0.x API).
 ✔ offline teacher a tool we RUN, never ship: it produces labels or ground truth for training/evaluation on our machines.
                   Example: rust-analyzer resolves a call site so the binder has a correct answer to learn from.
 ✔ ours            Parseltongue code, written and tested in this repo. Example: the indentation-recovery rule from experiments/spancut.
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
 retired   tiny-llm-span-boundary-tagger    boundaries come from the parser (DI-07 §16: 376/376 on real files)
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
 NOT called              any span, field, name token, use tree or body site on a file that parses clean — the parser is exact there (§16: 376/376);
                         any unique name join; any HLD derivation or roll-up; anything in the future/delta tables
```

Rules that hold for all four: text in, labels + confidence out; never write a table (a CPU tool writes the row with the label); always allowed to abstain; every output carries status `inferred` so hld-delta-dependency-view and the human reviewer can see which rows came from a model.

Name note: the four-word rule forces two-word descriptors; "grey" and "candidate" moved from the names into the "called when" column.

Roles in one line each:
- **ext-llm-coding-agent** — reads the product, writes intent and code; never writes facts.
- **human-architect-intent-author** — writes intent, reviews rows; never writes facts.
- **CPU tools** — move bytes, run the parser, run SQL, derive tables; decide nothing.
- **tiny models** — make the judgments grammar cannot: grey boundaries, macro-defined items, ambiguous binding; see text, return labels; never touch a table.
- **offline teacher and judge** — rust-analyzer/rustc/cargo-expand label, syn-oracle scores; absent at runtime.

### Three journeys through the workflow

**Journey A — the ordinary case (≈ 98 % of items): `pub fn send` calls `RetryPolicy::next_delay` through a private helper.** Measured path (DI-07 §16).
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

**Journey B — the broken file: a `}` deleted while the agent is mid-edit.** Measured path (DI-07 §16 cp 5, cp 7).
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
