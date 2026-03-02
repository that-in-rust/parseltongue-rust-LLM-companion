# Parseltongue Conversational MCP - Moment to Moment Simulation

## The Core Idea

```
OLD WAY: Tool returns raw data → Agent interprets → Agent asks user
NEW WAY: Tool returns data + context + recommendations + choices → Agent presents naturally
```

---

## MOMENT 0: Session Start

**User opens Claude Code in a Rust project:**

```
$ cd my-rust-auth-service
$ claude

╭─────────────────────────────────────────────────────────────╮
│  Claude Code                                                 │
│  Model: claude-sonnet-4-20250514                            │
│  Context: ./ (my-rust-auth-service)                         │
╰─────────────────────────────────────────────────────────────╯

USER: I want to refactor the authentication module

CLAUDE: Let me analyze the codebase structure first...

[Tool Call: LS, Read files...]
```

---

## MOMENT 1: Agent Discovers Parseltongue

**Agent's internal reasoning (visible in thinking):**

```
I should understand this Rust codebase before making changes.
Let me check if there are any tools available for code analysis...

[MCP Tool Discovery: parseltongue_* tools detected]

Available tools:
- parseltongue_status - Check if project is indexed
- parseltongue_search - Semantic code search
- parseltongue_blast_radius - Impact analysis
- parseltongue_context - Generate LLM-optimized context
- parseltongue_complexity - Complexity metrics

Let me check the project status first.
```

---

## MOMENT 2: First Parseltongue Call

**Agent calls:**
```
parseltongue_status()
```

**Response (what agent receives):**

```
┌─────────────────────────────────────────────────────────────┐
│  📊 PARSELTONGUE STATUS                                      │
├─────────────────────────────────────────────────────────────┤
│  Project: my-rust-auth-service                               │
│  Language: Rust                                              │
│  Index Status: ✅ Indexed (2 hours ago)                      │
│                                                              │
│  Statistics:                                                 │
│    • 847 entities (functions, structs, traits)              │
│    • 2,341 dependency relations                             │
│    • 156 file modules                                        │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  💡 SUGGESTED NEXT ACTIONS                             │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  For refactoring:                                      │  │
│  │    → parseltongue_blast_radius("auth")                 │  │
│  │    → parseltongue_complexity("--path", "src/auth")     │  │
│  │                                                        │  │
│  │  For exploration:                                      │  │
│  │    → parseltongue_context("authentication flow")       │  │
│  │    → parseltongue_search("login", limit=10)            │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  Type: "help" for full command reference                    │
└─────────────────────────────────────────────────────────────┘
```

**What agent does with this:**

```
The tool suggests using blast_radius for refactoring. Since the user
wants to refactor auth, I should analyze the impact first.
```

**Agent responds to user:**

```
I found a code analysis tool (Parseltongue) that's already indexed 
this project. Let me analyze the authentication module's dependencies 
before we proceed.
```

---

## MOMENT 3: Blast Radius Analysis

**Agent calls:**
```
parseltongue_blast_radius("auth::login")
```

**Response:**

