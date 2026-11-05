# Can an LLM parse Rust better than the compiler? Eight Rust analysis tools read, three measured, and a five-level universal representation

*Written 2026-11-05 (unix 1793864191). Branch `research/tiny-rust-code-mapper`. Companion to the Strata/Clarity thesis (`1793858459-…`) and the earlier decision docs (`1793853341-…`).*

This note answers a direct pushback: *"we cannot be running a compiler for everything … why can't an LLM do that job better? Isn't parsing supposed to be that territory? … 98% accurate is fine … maybe a five-level dependency graph, not just interfaces … check Cassini-1.0."*

I cloned eight Rust analysis tools, read how each one actually obtains control flow and data flow, ran three of them (or their substrate) against real crates with a stopwatch, measured what a parser alone can and cannot decide on 138k call sites, and read the Cassini-1.0 model card. Everything marked **[measured]** was run on this machine (8 cores, 31 GB RAM, rust-analyzer 1.99.1, cargo 1.99.1 stable, Linux); everything marked **[read]** was read in the cloned source; everything else is inference.

---

## 0. Where I now stand (the short version)

1. **You are right that running `rustc` for everything is the wrong design.** But the dichotomy "compiler vs LLM" is false. The most important fact in this note: **rust-analyzer never runs the compiler.** It is a parser plus its own incremental name-resolver and type-inferencer, built to work on broken, half-typed code. It analysed a 42k-LOC, 11-crate workspace fully (every expression typed, 0% unresolved) in 29 s / 2.3 GB **without compiling anything** [measured]. That *is* "parsing territory", done properly. Six of the eight tools below sit on it or on rustdoc, not on `rustc`.
2. **Parsing alone (tree-sitter / `syn`) gets you a complete *item* graph and about 20% of the *call* graph.** 65% of call sites in real Rust are method calls whose target depends on the receiver's type, and roughly a quarter of those are chained calls (`a.b().c()`) that need the return type of a method defined in another file or crate [measured on 138k call sites]. This is the exact gap a "contextual" LLM would have to fill, and it is also exactly what rust-analyzer's `Inference` phase already fills deterministically in a few seconds.
3. **Cassini-1.0 is the strongest evidence for your position and the strongest warning at the same time.** A 4B Qwen3 LoRA (rank 1!) that reads a file and emits imports/definitions/references/calls/type-assignments as JSON, at **77.6% F1** on Python/JS/TS, 17–43× cheaper than frontier models. It validates "small model as extractor". But: 77.6% is not 98%; it targets languages that *have no rust-analyzer equivalent*; and even Cassini feeds a deterministic resolver downstream, it does not replace it.
4. **So the LLM's job in a Rust system is not to re-derive levels 1–4 of the graph.** Deterministic tools produce those at near-100% in seconds, including on broken code. The LLM earns its keep at level 5 (meaning: what a module is *for*, which invariant a function protects, what "the lock ordering" is) and as a fallback extractor when the deterministic ladder cannot run (no `Cargo.toml`, snippets, patches, foreign build systems, 10 M LOC where loading everything is too expensive).
5. **Yes to five levels.** Not five *kinds* of graph, but one representation with a `level` field, where each record says which substrate produced it and how sure it is. The ladder already exists inside rust-analyzer (Syntax → ItemTree → DefMap → HIR/Types → MIR). Our job is to externalise it, make it addressable, and give the agent a budget knob over it.
6. **"LLM + CLI tools + markdown references" is the right form**, and the tools table below tells you which CLIs. The model orchestrates and interprets; it does not pretend to be a resolver.

---

## 1. The eight tools, and what each one actually does for control/data flow [read]

