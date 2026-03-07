# CR-factory-droid-control-flow-202601: Control Flow, Data Flow & Claude Code Differentiation

**Date**: 2026-02-18
**Binary**: `/Users/amuldotexe/.local/bin/droid` (Mach-O arm64, 95.9 MB)
**Version**: 1.0.71 (CLI), Bun 1.3.8 (embedded runtime)
**Method**: Binary decompilation via `strings`, `otool`, `nm`, minified JS analysis
**Companion doc**: `CR-factory-droid-202601.md` (architecture + ByteRank thesis)
**Cross-reference**: `ES-V200-attempt-01.md` (executable contract ledger)

---

## 1. Governing Insight (Minto Pyramid)

> **Factory Droid is Claude Code with a project-manager bolted on top.**
>
> The agent loop, tool set, TUI framework, and permission model are architecturally identical. What Droid adds is a *decomposition layer* (orchestrator/worker/handoff) that removes the hardest part of coding from the developer: deciding WHAT to build and in WHAT order. This — not any technical superiority — is why people love it.

---

## 2. Five-Phase Control Flow (Decompiled)

```
Phase 1          Phase 2              Phase 3              Phase 4            Phase 5
BOOTSTRAP        MODE DETECTION       MISSION PROPOSAL     ARTIFACT CREATION  WORKER DISPATCH
                                                                              + REVIEW LOOP

┌──────────┐    ┌──────────────┐    ┌─────────────────┐   ┌──────────────┐  ┌──────────────┐
│ Auth      │    │ Is prompt    │    │ Orchestrator     │   │ Write to     │  │ For each     │
│ check     │───▷│ complex      │───▷│ proposes         │──▷│ disk:        │─▷│ feature:     │
│ (WorkOS)  │    │ enough for   │    │ mission:         │   │              │  │              │
│           │    │ AGI mode?    │    │                  │   │ • mission.md │  │ 1. Spawn     │
│ Model     │    │              │    │ • title          │   │ • features   │  │    worker    │
│ selection │    │ YES → upgrade│    │ • feature list   │   │   .json      │  │ 2. Worker    │
│           │    │   to orch.   │    │ • skill mapping  │   │ • AGENTS.md  │  │    executes  │
│ Session   │    │              │    │ • env setup      │   │ • services   │  │ 3. Handoff   │
│ create    │    │ NO → single  │    │                  │   │   .yaml      │  │    returned  │
│           │    │   agent loop │    │ USER APPROVES?   │   │ • skills/    │  │ 4. Orch.     │
│ CWD       │    │              │    │ YES → Phase 4    │   │   *.md       │  │    reviews   │
│ detect    │    │              │    │ NO  → revise     │   │ • init.sh    │  │ 5. Retry if  │
└──────────┘    └──────────────┘    └─────────────────┘   └──────────────┘  │    fail      │
                                                                             └──────────────┘
```

### Phase 1: Bootstrap

From decompiled code — session creation, auth, model selection:

```
createNewSession() → loadSession(sessionId) →
  upgradeToOrchestratorSession() or
  standard single-agent loop
```

Auth via WorkOS (`api.workos.com`) or API key (`fk-...`). Model resolved from settings with provider detection (Anthropic, OpenAI, custom BYOK). Reasoning effort set per model (`low|medium|high|off`).

### Phase 2: Mode Detection

The binary contains two interaction modes:

| Mode | Trigger | Session Type | Behavior |
|------|---------|-------------|----------|
| **Standard** | Simple prompts, `/` commands | Single agent | Identical to Claude Code — one LLM session with tools |
| **AGI / Mission** | Complex multi-step tasks | Orchestrator + Workers | Hierarchical decomposition with artifact creation |

From decompiled JS:
```javascript
// Mode upgrade detection
if (aA() !== "agi") return;
eA.upgradeToOrchestratorSession().then(() => { ... })
```

The mode toggle uses `shift+tab` to cycle between `auto` and `spec` modes, and `ctrl+L` for autonomy level.

### Phase 3: Mission Proposal

The orchestrator (LLM session with architect role) proposes a mission via the `ProposeMission` tool:

