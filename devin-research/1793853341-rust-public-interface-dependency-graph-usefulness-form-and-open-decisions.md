# Rust Public-Interface Dependency Graph: Usefulness, Form, and Open Decisions

Context: this builds on `research/tiny-rust-code-mapper` (docs-lossfunk/01, 02, 03). Those documents already cover model shortlists, product timelines, and gates, so this note does not repeat them. It adds a direct verdict, a concrete graph form, and a decision register. Facts about rustdoc JSON, rust-analyzer and SCIP come from my general knowledge, not from the repo. Verify them before committing to them.

---

## 1. Is this useful? Yes, but not for the reason the pitch gives

### Why the pitch is weaker than the real opportunity
- **"Better than Tree-sitter" is too low a bar.** Tree-sitter is a parser, not a resolver. Any import-aware resolver beats it on edges. Reviewers will ask the real question: how does the model compare with **rustdoc JSON** and **rust-analyzer**? Those compiler-backed tools already produce an exact public API and exact references *for crates that build*.
- **Codex-generated labels are the wrong teacher for most of the graph.** For any crate that compiles, you can get labels that are free, exact, and scalable:
  - `cargo +nightly rustdoc -- -Z unstable-options --output-format json` gives the public items, re-exports, impls, generics, bounds, and fields.
  - `rust-analyzer scip .` (or LSIF) gives a definition and reference for every occurrence.

  Codex labels cost money, contain errors, and cannot be checked by anyone else. Use frontier LLMs only to adjudicate cases where the tools disagree.

### Where a small model genuinely wins (the niche to claim)
1. **Source that does not build:** a partial checkout, a missing toolchain or system libraries, a broken branch mid-refactor, or a single file pasted into an agent. Compiler tools produce nothing here; Tree-sitter produces unresolved names.
2. **Speed and incremental updates on huge workspaces.** rust-analyzer needs gigabytes of RAM and minutes of indexing on large workspaces. A per-file model plus a host-side linker can update a single file in milliseconds.
3. **Macro-shaped semantics without expansion.** For example, `#[derive(Serialize)]` implies `IMPLEMENTS serde::Serialize`, `#[tokio::main]` wraps `main`, and `thiserror` generates `Display` implementations. These patterns are learnable from the compiler output.
4. **Agent token economics.** A signatures-only view of a crate plus resolved edges can let an agent navigate with a fraction of the tokens. The "5%" figure is a hypothesis; measure it per crate.

### Reframed thesis
> Distill rustdoc and rust-analyzer into a small model. It gives compiler-quality interface graphs where compilers cannot run, and Tree-sitter speed where they are too slow.

The success criterion then becomes measurable:
- On crates that build: reach X% of the compiler graph's per-relation precision and recall.
- On source that does not build: beat Tree-sitter plus heuristics by Y.
- Do both within Z MB of RAM and W ms per file.

### Reality checks on the claims in the post
- **"~30M LOC operating system."** That size describes Linux, which is C. Rust operating systems (Redox, Theseus, Asterinas) are far smaller. Rust-in-Linux and very large monorepos are the realistic scale targets. Unverified; check the numbers before using them publicly.
- **"<100 MB RAM."** At 4-bit quantization, a 135M-parameter model needs about 68 MB for weights alone. A 0.5B model needs about 250 MB. KV cache, tokenizer, parser and graph store come on top. Doc 02 is right to target quality under 4 GB first and compress later.
- **"Top 50 crates" as training data.** These crates are heavily represented in base-model pretraining, which creates contamination risk. Split held-out data by repository, not by file. Include unpopular crates in the test set.

---

## 2. What form the graph should take

Use two layers: a **canonical store** that is exact, evidence-bearing and machine-facing, and **projections** that are compact and LLM-facing. Most debate goes away once these are kept separate.

### 2.1 Nodes

