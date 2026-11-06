# ELI5 in ASCII: train a 500 MB model on rust-analyzer output only (H-46) — what we'd do, what it would prove, why it would be useful

*Written 2026-11-05. Branch `research/tiny-rust-code-mapper`. Picture version of ledger entry H-46 in `1793881000-hypotheses-ledger-…md`; background in `1793877276-thesis-weight-space-programming-…md` and `1793882149-thesis-resolver-builds-llm-steers-…md`.*

## What we'd do

```text
   top-100 Rust repos            rust-analyzer (CPU, exact)          compact records
   ┌──────────────┐   read   ┌────────────────────────────┐  convert  ┌──────────────┐
   │ 15M lines    │ ───────► │ syntax tree                │ ───────►  │ item: Fn …   │
   │ incl. half-  │          │ item tree / pub skeleton   │           │ edge: CALLS  │
   │ edited,      │          │ every token → definition   │           │ type: &Store │
   │ broken files │          │ every expr  → type         │           │ unresolved!  │
   └──────────────┘          │ HIR/MIR control flow       │           └──────┬───────┘
                             │ diagnostics ("no such x")  │                  │
                             └────────────────────────────┘                  │  ~1B tokens, 0 LLM labels
                                                                             ▼
                                                                   ┌──────────────────┐
                                                                   │  500 MB model    │  phase 1: learn what RA "sees"
                                                                   │  (0.5B params)   │  phase 2: emit records per file
                                                                   └──────────────────┘  phase 3: cut its failure modes

   BEFORE (old plan)                       NOW
   big LLM labels files  ──► $$$, 78% F1   rust-analyzer labels files ──► free, 100% exact,
   only code that builds                   ALSO code that doesn't build
```

Notes: convert rust-analyzer's views (syntax tree, item tree, LSIF/SCIP definitions and hover types, HIR/MIR, diagnostics) into the compact record schema *before* training — raw dumps are ~10× source and version-unstable. Hold out whole repositories, not files. Compute: ~0.3–1 B tokens/epoch; ~2 h/epoch on one H100, ~2 weeks/epoch on an M4 Pro Mac mini (use the Mac for the 50–135 M model and for inference). Label generation ≈ 3 h CPU for 15 M LOC.

## What it would prove

```text
   1. "A compiler-grade teacher can be distilled into a pocket model"
      ┌───────────────────┐        ┌──────────┐
      │ rust-analyzer     │  ───►  │ 500 MB   │   same answers on held-out repos?
      │ 200k LOC, 2–3 GB  │ teach  │ one file │   if ≥95%: yes, for Rust, structure is learnable
      └───────────────────┘        └──────────┘   if ~78%:  the Cassini ceiling is real

   2. "You never need an LLM to make training data for code structure"
      labels come from the language's own tooling → reproducible, auditable, zero cost per label

   3. "Rust is the first language where this is even possible"
      static names + lexical visibility + coherence  ⇒  the teacher is exact  ⇒  the student can be scored exactly
```

## Why it would be useful

```text
   where rust-analyzer CAN run         where it CAN'T                     what the student does there
   ──────────────────────────          ─────────────────────────────      ────────────────────────────
   full workspace, cargo ok     ✓      a snippet in a chat               ✓ draws the local graph anyway
                                       a diff / a patch                  ✓ "this hunk calls X, breaks Y"
                                       no Cargo.toml / Bazel / Buck      ✓ graph with confidence tags
                                       10M LOC, 1 file changed, 50 ms    ✓ per-file update, no 2 GB index
                                       inside an editor keystroke loop   ✓ 100 MB, CPU, instant

   and underneath everything:  the converted top-100 output IS the shared background graph
                               of std + popular crates that every workspace links into (H-33)

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  resolver builds (exact)  →  student imitates (fast, anywhere)  →  LLM steers │
   └─────────────────────────────────────────────────────────────────────────────┘
```

## Either outcome is a result

- Pilot lands at **≥ 95 %** exact-match on held-out repos (item skeleton, definition paths, receiver types): "the structure of Rust code is learnable from the toolchain alone, with no human or LLM labels, in 500 MB" — publishable, and the student becomes the no-cargo / per-file / in-editor rung of the ladder.
- Pilot lands near **78 %** (Cassini's number, reached with a noisier teacher): the Cassini ceiling is measured for Rust with an exact teacher — also publishable, and it settles that the model's job is steering and meaning (L6), not construction (L1–L4).

## Pilot (from H-46)

10 repos, 1 epoch on a rented GPU; score on held-out repos: (a) item skeleton exact-match, (b) definition path for 1 000 sampled tokens, (c) receiver type for 1 000 method calls — against the same base model untrained and against rust-analyzer as truth. Evaluate at the shipped quantisation (int8/int4), not bf16.
