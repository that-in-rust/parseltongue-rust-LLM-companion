# Prep-V200: Max Adoption Architecture Strategy

**Date**: 2026-02-16
**Context**: Rubber duck debugging of v2.0.0 architecture options. Two lenses applied: (1) pragmatic/risk analysis, (2) max adoption + max differentiation (Shreyas Doshi lens, time is not a constraint).

---

## Part 1: Pragmatic Lens (4 Options)

### Option 0: "Clean Room Rewrite" (8 rust-llm-* crates) — THE CURRENT PLAN

```
rust-llm-core
rust-llm-01-fact-extractor         (tree-sitter queries)
rust-llm-02-cross-lang-edges       (boundary detector)
rust-llm-03-rust-analyzer          (semantic depth)
rust-llm-04-reasoning-engine       (Ascent Datalog)
rust-llm-05-knowledge-store        (HashMap + indexes)
rust-llm-06-http-server            (Axum)
rust-llm-07-mcp-server             (rmcp)
```

#### What's GOOD

```
+ Clean slate. Zero technical debt.
+ Every crate independently publishable.
+ TypedAnalysisStore replaces CozoDB = no C++ dependency.
+ Ascent gives us recursive reasoning CozoDB never actually gave us.
+ Tree-sitter queries replace cursor walking = declarative, maintainable.
+ Cross-language edges = unique differentiator nobody else has.
```

#### What HURTS

```
- 8 CRATES from scratch.
  You're designing type systems, traits, error types, key formats,
  serialization formats, HTTP APIs, CLI args — ALL from zero.

- SECOND SYSTEM EFFECT.
  "We'll do everything right this time."
  That's what every rewrite says.
  You already built v1.x to v1.7.3 = 22 endpoints, 7 algorithms,
  12 languages, file watching. That's ~15K lines of working code.
  You're throwing away ALL of it and starting with 0 lines.

- rust-analyzer BRIDGE IS HARD.
  ra_ide/ra_hir are internal APIs. They break every release.
  You're betting a core crate on an unstable interface.

- SCOPE CREEP MAGNET.
  8 crates = 8 surfaces for scope creep.

- YOU LOSE v1.x USERS (if any) during the rewrite gap.
  v1.x works. v2.0.0 doesn't exist yet.
```

---

### Option 1: "Strangler Fig" — Gradually Replace v1.x Internals

```
IDEA: Don't rewrite. STRANGLE.

Keep pt01, pt08, parseltongue-core.
Replace their GUTS one piece at a time.

Phase 1: Replace cursor walking with tree-sitter queries
         ┌─────────────────────────────────┐
         │  pt01                            │
         │  ┌───────────┐  ┌────────────┐  │
         │  │ OLD cursor │→│ NEW queries │  │
         │  │  walking   │  │ (richer)   │  │
         │  └───────────┘  └────────────┘  │
         │       ↓ same CozoDB tables       │
         └─────────────────────────────────┘
         You extract MORE data, store it in EXISTING CozoDB.
         pt08 endpoints grow to expose new data.
         Ship. Users get value NOW.

Phase 2: Add Ascent reasoning on TOP of existing pipeline
         ┌──────────────────────────────────┐
         │  pt08                             │
         │  CozoDB → HashMap → algorithms   │
         │                  → Ascent rules  │← NEW
         │                  → new endpoints │← NEW
         └──────────────────────────────────┘
         Ascent consumes the SAME data CozoDB has.
         New endpoints serve derived facts.

Phase 3: Add rust-analyzer as OPTIONAL enrichment
         pt01 writes syntactic facts.
         pt04 writes semantic facts INTO SAME STORE.
         pt08 reads both. If pt04 ran, richer responses.

Phase 4: SWAP CozoDB → TypedAnalysisStore.
         One file changes. Store interface stays same.
         (Martin Fowler's "Branch by Abstraction")
```

#### What's GOOD

```
+ SHIP VALUE EVERY PHASE.
  Phase 1 alone = richer tree-sitter extraction = better queries.
  Users benefit immediately. No "wait for the rewrite."

+ LOWER RISK.
  Each phase is a small change. Tests exist.
  If Phase 3 (rust-analyzer) is too hard, you still shipped 1+2.

+ NO NAMING CRISIS.
  No rust-llm-* vs pt* debate.

+ PRESERVES 15K LINES OF TESTED CODE.
  22 endpoints keep working. 7 algorithms keep working.
```

#### What HURTS

```
- MESSY MIDDLE.
  For a while you have cursor walking AND queries.
  CozoDB AND Ascent. Old types AND new types.

- CozoDB DRAGS ALONG.
  RocksDB C++ dependency stays until Phase 4.
  Windows users still blocked.

- parseltongue-core BECOMES A KITCHEN SINK.

- THE CUT NEVER COMES.
  Phase 4 ("swap CozoDB out") keeps getting deferred
  because "everything already works."

- CAN'T RETHINK KEY FORMAT.
  ISGL1 keys are baked into CozoDB relations.
```

---

### Option 2: "v1.5 Enrichment" — Just Extract More, Ship What You Have

