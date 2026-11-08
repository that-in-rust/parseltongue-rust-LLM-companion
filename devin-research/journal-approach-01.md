# Journal — Approach 01

*A chronological record of one research thread (roughly 135 exchanges, 2026-11-05 → 2026-11-08) on the question: should a Rust public-interface dependency graph exist, what shape should it take, and can a tiny model plus a CLI build and serve it? Written from the transcript. Each entry gives the question as asked, the answer given, and what it changed. Section 13 traces how the thinking evolved; Section 14 lists what stands and what is still open.*

---

## 1. The opening question (exchanges 1–8)

**Asked.** Whether a public-interface-dependency-graph representation of a large codebase would let an LLM reason about control flow and data flow better; then, sharply: "the point is not the name ideas — can you think of how useful this is, what should be the form of this graph for Rust, and which areas need decisions"; then "how can this help people write 10 million lines of code systems programming"; then the hard one — "do you really think a universal public interface dependency graph is a great idea? what should the level of aggregation be?"

**Answered.** Useful under conditions: Rust is the one language where "public interface" has an exact meaning (`pub`, modules, crates), so the public face of a file is a complete contract for every other file. Two views were proposed: a structural graph and a "Rust skeleton" — real signatures with bodies removed and a `calls:` line per function, like a header file. Open decisions were listed: aggregation level, inclusion of private items, handling of generics and macros, storage, freshness.

**Scope correction.** "I meant only for Rust — build 1 for Rust." One graph, one language, from here on.

**Housekeeping.** A `devin-research/` folder was created on branch `research/tiny-rust-code-mapper`; files prefixed with a unix timestamp for searchability.

## 2. Prior art, Strata and Clarity (exchanges 9–14)

**Asked.** "Imagine you are a language researcher — has someone tried building such a graph before? For Rust specifically? Why will it work for Rust and not others?" Then: decompile LegacyCode's Strata and read clarity-cli; then "do deep exploration of the Strata/Clarity ideas on the web and arXiv and create a long-form thesis with references."

