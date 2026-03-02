# Parseltongue Plugin Simulation

## What the user types vs what the agent receives

---

## SIMULATION 1: Without Plugin (Raw Agent)

```
USER: I want to refactor the authentication module

AGENT: I'll help you refactor the authentication module. Let me first explore the codebase...

[Agent uses generic tools: LS, Read, Grep]

AGENT: I found these files related to authentication:
- src/auth/login.rs
- src/auth/session.rs
- src/auth/middleware.rs

Which one would you like to focus on?
```

**Problem:** Agent doesn't know about:
- Which functions depend on auth code
- Complexity of each module
- Blast radius of changes
- Token-efficient context generation

---

## SIMULATION 2: With Plugin (Hook Injects Context)

### SessionStart Hook Output (Hidden from user):

```
=== PARSLETONGUE PLUGIN ACTIVE ===
Project: that-in-rust/parseltongue-rust-LLM-companion
Indexed: Yes (last: 2 hours ago)
Entities: 1,247 | Relations: 3,891

BEFORE MAKING ANY CODE CHANGES:
1. Check blast radius: parseltongue blast-radius <entity>
2. Check complexity: parseltongue complexity-hotspots
3. Generate context: parseltongue context <query>

These tools understand Rust semantics, not just text.
=====================================
```

### Agent Response (Guided by Skill):

```
USER: I want to refactor the authentication module

AGENT: I'll help you refactor authentication. Let me first understand the impact...

[Agent calls: parseltongue blast-radius auth::login]

AGENT: ## Blast Radius Analysis

Changing `auth::login` would affect:
┌─────────────────────────────────────────────────────────┐
│ Entity                      │ Type    │ Distance │ Risk │
├─────────────────────────────────────────────────────────┤
│ auth::login                 │ fn      │    0     │ HIGH │
│ auth::session::create       │ fn      │    1     │ HIGH │
│ middleware::auth_check      │ fn      │    1     │ MED  │
│ api::handlers::login_route  │ fn      │    2     │ MED  │
│ api::handlers::refresh      │ fn      │    2     │ LOW  │
│ cli::commands::login        │ fn      │    3     │ LOW  │
│ ... 12 more transitive deps │         │   3+    │ LOW  │
└─────────────────────────────────────────────────────────┘

[Agent calls: parseltongue complexity --path src/auth]

AGENT: ## Complexity Hotspots in auth/

| Function         │ Cyclomatic │ Nesting │ Params │ Rating  |
|------------------|------------|---------|--------|---------|
| login            │     12     │    4    │   5    | ⚠️ HIGH |
| validate_token   │      8     │    3    │   3    | MED     |
| refresh_session  │      4     │    2    │   2    | OK      |

**Recommendation:** Start with `refresh_session` (lowest risk), 
then `validate_token`, tackle `login` last with tests in place.

Would you like me to:
1. Generate test coverage for affected functions?
2. Create a refactoring plan with incremental steps?
3. Show the dependency graph visualization?
```

---

## SIMULATION 3: Skill-Based Decision Tree

### The `using-parseltongue` Skill (Injected at SessionStart):

```markdown
# When to Use Parseltongue

## Before ANY Code Change:
┌─────────────────────────────────────────────────────┐
│  User says: "refactor" / "change" / "modify"       │
│  ─────────────────────────────────────────────────  │
│  → RUN: parseltongue blast-radius <target>         │
│  → SHOW: Impact table with risk levels             │
└─────────────────────────────────────────────────────┘

## When Exploring Unfamiliar Code:
┌─────────────────────────────────────────────────────┐
│  User says: "understand" / "how does X work"       │
│  ─────────────────────────────────────────────────  │
│  → RUN: parseltongue context <entity> --budget 4k  │
│  → INJECT: Token-efficient context into reasoning  │
└─────────────────────────────────────────────────────┘

## When Planning Architecture:
┌─────────────────────────────────────────────────────┐
│  User says: "design" / "plan" / "architecture"     │
│  ─────────────────────────────────────────────────  │
│  → RUN: parseltongue complexity-hotspots           │
│  → RUN: parseltongue graph --format mermaid        │
│  → SHOW: Visual + metrics for decision support     │
└─────────────────────────────────────────────────────┘

## Red Flags (DO NOT):
❌ "Let me just read the files" → Use context generation instead
❌ "I'll search for usages" → Use blast radius instead
❌ "This seems simple" → Check complexity first
```

---

## SIMULATION 4: CLI Command Simulation

