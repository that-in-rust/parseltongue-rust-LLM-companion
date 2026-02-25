# v200-architecture-opinion-analysis: rust-llm v2.0.0 ELI5 + Opinion

**Generated**: 2026-02-16
**Context**: Analysis of Parseltongue v2.0.0 ("rust-llm") PRD, rubber duck Options A-D, three-layer bidirectional workflow, and relationship to production repo `that-in-rust/parseltongue-rust-LLM-companion`
**Status**: REFERENCE — captures architectural understanding and opinion at this point in time

---

## The Relationship Between Repos

```
that-in-rust/parseltongue-rust-LLM-companion   (GitHub, production)
│
│  = 904 commits, 92 stars, 670+ tests
│  = v1.0 → v1.7.2 (shipped, working, battle-tested)
│  = 4 crates: parseltongue, parseltongue-core, pt01, pt08
│  = codebase → tree-sitter → CozoDB → 22 HTTP endpoints
│
└── LOCAL (this machine): branch v173
    │
    │  UNCOMMITTED WORK:
    │  - PRD_v173.md (taint analysis, coverage bugs, MCP, Tauri)
    │  - THESIS-taint-analysis-for-parseltongue.md
    │  - v173-z3-tradeoff-analysis.md
    │  - v173-pt04-bidirectional-workflow.md
    │  - CR-v173-01 through CR-v173-04 (competitor research)
    │  - session-context-knowledge-dump.txt
    │
    └── THE v2.0.0 IDEA: rebrand to "rust-llm"
        clean-room rewrite, 8 problem-shaped crates,
        FactSet protocol, Ascent Datalog, MCP-first
```

---

## What v2.0.0 ("rust-llm") Actually IS

```
v1.x (TODAY)                          v2.0.0 (PROPOSED)
══════════════                        ══════════════════

parseltongue-core                     rust-llm-core
(types, tree-sitter,                  (FactSet protocol,
 CozoDB storage)                       typed entities, serde)

pt01-folder-to-cozodb-streamer        rust-llm-parse
(ingest → CozoDB)                     (tree-sitter .scm queries
                                       extract EVERYTHING)

pt08-http-code-query-server           rust-llm-cli
(22 HTTP endpoints)                   (MCP-first + HTTP)

─── THAT'S IT ───                     ─── PLUS THESE NEW ONES ───

(nothing)                             rust-llm-graph
                                      (7 algorithms, standalone)

(nothing)                             rust-llm-context    ← KILLER APP
                                      (token-budgeted architectural
                                       context for LLMs)

(nothing)                             rust-llm-crosslang  ← BLUE OCEAN
                                      (FFI, PyO3, WASM, gRPC edges)

(nothing)                             rust-llm-rules      ← MOAT
                                      (Ascent Datalog, .rlm files)

(nothing)                             rust-llm-safety
                                      (unsafe paths, taint)
```

---

## The Rubber Duck Analysis: Options A-D Synthesis

The rubber duck analysis proposed 4 options and concluded they're not mutually exclusive:

```
OPTION A: "Problem-Shaped Crates"
  Each crate solves ONE problem a developer would search for.
  rust-llm-context ("give my LLM the right code")
  rust-llm-crosslang ("detect cross-language connections")
  rust-llm-safety ("find unsafe paths")
  rust-llm-rules ("encode team knowledge")

OPTION B: "Protocol + Ecosystem"
  FactSet as interchange format. Producers + consumers decoupled.
  Third-party extensibility. LSP bridge for instant depth.
  Network effects.

OPTION C: "Embeddable CodeQL"
  Rule engine + .rlm files = institutional lock-in.
  50 custom rules = can't switch away.
  The CodeQL playbook ($500M+ acquisition) but open source.

OPTION D: "LLM-Native Code Intelligence"
  Purpose-built for LLMs, not humans.
  Output is token-budgeted, ranking uses architectural signals,
  MCP is primary interface. New category.

THE SYNTHESIS (from rubber duck):

  POSITION IT AS:   Option D ("code intelligence for LLMs")
  PACKAGE IT AS:    Option A (problem-shaped crates)
  ARCHITECT IT AS:  Option B (protocol + producers/consumers)
  DEFEND IT WITH:   Option C (embeddable rule engine)

  ┌─────────────────────────────────────────────┐
  │           "rust-llm"                        │
  │      Code Intelligence for LLMs             │
  │                                             │
  │   ┌──────────── PROTOCOL ──────────────┐    │
  │   │  rust-llm-facts (the interchange)  │    │
  │   └────────────────────────────────────┘    │
  │         ▲               ▲                   │
  │   ┌─────┴──────┐  ┌────┴────────┐          │
  │   │ PRODUCERS  │  │  CONSUMERS  │          │
  │   │ treesitter │  │  context    │← KILLER  │
  │   │ ra bridge  │  │  graph      │          │
  │   │ LSP bridge │  │  crosslang  │← UNIQUE  │
  │   │ 3rd party  │  │  safety     │          │
  │   └────────────┘  │  rules      │← MOAT    │
  │                   │  3rd party  │          │
  │                   └─────────────┘          │
  │                        │                    │
  │              ┌─────────┴──────────┐         │
  │              │    rust-llm        │         │
  │              │  MCP-first binary  │         │
  │              └────────────────────┘         │
  └─────────────────────────────────────────────┘
```

