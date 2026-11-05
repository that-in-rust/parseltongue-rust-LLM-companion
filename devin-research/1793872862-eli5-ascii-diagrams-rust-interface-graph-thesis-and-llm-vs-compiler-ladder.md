# ELI5 in ASCII: the Rust interface-graph thesis, and "LLM vs compiler" after measuring it

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Picture versions of two longer notes in this folder:*
- *`1793858459-thesis-spec-first-evidence-bearing-rust-interface-graphs-for-coding-agents-strata-clarity-literature.md`*
- *`1793864191-can-an-llm-parse-rust-better-than-the-compiler-eight-tools-measured-and-a-five-level-universal-representation.md`*

---

## Part A — The thesis in five pictures

```text
1. STRATA + CLARITY = TWO HALVES OF ONE OLD IDEA (reflexion model, 1995)

   what you MEANT to build          what you ACTUALLY built
   ┌───────────────┐                ┌───────────────┐
   │  the drawing  │   ???  <-- the │  the code's   │
   │   (Strata)    │  ─ ─ ─ ─ ─ ─ ─ │  real wiring  │
   │               │   missing      │   (Clarity)   │
   └───────────────┘   check        └───────────────┘
   Nobody compares the two. The comparison IS the product.
```

```text
2. AGENTS NEED A MAP, NOT THE WHOLE CITY

   grep everything        tree-sitter graph        what we want
   ┌───────────┐          ┌───────────┐            ┌───────────┐
   │ ████████ │ 92% right │ ██       │ 83% right  │ ██       │ ~92% right
   │ ████████ │ $$$$$$$$  │          │ $         │          │ $
   └───────────┘          └───────────┘            └───────────┘
   (file reading)         (Codebase-Memory 2026)   (item-level, exact)
   Cheap map exists but it's blurry. Make the cheap map sharp.
```

```text
3. DON'T ASK THE SMALL MODEL TO DRAW THE MAP. LET THE ANALYZER DRAW IT.

      WRONG                               RIGHT
   ┌───────┐  "here's the graph"      ┌──────────┐ exact edges  ┌───────┐
   │ model │ ───────────────►  ✗      │ analyzer │ ───────────► │ graph │
   └───────┘  (static tools win)      └──────────┘              └───▲───┘
                                      ┌───────┐  "I think X->Y"    │
                                      │ model │ ──► [checker] ─ok─►┘
                                      └───────┘       │
                                                   not ok → "unresolved"
   Model = fills gaps + says "don't know". Checker has the last word.
```

```text
4. NOTHING LANDS UNTIL IT VALIDATES (and the error message is the lesson)

   agent writes ──► [ validate ] ──pass──► saved as version N+1
                         │
                       fail
                         │
                         ▼
           "line 12: Foo does not exist"  ──► back to agent (now)
                                           ──► training reward (later)
   Same gate guards production AND teaches the model. Strata does this for
   diagrams; the whole constrained-decoding/RL literature does it for code.
```

```text
5. WHY RUST FIRST: THE SPEC AND THE CODE ARE THE SAME LANGUAGE

   other languages                      Rust
   ┌────────────┐      ┌───────┐        ┌───────────────────────────┐
   │ diagram /  │ --?--│ code  │        │ pub fn open(..) -> Store  │ <- spec
   │ design.md  │      │       │        │   { todo!() }             │    (compiles!)
   └────────────┘      └───────┘        └─────────────┬─────────────┘
    human has to map                         rustdoc JSON gives the
    boxes to code                            SAME graph for spec and
                                             for finished code
                                             => comparing them is free
```

---

## Part B — "Why not just let the LLM parse it?" in six pictures

All numbers below were measured on this machine on 2026-11-05 (details and caveats in the `1793864191-…` note).

