# ES-V200-User-Journey-Addendum: Tauri App Philosophy

## The Critical Reframe: Tauri is a Visual CLI Launcher

After reviewing the three-layer architecture from `v173-pt04-bidirectional-workflow.md`, the Tauri app's purpose must be radically reframed.

### What We Initially Thought (WRONG)

"Tauri app is a full-featured GUI where users do all their analysis, search entities, run graph algorithms, compose LLM prompts, and export results."

**Problem**: The three-layer architecture (pt04 compiler truth + LLM judgment + CPU graph algorithms) is inherently a **composable CLI workflow**. Real analysis requires:
- Piping curl outputs through jq
- Chaining multiple queries to build context
- Feeding XML exports to Claude/Cursor
- Scripting batch operations for CI/CD
- Combining graph metrics with LLM reasoning

GUIs don't do this well. Terminals do.

### What Tauri Actually Is (CORRECT)

**The Tauri app is a visual **launcher** for CLI commands, not a replacement for terminal workflows.**

Think of it as:
- **VSCode's Command Palette** for Parseltongue
- **GitHub Desktop** for git (shows status, but power users live in terminal)
- **Postman Collections** (generates curl commands you copy-paste)

### The Three User Modes

```
MODE 1: QUICK LOOKUP (Tauri app sufficient)
- "Show me codebase stats"
- "Find function X"
- "What are the top 10 hotspots?"
- User clicks, sees answer, done in <30 seconds

MODE 2: EXPLORATORY ANALYSIS (Tauri → Terminal handoff)
- "I need to understand this module"
- Tauri shows overview + top entities
- User clicks "Terminal Workflow" button
- Copies 5 curl commands to terminal
- Pipes through jq, saves outputs, composes context

MODE 3: AUTOMATED/LLM WORKFLOWS (Pure CLI)
- "Run this analysis in CI on every PR"
- "Feed blast radius into Claude for review"
- "Generate architecture report weekly"
- No GUI involved — scripts + cron + LLM APIs
```

### Tauri App Core Functionality (What to Build)

#### 1. Instant Visual Orientation
```
┌──────────────────────────────────────────────────┐
│ 📊 my-service: 5,150 entities, 18,942 edges     │
│                                                  │
│ Health: ⚠️ 1 cycle violation, 2 God Objects     │
│ Tokens: 847K (fits in Claude 3.5 context)       │
│                                                  │
│ Top Risk: auth::verify_token (142 callers)      │
└──────────────────────────────────────────────────┘
```

User sees: "Medium codebase, one problem to fix, token budget OK."

#### 2. One-Click Command Generation
```
User clicks "Top Risk: auth::verify_token"

→ Tauri displays:

┌───────────────────────────────────────────────────┐
│ CLI Commands for auth::verify_token              │
├───────────────────────────────────────────────────┤
│                                                   │
│ See details:                                      │
│ $ curl localhost:7777/my-service/entity-detail?\\ │
│        key=rust:fn:verify_token            [Copy] │
│                                                   │
│ Who calls this?                                   │
│ $ curl localhost:7777/my-service/reverse-callers?\\│
│        entity=rust:fn:verify_token         [Copy] │
│                                                   │
│ Blast radius (2 hops):                            │
│ $ curl localhost:7777/my-service/blast-radius?\\  │
│        entity=rust:fn:verify_token&hops=2  [Copy] │
│                                                   │
│ Export for LLM:                                   │
│ $ curl localhost:7777/my-service/entity-detail?\\ │
│        key=rust:fn:verify_token \\                │
│        > verify_token.xml                  [Copy] │
│                                                   │
│ Then paste verify_token.xml into Claude          │
└───────────────────────────────────────────────────┘
```

Every [Copy] button copies the exact curl command to clipboard. User pastes in terminal.