| Tool | What it computes | Substrate (how it *really* gets the data) | Toolchain needed | Works on code that doesn't build? | Size (Rust LOC, this clone) |
|---|---|---|---|---|---|
| **rust-analyzer** (`rust-lang/rust-analyzer`) | Full semantic model: module tree, name resolution, macro expansion, type inference, method resolution, call hierarchy, references, LSIF/SCIP export, MIR lowering + interpreter | Own parser (rowan) → `ItemTree` → `DefMap` → HIR bodies → Chalk/next-gen-solver types → MIR. Uses `cargo metadata` only to learn the crate graph; optionally `cargo check` for build-script `OUT_DIR`s and a proc-macro server. **Never invokes `rustc` for analysis.** | stable; `rust-src` for std | **Yes.** Core invariant in `AGENTS.md`: "malformed syntax, broken builds, and proc-macro failures must not cause ordinary IDE features to panic". Verified below. | 593k |
| **Flowistry** (`willcrichton/flowistry`) | Information flow (which statements can influence which), per function body; "focus mode" | `rustc_plugin` driver: runs as a replacement `rustc` inside `cargo check`, takes `BodyWithBorrowckFacts` from `rustc_borrowck`, runs a custom `rustc_mir_dataflow` analysis over MIR. PLDI 2022 paper "Modular Information Flow through Ownership" (arXiv 2111.13662) | **nightly** pinned (`nightly-2026-05-01`, `rustc-dev`, `llvm-tools-preview`) | No: needs borrowck-clean MIR of the whole crate | 5.1k |
| **cargo-modules** (`regexident/cargo-modules`) | Module tree; item-level `Owns`/`Uses` dependency graph; orphan files | `ra_ap_*` crates (rust-analyzer as a library, pinned `=0.0.358`): `load_workspace` → `hir::Crate` → walk every `ModuleDef`; `Uses` edges come from walking the *types* in fn params/return (`walk_and_push_type`: ADTs, `dyn Trait`, `impl Trait`, assoc-type parents) | stable | Yes (inherits rust-analyzer) | 8.9k |
| **cargo-semver-checks** (`obi1kenobi/cargo-semver-checks`) | Breaking-change detection between two versions of a crate's public API; 270+ lints | **rustdoc JSON** of baseline and current, loaded through `trustfall_rustdoc` and queried with **Trustfall** (GraphQL-like). Each lint is a `.ron` file holding a query, e.g. `function_missing` = "a `Function` with `importable_path` public in baseline, count = 0 in current". Lints carry a `witness` template that generates a *downstream program that would break*. | **nightly rustdoc** (`-Z unstable-options --output-format json`), via rustup | Needs `cargo doc` to succeed on both versions | 42k |
| **cargo-public-api** / `public-api` | Renders a crate's public API as a sorted list of Rust-like item signatures; diffs two versions | rustdoc JSON → `ItemProcessor` → `IntermediatePublicItem` → `render.rs` emits a **token stream** per item (this is essentially the "Rust skeleton" view) | **nightly** ≥ `nightly-2025-11-22` for the JSON format version | Needs `cargo doc` to succeed | 9.0k |
| **RustPräzi** (`praezi/rust`) | Call-based dependency network of *all of crates.io* (Hejderup, Beller, Gousios 2018) | `cargo rustc --lib -- --emit=llvm-bc` for every crate, then LLVM `opt -dot-callgraph`, `rustfilt` demangling, Python merge into one ecosystem graph | pinned old Rust (1.23/1.24) + LLVM 4.0 binaries, GNU parallel, "a very powerful system" | No: every crate must compile | 0.9k Rust + Python |
| **cargo-call-stack** (`japaric/cargo-call-stack`) | Whole-program call graph + max stack usage (embedded) | Builds with fat LTO and `--emit=llvm-ir`, parses the `.ll` text for `call` instructions and `-Z emit-stack-sizes`; indirect calls / `dyn Trait` are admitted to be "imperfect" | **nightly** (`-Z stack-sizes`), last tested 2023 | No | 5.0k |
| **cargo-geiger** (`geiger-rs/cargo-geiger`) | Counts `unsafe` usage per crate in the dependency tree | `syn` visitor (`visit_item_fn`, `visit_expr_unsafe`, `visit_item_impl`, …) over each `.rs` file; **but** it learns *which* files are actually compiled by implementing cargo's `Executor` trait and intercepting every `rustc` invocation (`custom_executor.rs`) | stable (links the `cargo` crate) | Partially: parsing is per file, but the file list comes from a build | 9.2k |
| **rust-code-analysis** (Mozilla) | Metrics (cyclomatic, cognitive, Halstead, MI, LOC, nargs, nexits) per function "space", 10 languages | **tree-sitter** (`tree-sitter-rust =0.23.2`); e.g. Rust cyclomatic = count of `If | For | While | Loop | MatchArm | TryExpression | && | \|\|` nodes; functions/closures found by node kind | stable, no cargo at all | **Yes, any file** | 28.5k |

Three things fall out of the table.

**(a) The "compiler" is not one thing; it is a ladder of six substrates**, from cheapest/dumbest to most expensive/most exact:

```
  substrate            needs            cost on 42k LOC       knows
  ─────────────────    ──────────────   ──────────────────    ────────────────────────────────────────
  1 tree-sitter        a file           0.07 s   [measured]   syntax, item names, call *shapes*, metrics
  2 syn                a file           ~same                 same, typed Rust AST
  3 rust-analyzer      files + cargo    29 s / 2.3 GB cold    resolved names, macro-expanded items,
    (hir)              metadata         [measured]            types, method targets, impls, references
  4 rustdoc JSON       cargo doc        ≈ a cargo check       the *public* contract, exactly, versioned
    (nightly)                                                 format; no bodies
  5 rustc_private/     nightly +        ≈ cargo check + ε     MIR, borrowck facts, dataflow, exact
    MIR (Flowistry)    whole crate                            information flow
  6 LLVM IR            full build       minutes–hours         monomorphised, post-inline call graph
    (Präzi, call-stack) (+LTO)                                 (what the machine actually calls)
```

