# Parseltongue Competitive Research: Existing Tools in the Space

**Research Date:** 2026-03-02
**Purpose:** Understand what exists, what's solved, what Parseltongue can innovate on

---

## Executive Summary

The code search / LLM context space is **crowded but shallow**. Most tools focus on:
1. **Semantic search** (embeddings + vector DB)
2. **Code chunking** (tree-sitter AST parsing)
3. **MCP integration** (Claude/Cursor/Codex)

**What's MISSING (Parseltongue opportunity):**
1. **Rust compiler integration** (ra_ap_*, MIR, type inference)
2. **True call graph analysis** (not just text-based)
3. **Graph algorithms** (centrality, SCC, Leiden communities)
4. **Blast radius / impact analysis** (who breaks if I change X)
5. **Disambiguation UX** (option cards with confidence scores)
6. **Execution flow tracing** (control dependencies, data flow)

---

## Tier 1: Direct Competitors (Semantic Code Search)

### 1. SeaGOAT (1270★)
**URL:** https://github.com/kantord/SeaGOAT

**What it does:**
- Local-first semantic code search
- Vector embeddings for conceptual queries
- Uses ripgrep + bat for display
- Server-based architecture

**Tech stack:** Python, vector embeddings, ripgrep

**What Parseltongue can learn:**
- Local-first philosophy
- Clean CLI UX

**What Parseltongue does better:**
- Rust compiler integration (not just text)
- Call graph analysis
- Blast radius

---

### 2. Probe (469★)
**URL:** https://github.com/probelabs/probe

**What it does:**
- Code + markdown context engine
- Tree-sitter AST parsing
- Elasticsearch-style queries
- "One call captures what takes other tools 10+ loops"

**Tech stack:** Node.js/TypeScript, tree-sitter, ripgrep

**Key insight:**
> "Today's AI coding tools use a caveman approach: grep some files, read random lines, hope for the best. It works on toy projects. It falls apart on real codebases."

**What Parseltongue can learn:**
- Focus on "reading and reasoning" not just search
- AST-aware extraction

**What Parseltongue does better:**
- Rust-specific semantic depth (rust-analyzer)
- Graph algorithms
- Blast radius analysis

---

### 3. Semantic Code Search (389★)
**URL:** https://github.com/sturdy-dev/semantic-code-search

**What it does:**
- Natural language code search
- Neural network embeddings
- Local processing (no cloud)

**Tech stack:** Python, embeddings

**Limitation:** Text-based, no structural understanding

---

### 4. Codeqai (496★)
**URL:** https://github.com/fynnfluegge/codeqai

**What it does:**
- Local-first semantic search + chat
- Fine-tuning dataset generation
- Alpaca/Conversational format support

**Tech stack:** Python, embeddings

---

### 5. Octocode (236★)
**URL:** https://github.com/Muvon/octocode

**What it does:**
- Semantic code searcher
- Codebase utility

---

## Tier 2: Code Intelligence / Knowledge Graph

### 6. Axon (416★) ⭐ HIGHLY RELEVANT
**URL:** https://github.com/harshkedia177/axon

**What it does:**
> "Indexes any codebase into a structural knowledge graph — every dependency, call chain, cluster, and execution flow — then exposes it through smart MCP tools."

**Features:**
- **Hybrid Search**: BM25 + Vector + Fuzzy with Reciprocal Rank Fusion
- **Impact Analysis**: Depth-grouped (will break / may break / review)
- **Dead Code Detection**: Multi-pass with framework awareness
- **Execution Flow Tracing**: Entry point detection + BFS tracing
- **Community Detection**: Leiden algorithm for clusters
- **Change Coupling**: Git history analysis (6 months)

**Tech stack:** Python, KuzuDB, igraph, leidenalg, embeddings

**Key insight:**
> "Your AI agent edits UserService.validate(). It doesn't know that 47 functions depend on that return type, 3 execution flows pass through it, and payment_handler.py changes alongside it 80% of the time."