#### 3. Pre-Built Workflow Templates
```
Tauri has 6 workflow templates (from README workflows):

┌────────────────────────────────────────────────────┐
│ 🎯 Workflow Templates                              │
├────────────────────────────────────────────────────┤
│                                                    │
│ 1. New Codebase Orientation (5 commands)   [Copy] │
│    → Get scale, health, hotspots, modules          │
│                                                    │
│ 2. Bug Hunting (4 commands)                [Copy] │
│    → Search → Detail → Callers → Blast radius      │
│                                                    │
│ 3. Safe Refactoring Check (4 commands)     [Copy] │
│    → Callers → Blast radius → Cycles → Context     │
│                                                    │
│ 4. Architecture Review (7 commands)        [Copy] │
│    → SCC, SQALE, K-Core, PageRank, etc.            │
│                                                    │
│ 5. LLM Code Review (3 commands)            [Copy] │
│    → Entity → Blast radius → Smart context → XML   │
│                                                    │
│ 6. CI Pre-Merge Gate (5 commands)          [Copy] │
│    → Blast radius < 100? Cycles? Unsafe? Coupling? │
│                                                    │
└────────────────────────────────────────────────────┘
```

[Copy] button copies **all commands in the workflow** as a bash script:

```bash
#!/bin/bash
# Parseltongue Workflow: New Codebase Orientation

BASE_URL="http://localhost:7777/my-service"

echo "=== Statistics ==="
curl -s "$BASE_URL/statistics" | jq

echo "\n=== Circular Dependencies ==="
curl -s "$BASE_URL/circular-deps" | jq

echo "\n=== Top 10 Hotspots ==="
curl -s "$BASE_URL/hotspots?top=10" | jq

echo "\n=== Semantic Clusters ==="
curl -s "$BASE_URL/clusters" | jq

echo "\n=== Coverage Report ==="
curl -s "$BASE_URL/coverage" | jq
```

User pastes this in terminal, runs it, gets full orientation in one command.

#### 4. Search with Command Generation
```
User types: "verify_token"

┌───────────────────────────────────────────────────┐
│ 🔍 Search Results (4 matches)                     │
├───────────────────────────────────────────────────┤
│                                                   │
│ ✓ rust:fn:verify_token                            │
│   src/auth/token.rs:45                            │
│   142 callers ⚠️ HIGH RISK                        │
│   [View in Terminal] [Copy curl command]          │
│                                                   │
│ ✓ rust:fn:verify_token_expiry                     │
│   src/auth/token.rs:89                            │
│   3 callers                                       │
│   [View in Terminal] [Copy curl command]          │
│                                                   │
│ ...                                               │
└───────────────────────────────────────────────────┘

Click [View in Terminal] → Opens terminal with command pre-filled
Click [Copy curl command] → Copies command to clipboard
```