```
IDEA: The problem isn't the architecture.
      The problem is you're extracting 20% of tree-sitter data.
      Extract 100%. Ship.

Step 1: Add 10 new CozoDB relations to parseltongue-core

  EXISTING:                    NEW:
  CodeGraph (entities)         GenericBounds (entity, trait)
  DependencyEdges (edges)      VisibilityInfo (entity, vis)
                               FunctionModifiers (entity, async, unsafe)
                               Attributes (entity, attr_name, attr_value)
                               ReturnTypes (entity, type_string)
                               ParamTypes (entity, param, type_string)
                               ClosureCaptures (entity, var, kind)
                               ImportPaths (file, path)
                               StringLiterals (file, line, value)
                               MacroInvocations (entity, macro_name)

Step 2: Update tree-sitter extraction in pt01
Step 3: Add 10 new pt08 endpoints
Step 4: Ship v1.8.0 — 32 endpoints instead of 22

TOTAL EFFORT: ~3 new relations + ~10 new endpoints.
              No new crates. No new binary. No new prefix.
```

#### What's GOOD

```
+ FASTEST PATH TO VALUE.
+ TESTS EVERYTHING THAT MATTERS (proves tree-sitter queries work).
+ ZERO RISK. Each new relation is independent.
+ ANSWERS OPEN QUESTIONS ("Is ISGL1 right?" — find out by using it more).
```

#### What HURTS

```
- DOESN'T SOLVE THE REAL PROBLEM. CozoDB is still a pass-through.
- NO ASCENT. NO DATALOG. No recursive queries. No taint analysis.
- NO RUST-ANALYZER. Still only see what tree-sitter sees.
- STILL COUPLED TO ROCKSDB. Windows still broken.
- DIMINISHING RETURNS. More tables of stringly-typed DataValue tuples.
```

---

### Option 3: "Two Crates, Not Eight" — Minimal v2.0.0

```
IDEA: You don't need 8 crates. You need 2.

Crate 1: rust-llm-core
  - Typed fact structs (not strings)
  - Tree-sitter query extraction (all 12 languages)
  - TypedAnalysisStore (HashMap + indexes)
  - Ascent reasoning rules
  - Graph algorithms (rewritten on typed data)
  - Serialization (MessagePack)

  THIS IS THE LIBRARY. One crate. Publishable.

Crate 2: rust-llm
  - CLI binary
  - HTTP server (Axum)
  - MCP server (rmcp)
  - File watcher
  - All endpoint handlers

  THIS IS THE APPLICATION.

┌───────────────────────────────────────┐
│  rust-llm (binary)                    │
│   - ingest, serve, mcp, watch         │
│   - all HTTP/MCP handlers             │
│                                       │
│   depends on:                         │
│   ┌─────────────────────────────────┐ │
│   │  rust-llm-core (library)        │ │
│   │   - types, extraction, store,   │ │
│   │     Ascent, algorithms, serde   │ │
│   └─────────────────────────────────┘ │
└───────────────────────────────────────┘

rust-analyzer? Feature flag.
  cargo build --features rust-analyzer

Cross-language edges? Module inside rust-llm-core.
MCP? Module inside rust-llm binary.
```

#### What's GOOD

```
+ AVOIDS PREMATURE CRATE SPLITTING.
  8 crates = 8 Cargo.toml, 8 version numbers, 8 CI pipelines.
  With 2 crates: ONE interface boundary.

+ FASTER BUILD. 2 crates build faster than 8 crates.
+ SIMPLER DEPENDENCY MANAGEMENT.
+ FEATURE FLAGS FOR OPTIONAL DEPTH (like serde, tokio).
+ STILL A CLEAN BREAK. Zero shared code with v1.x.
```

#### What HURTS

```
- CORE CRATE GETS BIG. Potentially 20K+ lines in one crate.
- NOT INDEPENDENTLY PUBLISHABLE. Someone who wants just extraction
  has to depend on the whole core.
- FEATURE FLAGS ADD COMPLEXITY. #[cfg(feature = "...")] everywhere.
```

---

### Part 1 Verdict Table

```
                    SHIPS     RISK    SCOPE    CLEAN    COMPOUND
                    WHEN?                      BREAK?   EFFECT?
                    ─────     ────    ─────    ──────   ────────
Option 0 (8 crates) LATE      HIGH    HUGE     YES      YES
Option 1 (strangler) SOON     LOW     MEDIUM   NO       PARTIAL
Option 2 (enrich)   SOONEST   LOWEST  SMALL    NO       NO
Option 3 (2 crates) MEDIUM    MEDIUM  MEDIUM   YES      YES
```

---

---

## Part 2: Max Adoption + Max Differentiation Lens (4 Options)

**Constraint shift**: Time is NOT a constraint. LLM is coding it. Side project. The question is PURELY: what architecture creates MAXIMUM adoption and MAXIMUM differentiation?

### Critique of Option 0 Through Adoption Lens

