# Parseltongue Architecture Options: Rubber Duck Debug Session
## What's the Right Differentiated Architecture for an "LSP Companion for LLMs"?

**Session Date**: 2026-02-28
**Context**: Strategic architecture planning after competitive analysis of Sourcegraph, Lucene/ES, and code search landscape

---

## The Core Framing: What Problem Are We Actually Solving?

### LSP Was Built for Humans, Not LLMs

LSP (Language Server Protocol) was built for humans editing code in IDEs. It answers:
- "What's the definition of this symbol?"
- "What completions are available here?"
- "What's the type of this expression?"

It's **cursor-position-centric**. A human is staring at line 47 of `main.rs`, and they want to know about *that specific spot*.

### LLMs Ask Different Questions

LLMs don't work like that. An LLM isn't staring at a cursor position. It's trying to understand *enough of the codebase* to generate or modify code correctly:

- "What are the constraints I need to respect if I modify this function?"
- "What's the shape of the type system around this module?"
- "What patterns does this codebase use for error handling?"
- "If I add a new variant to this enum, what else needs to change?"
- "What's the minimal context I need to generate a correct implementation of trait X for type Y?"

LSP can technically answer some of these through many sequential requests, but it's like using a microscope when you need a map.

### The Framing

> **"LSP companion for LLMs" means: the thing that gives LLMs the map, while LSP gives humans the microscope.**

---

## Challenge 1: Why Can't LLMs Just Read Files?

The naive objection: Claude Code right now reads files, understands them, modifies them, runs `cargo check`, fixes errors. Why does it need a graph?

### The Context Window Budget Problem

An LLM working on a large Rust workspace has maybe 200K tokens of context. The workspace might be 2 million lines. The LLM has to choose what to read.

Today's AI coding agents use heuristics:
- Read the file being modified
- Read its imports
- Maybe grep for related symbols

**These heuristics are bad.** They miss:
- Transitive dependencies
- Trait impls in other crates
- Macros that rewrite the struct

### The Core Failure Mode

> **The LLM generates code that's locally correct but globally wrong.**

It satisfies the type checker for the function it can see, but violates an invariant established three modules away that it never read.

A graph solves this by answering: "given that the LLM is about to modify function X, what is the *minimal set of context* it needs to see to do so correctly?"

That's a **graph reachability problem**.

### Structural vs Semantic Context

**Structural context**: "function `foo` calls `bar` which calls `baz` which uses type `Qux` which implements `Serialize`." — Graph gives you this.

**Semantic context**: "the team convention is that all error types in this module use `thiserror`, and we never use `unwrap` in production code, and this particular function is hot-path so we avoid allocation." — Comments, commit history, PR discussions, coding patterns give you this.

LLMs need both. A graph gives structural context well. Semantic context is a separate problem.

---

## Challenge 2: The Existing Solutions Landscape

| Tool | Approach | Depth |
|------|----------|-------|
| **Aider's repo map** | Tree-sitter-based map of definitions + signatures | Level 1 (syntax) |
| **Cursor/Windsurf** | Proprietary: embedding search + tree-sitter + ? | Unknown |
| **Claude Code** | Agentic: grep, read files, run commands, LSP | Variable |
| **CodeGraphContext** | Neo4j via MCP, syntax-level | Level 1-2 |

**The gap**: Nobody is doing **compiler-level semantic graph as a context source for LLMs.**

---

## Challenge 3: Why Hasn't Anyone Done This Already?

### Possible Explanations

1. **"It's hard and nobody's gotten to it yet."** The intersection of "deeply understands compiler internals" and "deeply understands LLM context needs" and "has time to build OSS tools" is very small.

2. **"The ROI doesn't justify the complexity."** Tree-sitter gives you 70% of the value at 10% of the complexity. Aider's repo map works well enough for most use cases.

3. **"LLMs are getting good enough that context quality matters less."** A frontier model in 2027 might handle large codebases with just file reading and `cargo check` feedback loops.

4. **"The maintenance burden against compiler internals is prohibitive."** Keeping a tool synced with rust-analyzer's constantly evolving internals is a full-time job.

**Honest assessment**: Mix of 1, 2, and 4. The gap is real, but the reason it exists is partly because it's genuinely hard to justify the effort.

---

## The Creative Reframe: Context Quality as Measurable Metric

The problem with "better context for LLMs" is that it's vague. Let's make it concrete:

> **What if Parseltongue's value proposition isn't "better context" but "fewer tokens for the same accuracy"?**

Every token in the LLM's context window costs money (API calls) or capacity (fewer tokens for reasoning).

If Parseltongue can give the LLM a **2,000-token graph summary** that's equivalent to reading **50,000 tokens of source files**, that's a **25x efficiency gain**.