**Answered.** Pieces exist (rustdoc JSON, SCIP, cargo-modules, Präzi, stack graphs, Aider's repo map, Kythe/Glean); the whole does not. Rust works because names are static, visibility is explicit, and the crate graph is declared. Strata and Clarity were read as two halves of the 1995 reflexion-model idea: what you meant to build versus what you actually built, compared by a checker. A long-form thesis with verified references was written, then compressed on request into a five-line ELI5 ASCII.

**What it set up.** The idea of a map that carries intent as well as fact — which later became the proposed/future records.

## 3. The first disagreement: compilers everywhere (exchanges 15–22)

**Asked.** "I do not agree with some of the approach here. We cannot be running a compiler for everything. How many control-flow and data-flow tools does Rust have, are they complicated to run, why can't an LLM do that job better — clone at least five of them." Then "can rust-analyzer draw everything accurately?" Then, after reading a weight-space-programming post: "which 100 MB model can we edit to work like rust-analyzer?"

**Done and answered.** Eight tools were cloned and inspected (Flowistry, rust-analyzer, cargo-modules, cargo-semver-checks, RustPräzi, cargo-call-stack, cargo-geiger, rust-code-analysis). The finding: "compiler versus LLM is the wrong fight — there is a ladder": text/regex at the bottom, tree-sitter, rust-analyzer's HIR, MIR, LLVM IR at the top, each rung costing more and seeing more. rust-analyzer was measured on real crates: it draws one build configuration almost exactly and cannot draw the others (`cfg` variants, proc-macro output without a check, `dyn`/generic fan-out). The weight-editing post's own results showed you cannot "edit a model into rust-analyzer"; candidate ≤100 MB bases were listed anyway.

**What it changed.** The compiler moved from "the tool" to "one rung of a ladder, with blind spots". The question became which rung the product should stand on.

## 4. Shape of the map, and Parseltongue as grep replacement (exchanges 23–29)

**Asked.** "What should be the shape of the code graph — how will it help LLMs navigate 10 million lines?" "I am thinking 2 levels — file level and public interface level." Then a set of screenshots: Parseltongue specialising as a pure-CPU grep replacement (LLM asks in seven words; Parseltongue finds the four nearest non-keywords; returns headlines then expands). Then: "if my job is to create a dependency graph at multiple levels, am I not seeking an LLM which only takes decisions, bidirectional with a search tool, and the net result beats the parser?"

**Answered.** Two levels is the right default view; it matches the three ways an LLM fails at 10 M lines (cannot find, cannot see the contract, cannot see blast radius). The seven-event grep-replacement flow was mapped onto CPU algorithms (symbol trie, trigram index, identifier-split BM25 with rank fusion). To the bidirectional question: "yes for using the graph, no for building it — the distinction is the whole design." Resolver builds, LLM steers. A hypotheses ledger (H-numbers) was started to collect ideas as they arrived.

**Side thread.** The Sikos 2023 survey of cybersecurity knowledge graphs was read for transferable ideas (typed entities, provenance, sharing, correlation).

## 5. Training on the compiler's output (exchanges 30–40)

**Asked.** "Estimate post-training a 100 MB model on a 24 GB Mac mini with 100 M input tokens." Then the reframing: "we first want the model to understand the syntax — what if we took a 500 MB model and trained it not on LLM labels but on rust-analyzer's output? This would save a lot of high-quality LLM labelling time." Then: "even if I completed this training, will it run faster than rust-analyzer or use less RAM?" — "assume GPU" — "say Mac mini."

**Answered.** The compiler-as-teacher idea was adopted as H-46 and judged better than the earlier plan for a reason not stated: rust-analyzer labels broken, mid-edit snapshots too, so every commit and every synthetic half-edit becomes training data. On the headline: on a workspace rust-analyzer can load, a 500 MB student will not be faster or smaller; on a Mac mini GPU the small model wins only in specific regimes (per-edit, snippets, broken code, no Cargo). The comparison tables became Deep-Insight-01 — "this is the key differentiation."

**What it changed.** The product's claim stopped being "faster than rust-analyzer" and became "works where rust-analyzer cannot run, and answers in an agent-shaped way."

## 6. Tooling detour: Z.ai, SOPs, a second opinion (exchanges 41–45, 54–64)

**Asked.** Set up a Z.ai GLM Coding Plan agent in the VM; have "zAI's best model" read the whole research folder and give an opinion; create an SOP; check live quotas; decide whether the plan suits label generation.

**Done.** Claude Code was pointed at GLM-5.3 via a `zcode` launcher; the first key failed with "insufficient balance", a Coding Plan key worked. GLM-5.3 read 18 research files and wrote a 2,262-word critique. Two SOPs were written into `devin-research/sop/`. Verdict on labelling: good for labels that need judgement, wrong for labels that must be exact — the compiler does those.

## 7. Training shortcuts and the three-part differentiation (exchanges 46–53, 65–69)

**Asked.** "How can we shortcut the whole training — model size or dataset size?" "So is our differentiation giving you a dependency map when LSPs fail?" "What would be an exceptional achievement — a 100 MB model doing this?" "What if I repeatedly train it on the same data?" Also: Jarvislabs GPU prices versus a Mac mini, and an ELI5 of post-training "assuming I know nothing."

**Answered.** Time = 6 × params × tokens ÷ chip speed; the dataset lever is larger than the model lever, and the biggest shortcut is procedural (do not teach the alphabet twice — start from a code encoder). "When LSPs fail" is the differentiation of the model, not of Parseltongue; leading with it positions the product by its fallback. A three-part differentiation was written in a Shreyas Doshi register into Deep-Insight-01. Repeated training on the same data memorises; a 100 MB model doing extraction at compiler-grade precision would be exceptional precisely because nobody has shown a model that small resolving Rust.

## 8. Append-only, from two devices (exchanges 70–74)

**Asked.** "Do git pull and check new ideas being merged — if there is a merge conflict just make files append-only, do not replace stuff." Then "just append all of it in a single file — create multiple sections." Then "all the changes you made across the different devices — just append and append new sections."

**Done.** An upstream commit from another device had rewritten Deep-Insight-01 wholesale; the original was restored and the file became one collected, append-only document with dated Parts (original, CODEMAP rewrite, four futures, training shortcuts…). Every later Deep-Insight document is mirrored into it as a new Part.

**Rule established.** Research is append-only. Nothing is replaced; new thinking is added as new sections.

## 9. The one big thing; Proposal 01; the verification cut (exchanges 75–90)

**Asked.** "What are we really trying to do based on the Deep-Insight documents? What is the big one thing? A Rust LLM companion that is a CLI tool as well as an LLM, bidirectionally talking, helping on broken code, with current and future maps." Then: restructure with the Minto pyramid as Deep-Insight-05; "remove all references of where stuff came from — focus hard on what actually is your thesis." Then the disagreement: "I do not agree — tool diff or something — why is it needed?" → "yes remove it from scope — unnecessary complication." Then: "the future edges thing should be just a framework that can be populated by the orchestrator LLM or the agent LLM — our tool should not be filling that." "Be a little rigorous in defining the entities." "What will Parseltongue be — a CLI interface to a bunch of CLI tools × tiny-LLMs who talk to each other to answer the agent-LLM." "No, I do not want tree-sitter and rust-analyzer in the deterministic CLI tools — why would I want that?" "You tell me what utility they have if the tinyLLM works." "And the tiny LLM can run on GPU also."

**Answered and changed.**
- Deep-Insight-04 became "Proposal 01": blueprint / as-built / inspector, mapped to Sikos and MISP. Then re-derived under full openness: open tool, open weights, open training data.
- Deep-Insight-05 restructured Minto-style, then stripped of provenance, then stripped of the verification product: no diff command, no as-built map, no conformance report, no CI gate. One map, with extracted records and proposed records; a proposed record becomes realized when later extraction matches it.
- The tool owns the slot for future structure (schema, status `proposed`, creator, reason) and never writes into it.
- Entities defined rigorously: identity, status (`extracted · abstained · inferred · proposed · realized · stale`), provenance, authority — who may write which status.
- Parseltongue defined as one CLI over Parseltongue-owned deterministic tools and tiny task models.
- rust-analyzer and tree-sitter removed from the runtime: if the tiny model works, they have no runtime utility; their jobs are off-line — teacher for labels, judge for evaluation.
- The tiny model runs on CPU or GPU; CPU is the floor, not a constraint.

## 10. Standalone thesis and the Broad Ideas (exchanges 91–104)

**Asked.** "Let Deep-Insight-05 be an independent document, Minto, write like Shreyas Doshi." Then a seven-point "broad ideas" list (graph storage; nerf a big model down; quantisation; …). "Add SurrealDB." "Why not add TursoDB itself." A prediction: 500 MB model versus rust-analyzer on 10 M lines — "tiny LLM on GPU" — "think of Mac mini." "Why is inference so slow — don't we read 10 M lines and output only 5 %?" "What size of tiny LLM gives meaningful classification faster than rust-analyzer at less peak RAM?" Include the Sikos PDF in the repo and research it. A Shreyas-Doshi product-structure section with timeline-traverser simulations. "What should be the model size to run really fast on a 2 GB GPU?"

**Answered.** DI-05 rewritten standalone. B1–B7 broad ideas, with embedded storage candidates (SQLite + recursive CTEs, Kùzu/LadybugDB, CozoDB, LanceDB, SurrealDB, Turso). Turso checked from its repository: Rust SQLite rewrite, MIT, recursive CTEs, FTS via Tantivy, vector functions, MVCC/`BEGIN CONCURRENT`, pre-1.0. B8/B8.1 prediction: a 0.5B model on a Mac mini is not faster than rust-analyzer on a loadable workspace; wins in the per-edit and broken-code regimes. B8.2: reading is not free — every token costs 2 × params operations whether it is easy or hard; four ways to make it cheaper (rules first, skip bodies, cache, cascade). B9: ~125M-parameter int8 encoder as the 2 GB sweet spot. B10: Sikos research. B11: five product-shape timelines.

## 11. The Linux yardstick and the time-to-map equation (exchanges 105–119)

**Asked.** "Think of the Linux codebase — pull that in and count the LOC." "Explain B9.1 slowly, step by step." "Apple 8 GB Mac Air — time to create the first map for a Linux-sized codebase, step by step." "What is L1 and L2 — are you doing it via CPU?" "I am asking you to simulate when EVERYTHING is done by model." "How can reading 400 M tokens be so difficult?" "If the model was 2 MB?" "Three methods using a tiny LLM to do it in 1 minute on an 8 GB Air." "If the base map takes 30 minutes, what opens up?" "Five variants — sizes and types of models post-trained to process the whole Linux codebase in 30 minutes on an 8 GB Air." "Time-to-map as y, a function of ten or more variables — int8 vs int4 vs int2, nerfed code model vs small model, parameters, model type, passes — research all the models on Hugging Face including Cassini."

**Measured.** Linux at `d48723711`, counted with tokei: 33.4 M code lines (20.1 M C, 8.7 M headers, 132 k Rust), ~1.49 GB of C/H/Rust source, ~370–430 M tokens. Used as the "very large codebase" yardstick, pretending it were Rust.

**Answered.** B9.1–B9.9. Everything-by-model on an 8 GB Air for 400 M tokens with a 125M model: ~30 hours; the scanner-first cascade brings L1/L2 to ~10 minutes; a 2 MB model reads everything in ~20 minutes. The equation: y = T_io + Σ passes [READ_p + WRITE_p] + T_store + T_index + T_feed, with READ driven by active parameters × routed fraction × (1 − cache hit) over compute, and WRITE by output ratio × bytes per weight over bandwidth. Conclusion: routing fraction and caching dominate; quantisation matters for decode, barely for prefill; encoder cascades avoid decode altogether. Cassini-1.0 checked from its model card: Qwen3-4B base, LoRA rank 1, Python/JS/TS, 77.6 % F1, Apache-2.0 — evidence that structured extraction by a model works, not evidence about Rust or about 100 MB.

## 12. Replacing LSP; the multi-level map; span-first (exchanges 120–135)

**Asked.** "Do we need a model which asks FFF a lot of questions, or is Cargo enough? The problem with rust-analyzer is it has no agentic interface." "What deterministic tool that is NOT LSP can search code?" "Point is to replace LSP with tinyLLM + CLI." "Expand: tinyLLM + CLI build a multi-level code map — folder-level dependency map, file-level, public-interface-level, dependencies as edges, metadata on node types." "Make B13 an independent Deep-Insight-06." "100 MB model trained on the top 100 Rust repos 40 times — 98 % accuracy versus the deterministic LSP tools? Timeline-traverser simulations." "Output is what we describe in Deep-Insight-06." "What if we took Cassini or Apache models which are bigger and trained the smaller nerfed model from them?" "In what context can a 100 MB model index Linux to create D06 — can it do it in 30 minutes?" "Why are L2 and L3 different?" "Going further do vertical sections and not side-by-side tables — unable to see them." "What if we used a tiny LLM to break code into spans of public stuff only, put them in a Turso table — does that change the game?" "Create this as D07 and B16." "Can we build a 100 MB LLM that converts code into spans in a Turso DB quickly?"

**Answered and created.**
- B12: FFF is the wrong oracle for dependencies (it answers "which file", not "which item"); Cargo is level 0; the LSP replacement is the store the tiny model fills plus the CLI verbs that read it — definition → `find`, references → `impact`, outline → the file's public face, hover → the span record; rename and diagnostics not claimed.
- B13 → Deep-Insight-06: one append-only set of leaf records (items and typed edges) from which file and folder views are computed by a fixed roll-up rule (weight = count, kinds = union, cfg = union, confidence = [min, max], status by contributors). Typed nodes, typed edges, producer matrix, worked `retry` example, identity-under-motion as the one risk.
- B14: 98 % is reachable as precision on extracted records with coverage printed beside it; L0–L2 ~99 %; L3 coverage ~85–90 % rising. 100 repos × 40 epochs memorises; candidates-with-signatures in the input and breadth-first distilled data are what generalise. Measured against the DI-06 output, where `abstained` is a correct record, the claim becomes well-defined.
- Distillation chain (offered as B15, not yet appended): compiler → Rust-adapted 4B teacher → 100 MB student; the big model contributes priors, calibration and pseudo-labels on broken code, never information absent from the input.
- L2 versus L3: the doorway (signature) versus the room (body); L2 eager and cheap, L3 lazy and the only place real resolution happens.
- B16 → Deep-Insight-07: span-first. Public items are the primitive; one table of spans (digest identity, kind, name, path, visibility, range, text, doc, cfg, creator, confidence, snapshot); edges derived later by SQL joins, by a model on ambiguous rows, and by the agent reading spans directly.
- On the 100 MB span model: feasible, larger than needed for well-formed code (a 2–5 MB tagger suffices), valuable for broken, half-typed and macro-heavy code.

## 13. The last four exchanges (this session)

- **Staged pipeline, three approaches.** Asked for the 15-minute Linux estimate and three ways to chain tiny models (span → clarify grey spans → node details → edge details). Answered with the arithmetic (100 MB reading everything: Air 30 h, Mac mini 7 h, desktop GPU 30 min; under 15 minutes only by having the heavy model read the grey few percent), six stages, and three builds: relay of specialists, one model with many heads, table-first with models as repair workers.
- **The stage-4 correction.** "Disagree — what scanner can run a Linux-level codebase into Turso?" Conceded: a regex-grade scanner gets generics, bounds and attributes subtly wrong, and once spans exist the later stages read only span text (10–20 % of the corpus), so a 30–100 MB model can own node fields outright. Principle: the tiny model makes every decision; the tool only moves bytes; the table is the contract. Written as DI-07 §9.
- **Tokenizing, and why 100 MB is slow.** Tokenizing is cutting bytes into the model's vocabulary with offsets — plumbing. The 100 MB model is slow at spanning only because each of 400 M tokens costs 200 M operations; the fix is reading fewer tokens, not a smarter model. Non-LLM spanning options (brace scanner, `syn`, tree-sitter, rust-analyzer's parser crates, git-diff forwarding) were laid out; the user restated the preference for a model because "LLMs can figure out more random spans" — agreed, with the size-versus-cost caveat and a training recipe built on synthetic breakage.
- **Parallel spanners and Turso.** One 100 MB instance ≈ 0.5–1 GB GPU; ten fit on an Air but one saturates a GPU — parallelism is batch size on one machine and more machines across crates; Turso's MVCC lets several writers share one file.
- **AgentFS.** The user recalled Turso's founder had written something close. Found: AgentFS by Pekka Enberg — files, tool calls and state in one SQLite/Turso file, FUSE mount, just-bash. It is a file layer, not a code index. DI-07 §10 compared files-in-DB with index-over-tree across the codebase-MCP tools (all keep files on disk) and set the middle path: span text in the DB, tree as byte source, AgentFS schema only when sandboxing an agent.
- **Has anyone built this?** Three tools run hands-on on `cargo-modules`: mcp-codebase-index (regex, 9 file edges of 146 `use` lines), CodeGraphContext (tree-sitter → Kuzu; confidence labels on calls, but 0 of 19 trait impls, visibility null, no file-level edges), code-graph-mcp (Rust, tree-sitter → SQLite; typed nodes and edges, `extracted | inferred | ambiguous` on every edge, blake3 file hashes, FTS5, file and folder views, 0.14 s). Verdict: the storage shape is proven by others; nobody stores visibility, defines a roll-up, keeps abstention as a record, holds proposed structure, works on broken code, or puts a model in the resolver. DI-07 §11.