```
THE PROBLEM WITH OPTION 0 ISN'T "TOO MANY CRATES."
THE PROBLEM IS THE CRATES AREN'T DIFFERENTIATED ENOUGH.

What someone searching crates.io sees:

  rust-llm-01-fact-extractor
    "Extracts code facts from source files"
    → So does tree-sitter. So does syn. So does ra_ide.
    → WHY would someone pick THIS?

  rust-llm-05-knowledge-store
    "Stores code analysis in HashMaps with indexes"
    → So does... a HashMap. Why a crate for this?

  rust-llm-06-http-server
    "HTTP server for code queries"
    → Extremely specific to YOUR workflow.
    → Nobody adopts "someone else's HTTP server."

THE CRATES ARE SLICED BY PIPELINE STAGE.
  extract → detect → analyze → reason → store → serve

BUT ADOPTION DOESN'T FOLLOW PIPELINE STAGES.
ADOPTION FOLLOWS PROBLEMS.

Nobody wakes up thinking:
  "I need a fact extractor for stage 1 of my pipeline."

People wake up thinking:
  "How do I find all unsafe call chains in my Rust project?"
  "How do I detect cross-language API mismatches?"
  "How do I give my LLM the right 3K tokens of context?"
  "How do I find architectural hotspots before they rot?"

OPTION 0 IS ORGANIZED BY HOW YOU BUILD IT.
MAX ADOPTION REQUIRES ORGANIZING BY HOW PEOPLE USE IT.
```

---

### Option A: "Problem-Shaped Crates" — Each Crate Solves One Problem That Hurts

```
IDEA: Don't organize by pipeline stage.
      Organize by PAIN POINT.

Each crate is a COMPLETE SOLUTION to ONE PROBLEM.
You install it. It solves the problem. Done.

No assembly required. No "also install crates 01 through 05."
```

#### The Crate Map (organized by WHAT HURTS)

```
rust-llm-core                          THE FOUNDATION
  │  Types, traits, tree-sitter infra
  │  Everyone depends on this
  │
  ├── rust-llm-context                 "I need to give my LLM
  │     │                               the right code context"
  │     │
  │     │  INPUT:  a codebase + a focus entity + a token budget
  │     │  OUTPUT: the best N tokens of context
  │     │
  │     │  This is the KILLER APP.
  │     │  The thing LLM agent builders actually need.
  │     │  cargo add rust-llm-context
  │     │  let ctx = rust_llm_context::extract("src/", "fn main", 4096);
  │     │  → structured, ranked, deduped, token-budgeted context
  │     │
  │     │  NOBODY ELSE OFFERS THIS AS A LIBRARY.
  │     │  Aider hardcodes it. Cursor hardcodes it.
  │     │  This makes it a crate anyone can use.
  │
  ├── rust-llm-graph                   "I need to understand my
  │     │                               codebase's architecture"
  │     │
  │     │  INPUT:  a codebase path
  │     │  OUTPUT: dependency graph + SCC + PageRank + k-core
  │     │          + Leiden clusters + entropy + CK metrics
  │     │
  │     │  One function call → full architectural analysis.
  │     │  let report = rust_llm_graph::analyze("src/");
  │     │  report.hotspots()    → Vec<Hotspot>
  │     │  report.cycles()      → Vec<Cycle>
  │     │  report.communities() → Vec<Community>
  │     │
  │     │  WHO WANTS THIS:
  │     │  - Tech leads doing architecture reviews
  │     │  - CI pipelines checking for coupling regression
  │     │  - LLM agents understanding codebase structure
  │
  ├── rust-llm-crosslang               "My codebase is Rust + 3
  │     │                               other languages, what
  │     │                               connects to what?"
  │     │
  │     │  INPUT:  a multi-language codebase
  │     │  OUTPUT: cross-language edges (FFI, WASM, PyO3, gRPC,
  │     │          message queues)
  │     │
  │     │  let edges = rust_llm_crosslang::detect(".");
  │     │  edges.ffi_boundaries()   → Rust<->C connections
  │     │  edges.wasm_exports()     → Rust→JS connections
  │     │  edges.grpc_contracts()   → service mesh map
  │     │
  │     │  NOBODY HAS THIS. ZERO COMPETITORS.
  │     │  This is the blue ocean crate.
  │
  ├── rust-llm-safety                  "Show me all unsafe paths,
  │     │                               taint flows, and
  │     │                               security concerns"
  │     │
  │     │  INPUT:  a Rust codebase
  │     │  OUTPUT: unsafe call chains, taint propagation,
  │     │          FFI boundary audit, Send/Sync violations
  │     │
  │     │  Uses Ascent rules internally.
  │     │  Users don't need to know about Datalog.
  │     │  They just get: "these 3 call chains reach unsafe code"
  │     │
  │     │  WHO WANTS THIS:
  │     │  - Security auditors
  │     │  - cargo-audit users who want deeper analysis
  │     │  - Safety-critical Rust teams
  │
  ├── rust-llm-rules                   "I want to write custom
  │     │                               code analysis rules"
  │     │
  │     │  THE MOAT CRATE.
  │     │  This is where institutional knowledge lives.
  │     │  Teams write rules like:
  │     │    "handler functions must not call database directly"
  │     │    "anything touching PII must go through sanitizer"
  │     │    "async functions must not hold mutex across await"
  │     │
  │     │  50 custom rules = can't switch away.
  │     │  This is the CodeQL play, but EMBEDDABLE.
  │
  └── rust-llm-server                  "I want all of the above
        │                               via HTTP/MCP"
        │
        │  The BINARY that wraps everything.
        │  rust-llm ingest .
        │  rust-llm serve
        │  rust-llm mcp
        │
        │  For people who don't write Rust.
        │  For LLM agents that speak HTTP.
        │  For Claude Desktop / Cursor via MCP.
```