### The New Pitch

> "Parseltongue reduces your AI coding agent's context consumption by 10-25x for structural understanding tasks while improving accuracy on modification-impact questions from ~60% to ~95%."

That's not a vibes claim. That's **benchmarkable**.

### Is the Compression Claim Realistic?

Consider: an LLM is about to add a new method to a struct. To do it correctly, it needs:
- The struct's fields and their types
- What traits the struct implements
- What other methods exist (to avoid conflicts)
- What modules use this struct (to understand conventions)
- Any derive macros that might be affected

**From source files**: 5-10 files, each 200-500 lines = ~20,000-40,000 tokens, most irrelevant.

**From a graph**: Focused subgraph extraction = ~500-2,000 tokens in structured format.

**The compression is real.** The graph is a *lossy compression of the codebase that preserves structural relationships while discarding implementation details*.

---

## The Depth Spectrum — Where's the Sweet Spot?

| Level | What | Cost | Value |
|-------|------|------|-------|
| **0** | File/directory structure | Near-zero | Near-zero |
| **1** | Symbol-level (tree-sitter): signatures, file locations | Low | 60-70% |
| **2** | Dependency-level: symbol X uses symbol Y | Moderate | 75-80% |
| **3** | Type-relationship-level: type X implements trait Y | Higher | ~90% |
| **4** | Full semantic model: inference, borrow checking, macros | Very high | ~98% |
| **5** | Runtime behavior (execution traces) | Beyond static | TBD |

### Recommended Sweet Spot: Level 3

- Jump from Level 1 (what exists today) to Level 3 is **massive** for LLMs
- Level 3 achievable via `ra_ap_*` crates without persisting entire HIR
- Level 4 details (borrow checker, lifetimes) are mostly handled by compiler feedback

**Key insight**:
> Parseltongue doesn't need to replace the compiler's feedback. It needs to give the LLM enough structural understanding to get *close* on the first try, then the compiler refines.

---

## The MCP Protocol Question

### Two Modes Needed

**Mode 1: Pre-computed context injection**
- Before the LLM starts reasoning, analyze the request + relevant files
- Automatically inject graph summary into the prompt
- No tool call needed — the LLM just *has* the structural context
- This is how Aider's repo map works

**Mode 2: On-demand deep queries via MCP**
- For specific questions mid-reasoning: "what types implement trait X?"
- Explicit tool calls

**Mode 1 is probably more valuable** — zero-friction, the LLM doesn't need to know Parseltongue exists.

---

## Architecture Dimension 1: Storage Engine

### Option 1A: CozoDB (Current)

```
┌─────────────────────────────────────────────────────────────┐
│                       COZODB                                 │
│  Pros: Recursive queries (Datalog), blast radius in 1 query │
│       Embedded, SQLite/RocksDB backends                      │
│  Cons: Niche ecosystem, single maintainer                    │
└─────────────────────────────────────────────────────────────┘
```

### Option 1B: Custom Rust Graph DB (Native)

```rust
/// A graph database native to code structure
pub struct CodeGraph {
    nodes: SlotMap<NodeId, Node>,
    outgoing: DenseMap<NodeId, EdgeKind, SmallVec<[EdgeId; 4]>>,
    incoming: DenseMap<NodeId, EdgeKind, SmallVec<[EdgeId; 4]>>,
    name_index: FxHashMap<String, SmallVec<[NodeId; 2]>>,
    type_index: FxHashMap<u64, SmallVec<[NodeId; 2]>>,
    file_index: FxHashMap<Arc<str>, Vec<NodeId>>,
}

pub enum NodeKind {
    Module, Crate,
    Struct, Enum, Trait, TypeAlias,
    Function, Method, Const, Static,
    Field, Variant, Impl, Use, Macro,
}

pub enum EdgeKind {
    Contains, Calls, References, Implements,
    Extends, HasType, DependsOn, GeneratedBy,
}
```

**Pros**:
- Zero external dependencies for graph logic
- Specialized indices (type fingerprints, embeddings)
- Optimized for code-specific query patterns
- Better memory layout for cache efficiency
- Incrementality natively

**Cons**:
- Reimplementing traversal algorithms
- No declarative query language
- More code to maintain

### Option 1C: Hybrid (Custom Graph + SQLite Persistence)

```
┌─────────────────────────────────────────────────────────────┐
│                      HYBRID ARCHITECTURE                     │
│                                                              │
│  IN-Memory Graph (Custom Rust)                              │
│  • Fast traversal, specialized indices, zero-copy           │
│                         │                                    │
│                         │ load/save                          │
│                         ▼                                    │
│  SQLite (Persistence Layer)                                 │
│  • nodes, edges, metadata, vectors                          │
│  • Battle-tested reliability                                │
└─────────────────────────────────────────────────────────────┘
```