## 14. How the thinking evolved

```text
 start          "a public-interface dependency graph helps LLMs reason about flow"
 narrow         Rust only — the one language where public interface is exact
 reject         compiler-for-everything; the compiler is one rung of a ladder with blind spots
 split          resolver builds, LLM steers — using the graph and building it are different jobs
 reframe        train the model on the compiler's output, not on LLM labels; broken snapshots become data
 re-position    not "faster than rust-analyzer" — works where it cannot run, answers agent-shaped
 define         one big thing: a CLI and a tiny model talking to each other; current and future in one map
 cut            verification / diff / as-built removed; the tool never authors the future
 rigour         entities, statuses, provenance, authority; Parseltongue = one CLI over tools × tiny models
 remove         tree-sitter and rust-analyzer leave the runtime; teacher and judge off-line only
 measure        Linux as the yardstick; reading is compute, not skimming; the time-to-map equation
 replace        LSP's answers become CLI verbs over a persisted store
 structure      folder / file / public-interface levels as roll-ups of one leaf record set   (DI-06)
 invert         spans first, graph later; the table is the floor product                      (DI-07)
 correct        the model owns every judgment, the tool only moves bytes                      (DI-07 §9)
 check          AgentFS and three code-graph MCP tools run hands-on; the gap is confirmed       (DI-07 §10–11)
```