```
┌─────────────────────────────────────────────────────────────┐
│  💥 BLAST RADIUS ANALYSIS: auth::login                       │
│  Query time: 47ms                                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  EPICENTER:                                                 │
│    auth::login (src/auth/login.rs:45-89)                    │
│    Signature: fn login(user: &str, pass: &str) -> Result<Token, Error>
│                                                              │
│  DIRECT IMPACT (distance=1) - Will break immediately:       │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ 🔴 auth::session::create                                ││
│  │    src/auth/session.rs:23                               ││
│  │    Calls: login() and expects Token return type         ││
│  │                                                         ││
│  │ 🔴 middleware::auth_check                               ││
│  │    src/middleware/auth.rs:67                            ││
│  │    Calls: login() for initial authentication            ││
│  │                                                         ││
│  │ 🟡 api::handlers::login_route                           ││
│  │    src/api/routes/auth.rs:112                           ││
│  │    Wraps: login() for HTTP endpoint                     ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  TRANSITIVE IMPACT (distance=2-3) - May break:              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ 🟡 api::handlers::refresh_route (d=2)                   ││
│  │ 🟡 cli::commands::login (d=2)                           ││
│  │ 🟢 websocket::authenticate (d=3)                        ││
│  │ 🟢 grpc::auth_service (d=3)                             ││
│  │ ... 12 more transitive dependencies                     ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  SUMMARY:                                                    │
│    Total affected entities: 18                               │
│    Files touched: 7                                          │
│    Risk level: ⚠️  HIGH                                      │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  ⚡ RISK ASSESSMENT                                     │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  This change has HIGH risk because:                    │  │
│  │    • 3 direct callers will break immediately           │  │
│  │    • 15 indirect callers may have type mismatches      │  │
│  │    • Changes propagate across 3 architectural layers   │  │
│  │                                                        │  │
│  │  Recommended approach:                                 │  │
│  │    1. Check complexity of auth::login first            │  │
│  │    2. Add characterization tests                       │  │
│  │    3. Make incremental, backward-compatible changes    │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  🎯 NEXT STEPS - What would you like to do?            │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  [A] Check complexity of auth::login                   │  │
│  │  [B] Show dependency graph (visualization)             │  │
│  │  [C] Generate test coverage report                     │  │
│  │  [D] Create incremental refactoring plan               │  │
│  │  [E] Proceed anyway (not recommended)                  │  │
│  │                                                        │  │
│  │  💡 Recommendation: Start with [A] to understand       │  │
│  │     the code complexity before planning changes.       │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Agent presents this to user:**

```
## Impact Analysis: auth::login

Changing `auth::login` would affect **18 functions** across 7 files.

**Direct impact (will break):**
- `auth::session::create` - calls login()
- `middleware::auth_check` - uses for initial auth
- `api::handlers::login_route` - wraps for HTTP

**Risk level: HIGH** ⚠️

This is a core function with deep dependencies across multiple layers.

What would you like to do?

A) Check complexity first (recommended)
B) See dependency graph
C) Generate test coverage  
D) Create step-by-step plan
E) Proceed anyway
```

---

## MOMENT 4: User Chooses Complexity Check

**User types:**
```
A
```

**Agent calls:**
```
parseltongue_complexity("--path", "src/auth", "--top", "10")
```

**Response:**

```
┌─────────────────────────────────────────────────────────────┐
│  📈 COMPLEXITY ANALYSIS: src/auth/                           │
│  Query time: 23ms                                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  TOP 10 MOST COMPLEX FUNCTIONS:                              │
│                                                              │
│  Rank │ Function           │ Score │ Cyclomatic │ Nesting │ │
│  ─────┼────────────────────┼───────┼────────────┼─────────│ │
│   1   │ auth::login        │ 89.2  │    12      │    4    │ │
│   2   │ auth::validate_jwt │ 76.1  │     9      │    5    │ │
│   3   │ auth::refresh      │ 71.4  │     8      │    3    │ │
│   4   │ auth::session_new  │ 54.2  │     6      │    3    │ │
│   5   │ auth::hash_pass    │ 42.1  │     4      │    2    │ │
│   6   │ auth::check_rate   │ 38.7  │     4      │    2    │ │
│   7   │ auth::log_attempt  │ 31.2  │     3      │    2    │ │
│   8   │ auth::cleanup      │ 28.4  │     2      │    1    │ │
│   9   │ auth::session_del  │ 22.1  │     2      │    1    │ │
│  10   │ auth::is_expired   │ 18.9  │     1      │    1    │ │
│                                                              │
│  SCORE INTERPRETATION:                                       │
│    0-30:   Low complexity, safe to change                    │
│    31-60:  Medium complexity, moderate care needed           │
│    61-80:  High complexity, significant testing required     │
│    81-100: Very high complexity, consider refactoring first  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  🔍 DETAILED ANALYSIS: auth::login (Score: 89.2)       │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  Location: src/auth/login.rs:45-89 (44 lines)         │  │
│  │                                                        │  │
│  │  Issues detected:                                      │  │
│  │    • Cyclomatic complexity: 12 (threshold: 10)        │  │
│  │      → Too many decision paths (if/match/loops)       │  │
│  │                                                        │  │
│  │    • Nesting depth: 4 (threshold: 3)                  │  │
│  │      → Deeply nested conditionals                      │  │
│  │                                                        │  │
│  │    • Parameter count: 5 (threshold: 4)                │  │
│  │      → Consider using a config struct                  │  │
│  │                                                        │  │
│  │  Suggested refactorings:                               │  │
│  │    1. Extract: validate_credentials() -> bool          │  │
│  │    2. Extract: create_session() -> Session             │  │
│  │    3. Extract: log_login_attempt() -> ()               │  │
│  │    4. Consider: LoginConfig struct for parameters      │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  💡 REFACTORING STRATEGY                               │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  Based on complexity + blast radius:                   │  │
│  │                                                        │  │
│  │  SAFEST APPROACH (recommended):                        │  │
│  │    Phase 1: Start with simpler functions               │  │
│  │      → auth::hash_pass (score: 42) - low blast radius  │  │
│  │      → auth::cleanup (score: 28) - isolated            │  │
│  │                                                        │  │
│  │    Phase 2: Add tests for auth::login                  │  │
│  │      → Characterization tests first                    │  │
│  │      → Then refactor incrementally                     │  │
│  │                                                        │  │
│  │    Phase 3: Extract functions from login               │  │
│  │      → One extraction at a time                        │  │
│  │      → Run tests after each change                     │  │
│  │                                                        │  │
│  │  RISKIER APPROACH (not recommended):                   │  │
│  │    Directly modify auth::login                         │  │
│  │    → High chance of breaking 18 downstream functions   │  │
│  │    → Difficult to debug due to complexity              │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  🎯 WHAT NEXT?                                         │  │
│  ├───────────────────────────────────────────────────────┤  │
│  │  [A] Start with simpler functions (Phase 1)            │  │
│  │      → I'll guide you through refactoring hash_pass    │  │
│  │                                                        │  │
│  │  [B] Generate characterization tests for auth::login   │  │
│  │      → Lock in current behavior before changes         │  │
│  │                                                        │  │
│  │  [C] Show step-by-step extraction plan for login       │  │
│  │      → Detailed breakdown of what to extract           │  │
│  │                                                        │  │
│  │  [D] Generate context for understanding the code       │  │
│  │      → Token-efficient summary of auth flow            │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Agent presents:**

