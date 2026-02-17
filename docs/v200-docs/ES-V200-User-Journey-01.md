# ES-V200-User-Journey-01
Status: Draft v01
Purpose: Define end-to-end user journeys for V200 with executable acceptance criteria, organized by consumption mode and user intent.

## Document Philosophy

This document follows Shreyas Doshi's product thinking framework:
- **Impact over Activity**: What outcomes matter, not just what features ship
- **Jobs-to-be-Done**: Why users choose this tool over alternatives
- **Friction Mapping**: Where users get stuck and abandon
- **Outcome Metrics**: How we know the journey succeeded

Every journey section includes:
1. User context (who, why now, what they tried before)
2. Entry point (how they discovered/installed)
3. Core flow (step-by-step with decision points)
4. Success criteria (measurable outcomes)
5. Friction points (where they might fail)
6. Acceptance tests (WHEN/THEN/SHALL format per Design101)

---

## Section 1: Tauri Desktop Companion App Journey

### 1.1 User Context: The "Lost in Codebase" Moment

**Who**: Senior engineer joining a new team, tasked with understanding a 400K-line Rust codebase to implement a cross-cutting feature (e.g., adding observability to 50+ service endpoints).

**The moment**: It's 10 AM on Day 3. They've spent 6 hours reading docs, tracing `grep` results, and drawing diagrams on paper. They have 47 browser tabs open. They don't know where to start making changes without breaking something critical.

**What they tried**:
- `grep -r "endpoint"` → 2,347 matches, mostly noise
- VS Code "Find All References" → crashes on large workspace
- Reading architecture docs → outdated, mentions files that don't exist
- Asking teammates → everyone's in meetings

**Why Parseltongue**: They need a **spatial understanding** of the codebase's dependency graph, not just text search. They need to see "if I change this, what breaks?" They need **visual navigation** with **instant context**.

### 1.2 Discovery & Installation: Zero-Friction Entry

#### Journey Step 1: Discovery via Search or Referral

**Trigger**: User searches "rust codebase dependency graph tool" or teammate sends Slack message: "Just use Parseltongue, it saved me 8 hours last week."

**First Impression Requirement**: Landing page or GitHub README must communicate value in 10 seconds:
```
"See your entire codebase as a graph. Click any function to see what calls it, 
what it calls, and what breaks if you change it. Works offline. Zero config."
```

**Acceptance Criteria**:
```
WHEN user lands on parseltongue.dev or GitHub repo
THEN they SHALL see a 15-second demo GIF showing:
  - Ingest a codebase in 3 seconds
  - Search "handle_request"
  - Click entity → see blast radius visualization
  - Click "Show callers" → see 8 functions highlighted
AND the README SHALL include one-line install command
AND install time SHALL be <2 minutes from discovery to first graph
```

#### Journey Step 2: One-Command Install

**Current Pain**: Most dev tools require 5-10 steps (clone, build, configure, restart shell, etc.). Users abandon at step 3.

**V200 Requirement**: Single command install that works on macOS/Linux/Windows with zero prerequisites beyond having Rust projects.

```bash
# macOS/Linux
curl -sSL https://parseltongue.dev/install.sh | sh

# Windows
irm https://parseltongue.dev/install.ps1 | iex

# Cargo fallback
cargo install parseltongue
```

**What Happens**:
1. Downloads Tauri `.app` (macOS) or `.exe` (Windows) or `.AppImage` (Linux)
2. Installs to `/Applications` or `~/bin` or `C:\Program Files`
3. Registers `parseltongue` CLI command
4. Opens welcome screen: "Drag a project folder here to analyze"

**Acceptance Criteria**:
```
WHEN user runs install command
THEN Tauri app SHALL be functional within 120 seconds
AND CLI SHALL be available in new terminal session
AND app SHALL auto-launch with onboarding prompt
AND user SHALL NOT need to install Rust, Node, Python, or any runtime
```

[Requirements R2, R6 mapped: auto-port discovery means Tauri app can find running server]

### 1.3 First-Run Experience: Immediate Value

#### Journey Step 3: Ingest First Codebase

**User Action**: Drags project folder onto Tauri app window OR clicks "Open Project" OR uses CLI:

```bash
parseltongue ingest ~/code/my-service
```

**What User Sees** (real-time, no "please wait" spinner):

```
Tauri Window UI (Live Updates):
┌─────────────────────────────────────────────┐
│ Analyzing: ~/code/my-service                │
│                                             │
│ [████████████████░░░░] 82% (4,231/5,150)   │
│                                             │
│ 📊 Entities found:    4,231                 │
│ 🔗 Dependencies:     18,942                 │
│ 📁 Files processed:  1,053 / 1,284          │
│ ⚡ Speed:            ~4,000 entities/sec    │
│                                             │
│ Currently parsing:                          │
│   src/handlers/auth.rs                      │
│   src/models/user.rs                        │
│   tests/integration/api_test.rs             │
└─────────────────────────────────────────────┘
```

**Why This Matters** (Shreyas Doshi friction mapping):
- **Zero Configuration**: No `.parseltonguerc` file, no setup wizard
- **Instant Feedback**: Progress bar + live entity count keeps user engaged
- **Transparent Process**: Showing file names builds trust ("it's actually working")
- **Fast Enough**: 3-5 seconds for 5K-entity project feels instant

**Server Lifecycle** (invisible to user, critical for reliability):

1. Tauri app checks for running server via port file lookup [R6: slug-aware port file]
2. If no server: spawns `rust-llm-interface-gateway` with auto-assigned port [R2: auto-port]
3. Port written to `~/.parseltongue/{project-slug}.port` [R6]
4. Tauri reads port file, connects to HTTP server
5. Sends ingest request with project path + slug [R5: project slug in URL]

**Acceptance Criteria**:
```
WHEN user ingests codebase via Tauri app
THEN server SHALL auto-start if not running within 500ms [R2]
AND port file SHALL be created at ~/.parseltongue/{slug}.port [R6]
AND progress SHALL update at least 4 times per second
AND ingest SHALL complete in <5 seconds for 5K-entity codebase
AND UI SHALL NOT freeze during ingest
AND user SHALL see total entity/edge counts immediately after ingest
```

[Requirements R2, R5, R6, R7 mapped here]

#### Journey Step 4: First Query - "Show Me the Graph"

**User Mental Model**: "I ingested the codebase. Now what? I want to see the big picture."

**CRITICAL REFRAME**: The Tauri app is a **visual launcher for CLI commands**, not a replacement for terminal workflows. The real power is in the CLI. The app provides:
1. Quick visual overview (instant orientation)
2. One-click command generation (copy pasteable curl/CLI commands)
3. Basic navigation (find entities, see top-level metrics)
4. **NOT**: Deep analysis, LLM prompting, or multi-step workflows

**Why**: The three-layer architecture (pt04 + LLM + CPU) requires composing multiple queries, piping outputs, and LLM reasoning. That's a CLI workflow. The Tauri app is for "show me something quickly" moments, not "debug this complex issue" sessions.

**Tauri UI Defaults to Overview** (no blank screen):

```
┌───────────────────────────────────────────────────────────┐
│  Parseltongue: my-service                      [⚙️ Settings] │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  📊 Codebase Overview                                     │
│                                                           │
│  Total Entities:      5,150                               │
│  Total Dependencies:  18,942                              │
│  Languages:           Rust (98%), TOML (2%)               │
│  Tokens (est.):       847,293   [R7: token count shown]  │
│                                                           │
│  Top 5 Most-Called Functions:                             │
│    1. common::logging::log_event       (284 callers)      │
│    2. db::query::execute                (156 callers)      │
│    3. auth::verify_token                (142 callers)      │
│    4. handlers::error::map_error        (128 callers)      │
│    5. models::user::from_row            (97 callers)       │
│                                                           │
│  [🔍 Search]  [🎯 Navigate]  [📈 Analyze]  [💾 Export]   │
└───────────────────────────────────────────────────────────┘
```

**Why This Succeeds** (Jobs-to-be-Done lens):
- **Instant Orientation**: User sees scale immediately (5K entities = medium-sized service)
- **Actionable Intel**: Top 5 most-called = "these are critical, changing them is risky"
- **Token Count Visible**: LLM users immediately see if this fits in context window [R7]
- **Clear Next Actions**: Four buttons map to user intents (find, navigate, analyze, share)