#### 5. Add Project Flow (Drag-and-Drop or Browse)
```
User opens Tauri app, sees project sidebar:

┌─────────────────────────────────────────────────┐
│ Projects                                        │
├─────────────────────────────────────────────────┤
│ ● my-service         :7777                      │
│ ● api-gateway        :7778                      │
├─────────────────────────────────────────────────┤
│ [+ Add Project]                                 │
└─────────────────────────────────────────────────┘

User clicks [+ Add Project] → Modal opens:

┌─────────────────────────────────────────────────┐
│ Add Project                            [✕ Close]│
├─────────────────────────────────────────────────┤
│                                                 │
│ Choose project folder:                          │
│                                                 │
│ ┌─────────────────────────────────────────────┐ │
│ │ Drag folder here                            │ │
│ │          or                                 │ │
│ │     [📁 Browse...]                          │ │
│ └─────────────────────────────────────────────┘ │
│                                                 │
│ [Cancel]                            [Add & Analyze]│
└─────────────────────────────────────────────────┘

User drags ~/code/billing-service → Tauri shows:

┌─────────────────────────────────────────────────┐
│ Add Project                            [✕ Close]│
├─────────────────────────────────────────────────┤
│                                                 │
│ 📁 ~/code/billing-service                       │
│                                                 │
│ Project slug: billing-service           [Edit]  │
│ (Used in URLs: /billing-service/...)            │
│                                                 │
│ ⚙️ Options:                                     │
│ ☑ Auto-start server if not running             │
│ ☑ Watch for file changes                        │
│ ☐ Rust-only mode (skip other languages)        │
│                                                 │
│ [Cancel]                     [Start Ingestion]  │
└─────────────────────────────────────────────────┘

User clicks [Start Ingestion] → Live progress:

┌─────────────────────────────────────────────────┐
│ Analyzing: billing-service                      │
├─────────────────────────────────────────────────┤
│                                                 │
│ [████████████████░░░░] 82% (3,421/4,150)        │
│                                                 │
│ 📊 Entities found:    3,421                     │
│ 🔗 Dependencies:     12,847                     │
│ 📁 Files processed:    853 / 1,042              │
│ ⚡ Speed:            ~4,000 entities/sec        │
│                                                 │
│ Server: localhost:7779 (auto-assigned)          │
│ Port file: ~/.parseltongue/billing-service.port │
│                                                 │
│ Currently parsing:                              │
│   src/handlers/payment.rs                       │
│   src/models/invoice.rs                         │
│   tests/integration/billing_test.rs             │
│                                                 │
│ [Show Terminal Command]    [Cancel Ingestion]   │
└─────────────────────────────────────────────────┘

Click [Show Terminal Command] → Shows what's running:
```
$ parseltongue ingest ~/code/billing-service
# Auto-spawned server on port 7779
# Writing to: rocksdb:parseltongue20260217151630/analysis.db
# Port file: ~/.parseltongue/billing-service.port
```

When ingestion completes → Sidebar updates:

┌─────────────────────────────────────────────────┐
│ Projects                                        │
├─────────────────────────────────────────────────┤
│ ● my-service         :7777                      │
│ ● api-gateway        :7778                      │
│ ● billing-service    :7779  ✨ NEW              │ ← Just added
├─────────────────────────────────────────────────┤
│ [+ Add Project]                                 │
└─────────────────────────────────────────────────┘

Click billing-service → Shows stats immediately
```

### What Tauri DOES NOT Do (Important Boundaries)

#### ❌ Multi-Step Analysis Workflows
**Tauri does NOT**: Chain 5 queries, filter results with jq, feed into next query
**Instead**: Copy the workflow template, run in terminal with pipes

Example of what belongs in terminal, NOT in Tauri:
```bash
# This is a terminal workflow, not a GUI workflow
ENTITY=$(curl -s "localhost:7777/my-service/search?q=verify_token" | \
         jq -r '.entities[0].key')

curl -s "localhost:7777/my-service/blast-radius?entity=$ENTITY&hops=2" | \
  jq '.affected_entities[] | select(.via_trait != null) | .key' | \
  while read caller; do
    curl -s "localhost:7777/my-service/entity-detail?key=$caller"
  done | jq -s '.'
```

This is composable, scriptable, pipeable. GUIs can't do this elegantly.

#### ❌ LLM Prompt Composition
**Tauri does NOT**: Have a built-in LLM chat interface
**Instead**: Generate XML exports, user pastes into Claude/Cursor

Why: Claude's interface is better than anything we'd build. Don't compete. Integrate.

#### ❌ Batch Processing / CI Integration
**Tauri does NOT**: Have a "run 20 queries" batch mode
**Instead**: Generate bash scripts, user runs in CI

Example CI script (NOT in Tauri):
```bash
#!/bin/bash
# .github/workflows/architecture-gate.sh

# Parse PR changed files
CHANGED=$(git diff --name-only main...HEAD | grep '\.rs$')

for file in $CHANGED; do
  # Find entities in changed file
  ENTITIES=$(curl -s "localhost:7777/my-proj/search?file=$file" | jq -r '.[].key')
  
  for entity in $ENTITIES; do
    # Check blast radius
    RADIUS=$(curl -s "localhost:7777/my-proj/blast-radius?entity=$entity&hops=2" | \
             jq '.total_affected')
    
    if [ "$RADIUS" -gt 100 ]; then
      echo "❌ $entity affects $RADIUS entities (limit: 100)"
      exit 1
    fi
  done
done

echo "✅ All changes pass blast radius gate"
```

