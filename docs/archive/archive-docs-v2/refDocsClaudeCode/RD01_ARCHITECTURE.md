# Claude Code: Complete Architectural Deconstruction

## Executive Summary

Claude Code is a sophisticated agentic CLI system built on a **hierarchical multi-agent architecture** with event-driven automation. This document represents a complete atomic deconstruction of the system.

---

## 1. Core Architecture Overview

```
                           ┌─────────────────────────────────────┐
                           │         CLAUDE CODE CLI             │
                           │         (cli.js - 10.9MB)           │
                           └──────────────┬──────────────────────┘
                                          │
              ┌───────────────────────────┼───────────────────────────┐
              │                           │                           │
              ▼                           ▼                           ▼
    ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
    │   TOOL SYSTEM   │         │  AGENT SYSTEM   │         │  HOOK SYSTEM    │
    │                 │         │                 │         │                 │
    │ • Read/Write    │         │ • Task Tool     │         │ • PreToolUse    │
    │ • Edit/Glob     │         │ • Subagents     │         │ • PostToolUse   │
    │ • Grep/Bash     │         │ • Model Select  │         │ • Stop          │
    │ • WebFetch      │         │ • Color Assign  │         │ • SessionStart  │
    │ • TodoWrite     │         │ • Context Pass  │         │ • SubagentStop  │
    │ • MCP Tools     │         │ • Resume        │         │ • 9 Events      │
    └─────────────────┘         └─────────────────┘         └─────────────────┘
              │                           │                           │
              └───────────────────────────┼───────────────────────────┘
                                          │
                           ┌──────────────┴──────────────┐
                           │      PLUGIN SYSTEM          │
                           │                             │
                           │  • Commands (Slash)         │
                           │  • Agents (Custom)          │
                           │  • Skills (Knowledge)       │
                           │  • Hooks (Automation)       │
                           │  • MCP Servers              │
                           └─────────────────────────────┘
```

---

## 2. The Agent System: Deep Dive

### 2.1 Task Tool Architecture

The `Task` tool (internal variable `R8`) is the nucleus of the agent system:

```javascript
// Extracted from cli.js
{
  agentType: "general-purpose",
  whenToUse: "General-purpose agent for researching complex questions,
              searching for code, and executing multi-step tasks.",
  tools: ["*"],  // ALL tools available
  source: "built-in",
  baseDir: "built-in",
  model: "sonnet",
  getSystemPrompt: () => `You are an agent for Claude Code...`
}
```

### 2.2 Built-in Agent Types

| Agent Type | Purpose | Model | Tools |
|------------|---------|-------|-------|
| `general-purpose` | Multi-step research & code tasks | sonnet | All (`*`) |
| `Explore` | Fast codebase exploration | haiku | Read, Grep, Glob |
| `Plan` | Architectural planning | sonnet | Read, Grep, Glob |
| `claude-code-guide` | Documentation lookup | inherit | WebFetch, Read |

### 2.3 Agent Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    AGENT LIFECYCLE                               │
└─────────────────────────────────────────────────────────────────┘

1. INVOCATION
   ┌──────────────────┐
   │ Task Tool Called │ ──► subagent_type, prompt, description
   └────────┬─────────┘     model (optional), resume (optional)
            │
            ▼
2. HOOK: SubagentStart
   ┌──────────────────┐
   │ Hook Event Fires │ ──► agent_id, agent_type passed
   └────────┬─────────┘     Exit 0: stdout shown to subagent
            │
            ▼
3. COLOR ASSIGNMENT
   ┌──────────────────┐
   │ Color Pool:      │
   │ red, blue, green │ ──► Avoids parent's color
   │ yellow, purple,  │     Uses agentColorMap
   │ orange, pink,    │
   │ cyan             │
   └────────┬─────────┘
            │
            ▼
4. CONTEXT INJECTION
   ┌──────────────────┐
   │ System Prompt    │ ──► "You are in a sub-agent context"
   │ Construction     │     "Only complete specific task assigned"
   └────────┬─────────┘
            │
            ▼