```bash
# What the plugin enables (via MCP tools or shell):

$ parseltongue status
Project: my-rust-app
Indexed: Yes (847 entities, 2,341 relations)
Last sync: 3 minutes ago

$ parseltongue blast-radius auth::login --format json
{
  "epicenter": "auth::login",
  "affected": [
    {"entity": "auth::session::create", "distance": 1, "risk": "high"},
    {"entity": "middleware::auth_check", "distance": 1, "risk": "medium"},
    {"entity": "api::handlers::login_route", "distance": 2, "risk": "medium"}
  ],
  "total_blast_radius": 18,
  "files_affected": 7
}

$ parseltongue context "authentication flow" --budget 4096
# Returns token-optimized context for LLM:
# - Function signatures only (not bodies)
# - Type definitions involved
# - Call slice showing the path
# ~800 tokens instead of 8000 for full files

$ parseltongue complexity --top 10
| Rank | Entity              | Score | Cyclomatic | Nesting |
|------|---------------------|-------|------------|---------|
| 1    | auth::login         | 89.2  | 12         | 4       |
| 2    | parser::tokenize    | 76.1  | 9          | 5       |
| 3    | db::migrate         | 71.4  | 8          | 3       |
...

$ parseltongue graph --from auth::login --depth 3 --format mermaid
graph TD
    A[auth::login] --> B[auth::session::create]
    A --> C[auth::validate]
    B --> D[middleware::auth_check]
    C --> D
    D --> E[api::handlers/*]
```

---

## SIMULATION 5: The "Choice" Mechanism

### Superpowers-style Hard Gate:

```markdown
## Refactoring Safety Gate

BEFORE modifying any Rust code, the agent MUST:

1. ☐ Run blast-radius analysis
2. ☐ Check complexity score
3. ☐ Identify test coverage
4. ☐ Get user confirmation on risk level

If blast_radius > 10 entities:
  ⚠️  WARNING: High impact change detected
  → Suggest breaking into smaller changes
  → Require explicit user approval

If complexity_score > 70:
  ⚠️  WARNING: Complex code, high regression risk
  → Suggest writing characterization tests first
  → Require test plan before changes
```

### Agent Behavior with Gate:

```
USER: Change the login function to support MFA

AGENT: Let me analyze the impact first...

[parseltongue blast-radius auth::login]
Result: 18 entities affected (HIGH)

[parseltongue complexity auth::login]
Result: Score 89.2 (HIGH - top 5% complex)

⚠️  SAFETY GATE TRIGGERED

This change has HIGH risk:
- 18 functions depend on auth::login
- Current complexity is 89.2 (very high)
- No existing tests detected

Recommended safer approach:
1. First: Add characterization tests for auth::login
2. Then: Extract MFA logic to separate function
3. Finally: Integrate with minimal signature change

Do you want me to:
[A] Proceed anyway (high risk)
[B] Create test coverage first (recommended)
[C] Design an incremental change plan

Your choice: _
```

---

## COMPARISON: MCP vs Plugin

| Aspect | MCP Server Only | Plugin with Skills |
|--------|-----------------|-------------------|
| Discovery | Agent may not know tools exist | Hook tells agent at session start |
| Guidance | None (agent guesses) | Skills specify WHEN to use |
| Safety | No enforcement | Hard gates prevent risky changes |
| Context | Manual tool calls | Auto-injected decision trees |
| Behavior | Same as any MCP tool | Custom workflow enforcement |

---

## What the Plugin Files Would Look Like

```
parseltongue-plugin/
├── plugin.json                    # Manifest
├── hooks/
│   ├── hooks.json                 # Hook registration
│   └── session-start              # Injects "using-parseltongue"
├── skills/
│   ├── using-parseltongue/
│   │   └── SKILL.md               # Meta-skill: when/how to use
│   ├── blast-radius-analysis/
│   │   └── SKILL.md               # Impact analysis workflow
│   ├── complexity-assessment/
│   │   └── SKILL.md               # Complexity-driven decisions
│   └── context-generation/
│       └── SKILL.md               # Token-efficient context
└── commands/
    ├── blast-radius.md            # /blast-radius command
    ├── context.md                 # /context command
    └── complexity.md              # /complexity command
```

---

## NEXT STEPS

1. **Create the plugin structure** following superpowers pattern
2. **Write SKILL.md files** with decision trees
3. **Implement SessionStart hook** that detects Rust projects
4. **Add hard gates** for high-risk operations
5. **Test with Claude Code** plugin marketplace

Want me to generate the actual plugin files?