```
## Complexity Analysis

`auth::login` is the **most complex** function (score: 89.2 / 100)

**Issues:**
- 12 decision paths (cyclomatic) - should be ≤10
- 4 levels of nesting - should be ≤3  
- 5 parameters - should be ≤4

**Suggested extractions:**
1. `validate_credentials()` - reduce complexity
2. `create_session()` - separate concerns
3. `log_login_attempt()` - isolate side effects

**Strategy options:**

A) Start with simpler functions first (recommended)
   - Refactor `hash_pass` (score: 42) as warmup
   
B) Generate tests for `auth::login` first
   - Lock in current behavior

C) Show extraction plan
   - Step-by-step breakdown

D) Get token-efficient context
   - Understand the auth flow
```

---

## MOMENT 5: User Wants Context

**User types:**
```
D - show me the auth flow
```

**Agent calls:**
```
parseltongue_context("authentication flow", token_budget=4096)
```

**Response:**

```
┌─────────────────────────────────────────────────────────────┐
│  📋 CONTEXT: authentication flow                             │
│  Generated: 847 tokens (budget: 4096, 21% used)             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ENTRY POINTS:                                               │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ // HTTP API                                             ││
│  │ POST /login → api::handlers::login_route()              ││
│  │   └→ auth::login(user: &str, pass: &str) → Token        ││
│  │       ├→ auth::validate_credentials(user, pass) → bool  ││
│  │       │   └→ auth::hash_pass(input) → Hash             ││
│  │       ├→ auth::session::create(user_id) → Session      ││
│  │       └→ auth::log_attempt(user, success) → ()         ││
│  │                                                         ││
│  │ // CLI                                                   ││
│  │ $ login → cli::commands::login()                        ││
│  │   └→ auth::login(...) [same as above]                   ││
│  │                                                         ││
│  │ // WebSocket                                             ││
│  │ WS connect → websocket::authenticate()                  ││
│  │   └→ auth::validate_jwt(token) → Claims                ││
│  │       └→ auth::refresh(token) → Token                   ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  KEY TYPES:                                                  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ pub struct Token {                                      ││
│  │     access: String,      // JWT access token           ││
│  │     refresh: String,     // Refresh token              ││
│  │     expires_at: u64,     // Unix timestamp             ││
│  │ }                                                       ││
│  │                                                         ││
│  │ pub struct Session {                                    ││
│  │     id: Uuid,            // Session identifier         ││
│  │     user_id: Uuid,       // User reference             ││
│  │     created_at: DateTime<Utc>,                         ││
│  │ }                                                       ││
│  │                                                         ││
│  │ pub trait AuthProvider {                                ││
│  │     fn login(&self, creds: Credentials) -> Result<Tok>;││
│  │     fn refresh(&self, token: &str) -> Result<Token>;   ││
│  │     fn validate(&self, token: &str) -> Result<Claims>; ││
│  │ }                                                       ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  📊 METRICS                                             ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  Entities included: 8 (of 847 in project)              ││
│  │  Functions: 5 signatures (no bodies)                   ││
│  │  Types: 3 struct/trait definitions                     ││
│  │  Call paths: 2 (API, CLI)                              ││
│  │                                                         ││
│  │  Token savings: ~85%                                    ││
│  │  (Full files would be ~5,600 tokens)                   ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  💡 KEY INSIGHTS                                        ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  1. auth::login is called by 3 different entry points  ││
│  │     → Any signature change affects all three           ││
│  │                                                         ││
│  │  2. Session creation is tightly coupled to login       ││
│  │     → Consider extracting to separate service          ││
│  │                                                         ││
│  │  3. AuthProvider trait allows swapping implementations ││
│  │     → Could add MFA provider without touching login    ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  🎯 NOW YOU UNDERSTAND THE FLOW. WHAT NEXT?            ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  [A] Create refactoring plan                            ││
│  │      → Step-by-step changes with safety checks         ││
│  │                                                         ││
│  │  [B] Generate tests first                               ││
│  │      → Characterization tests for login                ││
│  │                                                         ││
│  │  [C] Start refactoring                                  ││
│  │      → Begin with simpler functions                    ││
│  │                                                         ││
│  │  [D] Ask more questions                                 ││
│  │      → I can analyze specific parts deeper             ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

**Agent presents:**

```
## Authentication Flow Summary