```text
1. "COMPILER vs LLM" IS THE WRONG FIGHT. THERE'S A LADDER.

   cost ▲
        │                                   ┌──────────┐
        │                                   │ LLVM IR  │ full build, hours  (Präzi, call-stack)
        │                            ┌──────┴──────────┤
        │                            │ MIR / rustc_priv│ nightly, needs build (Flowistry)
        │                     ┌──────┴─────────────────┤
        │                     │ rustdoc JSON           │ nightly, needs `cargo doc` (semver-checks)
        │              ┌──────┴────────────────────────┤
        │   ★ THIS ★   │ rust-analyzer                 │ NO COMPILER. parser + its own resolver.
        │              │                               │ 42k LOC: 29 s, 2 GB, 99.8% typed, works on broken code
        │       ┌──────┴───────────────────────────────┤
        │       │ tree-sitter / syn                     │ 0.07 s. names + shapes, no types (Clarity, Parseltongue today)
        └───────┴───────────────────────────────────────┴──────────────────────────► knowledge

   "Don't run the compiler for everything."  Agreed.  Rung ★ isn't the compiler.
```

```text
2. WHAT A PARSER (OR AN LLM READING ONE FILE) CAN SEE  — 138,000 real call sites

   all calls:   ████████████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
                └────────────── 65%  x.method(..)  ──────────────────────────────┘└─ 20% plain fn ─┘└─ 15% misc ─┘
                                                                                     (parser gets ✓)

   the 65%:     ██████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
                └─ 25% typeable ──┘└──────── 75%: type lives in ANOTHER file ────────────┘
                   from this fn        (chained a.b().c(), `let x = f()`, loop vars)

   file-local ceiling ≈ 40% of call edges.  To reach 98% you need the other files' signatures.
   That's not "understanding", it's a lookup table — and rust-analyzer already has it.
```

```text
3. WHERE THE ANSWERS LIVE (10,340 references in one crate)

   same file ████████████████████████████████████████████ 43%
   std       ██████████████████████████████ 30%            <- model must have memorised std
   ext crate ██████████████ 14%                             <- ...and every popular crate
   same crate█████████ 9%
   other ws  ████ 4%
```

```text
4. CASSINI-1.0 = THE "SMALL MODEL AS EXTRACTOR" IDEA, SHIPPED. READ THE NUMBER.

   ┌──────────┐   one file   ┌─────────────────┐  JSON  ┌────────────────────┐
   │ .py file │ ───────────► │ Qwen3-4B, LoRA  │ ─────► │ deterministic      │ ──► graph
   └──────────┘              │ rank 1          │        │ resolver (code)    │
                             └─────────────────┘        └────────────────────┘
                              77.6% F1  (Py/JS/TS)       <- still not the model's job
                              17–43× cheaper than frontier models
   ✓ small model as extractor works      ✗ 77.6 ≠ 98      ✗ no Rust (rust-analyzer gives ~100% free)
```

```text
5. SO: ONE GRAPH, SIX LEVELS, EACH RECORD SAYS WHO MADE IT

   L6  meaning   "this module guards the lock order"   ◄── LLM     (confidence 0.8)
   L5  flow      a influences b                        ◄── MIR     (only when built)
   L4  bodies    f CALLS g  via:[private h]            ◄── rust-analyzer types
   L3  resolved  real paths, re-exports, impls, cfg    ◄── rust-analyzer / rustdoc
   L2  items     fn/struct/trait/impl per file         ◄── tree-sitter
   L1  workspace crates, features, deps                ◄── cargo metadata / Cargo.toml

   no cargo?  → L1–L2 exact, L3–L4 from the LLM, tagged "guess".   full toolchain? → all exact.
```

```text
6. THE SHAPE OF "OUR LLM"

                 ┌─────────────┐
     agent ────► │ small model │ ── plans query, writes L6, fills gaps when tools can't run
                 └──┬─────┬────┘
          calls ────┘     └──── reads
          ▼                      ▼
   ┌──────────────┐      ┌──────────────┐
   │ CLIs (exact) │      │ markdown     │
   │ rsgraph      │      │ schema.md    │
   │ extract/ts/  │      │ cookbook.md  │
   │ query/diff   │      │ rust-rules.md│
   └──────────────┘      └──────────────┘

   Analyzer draws the skeleton.  Model names the bones.  Never the other way round.
```