#### Dependency Tree

```
  rust-llm-core ─────────────────────────────── FOUNDATION
       │
       ├──→ rust-llm-context ────────────────── STANDALONE
       ├──→ rust-llm-graph ──────────────────── STANDALONE
       ├──→ rust-llm-crosslang ──────────────── STANDALONE
       ├──→ rust-llm-safety ─────────────────── uses graph + rules
       ├──→ rust-llm-rules ──────────────────── uses core facts
       │
       └──→ rust-llm-server ─────────────────── uses ALL above
```

#### What's GOOD

```
+ EACH CRATE HAS A README THAT SELLS ITSELF.

  rust-llm-context README:
  "Give your LLM the right code context. One function call.
   4,096 tokens of the most relevant code for any entity."
   → Aider developers, Cursor plugin authors, agent builders
     IMMEDIATELY understand the value.

  rust-llm-crosslang README:
  "Detect cross-language connections in your codebase.
   Finds FFI boundaries, WASM exports, gRPC contracts."
   → Microservice teams, polyglot shops, security auditors
     IMMEDIATELY understand the value.

  Compare to rust-llm-01-fact-extractor:
  "Extracts facts from source code using tree-sitter queries"
  → ...so? What do I DO with facts?

+ ADOPTION IS INDEPENDENT.
  Someone can use rust-llm-context without knowing
  rust-llm-graph exists. Just: cargo add rust-llm-context.

+ THE BINARY IS FOR "GIVE ME EVERYTHING" USERS.
  Power users use crates directly.

+ NATURAL DIFFERENTIATION.
  rust-llm-context: no competitor as a LIBRARY
  rust-llm-crosslang: no competitor AT ALL
  rust-llm-safety: CodeQL competitor but EMBEDDABLE
  rust-llm-rules: institutional knowledge moat

+ MARKETING WRITES ITSELF.
  "rust-llm-context: like tree-sitter, but for LLMs"
  "rust-llm-crosslang: see what connects your Rust to your C"
  "rust-llm-safety: find every path to unsafe in your codebase"
  Each crate is a tweet-sized pitch.
```

#### What HURTS

```
- SHARED INTERNALS ARE HIDDEN.
  Someone who wants JUST the Ascent reasoning engine
  can't get it without pulling rust-llm-safety or rust-llm-rules.

  Counter: rust-llm-core re-exports the primitives.
  Power users go to core. Problem-solvers go to the problem crate.

- DUPLICATION RISK.
  rust-llm-context and rust-llm-graph both need to parse code.

  Counter: Both depend on rust-llm-core which has extraction.
  The problem crates COMPOSE core primitives.

- MORE CRATES = MORE MAINTENANCE.
  Counter: Time is free. LLM maintains it. Not a constraint.
```

---

### Option B: "Protocol + Ecosystem" — Own the Format, Not Just the Tool

```
IDEA: The biggest wins in dev tools come from PROTOCOLS, not tools.

  LSP didn't win because VS Code is great.
  LSP won because it DEFINED HOW editors talk to language servers.

  Protobuf didn't win because gRPC is fast.
  Protobuf won because it DEFINED HOW services describe their data.

  SCIP (Sourcegraph) tried this for code intelligence.
  But SCIP only covers navigation (go-to-definition, find-references).
  It doesn't cover: architecture, safety, cross-language, LLM context.

WHAT IF rust-llm DEFINED THE PROTOCOL for code analysis facts?
```

#### The Architecture

```
Layer 1: THE PROTOCOL (this is the moat)
─────────────────────────────────────────
  rust-llm-facts
    │
    │  A crate that defines:
    │  - CodeFact enum (entity, edge, attribute, cross-lang-edge)
    │  - FactSet struct (collection of facts)
    │  - Serialization format (MessagePack + JSON schema)
    │  - Versioned schema (v1, v2, v3...)
    │
    │  ZERO DEPENDENCIES except serde.
    │  ZERO logic. Just types and serialization.
    │
    │  This is the LINGUA FRANCA of code analysis.
    │  Anyone can produce facts. Anyone can consume facts.
    │
    │  Like protobuf for code analysis.

Layer 2: PRODUCERS (things that CREATE facts)
─────────────────────────────────────────
  rust-llm-extract-treesitter
    │  Produces FactSet from tree-sitter parsing
    │  12 languages
    │
  rust-llm-extract-rust-analyzer
    │  Produces FactSet from rust-analyzer
    │  Semantic depth for Rust
    │
  rust-llm-extract-lsp
    │  Produces FactSet from ANY LSP server
    │  Generic bridge: pyright, gopls, tsserver, clangd
    │  → instant depth for Python, Go, TS, C++
    │     WITHOUT writing language-specific extractors
    │
  THIRD PARTY PRODUCERS:
    │  Anyone can write a producer.
    │  Semgrep findings → FactSet? Sure.
    │  SonarQube results → FactSet? Sure.
    │  Just implement: fn produce() -> FactSet

Layer 3: CONSUMERS (things that USE facts)
─────────────────────────────────────────
  rust-llm-reason
    │  Ascent Datalog rules on FactSet
    │  Derives new facts from existing facts
    │
  rust-llm-graph
    │  Graph algorithms on FactSet
    │  SCC, PageRank, k-core, Leiden, entropy
    │
  rust-llm-context
    │  LLM context extraction from FactSet
    │  Token-budgeted, ranked, deduped
    │
  rust-llm-safety
    │  Security analysis on FactSet
    │  Unsafe chains, taint, FFI audit
    │
  THIRD PARTY CONSUMERS:
    │  CI dashboard? Consume FactSet.
    │  IDE plugin? Consume FactSet.
    │  Write your own consumer in Python, Go, whatever.

Layer 4: THE BINARY (wraps everything)
─────────────────────────────────────────
  rust-llm
    │  CLI: rust-llm ingest → FactSet file
    │  CLI: rust-llm serve  → HTTP server on FactSet
    │  CLI: rust-llm mcp    → MCP server on FactSet
    │  CLI: rust-llm query  → Ascent rules on FactSet
```