Only rungs 5–6 are "running the compiler". Rung 3 is what you were asking for: parsing done well enough to resolve. Rungs 1–2 are what Parseltongue and Clarity do today.

**(b) Nobody who needs *correct* flow edges uses rungs 1–2.** Geiger uses `syn` but only to count keywords, and even then it asks cargo which files are real. Mozilla uses tree-sitter but only for metrics that are definable on syntax (cyclomatic complexity is literally a node count). Every tool that emits a call or type edge goes to rung 3 or above. Clarity's Rust adapter is the exception, and its README disclaims symbol-level call graphs for that reason.

**(c) Every rung above 3 requires a successful build, and rung 4 requires nightly.** That is a real operational cost, and it is the legitimate core of your objection. The answer is not to replace rung 3 with a model; it is to make rung 3 the default and reserve 4–6 for the questions that need them (semver, information flow, exact monomorphised calls).

---

## 2. Measurements [measured]

### 2.1 How expensive is "not running the compiler", really?

Workspace: `cargo-semver-checks` (42k LOC, 11 workspace crates + sysroot, 1121 declarations, 1057 bodies, 34 618 expressions). All runs on the same 8-core box; `cargo check` cold includes compiling ~300 dependency crates.

| Step | Wall time | Peak RSS | Notes |
|---|---|---|---|
| tree-sitter parse of all 30 workspace `.rs` files (Python bindings) | **0.07 s** | – | 7.8 MB/s; 0 `ERROR` nodes |
| `rust-analyzer analysis-stats --disable-build-scripts --disable-proc-macros` (full: item tree, name resolution, bodies, inference) | **29.0 s** (31.6 s wall) | 2.3 GB (2.9 GB RSS) | Item-tree collection 13.9 s, item collection 10.7 s, inference 3.4 s. Unknown types: 83 of 34 618 exprs (0.2%), with proc macros *off* |
| `cargo check --workspace`, cold | 55.1 s | 0.39 GB | compiles all deps |
| `cargo check --workspace`, after `touch src/lib.rs` | **0.69 s** | 0.19 GB | incremental |

Two more workspaces for scale: `rust-code-analysis` (15 crates, 51 893 exprs): rust-analyzer 35.0 s / 2.0 GB, unknown types 0.1%. `cargo-geiger` (7 crates but the whole `cargo` crate as a dependency, 14 534 exprs): 21.4 s / 2.6 GB, unknown types 0.1%.

Reading: on a 42k-LOC workspace, a *complete* resolved model costs half a minute and two gigabytes with no compilation; the resolution is essentially total (0.1–0.2% unknown types, mostly because proc macros were disabled). Roughly 80% of that time is spent in the dependency crates and the standard library, which are identical across projects and cacheable. And note the incremental compiler is not the villain either: a warm `cargo check` is sub-second here. The "compiler takes five minutes" regime is real at 10 M LOC, but that regime kills *every* whole-program approach, including a model that wants to see every file; the fix there is incrementality and caching, which rust-analyzer's salsa database is designed around ("typing inside a function's body never invalidates global derived data", architecture.md).

### 2.2 Does rust-analyzer survive broken code?

I copied `cargo-public-api`, added `use crate::does_not_exist::Missing;` at line 1 of `public-api/src/render.rs` (which also makes the `#![allow]` inner attribute on line 2 a syntax error) and spliced `let x: Missing = ;  fn (broken {{` into line 40. Result: `analysis-stats` completed normally (14 crates, 11 980 exprs typed, 11.7 s), `rust-analyzer diagnostics` reported the syntax errors in `render.rs` with positions and carried on to report type errors in unrelated crates, and the LSIF export still contained the `render.rs` document. That is the behaviour we want from a graph builder on a mid-edit snapshot, and it is already implemented.

### 2.3 What can a parser alone decide? 138k call sites classified

Script: `tree-sitter-rust`, every `call_expression` in a repo, classify the callee syntactically and check whether a bare identifier or path head is defined or imported *in the same file*. This is a generous proxy for "what a per-file extractor (tree-sitter, `syn`, or an LLM reading one file) can resolve without looking elsewhere".

