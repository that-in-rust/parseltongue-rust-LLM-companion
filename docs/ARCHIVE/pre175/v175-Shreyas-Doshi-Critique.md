# v1.7.5 — Shreyas Doshi Critique of v1.7.3 Research Docs

**Date**: 2026-02-13
**Framework**: LNO (Leverage / Neutral / Overhead) applied to every doc and decision.
**Core question for each doc**: "Did this move us closer to shipping, or did it feel like progress?"

---

## The One-Paragraph Version

You spent 5 docs and ~1,500 lines of research to arrive at a conclusion you could have reached in 30 minutes: "serialize to MessagePack, load into CozoDB mem, drop code bodies." The DECISION doc was made obsolete the same day by the slim model insight. The RESEARCH doc evaluated 14 formats when the answer was always "the one that works with serde." pt03 was designed as a separate crate for a feature nobody asked for. The only truly high-leverage artifact was the THESIS (slim graph model), and even that rated 24 endpoints individually when the decision rule was binary: "uses code bodies = drop, uses edges = build."

---

## Doc 1: `RESEARCH-v173-serialization-formats.md`

**LNO Rating: OVERHEAD**

### What It Does
260 lines evaluating 14 serialization formats across 8 dimensions with a weighted scoring table.

### The Uncomfortable Truth

The answer was MessagePack before the research started. Here's the actual decision tree:

```
Does it work with #[derive(Serialize, Deserialize)]?
├── No → reject (protobuf, flatbuffers, cap'n proto, rkyv, bitcode)
│        That's 5 formats eliminated in one question.
│
└── Yes → Is it actively maintained?
    ├── No → reject (bincode — dev doxxed)
    │
    └── Yes → Is it smaller than JSON?
        ├── No → reject (RON — also broken with tagged enums)
        │
        └── Yes → Pick the one with schema evolution.
                  = MessagePack. Done.
```

That's a 15-line decision tree. Not a 260-line research doc with CBOR, Parquet, and Postcard analysis.

### What Was Actually Leverage
- The bincode doxxing finding. Genuine risk avoidance.
- The memory crash projections at scale. Used in every subsequent doc.

### What Was Overhead
- Evaluating Protobuf, FlatBuffers, Cap'n Proto (none work with serde — instant reject)
- Evaluating Parquet (wrong paradigm for graph data — obvious in 10 seconds)
- Evaluating rkyv (explicitly labeled "future" — why evaluate it now?)
- The 10-row weighted comparison table (scored rkyv 17/35 — nobody will ever reference this number)
- SQLite analysis (already failed in v1.7.0 and v1.7.1)

### Shreyas Would Say
"You researched 14 options to pick the obvious one. The research felt productive but the decision was never in doubt. Next time, start with the decision tree. If it narrows to 1-2 candidates in under 5 minutes, stop researching."

---

## Doc 2: `DECISION-v173-pt02-pt03-endpoint-selection.md`

**LNO Rating: OVERHEAD (obsoleted same day)**

### What It Does
312 lines of RAM analysis at 1.6M edge scale, tier classification of 24 endpoints, 4 architectural options.

### The Uncomfortable Truth

This doc analyzed the FULL model (3,000 bytes/entity, 3.2 GB base RAM). Then the slim model idea happened (~151 bytes/entity, 504 MB base RAM), making the entire analysis irrelevant.

The doc's conclusion — "only 7 endpoints safe on 8GB" — was correct for the full model and completely wrong for the slim model (21 endpoints safe on 8GB).

```
Timeline:
  Hour 1: Write DECISION doc → "only 7 endpoints on 8GB, RocksDB wins"
  Hour 2: Slim model idea → "21 endpoints on 8GB, everything fits"
  Hour 3: DECISION doc is now wrong but still exists in the repo
```

### What Was Actually Leverage
- The per-handler RAM analysis (which handlers load all entities vs all edges vs key lookups). This pattern analysis directly informed which endpoints to drop in the slim model.
- The insight that "RocksDB's advantage is it only loads what each query needs" — this correctly identified the problem. The slim model is the real fix: make ALL data small enough to load.