```
POST /login ──┐
              ├──→ auth::login() ──→ Token
$ login ──────┘        │
                       ├── validate_credentials()
                       ├── session::create()  
                       └── log_attempt()
```

**Key insight:** `auth::login` has 3 callers (API, CLI, eventually WebSocket).
Any signature change breaks all of them.

**Token savings:** 847 tokens vs ~5,600 for full files (85% reduction)

What next?

A) Create step-by-step refactoring plan
B) Generate characterization tests first
C) Start with simpler functions
D) Dig deeper into something specific
```

---

## MOMENT 6: User Wants a Plan

**User types:**
```
A - create a plan
```

**Agent calls:**
```
parseltongue_plan(
  target="auth::login",
  goal="reduce complexity from 89 to <60",
  constraints=["backward-compatible", "incremental", "tested"]
)
```

**Response:**

```
┌─────────────────────────────────────────────────────────────┐
│  📝 REFACTORING PLAN: auth::login                            │
│  Goal: Reduce complexity 89 → <60                            │
│  Estimated: 4-6 sessions                                     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  PHASE 0: PREPARATION (Session 1)                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ □ Step 0.1: Create test file                            ││
│  │     File: tests/auth_login_characterization.rs          ││
│  │     Action: Generate tests from current behavior        ││
│  │     Verify: cargo test auth_login                       ││
│  │                                                         ││
│  │ □ Step 0.2: Add mutation tests                          ││
│  │     Tool: cargo mutagen                                 ││
│  │     Action: Verify tests catch bugs                     ││
│  │                                                         ││
│  │ □ Step 0.3: Document current API contract               ││
│  │     File: docs/auth-api-contract.md                     ││
│  │     Action: Record inputs, outputs, edge cases          ││
│  │                                                         ││
│  │ ✓ Checkpoint: All tests pass, contract documented       ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  PHASE 1: WARMUP - Simpler Functions (Session 2)             │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ □ Step 1.1: Refactor auth::hash_pass (score: 42)        ││
│  │     Change: Add const generic for salt length           ││
│  │     Blast radius: 1 (only login uses it)                ││
│  │     Risk: LOW                                            ││
│  │     Verify: cargo test hash_pass                        ││
│  │                                                         ││
│  │ □ Step 1.2: Refactor auth::cleanup (score: 28)          ││
│  │     Change: Extract to trait method                     ││
│  │     Blast radius: 0 (isolated)                          ││
│  │     Risk: VERY LOW                                       ││
│  │     Verify: cargo test cleanup                          ││
│  │                                                         ││
│  │ ✓ Checkpoint: Simpler functions refactored, tests pass  ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  PHASE 2: EXTRACT FROM LOGIN (Sessions 3-4)                  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ □ Step 2.1: Extract validate_credentials()               ││
│  │     From: auth::login lines 52-61                       ││
│  │     To: auth::validate_credentials(user, pass) → bool   ││
│  │     Impact: Reduces cyclomatic by ~4                    ││
│  │     Blast radius: 1 (internal to login)                 ││
│  │     Risk: MEDIUM - needs careful testing                ││
│  │     Verify: cargo test validate_credentials             ││
│  │                                                         ││
│  │ □ Step 2.2: Extract create_session()                    ││
│  │     From: auth::login lines 63-71                       ││
│  │     To: auth::session::create_from_login(user) → Sess   ││
│  │     Impact: Reduces cyclomatic by ~2                    ││
│  │     Blast radius: 1 (internal to login)                 ││
│  │     Risk: MEDIUM                                         ││
│  │     Verify: cargo test create_session                   ││
│  │                                                         ││
│  │ □ Step 2.3: Extract log_login_attempt()                 ││
│  │     From: auth::login lines 73-78                       ││
│  │     To: auth::log::login_attempt(user, success) → ()    ││
│  │     Impact: Reduces nesting by 1                        ││
│  │     Blast radius: 1 (internal to login)                 ││
│  │     Risk: LOW - pure side effect                        ││
│  │     Verify: cargo test log_login_attempt                ││
│  │                                                         ││
│  │ ✓ Checkpoint: New complexity ~65, all tests pass        ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  PHASE 3: CLEANUP (Session 5)                                │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ □ Step 3.1: Reduce parameter count                      ││
│  │     Create: struct LoginConfig { user, pass, opts }     ││
│  │     Update: auth::login(config: LoginConfig)            ││
│  │     Impact: Better ergonomics, easier to extend         ││
│  │     Blast radius: 18 (all callers)                      ││
│  │     Risk: HIGH - many files to update                   ││
│  │     Verify: cargo test --all                            ││
│  │                                                         ││
│  │ □ Step 3.2: Final complexity check                      ││
│  │     Target: Score < 60                                  ││
│  │     Tool: parseltongue_complexity("auth::login")        ││
│  │                                                         ││
│  │ ✓ Checkpoint: Goal achieved, full test suite passes     ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  📊 PLAN SUMMARY                                        ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  Total steps: 9                                         ││
│  │  Estimated sessions: 4-6                                ││
│  │  Risk progression: LOW → MEDIUM → HIGH                  ││
│  │                                                         ││
│  │  Success metrics:                                       ││
│  │    • Complexity: 89 → <60 (33% reduction)              ││
│  │    • Cyclomatic: 12 → <8                               ││
│  │    • Nesting: 4 → 2                                    ││
│  │    • Test coverage: Current → 100% for affected code   ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  🎯 READY TO START?                                     ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  [A] Start Phase 0 - Create characterization tests      ││
│  │      → I'll generate the test file for you              ││
│  │                                                         ││
│  │  [B] Modify the plan                                    ││
│  │      → Adjust phases, steps, or priorities              ││
│  │                                                         ││
│  │  [C] Save plan for later                                ││
│  │      → Export to docs/refactor-plan-auth-login.md       ││
│  │                                                         ││
│  │  [D] Ask more questions                                 ││
│  │      → I can explain any step in detail                 ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