#### The Key Insight: LSP for Code Analysis

```
  BEFORE LSP:
    Every editor had to write a plugin for every language.
    N editors × M languages = N×M plugins.

  AFTER LSP:
    Every editor speaks LSP. Every language speaks LSP.
    N editors + M languages = N+M implementations.

  BEFORE rust-llm-facts:
    Every analysis tool has its own format.
    Semgrep SARIF ≠ CodeQL SARIF ≠ SonarQube JSON ≠ custom.
    Tools can't compose. Results can't merge.

  AFTER rust-llm-facts:
    tree-sitter → FactSet
    rust-analyzer → FactSet
    LSP server → FactSet
    semgrep → FactSet (via adapter)
    ────────────────────
    ALL feed into the SAME reasoning engine.
    ALL queryable by the SAME rules.
    ALL serveable by the SAME server.

  THIS IS THE 10x PLAY.
  You're not building a tool.
  You're building the STANDARD.
```

#### What's GOOD

```
+ THE PROTOCOL IS THE MOAT.
  If rust-llm-facts becomes the standard format,
  EVERY new producer and consumer strengthens YOUR ecosystem.
  Network effect.

+ LSP BRIDGE IS GENIUS.
  rust-llm-extract-lsp means you get SEMANTIC DEPTH
  for Python (pyright), Go (gopls), TypeScript (tsserver),
  C++ (clangd) — WITHOUT writing extractors for each.

  You call the LSP server's textDocument/definition,
  textDocument/references, textDocument/hover —
  and convert responses to FactSet.

  This LEAPFROGS tree-sitter for non-Rust languages.
  Tree-sitter = syntax. LSP = semantics.
  You get BOTH for every language that has an LSP server.
  Which is: ALL of them.

+ THIRD-PARTY EXTENSIBILITY.
  Producers and consumers are decoupled by the protocol.

+ COMPOSABILITY.
  Multiple producers → merge FactSets → richer analysis.
  Cross-language edges EMERGE from the merged fact set.

+ THE BINARY IS JUST ONE CONSUMER.
  The protocol is language-agnostic.
```

#### What HURTS

```
- PROTOCOL DESIGN IS HARD.
  Get the FactSet schema wrong and you're stuck with it.

  Counter: Start with v1 that covers tree-sitter output.
  MessagePack is forwards-compatible (ignore unknown fields).

- "STANDARDS" USUALLY FAIL. (XKCD 927)

  Counter: SARIF is for findings (lint results).
           SCIP is for navigation (go-to-def).
           Neither covers ARCHITECTURE.
           Neither covers LLM CONTEXT.
           Neither covers CROSS-LANGUAGE EDGES.
           rust-llm-facts fills a gap, not compete in a crowd.

- LSP EXTRACTION IS NOISY.
  LSP servers are designed for IDE use, not batch analysis.

  Counter: Cache aggressively. Parallelize.
  pyright can analyze a full codebase in seconds.

- MORE CRATES THAN OPTION 0. ~10-12 crates.
  Counter: Time is free. Each crate is focused.
```

---

### Option C: "Embeddable CodeQL" — The Query Language IS the Product

```
IDEA: CodeQL's adoption comes from the QUERY LANGUAGE.
      Security researchers write QL queries.
      They SHARE queries. Queries become institutional knowledge.

      What if rust-llm's differentiator isn't the analysis —
      it's that you can WRITE YOUR OWN analysis rules
      and they run as compiled Rust code via Ascent?
```

#### The Architecture