### What Was Overhead
- The 4 architectural options (A/B/C/D) — the slim model made Options B/C/D unnecessary
- The tier classification (Tier 1/2/3) — with slim model, everything is effectively Tier 1
- The endpoint coverage matrix (8GB/16GB/32GB) — with slim model, 8GB handles 21/24
- The recommendation "start with Option A, consider Option C for v1.8.0" — stale advice

### Shreyas Would Say
"This doc was dead on arrival. You did the math for the wrong model, then invented a better model that invalidated all the math. The lesson: don't optimize the wrong thing. If the model is too big, shrink the model — don't build a tiered system to cope with the bigness."

---

## Doc 3: `THESIS-v173-slim-graph-address-model.md`

**LNO Rating: HIGH LEVERAGE (the breakthrough)**

### What It Does
~500 lines. Queried every endpoint on a live server. Rated each 1-10 for slim compatibility. Defined the slim entity schema. Showed 27x RAM reduction.

### This Is the One That Mattered

The core insight — "Parseltongue's value is the dependency GRAPH, not code storage" — is the entire v1.7.3 strategy in one sentence. Everything else is implementation detail.

The slim model transforms the problem:
```
Before: "How do we fit 3.2 GB into 8 GB RAM?"
After:  "504 MB fits everywhere. Problem gone."
```

### But Even This Over-Engineered

24 individual endpoint analyses with live query captures. The decision rule was binary:

```
Does the endpoint read Current_Code or diagnostic tables?
├── Yes → Drop (smart-context, diagnostics, coverage-folder)
└── No  → Build (everything else)
```

That's 3 lines, not 24 sections with captured JSON responses.

The root_subfolder_L1/L2 finding was useful — confirmed that scope filtering works with the slim model. But this could have been 1 paragraph, not a full section with live query captures showing `"scope": "crates||parseltongue-core"` responses.

### What Was Actually Leverage
- "Drop code bodies, keep addresses" — the insight
- root_subfolder_L1/L2 must be in the slim schema — would have been missed without checking
- The RAM math: 504 MB vs 3,200 MB — proves the thesis quantitatively

### What Was Overhead
- Rating each endpoint 1-10 (they're either 10 or <3, the gradient doesn't matter)
- Capturing exact JSON responses for all 24 endpoints (useful for debugging, not for decision-making)
- The detailed comparison table at the end (repeats what the individual sections already said)

### Shreyas Would Say
"This is the only doc that changed the architecture. Good. But you spent half the doc proving what was obvious after the first paragraph. When the insight is 'drop the ballast,' you don't need to weigh each piece of ballast individually."

---

## Doc 4: `THESIS-v173-storage-architecture.md`

**LNO Rating: NEUTRAL (interesting, not actionable)**

### What It Does
6 full architectural simulations with scoring matrices. Explores what if we dropped CozoDB entirely.

### The Uncomfortable Truth

The key finding — "All 7 graph algorithms run on AdjacencyListGraphRepresentation (pure Rust), not CozoDB" — is genuinely important. It means CozoDB is a query convenience layer, not load-bearing infrastructure.

But then the doc runs 6 simulations scored on 5 dimensions when the pragmatic answer is: "Keep CozoDB for now. It works. The slim model fixes the RAM problem. Revisit when it doesn't."

```
Simulation 1: Keep CozoDB (current)      → "Works, keep it"
Simulation 2: Drop CozoDB, use HashMaps  → "2000+ lines rewrite, maybe later"
Simulation 3: Trait abstraction           → "Elegant but premature"
Simulation 4: Hybrid                      → "Complexity for no gain"
Simulation 5: SQLite direct               → "Already failed twice"
Simulation 6: Embedded KV                 → "Reinventing CozoDB"
```

Every simulation except #1 concluded "not now." That's 5 simulations to confirm inaction.

### Shreyas Would Say
"You simulated 6 futures to decide to do nothing differently. That's research theater. The right output was: 'CozoDB works. Move on. Here's a 2-paragraph note on why we might drop it someday.' Save the 6 simulations for when you actually need to choose."

---

## Doc 5: `IMPLEMENTATION-v173-slim-snapshot-plan.md`

**LNO Rating: LEVERAGE (but bloated)**

### What It Does
Implementation plan: 11 tasks, 14 files, 712 estimated lines. The actual build spec.