**Agent presents:**

```
## Refactoring Plan: auth::login

**Goal:** 89 → <60 complexity (33% reduction)
**Time:** 4-6 sessions

### Phase 0: Preparation
- Generate characterization tests
- Document API contract

### Phase 1: Warmup (simpler functions)
- Refactor `hash_pass` (low risk)
- Refactor `cleanup` (very low risk)

### Phase 2: Extract from login
- Extract `validate_credentials()` - saves 4 cyclomatic
- Extract `create_session()` - saves 2 cyclomatic
- Extract `log_login_attempt()` - saves 1 nesting level

### Phase 3: Cleanup
- Create `LoginConfig` struct (high risk - 18 callers)
- Final complexity verification

Ready to start?

A) Start Phase 0 - create tests now
B) Modify the plan
C) Save for later
D) Ask questions
```

---

## MOMENT 7: User Starts Phase 0

**User types:**
```
A - let's start
```

**Agent calls:**
```
parseltongue_generate_tests(
  target="auth::login",
  style="characterization",
  coverage="edge-cases"
)
```

**Response:**

```
┌─────────────────────────────────────────────────────────────┐
│  🧪 TEST GENERATION: auth::login                             │
│  Style: Characterization (captures current behavior)         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Generated file: tests/auth_login_characterization.rs        │
│  Test count: 12                                              │
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │ // tests/auth_login_characterization.rs                 ││
│  │                                                         ││
│  │ use crate::auth::login;                                 ││
│  │                                                         ││
│  │ /// HAPPY PATH TESTS                                    ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_valid_credentials() {                     ││
│  │     let result = login("alice", "correct_password");    ││
│  │     assert!(result.is_ok());                            ││
│  │     let token = result.unwrap();                        ││
│  │     assert!(!token.access.is_empty());                  ││
│  │     assert!(!token.refresh.is_empty());                 ││
│  │ }                                                       ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_returns_valid_jwt() {                     ││
│  │     let result = login("alice", "correct_password");    ││
│  │     // JWT should have 3 parts separated by dots       ││
│  │     assert_eq!(result.unwrap().access.matches('.').count(), 2);│
│  │ }                                                       ││
│  │                                                         ││
│  │ /// ERROR CASE TESTS                                    ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_wrong_password() {                        ││
│  │     let result = login("alice", "wrong_password");      ││
│  │     assert!(result.is_err());                           ││
│  │     // Current behavior: returns AuthError::InvalidCredentials│
│  │ }                                                       ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_nonexistent_user() {                      ││
│  │     let result = login("ghost", "any_password");        ││
│  │     assert!(result.is_err());                           ││
│  │     // Current behavior: returns AuthError::UserNotFound││
│  │ }                                                       ││
│  │                                                         ││
│  │ /// EDGE CASE TESTS                                     ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_empty_username() {                        ││
│  │     let result = login("", "password");                 ││
│  │     assert!(result.is_err());                           ││
│  │ }                                                       ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_empty_password() {                        ││
│  │     let result = login("alice", "");                    ││
│  │     assert!(result.is_err());                           ││
│  │ }                                                       ││
│  │                                                         ││
│  │ #[test]                                                 ││
│  │ fn test_login_sql_injection_attempt() {                 ││
│  │     let result = login("admin'--", "anything");         ││
│  │     assert!(result.is_err());                           ││
│  │     // Verify: No SQL error exposed to user             ││
│  │ }                                                       ││
│  │                                                         ││
│  │ // ... 5 more tests                                     ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  ▶ RUN TESTS                                            ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  $ cargo test auth_login_characterization               ││
│  │                                                         ││
│  │  Running 12 tests...                                    ││
│  │  test test_login_valid_credentials ... ok               ││
│  │  test test_login_returns_valid_jwt ... ok               ││
│  │  test test_login_wrong_password ... ok                  ││
│  │  test test_login_nonexistent_user ... ok                ││
│  │  test test_login_empty_username ... ok                  ││
│  │  test test_login_empty_password ... ok                  ││
│  │  test test_login_sql_injection_attempt ... ok           ││
│  │  ... 5 more ...                                         ││
│  │                                                         ││
│  │  ✅ 12/12 passed (0 failed)                             ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  📋 PHASE 0 CHECKPOINT                                  ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  ✅ Step 0.1: Characterization tests created            ││
│  │  ✅ Tests passing: 12/12                                ││
│  │  ⬜ Step 0.2: Mutation tests (optional)                 ││
│  │  ⬜ Step 0.3: Document API contract                      ││
│  │                                                         ││
│  │  Progress: ████████░░░░░░░░ 33% (Phase 0)               ││
│  └─────────────────────────────────────────────────────────┘│
│                                                              │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  🎯 NEXT STEP                                           ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  [A] Continue to Phase 1 - Refactor simpler functions   ││
│  │      → Start with auth::hash_pass (low risk)            ││
│  │                                                         ││
│  │  [B] Add mutation tests first (optional)                ││
│  │      → Verify tests catch real bugs                     ││
│  │                                                         ││
│  │  [C] Document API contract                              ││
│  │      → Create docs/auth-api-contract.md                 ││
│  │                                                         ││
│  │  [D] Take a break - save progress                       ││
│  │      → Export state to resume later                     ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

**Agent presents:**

```
Created: tests/auth_login_characterization.rs