```
The product is: A RULE ENGINE FOR CODE.

  ┌──────────────────────────────────────────────┐
  │                                              │
  │   User writes rules in a DSL:                │
  │                                              │
  │   rule unsafe_chain(f, g) :-                 │
  │     calls(f, g),                             │
  │     is_unsafe(g).                            │
  │                                              │
  │   rule unsafe_transitive(f, g) :-            │
  │     calls(f, h),                             │
  │     unsafe_chain(h, g).                      │
  │                                              │
  │   rule pii_leak(src, sink) :-                │
  │     touches_pii(src),                        │
  │     reachable(src, sink),                    │
  │     is_external_api(sink).                   │
  │                                              │
  │   rule architectural_violation(f) :-         │
  │     in_module(f, "handlers"),                │
  │     calls(f, g),                             │
  │     in_module(g, "database").                │
  │                                              │
  └──────────────────────────────────────────────┘
                        │
                        ▼
  ┌──────────────────────────────────────────────┐
  │  rust-llm compiles rules to Ascent Datalog   │
  │  at BUILD TIME → native Rust code            │
  │  runs over extracted facts                   │
  │  returns: Vec<Violation>                     │
  └──────────────────────────────────────────────┘
```

#### The Crate Map

```
  rust-llm-core               TYPES + EXTRACTION
    │
    ├── rust-llm-engine        THE RULE ENGINE
    │     │
    │     │  - Parses .rlm rule files (rust-llm markup)
    │     │  - Compiles to Ascent at build time
    │     │  - OR interprets at runtime (for REPL)
    │     │  - Built-in relations: calls, uses, implements,
    │     │    is_async, is_unsafe, return_type, in_module,
    │     │    has_attribute, captures, etc.
    │     │  - User adds: custom relations + rules
    │
    ├── rust-llm-rules-std     STANDARD RULE LIBRARY
    │     │
    │     │  30+ built-in rules, organized by category:
    │     │
    │     │  safety/
    │     │    unsafe_chains.rlm
    │     │    ffi_boundary_audit.rlm
    │     │    send_sync_violations.rlm
    │     │
    │     │  architecture/
    │     │    layer_violations.rlm
    │     │    circular_deps.rlm
    │     │    god_object_detection.rlm
    │     │    coupling_hotspots.rlm
    │     │
    │     │  llm/
    │     │    context_ranking.rlm
    │     │    token_budgeting.rlm
    │     │    dependency_ordering.rlm
    │     │
    │     │  cross_lang/
    │     │    ffi_mismatches.rlm
    │     │    wasm_boundary.rlm
    │     │    grpc_contracts.rlm
    │     │
    │     │  THESE RULES ARE THE PRODUCT.
    │     │  Open source. Forkable. Extendable.
    │     │  Like semgrep's rule registry, but for architecture.
    │
    ├── rust-llm-graph         GRAPH ALGORITHMS
    │     │  Algorithms ARE rules in the engine.
    │
    └── rust-llm               THE BINARY
          │
          │  rust-llm ingest .                    → extract facts
          │  rust-llm query unsafe_chains         → run one rule
          │  rust-llm audit safety/               → run rule suite
          │  rust-llm serve                       → HTTP server
          │  rust-llm mcp                         → MCP server
          │  rust-llm repl                        → interactive query
          │
          │  AND THE KILLER FEATURE:
          │
          │  rust-llm init                        → creates .rust-llm/
          │    .rust-llm/
          │      rules/
          │        my_team_rules.rlm              ← YOUR rules
          │      config.toml                      ← YOUR settings
          │
          │  Teams check .rust-llm/ into their repo.
          │  CI runs: rust-llm audit
          │  Custom architectural rules enforced on every PR.
          │
          │  THIS IS HOW CodeQL WORKS.
          │  But CodeQL is GitHub-only, proprietary, cloud.
          │  rust-llm is local, open source, embeddable.
```

#### Why The Rules Are The Moat

```
  MONTH 1:  Team writes 5 custom rules.
            "handlers can't call DB directly"
            "PII types must go through sanitizer"
            "new endpoints need auth middleware"

  MONTH 6:  Team has 30 rules.
            Architectural decisions ENCODED as rules.
            New devs learn architecture BY READING RULES.
            CI catches violations BEFORE code review.

  MONTH 12: Team has 80 rules.
            These rules ARE the team's architecture docs.
            Switching away means losing 80 rules.
            Rewriting means re-encoding a year of decisions.

  THIS IS THE LOCK-IN. NOT THE TOOL. THE RULES.
```

#### What's GOOD

```
+ CODEQL'S MOAT, BUT OPEN AND EMBEDDABLE.
  CodeQL is:
    - GitHub Actions only (cloud lock-in)
    - Proprietary query language (vendor lock-in)
    - Security focused only (narrow)
  rust-llm is:
    - Local first (no cloud)
    - Ascent/Datalog (standard, embeddable)
    - Architecture + safety + LLM (broad)

+ THE RULE FILE FORMAT IS SHAREABLE.
  Like semgrep rules or ESLint configs.
  A COMMUNITY of rules forms.

+ LLMs CAN WRITE RULES.
  "Write me a rule that finds all functions
   that take user input and reach a database query
   without going through validation."
  An LLM can generate .rlm files.
  The tool makes itself smarter via LLM-generated rules.

+ REPL IS A KILLER DEMO.
  $ rust-llm repl
  > calls(F, G), is_unsafe(G)?
  rust:fn:process_buffer → rust:fn:unsafe_parse
  rust:fn:handle_input → rust:fn:raw_pointer_deref
```

#### What HURTS