| Callee shape | cargo-semver-checks (4 268 calls) | rust-analyzer repo (133 536 calls) | Decidable from the file alone? |
|---|---|---|---|
| `recv.method(..)` (field_expression) | **65.2%** | **65.6%** | **No** — target depends on receiver type and on which traits are in scope |
| path rooted in a local `use`/def (`Foo::new`, `module::f`) | 10.8% | 10.6% | Mostly yes (needs the module tree to finish the path) |
| bare fn defined in the file | 8.0% | 9.7% | Yes |
| prelude ctor/fn (`Some`, `Ok`, `Box`, `Vec`, `String`, …) | 7.6% | 4.6% | Yes (fixed list) |
| path with unknown root (`fs_err::`, `toml::`, `serde_json::`, `Self::`, `Default::`) | 5.5% | 3.6% | Needs the crate graph (extern crate names) or impl context (`Self`) |
| bare fn **not** defined/imported in the file (glob `use`, macro-generated, nested fn, closure variable) | 1.9% | 4.6% | No |
| turbofish `f::<T>(..)` | 0.5% | 0.8% | Partially |
| `crate::`/`self::`/`super::` paths | 0.4% | 0.2% | Yes, given the module tree |

So a file-local extractor nails roughly **20%** of call sites outright (local fn + prelude + crate-rooted), gets another ~10% with a module tree, and is blind on the 65% that are method calls. Now the receivers of those method calls:

| Receiver of `recv.method(..)` | cargo-semver-checks (2 781) | rust-analyzer (87 472) | What you need to type it |
|---|---|---|---|
| chained call `a.b().c()` | **31.3%** | **24.3%** | return type of `b`, usually declared elsewhere |
| `let x = <expr>` (untyped) | 15.1% | 10.6% | inference of the rhs |
| typed parameter `x: T` | 9.1% | 13.1% | **local**: read the signature |
| pattern / `for` / `if let` / closure binding | 8.2% | 12.0% | inference |
| `let x = Type::ctor(..)` or struct literal | 7.9% | 3.9% | **local-ish**: path head names the type |
| `f(..).m()` | 7.2% | 2.0% | return type of `f` |
| `self` | 1.2% | 7.5% | **local**: enclosing impl |
| `self.field` | 2.8% | 4.8% | struct definition, same crate |
| identifier not bound in the enclosing fn (static, const, capture) | 2.5% | 10.2% | crate-level lookup |
| other (field chains, literals, `?`, macros, indexing) | 14.7% | 11.6% | mixed |

The locally decidable receiver classes (`self`, typed param, `let x: T`, `Type::ctor(..)`, `Type::ctor(..).m()`) sum to **≈22% (semver-checks) / ≈26% (rust-analyzer)** of method calls. Everything else needs information from another item, usually another file: a return type, a field type, a trait impl. **Receiver type is a global property computed locally**; that is why a per-file model plateaus.

Rough ceiling, both repos: *file-local parsing + local type heuristics ≈ 20% + 65% × 25% ≈ 36–40% of call edges correct, before any cross-file work.* Add a crate-wide signature table (return types of every fn/method, field types, impl blocks — all of which a parser *can* extract because they are syntactic in Rust) and a chained-call resolver, and the ceiling rises a lot; but at that point you have re-implemented rust-analyzer's `hir-ty` crate (63k LOC) badly. **This is the quantitative reason the Rust extractor should sit on rust-analyzer rather than on a parser-plus-model.**

### 2.4 Where the deterministic answer comes from, by locality (LSIF of cargo-public-api)

Of 10 340 resolved references in `cargo-public-api` (rust-analyzer LSIF, 18 s, 1.1 GB): 43.2% resolve to a definition in the **same file**, 9.1% to the same crate, 3.8% to another workspace crate, 13.6% to an **external crate** in the registry, **30.3% to the standard library**. So 44% of what any reference means lives outside the workspace entirely. A model that has not memorised `std` and the popular crates' signatures cannot type those receivers; rust-analyzer loads them from `rust-src` and the registry source in seconds. (Also a nice consequence: whatever we build should index std and the top-N crates once, globally, and share it.)

---

## 3. Cassini-1.0, read carefully

Model card facts (huggingface.co/LatentForce-ai/Cassini-1.0, fetched today): base `Qwen/Qwen3-4B`; LoRA with **rank 1, alpha 2**, 5 epochs, lr 8e-4, 20 480-token context, Apache-2.0; trained for Python, JavaScript, TypeScript; served with vLLM; output is one JSON object per source file with six fields: `imports`, `definitions`, `references`, `calls`, `type_assignments`, `definitions_rich`. Claimed **77.6% F1** at $0.17 total inference cost on their benchmark, vs 77.6% / $3.03 for "Qwen3.7-Max", 74.2% / $5.84 "Gemini 3.5 Flash", 72.5% / $7.57 "Gemini 3.1 Pro". The prompt template (`prompts/python.yaml`, v6) is a long rule list: include internal imports, exclude stdlib/external, include private helpers, exclude dunders and nested functions, record "first attribute accessed" for module imports, etc. The outputs feed a separate deterministic "Unified Dependency Analyzer" / resolver (`resolver: "python"` in the YAML), with a `prompt_version` used to invalidate an incremental extraction cache.