✅ 12 tests generated
✅ All tests pass

Tests cover:
- Happy path (valid credentials, JWT format)
- Error cases (wrong password, nonexistent user)
- Edge cases (empty inputs, SQL injection)

Progress: Phase 0 is 33% complete

What next?

A) Continue to Phase 1 - refactor hash_pass
B) Add mutation tests (optional)
C) Document the API contract
D) Save progress and take a break
```

---

## KEY PATTERNS DEMONSTRATED

### 1. Conversational Response Structure

Every CLI response includes:
- **Data** (what you asked for)
- **Interpretation** (what it means)
- **Recommendations** (what to do)
- **Choices** (next actions)

### 2. Progressive Disclosure

```
Level 1: Summary (visible by default)
Level 2: Details (expandable)
Level 3: Raw data (optional)
```

### 3. Risk-Based Guidance

```
LOW risk → "Go ahead"
MEDIUM risk → "Here's what to watch for"
HIGH risk → "⚠️ Consider alternatives"
```

### 4. Checkpoint System

```
✅ Completed
⬜ Pending
⚠️ Blocked
```

### 5. Always Provide Next Actions

Never leave the user asking "what now?" - always offer [A], [B], [C], [D] choices.

---

## HOW TO IMPLEMENT IN PARSELTONGUE

### Response Template (Rust)

```rust
#[derive(Serialize)]
pub struct ConversationalResponse<T> {
    // Core data
    pub data: T,
    