### What's Right
- The codebase exploration findings (pipeline boundary at line 765, getter on FileStreamerImpl, CozoDB schema column nullability) — this is real engineering intelligence
- The handler column access audit — discovered Current_Code must be "" not NULL, interface_signature unused — prevents bugs

### What's Wrong
- **pt03 as a separate crate.** This is a `--format` flag. Not a crate. Not a CLI command. Not another test suite. One. Flag.
- **712 lines.** The actual novel code is maybe 300 lines. The rest is Cargo.toml boilerplate, CLI argument parsing, and duplicated logic between pt02/pt03 (which shouldn't exist as separate things).
- **11 tasks.** Should be 7:
  1. Slim types in entities.rs
  2. DB getter on streamer
  3. Export + import methods on CozoDbStorage
  4. pt02 crate (with `--format` flag)
  5. pt08 snapshot loader + startup integration
  6. Endpoint guards
  7. CLI subcommand + workspace config

### Shreyas Would Say
"You're building two tools when one tool with a flag does the job. Every additional crate is ongoing maintenance cost — its own Cargo.toml, its own test suite, its own entry in the workspace. The cost of pt03 isn't the 100 lines to write it. It's the infinite lines to maintain it."

---

## The Meta-Critique: Research vs Shipping

```
DOCS WRITTEN:          5
TOTAL LINES:           ~2,300
LINES THAT MATTERED:   ~200 (slim model insight + implementation findings)
LINES OF CODE SHIPPED: 0
```

### The Effort Distribution Problem

```
                            EFFORT
  ┌─────────────────────────────────────────────────┐
  │████████████████████████████████████              │  Research (90%)
  │██                                               │  Implementation (0%)
  │████                                             │  Actual decisions (10%)
  └─────────────────────────────────────────────────┘

                    vs. WHAT SHIPS VALUE

  ┌─────────────────────────────────────────────────┐
  │██                                               │  Research (5%)
  │████████████████████████████████████████████████  │  Implementation (90%)
  │██                                               │  Decisions (5%)
  └─────────────────────────────────────────────────┘
```

### What Should Have Happened

```
Day 1, Hour 1:
  "CozoDB breaks on Windows. Serialize to file instead."
  "MessagePack (serde-compatible, maintained, schema-safe)."
  "Drop code bodies — LLMs read files directly."
  → 3 sentences. Done deciding.

Day 1, Hour 2-8:
  Write code. Ship pt02.

Day 2:
  Users are running on Windows.
```

Instead:
```
Day 1: RESEARCH doc (14 formats analyzed)
Day 1: THESIS doc (6 architecture simulations)
Day 2: DECISION doc (endpoint tiers for full model — obsoleted same day)
Day 2: THESIS doc (slim model — the breakthrough, but over-analyzed)
Day 3: IMPLEMENTATION doc (plan for code that hasn't been written)
Day 3: Still zero lines of implementation code shipped.
```

---

## The Shreyas Doshi Playbook for v1.7.3

If I had to restart v1.7.3 from scratch with Shreyas's mindset:

### 1. Decision (10 minutes)
- MessagePack. Serde-compatible. Done.
- Drop code bodies. Keep edges + addresses.
- One crate: pt02 with `--format` flag.
- pt08 loads .ptgraph into CozoDB mem.

### 2. Implementation (the actual work)
- Write slim types
- Write export/import methods
- Write pt02 crate
- Wire pt08 loader + startup
- Add CLI command + guards
- Test end-to-end

### 3. Documentation (after shipping)
- One doc: "How pt02 works and why." 50 lines max.
- The research stays in git history for anyone who's curious.

### The Doshi Rule
> "The quality of a decision is not determined by the amount of analysis that preceded it. Some of the best decisions are fast decisions with just enough information."

---

## Action Items

| # | Action | LNO |
|---|--------|:---:|
| 1 | Kill pt03. Make it a `--format` flag on pt02. | Leverage |
| 2 | Stop writing research docs. Start writing code. | Leverage |
| 3 | Ship pt02 with MessagePack only. Add JSON flag later if asked. | Leverage |
| 4 | Archive DECISION doc — it's wrong (based on full model, not slim). | Overhead reduction |
| 5 | The IMPLEMENTATION doc is the only one that should guide coding. | — |