---

## The Three-Layer Bidirectional Architecture

The `pt04-bidirectional-workflow.md` is the intellectual breakthrough that ties v1.7.3 research to v2.0.0:

```
┌────────────────────────────────────────────────────────────┐
│                    LAYER 3: LLM (JUDGMENT)                 │
│                                                            │
│  "Is this cycle intentional?"        ← can't automate     │
│  "Is this code revenue-critical?"    ← needs context       │
│  "Name this cluster meaningfully"    ← needs language       │
│  "Should we refactor this?"          ← needs taste          │
│                                                            │
│  Only called when JUDGMENT needed. 5 prompts, not 230.     │
│  3 seconds, not 45.                                        │
├────────────────────────────────────────────────────────────┤
│                    LAYER 2: COMPILER (TRUTH)                │
│                                                            │
│  rust-analyzer / LSP bridge                                │
│                                                            │
│  "What type does this return?"       ← 100% correct       │
│  "Which trait dispatches here?"      ← 100% correct       │
│  "Is this async? unsafe?"            ← 100% correct       │
│  "What does this closure capture?"   ← 100% correct       │
│                                                            │
│  Zero cost — already computed during type-checking.        │
│  Replaces LLM guessing at things with correct answers.     │
├────────────────────────────────────────────────────────────┤
│                    LAYER 1: GRAPH (SPEED)                  │
│                                                            │
│  SCC, PageRank, k-core, Leiden, CK metrics,               │
│  blast radius, entropy, taint propagation                  │
│                                                            │
│  Milliseconds. Pure math on typed FactSet.                 │
│  Same 7 algorithms from v1.x, but now with TYPED edges.   │
└────────────────────────────────────────────────────────────┘
```

### The Five Workflows Rebuilt

```
WORKFLOW 1: "What are the modules in this codebase?"
═══════════════════════════════════════════════════════

BEFORE (v1.x bidirectional):
┌─────────┐   read 230 files   ┌─────────┐  keyword seeds  ┌─────┐
│  LLM    │──────────────────→ │  LLM    │────────────────→│ CPU │
│ (slow)  │   guess domains    │ (slow)  │  run Leiden      │     │
└─────────┘                    └─────────┘                  └─────┘
45 seconds, 91% accuracy

AFTER (v2.0 three-layer):
┌──────────────┐  trait dispatch  ┌─────┐  clusters  ┌─────┐  names
│rust-analyzer │─────────────────→│ CPU │───────────→│ LLM │──────→
│  (instant)   │  GROUND TRUTH    │     │  Leiden     │(fast)│ result
└──────────────┘                  └─────┘            └─────┘
0.8 seconds, 96% accuracy


WORKFLOW 2: "Is this dependency cycle a bug?"
═══════════════════════════════════════════════

BEFORE:  ALL 5 cycles → LLM reads code → guesses pattern
AFTER:   compiler says "all edges = trait dispatch" → INTENTIONAL (no LLM)
         compiler says "all edges = direct calls"  → VIOLATION (no LLM)
         compiler says "mixed"                     → ask LLM (1 of 5)

4 of 5 cycles classify themselves. LLM only needed for ambiguity.


WORKFLOW 3: "Is this complexity essential or accidental?"
═════════════════════════════════════════════════════════

BEFORE:  LLM reads 500 lines, guesses "single responsibility"
AFTER:   compiler counts: "dispatches through 5 traits from 4 modules"
         5 responsibilities = accidental. No guessing needed.


WORKFLOW 4: "What's the real tech debt priority?"
════════════════════════════════════════════════════

BEFORE:  SQALE x business_weight(LLM)
AFTER:   SQALE x business_weight(LLM)
               x padding_waste(compiler)      ← 40 bytes wasted
               x unnecessary_pub(compiler)     ← 12 pub fn nobody calls
               x unused_impls(compiler)        ← 3 dead trait impls


WORKFLOW 5: "How should we refactor this?"
═══════════════════════════════════════════

BEFORE:  LLM reads source → pattern-matches → suggests refactoring
AFTER:   compiler finds: "StripeProcessor and PayPalProcessor both have
         process(), refund(), authorize()" → EXTRACT TRAIT
         LLM writes: "Create PaymentProcessor trait" (from evidence, not vibes)
```

