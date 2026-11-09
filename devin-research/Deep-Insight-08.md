# Deep Insight 08 — Parseltongue end to end: what is built, by whom, in what order

## 0. How to read this document

Parseltongue is a command-line tool that turns a large Rust codebase into a small set of database tables an AI coding agent can query: *which public item depends on which*, *where is X*, *what breaks if X changes*, *what is planned but not yet built*. It does this on an ordinary laptop (the design target is an 8 GB MacBook Air, first map of a 10 M-line tree in under 30 minutes) without running the compiler or an IDE server.

This document is the single, self-contained description of the pipeline. It is written to be read top to bottom:

```text
 §1  glossary        every term used below, defined once, in the order it first appears
 §2  the thesis      what the product is (one table) and what everything else exists for
 §3  the decision    the one engineering choice the pipeline rests on, and the measurements behind it
 §4  the journey     the fifteen steps in one line each
 §5  the table       the same fifteen steps with owners, outputs, borrowed ideas, and where the tiny LLMs sit
 §6  the roles       who is allowed to write what
 §7  the tiny LLMs   when each is called, when it is not, what it sees and returns
 §8  the cost        training data, build time, and the 30-minute / 8 GB budget
 §9  three journeys  one ordinary item, one broken file, one macro plus an ambiguous name, walked through every step
 §10 the names       every four-word name used here, plus names retired and not to be reused
 §11 the open question
 §12 fields           once spans are known: why fields, what is done with them, how a public-interface signature is built
 §13 parked ideas     explorations not yet part of the design: public-only tiers, the six axes of a span (+ ELI5), virtual spans
 §14 coverage         the byte-coverage invariant: spans + classified gaps = file length, measured on 3,370 files; TDD log
```

Conventions used throughout:

```text
 four-word names    every named thing (table, tool, model, role) has a four-word hyphenated name that says what it is:
                    hld-current-dependency-edges, cpu-tool-syntax-tree-walker, tiny-llm-edge-binder, ext-llm-coding-agent
 M                  a number marked M was measured in this project (parser throughput, oracle agreement). Every other number
                    is an estimate and says so
 →                  "produces" or "becomes"
 ✔ / ○ / —          in the inspiration columns: a thing we use as code / an idea we copy / nothing external (full legend in §5)
```

## 1. Glossary

Terms in the order they appear. Each is defined once here and then used without further explanation.

```text
 working tree             the checked-out source folder on disk, exactly as the developer has it — including half-edited, broken files
 file row                 one file's bytes stored UNCHANGED (no newline, whitespace or encoding normalisation) plus a digest, in the database
 digest                   a content hash. Same bytes → same digest. Used to skip unchanged files and to detect stale rows
 database                 one Turso / SQLite-compatible file holding every table below. No server process
 span                     one item of source (a fn, struct, impl, trait, const, use, mod, macro call…) addressed by (file, start byte, end byte).
                          Line and column are computed from the bytes when a human needs them; they are never stored as identity
 kind · name · visibility what a span is (fn, struct…), what it is called, and pub / pub(crate) / private
 module-level item        an item directly inside a file or a mod block, as opposed to one nested inside a function body
 parse error              the parser could not read the file as valid Rust (missing brace, half-typed line). The file is still stored and cut
 recovery rule            our extra rule for files with parse errors: an item keyword, doc comment or attribute that starts its own line at the
                          column of the currently open item ends that item. It is a heuristic about indentation, not grammar
 grey                     a span produced by the recovery rule. It carries confidence < 1 (typically 0.3) and is re-examined by a tiny LLM later
 confidence               a number 0–1 on every span and edge saying how sure extraction is. 1.0 = read directly from a clean parse
 levels L0–L3             L0 crate · L1 folder and file · L2 public item (the product) · L3 body-level evidence (calls, field uses, awaits…)
 fields                   the parts of a signature: parameters, return type, generics, bounds, struct fields, trait items, attributes, docs
 use tree                 a `use a::b::{c, d};` line — the path information that lets a name be joined to the item it refers to
 body site                one place inside a function body where something happens: a call, a method call, a constructor, a field read or
                          write, an .await, an unsafe block, a panic!/? exit. Each is an L3 evidence row
 pending                  status of a body site that has been found but whose target is not yet known
 name join                matching a name at a site or in a signature to span rows through use trees and paths, in SQL
 extracted · candidates · unresolved   the three outcomes of a name join: exactly one target / several possible targets / none
 join rate                the share of names that joined to exactly one target. The one number that decides how much model work remains
 CPU resolver             deterministic rules that try to pick one candidate from several (receiver type, field type, use tree) before any model
 inferred                 status of a target chosen by a tiny LLM; always carries the model's confidence
 abstained                status of a row where neither the resolver nor the model had enough evidence; it stays visible and counted
 edge                     "item A depends on item B", with a type: calls · uses_type · implements · reexports · contains
 evidence row             an L2 or L3 fact behind an edge, with its span, byte range, status and confidence
 HLD                      the public-interface dependency graph: public item → public item edges, with private chains collapsed away
 collapsing private chains  if public A calls private helper h, and h calls public B, HLD gets A → B; the helper rows remain as evidence
 roll-up                  file → file and folder → folder counters derived from HLD rows by database trigger, so they are never stale
 first map                the first run over a tree, from empty tables to a usable HLD. The 30-minute target applies to it
 background               work that runs after the first map is usable, without blocking it
 banner                   the published coverage summary: per-level coverage, parser errors, grey spans, join rate, backlog
 ext-llm-coding-agent     the external coding agent (Claude Code, Codex, Aider…) that talks to Parseltongue through one CLI
 human-architect-intent-author  the person who writes intended structure and reviews rows
 CPU tool                 a deterministic Parseltongue program: moves bytes, runs the parser, runs SQL, derives tables. Decides nothing
 tiny LLM                 a small model (tens of MB) that reads text and returns labels with confidence for the cases grammar cannot decide
 known-macro table        a hand-written list of common macros (std, kernel) and the items each one defines, consulted before any model
 offline teacher · offline judge   rust-analyzer, rustc, cargo-expand used on our machines to label training data and score the models.
                          They never run on the user's machine
 intent                   a row describing what SHOULD exist: a new item, a new edge, an edge to retire
 delta                    the computed difference between what exists and what is intended
```

## 2. The thesis

1. **hld-current-dependency-edges is the product.** One table of current public item → public item dependencies, derived from source, never hand-written.
2. **It is read on every task and rebuilt only where the tree changed.**
   - One row = a public item depending on a public item: node01 (signature, file, folder, kind, span id) · node02 (same) · edge type · status · confidence [min, max] · evidence count · snapshot. No bodies, no private items — those are evidence behind a row, in edge-evidence-record-table and span-reference-text-table.
   - Raw code is not in the row. It lives once in span-reference-text-table, keyed by digest, where full-text search also lives; search starts there and lands in HLD rows by one join.
   - **Written only by extraction** — cpu-tool-hld-table-builder deriving from edge and span rows that the parser pass (cpu-tool-syntax-tree-walker) and the tiny LLMs produced. Never by ext-llm-coding-agent, never by human-architect-intent-author.
   - **hld-future-dependency-edges** holds the intended design. Written only by ext-llm-coding-agent or human-architect-intent-author through cpu-tool-propose-future-rows, which validates and never authors. Same row shape, status `proposed`, plus creator and reason.
   - **hld-delta-dependency-view** is the gap: a full outer join of the two. Nobody writes it. States: to_build · to_remove · realized · unplanned.
   - **hld-file-level-rollup** and **hld-folder-level-rollup** are counters (weight, kinds, confidence range) updated by trigger on every HLD row — the folder map is never stale relative to the item map.
   - **agent-learned-notes-table** is the only place prose is allowed: what ext-llm-coding-agent learned about the map, keyed to span ids. Agent-written; never read by extraction.