**What Parseltongue can learn:**
- Impact analysis with depth grouping
- Community detection (Leiden)
- Git coupling analysis
- Dead code detection heuristics

**What Parseltongue does better:**
- Rust-specific via rust-analyzer (not tree-sitter)
- MIR-level control flow
- Type inference accuracy

---

### 7. Flowistry (3028★) ⭐ MOST RELEVANT FOR RUST
**URL:** https://github.com/willcrichton/flowistry

**What it does:**
> "Flowistry is a tool that analyzes the information flow of Rust programs. Flowistry understands whether it's possible for one piece of code to affect another."

**Features:**
- Information flow analysis
- Focus mode (fade irrelevant code)
- IDE integration (VSCode)
- rustc plugin architecture

**Tech stack:** Rust, rustc_private, rustc_plugin

**Key insight:**
> "When the user clicks a given variable or expression, Flowistry fades out all code that does not influence that code, and is not influenced by that code."

**Academic paper:** "Modular Information Flow through Ownership" (PLDI 2022)

**What Parseltongue can learn:**
- rustc plugin integration pattern
- Information flow analysis
- Ownership-based reasoning

**What Parseltongue does better:**
- LLM context generation (Flowistry is IDE-focused)
- Multi-entity disambiguation
- Blast radius for change planning

---

### 8. SCIP (526★)
**URL:** https://github.com/sourcegraph/scip

**What it does:**
- Code Intelligence Protocol
- Language-agnostic indexing format
- Powers Sourcegraph

**What Parseltongue can learn:**
- Standard indexing format (maybe adopt?)

**Decision:** SKIP - too generic, not Rust-specific

---

## Tier 3: MCP Servers for Code

### 9. Smart Coding MCP (188★)
**URL:** https://github.com/omar-haris/smart-coding-mcp

**What it does:**
- MCP server for semantic code search
- Tree-sitter parsing
- Matryoshka Representation Learning (MRL)
- Local AI models

**Tech stack:** TypeScript, tree-sitter, embeddings

---

### 10. Sourcerer MCP (107★)
**URL:** https://github.com/st3v3nmw/sourcerer-mcp

**What it does:**
- MCP server for semantic search
- Tree-sitter chunking
- Chromem-go vector DB

**Tech stack:** Go, tree-sitter, OpenAI embeddings

**Chunk ID format:** `file.ext::Type::method`

---

### 11. CocoIndex Code (240★) ⭐ RELEVANT
**URL:** https://github.com/cocoindex-io/cocoindex-code

**What it does:**
- Lightweight MCP for code
- AST-based understanding
- 70% token saving
- Ultra performant (Rust engine)

**Tech stack:** Python + Rust backend (CocoIndex)

**What Parseltongue can learn:**
- Performance-first architecture
- AST-based chunking

---

### 12. CocoIndex (6252★)
**URL:** https://github.com/cocoindex-io/cocoindex

**What it does:**
- Data transformation framework for AI
- Incremental processing
- Data lineage

**Tech stack:** Rust + Python

---

## Tier 4: Code Context Generation

### 13. Repomix (22,161★)
**URL:** https://github.com/yamadashy/repomix

**What it does:**
- Packs entire repository into single AI-friendly file
- For Claude, ChatGPT, Gemini, etc.

**Limitation:** Dumps everything, no intelligence

---

### 14. Codebase Context Spec (137★)
**URL:** https://github.com/Agentic-Insights/codebase-context-spec

**What it does:**
- Proposal for tool-agnostic context system
- `.context` directory with `index.md`

---

### 15. Cody (Sourcegraph) (3792★)
**URL:** https://github.com/sourcegraph/cody-public-snapshot

**What it does:**
- AI code assistant
- Advanced search + codebase context

**What Parseltongue can learn:**
- Enterprise-scale architecture

**Decision:** Reference implementation, not direct competitor

---

## Tier 5: Rust Compiler Tools

### 16. rustc_plugin (164★) ⭐ INFRASTRUCTURE
**URL:** https://github.com/cognitive-engineering-lab/rustc_plugin

