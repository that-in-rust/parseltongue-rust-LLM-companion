---
name: context-engineering-agents
description: Design and operate LLM agents using 7 context management patterns for 2026. Covers filesystem-first primitives, multi-layer action spaces, progressive disclosure, context offloading, prompt caching, sub-agent isolation, and continual learning from trajectories. Use this skill when building, configuring, or reasoning about agent architectures.
---

# Context Engineering for Agents

## Overview

Use this skill when designing agent workflows, configuring agent systems, or reasoning about how to structure LLM-driven automation. The core thesis:

> **Effective agent design largely boils down to context management.**
> -- Synthesized from Karpathy, Manus, Claude Code, and Amp Code production deployments (2025-2026)

Context windows are finite. As conversations extend, early context is lost. Every new token depletes the model's "attention budget." Agent task length doubles every 7 months (METR), but models get worse as context grows. These 7 patterns address that tension.

## When To Use

Use this skill when:
- Designing a new agent workflow or multi-agent system.
- Configuring tools, skills, or action spaces for an LLM agent.
- Debugging agent failures caused by context loss, hallucination, or inconsistency.
- Deciding how to structure long-running autonomous tasks.
- Optimizing agent cost (prompt caching, token budgets).

Do not use this skill for writing Rust code (use `idiomatic-rust-coding`) or specifications (use `ai-native-spec-writing`).

## The 7 Patterns

### Pattern 1: Give Agents A Computer

Agents live on the filesystem. Shell, CLIs, and agent-written scripts replace custom tool APIs.

**Why it works:**

| Capability | Traditional | Agent Computer |
|------------|-------------|----------------|
| Persistence | Lost in context | Saved to filesystem |
| Composition | Pre-built tools | Chain via shell |
| Domain | Generic | Use existing CLIs |
| Flexibility | Fixed interface | Write new code |

**Key insight** (rauchg): "The primary lesson from actually successful agents is the return to Unix fundamentals: file systems, shells, processes and CLIs. Bash is all you need."

**Application:**
- Give agents read/write access to the filesystem.
- Prefer shell commands over custom tool implementations.
- Let agents write and execute their own scripts for novel tasks.
- Use existing CLIs (cargo, git, curl, rg) as the primary action surface.

### Pattern 2: Multi-Layer Action Space

Keep the tool count small. Push complex actions down to shell and agent-written code.

**The hierarchy:**

| Layer | What | Example |
|-------|------|---------|
| Layer 1: Tool Calling | ~10 atomic tools with constrained decoding | read_file, write_file, bash |
| Layer 2: Shell | bash tool executing commands | `cargo test --quiet` |
| Layer 3: Computer | Utilities, CLIs, agent-written code | Custom scripts chaining multiple operations |

**Production numbers:**

| Agent | Tool Count | Design |
|-------|-----------|--------|
| Claude Code | ~12 | Curated atomic tools |
| Manus | <20 | Hierarchy with bash |
| Amp Code | Few | Curated action space |

**The CodeAct Pattern:** Agents write and execute code to chain actions without processing intermediate results:

```bash
# Instead of 3 separate tool calls, one bash action:
cp src/main.rs src/main.rs.bak
sed -i 's/old/new/g' src/main.rs
cargo test --quiet
```

### Pattern 3: Progressive Disclosure

Do not load all tool definitions upfront. Reveal details on demand.

**Implementations:**
- **Claude Code Skills**: Folders containing SKILL.md files. YAML frontmatter loaded into instructions; full markdown read only if triggered.
- **Cursor Agent**: MCP tool descriptions synced to folder. Agent gets short list, reads full description only when needed.
- **Manus**: List of utilities in instructions. Agent uses `--help` flags to learn details on-demand.

**Application to this repo:**
- Skills in `docs/skills/*/SKILL.md` follow this pattern.
- The `---` frontmatter block (name, description) is the index entry.
- The full markdown body is the on-demand detail.
- Agents should scan frontmatter first, then read full SKILL.md only when the task matches.

### Pattern 4: Offload Context

Use the filesystem as a context extension. When the token budget nears its limit, summarize old context and write it to files.

**Techniques:**
- **Plan files**: Write a plan to disk and periodically read it back to reinforce objectives.
- **Trajectory offload**: Write old tool results to files. Summarize once diminishing returns hit.
- **Steering pattern**: Plan file written and periodically re-read to verify work stays on track.