Everything else in this document — the CPU tools, the parser, the tiny LLMs, the future and delta tables — exists to create, keep current, query and extend that one table.

## 3. The one design decision

**Each file is parsed exactly once, with rust-analyzer's parser used as a pinned library (`ra_ap_syntax` + `ra_ap_parser`, on top of rustc's own lexer `ra-ap-rustc_lexer`). From that one parse come the spans, the fields, the names, the use trees and the body sites. No model reads ordinary source. The rust-analyzer server, its analysis database, the compiler and the Cargo build loop are not run.**

Why this is safe to rely on — the measurements (M):

```text
 agreement on valid code     376 / 376 module-level items on 125 real files (rust-analyzer, cargo-modules, flowistry, linux/rust)
                             byte-identical in start, end, kind, name and visibility to the `syn` parser used as oracle
 throughput                  789 k lines per second per core for a full lossless syntax tree; 1 M lines in 1.3 s on one core
                             → a 10 M-line tree parses in ≈ 13 s on one core, ≈ 5 s on four
 broken files                a grammar-only parser nests every later item inside an unterminated fn, or lets a stray `{` swallow the rest
                             of the file. With the recovery rule, 98–100 % of untouched items stay exact and the damage is marked grey
```

What follows from it:

```text
 steps 1–3 need no model      boundaries, fields and sites are read from the tree, exactly
 the only model-shaped work   deciding between several candidate targets for a name (step 5), repairing grey spans, and guessing what a
                              macro call defines — and even binding is done by SQL name-join first and measured before any model runs
 the 30 minutes               are spent on disk and database, not on reading tokens
```

## 4. The journey in one line per step

Legend: `X → Y` = X is turned into Y. Steps 0–7 are the first map; 8–14 are the daily loop with the agent.

```text
 0   codebase  →  files           the working tree is walked; every .rs file's bytes are stored unchanged as one file row, with a digest;
                                  a file whose digest did not change is skipped
 1   file      →  spans           each file is parsed once; every module-level item becomes a span row: byte start, byte end, kind,
                                  name, visibility; a file with parse errors is re-cut by the recovery rule and its doubtful spans are marked grey
 2   span      →  fields + names  from the same tree: params, return, generics, bounds, struct fields, trait items, use trees, cfg/attrs, docs;
                                  names are split into tokens and indexed for full-text search
 3   span      →  body sites      from the same tree: every call, method call, construction, field read/write, .await, unsafe block, panic!/? inside
                                  a body becomes a pending L3 row pointing at its byte range — found, not yet resolved
 4   name      →  target          each site/field name is joined to span rows through use trees and paths: one match = extracted edge,
                                  several = candidate list, none = unresolved; the join rate is published
 5   candidates →  one target     the CPU resolver tries first; only still-ambiguous candidates go to tiny-llm-edge-binder; macro invocations at
                                  item position go to tiny-llm-macro-guesser; no evidence = abstain, never guess
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

## 5. The workflow table

Legend for the columns:

```text
 step                                   the same numbering as §4
 what happens                           the work, in full
 who does it → writes                   the four-word name of the tool, model or person, and the table(s) it writes. "→ tiny-llm-…" under a
                                        CPU tool means the tool hands rows to that model and writes the model's answer back
 inspiration (use)                      crates and projects whose code or ideas this step uses, each tagged with how (legend after the table)
 tiny LLM in this step · when           which model, if any, is involved in this step and the exact condition that triggers it
 tiny-LLM estimate … · in the 30 min?   the volume of work and time on an 8 GB Air for a 10 M-line tree (40k files, 1.5 M spans, 1.0 M L2 edges,
                                        3.9 M L3 call sites — medium estimates), and whether the step is on the first-map critical path (IN) or
                                        runs afterwards (BACKGROUND). Only parser throughput and the oracle are M; the rest are estimates
```

```text
 step  what happens                                           who does it → writes                         inspiration (use)                              tiny LLM in this step · when            tiny-LLM estimate, 10 M lines, 8 GB Air · in the 30 min?
 ───── ───────────────────────────────────────────────────── ──────────────────────────────────────────── ────────────────────────────────────────────── ─────────────────────────────────────── ────────────────────────────────────────────────────────
 0     tree walked; file bytes stored VERBATIM (no CRLF/      cpu-tool-ingest-source-files                 agentfs: files as rows (○ pattern)             none                                    0 model time. CPU: read 350 MB + digests ≈ 1–5 min · IN
       whitespace/encoding change); digest per file;          → file-source-text-table                     turso / libsql (✔ as is)
       unchanged digests skipped                                                                           ignore walker, blake3 (✔ as is)
 1     ONE PARSE PER FILE: module-level items → byte range,   cpu-tool-syntax-tree-walker                  ra-ap-rustc_lexer (✔ as is)                    tiny-llm-span-clarifier · only on a     ≤ 2 % of 40k files broken → 1–2k grey spans × ~300 tokens
       kind, name, visibility, conf 1.0; nested items under   → span-reference-text-table                  ra_ap_parser + ra_ap_syntax (✔ as is, pinned)  file whose parse reported errors AND   ≈ 0.5 M tokens; 30–100 M model ≈ 1–2 min GPU / 5 min CPU ·
       a parent id; tree dropped per file. Parse errors →     → tiny-llm-span-clarifier (grey rows,        syn 2: oracle in tests, fallback (✔ as is)     the recovery rule left conf < 1;        BACKGROUND. First map ships them grey. Parse itself:
       recovery rule re-cuts; those spans conf < 1 = grey      background)                                 rust-analyzer resync set (○ pattern)           never on a clean file                   ≈ 13 s/core (M), ≈ 1 min with row writing · IN
                                                                                                           our indentation recovery rule (✔ ours)
 2     same pass: params, returns, generics, bounds, struct    cpu-tool-syntax-tree-walker (no model)       rustdoc JSON (○ schema idea)                   none — fields come from the tree        0 model time. Rows: 1.5 M spans, 4 M fields, 7 M name
       fields, trait items, use trees, cfg/attrs, docs;        → field-signature-detail-table               SQLite FTS5 / Turso FTS (✔ as is)                                                      tokens; DB write + FTS ≈ 5–10 min (shared with 6) · IN
       name tokens for FTS                                     → name-token-index-table                     cargo-modules (○ pattern)
 3     same pass: body sites — calls, method calls,            cpu-tool-syntax-tree-walker (no model)       ra_ap_syntax ast::CallExpr / MethodCallExpr /  none — sites come from the tree         0 model time. 3.9 M call sites + 0.9 M field/await/
       constructions, field reads/writes, .await, unsafe,      → edge-evidence-record-table                 FieldExpr / AwaitExpr (✔ as is)                                                        unsafe/? rows written pending, same pass · IN
       panic!/? → PENDING L3 rows with byte ranges              (status pending)                            rust-code-analysis, cargo-geiger (○ pattern)
 4     names joined to spans via use trees + paths: unique →  cpu-tool-name-join-edges                     SCIP symbol strings (○ id format)              none — pure SQL; publishes the JOIN     0 model time. Indexed joins over 1.0 M L2 edges ≈ 2–5 min ·
       extracted, several → candidates, none → unresolved;     → edge-evidence-record-table                 stack-graphs (○ pattern)                       RATE, the number that sizes step 5      IN. Join rate is the one unmeasured input to every
       join rate published                                                                                 rust-analyzer nameres (✔ offline teacher)                                              estimate below.
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
 ───── first map: CPU critical path ≈ 10–20 min for 10 M lines (read 1–5 · parse 1 · DB/FTS 5–10 · join 2–5) → headroom for a 33 M-line Linux-sized tree (×3.3); zero tiny-LLM time inside it.
       Then, in background: clarifier 1–5 min · guesser 1–5 min · binder L2 15–45 min · binder L3 ≈ 1.5 h by region.
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

Legend for the inspiration column:

```text
 ✔ as is           a crate or tool we take unchanged. At runtime this is always a LIBRARY linked into the Parseltongue binary
                   (ra_ap_syntax, ra-ap-rustc_lexer, turso, blake3, ignore, SQLite FTS5) — never a server, build loop or compiler.
                   Example: ra_ap_syntax parses the file; we call it, we do not re-implement it.
 ✔ as is, pinned   same, but version-locked; bumped only when the syn-oracle test suite stays green (ra_ap_* releases weekly, 0.0.x API).
 ✔ offline teacher a tool we RUN, never ship: it produces labels or ground truth for training/evaluation on our machines.
                   Example: rust-analyzer resolves a call site so the binder has a correct answer to learn from.
 ✔ ours            Parseltongue code, written and tested in this project. Example: the indentation recovery rule.
 ✔ external        the thing on the other side of the CLI; we talk to it, we do not contain it. Example: Claude Code / Codex as ext-llm-coding-agent.
 ✔ dev tool        used by humans/agents while developing Parseltongue; not part of the shipped runtime. Example: codebase-memory-mcp.
 ○ pattern         an IDEA we copy, no code dependency. We read how the project does it and write our own version.
                   Example: agentfs stores files as rows in SQLite → our file-source-text-table does the same; no agentfs crate is linked.
 ○ id format       a naming/encoding convention we adopt for compatibility. Example: SCIP symbol strings as the shape of our item ids.
 ○ schema idea     field names and shapes borrowed from a spec. Example: rustdoc JSON tells us which fields a public item has.
 ○ teacher/base    a larger model used only to produce training labels, or as the starting weights a tiny LLM is distilled from.
 —                 nothing external; plain Parseltongue logic or a by-definition view.
```

Reading rule: anything ✔ at runtime is a *library* (parser, lexer, database, full-text search, hashing, directory walker) — never a server, a build loop, or a compiler. Everything that needs a compiler or a whole-program database is ✔ only in the offline lanes.

## 6. The roles — who may write what

```text
 role                               reads                                   writes                                             never
 ────────────────────────────────── ─────────────────────────────────────── ────────────────────────────────────────────────── ───────────────────────────────────
 ext-llm-coding-agent               every table, through one CLI            hld-future-dependency-edges (via the propose tool), facts: no span, edge or HLD row
                                                                            agent-learned-notes-table, the working tree
 human-architect-intent-author      every table; the delta in review        hld-future-dependency-edges (via the propose tool)  facts
 CPU tools                          files, trees, tables                    every extraction table and the HLD                 a judgment: they decide nothing
 tiny LLMs                          text handed to them by a CPU tool       nothing — they return labels + confidence; a CPU    a table, a file
                                                                            tool writes the row
 offline teacher · offline judge    corpora on our machines                 training labels, evaluation scores                 anything on the user's machine
 hld-delta-dependency-view          hld-current + hld-future                nothing — it is a view, computed on read            —
```

## 7. The tiny LLMs — when each is called, and when it is not

Legend: *called when* is the exact trigger; *input* is the text a CPU tool hands over; *output* is what comes back (always with a confidence); *if it abstains* is what the table shows when the model declines; *share of work* is the fraction of the tree the model ever sees.

```text
 name                     called when                                   input                              output                      if it abstains                  share of work
 ──────────────────────── ───────────────────────────────────────────── ────────────────────────────────── ─────────────────────────── ─────────────────────────────── ─────────────────
 tiny-llm-span-clarifier  a file's parse reported errors AND the         the grey span's bytes plus the     confirmed/moved end byte,   span stays grey (conf ≤ 0.3);   ≈ 1 span per
                          recovery rule produced a span with conf < 1    lines around it                    kind, name, visibility,     its edges stay abstained;       broken file;
                          (step 1). Never on a file that parses clean.                                      new confidence              counted in the banner           0.01–0.5 % of spans
 tiny-llm-edge-binder     a site/field name joined to SEVERAL candidate  the site's span text, the          one target span id + conf,  row stays pending with its      1 − join rate;
                          spans and the CPU resolver (use trees, field   candidate signatures, the          or abstain                  candidate list; never a         measured next
                          types, receiver types) could not pick one      enclosing item's use tree                                      guessed edge
                          (step 5). Never on unique joins.
 tiny-llm-macro-guesser   a macro invocation sits at item position       the invocation bytes + the         zero or more proposed       span stays kind macro_call;     1–3 % of public
                          (`foo! { … }` / `foo!(…);` at module or impl   macro_rules!/proc-macro span       items (kind, name, vis)     no items claimed; counted       spans, minus the
                          level) and is not in the known-macro table     if found by name                   with status inferred        in the banner                   known-macro table
 tiny-llm-result-ranker   optional; only when `find` returns more rows   query + candidate rows             reordered rows              CPU BM25 order is used          0 at first map
                          than the agent's budget
 NOT called              any span, field, name token, use tree or body site on a file that parses clean — the parser is exact there (M: 376/376);
                         any unique name join; any HLD derivation or roll-up; anything in the future/delta tables
```

Rules that hold for all four: text in, labels + confidence out; never write a table (a CPU tool writes the row with the label); always allowed to abstain; every output carries status `inferred` so hld-delta-dependency-view and the human reviewer can see which rows came from a model.

Name note: the four-word rule leaves two words for the descriptor, so "grey" and "candidate" live in the *called when* column rather than in the names.

## 8. The cost — training data, build time, and the 30-minute / 8 GB budget

Legend: *training INPUT* is what the model sees during training; *training OUTPUT / label* is the correct answer and who produces it; *how much data · time to build* is the size of the training set and how long it takes to generate; *runtime volume · time* is the work on a 10 M-line tree (medium estimates: 40k files, 1.5 M spans, 1.0 M L2 edges, 3.9 M L3 call sites; a 33 M-line Linux-sized tree is ×3.3) and the time on an 8 GB Air; *where it sits* says whether the work is inside the first-map 30 minutes or after. Cost rule used for every estimate: encoder cost ≈ 2 · parameters · tokens; Air ≈ 1.1–1.7e12 FLOP/s GPU, ≈ 4e11 FLOP/s CPU. Training always happens offline on a desktop GPU or a rented box — never on the Air, never at runtime.

```text
 name                     training INPUT dataset (what the model sees)           training OUTPUT / label set (who produces it)          how much data · time to build it               runtime volume on the 10 M-line tree · time on the Air          where it sits in the 30 min
 ──────────────────────── ────────────────────────────────────────────────────── ────────────────────────────────────────────────────── ────────────────────────────────────────────── ─────────────────────────────────────────────────────────────── ─────────────────────────────
 tiny-llm-span-clarifier  grey span bytes + ~20 lines of context, taken from     the TRUE item boundaries, kind, name, visibility of     synthetic: 1 M-line parseable corpus ×         ≤ 2 % of 40k files have parse errors ≈ 800 files × 1–2 grey     NOT in the 30 min. First map
                          DELIBERATELY BROKEN copies of parseable files          the same file BEFORE it was broken — from syn /         5 breakage kinds × several sites per file      spans ≈ 1–2k spans × ~300 tokens ≈ 0.5 M tokens.                 writes them grey (conf ≤ 0.3);
                          (delete `}`, truncate mid-item, insert a half-typed    ra_ap_syntax on the unbroken file. Fully automatic,    → 0.5–1 M labelled examples; generation        30–100 M model → ~1e14 FLOP → 1–2 min GPU / ~5 min CPU.          clarifier runs in background
                          item, delete `;`, insert a stray `{`) + the recovery   no human, no compiler run. (M: the oracle exists        ≈ minutes (parser at 789 k lines/s, M).        Per broken file: milliseconds.                                     and flips rows to extracted.
                          rule's own grey span as the "question"                 today, 376/376.)                                       Training: ~3e8 tokens × 3 epochs on a 30–100 M
                                                                                                                                        encoder ≈ 2–6e17 FLOP ≈ 1–3 h desktop GPU.
 tiny-llm-edge-binder     a call/field site's text + the enclosing item's use    the ONE span the name resolves to — from the           real: every ambiguous site in a 1 M-line       L2: 1.0 M edges × (1 − join rate). At 20 % ambiguous (to be      NOT in the 30 min. First map
                          tree + the 2–8 candidate spans' signatures, from REAL  offline teacher (rust-analyzer go-to-definition /      open-source corpus (≈ 400k call sites,         measured, step 4) ≈ 200k rows × ~400 tokens ≈ 80 M tokens;        publishes unique joins
                          files in the corpus, only where name-join returned     rustc HIR), run once per corpus on a desktop. Human    ~80k ambiguous). Teacher pass: rust-analyzer   100 M model → 1.6e16 FLOP ≈ 15–45 min on the Air GPU.            (extracted) and candidate
                          several candidates                                     correction rows from step 14 are added over time.      load + per-site resolve ≈ 1 h per 1 M lines    L3: 3.9 M sites, ~0.8 M ambiguous ≈ 320 M tokens → 6e16 ≈ 1.5 h  lists; binder drains L2 then
                                                                                                                                        (estimate). Training: 100 M encoder,           background; per region on demand: seconds.                       L3 in the background (the
                                                                                                                                        ~1.6e8 tokens × 3 epochs ≈ 3e17 ≈ 2 h GPU.                                                                      "L3 drain").
 tiny-llm-macro-guesser   an item-position macro invocation's bytes + the        the items the macro actually produces (kind, name,     real: top ~1,000 crates.io crates + kernel     1–3 % of 480k public spans are item-position macro calls ≈        NOT in the 30 min. First map
                          macro_rules!/proc-macro definition's bytes when it is  visibility, impl-of) — from cargo-expand (offline       Rust; cargo-expand needs a BUILD → 1–5 min     5–15k × ~500 tokens ≈ 2.5–7.5 M tokens; 100 M model → ~1e15      stores the span as kind
                          in the tree, from real crates                          teacher) on crates that compile, diffed against the    per crate, 1–3 CPU-days total, parallel;       FLOP ≈ 1 min GPU / ~5 min CPU. The known-macro table removes      macro_call; guesser adds
                                                                                 un-expanded tree. Automatic for compiling crates.      ≈ 50–200k examples. Training ≈ 1 h GPU.        most of these before the model.                                   inferred items in background.
 tiny-llm-result-ranker   a `find` query + the 20–50 candidate rows (names,       the row the agent actually used — from                 starts empty; grows with use; thousands of     per query: ~20 rows × 100 tokens ≈ 2k tokens → milliseconds.      not part of the map at all;
                          signatures, doc first line, BM25 score)                agent-learned-notes-table and step-14 review clicks.   examples before it is worth training.                                                                           per-query, optional.
```

How the 30 minutes are spent (10 M lines; a 33 M-line tree still fits):

```text
 inside the 30 min — CPU only, zero tiny-LLM time on the critical path
   step 0  read 350 MB + digests                                ≈ 1–5 min   (disk-bound)
   step 1–3 one parser pass, rows out                           ≈ 1 min    (M: 789 k lines/s/core → 13 s/core; 4 cores ≈ 5 s parse, the rest is row writing)
   step 2/6 DB write + FTS + roll-up triggers                    ≈ 5–10 min (1.5 M spans, 7 M name tokens, 1.0 M edges ≈ 1 GB on disk)
   step 4  name-join SQL                                         ≈ 2–5 min  (indexed joins over use trees × names)
   step 7  banner                                                seconds
   total                                                         ≈ 10–20 min → headroom for the ×3.3 Linux-sized tree
 after the 30 min — one model loaded at a time (≤ 100 MB int8 → < 0.5 GB RAM; parser and DB stay under ~2 GB; fits 8 GB)
   clarifier   ≈ 1–5 min      macro guesser ≈ 1–5 min      binder L2 ≈ 15–45 min      binder L3 ≈ 1.5 h, by region on demand
 second map (edit loop)   only changed files re-enter; per file: parse ms, join ms, binder seconds
```

Why no model sits on the critical path: a model that reads every token of a 10 M-line tree costs minutes to hours on an Air no matter how small it is, while the parser reads the same tokens in about a minute and is exact on valid code. So the tiny LLMs are background workers whose cost scales with the *ambiguous* fraction of the tree (grey spans, unresolved names, unknown macros), not with the tree itself.

## 9. Three journeys through the workflow

Legend for the notation: `d1, d2…` = file digests · `#41` = a span id · `[bytes 1180..2630)` = byte start inclusive, end exclusive · `conf` = confidence · `pub`/`priv` = visibility · statuses as defined in §1 (extracted, candidates, unresolved, pending, inferred, abstained; grey for spans; proposed / to_build / realized in the future and delta tables).

**Journey A — the ordinary case (≈ 98 % of files): `pub fn send` calls `RetryPolicy::next_delay` through a private helper.** Every step here is the measured path.
```text
 0  http/client.rs → file row, digest d1
 1  tree → span #41 [bytes 1180..2630) fn send pub conf 1.0; span #44 fn backoff priv (parent: module)
 2  fields: send(req: Request) -> Response; use crate::retry::RetryPolicy; doc "Sends one request…"
 3  sites: send → backoff() [byte 1402]; backoff → self.policy.next_delay() [byte 2210]      both status pending
 4  Request, Response, RetryPolicy: one span each in the tree → extracted, conf 1.0
    next_delay: TWO spans named next_delay (RetryPolicy::next_delay, Backoff::next_delay) → candidates
 5  CPU resolver sees the receiver type RetryPolicy from the use tree + the field declaration → RetryPolicy::next_delay, extracted;
    had the type not been visible, tiny-llm-edge-binder would have been asked and its answer written as inferred
 6  HLD row: send → RetryPolicy::next_delay   calls   extracted   conf 1.0   evidence: 2 L3 rows, private backoff collapsed
 7  banner: L2 100 % for this file; L3 2 of 2 sites bound
 8  agent: "what breaks if RetryPolicy changes?" → graph walk → send, and send's callers
 9  agent writes an intent row: send → Metrics::record (calls, proposed)       → delta shows it as to_build
 11 agent edits send's body; 12 client.rs d1 → d2, steps 1–6 for that one file; 13 the delta row flips to realized; 14 human sees intent = fact
```

**Journey B — the broken file: a `}` deleted while the agent is mid-edit.** Measured path (oracle and breakage runs).
```text
 0  file row stored verbatim, digest d3 (the file is broken; bytes don't care)
 1  parse reports 1 error; the tree alone would nest every later item inside the open fn → the recovery rule fires:
    `pub fn g` first on its line at column 0 ends the open item → fn f [..] conf 0.3 (grey); g and S clean, conf 1.0
    the grey row is queued → tiny-llm-span-clarifier later confirms or moves f's end
 2–3 fields and sites for g and S exact; f's fields and sites are written but inherit f's grey confidence
 4–6 edges from g and S flow normally; edges touching f carry conf ≤ 0.3 and are shown with it in HLD
 7  banner shows 1 grey span in this file; nothing is lost, nothing is silently wrong
 12 next save fixes the brace → d4 → f re-cut clean → its rows become conf 1.0, extracted
```

**Journey C — what a parser cannot see: an item made by a macro, and a name with no unique target.** This is the design for the unmeasured slice.
```text
 1  kernel/platform.rs: `kernel::impl_device_context_deref!(unsafe { Device });` at item position → span kind macro_call, conf 0.9
    (a parser knows it is a macro call; it does not know it DEFINES a Deref impl). Not in the known-macro table → queued for the guesser
 5  tiny-llm-macro-guesser reads the invocation + the macro's own definition span (found by name) → proposes "impl Deref for Device", inferred 0.7
 3–4 elsewhere, site `ctx.lock()` → candidates: Mutex::lock, SpinLock::lock, RwLock::lock — no receiver type visible at the site
 5  CPU resolver: use tree has `use crate::sync::SpinLock` and the field is declared `ctx: SpinLock<…>` → SpinLock::lock, extracted.
    If neither were visible, tiny-llm-edge-binder would decide (inferred, with its confidence) or ABSTAIN → the row stays pending with its
    candidate list, counted in the banner, never guessed silently
 6  HLD gains Device → Deref (inferred 0.7) and Driver::probe → SpinLock::lock (extracted); each carries its status into the delta
 14 the human sees exactly which rows are inferred and can confirm or correct — every correction becomes a training row for the binder
```

## 10. The names

Every four-word name used in this document, grouped. Retired names are listed so they are not reused.

```text
 ROLES
 ext-llm-coding-agent              the external coding agent (Claude Code, Codex, Aider…); writes intent and code, never facts
 human-architect-intent-author     the person; writes intent, reviews rows
 offline-teacher-rust-analyzer     rust-analyzer / rustc / cargo-expand producing training labels on our machines; never at runtime
 offline-judge-heldout-evaluation  the syn oracle and held-out repositories scoring the tiny LLMs

 TABLES AND VIEWS (one Turso / SQLite-compatible file)
 file-source-text-table            file rows: path, folder, digest, bytes
 span-reference-text-table         one row per span: id, kind, name, visibility, byte range, parent, confidence, doc, cfg
 name-token-index-table            tokens of names and paths, for joins and full-text search
 field-signature-detail-table      per-span fields: params, return, generics, bounds, struct fields, trait items, attributes
 edge-evidence-record-table        every L2/L3 evidence row with status, confidence, span, byte range, snapshot
 hld-current-dependency-edges      THE PRODUCT: current public item → public item rows, extraction-only
 hld-future-dependency-edges       intended rows; written only by the agent or the human through the propose tool
 hld-delta-dependency-view         full outer join of current and future; nobody writes it: to_build · to_remove · realized · unplanned
 hld-file-level-rollup             file → file counters maintained by trigger
 hld-folder-level-rollup           folder → folder counters maintained by trigger
 agent-learned-notes-table         what the agent learned about the map; the only table with prose
 per-level-coverage-banner         the published coverage summary

 CPU TOOLS (decide nothing)
 cpu-tool-ingest-source-files      walk the tree · store bytes verbatim · digest · skip unchanged
 cpu-tool-syntax-tree-walker       one parser pass per file → span, field, name-token and pending L3 rows; recovery rule on error files
 cpu-tool-name-join-edges          SQL join of names to spans → extracted / candidates / unresolved; publishes the join rate
 cpu-tool-bind-resolve-edges       CPU resolver first; hands ambiguous rows to the binder and macro calls to the guesser; writes their answers
 cpu-tool-hld-table-builder        derives hld-current-dependency-edges and the roll-ups from evidence rows
 cpu-tool-find-search-query        full-text / BM25 search → ranked rows with span text
 cpu-tool-impact-graph-walk        recursive SQL over edges: callers, callees, blast radius, cycles
 cpu-tool-propose-future-rows      validates and writes hld-future-dependency-edges rows for the agent or human
 cpu-tool-status-coverage-banner   writes per-level-coverage-banner

 TINY LLMS (only where grammar cannot decide; text in → labels + confidence out; never write a table)
 tiny-llm-span-clarifier           grey spans on files with parse errors → confirmed boundary, kind, name, visibility
 tiny-llm-edge-binder              a site with several candidate targets, after the CPU resolver failed → one target or abstain
 tiny-llm-macro-guesser            a macro call at item position, not in the known-macro table → the items it defines, inferred
 tiny-llm-result-ranker            optional; reorders find results beyond the agent's budget

 RETIRED — do not reuse
 tiny-llm-span-boundary-tagger · tiny-llm-field-detail-tagger · tiny-llm-body-site-tagger      boundaries, fields and sites come from the parser
 cpu-tool-span-cutting-driver · cpu-tool-text-copy-indexer · cpu-tool-field-extraction-driver   folded into cpu-tool-syntax-tree-walker
```

## 11. The open question

Everything above the binder is measured or follows from the parser. The one number nobody has measured yet is the **join rate** (step 4): on a real 10 M-line tree, what share of names join to exactly one target through use trees and paths alone? If it is high, tiny-llm-edge-binder is a small background worker; if it is low, binding becomes the long pole and the tiny LLM strategy must be re-examined. That measurement comes next.

## 12. Once spans are known — why fields, what is done with them, how a public-interface signature is built

A span (§1) is a box around an item: file, start byte, end byte, kind, name, visibility. That is enough to say *where* `send` is. It is not enough to say *what `send` depends on*, *what it looks like to a caller*, or *whether "retry policy" typed into `find` should land on it*. Fields and names exist for those three jobs. Everything in this section comes from the same parser pass as the span; no model is involved.

### 12.1 One item, three layers

```rust
/// Sends one request with retries.
#[cfg(feature = "http")]
pub async fn send<T: Into<Request>>(&self, req: T, policy: &RetryPolicy) -> Result<Response, Error> { … }
```

```text
 layer    what it is                                  stored in                       example
 ──────── ─────────────────────────────────────────── ─────────────────────────────── ─────────────────────────────────────────────────────────
 span     the box: where the item is and what kind    span-reference-text-table       client.rs [1180..2630) fn send pub
 fields   the typed parts inside the signature        field-signature-detail-table    param policy: &RetryPolicy · return Result<Response, Error>
 names    the searchable words in span and fields     name-token-index-table          send · retry · policy · retry_policy · response · error
```

Legend: *span* = the box · *fields* = the typed parts inside the box · *names* = the words in those parts, tokenised.

### 12.2 Why fields, once the span is known

```text
 job                              what the span alone cannot do                     what fields make possible
 ──────────────────────────────── ───────────────────────────────────────────────── ──────────────────────────────────────────────────────────
 1  dependencies (the product)    a box has no idea that `send` mentions            every type NAME in a field is a dependency waiting to be
                                  RetryPolicy, Request, Response, Error             joined to its span (step 4) → the uses_type edges of HLD
 2  the public interface          the box's text is 1,450 bytes including the body; a signature string built from fields only — no body, no
                                  the agent must not read bodies to use an item      comments — is what the HLD row shows to the agent
 3  search                        `find retry policy` does not match the bytes       name tokens: RetryPolicy → retry · policy · retry_policy
                                  "RetryPolicy" by substring                         → BM25 search lands on this span
 4  filtering and roll-ups        two cfg variants of `send` are two boxes with     attribute rows (cfg, derive, test) and flag rows (async,
                                  the same name                                      unsafe, const) let impact questions filter precisely
 5  change detection              a body edit changes the span digest               a signature digest built from fields only stays the same
                                                                                     → callers' HLD rows are NOT marked stale by a body edit
```

### 12.3 What the walker writes for `send`

```text
 field-signature-detail-table rows for span #41 (fn send)
 kind        ordinal  text                              type_names                    notes
 ─────────── ──────── ───────────────────────────────── ───────────────────────────── ──────────────────────────────────────────────
 doc         —        Sends one request with retries.   —                             first line is what `find` shows
 attribute   —        cfg(feature = "http")             —                             used to filter and to tell cfg twins apart
 flag        —        pub · async                       —
 generic     1        T: Into<Request>                  Into, Request                 the bound names a type → candidate edge
 param       1        self: &Self                       —                             marks a method; Self resolves to the impl's type
 param       2        req: T                            T (generic, not a target)
 param       3        policy: &RetryPolicy              RetryPolicy                   → candidate edge send → RetryPolicy
 return      —        Result<Response, Error>           Result, Response, Error       → three candidate edges

 name-token-index-table rows (span #41 and its fields)
 token          source
 ────────────── ────────────────────────────────────
 send           item name
 client         module path crate::http::Client
 http           module path
 retry          RetryPolicy, split at the case change
 policy         RetryPolicy
 retry_policy   RetryPolicy, snake form
 request        Request
 response       Response
 error          Error
```

Rules the walker follows:
- a field row stores the *name* of a type, never its target span — resolving names is step 4's job, so step 2 stays exact and model-free;
- `Self`, generics (`T`) and lifetimes are recorded but are not dependency candidates;
- every struct field, enum variant, trait item and impl item gets the same treatment, so `pub struct Config { pub retries: u8, policy: RetryPolicy }` yields field rows `retries: u8 (pub)` and `policy: RetryPolicy (priv)` — the private one is still evidence for an edge Config → RetryPolicy;
- `use` lines are also field rows (`use crate::retry::RetryPolicy` → path row), because they are what step 4 joins on.

### 12.4 How the public-interface signature is built from fields

The HLD row's "signature" column is not copied from the source; it is *assembled* from field rows, in a fixed order, so that two items with the same interface produce the same string and digest regardless of formatting, comments, or body.

```text
 assembly order            take from fields                                       for `send`
 ───────────────────────── ────────────────────────────────────────────────────── ───────────────────────────────────────────────────────────
 1  visibility             flag rows                                              pub
 2  qualifiers             flag rows, canonical order const · async · unsafe      async
 3  kind + name            span                                                   fn send
 4  generics               generic rows in ordinal order, bounds kept             <T: Into<Request>>
 5  parameters             param rows in ordinal order, `name: type`              (&self, req: T, policy: &RetryPolicy)
 6  return                 return row                                             -> Result<Response, Error>
 7  where / cfg            attribute rows that change the interface               #[cfg(feature = "http")]
 8  (dropped)              doc, body, comments, whitespace, derive-only attrs     —

 result   pub async fn send<T: Into<Request>>(&self, req: T, policy: &RetryPolicy) -> Result<Response, Error>  #[cfg(feature = "http")]
 digest   blake3 of that string = the signature digest stored on the HLD node
```

For other kinds the same order applies with the kind's own parts:

```text
 struct   pub struct Config { pub retries: u8 }                 public fields only; private fields are evidence, not interface
 enum     pub enum Mode { Fast, Safe(u8) }                      all variants (variants are public if the enum is)
 trait    pub trait Retry { fn next_delay(&self) -> Duration; } required and provided item signatures, bodies dropped
 impl     impl Retry for Backoff                                the impl header; its items are their own spans with their own signatures
 type     pub type Res<T> = Result<T, Error>;                   the alias and its target
 const    pub const MAX: u8 = 3;                                name and type; the value is dropped from the signature
 use      pub use retry::RetryPolicy;                           re-export: the path, because it is interface for callers
```

### 12.5 What the signature then does

```text
 the agent asks `find send`           → cpu-tool-find-search-query returns the signature string, the doc first line and the file — not the body
 the HLD row for send → RetryPolicy   → node01 = send's signature digest, node02 = RetryPolicy's; the edge type uses_type came from the param row
 the agent edits send's BODY          → span digest changes, signature digest does not → callers' rows stay valid, only send's own L3 rows re-run
 the agent changes a PARAM type       → signature digest changes → every HLD row with send as node01 or node02 is re-derived and the delta
                                        shows the change against the intended signature in hld-future-dependency-edges
 the human reviews a to_build row     → sees the intended signature string beside the current one, same assembly order, so the diff is readable
```

One line: **the span says where an item is; the fields say what it is made of; the signature built from the fields is what every other item, the agent, and the human see of it — and the only part whose change should ripple through the map.**

## 13. Ideas and explorations — not yet part of the design

*Parked here so they are not lost. Nothing in this section is decided; §1–§12 remain the design. Each idea has a plain-words summary first, the detail after, and the question that would decide it.*

### 13.1 Should the map focus on public-interface spans only?

**Plain words.** The product is public → public edges, so it is tempting to cut only public items. That does not work, because a public item can reach another public item *through* a private helper, and the map must still show that edge. The likely answer is two tiers: public items get everything; private items get a cheap "light" row that exists only so chains can be collapsed.

```text
 the product                 hld-current-dependency-edges = public item → public item. Private items never appear as nodes.

 why private spans must      public A calls private helper h, h calls public B  →  HLD must show A → B.
 still be CUT                To collapse that chain the walker needs h's box and the call sites inside h.
                             A private struct field `policy: RetryPolicy` is an edge Config → RetryPolicy — evidence, not interface.
                             A private item becomes public with one keyword edit; an existing light row makes that a flip, not a re-parse.

 what "public" would mean    pub, pub(crate), pub(super), pub(in path), and anything re-exported by `pub use` — the crate's visible surface.
                             Items inside a private module are NOT public even if marked pub (not reachable from outside).
                             Rule: public = reachable from the crate root through pub paths. Computable from the module tree; no model.
```

Proposed tiers (if adopted):

```text
 tier            which spans                        what steps 1–3 store                                            cost share (10 M lines, medium)
 ─────────────── ────────────────────────────────── ─────────────────────────────────────────────────────────────── ───────────────────────────────
 public (full)   reachable from the crate root via   span · fields · names/FTS · docs · signature string + digest ·  ≈ 880k spans → all field, name
                 pub paths; pub use re-exports        body sites                                                      and FTS rows
 private (light) everything else at module level,    span (kind, name, range, parent, digest) · body sites only       ≈ 590k spans, ~1/5 of the bytes:
                 incl. nested fns                     NO fields, NO docs, NO FTS, NO signature                          no FTS, no field rows
 body-only       locals, closures, let bindings       nothing — not a span; their calls are sites of the enclosing item —
```

Effects if adopted: `find` only ever returns public items (what the agent may use); the field table shrinks ≈ 40 %; private rows exist solely for chain collapsing and for a cheap `pub` flip; the L3 drain order follows — sites in public bodies first, private bodies only when a chain needs them.

**Deciding question.** On a real tree, what share of public → public edges pass through at least one private helper? If it is small, the light tier could be deferred; if it is large (likely), the two-tier rule goes into §5 steps 1–2 and §12.3.

### 13.2 There are many more kinds of span than "fn, struct, impl…" — six axes

**Plain words.** A span is always one shape (a byte range with a kind), but it varies along six independent axes, and most extraction bugs come from treating only the first axis as real. The glossary's "kind · name · visibility" is a simplification of this.

```text
 AXIS 1 — grammar kind (what the parser hands over directly)
   fn · struct · enum · union · trait · trait alias · impl (inherent) · impl (trait for type) · type alias · const · static
   mod (inline `mod x { }` vs file `mod x;`) · use · extern crate · extern block + its foreign fn/static · macro_rules! · `pub macro` (2.0)
   macro invocation at item position · module header (the file's `//!` docs + `#![…]` attrs — crate-level docs ARE interface)

 AXIS 2 — nesting (where the box sits)
   module-level item · impl member (method, assoc const, assoc type) · trait member (required / provided) · enum variant · struct field
   · fn nested inside a body · item inside a block or closure
   → members of a public type are addressable interface (RetryPolicy::next_delay), so they are SPANS, not merely fields of their parent

 AXIS 3 — reachability (what "public" means)
   reachable from the crate root via pub paths · pub(crate) · pub(super) / pub(in path) · private · public-but-#[doc(hidden)]
   · #[deprecated] · re-exported under another path by `pub use` · exported symbol (#[no_mangle], extern "C" — interface to OTHER languages)
   → "pub" inside a private mod is NOT public; computable from the module tree, no model

 AXIS 4 — origin (where the bytes came from)
   written by hand · generated by a macro invocation · generated by #[derive] (Clone, Debug… impls that exist with NO text of their own)
   · one of several #[cfg] twins (same name, different cfg) · #[cfg(test)] / #[test] / bench / example / build.rs · generated file (bindgen, prost)
   · doc-test code block inside a /// comment (compiles; references items; worth a tag, not a span)

 AXIS 5 — parse status (how sure we are of the box)
   clean (conf 1.0) · grey from the recovery rule · error region (garbage the parser wrapped) · truncated at EOF (half-typed)
   · inside git conflict markers <<<<<<< ======= >>>>>>> (both halves are real code; neither should be mapped as truth)

 AXIS 6 — identity across time (does the digest stay put)
   same bytes moved (new offsets, same digest) · body edit (span digest changes, signature digest does not) · signature edit · rename
   · visibility flip (private → pub: the row already exists, it changes tier) · cfg twin added
```

Two consequences, if this view is adopted:

```text
 1  VIRTUAL SPANS      three kinds of interface have no bytes of their own and would be DERIVED rows pointing at the span that caused them:
      #[derive(Clone)] on Config        → impl Clone for Config            CPU, from the known-macro table
      pub use a::b::X;                  → an alias of span X at a new path  CPU, after step 4 resolves X; `pub use a::*` expands to many
      foo! { … } at item position       → the items the guesser proposes    tiny-llm-macro-guesser
    same table, origin = derived, source_span = the attribute / use / invocation span

 2  TIER IS A FUNCTION OF AXES 2–4, NOT OF KIND
      full      reachable public items and their public members, re-export aliases, extern "C" exports
      light     private items, test-only items, nested fns
      tag only  doc-tests, conflict regions
      nothing   locals, closures
```

Proposed four-word name, if adopted: `span-kind-axis-taxonomy`; each axis would become a column of span-reference-text-table.

**Deciding question.** Which axes does the agent actually ask about? Reachability (axis 3) and parse status (axis 5) are needed for the product's correctness; origin (axis 4) matters for cfg twins and derives; the rest can wait until a `find` or impact question needs them.

### 13.3 ELI5 — the six questions every span answers

```text
 A SPAN IS A BOX DRAWN AROUND A PIECE OF CODE.  Every box answers SIX questions.  Same box, six labels.

                 ┌──────────────────────────────────────────────────────┐
                 │  /// Sends one request.                              │
                 │  #[cfg(feature = "http")]                            │
                 │  pub async fn send(&self, req: Request) -> Response  │
                 │  { … body … }                                        │
                 └──────────────────────────────────────────────────────┘
                   file client.rs · bytes 1180..2630

 1  WHAT IS IT?            ──►  fn      (or struct, enum, trait, impl, const, static, mod, use, macro, extern block, module header…)
                                the parser tells us this for free.  "a fn" / "a struct" / "a macro call we can't see inside"

 2  WHERE DOES IT SIT?     ──►  inside `impl Client`   (a member)
                                top of the file = module-level · inside an impl or trait = member · a field or variant = child of a type
                                · a fn hiding inside another fn's body = nested.  Members of a public type are boxes too: Client::send

 3  WHO CAN SEE IT?        ──►  everyone  (pub, and every folder on the way up is pub too)
                                pub seen from the crate root · pub(crate) · pub(super) · private · pub but #[doc(hidden)] · re-exported
                                by `pub use` under another name · exported to C with extern "C".   ⚠ `pub` inside a private mod = NOT public

 4  WHERE DID IT COME FROM? ──► typed by a human
                                or made by a macro `foo! { }` · or made by #[derive(Clone)] (an impl with NO text of its own) · or one of
                                two #[cfg] twins with the same name · or test-only · or a generated file (bindgen) · or a doc-test inside ///

 5  HOW SURE IS THE BOX?   ──►  sure (conf 1.0)
                                clean parse · grey (we guessed the end by indentation) · error region (garbage) · cut off at end of file
                                · inside <<<<<<< ======= >>>>>>> conflict markers (two versions of the truth, map neither)

 6  IS IT THE SAME BOX AS YESTERDAY? ──► yes, it just moved down 3 lines
                                same bytes moved (same digest, new offsets) · body edited (box digest changes, signature digest does not)
                                · signature edited · renamed · private → pub (same box, new tier) · a cfg twin appeared

 THREE BOXES WITH NO BYTES OF THEIR OWN  ("virtual" spans — rows that point at the box that caused them)
   #[derive(Clone)] on Config   ──►  impl Clone for Config          made by CPU, from the known-macro table
   pub use a::b::X;             ──►  X is ALSO reachable here        made by CPU, once step 4 knows which X
   foo! { … } at item position  ──►  whatever foo makes             made by tiny-llm-macro-guesser

 HOW MUCH WE STORE DEPENDS ON LABELS 2–4, NOT ON LABEL 1
   public & reachable (and its public members, re-exports, extern "C")  ──►  FULL:  box + fields + names + docs + signature + body sites
   private, test-only, nested                                            ──►  LIGHT: box + body sites only (so chains can be collapsed)
   doc-tests, conflict regions                                           ──►  TAG only
   locals, closures, let bindings                                        ──►  not a box at all; their calls belong to the enclosing box
```

## 14. The byte-coverage invariant — did we look at everything? (TDD log)

**Plain words.** A table of spans is only trustworthy if every byte of every file is inside some span or inside a gap we can name (whitespace, a comment, crate docs, an inner attribute, a parse error). If a single byte is in neither, something that could be an item — or a call into one — is invisible, and the agent would be told "no dependency" when the truth is "we did not look". So the check is an *identity*, not a percentage: for every file, item bytes + classified gap bytes = file length, with no overlaps and no gap of kind "other". The ingest tool enforces it per file; a file that fails it is reported in the banner, never silently mapped.

### 14.1 The rule

```text
 for each file:
   items   = module-level item spans from the one parser pass            (byte ranges, sorted)
   gaps    = every byte between items, classified by the parser's own token/node kind:
             whitespace · comment · doc (//! crate docs) · frontmatter (#! shebang, --- cargo-script header) · inner_attr (#![…]) · error
   invariant 1   items ∪ gaps tile [0, len) exactly: sorted, contiguous, non-overlapping, sum = len
   invariant 2   no gap has kind "other" — every byte outside an item has a name
   reporting     error-gap bytes > 0  → the file appears in the banner as "has unparsed region"
                 doc-gap bytes        → crate/module-level docs; interface text for the module header row
```

Nested items (a fn inside a fn) are inside their parent's bytes, so they are covered by the parent; the parent chain (§12, parent_id) is what makes them addressable. Members of impls and traits likewise sit inside the impl/trait item bytes.

### 14.2 Measured result

```text
 corpus                              files   identity failures   bytes        inside items   whitespace   comment   doc        frontmatter   inner_attr   error    other
 ─────────────────────────────────── ─────── ─────────────────── ──────────── ────────────── ──────────── ───────── ────────── ───────────── ──────────── ──────── ─────
 all .rs on the VM (rust-analyzer,    3,370   0                   31,752,951   97.83 %        89,408       192,670   373,641    247           28,796       3,365    0
 rustc, linux/rust, cargo-modules…)
 the 125-file oracle sample           125     0                   1,102,039    97.79 %        3,042        6,971     12,930     0             1,098        306      0
```

Every one of the 3,370 files tiles exactly (per-row column sum re-checked independently: 0 failures). The 3,365 error bytes are in 120 files, all of them rust-analyzer's own parser *error fixtures* (`test_data/parser/err/…`) and rust-code-analysis's Jinja templates (`enums/templates/*.rs`, not Rust) — i.e. files that are supposed to be broken. No production file has an unparsed region. "other" is zero everywhere.

### 14.3 TDD log — STUB → RED → GREEN → REFACTOR

Crate: `experiments/spancut/coverage` (library + `coverage <file-list> [base-dir]` binary + `examples/dump.rs`).

```text
 STUB      cover_file(&str) -> Coverage { file_len, items: Vec<Item>, gaps: Vec<Gap{start,end,kind}> }
           covered_bytes · is_identity · gap_bytes_by_kind — all todo!(). cargo check green.
 RED       8 tests, all failing on todo!():
           test_sum_equals_file_length · test_empty_file_identity_holds · test_gap_whitespace_comment_classified
           test_inner_attr_gap_classified · test_error_region_gap_flagged · test_nested_items_not_module_level
           test_items_sorted_non_overlapping · test_gap_bytes_sum_matches
 GREEN 1   walk the root node's children: Item nodes → items; ATTR → inner_attr; ERROR → error; WHITESPACE/COMMENT tokens → their kind;
           anything else → other. 7/8 green. test_inner_attr_gap_classified RED: the inner attr was found but "other" bytes appeared.
           First corpus run: identity 3,370/3,370, but 373,785 bytes (1.2 %) of kind OTHER in 1,060 files — exactly the failure the
           invariant exists to catch. examples/dump.rs showed them: `//!` doc comments are DOC_COMMENT *nodes* in this parser version,
           not COMMENT tokens. → new kind Doc. RED test added: test_module_docs_not_other. 9/9 green; other = 144 bytes in 3 files.
 GREEN 2   the 144 bytes: SHEBANG and FRONTMATTER tokens (cargo-script headers in parser fixtures). → kind Frontmatter;
           RED test test_frontmatter_shebang_classified; 10/10 green; other = 0 in 3,370 files.
 GREEN 3   a reporting bug the identity did NOT catch: after `cargo fmt` rewrapped a line, the by-kind report omitted Frontmatter while
           the identity still held (it sums the real gaps, the report sums a list of kinds). Caught only by eye in the per-file output
           (len 109 ≠ 14 reported). → RED test test_by_kind_covers_every_gap (sum of reported kinds == sum of gaps on a file containing
           every kind); fixed; 11/11 green. Second guard added to the binary run: per-row column sum re-checked with awk → 0 failures.
           Also: the 125-file list mixed relative and absolute paths; only 100 were read. Fixed (absolute paths bypass the base dir).
 REFACTOR  cargo fmt · cargo clippy --all-targets -D warnings clean · 11/11 · corpus re-run identical.
```

Lessons recorded: (1) a 100 % identity can still hide a classification hole — the "other" bucket must be asserted zero, not merely reported; (2) a report derived from a *list of kinds* can silently drop a kind; the test must compare it to the ground sum; (3) parser-version detail (doc comments as nodes) is exactly why the parser is pinned and this check re-runs on every bump.

### 14.4 What this settles about the six axes (§13.2)

```text
 axis 1  grammar kind      REQUIRED for coverage — the identity is defined over item kinds + gap kinds. Measured complete: other = 0.
 axis 5  parse status      REQUIRED — error gaps are the only bytes without an item; they must reach the banner. Measured: 120 fixture files.
 axis 2  nesting           not a coverage question — nested bytes are inside the parent; needed for paths and effective visibility (§12).
 axis 3  reachability      not a coverage question — a label on rows; needed for the product (what is public), computable from parents.
 axis 4  origin            not a coverage question — derive/macro/cfg labels; virtual spans point back into covered bytes.
 axis 6  identity in time  not a coverage question — digests over covered bytes; follows from 1 + 5.
```

So: coverage needs axes 1 and 5 and is now measured; axes 2 and 3 are needed for correctness of the product and are computable from the parent chain; 4 and 6 are labels that can be added without touching coverage.
