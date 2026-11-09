# Deep Insight 08

*Names follow the four-word rule in AGENTS.md.*

1. **hld-current-dependency-edges is the product.**
2. **It is read on every task and rebuilt only where the tree changed.**
   - One row = a public item depending on a public item: node01 (signature, file, folder, kind, span id) · node02 (same) · edge type · status · confidence [min, max] · evidence count · snapshot. No bodies, no private items — those are evidence behind a row, in edge-evidence-record-table and span-reference-text-table.
   - Raw code is not in the row. It lives once in span-reference-text-table, keyed by digest, where full-text search also lives; search starts there and lands in HLD rows by one join.
   - **Written only by extraction** — cpu-tool-hld-table-builder deriving from edge and span rows that CPU tools and tiny models produced. Never by ext-llm-coding-agent, never by human-architect-intent-author.
   - **hld-future-dependency-edges** holds the intended design. Written only by ext-llm-coding-agent or human-architect-intent-author through cpu-tool-propose-future-rows, which validates and never authors. Same row shape, status `proposed`, plus creator and reason.
   - **hld-delta-dependency-view** is the gap: a full outer join of the two. Nobody writes it. States: to_build · to_remove · realized · unplanned.
   - **hld-file-level-rollup** and **hld-folder-level-rollup** are counters (weight, kinds, confidence range) updated by trigger on every HLD row — the folder map is never stale relative to the item map.
   - **agent-learned-notes-table** is the only place prose is allowed: what ext-llm-coding-agent learned about the map, keyed to span ids. Agent-written; never read by extraction.

## End-to-end workflow, with roles

```text
 step  what happens                                                       who does it                                   writes
 ───── ───────────────────────────────────────────────────────────────── ───────────────────────────────────────────── ──────────────────────────────
 0     tree is walked, hashed, saved                                      cpu-tool-ingest-source-files                  file-source-text-table
 1     files cut into public spans; grey spans re-read                   cpu-tool-span-cutting-driver                  span-reference-text-table
                                                                           → tiny-model-span-boundary-tagger
                                                                           → tiny-model-grey-span-clarifier
 2     span text/doc copied, names tokenised, FTS built                   cpu-tool-text-copy-indexer                    span, name-token-index-table
 3     signature details extracted                                        cpu-tool-field-extraction-driver              field-signature-detail-table
                                                                           → tiny-model-field-detail-tagger
 4     names joined to targets; unique = extracted, else candidates       cpu-tool-name-join-edges                      edge-evidence-record-table
 5     candidates resolved; L3 sites found and queued                     cpu-tool-bind-resolve-edges                   edge-evidence-record-table
                                                                           → tiny-model-candidate-edge-binder
                                                                           → tiny-model-body-site-tagger
 6     HLD rows and roll-ups derived                                      cpu-tool-hld-table-builder                    hld-current-dependency-edges,
                                                                                                                         hld-file/folder-level-rollup
 7     coverage per level published                                       cpu-tool-status-coverage-banner               per-level-coverage-banner
 ───── first map ends here: ≤ 30 min for a Linux-sized tree on an 8 GB Air; L3 pending rows counted, draining in background
 8     agent asks: where is X · what breaks if X changes                  ext-llm-coding-agent                          —
       answered from the tables                                           cpu-tool-find-search-query, cpu-tool-impact-graph-walk
 9     intent written as rows: new node, new edge, retire edge            ext-llm-coding-agent or human-architect-intent-author
                                                                           via cpu-tool-propose-future-rows             hld-future-dependency-edges
 10    gap read: to_build rows, each with file, folder, neighbours, sig   ext-llm-coding-agent (reads hld-delta-dependency-view)   —
 11    code written for exactly those rows                                ext-llm-coding-agent                          the working tree
 12    changed files re-run steps 0–7 for those files only                cpu tools + tiny models                       all extraction tables
 13    matching rows appear in hld-current → delta flips to realized      hld-delta-dependency-view (by definition)        —
 14    review: rows of intent vs rows of fact                             human-architect-intent-author                 —
 ───── always in background: L3 drain (steps 5–6 per region), labels, re-map on edit; offline-teacher-rust-analyzer and
       offline-judge-heldout-evaluation train and score the tiny models, never at runtime
```