```typescript
// Decompiled Zod schema
ProposeMission.inputSchema = {
  title:            string,         // Mission title
  proposal:         string,         // Detailed markdown proposal
  workingDirectory: string?,        // Workers spawn here
}
// Returns:
{
  accepted:   boolean,              // User approval gate
  missionDir: string?,              // Path to created mission dir
}
```

Key constraint from decompiled system prompt:
> "You are an architect. You NEVER write implementation code yourself."

The orchestrator's ONLY job is decomposition. It produces a plan, never touches code.

### Phase 4: Artifact Creation

On approval, the orchestrator writes these files to disk BEFORE any code is written:

```
{missionDir}/
├── mission.md                    # Full mission brief
├── features.json                 # Structured feature list
├── AGENTS.md                     # Instructions for workers
└── (references)
    ├── .factory/services.yaml    # Commands/services manifest (CRITICAL)
    ├── .factory/skills/{type}/   # Worker skill definitions
    │   └── SKILL.md
    └── .factory/init.sh          # Idempotent environment setup
```

**`features.json` schema** (decompiled):
```json
{
  "features": [
    {
      "id": "F1",
      "description": "Extract JWT logic into dedicated module",
      "skillName": "backend-worker",
      "milestone": "M1",
      "preconditions": ["auth module exists"],
      "expectedBehavior": ["JWT validation in separate file"],
      "verificationSteps": ["npm test passes", "curl /auth returns 200"],
      "status": "pending"
    }
  ]
}
```

**`services.yaml`** — single source of truth for operational commands:
```yaml
commands:
  test: "npm test"
  build: "npm run build"
  lint: "npm run lint"
services:
  - name: dev-server
    start: "npm run dev"
    stop: "kill -9 $(lsof -ti:3000)"
    port: 3000
```

### Phase 5: Worker Dispatch + Review Loop

```
                    ┌────────────────────────┐
                    │    ORCHESTRATOR         │
                    │    (never writes code)  │
                    └───────┬────────────────┘
                            │
               RunNextWorker│tool call
                            │
              ┌─────────────┼─────────────────┐
              ▼             ▼                  ▼
        ┌──────────┐  ┌──────────┐      ┌──────────┐
        │ Worker 1  │  │ Worker 2  │      │ Worker N  │
        │           │  │           │      │           │
        │ 1. Read   │  │ 1. Read   │      │ 1. Read   │
        │    mission│  │    mission│      │    mission│
        │    .md    │  │    .md    │      │    .md    │
        │ 2. Read   │  │ 2. Read   │      │ 2. Read   │
        │    skill  │  │    skill  │      │    skill  │
        │ 3. Execute│  │ 3. Execute│      │ 3. Execute│
        │ 4. Verify │  │ 4. Verify │      │ 4. Verify │
        │ 5. Handoff│  │ 5. Handoff│      │ 5. Handoff│
        └─────┬─────┘  └─────┬─────┘      └─────┬─────┘
              │               │                   │
              ▼               ▼                   ▼
        ┌──────────┐  ┌──────────┐      ┌──────────┐
        │ Handoff   │  │ Handoff   │      │ Handoff   │
        │ pass ✓    │  │ fail ✗    │      │ pass ✓    │
        └──────────┘  └──────────┘      └──────────┘
                            │
                   Orchestrator retries
                   Worker 2 or adjusts
                   mission scope
```

**RunNextWorker tool** (decompiled):
```typescript
RunNextWorker.outputSchema = {
  started:              boolean,
  workerHandoffs:       WorkerHandoff[],    // All handoffs since last run
  latestWorkerHandoff: {
    featureId:          string,
    resultState:        "pass" | "fail",
    handoffFile:        string,             // Path to JSON on disk
    handoffJson:        string,             // Full JSON inline
  },
  systemMessage:        string?,
  completedFeatures:    { id, description }[],
  totalFeatures:        number,
  workerCount:          number,
  startedAt:            string,             // ISO timestamp
}
```

---

## 3. Data Flow: Orchestrator ↔ Workers ↔ Disk