| Kind | Notes |
|---|---|
| `Crate` | name, version, edition, features enabled |
| `Module` | file-backed or inline |
| `Fn`, `Struct`, `Enum`, `Union`, `Trait`, `TypeAlias`, `Const`, `Static`, `Macro` | standard items |
| `Impl` | **first-class node.** Impl blocks have no names, so key them by `(trait?, self_ty, generics-hash)` |
| `AssocItem` | method, associated type, or associated const belonging to a trait or impl |
| `Field`, `Variant` | needed because public fields and variants are part of the contract |
| `ExternalItem` | a stub for items in dependency crates: path plus crate version, nothing more |

### 2.2 Identity (an existing design issue in Parseltongue)
The current key is `rust:fn:main:src_main_rs:1-50`. It **contains line ranges**, so it changes whenever someone edits code above the item. That breaks diffs, caches, and training-label alignment.

Use a semantic key instead:
- Shape: `crate@ver::module::path::Item` plus a kind discriminator, with impl disambiguators such as `impl<T> Display for Wrapper<T>`.
- Store spans and content hashes as attributes, not as part of the identity.

This choice is cheap now and expensive later.

### 2.3 Edges (v1 set, each scored separately)

| Edge | Meaning | Checkable against |
|---|---|---|
| `CONTAINS` | module or item ownership | rustdoc / syn |
| `EXPOSES_AS` | `pub use` re-export, giving the public path to the canonical definition | rustdoc JSON |
| `SIGNATURE_USES{pos}` | a type appears in a parameter, return type, field, bound, or associated type; `pos` records where | rustdoc JSON |
| `IMPLEMENTS` | impl to trait, and impl to self type | rustdoc JSON |
| `DERIVES` / `MACRO_GENERATES` | an implementation produced by an attribute or macro | rustdoc (it sees expanded impls) |
| `CALLS` | a statically resolved call | rust-analyzer / SCIP |
| `MAY_CALL` | trait-method or `dyn` dispatch; points at the **trait method**, with possible implementors derivable from `IMPLEMENTS` | RA |
| `DEPENDS_ON_CRATE` | Cargo.toml dependency, feature-annotated | cargo metadata |

### 2.4 Attributes on every edge and node
- `visibility`: one of `pub`, `pub(crate)`, `pub(in path)`, or private, plus **effective reachability** from the crate root
- `cfg`: the predicate, for example `feature="tokio" && unix`
- `evidence_span`: file, byte range, and content hash
- `provenance`: `parser`, `compiler`, `model@rev`, or `llm-adjudicated`
- `status`: `resolved`, `ambiguous`, `external`, or `unresolved`; the model must be allowed to abstain
- Fn flags: `unsafe`, `async`, `const`, `extern "ABI"`, generics and lifetimes

### 2.5 Projections
1. **Machine projection (JSONL, ideally SCIP-compatible).** Agents and tools query this, and the benchmark scores against it.
2. **LLM projection: a "Rust skeleton".** This is valid-looking Rust with bodies elided (`{ … }`), stable IDs in comments, and a short `// calls:` line per function.

   LLMs already read Rust fluently, so this is likely more token-efficient and more accurate for agents than JSON triples. It is the Rust equivalent of Python's `.pyi` stubs. Recommended default for agents.
   ```rust
   // #storage::Store  (exposed as: mycrate::Repository)
   pub struct Store { pub path: PathBuf, /* 2 private */ }
   impl Store {
       pub fn open(p: &Path) -> Result<Self, StoreError> { … }   // calls: fs::create_dir_all, Self::load
   }
   impl Repository for Store { … }                               // via #[derive]? no; manual
   ```

---

## 3. Decisions you need to make
Ordered by impact. Each entry gives the question, the options, and my recommendation.