What it proves for our purposes:

- **The "small model as extractor" architecture ships.** A 4B model, trivially small LoRA, one pass per file, structured JSON. This is exactly the shape the earlier Parseltongue docs hypothesised. The `type_assignments` field ("`self.db = Database()` → so later `self.db.query()` resolves") is precisely the receiver-typing problem from §2.3, attacked by a model.
- **77.6% F1 is the current state of the art for that architecture**, with a purpose-trained model, on languages that have *weak* static alternatives. 98% is a long way from 77.6%, and the gap is concentrated in exactly the cases a per-file model cannot see (what `build_result()` returns, which class `self.cache` is).
- **They did not do Rust**, and I think the reason is instructive: for Rust, rust-analyzer gives ~100% at rung 3 for free, so a 77%-F1 extractor has no market *as a resolver*. It would have one as (i) a fallback for code that cannot be loaded as a crate and (ii) a semantic layer on top of the resolved graph.
- **Even Cassini keeps the resolver deterministic.** The model proposes per-file facts; code joins them across files. That is the "model proposes, validator disposes" division of labour from the thesis, confirmed by a team that bet on the model.
- The honest comparison for Rust, per crate: rust-analyzer ≈ 20–35 s CPU, ~2 GB, ~100% resolved, works mid-edit; Cassini-style ≈ one GPU forward pass per file (≈ 1–5 s per file on a consumer GPU at 20k context, my estimate), ~78% F1 for Py/JS, unknown for Rust, works on anything. The model is only the better tool when the deterministic one *cannot run*, or when the question is not one the deterministic one answers.

---

## 4. Where an LLM genuinely beats the compiler on "understanding" — and where it doesn't

Your intuition that a model has "a more contextual understanding of code, something even a compiler will take 5 minutes to understand" is right about *one kind* of understanding and wrong about another. Separate them:

| Question | Compiler-class tool (rung 3–5) | LLM |
|---|---|---|
| What does `x.foo()` call? | exact, ms–s, including trait dispatch | guesses; wrong whenever the receiver type is non-local (≥ 60% of method calls) |
| Who implements trait `T`? | exact and enumerable (coherence) | cannot enumerate; recalls |
| Does changing this signature break downstream crates? | exact (cargo-semver-checks, 270 lints, with a witness program) | plausible list |
| Which statements can influence this value? | exact modulo interior mutability (Flowistry, MIR) | plausible |
| **What is this module *for*? Which invariant does this `unsafe` block rely on? What is the lock order? Is this the "hot path"? Which of these 40 public items are the real entry points?** | no answer at all | **good, and nothing else gives it** |
| **A snippet, a diff, a file outside any `Cargo.toml`, a repo whose build system we cannot drive** | nothing | **something** (Cassini regime) |
| Summarise a 300-node subgraph into 20 lines for another agent | nothing | the whole point |

So the five-level graph needs a **sixth, non-structural layer**, and that is where the model's "contextual understanding" lives: role/purpose/invariant annotations attached to exact nodes. The compiler gives the skeleton; the model puts names on the bones. Running the model to re-derive the skeleton is the one job where it is both slower and worse.

---

## 5. Patterns worth stealing, tool by tool [read]