Roles in one line each:
- **ext-llm-coding-agent** — reads the product, writes intent and code; never writes facts.
- **human-architect-intent-author** — writes intent, reviews rows; never writes facts.
- **CPU tools** — move bytes, run SQL, derive tables; decide nothing.
- **tiny models** — make every judgment; see text, return labels; never touch a table.
- **offline teacher and judge** — rust-analyzer/rustc label and score; absent at runtime.

## End-to-end workflow, revised after DI-07 §16 (2026-11-09)

*Append-only: the table above is kept as the pre-§16 version. This section supersedes it. ◆ = changed row. Inspiration column: ✔ as is = a library we depend on or a tool we run offline; ○ pattern = idea only, never at runtime; ours = Parseltongue code.*

Decision carried in from DI-07 §16: one parse per file with `ra_ap_syntax` (pinned library, not the rust-analyzer server) does the cutting; models leave steps 1–3 entirely; the only model-shaped step left is binding (step 5), and even there pure name-join is measured first.

```text
 step  what happens                                                        who does it → writes                                 inspiration crates / repos                           use
 ───── ──────────────────────────────────────────────────────────────────── ──────────────────────────────────────────────────── ──────────────────────────────────────────────────── ───────────
 0     tree walked, bytes stored VERBATIM (no CRLF/whitespace/encoding      cpu-tool-ingest-source-files                         tursodatabase/agentfs (files as rows)                ○ pattern
       change), digest per file; unchanged digests skipped                  → file-source-text-table                             turso / libsql crate (embedded DB)                   ✔ as is
                                                                                                                                 ignore crate (ripgrep's .gitignore walker)           ✔ as is
                                                                                                                                 blake3 crate (file + span digests)                   ✔ as is
 1 ◆   ONE PARSE PER FILE: module-level items → byte range, kind, name,     cpu-tool-syntax-tree-walker                          ra-ap-rustc_lexer (= rustc's own lexer)              ✔ as is
       visibility, conf 1.0; nested items kept under a parent id; tree      → span-reference-text-table                          ra_ap_parser + ra_ap_syntax (lossless tree, ranges)  ✔ as is, pinned
       dropped after the file. Files whose parse reports errors →           → tiny-model-grey-span-clarifier (grey rows only,    syn 2 (oracle in tests; fallback parser)             ✔ as is
       indentation-recovery re-cut; those spans conf < 1 = grey               background)                                        rust-analyzer ITEM_RECOVERY_SET / error_block        ○ pattern
                                                                                                                                 experiments/spancut (indentation recovery rule)      ✔ ours
 2 ◆   same pass: params, returns, generics, bounds, struct fields, trait   cpu-tool-syntax-tree-walker (no model)               rustdoc JSON (which fields a public item has)        ○ schema idea
       items, use trees, cfg/attrs, docs; name tokens for FTS               → field-signature-detail-table                       SQLite FTS5 / Turso FTS                              ✔ as is
                                                                            → name-token-index-table                             cargo-modules (what counts as a module item)         ○ pattern
 3 ◆   same pass: body sites — calls, method calls, constructions, field    cpu-tool-syntax-tree-walker (no model)               ra_ap_syntax ast::CallExpr / MethodCallExpr /        ✔ as is
       reads/writes, .await, unsafe, panic!/? — written as PENDING L3 rows  → edge-evidence-record-table (status pending)        FieldExpr / AwaitExpr / BlockExpr(unsafe)
       pointing at span + byte range; nothing resolved yet                                                                      rust-code-analysis (Mozilla; per-fn metrics)         ○ pattern
                                                                                                                                 cargo-geiger (unsafe counting)                       ○ pattern
 4 ◆   names joined to spans via use trees + paths: unique → extracted,     cpu-tool-name-join-edges                             SCIP (Sourcegraph symbol naming scheme)              ○ id format
       several → candidates, none → unresolved; JOIN RATE published —       → edge-evidence-record-table                         github/stack-graphs (name resolution w/o compiler)   ○ pattern
       the number that decides how much binding is needed                                                                       rust-analyzer hir-def nameres                        ○ offline teacher only
 5 ◆   only candidate rows go to the binder (CPU resolver first); macro     cpu-tool-bind-resolve-edges                          rust-analyzer (labels to train the binder)           ✔ offline teacher
       invocations at item position go to the guesser; background after    → tiny-model-candidate-edge-binder                   cargo-expand (macro truth for offline labels)        ✔ offline teacher
       the first map; abstain when evidence is missing                     → tiny-model-macro-item-guesser                      Qwen2.5-Coder small / Cassini-class encoders         ○ teacher / base
                                                                            → edge-evidence-record-table
 6     public→public edges collapsed through private chains → HLD rows;     cpu-tool-hld-table-builder                           cargo-modules dependency graph                       ○ pattern
       roll-ups by trigger                                                  → hld-current-dependency-edges,                      RustPräzi (ecosystem call graph)                     ○ pattern
                                                                              hld-file/folder-level-rollup                       cargo-semver-checks (what "public" means)            ○ pattern
                                                                                                                                 SQL recursive CTEs (Turso/SQLite)                    ✔ as is
 7 ◆   coverage per level + parser-error count + grey-span count +          cpu-tool-status-coverage-banner                      —                                                    ours
       join rate + unresolved/binding backlog published                     → per-level-coverage-banner
 ───── ◆ first map: parse ≈ 1 min for 10 M lines (measured 789 klines/s/core, DI-07 §16 cp 8); DB write and binding are the long poles;
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

Reading rule for the inspiration column: anything ✔ at runtime is a *library* (parser, lexer, DB, FTS, hashing, walker) — never a server, a build loop, or a compiler. Everything that needs a compiler or a whole-program database is ✔ only in the offline lanes.

### Names changed by this revision

```text
 new       cpu-tool-syntax-tree-walker        one ra_ap_syntax pass per file → span, field, name-token and pending L3 rows (steps 1–3)
 new       tiny-model-macro-item-guesser      macro invocation at item position → which item(s) it defines; status inferred
 retired   tiny-model-span-boundary-tagger    boundaries come from the parser (DI-07 §16: 376/376 on real files)
 retired   tiny-model-field-detail-tagger     fields come from the tree
 retired   tiny-model-body-site-tagger        sites come from the tree
 retired   cpu-tool-span-cutting-driver       folded into the walker
 retired   cpu-tool-text-copy-indexer         folded into the walker
 retired   cpu-tool-field-extraction-driver   folded into the walker
 kept      tiny-model-grey-span-clarifier     now only for spans the recovery rule marks conf < 1
 kept      tiny-model-candidate-edge-binder   now gated behind the measured join rate