**Acceptance Criteria**:
```
WHEN ingest completes
THEN Tauri SHALL display overview within 200ms
AND overview SHALL show entity count, edge count, languages, token count [R7]
AND top-N entities SHALL be ranked by in-degree (callers)
AND user SHALL see actionable next steps (Search/Navigate/Analyze buttons)
AND token count SHALL match persisted value from ingest [R7 persistence requirement]
```

### 1.4 Core Use Case: "What Calls This Function?"

#### Journey Step 5: Search for Entity

**User Intent**: "I need to change `auth::verify_token`. I want to see everything that depends on it before I break production."

**User Action**: Clicks Search box, types `verify_token`

**Tauri Fuzzy Search UI**:

```
┌─────────────────────────────────────────────┐
│ 🔍 Search: verify_token                     │
├─────────────────────────────────────────────┤
│ Matches (4):                                │
│                                             │
│ ✓ rust:fn:verify_token                      │
│   📄 src/auth/token.rs:45                   │
│   🔗 142 callers                            │
│                                             │
│ ✓ rust:fn:verify_token_expiry               │
│   📄 src/auth/token.rs:89                   │
│   🔗 3 callers                              │
│                                             │
│ ✓ rust:test:test_verify_token               │
│   📄 tests/unit/auth_test.rs:12             │
│   🔗 0 callers                              │
│                                             │
│ ✓ rust:struct:VerifyTokenRequest            │
│   📄 src/api/types.rs:203                   │
│   🔗 12 callers                             │
└─────────────────────────────────────────────┘
```

**User Clicks First Result** → Detail view loads

**Acceptance Criteria**:
```
WHEN user searches entity name
THEN results SHALL appear within 100ms (graph query, not grep)
AND results SHALL show entity type, file path, line number, caller count
AND results SHALL rank by relevance (exact match > prefix match > fuzzy match)
AND user SHALL be able to click result to see detail view
```

#### Journey Step 6: Blast Radius Visualization

**User Clicks** "rust:fn:verify_token" → Tauri loads detail view:

```
┌──────────────────────────────────────────────────────────────┐
│  rust:fn:verify_token                         [↩️ Back] [💾 Export] │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  📄 Location: src/auth/token.rs:45-67                        │
│  📦 Module:   auth::token                                    │
│  🏷️  Type:     Function (public)                             │
│                                                              │
│  📊 Impact Analysis:                                         │
│    Direct Callers:     142 functions                         │
│    Transitive Impact:  847 functions (within 3 hops)         │
│    ⚠️  Risk Level:      HIGH (top 3% of codebase)            │
│                                                              │
│  🎯 Top 10 Callers:                                          │
│    [Graph Visualization: Force-directed layout]              │
│    [verify_token] ──→ handlers::auth::login (89 paths)       │
│                  ──→ handlers::api::protected (42 paths)     │
│                  ──→ middleware::auth_check (11 paths)       │
│    ...                                                       │
│                                                              │
│  [Show Source Code]  [Blast Radius (Hops: 2 ▼)]             │
└──────────────────────────────────────────────────────────────┘
```

**Critical Feature** [R4: XML-tagged responses for LLM consumption]:

**Export Button** → generates LLM-ready context:

```xml
<code_context>
  <entity>
    <key>rust:fn:verify_token:auth/token.rs:T1234567890</key>
    <type>function</type>
    <impact_score>0.94</impact_score>
    <callers count="142">
      <caller>
        <key>rust:fn:login:handlers/auth.rs:T9876543210</key>
        <call_count>89</call_count>
        <path_length>1</path_length>
      </caller>
      <!-- ... -->
    </callers>
  </entity>
  <source_code file="src/auth/token.rs" lines="45-67">
    <![CDATA[
pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, AuthError> {
    let validation = Validation::new(Algorithm::HS256);
    // ... (actual source lines from disk)
    ]]>
  </source_code>
</code_context>
```