### Updated Tauri UI Mockup (Terminal-First Design)

```
┌──────────────────────────────────────────────────────────────┐
│  Parseltongue: my-service           [📋 Copy All Commands]   │ ← NEW: Copy everything
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  📊 Quick Stats                                              │
│  5,150 entities │ 18,942 edges │ 847K tokens │ Rust 98%     │
│                                                              │
│  ⚠️  Health Alerts                                           │
│  • 1 cycle violation in parser module          [Fix Guide]  │ ← Links to terminal workflow
│  • 2 God Objects (RequestHandler, Orchestrator) [Analyze]   │
│                                                              │
│  🔥 Top 5 Hotspots                      [Copy curl commands]│
│  1. verify_token (142 callers) ⚠️ HIGH [Terminal Workflow]  │ ← One-click workflow
│  2. db::execute (156 callers)           [Terminal Workflow]  │
│  3. log_event (284 callers)              [Terminal Workflow]  │
│  4. map_error (128 callers)              [Terminal Workflow]  │
│  5. user::from_row (97 callers)          [Terminal Workflow]  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ 🔍 Search: [__________________]   [🎯 Advanced Search] │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  Quick Actions:                                              │
│  [📊 6 Workflow Templates] [🧪 Test a Command] [📚 CLI Docs]│
└──────────────────────────────────────────────────────────────┘
```

Click [Terminal Workflow] next to verify_token:
```
┌──────────────────────────────────────────────────────────────┐
│  Terminal Workflow: Analyze verify_token                     │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  Step 1: Get entity details                                  │
│  $ curl localhost:7777/my-service/entity-detail?\\           │
│         key=rust:fn:verify_token                      [Copy] │
│                                                              │
│  Step 2: Who calls this?                                     │
│  $ curl localhost:7777/my-service/reverse-callers?\\         │
│         entity=rust:fn:verify_token                   [Copy] │
│                                                              │
│  Step 3: Blast radius (2 hops)                               │
│  $ curl localhost:7777/my-service/blast-radius?\\            │
│         entity=rust:fn:verify_token&hops=2            [Copy] │
│                                                              │
│  Step 4: Get LLM context (XML export)                        │
│  $ curl localhost:7777/my-service/entity-detail?\\           │
│         key=rust:fn:verify_token > verify_token.xml   [Copy] │
│                                                              │
│  Step 5: Ask Claude                                          │
│  Paste verify_token.xml into Claude and ask:                 │
│  "Is this function safe to refactor? Show me the risks."     │
│                                                              │
│  [📋 Copy All 4 Commands]  [🚀 Open Terminal with Commands]  │
└──────────────────────────────────────────────────────────────┘
```

### The Power-User Graduation Path

```
DAY 1 (Tauri only):
  User clicks around, sees stats, searches entities
  → "This is neat, but feels limited"

DAY 3 (Tauri → Terminal handoff):
  User clicks [Terminal Workflow] button
  Copies 5 curl commands to terminal
  Sees rich JSON outputs, pipes through jq
  → "Oh, there's way more data here than the GUI shows!"

DAY 7 (Terminal-first):
  User writes their own bash scripts
  Combines Parseltongue queries with git logs
  Feeds outputs into LLM prompts
  → "I don't need the GUI anymore, I have my own workflows"

DAY 30 (CI integration):
  User adds Parseltongue gates to CI pipeline
  Auto-generates architecture reports
  LLM reviews PRs with blast radius context
  → "Parseltongue is part of our dev infrastructure now"
```

**The Tauri app's job is to get users from Day 1 to Day 7.** By Day 30, they don't need it.

### Acceptance Criteria (Revised for Terminal-First Design)