**Verdict**: This is probably the right answer for a production system.

---

## Architecture Dimension 2: Semantic Depth

| Option | Depth | Languages | Effort |
|--------|-------|-----------|--------|
| Tree-sitter only | Level 1-2 | 40+ | Low |
| rust-analyzer integration | Level 3-4 | Rust only | High |
| Hybrid (configurable) | Variable | Extensible | Medium |

**Recommended**: Hybrid with configurable depth. Query can specify "I need depth 3 for module X" or "Fast mode, depth 1 only."

---

## Architecture Dimension 3: Freshness Model

| Model | Pros | Cons |
|-------|------|------|
| **Static index** | Simple, predictable | Stale until re-index |
| **Live LSP bridge** | Always fresh | Requires editor running, latency |
| **Watch + incremental** | Near-real-time, no LSP | Complexity |

**Recommended**: Watch mode + incremental updates (Merkle tree for change detection, like Warp does).

---

## Architecture Dimension 4: Context Extraction Intelligence

### The Moat

The moat isn't the storage. The moat is **knowing what context to extract**.

### Option 4A: Rule-Based Extraction

```rust
fn extract_context_for_modification(
    graph: &CodeGraph,
    target: NodeId,
    modification_type: ModificationType,
) -> ContextSummary {
    match modification_type {
        ModificationType::AddField => {
            // Struct + trait impls + existing fields + usages
        }
        ModificationType::ChangeSignature => {
            // Function + callers + callees + trait method
        }
        // ... more rules
    }
}
```

**Pros**: Deterministic, debuggable, fast
**Cons**: Manual rules for each scenario

### Option 4B: LLM-Guided Extraction

```
Prompt: "The user wants to add async support to auth module.
        Available graph relationships: [list].
        Which entities and relationships are most relevant?"

LLM returns: { focus: [...], expand: [...], depth: 2 }
```

**Pros**: Adaptive, handles novel scenarios
**Cons**: Non-deterministic, adds latency/cost

### Option 4C: Hybrid (Rules + LLM Fallback)

```
1. Classify task type
2. If matches known pattern → rule-based
3. If novel → LLM-guided
4. Cache successful extractions as new rules
```

**Verdict**: This is the product differentiator. The extraction logic IS the product.

---

## Architecture Dimension 5: Interface Model

| Interface | How | Pros | Cons |
|-----------|-----|------|------|
| **MCP tools** | LLM calls tool | Standard protocol | LLM must know to call |
| **Context injection** | Auto-prepend to prompt | Zero-friction | May inject irrelevant context |
| **Smart proxy** | Parseltongue sits between user and LLM | Transparent, powerful | More infrastructure |

**Recommended**: Multi-modal — MCP tools for explicit queries, context injection for zero-friction.

---

## Four Differentiated Architectures

### Architecture A: "The Semantic Depth Play"

| Dimension | Choice |
|-----------|--------|
| Storage | Custom Rust graph + SQLite |
| Depth | rust-analyzer (Level 3-4) |
| Freshness | Watch + incremental |
| Extraction | Rules + LLM fallback |
| Interface | MCP + context injection |

**Differentiation**: "See what the compiler sees" — type relationships, trait impls, generics
**Target**: Rust-heavy teams, complex codebases
**Risk**: Rust-only, high maintenance burden

### Architecture B: "The Multi-Language Coverage Play"

| Dimension | Choice |
|-----------|--------|
| Storage | CozoDB |
| Depth | Tree-sitter (Level 1-2) |
| Freshness | Static index |
| Extraction | Rules only |
| Interface | MCP only |

**Differentiation**: "Works on any codebase, today" — 40+ languages
**Target**: Polyglot teams, broad adoption
**Risk**: Competes with Aider, lower differentiation

### Architecture C: "The Intelligence Play"

| Dimension | Choice |
|-----------|--------|
| Storage | Custom Rust + SQLite |
| Depth | Tree-sitter + optional plugins |
| Freshness | Watch + incremental |
| Extraction | LLM-guided with rule fallback ← THE MOAT |
| Interface | Smart proxy (auto-context injection) |

**Differentiation**: "Knows what the LLM needs before it asks"
**Target**: Power users, AI-forward teams
**Risk**: LLM calls add latency and cost

### Architecture D: "The Protocol Play"

Define a standard "Code Context Protocol" (CCP):
- Standard graph schema
- Standard query format
- Standard response format

Parseltongue becomes the reference implementation. Others build alternative servers, clients, tooling.