    // What it means
    pub interpretation: Interpretation,
    
    // What to do
    pub recommendations: Vec<Recommendation>,
    
    // Next actions
    pub next_actions: Vec<NextAction>,
    
    // Metadata
    pub query_time_ms: u64,
    pub token_usage: Option<TokenUsage>,
}

#[derive(Serialize)]
pub struct NextAction {
    pub key: char,           // 'A', 'B', 'C', 'D'
    pub label: String,       // "Create refactoring plan"
    pub description: String, // "Step-by-step changes with safety checks"
    pub recommended: bool,   // Highlight the suggested option
    pub tool_call: Option<ToolCall>, // Pre-filled next call
}
```

### Example Implementation

```rust
pub fn blast_radius(entity: &str) -> ConversationalResponse<BlastRadiusData> {
    let data = compute_blast_radius(entity);
    
    ConversationalResponse {
        data: data.clone(),
        interpretation: Interpretation {
            summary: format!("{} entities affected", data.total_affected),
            risk_level: data.risk_level,
            key_insights: vec![
                "3 direct callers will break immediately".into(),
                "Changes propagate across 3 layers".into(),
            ],
        },
        recommendations: vec![
            Recommendation {
                priority: 1,
                action: "Check complexity first".into(),
                reason: "Understand code difficulty before planning".into(),
            },
        ],
        next_actions: vec![
            NextAction {
                key: 'A',
                label: "Check complexity".into(),
                description: "Analyze auth::login complexity score".into(),
                recommended: true,
                tool_call: Some(ToolCall {
                    tool: "parseltongue_complexity".into(),
                    args: vec![entity.into()],
                }),
            },
            NextAction {
                key: 'B',
                label: "Show graph".into(),
                description: "Visualize dependency graph".into(),
                recommended: false,
                tool_call: Some(ToolCall {
                    tool: "parseltongue_graph".into(),
                    args: vec![entity.into(), "--format".into(), "mermaid".into()],
                }),
            },
        ],
        query_time_ms: 47,
        token_usage: None,
    }
}
```

---

## SUMMARY

The conversational MCP approach means:

1. **Every response is a mini-guide** - not just data
2. **Agent never gets stuck** - always has next steps
3. **User stays informed** - risk levels, progress, options
4. **Natural dialogue** - choices feel like conversation, not API

This transforms Parseltongue from a tool the agent "might call" into a 
collaborator that "guides the conversation."
