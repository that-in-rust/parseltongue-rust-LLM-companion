# Parseltongue Skills Library

This directory contains reusable skills for AI-native software development using LLM coding companions.

## Skills Overview

### 1. **ai-native-spec-writing** 
**When to use**: Writing requirements, acceptance criteria, and design specs before implementation.

Covers:
- Four-Word Naming Convention (4WNC) - 67% faster development
- WHEN/THEN/SHALL executable specifications  
- TDD-first specification cycle
- Contract-driven design
- One-feature-per-version release philosophy
- Pre-commit checklists

Source: `agents-used-202512/notes01-agent.md` (Parts I-VIII, X)

---

### 2. **idiomatic-rust-coding**
**When to use**: Implementing Rust code following best practices and production patterns.

Covers:
- Layered architecture (L1 Core / L2 Standard / L3 External)
- 184 curated idioms organized by category
- Error handling strategies (thiserror vs anyhow)
- Concurrency patterns and async safety
- Performance optimization techniques
- Testing strategies (property-based, fuzzing, snapshot testing)
- Macro hygiene and proc-macro best practices
- CI quality gates and supply-chain security

Companion: `references/idiom-catalog.md` - complete index of 184 idioms

Source: `agents-used-202512/rust-coder-01.md` (Sections A.1-A.184)

---

### 3. **context-engineering-agents**
**When to use**: Designing and architecting autonomous LLM agent systems.

Covers the 7 context management patterns for 2026:
1. **Give Agents A Computer** - Filesystem + shell primitives over custom APIs
2. **Multi-Layer Action Space** - Tool hierarchy (atomic → bash → scripts)
3. **Progressive Disclosure** - Load details on-demand, index upfront
4. **Offload Context** - Use filesystem for persistent storage and summarization
5. **Cache Context** - Prompt caching for cost optimization
6. **Isolate Context** - Sub-agents for parallel/independent work (Ralph Loop)
7. **Evolve Context** - Continual learning via trajectory reflection (GEPA)

Source: `agents-used-202512/notes01-agent.md` (Part IX + agent design patterns)

---

## How To Use These Skills

Each skill is self-contained with:
- **YAML frontmatter** for metadata and trigger keywords
- **Overview section** explaining the core thesis
- **When To Use** for disambiguation
- **Detailed patterns** with code examples
- **Anti-patterns** to avoid
- **Resources section** with source attribution

### Skill Triggers

Skills are loaded progressively. Include the skill name in your prompt when you need it:

```
Use the ai-native-spec-writing skill to write requirements for...
Use the idiomatic-rust-coding skill to implement...
Use the context-engineering-agents skill to design...
```

### Companion References

- **idiomatic-rust-coding**: See `references/idiom-catalog.md` for the complete 184-idiom index
- **dependency-map-cli-tools**: See `references/internet-precedents.md` for CLI tool precedents

---

## Integration Notes

All three skills reference each other appropriately:
- Specs (ai-native-spec-writing) drive implementation (idiomatic-rust-coding)
- Agent design (context-engineering-agents) uses specs and code patterns
- Each skill includes "Related skill:" pointers for cross-navigation

---

## Statistics

| Skill | Lines | Sections | Examples | Diagrams |
|-------|-------|----------|----------|----------|
| ai-native-spec-writing | 203 | 8 | 4 code | - |
| idiomatic-rust-coding | 279 | 12 | 8 code | - |
| idiom-catalog (ref) | 450+ | 30 categories | 184 idioms | - |
| context-engineering-agents | 442 | 10 + future | 4 code | 9 mermaid |

**Total new content**: 1,000+ lines capturing 400+ hours of LLM-assisted development research.

---

## Source Attribution

All skills are derived from two authoritative reference documents:

1. **notes01-agent.md** (1,644 lines)
   - AI-Native Thesis: 7 principles + measured results
   - Four-Word Naming Convention
   - TDD-First methodology
   - Context optimization for token budgets
   - 2026 Agent design patterns

2. **rust-coder-01.md** (3,681 lines)
   - Layered architecture patterns
   - 184 curated Rust idioms
   - Design patterns for production systems
   - Advanced patterns (GATs, unsafe, FFI, macros)
   - CI/quality/testing best practices

Both source files are in `agents-used-202512/` directory.

---

## Next Steps

To use these skills:
1. Read the SKILL.md file for your domain
2. Review the "When To Use" section to confirm it applies
3. Check the companion resources (idiom-catalog.md if using idiomatic-rust-coding)
4. Follow the patterns and examples provided
5. Reference the "Anti-patterns" section to avoid common pitfalls