---

## Full v2.0.0 Data Flow

```
YOUR CODEBASE
├── src/auth.rs          (Rust)
├── src/api.py           (Python, via PyO3)
├── proto/service.proto  (gRPC)
└── web/handler.ts       (TypeScript, via WASM)

     │
     │  PRODUCERS (extract facts)
     │
     ├──→ rust-llm-parse (tree-sitter)
     │      entities: fn authenticate, class ApiClient, ...
     │      edges: calls, imports, inheritance
     │      NEW: generics, visibility, async, unsafe, closures
     │
     ├──→ rust-analyzer bridge (Rust-only, deep)
     │      typed edges: TraitMethod dispatch, DynDispatch
     │      trait impls, closure captures, type layouts
     │      100% compiler truth
     │
     └──→ .proto parser
            service definitions, RPC methods
                           │
                           ▼
                  ┌────────────────┐
                  │    FACTSET     │
                  │                │
                  │  typed entities│
                  │  typed edges   │
                  │  cross-lang    │──→ MessagePack on disk
                  │  taint labels  │
                  │  attributes    │
                  └───────┬────────┘
                          │
     CONSUMERS (analyze facts)
     │
     ├──→ rust-llm-crosslang
     │      "auth.rs calls api.py via PyO3 #[pyfunction]"
     │      "handler.ts imports auth.rs via WASM export"
     │      "service.proto connects api.py to handler.ts via gRPC"
     │
     ├──→ rust-llm-graph
     │      SCC: these 3 functions form a cycle
     │      PageRank: authenticate() is the most central function
     │      Leiden: 4 communities detected
     │      blast radius: changing this affects 47 entities
     │
     ├──→ rust-llm-safety
     │      taint: request.body → authenticate → db.query (NO SANITIZER!)
     │      unsafe: process_buffer → unsafe_parse → raw_pointer_deref
     │      CWE-89: SQL injection path detected
     │
     ├──→ rust-llm-rules (Ascent Datalog)
     │      .rlm: "handlers must not call database directly" → VIOLATION
     │      .rlm: "PII types must go through sanitizer" → 2 violations
     │      .rlm: "async fns must not hold mutex across await" → OK
     │
     └──→ rust-llm-context          ← THE KILLER APP
            │
            │  INPUT:  "fix the SQL injection in authenticate()"
            │          token budget: 4096
            │
            │  RANKING:
            │    1. authenticate() itself           (direct mention)
            │    2. db.query() it calls             (1-hop callee)
            │    3. request handler that calls it   (1-hop caller)
            │    4. sanitize() in same module       (same community)
            │    5. AuthService trait definition     (trait dispatch)
            │    6. Cross-lang: PyO3 bridge         (cross-lang edge)
            │
            │  OUTPUT: 4,096 tokens of THE RIGHT CODE
            │          with relationship annotations
            │          with "what's NOT included" report
            │
            └──→ LLM gets this instead of 400K raw tokens
                 99% reduction. Architecturally ranked. Not grep.
```

---

## Opinion: What's Right, What's Risky

### What's Genuinely Brilliant

1. **The three-layer insight** (compiler truth + LLM judgment + graph speed). 45s→3.3s. 91%→96%. This is the real intellectual breakthrough.

2. **"CozoDB was a pass-through."** Admitting the core DB was unused for its primary purpose (Datalog) takes honesty. Replacing with typed in-memory FactSet + Ascent is the right call.

3. **Cross-language = genuine blue ocean.** 38 years of papers, zero production tools. FFI/PyO3/WASM detection is deterministic. Defensible differentiation.