```
┌─────────────────────────────────────────────────────────────────────┐
│                       ORCHESTRATOR SESSION                          │
│                                                                     │
│  Input:   User prompt ("Refactor the auth system")                 │
│  Reads:   .factory/services.yaml, .factory/skills/*, AGENTS.md     │
│  Writes:  mission.md, features.json                                │
│  Calls:   ProposeMission → RunNextWorker → (loop)                  │
│  NEVER:   edit_file, create_file, shell (code-writing tools)       │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │  ProposeMission                                              │   │
│  │  ├── title: "Auth Refactoring"                               │   │
│  │  ├── proposal: "## Plan\n1. Extract JWT..."                  │   │
│  │  └── workingDirectory: "/path/to/repo"                       │   │
│  │       │                                                      │   │
│  │       ▼ USER APPROVAL GATE                                   │   │
│  │       │                                                      │   │
│  │  RunNextWorker (loop)                                        │   │
│  │  ├── spawns worker sub-session                               │   │
│  │  ├── worker reads: mission.md + SKILL.md + services.yaml     │   │
│  │  ├── worker executes: code changes + verification            │   │
│  │  └── worker returns: structured handoff JSON                 │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘

          ▲                    │                    ▼
          │                    │              ┌───────────┐
    Handoff JSON         Feature ID           │  DISK     │
    (inline +            assignment           │           │
     file path)                               │ mission/  │
                                              │ ├─ handoffs/
                                              │ │  ├─ F1.json
                                              │ │  ├─ F2.json
                                              │ │  └─ F3.json
                                              │ ├─ mission.md
                                              │ ├─ features.json
                                              │ └─ AGENTS.md
                                              │           │
                                              │ .factory/ │
                                              │ ├─ services.yaml
                                              │ ├─ skills/
                                              │ │  ├─ backend-worker/
                                              │ │  │  └─ SKILL.md
                                              │ │  └─ frontend-worker/
                                              │ │     └─ SKILL.md
                                              │ ├─ library/
                                              │ │  ├─ perf-state.md
                                              │ │  └─ migration-status.md
                                              │ └─ init.sh
                                              └───────────┘
```

### Handoff Structure (Complete Decompiled Zod Schema)

```typescript
{
  // Required fields
  salientSummary:     string (20-400 chars, no newlines),
  whatWasImplemented:  string (min 50 chars),
  whatWasLeftUndone:   string,

  // Verification
  verification: {
    commandsRun: [{
      command:      string,    // "npm test"
      exitCode:     number,    // 0
      observation:  string,    // "47 tests passing"
    }],
    interactiveChecks?: [{
      action:       string,    // "Clicked login button"
      observed:     string,    // "Redirected to dashboard"
    }],
  },

  // Testing
  tests: {
    added:    [{ file: string, cases: [{ name, verifies }] }],
    updated:  string[],
    coverage: string,
  },

  // Issues
  discoveredIssues: [{
    severity:     "blocking" | "non_blocking" | "suggestion",
    description:  string,
    suggestedFix: string?,
  }],

  // Skill compliance (optional)
  skillFeedback?: {
    followedProcedure:  boolean,
    deviations: [{
      step:              string,
      whatIDidInstead:    string,
      why:               string,
    }],
    suggestedChanges:   string[]?,
  },

  // Skill compliance report (from scrutiny validator)
  skillComplianceReport?: {
    skillName:              string,
    systematicDeviations: [{
      step:                 string,
      expectedBehavior:     string,
      observedBehavior:     string,
      workerCount:          number,
    }],
  },

  // Git
  commitId?: string,
}
```

Real handoff examples from binary:
```json
{
  "salientSummary": "Migrated UserService to the functional pattern and updated 12 call sites; ran `npm test` (47 passing) and updated `.factory/library/migration-status.md` with the MIGRATED status + commit.",
  "verification": {
    "commandsRun": [
      {"command": "grep 'Status: MIGRATED' .factory/library/migration-status.md", "exitCode": 0, "observation": "Status updated"}
    ]
  }
}
```

---

## 4. Risk Assessment System

Every shell command execution goes through a tiered risk assessment:

