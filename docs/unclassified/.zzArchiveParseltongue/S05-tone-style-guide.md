# Tone & Style Guide

This document defines the voice and communication style for Parseltongue project, based on @amuldotexe's low-drama, understated approach.

## CRITICAL: FOUR-WORD NAMING CONVENTION

**ALL function names: EXACTLY 4 words** (underscores: `_`)
**ALL crate/folder/command names: EXACTLY 4 words** (hyphens: `-`)

Pattern: `verb_constraint_target_qualifier()`

**Why**: 4 words = optimal LLM tokenization. This is non-negotiable.

Examples:
```rust
✅ parse_code_to_graph_structure()
✅ export_analysis_results_to_json()
✅ pt01-folder-to-cozodb-streamer

❌ parse()               // Too short (1)
❌ parse_code()          // Too short (2)
❌ parse_code_and_export_to_database()  // Too long (5)
```

**Pre-commit ritual**: Count words in every name. Fix before committing.

## Core Principles

### Low Drama Style
- **Understated confidence**: State facts without superlatives
- **Direct language**: Say what you mean, simply and clearly
- **No hype language**: Avoid "revolutionary", "game-changing", "unprecedented"
- **Factual over emotional**: Let results speak for themselves

### Voice Characteristics
- **Calm and measured**: No exclamation points or enthusiastic language
- **Slightly casual but professional**: Like talking to a respected colleague
- **Efficient**: Get to the point without unnecessary words
- **Confident but humble**: Know the tool's value without bragging

## Writing Guidelines

### Do ✅
- Use simple, direct sentences
- State facts and metrics plainly
- Keep section headers clean and emoji-light
- Be specific about capabilities
- Use "shows" instead of "reveals"
- Use "generates" instead of "magically creates"
- Focus on what the tool does, not how amazing it is

### Don't ❌
- Use superlatives ("best", "fastest", "revolutionary")
- Add exclamation points for enthusiasm
- Use corporate buzzwords ("synergy", "paradigm shift")
- Make exaggerated comparisons ("125x faster", "Enterprise scale")
- Use dramatic language ("record time", "breathtaking")
- Force excitement ("That's it!", "The vibe")

## Examples

### Before (High Drama)
```
🏆 **Real-World Showcase: Tokio Codebase Analysis**

Parseltongue analyzed the complete Tokio async runtime in RECORD TIME:
- **125x faster** than target
- **Enterprise scale** processing
- **Revolutionary** performance guarantees
```

### After (Low Drama)
```
## Real-World Example: Tokio Codebase

Parseltongue analyzed the Tokio async runtime:
- 0.24s ingestion time
- 2,576 entities found
- 1μs query performance
```

### Before (Hype Language)
```
🚀 **Core Superpowers**
- **Instant** architectural understanding
- **Beautiful** diagrams generated automatically
- **Perfect** context for AI assistance
```

### After (Factual)
```
## What You Get
- Parse code in seconds
- Generate architecture diagrams
- Export context for AI tools
```

## Technical Documentation

### README Style
- Start with simple, direct description
- Use clean section headers (minimal emojis)
- Present metrics as facts, not achievements
- Keep examples focused and practical

### Commit Messages
- Describe what changed and why
- Avoid hype or superlatives
- Focus on technical impact
- Keep it concise and factual

### Error Messages
- Be clear and helpful
- No blame or drama
- Suggest next steps when possible
- Keep it technical and precise

## Communication Patterns

### Describing Performance
❌ "Blazing fast queries in microseconds!"
✅ "Queries: < 50μs"

❌ "Crushes large codebases with ease"
✅ "Processes 150K+ lines of code"

### Feature Announcements
❌ "Excited to announce our revolutionary new feature!"
✅ "Added call graph analysis"

### Problem Solving
❌ "Struggling with complex codebases? We've got the solution!"
✅ "Finding your way around a new Rust codebase takes time. Answering questions about it should be fast."

## Tone Maintenance

When writing or reviewing content, ask:
1. Is this stated as fact rather than hype?
2. Could this be said more simply?
3. Are there unnecessary superlatives or dramatic words?
4. Does this sound like something @amuldotexe would write?
5. Is the focus on utility rather than excitement?

Remember: Let the results speak for themselves. The tool's capabilities should be impressive on their own, without needing dramatic language to sell them.