**Why XML Tags** [R4 deep dive]:
- **Structured for LLMs**: Claude/GPT can parse `<entity>`, `<callers>`, `<source_code>` reliably
- **Self-Describing**: No schema doc needed, tags explain themselves
- **Paste-Ready**: User copies this, pastes in Claude, asks "Is this safe to refactor?"
- **Token-Efficient**: Nested structure = fewer delimiter tokens than JSON

**Acceptance Criteria**:
```
WHEN user views entity detail
THEN detail SHALL show file path, line range, entity type, module path
AND blast radius SHALL compute within 500ms for 2-hop queries
AND impact score SHALL be normalized 0.0-1.0 (percentile rank by in-degree)
AND user SHALL see top 10 callers with call counts
AND "Show Source Code" SHALL fetch current disk content [G3: filesystem source-read contract]
AND Export SHALL generate XML-tagged response with entity metadata + callers + source [R4]
AND XML structure SHALL be parseable by LLM without schema
```

[Requirements G3, R4 mapped: filesystem read contract + XML responses]

### 1.5 Power User Flow: Multi-Project Navigation

#### Journey Step 7: Working Across Multiple Projects

**User Context**: Senior architect reviewing a microservices migration. They have 6 services open, each with its own dependency graph.

**V200 Multi-Project UX**:

**Tauri Sidebar** (persistent across sessions):

```
┌─────────────────────┐
│ Projects            │
├─────────────────────┤
│ ● auth-service      │  ← Green dot = server running
│   :7777             │     [R5: project slug visible]
│                     │
│ ● api-gateway       │
│   :7778             │
│                     │
│ ○ billing-service   │  ← Gray dot = not analyzed yet
│                     │
│ ● user-service      │
│   :7779             │
├─────────────────────┤
│ [+ Add Project]     │
└─────────────────────┘
```

**Why Slug-Based Architecture Matters** [R5, R6 rationale]:

**Without Slugs** (naive approach):
- User opens `auth-service` → server on port 7777
- User opens another terminal, ingests `api-gateway` → FAILS, port 7777 in use
- User manually kills server, restarts with `--port 7778`
- User forgets which port is which
- Chaos

**With Slugs** (V200 design):
- `auth-service` auto-slugs to `auth-service`, port 7777, writes `~/.parseltongue/auth-service.port`
- `api-gateway` auto-slugs to `api-gateway`, port 7778, writes `~/.parseltongue/api-gateway.port`
- Tauri discovers both via port files [R6]
- URLs are self-describing: `http://localhost:7777/auth-service/query` [R5]
- Server logs show `[auth-service]` prefix, no confusion

**Acceptance Criteria**:
```
WHEN user ingests multiple projects
THEN each project SHALL receive unique slug derived from folder name [R5]
AND each project SHALL receive unique auto-assigned port [R2]
AND port files SHALL be named ~/.parseltongue/{slug}.port [R6]
AND Tauri SHALL discover all running servers via port file scan
AND project URLs SHALL include slug: /{slug}/endpoint [R5]
AND switching projects in Tauri SHALL be instant (<100ms tab switch)
AND server logs SHALL prefix messages with [slug] for disambiguation
```

[Requirements R2, R5, R6 all connected: auto-port + slug in URL + slug-aware port file]

### 1.6 End-of-Session: Graceful Shutdown

#### Journey Step 8: Closing the App

**User Action**: Quits Tauri app via Cmd+Q (macOS) or closes window.

**What Should Happen** [R3: shutdown CLI command requirement]:

**Option 1: Leave Servers Running** (default, for next session speed)
- Tauri exits
- Backend servers stay alive
- Next launch reconnects instantly via port files

**Option 2: Clean Shutdown** (user preference or system shutdown)
- Tauri sends shutdown command to each server [R3]
- Servers flush any pending writes
- Servers delete port files
- Servers exit with code 0

**CLI Equivalent**:
```bash
# Shutdown specific project
parseltongue shutdown auth-service

# Shutdown all
parseltongue shutdown --all
```

**Why This Matters** (reliability lens):
- **No Orphaned Processes**: User doesn't accumulate zombie servers over weeks
- **Clean Restarts**: System reboots don't leave stale port files
- **Explicit Control**: Power users can kill servers without `pkill`