```
User prompt
    │
    ▼
┌───────────────┐
│ AUTONOMY LEVEL│
│               │
│ off  → all actions require approval
│ low  → edits and read-only commands only
│ med  → allow reversible commands
│ high → allow all commands
│               │
│ skip-permissions-unsafe → NO GATES AT ALL
└───────┬───────┘
        │
        ▼
┌───────────────────────────┐
│ TOOL RISK CLASSIFICATION  │
│                           │
│ Read tools:               │
│   view_file, view_folder, │
│   glob_tool, grep_tool    │
│   → Always allowed        │
│                           │
│ Edit tools:               │
│   edit_file, create_file, │
│   apply_patch             │
│   → Allowed at low+       │
│                           │
│ Execute tools:            │
│   shell                   │
│   → Risk-assessed per cmd │
│                           │
│ Each shell command gets:  │
│   riskLevel: {            │
│     value: "low"|"med"|   │
│            "high",        │
│     reason: string        │
│   }                       │
└───────────────────────────┘
```

From decompiled autonomy level descriptions:
- **Auto (Off)**: "all actions require approval"
- **Auto (Low)**: "edits and read-only commands"
- **Auto (Med)**: "allow reversible commands"
- **Auto (High)**: "allow all commands"

Specific risk rules (from decompiled prompts):
- Port-based kills on ports NOT declared in `.factory/services.yaml` = **high risk**
- Filesystem writes to user data directory `~/.factory/blog_posts/` = **low risk**
- `droid exec --skip-permissions-unsafe` = bypasses ALL permission gates (danger flag)

---

## 5. Stop Conditions (Decompiled from Orchestrator System Prompt)

From `extracted_agent_framework.txt` line 20:

> **"Stop the mission and return control to the user when:"**

Six conditions where the orchestrator halts:

| # | Condition | Category |
|---|-----------|----------|
| 1 | User explicitly asks to stop | User intent |
| 2 | No features matching assigned skill are pending | Completion |
| 3 | Large scope issues outside worker's skill | Scope boundary |
| 4 | Blocking discovered issue that can't be resolved | Blocker |
| 5 | Environment setup fails repeatedly | Environment |
| 6 | Worker skill requires user input/decision | Decision gate |

Worker-level stop rules (decompiled):
- "Large scope or outside their skill -> report to orchestrator"
- "Manageable existing issues under their skill -> fix them"
- Workers NEVER use `ExitSpecMode` — only orchestrator can

---

## 6. Why People Love It: Six Predictions from Decompiled Control Flow

### Prediction 1: It Removes the Decomposition Tax

```
BEFORE (Claude Code):                 AFTER (Droid):

User thinks:                          User says:
  "What should I build first?"          "Refactor the auth system"
  "How should I split this?"
  "What are the dependencies?"        Orchestrator returns:
  "What order do I do things?"          features.json with F1..F12
                                        skill assignments
Then user tells Claude Code            dependency ordering
what to do, one step at a time        verification steps
```

The orchestrator does the HARD cognitive work — task decomposition, ordering, skill matching. The user just approves or rejects.

### Prediction 2: Artifacts Build Trust Before Code Exists

```
mission.md          ← "Here's what I'll do"
features.json       ← "Here's every feature, ordered"
services.yaml       ← "Here's how to run things"
skills/*.md         ← "Here's the procedure for each type"
init.sh             ← "Here's how to set up the env"

USER READS ALL THIS → then says "yes, proceed"

Only THEN does code get written.
```

This is a trust-building mechanism. The user sees the full plan materialized as files on disk — not just a chat message that scrolls away.

### Prediction 3: Structured Handoffs Make Progress Visible

Each worker returns a handoff with:
- `salientSummary` (20-400 chars — forced to be concise)
- `verification.commandsRun` (what tests passed)
- `tests.added` (what new tests were written)
- `discoveredIssues` (what problems were found)

These are written to `{missionDir}/handoffs/F1.json`, `F2.json`, etc. Progress is visible as JSON files on disk — not buried in chat history.

### Prediction 4: The Orchestrator Never Codes