1. **rust-analyzer `ItemTree` as an invalidation barrier.** Per-file, crate-independent summary of items ("doesn't know which `#[cfg]`s are active or which module it belongs to"); editing a body never invalidates it. Our level-2 records should have exactly this contract: a hash of the item tree per file, so a body edit re-derives only level-4 for that body.
2. **rust-analyzer `DefMap` fixed-point algorithm** (`nameres.rs` doc comment): collect raw items → collect modules → iterate: resolve imports, expand macros, repeat until no progress. Unresolved imports and macros remain *as records*. If we ever write a lighter resolver, that is the algorithm; and "unresolved" is a first-class output, not an error.
3. **rust-analyzer `Semantics` / `source_to_def`**: the syntax↔semantics mapping is one-to-many (macros). Our span-based evidence must allow one source range to back several graph records and vice versa.
4. **cargo-modules `walk_and_push_type`**: `SIGNATURE_USES` edges are "walk every type in params/return; emit ADT, `dyn Trait`, `impl Trait`, and assoc-type parent traits". Fifty lines; copy it.
5. **cargo-semver-checks: lints are queries, queries are data, every lint has a witness.** 270 `.ron` files, each a Trustfall query over the rustdoc-JSON graph plus a template for a program that would break. For us: architecture rules (`no_std` crates must not reach `std`, drivers must not import fs internals) should be query files over the graph, with a witness, not Rust code.
6. **cargo-public-api `render.rs`**: the rendering of an item as a token stream (`pub fn open(path: &Path) -> Result<Store>`) sorted by path is the "Rust skeleton" view; a solved problem, including generics and where-clauses.
7. **cargo-geiger's cargo `Executor` intercept**: ask cargo which `.rs` files it actually hands to `rustc` for the active target/features. That is the cheapest honest answer to "which `cfg` is live"; it does not require the build to succeed past the invocation.
8. **Flowistry's modularity theorem**: information flow computed per function body using only the *signature* of callees (ownership makes this sound). That is the theoretical justification for the private-path collapsing in our earlier doc (`A -> B via [h1]`): in Rust, a body's effects on the outside world are bounded by its signature.
9. **Präzi's pipeline shape**: per-crate graph → demangle to a universal function identifier (`ufi`) → prune → merge into one ecosystem graph → infer the package graph *from* the call graph. The `ufi` is exactly our "stable semantic id" problem, solved in 2018 for monomorphised symbols.
10. **rust-code-analysis "spaces"**: a function/closure/impl/trait/module boundary is a *space*; metrics are computed per space from node kinds and aggregated upward (`_sum`, `_min`, `_max`, `_average`). That is the cheap, parser-only level of our ladder, with multi-level aggregation already designed.
11. **Clarity's evidence record** (from the earlier note): `{symbol, kind, relationship, reference_file:line, declaration_file:line, confidence}`. Keep it, add `level` and `substrate`.

---

## 6. The universal Rust representation: one schema, five structural levels, one semantic level

Not five graphs. One stream of records, each tagged with the level it belongs to and the substrate that produced it, addressable by a stable id. An agent (or a CLI) asks for a *projection*: "levels ≤ 2 for the whole workspace, level 4 for crate X, level 6 summaries everywhere", under a token budget.

```
level  name            node kinds                        edge kinds                              substrate(s)                 exactness
─────  ──────────────  ────────────────────────────────  ──────────────────────────────────────  ───────────────────────────  ─────────────────
L1     workspace       Workspace, Crate(version,         DEPENDS_ON_CRATE(features, kind)        cargo metadata (or Cargo.toml  exact
                       features, edition, targets)                                               parse if cargo unavailable)
L2     items           Module, Fn, Struct, Enum, Union,  CONTAINS, DECLARES_MOD, USE(import),    tree-sitter/syn ItemTree      exact for what is
                       Trait, Impl(unnamed→id by         RE_EXPORT (syntactic)                   (per file, no resolution)     written; blind to
                       trait×self×generics), TypeAlias,                                                                        macros & cfg
                       Const, Static, Macro, Field,
                       Variant, AssocItem, ExternBlock
L3     resolved        same nodes, now with canonical    EXPOSES_AS (resolved re-exports),        rust-analyzer DefMap +        exact; includes
       items           paths, visibility *reach*, cfg    SIGNATURE_USES, IMPLEMENTS, DERIVES,     hir (or rustdoc JSON for the  macro-generated
                       predicate, macro-generated flag   MACRO_GENERATES, SUPERTRAIT               public subset)                items
L4     bodies          Body (per fn), Closure folded     CALLS(static/trait/dyn, via:[private]),  rust-analyzer hir-ty          exact modulo
                                                         CONSTRUCTS, READS_FIELD/WRITES_FIELD,     (inference + method           dyn fan-out and
                                                         PANICS_VIA, UNSAFE_USES, AWAITS           resolution); optional MIR     unknown types
L5     flow            same                               INFLUENCES (information flow),           Flowistry / MIR dataflow;     exact, needs
                                                         LOCK_ORDER, SEND_SYNC_BOUNDARY,           rustc_private, nightly        build
                                                         MAY_ALIAS
L6     meaning         Summary, Role(entry point,         ABOUT, PROTECTS(invariant), HOT_PATH,    LLM over L1–L5 projections    not exact;
                       plumbing, driver, test…),          OWNS_CONCEPT, SPEC_DIVERGES (reflexion)  + docs + commit history       confidence-scored,
                       Invariant, Concept                                                                                         re-derivable
```

Record shape (JSON lines, one per node or edge; the fields are the Clarity schema plus the three that make it multi-level):