5. EXECUTION
   ┌──────────────────┐
   │ Agent Runs       │ ──► Uses allowed tools
   │ Autonomously     │     Single context window
   └────────┬─────────┘     Stateless execution
            │
            ▼
6. HOOK: SubagentStop
   ┌──────────────────┐
   │ Hook Event Fires │ ──► Exit 0: silent
   └────────┬─────────┘     Exit 2: stderr to subagent, continue
            │
            ▼
7. RESULT RETURN
   ┌──────────────────┐
   │ Single Message   │ ──► Not visible to user directly
   │ Returned         │     Parent processes results
   └──────────────────┘
```

### 2.4 Agent Definition Structure

```yaml
# agents/agent-name.md
---
name: agent-identifier        # Required: 3-50 chars, lowercase, hyphens
description: |                # Required: Triggers + 2-4 <example> blocks
  Use this agent when [conditions]. Examples:

  <example>
  Context: [Situation]
  user: "[Request]"
  assistant: "[Response]"
  <commentary>[Why triggers]</commentary>
  </example>

model: inherit|sonnet|opus|haiku  # Required: Model selection
color: blue|cyan|green|yellow|magenta|red  # Required: Visual ID
tools: ["Read", "Write", "Grep"]  # Optional: Tool restrictions
---

You are [role] specializing in [domain].

**Your Core Responsibilities:**
1. [Responsibility]

**Analysis Process:**
1. [Step]

**Output Format:**
[What to return]
```

### 2.5 Model Selection Strategy

```
┌─────────────────────────────────────────────────────────────────┐
│                    MODEL SELECTION MATRIX                        │
└─────────────────────────────────────────────────────────────────┘

┌──────────┬─────────────────────────────────────────────────────┐
│  MODEL   │  USE CASE                                           │
├──────────┼─────────────────────────────────────────────────────┤
│ inherit  │ Default - inherits from parent (RECOMMENDED)        │
├──────────┼─────────────────────────────────────────────────────┤
│ haiku    │ Fast exploration, simple tasks, low cost            │
│          │ Used by: Explore agent                              │
├──────────┼─────────────────────────────────────────────────────┤
│ sonnet   │ Balanced analysis & generation                      │
│          │ Used by: general-purpose, Plan, most custom agents  │
├──────────┼─────────────────────────────────────────────────────┤
│ opus     │ Complex analysis, architectural decisions           │
│          │ Used for: High-stakes reasoning                     │
└──────────┴─────────────────────────────────────────────────────┘

Special Mode: "SonnetPlan" - Haiku uses Sonnet only in plan mode
```

---

## 3. The general-purpose Agent: Complete Implementation

### 3.1 System Prompt (Extracted from cli.js)

```
You are an agent for Claude Code, Anthropic's official CLI for Claude.
Given the user's message, you should use the tools available to complete
the task. Do what has been asked; nothing more, nothing less. When you
complete the task simply respond with a detailed writeup.

Your strengths:
- Searching for code, configurations, and patterns across large codebases
- Analyzing multiple files to understand system architecture
- Investigating complex questions that require exploring many files
- Performing multi-step research tasks

Guidelines:
- For file searches: Use Grep or Glob when you need to search broadly.
  Use Read when you know the specific file path.
- For analysis: Start broad and narrow down. Use multiple search
  strategies if the first doesn't yield results.
- Be thorough: Check multiple locations, consider different naming
  conventions, look for related files.
- NEVER create files unless absolutely necessary. ALWAYS prefer
  editing existing files.
- NEVER proactively create documentation files (*.md) or README files.
- In final response always share relevant file names and code snippets.
  File paths MUST be absolute.
- Avoid using emojis.
```

### 3.2 Key Characteristics

| Property | Value |
|----------|-------|
| Default Model | sonnet |
| Tool Access | All (`*`) |
| Source | built-in |
| Execution | Stateless, single-turn |
| Output | Single detailed writeup |

---

## 4. Hook System: Event-Driven Automation

### 4.1 All 9 Hook Events

```
┌─────────────────────────────────────────────────────────────────┐
│                    HOOK EVENT LIFECYCLE                          │
└─────────────────────────────────────────────────────────────────┘