Enforced by system prompt: the orchestrator session has code-writing tools DISABLED. It can only:
- `ProposeMission`
- `RunNextWorker`
- `SelectFeature`
- Read tools (`view_file`, `grep_tool`)

This separation means the "architect" can't accidentally start coding and lose the big picture.

### Prediction 5: Casual Requirements Are Treated Seriously

```
User says:  "fix the auth bug and add rate limiting"

Droid produces:
  mission.md:     2-page brief with environment analysis
  features.json:  F1: "Diagnose auth bug" (backend-worker)
                  F2: "Fix auth validation" (backend-worker)
                  F3: "Add rate limiter middleware" (backend-worker)
                  F4: "Write rate limit tests" (backend-worker)
  services.yaml:  test: "npm test", dev: "npm run dev"
```

A 10-word prompt becomes a structured project with verification steps. This makes developers feel their request is taken seriously.

### Prediction 6: Risk-Aware Tool Execution

Every `shell` command gets a risk assessment. At `--auto medium`, the user sees:
```
Auto (Med) - allow reversible commands
```

This gives users confidence to grant more autonomy without fear of destructive actions. The tiered system (`off/low/med/high`) lets users gradually increase trust.

---

## 7. Tool-for-Tool Comparison: Claude Code vs Factory Droid

### Identical Tools (9/14)

| Claude Code | Factory Droid | Notes |
|-------------|--------------|-------|
| `Read` | `view_file` | Same: read file contents |
| `Write` | `create_file` | Same: create new files |
| `Edit` | `edit_file` | Same: modify existing files |
| `Bash` | `shell` | Same: execute CLI commands via PTY |
| `Glob` | `glob_tool` | Same: pattern-based file finding |
| `Grep` | `grep_tool` | Droid embeds ripgrep in binary; Claude Code calls external rg |
| `WebSearch` | `web_search` | Same: internet search |
| `WebFetch` | `fetch_url` | Same: fetch and analyze web content |
| `TodoWrite` | `todo_write` | Same: task management |

### Claude Code Only