```json
{"id":"semver_checks@0.44::query::SemverQuery::from_ron",
 "kind":"Fn","level":3,"substrate":"ra-hir","substrate_rev":"1.99.1",
 "vis":"pub(crate)","reach":"crate","cfg":null,"macro_generated":false,
 "span":{"file":"src/query.rs","bytes":[1220,1890]},"src_hash":"blake3:…",
 "sig":"pub(crate) fn from_ron(s: &str) -> Result<Self, ron::Error>",
 "confidence":"exact"}
{"src":"…::SemverQuery::from_ron","dst":"ron::de::from_str","rel":"CALLS","level":4,
 "substrate":"ra-hir","dispatch":"static","via":[],"span":{…},"confidence":"exact"}
{"src":"…::query","rel":"ABOUT","level":6,"substrate":"llm:qwen3-4b-ft@r12",
 "text":"Loads lint definitions from .ron files and compiles them into Trustfall queries; the single entry point other modules use is SemverQuery::all_queries.",
 "evidence":["…::SemverQuery::from_ron","…::SemverQuery::all_queries"],"confidence":0.8}
```

Rules that make it "universal":

- **Same id at every level.** L2 (parser) and L3 (resolver) records for the same item share the id once the resolver confirms the path; before that, L2 records carry a *provisional* id (`file:line:name`) that is upgraded, never silently replaced.
- **Every record names its substrate and confidence.** `exact` from rung 3+, `syntactic` from rung 1–2, a number from a model. Agents can filter: "give me only exact edges" or "include model guesses but mark them".
- **Lower levels never depend on higher ones**, so the graph degrades gracefully: no cargo → L1 from `Cargo.toml` text, L2 from tree-sitter, L3/L4 missing or model-proposed; cargo but no nightly → everything but L5 and the rustdoc-JSON cross-check; full toolchain → everything.
- **Budgeted projections are the API**, not the dump: `surface(scope, levels, budget_tokens)` returns the most-connected items first (Clarity's `show-surface` idea), with `via` collapsing of private paths at L4.
- **98% is a per-level statement.** L1–L3 should be ~100% exact whenever rust-analyzer can load the crate; L4 ~99% (the remainder is `dyn` fan-out and proc-macro bodies); L5 only where built; L6 is scored, never "accurate". Asking for "98% overall" is asking for L3 plus most of L4, which the deterministic ladder gives for free. The model-only path (Cassini regime) should be expected to land near 75–85% on L2–L4 for Rust until proven otherwise, and that is fine *as a fallback with a lower confidence tag*.

---

## 7. The form of "the LLM we create": model + CLIs + markdown

```
                     ┌───────────────────────────────────────────────┐
  agent / human ───► │  small model (orchestrator + L6 + fallback)   │
                     └──────┬───────────────┬───────────────┬────────┘
                            │ calls         │ reads         │ emits, when tools can't run
                            ▼               ▼               ▼
                 ┌──────────────────┐ ┌─────────────┐ ┌─────────────────────────┐
                 │ CLIs (exact)     │ │ markdown    │ │ L2–L4 records tagged    │
                 │ rsgraph extract  │ │ schema.md   │ │ substrate=llm,          │
                 │   --levels 1-4   │ │ queries.md  │ │ confidence<1            │
                 │   (ra_ap_* based)│ │ cookbook.md │ │ validated by rsgraph    │
                 │ rsgraph ts       │ │ rust-rules  │ │ when a crate loads later│
                 │   (tree-sitter,  │ │ .md (cfg,   │ └─────────────────────────┘
                 │   any file)      │ │ macros, dyn)│
                 │ rsgraph query    │ └─────────────┘
                 │   (Trustfall-ish)│
                 │ rsgraph diff     │
                 │ cargo semver-chk │
                 │ cargo doc --json │
                 └──────────────────┘
```

- **CLIs are the extractors.** `rsgraph extract` is a thin binary on `ra_ap_hir`/`ra_ap_ide` (cargo-modules is the 9k-LOC existence proof; `rust-analyzer lsif/scip` already emits the raw material) that writes L1–L4 records for a workspace, with the std/registry index cached globally. `rsgraph ts` is the tree-sitter path for anything that is not a loadable crate. Rung 4–6 tools are invoked only on demand.
- **Markdown is the shared memory.** Schema, the query cookbook ("what breaks if I change X" = `impact(id, upstream, depth=2, levels≤4)`), and the Rust rules the model must respect (how `cfg` splits the graph, why macro-generated items carry a flag, why `dyn` edges are `MAY_CALL`). This is what Strata ships as `describe_kit` and Cassini ships as `python.yaml`.
- **The model does three things**: plan which projection to ask for; write L6 (summaries, roles, invariants, reflexion verdicts against a spec skeleton); and act as the Cassini-style extractor when the CLIs cannot run, with its output validated by `rsgraph` the moment the crate becomes loadable. Training data for the third role is free: run the CLI on crates that load, train the model to reproduce it from single files, score with edge F1, exactly Cassini's recipe but with a 100%-exact teacher.

---

## 8. What I concede, what I hold, and the experiment that settles it

**Conceded.** (i) Running `rustc`/`cargo doc`/LLVM per edit is the wrong default; the earlier docs leaned too hard on rustdoc JSON (nightly, needs a successful build). (ii) One level (public interfaces) is not enough; the multi-level stream above is the right object. (iii) A model-based extractor is a shipped, cost-effective architecture (Cassini), and we should have one as the fallback rung. (iv) "Map that helps the agent converge" is the correct success metric, not edge-exactness per se; the earlier benchmark proposal (task completion on held-out repos, with token budget) stands.

**Held.** (i) For Rust the deterministic rung 3 is not "the compiler", it is a parser that finishes the job, it runs on broken code, and it is ~100% exact in ~30 s on a 42k-LOC workspace; a per-file model's structural ceiling is ~40% of call edges before cross-file work and ~78% F1 end-to-end in the best shipped instance. Putting the model *under* rust-analyzer (fallback) rather than *instead of* it is not conservatism, it is arithmetic. (ii) The model's unique value is L6 and the no-toolchain regime; that is where training effort should go.

**The experiment** (two weeks, no new infrastructure beyond the scripts in this note): take 50 crates that `rust-analyzer` loads; emit L2–L4 with `rsgraph extract` as ground truth; measure edge-level precision/recall for (a) tree-sitter + file-local heuristics, (b) tree-sitter + a crate-wide signature table + chained-call resolver, (c) a 4B model fine-tuned Cassini-style on the ground truth of the *other* 49 crates, (d) (c) with the signature table in the prompt. Report F1 per level and per callee class from §2.3, plus wall time and memory per crate for each. If (d) reaches ≥ 95% on L4 `CALLS` at ≤ 2× the cost of rust-analyzer, I am wrong about the fallback-only role and the model should be promoted. If it lands near Cassini's 78%, this note stands.

---

## 9. Provenance and caveats

- Repos cloned 2026-11-05 at depth 1 into `~/repos/rust-tools/`: `willcrichton/flowistry`, `rust-lang/rust-analyzer`, `regexident/cargo-modules`, `obi1kenobi/cargo-semver-checks`, `praezi/rust`, `japaric/cargo-call-stack`, `geiger-rs/cargo-geiger`, `mozilla/rust-code-analysis`, `cargo-public-api/cargo-public-api`. LOC are `wc -l` over `*.rs` in each clone. Statements about substrates come from `Cargo.toml` dependencies, `extern crate rustc_*` lines, and the named source files (`crates/flowistry_ide/src/plugin.rs`, `src/graph/builder.rs`, `src/lints/function_missing.ron`, `src/bin/prazi.rs`, `cargo-geiger/src/scan/rs_file/custom_executor.rs`, `src/metrics/cyclomatic.rs`, `crates/hir-def/src/nameres.rs`, `crates/hir-def/src/item_tree.rs`, `docs/book/src/contributing/architecture.md`).
- Measurements: single runs, one machine, some overlap between jobs (the `cargo check` cold run and a tree-sitter probe overlapped briefly; rust-analyzer runs were alone). Treat times as ±20%. `analysis-stats` was run with `--disable-build-scripts --disable-proc-macros --skip-mir-stats --skip-const-eval --skip-data-layout`, so the "unknown type" percentages are *pessimistic* for proc-macro-heavy code and would improve with the proc-macro server on. Running `analysis-stats` on the rust-analyzer repo itself hit a `str` char-boundary panic in the stats printer (a test fixture containing `│`), so the 593k-LOC data point is missing; the tree-sitter probe on that repo did complete.
- The call-site and receiver classifiers (`exp/ts_probe.py`, `exp/ts_receivers.py`, kept outside this repo) are regex-assisted syntactic heuristics, deliberately generous to the parser; the true file-local ceiling is at or below the numbers given. 601 tree-sitter `ERROR` nodes in the rust-analyzer repo are concentrated in parser test fixtures and were not excluded.
- Cassini facts are from the public model card and `prompts/python.yaml` only; the benchmark behind 77.6% F1 is not described there, and the competitor model names on the card are as the card states them. No weights were run.
- Flowistry paper: Crichton, Patrignani, Agrawala, Hanrahan, "Modular Information Flow through Ownership", PLDI 2022, arXiv 2111.13662 (link in the repo README). Präzi: Hejderup, Beller, Gousios, "Präzi: From Package-based to Precise Call-based Dependency Network Analyses", 2018 (linked from the RustPräzi README). rustdoc JSON: RFC 2963. Trustfall: `obi1kenobi/trustfall`. salsa: `salsa-rs/salsa`.