| # | Decision | Options | Recommendation |
|---|---|---|---|
| D1 | **Ground truth / teacher** | Codex labels · compiler (rustdoc JSON + RA/SCIP) · hybrid | **Compiler first.** Use Codex only for disagreements and for the non-building slice. This changes your dataset plan more than any other decision. |
| D2 | **Model's job** | whole-repo graph generation · per-file structured extraction with host linking · edge resolver over parser candidates · iterative constructor (doc 02, Option 2) | **Per-file extraction plus host linker** for v1. It has bounded output, fits a small context, and supports incremental updates. Resolver and iterative modes come later. |
| D3 | **Meaning of "public"** | syntactic `pub` · effective crate-external reachability · all items | **Store all items. Make public reachability the default view.** Private callables stay in the store as support for call paths. |
| D4 | **v1 edge set** | contracts only · contracts + calls | **Contracts first** (`EXPOSES_AS`, `SIGNATURE_USES`, `IMPLEMENTS`, `DERIVES`), because they are 100% checkable via rustdoc. Report `CALLS` as a separate, later slice. |
| D5 | **Build configuration (cfg/features/target)** | one graph per configuration · union graph with cfg predicates | **Union graph with predicate annotations.** Score against the default features plus one all-features configuration. |
| D6 | **Macros** | ignore · flag regions · model derive/attr semantics · require expansion | **Learn derives and common attribute macros** (this is where the model's edge is). Flag `macro_rules!`/proc-macro-generated items as `MACRO_GENERATES` with lower confidence. |
| D7 | **Trait / generic dispatch** | point at one guessed impl · fan out to all implementors · point at trait method | **Point at the trait method (`MAY_CALL`).** Implementors are derivable from `IMPLEMENTS`. Never invent a single concrete target. |
| D8 | **Node identity** | current line-range keys · semantic path keys | **Semantic path keys**, with spans as attributes. Migrate early. |
| D9 | **External crates** | stop at boundary · stubs · include dependency public APIs | **Stubs with crate@version.** Optionally import dependency rustdoc JSON as a cached layer. |
| D10 | **Output format** | JSON triples · typed JSONL / SCIP · Rust skeleton text | **Both projections:** JSONL/SCIP for machines and skeleton text for LLMs. Use the model's output grammar with constrained decoding, for example GBNF in llama.cpp. |
| D11 | **RAM budget definition** | model weights · inference process · whole Parseltongue process | Declare it explicitly. **Prove quality under 4 GB first**, then distill toward 100 MB (as doc 02 says). |
| D12 | **Benchmark and success metric** | "better than Tree-sitter" · per-relation P/R vs compiler · agent task success | **Per-relation precision/recall vs the compiler on repo-held-out crates,** plus a non-building slice, plus one agent navigation task (tokens and correctness). |
| D13 | **Dataset split and contamination** | random file split · repo-held-out · time-held-out | **Repo-held-out plus recent crates published after the base model's cutoff.** |
| D14 | **Incremental semantics** | full reindex · per-file re-extract plus relink | **Per-file re-extract plus relink**, keyed by content hash. Identity decision D8 makes this work. |
| D15 | **Licensing of dataset/weights** | — | Record the license of every crate. Most top crates are MIT/Apache-2.0, but **verify**. Base-model license must allow redistribution of fine-tuned weights. |

### The three decisions that gate everything else
1. **D1 (teacher).** It determines cost, label quality, and whether results can be reproduced.
2. **D2 (model's job).** It determines architecture, context size, and whether the 100 MB target is realistic.
3. **D12 (success metric).** It determines whether anyone believes the result.

---

## 4. Minimal first experiment (about 1–2 weeks)
1. Pick about 30 training crates and 10 held-out crates, all of which build.
2. Generate labels with rustdoc JSON and `rust-analyzer scip`. Convert them to the canonical schema above. This step is deterministic and needs no LLM.
3. Baselines:
   - (a) the current Parseltongue Tree-sitter extractor
   - (b) Tree-sitter plus a `use`-aware resolver
4. Fine-tune Qwen2.5-Coder-0.5B (or similar) to emit per-file skeleton/JSONL under a grammar. The host links results across files.
5. Score per relation (`EXPOSES_AS`, `SIGNATURE_USES`, `IMPLEMENTS`, `DERIVES`) on the held-out crates. Then break the build by deleting `Cargo.lock`, a dependency, or a file, and re-score. That second score is your headline result.
6. Only then quantize and distill toward 135M parameters and below 100 MB.

**Kill criterion:** if the model does not beat baseline (b) on either the building or the non-building slice at acceptable cost, ship the improved deterministic pipeline (b) plus compiler import. In that case the graph contract is the product, and the model is not.

---

## 5. Prior art, and why this works for Rust specifically

All of this is from general knowledge, written without lookups. Verify names, dates and feature claims before citing them.

### Has anyone built this for Rust?
Pieces, yes. The whole thing, no.

| Prior work | What it is | Gap relative to this project |
|---|---|---|
| **rustdoc JSON** (RFC 2963, nightly since ~2020) | The compiler emits the public API as structured data: items, visibility, re-exports, impls, generics. No other mainstream language has a compiler-emitted public-interface dataset like this. | Per crate, requires a build, no call edges. |
| **cargo-semver-checks** (Predrag Gruevski), **cargo-public-api** | Query rustdoc JSON as a graph (via Trustfall) to detect breaking API changes. Closest existing "public interface dependency graph for Rust"; proves the graph is machine-checkable. | Per crate, no call edges, build required. |
| **Präzi** (Hejderup et al., ~2018–2022) | Call graphs for the whole crates.io ecosystem from MIR/LLVM, replacing package-level dependency networks with call-level ones. Closest academic precedent for cross-crate call edges. | Build required, offline batch, not agent-facing. |
| **cargo-modules**, **cargo-depgraph** | Module trees and crate graphs. | Structure only. |
| **cargo-call-stack** | Call graph from LLVM IR for embedded stack-usage analysis. | Build required, no public-contract notion. |
| **cargo-geiger** | Unsafe surface counting. | Not a graph. |
| **rust-analyzer SCIP/LSIF**, earlier RLS **save-analysis** (removed 2023), a Kythe Rust indexer on top of it | Compiler-grade cross-references. | References, not contracts; needs a working project; heavy at scale. |
| **Charon/Aeneas, Rudra, MirChecker, Kani, Prusti** | Analyses and verification on MIR. | All require a successful build. |

Nothing known to me trains a model to produce this graph for Rust. The model bet is genuinely open.

### Why it works for Rust and not for other languages
The graph needs three properties: node identity is decidable, edges are decidable, and the public boundary is real. Rust is the only widely used language with all three.

1. **Visibility is lexical and decidable.** `pub`, `pub(crate)`, `pub(in path)`, private by default. Python has no privacy. JS exports are static but values are mutable. C has headers plus `static`, nothing in between. Java has modifiers but reflection bypasses them.
2. **Name resolution is static and closed.** No `getattr`, `eval`, monkeypatching, reflection or classloaders. `use` is static, paths are explicit. In Python/JS/Ruby/Java-with-Spring the call graph is undecidable in practice; the 2024 call-graph-vs-LLM study cited in docs-lossfunk fails largely for this reason. For Rust the answer exists; the only question is who computes it.
3. **Trait coherence (the orphan rule).** For any type `T` and trait `Tr` there is at most one impl, and it lives in `T`'s crate or `Tr`'s crate. `IMPLEMENTS` edges are therefore enumerable and local. Go's structural interfaces need a global search to know what implements what. Python's duck typing has no answer. Haskell is close but tolerates orphan instances.
4. **No inheritance, explicit dispatch.** Dynamic dispatch is spelled `dyn`; generic calls are monomorphised and resolvable per instantiation. No virtual-override hierarchies that make C++/Java call graphs over-approximate.
5. **Types are fully explicit at every boundary.** Every type in a signature is a real dependency. TypeScript erases types and leaks `any`; C uses `void*`.
6. **The crate is a real unit.** Cargo gives a versioned DAG with features and semver discipline, and the ecosystem favours many small crates, so the cross-crate public boundary is where the information lives. C/C++ textual `#include` has no boundary.

### Why this matters for the small-model bet
Because Rust's rules are local (a file, its `use` lines and the crate's module tree determine most resolutions), a per-file model can learn to resolve names. That is what makes distillation feasible for Rust and hopeless for Python.

### Where Rust is hard
`macro_rules!` and proc macros (the one hole in static resolution, though expansion is deterministic and rustdoc sees the expanded result), `cfg`/feature matrices (several valid graphs), and trait-object fan-out. These are the three research problems; everything else is engineering.

### Second-best language
Go: static, capitalisation visibility, real packages. It falls short on structural interfaces. Useful sanity check: if the design only works for Rust, it should be because of coherence and lexical visibility, not for incidental reasons.
