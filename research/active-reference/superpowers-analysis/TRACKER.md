## Exploration State: 2026-03-02

### Current Phase: Synthesis (Complete)

### Areas Explored:
- **Architecture**: COMPLETE - Full structure mapped
- **Skills System**: COMPLETE - 14 skills analyzed
- **Hooks System**: COMPLETE - SessionStart hook mechanism understood
- **Core Workflow**: COMPLETE - TDD, subagent-driven development documented
- **Cross-platform Support**: COMPLETE - Claude Code, Cursor, Codex, OpenCode analyzed

### Files Analyzed:
- `README.md`: Main documentation - Overview of workflow and installation
- `RELEASE-NOTES.md`: Version history - Evolution of the framework
- `skills/brainstorming/SKILL.md`: Design refinement skill - Socratic dialogue process
- `skills/writing-plans/SKILL.md`: Implementation planning - Bite-sized task creation
- `skills/test-driven-development/SKILL.md`: TDD enforcement - RED-GREEN-REFACTOR cycle
- `skills/subagent-driven-development/SKILL.md`: Execution via subagents - Two-stage review
- `skills/executing-plans/SKILL.md`: Batch execution - Checkpoint-based workflow
- `skills/using-superpowers/SKILL.md`: Meta-skill - How to find and use skills
- `skills/writing-skills/SKILL.md`: Skill creation - TDD for documentation
- `skills/systematic-debugging/SKILL.md`: Debugging methodology - 4-phase root cause
- `skills/using-git-worktrees/SKILL.md`: Isolation - Git worktree management
- `skills/requesting-code-review/SKILL.md`: Review workflow - Code reviewer dispatch
- `skills/finishing-a-development-branch/SKILL.md`: Completion - Merge/PR/cleanup
- `lib/skills-core.js`: Core utilities - Skill discovery and resolution
- `hooks/hooks.json`: Hook configuration - SessionStart registration
- `hooks/session-start`: Bootstrap script - Context injection on startup
- `agents/code-reviewer.md`: Reviewer agent - Code review prompt template
- `commands/brainstorm.md`: Slash command - Invokes brainstorming skill
- `commands/write-plan.md`: Slash command - Invokes writing-plans skill
- `commands/execute-plan.md`: Slash command - Invokes executing-plans skill
- `.claude-plugin/plugin.json`: Claude Code manifest - Plugin metadata
- `.claude-plugin/marketplace.json`: Marketplace config - Dev marketplace
- `.cursor-plugin/plugin.json`: Cursor manifest - Skills/agents/hooks paths
- `.codex/INSTALL.md`: Codex installation - Clone + symlink approach
- `.opencode/INSTALL.md`: OpenCode installation - Plugin + skills symlinks
- `docs/testing.md`: Testing guide - Integration test methodology

### Current Focus:
Exploration complete. All tasks finished.

### Next Steps:
None - exploration complete.

### Key Insights:
- **Skills are YAML-frontmatter + markdown** with specific structure for discoverability
- **Hooks inject context at session start** via SessionStart hook that loads using-superpowers
- **Two execution modes**: subagent-driven (same session) vs executing-plans (parallel session)
- **TDD is non-negotiable** - Iron Law enforced throughout all implementation skills
- **Skill creation uses TDD** - Write test scenarios first, then create skill
- **CSO (Claude Search Optimization)** critical for skill discovery
- **Personal skills shadow superpowers skills** - User overrides built-in
- **Plugin architecture differs by platform** - Marketplaces vs manual symlinks

---

# Superpowers Framework - Comprehensive Summary

## Overview
Superpowers is an agentic software development workflow framework built on composable "skills" that automatically trigger to enforce disciplined development practices (TDD, systematic debugging, design-first approach).

## Architecture

```
superpowers/
├── skills/              # 14 composable skills (SKILL.md files)
├── hooks/               # SessionStart hook for context injection
├── lib/                 # skills-core.js (discovery/resolution utilities)
├── commands/            # Slash commands (/brainstorm, /write-plan, /execute-plan)
├── agents/              # Agent definitions (code-reviewer.md)
├── .claude-plugin/      # Claude Code plugin manifest
├── .cursor-plugin/      # Cursor plugin manifest
├── .codex/              # Codex installation instructions
├── .opencode/           # OpenCode installation instructions
└── tests/               # Integration tests
```