4. **Context optimizer as embeddable library.** `cargo add rust-llm-context` solving a problem every AI coding tool has. Academic papers prove graph-ranked > flat retrieval (GraphCoder +6.06 EM, InlineCoder +29.73% EM).

5. **v1.x lessons table is brutally honest.** 11 items that say "we got this wrong" with specific fixes. This is rare.

### What's Risky

1. **Key format design replacing ISGL1.** The PRD says this is week 1-2 but it deserves its own RFC. Everything builds on it. Get it wrong, rebuild everything.

2. **22-week timeline on a side project.** Realistic if LLM codes it, but sustained effort.

3. **rust-analyzer in-process.** Proven pattern (cargo-semver-checks) but full workspace loading has memory implications for the "embeddable" story.

4. **The .rlm rule format.** DSL design is genuinely hard. "Just use Ascent syntax" is the right starting point but may not survive contact with non-Rust users.

5. **"rust-llm" as a name collides** with existing crates on crates.io. "Parseltongue" has 92 stars + 904 commits of SEO. Rebranding is a loss.

### Precedent Table (What's Proven vs New)

```
CRATE                    PRECEDENT              RISK
──────────────────────  ────────────────────── ──────
rust-llm-core           parseltongue-core ✓    Key format is MEDIUM
rust-llm-parse          pt01 ✓                 .scm queries are LOW
rust-llm-graph          7 algorithms ✓         Port is LOW
rust-llm-context        smart-context ✓        Ranking is NEW
rust-llm-cli            pt08 ✓                 MCP is LOW-MEDIUM
rust-llm-crosslang      NOTHING                100% NEW
rust-llm-rules          CozoDB Datalog ✓       Ascent swap is LOW
rust-llm-safety         taint thesis ✓         Implementation is MEDIUM

SCORE: ~75% proven ground, ~25% genuinely new
```

### Uncertainty Table

```
REAL UNCERTAINTIES (3):
  1. Cross-language edge detection accuracy    MEDIUM
  2. MCP protocol stability (rmcp v0.15)       LOW-MEDIUM
  3. Key format design replacing ISGL1         MEDIUM

MINOR UNCERTAINTIES (2):
  4. Tree-sitter per-language grammar quirks   LOW
  5. rust-analyzer project loading setup       LOW

ELIMINATED (WAS FUD, NOW PROVEN):
  - Ascent scaling: same Datalog math as CozoDB, compiled Rust
  - rust-analyzer API breakage: pin the tag, ~15 API calls
```

### Build Order Assessment

```
Phase 1 (weeks 1-6):   Foundation     rust-llm-core, parse, graph
                        THIS IS THE HARDEST (key format)

Phase 2 (weeks 7-10):  Killer App     rust-llm-context, cli
                        THIS GETS ATTENTION

Phase 3 (weeks 11-18): Differentiators  crosslang, rules
                        THIS BUILDS MOAT

Phase 4 (weeks 19-22): Specialized     safety (taint, unsafe)
                        THIS LEVERAGES EVERYTHING
```

### The Bottom Line

The PRD is strong. The three-layer architecture (compiler + LLM + graph) is the insight that makes this more than "parseltongue with more crates." The FactSet protocol is good internal architecture, not premature standardization. The build order (foundation → killer app → differentiators → specialized) is correct.

Critical path: don't start coding until the key format RFC is done. Everything else can be iterated. The key format can't.

---

## Connected Documents

- **v1.7.3 PRD**: `docs/PRD_v173.md` — current version's roadmap including taint analysis
- **Taint Thesis**: `docs/THESIS-taint-analysis-for-parseltongue.md` — 786 lines, feeds into rust-llm-safety
- **Z3 Tradeoff**: `docs/v173-z3-tradeoff-analysis.md` — why Datalog over Z3
- **Bidirectional Workflow**: `docs/v173-pt04-bidirectional-workflow.md` — three-layer architecture detail
- **Competitor Research**: `docs/CR-v173-01.md` through `docs/CR-v173-04-oh-my-pi.md`
- **Session Context**: `docs/session-context-knowledge-dump.txt` — full context at time of analysis
- **Production Repo**: `https://github.com/that-in-rust/parseltongue-rust-LLM-companion`

---

*Generated 2026-02-16. Analysis based on v2.0.0 PRD draft, rubber duck Options A-D, pt04 bidirectional workflow document, and production repo inspection.*