| Tool | What It Does | Droid Equivalent |
|------|-------------|------------------|
| `Task` (sub-agents) | Spawn specialized sub-agents | Workers (but with mission/handoff structure) |
| `NotebookEdit` | Edit Jupyter notebooks | None |
| `EnterPlanMode` | Switch to planning mode | `SpecMode` (similar concept, different implementation) |
| `AskUserQuestion` | Structured user questions | None (uses chat messages) |
| `mcp__*` tools | MCP server integrations | MCP client support (consumes, doesn't expose) |

### Factory Droid Only

| Tool | What It Does | Claude Code Equivalent |
|------|-------------|----------------------|
| `apply_patch` | Apply unified diff patches | `Edit` (line-by-line replacement) |
| `ProposeMission` | Orchestrator proposes mission | None (no decomposition concept) |
| `RunNextWorker` | Dispatch worker for feature | `Task` (but without handoff structure) |
| `SelectFeature` | Choose feature for worker | None |
| `EndFeatureRun` | Worker submits handoff | None |
| `Skill` | Execute `.factory/skills/` | None (Claude has hooks, not skills) |
| `GenerateDroid` | Create custom droid config | None |
| `store_agent_readiness_report` | ByteRank report persistence | None |
| `slack_post_message` | Post to Slack | None |
| Browser tools (CDP) | Chrome DevTools Protocol | None |

---

## 8. Directory Convention Comparison

```
CLAUDE CODE                           FACTORY DROID
-----------                           -------------

.claude/                              .factory/
├── settings.json                     ├── droids/           ← custom agent configs
└── plans/                            │   └── my-droid.yaml
                                      ├── skills/           ← reusable procedures
CLAUDE.md  (project instructions)     │   ├── backend-worker/
                                      │   │   └── SKILL.md
                                      │   └── frontend-worker/
                                      │       └── SKILL.md
                                      ├── commands/         ← custom slash commands
                                      ├── library/          ← shared state files
                                      │   ├── perf-state.md
                                      │   └── migration-status.md
                                      ├── services.yaml     ← operational manifest
                                      └── init.sh           ← env setup script

                                      AGENTS.md  (project instructions)

                                      {missionDir}/         ← per-mission state
                                      ├── mission.md
                                      ├── features.json
                                      ├── AGENTS.md
                                      └── handoffs/
                                          ├── F1.json
                                          └── F2.json
```

**Key difference**: Claude Code has a minimal footprint (`.claude/` + `CLAUDE.md`). Droid creates a full project scaffold (`.factory/` with skills, services, library, commands) plus per-mission directories. This is heavier but provides more structure for complex multi-feature projects.

---

## 9. Architectural Differentiation

### Single Agent (Claude Code) vs Multi-Agent Hierarchy (Droid)

```
CLAUDE CODE:                          FACTORY DROID:

┌──────────────┐                      ┌──────────────────┐
│ ONE SESSION   │                      │ ORCHESTRATOR      │
│               │                      │ (architect only)  │
│ Reads code    │                      │                   │
│ Plans         │                      │ Decomposes        │
│ Writes code   │                      │ Assigns           │
│ Tests         │                      │ Reviews           │
│ Debugs        │                      │ NEVER codes       │
│               │                      └────────┬──────────┘
│ User steers   │                               │
│ every step    │                      ┌────────┼────────┐
│               │                      ▼        ▼        ▼
└──────────────┘                      Worker   Worker   Worker
                                      (codes)  (codes)  (codes)
                                      Handoff  Handoff  Handoff
```

| Dimension | Claude Code | Factory Droid |
|-----------|------------|---------------|
| **Agent count** | 1 (+ sub-agents via Task) | 1 orchestrator + N workers |
| **Who plans?** | User + single agent | Orchestrator (dedicated) |
| **Who codes?** | Same agent that plans | Workers only (orchestrator can't) |
| **State persistence** | Chat history only | Disk artifacts (mission.md, handoffs/) |
| **Progress tracking** | TodoWrite (in-memory) | features.json + handoff files (on disk) |
| **Decomposition** | Manual (user breaks down tasks) | Automatic (orchestrator proposes) |
| **Verification** | Ad-hoc (agent decides) | Structured (verificationSteps in features.json) |
| **Error recovery** | User re-prompts | Orchestrator retries failed workers |
| **Context isolation** | One big context window | Each worker gets fresh context |
| **Skill reuse** | None (each session starts fresh) | `.factory/skills/` persisted across missions |

### What This Means in Practice

**Claude Code excels at**: Single-feature work, debugging, exploration, code review, quick edits. Low ceremony, fast iteration.

**Droid excels at**: Multi-feature projects, greenfield builds, large refactors, team onboarding. High ceremony, structured execution.

**The tradeoff**: Droid's decomposition overhead is wasted on simple tasks. Claude Code's lack of decomposition is painful on complex tasks.

---

## 10. SpecMode vs PlanMode

Both tools have a "planning before coding" concept:

| Aspect | Claude Code (PlanMode) | Factory Droid (SpecMode) |
|--------|----------------------|------------------------|
| **Trigger** | `EnterPlanMode` tool | `shift+tab` to cycle to spec mode |
| **What happens** | Agent explores codebase, writes plan to file, asks user approval | Separate model selection for planning, orchestrator creates mission artifacts |
| **Model** | Same model (can switch) | Dedicated spec mode model (configurable separately) |
| **Output** | Markdown plan file in `.claude/plans/` | mission.md + features.json + services.yaml on disk |
| **Approval** | `ExitPlanMode` → user reviews | `ProposeMission` → user accepts/rejects |
| **After approval** | Same session continues to implement | Workers spawned as separate sessions |

---

## 11. What Droid Adds That Matters (vs Claude Code)

### 11.1 Mission Decomposition (HIGH value)

The single biggest differentiator. Droid turns "build me X" into a structured feature list with skill assignments, dependencies, and verification steps. Claude Code's `Task` tool can spawn sub-agents, but there's no structured decomposition — the user or agent decides ad-hoc what to delegate.

### 11.2 Persistent Artifacts (HIGH value)

Mission state lives on disk as JSON and Markdown. This survives context window limits, session restarts, and even tool crashes. Claude Code's state is ephemeral (chat history + plan file).

### 11.3 Skill System (MEDIUM value)

Reusable worker procedures with YAML frontmatter. Skills define: what tools to use, what procedure to follow, what a good handoff looks like. Claude Code has hooks (shell commands on events) and `CLAUDE.md` instructions, but no equivalent of parameterized skill procedures.

### 11.4 ByteRank (LOW value for V200)

100+ criterion repo quality scorer. But: LLM-evaluated (expensive, non-deterministic). Parseltongue's graph algorithms are strictly superior for code quality metrics.

### 11.5 Browser Automation (LOW value for V200)

Full CDP support. Outside Parseltongue's scope entirely.

### 11.6 Custom Droids (MEDIUM value)

`.factory/droids/my-droid.yaml` — define custom agent configurations with specific system prompts, tools, and models. Claude Code has no equivalent of per-project agent personas.

---

## 12. What Claude Code Has That Droid Doesn't

### 12.1 MCP Server Mode

Claude Code can BE an MCP server (exposing tools to other agents). Droid is only an MCP CLIENT (consuming tools from servers like Braintrust, Fireflies). This is critical for the Parseltongue use case — V200 exposes analysis as MCP tools.

### 12.2 Notebook Support

`NotebookEdit` for Jupyter notebooks. Droid has no equivalent.

### 12.3 LSP Integration

Claude Code integrates with LSP servers (cclsp) for `find_definition`, `find_references`, `get_diagnostics`. Droid uses ripgrep for code search — text-level, not semantic.

### 12.4 Structured Questions

`AskUserQuestion` with multiple-choice options. Droid uses free-text chat for all user interaction.

### 12.5 Simpler Mental Model

One agent, one context, one conversation. No mission directories, no handoff files, no services.yaml to maintain. Lower barrier to entry.

---

## 13. V200 Implications

| Insight | V200 Action | Contract |
|---------|------------|----------|
| Handoff format is a model for MCP responses | Add `provenance`, `verification`, `confidence` fields to tool responses | TRN-C01 |
| Features.json = decomposition receipt | V200's `get_context` should return results structured for agent decomposition (entity clusters = natural feature boundaries) | SEM-C04 |
| Workers get fresh context per feature | V200's MCP tools should be stateless — each call returns complete context, no session dependency | TRN-C01 |
| Skill system = reusable analysis recipes | V200 could expose "analysis presets" — common query patterns saved as named configurations | TRN-C03 |
| Services.yaml = operational manifest | V200 could read a similar file for per-project analysis config (taint sources/sinks, entry points, ignore patterns) | TRN-C02 |
| ByteRank criteria = feature roadmap | V200 can provide 8/11 ByteRank categories deterministically via graph algorithms | SEM-C02, CRT-C05 |

---

## 14. Summary: The Real Difference

```
CLAUDE CODE = A skilled programmer who does what you tell them

FACTORY DROID = A project manager + team of programmers who decide
                what to do, do it, and report back with receipts

PARSELTONGUE = The X-ray machine that both should consult before
               and after surgery
```

Droid's commercial success comes from removing cognitive load (decomposition, ordering, verification) — not from technical superiority in any single tool. Its tools are Claude Code's tools with different names.

**For V200**: The strategic insight is that Droid is a CONSUMER of the kind of analysis Parseltongue provides. The ideal integration is:
1. Droid's orchestrator calls Parseltongue MCP to understand the codebase (graph structure, coupling, communities)
2. Orchestrator uses Parseltongue data to decompose better (Leiden clusters = feature boundaries, SCC = circular dep warnings, blast radius = risk assessment)
3. Workers call Parseltongue MCP for context during implementation (smart-context-token-budget, forward/reverse callees)
4. Post-implementation, orchestrator validates against Parseltongue metrics (did CBO decrease? did dead code increase?)

Droid has the agent orchestration. Parseltongue has the code intelligence. They're complementary by architecture.

---

*Generated 2026-02-18. Control flow and differentiation thesis for Parseltongue V200. Source: CR05/factory-droid/ (gitignored). Method: binary decompilation (strings, otool, minified JS analysis) on Mach-O arm64 binary.*