SESSION LIFECYCLE
─────────────────
SessionStart ──► Session begins (load context, set env)
SessionEnd   ──► Session ends (cleanup, logging)

USER INTERACTION
────────────────
UserPromptSubmit ──► User submits prompt (validation, context)

TOOL EXECUTION
──────────────
PreToolUse  ──► Before tool runs (approve/deny/modify)
PostToolUse ──► After tool completes (feedback, logging)

AGENT LIFECYCLE
───────────────
SubagentStart ──► Subagent begins
SubagentStop  ──► Subagent completing (task validation)

TERMINATION
───────────
Stop ──► Main agent considers stopping (completeness check)

CONTEXT
───────
PreCompact   ──► Before context compaction (preserve info)
Notification ──► Notifications sent (logging, reactions)
```

### 4.2 Hook Configuration Format

```json
// Plugin hooks.json (with wrapper)
{
  "description": "Hook description",
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "prompt",  // or "command"
            "prompt": "Validate file write safety",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

### 4.3 Hook Types Comparison

| Type | Mechanism | Use Case | Timeout |
|------|-----------|----------|---------|
| `prompt` | LLM-driven | Complex reasoning, context-aware | 30s |
| `command` | Bash script | Deterministic, fast checks | 60s |

### 4.4 Hook Input/Output Protocol

```
INPUT (via stdin):
{
  "session_id": "abc123",
  "transcript_path": "/path/to/transcript.txt",
  "cwd": "/current/working/dir",
  "permission_mode": "ask|allow",
  "hook_event_name": "PreToolUse",
  "tool_name": "Write",
  "tool_input": {...}
}

OUTPUT:
{
  "hookSpecificOutput": {
    "permissionDecision": "allow|deny|ask",
    "updatedInput": {"field": "modified"}
  },
  "systemMessage": "Message for Claude"
}

EXIT CODES:
0 - Success (stdout in transcript)
2 - Block (stderr fed back to Claude)
Other - Non-blocking error
```

---

## 5. Plugin System Architecture

### 5.1 Plugin Directory Structure

```
plugin-name/
├── .claude-plugin/
│   └── plugin.json         # Metadata (name, version, author)
├── commands/               # Slash commands
│   └── command.md
├── agents/                 # Autonomous agents
│   └── agent.md
├── skills/                 # Knowledge bundles
│   └── skill-name/
│       ├── SKILL.md        # Core definition
│       ├── references/     # Detailed docs
│       ├── examples/       # Working examples
│       └── scripts/        # Utilities
├── hooks/
│   ├── hooks.json          # Hook configuration
│   └── *.sh, *.py          # Handler scripts
└── .mcp.json               # MCP server config
```

### 5.2 Component Auto-Discovery

Components load automatically from standard directories:
- `commands/` → Slash commands
- `agents/` → Subagents
- `skills/*/SKILL.md` → Agent skills
- `hooks/hooks.json` → Event handlers

### 5.3 Available Plugins (14 Official)

| Plugin | Category | Key Components |
|--------|----------|----------------|
| `feature-dev` | Workflow | 7-phase dev, 3 agents |
| `plugin-dev` | Development | 7 skills, 3 agents |
| `pr-review-toolkit` | Review | 6 parallel agents |
| `code-review` | Review | Confidence scoring |
| `hookify` | Automation | Dynamic hooks |
| `security-guidance` | Security | 9 pattern detection |
| `ralph-wiggum` | Iteration | Self-referential loops |
| `agent-sdk-dev` | SDK | Project scaffolding |
| `commit-commands` | Git | Workflow automation |
| `frontend-design` | Design | Design skill |
| `claude-opus-4-5-migration` | Migration | Model migration |
| `learning-output-style` | UX | Interactive learning |
| `explanatory-output-style` | UX | Educational mode |

---

## 6. Tool System

### 6.1 Core Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| `Read` | Read files | `file_path`, `offset`, `limit` |
| `Write` | Create files | `file_path`, `content` |
| `Edit` | Modify files | `file_path`, `old_string`, `new_string` |
| `Glob` | Pattern match | `pattern`, `path` |
| `Grep` | Content search | `pattern`, `path`, `output_mode` |
| `Bash` | Execute commands | `command`, `timeout` |
| `BashOutput` | Background output | `bash_id` |
| `KillShell` | Kill process | `shell_id` |
| `WebFetch` | Fetch URLs | `url`, `prompt` |
| `WebSearch` | Search web | `query` |
| `TodoWrite` | Task management | `todos[]` |
| `NotebookEdit` | Jupyter cells | `notebook_path`, `cell_id` |
| `Task` | Launch agents | `subagent_type`, `prompt` |
| `AskUserQuestion` | User input | `questions[]` |

### 6.2 Tool Input Schemas (from sdk-tools.d.ts)

```typescript
export interface AgentInput {
  description: string;        // 3-5 word task description
  prompt: string;             // Detailed task instructions
  subagent_type: string;      // Agent type to use
  model?: "sonnet" | "opus" | "haiku";  // Optional model override
  resume?: string;            // Optional agent ID to resume
}
```

---

## 7. Skill System: Progressive Disclosure

### 7.1 Three-Level Architecture

```
LEVEL 1: METADATA (Always Loaded)
├── Skill name, version
├── Trigger phrases
└── ~50-100 words

LEVEL 2: SKILL.md (When Triggered)
├── Core API reference
├── Essential patterns
└── ~1,500-2,000 words

LEVEL 3: RESOURCES (As Needed)
├── references/ - Detailed guides (2,000+ words each)
├── examples/ - Working code
└── scripts/ - Validation tools
```

### 7.2 Skill Definition Format

```yaml
---
name: Skill Name
description: |
  This skill should be used when [trigger phrases].
  Provides comprehensive guidance on [topic].
version: 0.1.0
---

# Skill Content

## Overview
[Essential API reference]

## Patterns
[Core patterns and examples]

## Best Practices
[Guidance and recommendations]
```

---

## 8. Multi-Agent Orchestration Patterns

### 8.1 Sequential Pattern

```
Parent ──► Agent 1 ──► Result ──► Agent 2 ──► Result ──► Final
```

### 8.2 Parallel Pattern (feature-dev)

```
                    ┌──► code-explorer (aspect A) ──┐
Parent ──► Launch ──┼──► code-explorer (aspect B) ──┼──► Consolidate
                    └──► code-explorer (aspect C) ──┘
```

### 8.3 Cascading Pattern (pr-review-toolkit)

```
                    ┌──► comment-analyzer ──────────┐
                    ├──► pr-test-analyzer ──────────┤
Parent ──► Launch ──┼──► silent-failure-hunter ────┼──► Filter ──► Report
                    ├──► type-design-analyzer ─────┤    (≥80 conf)
                    ├──► code-reviewer ────────────┤
                    └──► code-simplifier ──────────┘
```

### 8.4 Confidence Scoring

```
0-25:  False positive
26-50: Minor nitpick
51-75: Valid but low-impact
76-90: Important issue
91-100: Critical bug

Filter threshold: ≥80 confidence
```

---

## 9. Environment & Variables

### 9.1 Plugin Environment Variables

| Variable | Scope | Purpose |
|----------|-------|---------|
| `${CLAUDE_PLUGIN_ROOT}` | Hooks | Plugin directory (portable paths) |
| `$CLAUDE_PROJECT_DIR` | All | Project root |
| `$CLAUDE_ENV_FILE` | SessionStart | Persist env vars |
| `$CLAUDE_CODE_REMOTE` | All | Remote context flag |
| `$TOOL_INPUT` | Prompt hooks | Tool input data |
| `$TOOL_RESULT` | PostToolUse | Tool result data |
| `$USER_PROMPT` | UserPromptSubmit | User's prompt |

### 9.2 Command Variables

| Variable | Context | Purpose |
|----------|---------|---------|
| `$ARGUMENTS` | Commands | Arguments passed to slash command |
| `$1, $2` | Commands | Positional arguments |

---

## 10. Security Architecture

### 10.1 Tool Restrictions

```markdown
---
allowed-tools: Read, Grep, Bash(git:*)
---
```

### 10.2 Hook Security Patterns

```bash
#!/bin/bash
set -euo pipefail

# Always validate inputs
input=$(cat)
tool_name=$(echo "$input" | jq -r '.tool_name')

# Path traversal protection
if [[ "$file_path" == *".."* ]]; then
  echo '{"decision": "deny", "reason": "Path traversal"}' >&2
  exit 2
fi

# Sensitive file protection
if [[ "$file_path" == *".env"* ]]; then
  echo '{"decision": "deny", "reason": "Sensitive file"}' >&2
  exit 2
fi
```

### 10.3 Monitored Security Patterns (security-guidance plugin)

1. Command injection
2. XSS attacks
3. eval() usage
4. Dangerous HTML
5. pickle deserialization
6. os.system calls
7. SQL injection patterns
8. Path traversal
9. Credential exposure

---

## 11. Key Architectural Insights

### 11.1 Design Philosophy

1. **Hierarchical Delegation**: Main Claude delegates to specialized subagents
2. **Stateless Execution**: Each agent invocation is independent
3. **Single-Turn Response**: Agents complete and return one message
4. **Progressive Disclosure**: Information revealed in layers
5. **Event-Driven**: Hooks respond to lifecycle events
6. **Composability**: Small, focused agents orchestrated together

### 11.2 Critical Implementation Details

1. **Color Pool**: 8 colors to distinguish nested agents visually
2. **Context Isolation**: Each agent has its own context window
3. **Tool Inheritance**: Agents can access subset or all tools
4. **Hook Parallelism**: All matching hooks run in parallel
5. **Resume Capability**: Agents can continue from previous state

### 11.3 Bundle Analysis

```
cli.js: 10.9MB minified JavaScript
├── Core runtime
├── Tool implementations
├── Agent system
├── Hook system
├── API client
├── Terminal UI (Ink)
└── Bundled dependencies
```

---

## 12. File Reference

### 12.1 NPM Package Structure

```
package/
├── cli.js           # Main bundle (10.9MB)
├── package.json     # Manifest
├── sdk-tools.d.ts   # TypeScript definitions
├── tree-sitter.wasm # Parser
├── tree-sitter-bash.wasm
└── vendor/
    ├── ripgrep/     # Platform binaries
    └── claude-code-jetbrains-plugin/
```

### 12.2 GitHub Repository Structure

```
github-repo/
├── .claude/commands/    # Project commands
├── .github/workflows/   # 9 GitHub Actions
├── examples/            # Hook examples
├── plugins/             # 14 official plugins
├── scripts/             # Automation scripts
├── CHANGELOG.md         # Version history
└── README.md            # Documentation
```

---

## 13. Conclusion

Claude Code represents a sophisticated **agentic system architecture** where:

1. A **main Claude instance** orchestrates work through the **Task tool**
2. **Specialized subagents** handle complex tasks autonomously
3. **Hooks** provide event-driven customization and policy enforcement
4. **Plugins** extend functionality with commands, agents, skills, and hooks
5. **Progressive disclosure** manages cognitive load for users and agents

The system achieves powerful automation while maintaining human oversight through:
- Confidence-based filtering
- Hook-based validation
- Explicit user approvals
- Tool restrictions

This architecture enables Claude Code to decompose complex software engineering tasks into specialized autonomous workflows while preserving safety, extensibility, and user control.

---

*Generated by architectural analysis on 2025-11-30*
*Claude Code v2.0.55*