```
WHEN user opens Tauri app for the first time
THEN they SHALL see stats + hotspots + health within 200ms
AND every entity SHALL have a [Terminal Workflow] button

WHEN user clicks [Terminal Workflow] for any entity
THEN Tauri SHALL display 4-5 copy-pasteable curl commands
AND each command SHALL include the correct project slug in URL [R5]
AND user SHALL be able to copy all commands with one click

WHEN user clicks [Copy] next to a command
THEN command SHALL be copied to clipboard exactly as shown
AND command SHALL be executable in terminal without modification
AND command SHALL use localhost:{port}/{slug}/endpoint format [R2, R5]

WHEN user clicks [📊 6 Workflow Templates]
THEN Tauri SHALL display README workflows as bash scripts
AND each template SHALL have [Copy All] button
AND copied script SHALL include error handling + jq formatting

WHEN user searches for an entity
THEN results SHALL show [View in Terminal] and [Copy curl command]
AND [View in Terminal] SHALL open terminal with command pre-filled (if OS supports it)
AND [Copy curl command] SHALL copy: curl localhost:7777/my-service/entity-detail?key=X

WHEN user clicks [🧪 Test a Command]
THEN Tauri SHALL have a command playground with curl → JSON preview
AND user SHALL see live query results without leaving GUI
AND this is for LEARNING, not for production workflows

WHEN user clicks [+ Add Project]
THEN Tauri SHALL show modal with drag-and-drop or browse option
AND user SHALL be able to drag folder onto drop zone
AND Tauri SHALL auto-detect project slug from folder name
AND user SHALL be able to edit slug before ingestion

WHEN user starts ingestion for new project
THEN Tauri SHALL spawn server with auto-assigned port [R2]
AND server SHALL write port file at ~/.parseltongue/{slug}.port [R6]
AND Tauri SHALL show live progress (entities, edges, files, speed)
AND progress SHALL update at least 4 times per second
AND user SHALL see [Show Terminal Command] to see equivalent CLI

WHEN ingestion completes for new project
THEN project SHALL appear in sidebar with green dot + port number
AND clicking project SHALL switch to its overview immediately
AND server SHALL remain running for future queries
AND Tauri SHALL reconnect to existing server on next launch

WHEN user wants to remove a project
THEN sidebar SHALL have [...] menu next to each project
AND menu SHALL offer: "Stop Server" | "Remove from List" | "Open Folder"
AND "Stop Server" SHALL run `parseltongue shutdown {slug}` [R3]
AND "Remove from List" SHALL remove from sidebar but keep server running
```

### What This Means for Implementation

**Don't build**:
- Complex graph visualizations (force-directed layouts, zoomable graphs)
- In-app LLM chat interface
- Multi-query wizards with 5 steps
- "Export to PDF" architecture reports

**Do build**:
- Fast stats dashboard (< 200ms load)
- Search with instant results
- One-click command copy buttons everywhere
- Workflow template library
- Simple command playground (curl tester)
- "Open in Terminal" integration (if OS allows)

**The Tauri app is a bridge to the CLI, not a destination.**

### Mapping to V200 Requirements

| Requirement | How Tauri Supports It |
|-------------|----------------------|
| R1: Route prefix nesting | All generated curl commands use `/{slug}/endpoint` format |
| R2: Auto port | Tauri discovers port via port file, generates `localhost:{port}` commands |
| R3: Shutdown command | Tauri has [Stop Server] button that runs `parseltongue shutdown {slug}` |
| R4: XML responses | [Export for LLM] buttons generate commands with XML output |
| R5: Project slug in URL | All commands include slug: `localhost:7777/my-service/...` |
| R6: Slug port file | Tauri reads `~/.parseltongue/{slug}.port` to find server |
| R7: Token count | Shown in stats, helps users decide if context fits in LLM window |
| R8: Data-flow queries | NOT in Tauri (CLI-only feature for power users) |

### Updated Section 1 Summary

**Tauri App Purpose**: Visual launcher for CLI commands
**Primary Users**: Day 1-7 learners, quick-lookup needs
**Power Users**: Graduate to pure CLI by Day 7
**Not For**: Multi-step analysis, LLM composition, CI automation (use terminal)

**Core Value**: Teaches users the CLI by showing them exactly which commands to run.