```bash
# Plan file pattern
cat > /tmp/plan.md << 'EOF'
# Task: Implement feature X

## Steps
1. [ ] Write test for filter_implementation_entities_only
2. [ ] Implement function
3. [ ] Verify performance < 500us
4. [ ] Update documentation

## Progress
- Step 1: Complete
- Step 2: In progress
EOF
```

**When to offload:**
- Conversation exceeds ~50% of context window.
- Agent starts repeating earlier points or contradicting previous decisions.
- Multiple large tool outputs have been consumed.

### Pattern 5: Cache Context

Prompt caching is the most important cost metric for production agents.

Without caching, agents become cost-prohibitive. A higher-capacity model with caching can be cheaper than a lower-cost model without it.

**Cache optimization priority:**

| Strategy | Impact |
|----------|--------|
| Static system prompt | High -- rarely changes |
| Tool definitions | High -- stable across turns |
| Agent instructions | High -- stable per session |
| Recent conversation | Low -- changes frequently |

**Application:**
- Keep system prompts and tool definitions stable across turns.
- Place volatile content (user messages, tool results) at the end of the context.
- Structure prompts so the cacheable prefix is as long as possible.

### Pattern 6: Isolate Context

Use sub-agents with isolated context for parallel work. Each sub-agent gets only the context it needs.

**Use cases:**
- **Parallel code review**: Sub-agents independently check different issues.
- **Map-reduce**: Lint rules, migrations, any embarrassingly parallel task.
- **Long-running tasks**: The Ralph Loop pattern.

**The Ralph Loop** (GeoffreyHuntley):

```
1. Initializer agent creates a plan file (git-backed).
2. Sub-agents each tackle one task from the plan.
3. Each sub-agent commits progress back to the plan.
4. Stop hook / verify agent checks work when all tasks are done.
5. If verification fails, loop back to step 2.
```

**Key principle:** Main agent holds full task context. Sub-agents get only the slice they need. Results flow back as summaries, not raw context.

### Pattern 7: Evolve Context

Learn from agent trajectories over time. Update context (not model weights) with learnings.

**The GEPA Pattern:**
1. **G**enerate trajectories (run agent sessions).
2. **E**valuate outcomes (success/failure scoring).
3. **P**ropose variants (reflect on failures, suggest prompt changes).
4. **A**dopt winners (update prompts, skills, memory).

**The Diary Pattern** (RLanceMartin):
- Reflect over session logs.
- Distill preferences and feedback from actual use.
- Update memory files (CLAUDE.md, skills, etc.).

```
~/.claude/diary/
  2025-12-01.md  # What worked, what didn't
  2025-12-02.md  # Patterns discovered
  2025-12-03.md  # Skill distillations
```

**Application:**
- After each significant session, document what worked and what failed.
- Update CLAUDE.md / AGENTS.md with learned patterns.
- Create new skills when a pattern is used 3+ times.
- Remove or update skills that consistently produce poor results.

## Pattern Summary

| Pattern | Core Idea | Production Example |
|---------|-----------|-------------------|
| Give Agents A Computer | Filesystem + shell primitives | Claude Code, Manus |
| Multi-Layer Action Space | Few atomic tools, push to bash | Manus (<20 tools) |
| Progressive Disclosure | Reveal details on-demand | Claude Skills, Cursor |
| Offload Context | Filesystem extends context | Manus, Cursor |
| Cache Context | Prompt caching = viability | All production agents |
| Isolate Context | Sub-agents for parallel work | Claude Code review |
| Evolve Context | Learn from trajectories | Claude diary, GEPA |

## LLM Response Contract

When designing agent systems with this skill, deliver:
- Explicit context budget analysis (what fits, what gets offloaded).
- Tool count and action space hierarchy.
- Sub-agent decomposition with context isolation boundaries.
- Caching strategy for the prompt structure.
- Evolution plan: how learnings feed back into the system.

## Anti-Patterns to Avoid

- Loading all tool definitions upfront (context bloat).
- Monolithic context with no sub-agent isolation.
- Unbounded conversations without offloading or summarization.
- Ignoring prompt caching (cost explosion).
- Static prompts that never incorporate session learnings.
- Fire-and-forget sub-agents with no verification loop.

## Resources

- Source: `agents-used-202512/notes01-agent.md` (Part IX: Agent Design Patterns for 2026)
- Related skill: `ai-native-spec-writing` (for requirements and naming)
- Related skill: `idiomatic-rust-coding` (for implementation patterns)
- Related skill: `dependency-map-cli-tools` (example of Pattern 1 in practice)