```
- DSL DESIGN IS GENUINELY HARD.
  Counter: Start with raw Ascent syntax. Don't invent a language.

- RUNTIME VS COMPILE-TIME tension.
  Counter: Two modes. Interpret for REPL. Compile for production.

- "ARCHITECTURE RULES" IS A NICHE MARKET.
  Counter: Standard rules do the work for 99%.
  The 1% who write custom rules = enterprise lock-in.
```

---

### Option D: "LLM-Native Code Intelligence" — The AI Sees Code Through Your Eyes

```
IDEA: Every other code analysis tool was built for HUMANS.
      What if you built one specifically for LLMs?

      Not "code analysis that also works for LLMs."
      CODE ANALYSIS THAT IS *FOR* LLMs. BY DESIGN.
```

#### The Insight

```
  HOW HUMANS READ CODE:
    Open file → read top to bottom → follow imports → repeat
    Tools optimize for THIS: syntax highlighting, go-to-def, search

  HOW LLMs READ CODE:
    Receive context window → understand relationships →
    generate/modify code → need to know what's connected
    Tools optimize for THIS: ??? (nothing, currently)

  WHAT LLMs ACTUALLY NEED:
  1. "What are the 3K most relevant tokens for THIS task?"
  2. "What depends on the thing I'm about to change?"
  3. "What patterns does this codebase use?"
  4. "What constraints exist that I can't violate?"
  5. "What's the blast radius if I change X?"

  CURRENT STATE OF THE ART:
    - Cursor: dumps ~50 files, hopes for the best
    - Aider: repo-map via ctags, limited ranking
    - Continue: embeddings + RAG, semantic search
    - Claude Code: tree-sitter + grep, manual

  ALL OF THESE ARE HACKS.
  None of them understand ARCHITECTURE.
  None of them understand CONSTRAINTS.
  None of them understand CROSS-LANGUAGE.
```

#### The Crate Map

```
  rust-llm-core                    FOUNDATION
    │
    ├── rust-llm-parse             UNIVERSAL PARSING
    │     │  Tree-sitter (12 langs) + LSP bridge
    │     │  Extracts typed facts
    │     │  Output: FactSet
    │     │
    │     │  THIS IS A COMMODITY.
    │     │  Not the differentiator.
    │     │  Just needs to be correct and fast.
    │
    ├── rust-llm-understand        ARCHITECTURAL UNDERSTANDING
    │     │  Graph algorithms + Ascent rules
    │     │  Knows: coupling, cohesion, layers, boundaries,
    │     │         patterns, hotspots, cycles, communities
    │     │
    │     │  NOT a query engine.
    │     │  A COMPREHENSION engine.
    │     │  "This codebase has 3 layers.
    │     │   Layer 1 is handlers.
    │     │   Layer 2 is services.
    │     │   Layer 3 is storage.
    │     │   Handler→Storage violations exist at: X, Y, Z."
    │     │
    │     │  THIS IS DIFFERENTIATION #1.
    │
    ├── rust-llm-context            CONTEXT WINDOW OPTIMIZATION
    │     │
    │     │  THE KILLER CRATE.
    │     │
    │     │  Given:
    │     │    - A task description (natural language)
    │     │    - A codebase (already parsed into FactSet)
    │     │    - A token budget (e.g., 8192 tokens)
    │     │
    │     │  Returns:
    │     │    - The OPTIMAL set of code to include
    │     │    - Ranked by relevance to the task
    │     │    - Structured (not raw text)
    │     │    - With relationship annotations
    │     │    - Within the token budget
    │     │
    │     │  HOW IT RANKS:
    │     │    1. Entity directly mentioned in task → include
    │     │    2. Blast radius neighbors (1-hop) → include
    │     │    3. Same SCC (tightly coupled) → include
    │     │    4. Same Leiden community → maybe include
    │     │    5. High PageRank (architectural pillars) → include
    │     │    6. Cross-language edges → include if multi-lang
    │     │    7. Constraints/rules that apply → include
    │     │
    │     │  THE RESULT: LLM gets 4K tokens instead of 400K.
    │     │  Those 4K tokens are the RIGHT 4K tokens.
    │     │  Not grep results. Not file dumps.
    │     │  ARCHITECTURALLY RELEVANT context.
    │     │
    │     │  THIS IS DIFFERENTIATION #2.
    │     │  Nobody else does this.
    │     │  Cursor doesn't know about coupling.
    │     │  Aider doesn't know about communities.
    │     │  Continue doesn't know about blast radius.
    │
    ├── rust-llm-rules              CUSTOM RULE ENGINE
    │     │  Ascent rules, .rlm files, CI integration
    │     │
    │     │  THIS IS DIFFERENTIATION #3 (the moat).
    │
    └── rust-llm                    BINARY + SERVERS
          │
          │  MCP server as the PRIMARY interface.
          │  Not HTTP-first. MCP-FIRST.
          │
          │  Because the PRIMARY USER is an LLM.
          │
          │  Claude Desktop calls rust-llm via MCP.
          │  Cursor calls rust-llm via MCP.
          │  Custom agents call rust-llm via MCP.
          │
          │  HTTP exists for humans and dashboards.
          │  MCP exists for LLMs. MCP is first-class.
```

#### The Positioning