```

Role line that changes: **tiny models** — "make every judgment" → "make the judgments grammar cannot: grey boundaries, macro-defined items, ambiguous binding; see text, return labels; never touch a table."

### Three journeys through the revised workflow

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
    grey row queued → tiny-model-grey-span-clarifier later confirms/adjusts f's end
 2–3 fields and sites for g and S exact; f's sites written but flagged from a grey span
 4–6 edges from g and S flow normally; HLD rows involving f carry conf ≤ 0.3 and status abstained
 7  banner shows 1 grey span in this file; nothing is lost, nothing is silently wrong
 12 next save fixes the brace → d4 → f re-cut clean → its rows flip to extracted
```

**Journey C — what a parser cannot see: an item made by a macro, and a name with no unique target.** Design for the unmeasured slice.
```text
 1  kernel/platform.rs: `kernel::impl_device_context_deref!(unsafe { Device });` at item position → span kind macro_call conf 0.9
    (a parser knows it is a macro call; it does not know it DEFINES a Deref impl)
 5  tiny-model-macro-item-guesser reads the span + the macro's own span (found by name) → proposes "impl Deref for Device", inferred 0.7
 3–4 elsewhere, site `ctx.lock()` → candidates: Mutex::lock, SpinLock::lock, RwLock::lock → no receiver type visible at the site
 5  binder: use-tree (`use crate::sync::SpinLock`) + field decl `ctx: SpinLock<…>` → SpinLock::lock inferred 0.85;
    if evidence is missing it ABSTAINS → row stays pending, counted in the banner, never guessed silently
 6  HLD gains Device → Deref (inferred) and Driver::probe → SpinLock::lock (inferred); both carry their confidence into the delta
 14 the human sees exactly which rows are inferred and can confirm or correct — that correction becomes training data for the judge
```

Open question carried forward (replaces "span boundaries on broken code"): **how good is pure name-join (step 4) before any model?** That is the next measurement.
