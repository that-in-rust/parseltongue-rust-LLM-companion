# Codebase Wisdom 101
Constantly do cargo clean etc so that unnecessary files do not messs up your context or space

# Technical Design101: TDD-First Architecture Principles

Test-First Development: I should be writing tests FIRST, following the STUB → RED → GREEN → REFACTOR cycle

# CRITICAL: FOUR-WORD NAMING CONVENTION (LLM Optimization)

**ALL function names: EXACTLY 4 words** (underscores separate)
**ALL crate names: EXACTLY 4 words** (hyphens separate)
**ALL folder names: EXACTLY 4 words** (hyphens separate)
**ALL commands: EXACTLY 4 words** (hyphens separate)
**ALL CLI invocations: Use FULL 4-word crate name** (not shortened versions)

**Why**: LLMs parse by tokenizing. 4 words = optimal semantic density for understanding and recall.

**Pattern**: `verb_constraint_target_qualifier()`
- Verb: `filter`, `render`, `detect`, `save`, `create`, `process`, `stream`
- Constraint: `implementation`, `box_with_title`, `visualization_output`, `folder_to`
- Target: `entities`, `unicode`, `file`, `database`, `cozodb`
- Qualifier: `only`, `to`, `in`, `from`, `with`, `streamer`, `writer`, `terminal`

**Examples**:

**Functions** (underscores):
```rust
✅ filter_implementation_entities_only()
✅ render_box_with_title_unicode()
✅ save_visualization_output_to_file()

❌ filter_entities()                    // Too short (2)
❌ detect_cycles_in_dependency_graph()  // Too long (5)
```

**Crates/Commands** (hyphens):
```bash
✅ pt01-folder-to-cozodb-streamer      // 4 words: folder, to, cozodb, streamer
✅ pt02-llm-cozodb-to-context          // 4 words: llm, cozodb, to, context
✅ pt07-visual-analytics-terminal      // 4 words: visual, analytics, terminal, (implied: tool)

❌ pt01                                 // Too short (1)
❌ pt02-level00                         // Too short (2)
❌ pt07 entity-count                    // Too short (3)
```

**CLI Usage - ALWAYS use full crate names**:
```bash
# ✅ CORRECT - Full 4-word names
parseltongue pt01-folder-to-cozodb-streamer . --db "rocksdb:db.db"
parseltongue pt02-llm-cozodb-to-context level00 --where-clause "ALL" --output edges.json --db "rocksdb:db.db"
parseltongue pt07-visual-analytics-terminal entity-count --db "rocksdb:db.db"

# ❌ WRONG - Shortened versions (harder for LLMs to parse)
parseltongue pt01 . --db "rocksdb:db.db"
parseltongue pt02-level00 --where-clause "ALL" --output edges.json --db "rocksdb:db.db"
parseltongue pt07 entity-count --db "rocksdb:db.db"
```

**Why This Matters for LLMs**:
1. **Tokenization**: LLMs break text into tokens. 4-word patterns create consistent token boundaries
2. **Semantic Density**: Each word carries meaning (verb, what, where, how)
3. **Recall**: LLMs remember structured patterns better than arbitrary abbreviations
4. **Disambiguation**: "pt01" is ambiguous, "folder-to-cozodb-streamer" is self-documenting

**Make this a ritual** - check every function/folder/command name before committing.

# Product thinking for us
Think like Shreyas Doshi  - the famous product leader - his minimalism - user journeys mindset

## The Essence: Executable Specifications Drive Everything

Exectuable Specifications is the concept  - stick to it 


**Core Truth**: Traditional user stories fail LLMs because they're designed for human conversation. LLMs need executable blueprints, not ambiguous narratives.

**The Solution**: Transform all specifications into formal, testable contracts with preconditions, postconditions, and error conditions. Every claim must be validated by automated tests.

**Why This Matters**: Eliminates the #1 cause of LLM hallucination - ambiguous requirements that lead to incorrect implementations.

## The Non-Negotiables: 8 Architectural Principles

These principles are derived from the Parseltongue AIM Daemon design process and prevent the most common architectural failures in Rust systems:

### 1. Executable Specifications Over Narratives
**Contract-driven development with measurable outcomes**

### 2. Layered Rust Architecture (L1→L2→L3)
**Clear separation: Core → Std → External dependencies**

### 3. Dependency Injection for Testability
**Every component depends on traits, not concrete types**

### 4. RAII Resource Management
**All resources automatically managed with Drop implementations**

### 5. Performance Claims Must Be Test-Validated
**Every performance assertion backed by automated tests**

### 6. Structured Error Handling
**thiserror for libraries, anyhow for applications**

### 7. Complex Domain Model Support
**Handle real-world complexity, not simplified examples**

### 8. Concurrency Model Validation
**Thread safety validated with stress tests**

### 9. MVP-First Rigor (New Pattern)
**Proven architectures over theoretical abstractions**

## IMPORTANT FOR VISUALS AND DIAGRAMS

ALL DIAGRAMS WILL BE IN MERMAID ONLY TO ENSURE EASE WITH GITHUB - DO NOT SKIP THAT - use MermaidSteering.md file for that - it must be somewhere - use it