Three turns mattered more than the rest. The first was the refusal to run a compiler everywhere, which forced the ladder and the teacher/judge split. The second was cutting verification from scope and making the tool a custodian of proposed structure rather than its author, which made the one-map model possible. The third was span-first, which moved the product's first artefact from edges to a table of public spans and made every later stage cheap because it reads span text, not the corpus.

## 15. What stands, and what is open

**Stands.**
- Parseltongue: one CLI interface; Parseltongue-owned deterministic tools (store, index, search, find, impact, propose, feed) that move bytes and decide nothing; tiny task models (span tagger, clarifier, field-tagger, binder, abstainer, labeller) that make every judgment.
- One map with statuses `extracted · abstained · inferred · proposed · realized · stale`; the tool never writes `proposed`; LLMs and humans never write `extracted`.
- Levels L0 crate, L1 folder and file, L2 public-interface item, L3 body; upper levels defined by the roll-up rule; L3 lazy.
- Span-first storage in Turso/SQLite-compatible embedded files; edges derived; private items as light rows for anchor traversal.
- rust-analyzer and rustc off-line as teacher and judge; no runtime compiler, tree-sitter or Cargo loop.
- Research is append-only; new documents are standalone and mirrored into the collected DI-01; vertical sections, no side-by-side tables.

**Open.**
- Whether a 2–5 MB tagger and a 100 MB encoder reach the boundary accuracy claimed on broken Rust — the one-afternoon benchmark on deliberately broken compiling commits has not been run.
- Whether a 100 MB binder over retrieved candidates reaches 98 % precision at useful coverage on held-out repositories.
- Whether distillation from a Rust-adapted 4B teacher adds the predicted 3–8 points (the B15 section is drafted in conversation, not appended).
- Whether Turso is the right backend after benchmarking against SQLite and the graph-native options.
- Whether span digests survive real rename/move histories.
- Whether the timing envelopes hold on actual Apple hardware; every number in this journal is arithmetic from published throughput figures, not a measurement, except the Linux line counts, the rust-analyzer measurement on a 42 k-line workspace, and the three code-graph tools run on `cargo-modules`.