## Skills System

### Skill Format
```yaml
---
name: skill-name-with-hyphens
description: Use when [specific triggering conditions]
---

# Skill Name
## Overview (1-2 sentences, core principle)
## When to Use (flowchart if non-obvious)
## The Process (numbered steps)
## Red Flags (what to avoid)
## Common Rationalizations (table)
## Quick Reference (table)
```

### Key Skills (14 total)
| Skill | Purpose |
|-------|---------|
| brainstorming | Design refinement through Socratic dialogue |
| writing-plans | Create bite-sized implementation tasks |
| subagent-driven-development | Execute via subagents with two-stage review |
| executing-plans | Batch execution with checkpoints |
| test-driven-development | RED-GREEN-REFACTOR enforcement |
| systematic-debugging | 4-phase root cause investigation |
| using-git-worktrees | Isolated workspace creation |
| requesting-code-review | Dispatch code reviewer subagent |
| finishing-a-development-branch | Merge/PR/cleanup workflow |
| writing-skills | Create new skills using TDD |
| using-superpowers | Meta-skill for skill discovery |

### Skill Discovery Flow
1. SessionStart hook injects `using-superpowers` content
2. Agent checks for applicable skills before ANY action
3. Skills invoke via Skill tool (Claude Code) or platform equivalent
4. Personal skills (~/.claude/skills) shadow superpowers skills

## Hooks System

### SessionStart Hook
- **Trigger**: startup, resume, clear, compact events
- **Action**: Injects `using-superpowers` skill content as context
- **Script**: bash script that escapes content for JSON injection
- **Critical**: Runs synchronously (v4.3.0+) to ensure context is available

### Hook Configuration (hooks.json)
```json
{
  "hooks": {
    "SessionStart": [{
      "matcher": "startup|resume|clear|compact",
      "hooks": [{ "type": "command", "command": "...run-hook.cmd session-start" }]
    }]
  }
}
```

## Core Workflow

### Complete Development Flow
```
1. BRAINSTORMING → Explore context → Ask questions → Present design → Get approval
2. WORKTREES → Create isolated workspace → Run setup → Verify clean baseline
3. WRITING-PLANS → Break into bite-sized tasks (2-5 min each) → Save plan
4. SUBAGENT-DRIVEN-DEV or EXECUTING-PLANS → Implement with reviews
5. TDD → RED (write failing test) → GREEN (minimal code) → REFACTOR
6. CODE-REVIEW → Spec compliance → Code quality → Fix issues
7. FINISHING-BRANCH → Verify tests → Present options (merge/PR/keep/discard)
```

### Two Execution Modes
| Mode | Session | Review | Best For |
|------|---------|--------|----------|
| subagent-driven-development | Same session | Per-task two-stage | Fast iteration |
| executing-plans | Parallel session | Batch checkpoints | Long-running work |

### Two-Stage Review Process
1. **Spec Compliance Review**: Did implementer build exactly what was requested?
2. **Code Quality Review**: Is the implementation well-built?

## TDD Enforcement (The Iron Law)
```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST
```
- Write test → Watch it fail → Write minimal code → Watch it pass → Refactor
- Code before test? Delete it and start over
- No exceptions, no rationalizations

## Cross-Platform Support

| Platform | Installation | Discovery |
|----------|-------------|-----------|
| Claude Code | Plugin marketplace | Native Skill tool |
| Cursor | Plugin marketplace | Skills directory |
| Codex | Clone + symlink | ~/.agents/skills/ |
| OpenCode | Clone + symlinks | Native skill tool |

## Creating New Skills

### Process (TDD for Documentation)
1. **RED**: Run pressure scenario WITHOUT skill → Document failures/rationalizations
2. **GREEN**: Write minimal skill addressing those specific issues
3. **REFACTOR**: Close loopholes, add rationalization tables, test again

### CSO (Claude Search Optimization)
- Description = triggering conditions only (NOT workflow summary)
- Use keywords: error messages, symptoms, tools
- Name with verbs/hyphens: `condition-based-waiting`
- Keep frequently-loaded skills under 200 words