**What it does:**
- Framework for writing rustc plugins
- Powers Flowistry

**Tech stack:** Rust, rustc_private

**What Parseltongue MUST use:**
- This is the foundation for rustc integration
- Same author as Flowistry

---

## Competitive Matrix

| Feature | SeaGOAT | Probe | Axon | Flowistry | CocoIndex | **Parseltongue** |
|---------|---------|-------|------|-----------|-----------|------------------|
| Semantic Search | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ |
| AST Parsing | ❌ | ✅ | ✅ | ❌ | ✅ | ✅ |
| Rust Compiler | ❌ | ❌ | ❌ | ✅ | ❌ | **✅** |
| Call Graph | ❌ | ❌ | ✅ | Partial | ❌ | **✅** |
| Blast Radius | ❌ | ❌ | ✅ | ❌ | ❌ | **✅** |
| Community Detection | ❌ | ❌ | ✅ | ❌ | ❌ | **✅** |
| Type Inference | ❌ | ❌ | ❌ | ✅ | ❌ | **✅** |
| MIR Access | ❌ | ❌ | ❌ | ✅ | ❌ | **✅** |
| Disambiguation UX | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Option Cards | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |

---

## What's Already Solved

1. **Semantic search via embeddings** - Solved by SeaGOAT, Probe, others
2. **AST-based chunking** - Solved by tree-sitter ecosystem
3. **MCP integration** - Solved by CocoIndex, Sourcerer, others
4. **Basic call graph** - Solved by Axon (but text-based)
5. **Community detection** - Solved by Axon (Leiden algorithm)

---

## What's NOT Solved (Parseltongue Opportunity)

### 1. Rust Compiler Integration
- **No one** is using ra_ap_* crates for entity extraction
- **No one** is using MIR for control flow
- **No one** is using rustc_middle for type inference
- **Only Flowistry** uses rustc_private, but IDE-focused

### 2. True Blast Radius
- Axon has impact analysis, but text-based
- No one has MIR-level change impact
- No one has type-based breaking change detection

### 3. Disambiguation UX
- Everyone dumps search results
- No one has "option cards with confidence scores"
- No one has "why this card exists" explanations

### 4. Rust-Specific Intelligence
- No one understands ownership/borrowing
- No one understands trait bounds
- No one understands lifetime relationships

### 5. LLM Context Compression
- Everyone dumps code
- No one has token-budgeted context packages
- No one has "signature + type flow + call slice" compression

---

## Parseltongue Differentiation Strategy

### What to ADOPT (don't reinvent)
1. **Hybrid search** (BM25 + Vector) from Axon
2. **Community detection** (Leiden) from Axon
3. **rustc_plugin** from Flowistry for infra
4. **MCP integration** pattern from CocoIndex

### What to INNOVATE on
1. **ra_ap_* integration** for entity extraction
2. **MIR-based call graph** for true dependencies
3. **Type-based blast radius** for Rust
4. **Disambiguation UX** with option cards
5. **Token-budgeted context** for LLM efficiency

### What to SKIP
1. Multi-language support (Rust only)
2. Cloud embeddings (local first)
3. IDE plugins (CLI first)
4. Generic indexing (compiler first)

---

## Recommended Next Steps

1. **Study Flowistry architecture** - rustc_plugin integration
2. **Study Axon pipeline** - Graph construction + algorithms
3. **Study Probe UX** - Agent integration pattern
4. **Prototype ra_ap_hir extraction** - Entity key generation
5. **Design option card format** - UX contract

---

## Key Repos to Clone

```bash
# Must study
git clone https://github.com/willcrichton/flowistry
git clone https://github.com/harshkedia177/axon
git clone https://github.com/cognitive-engineering-lab/rustc_plugin

# Reference
git clone https://github.com/probelabs/probe
git clone https://github.com/cocoindex-io/cocoindex
git clone https://github.com/cocoindex-io/cocoindex-code
```

---

*Document Version: 1.0*
*Created: 2026-03-02*
*Status: Research Complete*