**Differentiation**: "The standard for code context"
**Risk**: Adoption chicken-and-egg, standards are hard

---

## Recommended Architecture: "Intelligent Context Server"

```
┌─────────────────────────────────────────────────────────────────────┐
│                    PARSELTONGUE v2 ARCHITECTURE                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  LAYER 1: GRAPH STORAGE (Custom Rust + SQLite)                      │
│  • In-memory graph for fast traversal                               │
│  • SQLite for persistence                                           │
│  • Specialized indices: name, type fingerprint, file                │
│  • Incremental updates via file watch                               │
│                                                                      │
│  LAYER 2: MULTI-DEPTH ANALYSIS                                      │
│  • Tree-sitter: Always (Level 1-2, all languages)                   │
│  • rust-analyzer: Optional (Level 3, Rust only)                     │
│  • Future: TypeScript, Python semantic plugins                      │
│                                                                      │
│  LAYER 3: CONTEXT EXTRACTION (THE MOAT)                             │
│  • 20-50 hand-crafted rules for common tasks                        │
│  • LLM-guided extraction for novel tasks (optional)                 │
│  • Learning: successful extractions become rules                    │
│  • Compression: graph → structured summary (~1-3KB)                 │
│                                                                      │
│  LAYER 4: INTERFACE                                                  │
│  • MCP tools: For explicit queries                                  │
│  • Context injection: Auto-prepend for configured tasks             │
│  • HTTP API: For integration with any agent                         │
│  • CLI: For human inspection/debugging                              │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Why This Architecture

| Dimension | Choice | Reasoning |
|-----------|--------|-----------|
| Storage | Custom Rust + SQLite | Control + reliability |
| Depth | Multi-level | Coverage + differentiation option |
| Freshness | Watch + incremental | Near-real-time, no LSP dependency |
| Extraction | Rules + LLM fallback | Deterministic common case, adaptive edge cases |
| Interface | Multi-modal | Works with any agent architecture |

---

## The One-Liner Differentiator

> **"Parseltongue is the context quality layer for AI coding agents — it knows what the LLM needs to see before the LLM knows it needs to see it."**

The moat isn't the graph. The moat is the **extraction intelligence** — the rules and heuristics that say "if you're adding a field to a struct, here's exactly what context you need."

---

## Should You Build a Custom Graph DB?

### Yes, But for the Right Reasons

**Don't build it because**:
- "CozoDB might be abandoned"
- "I want to own the stack"

**Do build it because**:
1. You can optimize for code-specific query patterns
2. You can add specialized indices (type fingerprints, embeddings)
3. You can make incremental updates native
4. You control memory layout for cache efficiency
5. You can add features CozoDB won't (hybrid vector + graph search)

### Recommended Approach

```
Phase 1: Thin wrapper around CozoDB
Phase 2: Identify performance bottlenecks via benchmarks
Phase 3: Build custom where it matters
```

Don't premature-optimize. Let the benchmarks guide what needs custom work.

---

## The Honest v2 Roadmap

### Phase 0 (2 weeks): Build the Eval
- 20 Rust codebase modification tasks of varying complexity
- Measure current AI agent performance with naive context
- Manually identify what "ideal context" looks like for each
- Quantify the tree-sitter vs semantic gap

### Phase 1 (1-2 months): Context Server, Tree-sitter Level
- CozoDB with persistent SQLite backend
- MCP server interface
- "Auto-context" mode: given file + intent, output relevant subgraph
- Benchmark against Phase 0 baseline

### Phase 2 (2-4 months): Add rust-analyzer Semantic Depth
- Type relationship edges, trait implementation graphs, generic bounds
- Focus on relationships that Phase 0 showed tree-sitter misses
- Don't persist full HIR — persist extracted relationship graph
- Re-benchmark

### Phase 3 (4-6 months): Intelligent Context Extraction
- The real moat
- Given task description + code location, extract minimum sufficient context
- May use small LLM to classify which relationships are relevant
- This is what makes Parseltongue "the LSP companion for LLMs"

### Phase 4 (6+ months): Multi-language
- Tree-sitter plugins for coverage
- Community semantic plugins for other languages

---

## Key Insights Summary

1. **LLMs need maps, not microscopes** — structural understanding at the right abstraction level
2. **Compression is measurable** — 2,000 tokens of graph can equal 50,000 tokens of source
3. **Level 3 is the sweet spot** — type relationships without full borrow checker complexity
4. **Extraction is the moat** — knowing what context to extract is more valuable than storage
5. **Multi-modal interface** — context injection for zero-friction, MCP for power queries
6. **Benchmark first** — prove the value before over-engineering the architecture

---

*Rubber duck session complete. Next step: Build the eval framework to validate assumptions.*