```
  "tree-sitter is for EDITORS.
   Language servers are for IDEs.
   rust-llm is for LLMs."

  Every previous code analysis tool:
    INPUT → ANALYSIS → OUTPUT FOR HUMANS

  rust-llm:
    INPUT → ANALYSIS → OUTPUT FOR LLMs

  The output format is different.
  The ranking is different.
  The context selection is different.
  The interface is different (MCP, not GUI).

  This is not a better Sourcegraph.
  This is INFRASTRUCTURE FOR THE AI CODING ERA.
```

#### What's GOOD

```
+ MAXIMUM DIFFERENTIATION.
  Nobody is building code analysis FOR LLMs.
  Everyone is building code analysis and then
  cramming the output into an LLM context window.

  Building FOR LLMs from day 1 means:
  - Output is token-budgeted (not afterthought)
  - Ranking uses architectural signals (not TF-IDF)
  - MCP is primary interface (not HTTP)
  - Facts are structured (not text dumps)

+ MASSIVE ADDRESSABLE MARKET.
  Every AI coding tool (Cursor, Aider, Continue, Claude Code,
  Copilot, Cody, Tabnine, Amazon Q, Gemini Code Assist)
  needs better code context.

  ALL of them would benefit from rust-llm-context.
  As a LIBRARY that they embed.
  As an MCP SERVER that they call.

+ THE PITCH IS IRRESISTIBLE.
  "Your LLM gets 99% fewer tokens and better results."
  That's not a feature. That's a CATEGORY.

+ MCP-FIRST IS FORWARD-LOOKING.
  MCP is becoming the standard for LLM tool integration.
  Being MCP-native means automatic integration with
  every MCP-compatible client.

+ COMPOUNDS WITH LLM IMPROVEMENTS.
  As LLMs get smarter, they can USE the structured
  architectural information MORE effectively.
  The tool gets better as its PRIMARY USER (LLMs) gets smarter.
```

#### What HURTS

```
- "FOR LLMs" IS A BET ON THE FUTURE.
  What if 1M token windows make context selection irrelevant?

  Counter: Even with 1M window, sending 400K tokens costs 20x more
  than 4K tokens. Token budgeting is ALWAYS valuable.
  Structured > unstructured even in infinite windows.

- MCP IS YOUNG. The protocol is still evolving.
  Counter: HTTP is the fallback. Being EARLY to MCP = being the default.

- "rust-llm-understand" IS VAGUE.
  Counter: It means 7 algorithms + Ascent rules,
  but PACKAGED as comprehension, not raw numbers.
```

---

---

## Part 3: The Synthesis

### Verdict Table (Adoption + Differentiation Lens)

```
                     ADOPTION    DIFFERENTIATION    MOAT       MARKET
                     ────────    ───────────────    ────       ──────
Option 0 (8 crates)  MEDIUM     MEDIUM             LOW        dev tools
  Pipeline-shaped crates. Clear to builders. Fuzzy to users.

Option A (problems)  HIGH       HIGH               MEDIUM     dev tools
  Problem-shaped crates. Each crate = one tweet pitch.

Option B (protocol)  HIGHEST    VERY HIGH           VERY HIGH  ecosystem
  The FORMAT is the product. Network effects. Third-party extensibility.

Option C (CodeQL)    MEDIUM     VERY HIGH           HIGHEST    enterprise
  Rules = institutional lock-in. Hard to adopt. Impossible to leave.

Option D (LLM-first) HIGH       HIGHEST            HIGH       AI tooling
  Category-defining. "code analysis for LLMs" = new category.
```

### These Options Are NOT Mutually Exclusive

```
Option D is the POSITIONING.
  "We build code intelligence for LLMs."
  This is the category you own.
  This is the README header. The tweet. The pitch.

Option A is the PACKAGING.
  Problem-shaped crates that each solve one pain.
  rust-llm-context, rust-llm-graph, rust-llm-crosslang.
  Each independently adoptable.

Option B is the ARCHITECTURE.
  FactSet protocol that decouples producers from consumers.
  Third-party extensibility. LSP bridge for instant depth.
  Network effects.

Option C is the MOAT.
  Rule engine + standard rule library + custom .rlm files.
  Teams encode institutional knowledge.
  Can't switch away.
```

### The Combined Architecture

```
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

This isn't 8 crates OR 2 crates.
It's 10-12 crates, each with a REASON TO EXIST,
organized around a PROTOCOL,
solving REAL PROBLEMS,
defended by a RULE ENGINE,
positioned for the AI CODING ERA.
```

---

## Appendix: Competitor Gap Analysis

```
                    Architecture  Cross-Lang  LLM-Optimized  Embeddable  Rules
                    ────────────  ──────────  ─────────────  ──────────  ─────
CodeQL              ✗             ✗           ✗              ✗           ✓
Semgrep             ✗             ✗           ✗              ✓           ✓
Sourcegraph/SCIP    ✗             ✗           ✗              ✗           ✗
SonarQube           partial       ✗           ✗              ✗           ✓
tree-sitter         ✗             ✗           ✗              ✓           ✗
rust-analyzer       ✗             ✗           ✗              ✗           ✗
rust-llm (v2.0.0)   ✓             ✓           ✓              ✓           ✓
```