**Acceptance Criteria**:
```
WHEN user quits Tauri app
THEN servers SHALL continue running by default (persistent mode)
AND Tauri SHALL reconnect to existing servers on next launch

WHEN user runs "parseltongue shutdown {slug}"
THEN server SHALL flush pending writes
AND server SHALL delete ~/.parseltongue/{slug}.port
AND server SHALL exit with code 0 within 2 seconds [R3]
AND CLI SHALL confirm shutdown: "auth-service stopped"

WHEN system shuts down
THEN servers SHALL trap SIGTERM
AND servers SHALL delete port files before exit
AND no stale .port files SHALL remain after reboot
```

[Requirement R3 mapped: graceful shutdown contract]

### 1.7 Tauri App Success Metrics

**Leading Indicators** (user engagement):
- Time to first graph: <120 seconds from install
- Entities searched per session: >5 (engaged users explore)
- Export clicks per week: >3 (using LLM integration)
- Multi-project usage: >30% of users have 2+ projects

**Lagging Indicators** (outcome):
- "Saved me from a bad refactor": qualitative feedback
- Reduced code review time: team metric (hard to measure)
- Adoption in new-hire onboarding: 60%+ of teams using PT for onboarding

---

## Section 2: CLI Power User Journey
[TO BE WRITTEN: Covers developers who prefer terminal workflows, CI/CD integration, scripting]

## Section 3: MCP Client Journey (Claude/Cursor/Copilot)
[TO BE WRITTEN: Covers LLM-as-user, agent workflows, context injection]

## Section 4: HTTP API Journey (Custom Tooling)
[TO BE WRITTEN: Covers teams building internal dashboards, bots, automation]

---

## Appendix A: Cross-Journey Requirements Mapping

```
+----+--------------------------------------+-------------------+----------------------------+
| ID | Requirement                          | Mapped Journeys   | Acceptance Criteria Count  |
+----+--------------------------------------+-------------------+----------------------------+
| R1 | Route prefix nesting                 | Tauri 1.5, 1.7    | 2 criteria                 |
| R2 | Auto port + port file lifecycle      | Tauri 1.3, 1.5    | 4 criteria                 |
| R3 | Shutdown CLI command                 | Tauri 1.6         | 3 criteria                 |
| R4 | XML-tagged responses                 | Tauri 1.4         | 2 criteria                 |
| R5 | Project slug in URL                  | Tauri 1.5, 1.7    | 4 criteria                 |
| R6 | Slug-aware port file naming          | Tauri 1.3, 1.5    | 3 criteria                 |
| R7 | Token count at ingest                | Tauri 1.3, 1.4    | 3 criteria                 |
| R8 | Data-flow tree-sitter queries        | [Section 2: CLI]  | [Pending]                  |
| G3 | Filesystem source-read contract      | Tauri 1.4         | 1 criterion                |
+----+--------------------------------------+-------------------+----------------------------+
```

## Appendix B: Friction Point Catalog

**Identified friction points from user journey mapping**:

1. **Install Abandonment**: If install takes >2 minutes or requires prerequisites, 60% drop-off
   - **Mitigation**: Single-command install, pre-built binaries, zero config

2. **Blank Screen After Ingest**: If user sees no immediate value, they close the app
   - **Mitigation**: Auto-load overview with stats, show top entities, suggest actions

3. **Slow Queries**: If blast radius takes >2 seconds, feels broken
   - **Mitigation**: Pre-computed graph, CozoDB indexing, 500ms query budget

4. **Port Conflicts**: If second project fails with "port in use", user gives up
   - **Mitigation**: Auto-port assignment, slug-based namespacing, port file discovery

5. **Stale Port Files**: If reboot leaves .port files, next launch connects to nothing
   - **Mitigation**: SIGTERM handler, cleanup on shutdown, port file validation

6. **LLM Context Copy-Paste**: If export is JSON, LLMs hallucinate field names
   - **Mitigation**: XML-tagged responses with self-describing structure

## Appendix C: Design101 Acceptance Criteria Summary

Total acceptance criteria defined: **21**
- Tauri App journeys: 21 criteria across 8 journey steps
- Pending (CLI/MCP/HTTP): ~40 additional criteria estimated

All criteria follow WHEN/THEN/SHALL format per Design101 executable specifications principle.
