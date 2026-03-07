I am trying to build Parseltongue v2.0.0. PRD

but it is confusing

what is important, what is not important

it is overwhelming what needs to be done

Not sure

# bookmarks

- https://x.com/mfranz_on/status/2021364017147818434
- 
# some good ideas



⛬  Let me think creatively about more dimensions of variation:

   ──────────────────────────────────────────

   DIMENSION 1: TRAVERSAL DIRECTION

   Top-Down (Entry Point → Implementation)

     USER: "authentication"
           ↓
     Start from main.rs → find auth route → trace down to implementation

   Use case: Understanding how a feature works from outside-in

   Bottom-Up (Implementation → Usage)

     USER: "auth::login"
           ↓
     Start from function → find all callers → trace up to entry points

   Use case: Impact analysis, "who uses this?"

   Bidirectional (Both simultaneously)

     USER: "auth::login"
           ↓
     ↓ Trace DOWN (what it calls)
     ↑ Trace UP (who calls it)
     → Merge at entity

   Use case: Full picture in one query

   ──────────────────────────────────────────

   DIMENSION 2: WHO DRIVES

   User-Driven (Current approach)

     System presents → User picks → System deep dives

   LLM-Driven (Autonomous)

     System presents → LLM picks best cluster → Returns full answer

   Trade-off: Faster, but LLM might pick wrong

   Collaborative (Both)

     System presents → LLM recommends [1] with reasoning → User confirms or overrides

   ──────────────────────────────────────────

   DIMENSION 3: TEMPORAL VARIATIONS

   Snapshot (Current state)

     Query code as it is NOW

   Time-Travel (Git history)

     USER: "authentication @ 3-months-ago"
           ↓
     Query code as it was 3 months ago, compare with now

   Evolution View

     USER: "authentication evolution"
           ↓
     Show how auth changed over last 20 commits

   ──────────────────────────────────────────

   DIMENSION 4: PROACTIVE VARIATIONS

   Reactive (Current)

     User asks → System answers

   Predictive Pre-fetch

     User asks about "login" → System ALSO pre-computes:
       - password_reset (related)
       - session_management (related)
       - auth_middleware (caller)
     Cache results silently

   Next query is instant

   Code Smell Radar

     User asks about "auth" → System returns:
       - Context for auth
       - WARNING: High complexity in auth::login (score: 47)
       - WARNING: No tests for auth::session

   ──────────────────────────────────────────

   DIMENSION 5: OUTPUT FORMAT VARIATIONS

   Text Summary (Current)

     [1] API HANDLER LAYER - login_route

   Code Slice

     Return minimal executable code path:
     fn main() → route() → auth::login() → return

   Graph Visualization

     Return DOT graph for rendering:
     digraph {
       login_route → auth::login → session::create
     }

   Diff Preview

     "Here's what would change if you modified auth::login:
      - 3 callers affected
      - 12 test files need updates"

   ──────────────────────────────────────────

   DIMENSION 6: SESSION VARIATIONS

   Stateless (Current)

     Each query is independent

   Session Memory

     Query 1: "auth"
     Query 2: "show me the tests" ← System knows "tests for auth"
     Query 3: "complexity?" ← System knows "complexity of auth"

   Breadcrumb Trail

     auth → login_route → session → (back) → login_route → (back) → auth

   ──────────────────────────────────────────

   DIMENSION 7: CONSTRAINT VARIATIONS

   Token-Bounded

     "Give me auth context in 500 tokens max"
     → System compresses aggressively

   Time-Bounded

     "Give me answer in 100ms max"
     → System returns partial results with progress indicator

   Depth-Bounded

     "Depth = 2" (only 2 hops in call graph)
     vs "Depth = 5" (full transitive closure)

   ──────────────────────────────────────────

   DIMENSION 8: SPECIALIZED VARIATIONS

   Test-Guided

     USER: "auth failing test"
           ↓
     Start from failing test → trace code path → highlight likely culprits

   Security-Focused

     USER: "auth security"
           ↓
     Find: input validation, encryption, SQL queries, external calls
     Flag: potential vulnerabilities

   Refactoring-Ready

     USER: "auth refactor"
           ↓
     Return: complexity scores, test coverage, blast radius, suggested safe order

   ──────────────────────────────────────────

   THE META-VARIATION: COMPOSABLE PIPELINE

   Instead of fixed workflow, let user/LLM compose:

     QUERY: "auth"
     │
     ├─ RETRIEVE: RRF(symbol, fuzzy, git) → 4 candidates
     │
     ├─ ANCHOR: BFS(public) → 2 clusters
     │
     ├─ ENRICH:
     │   ├─ complexity_score()
     │   ├─ test_coverage()
     │   ├─ recent_changes()
     │   └─ blast_radius()
     │
     ├─ FILTER: complexity > 30 AND coverage < 50%
     │
     ├─ RANK: BY blast_radius DESC
     │
     └─ OUTPUT: top 2 in 1000 tokens

   Each step is a CPU-only operation. User/LLM can modify the pipeline.

   ──────────────────────────────────────────

   WHICH VARIATION WINS?

   Your core thesis still holds:

   Criterion             │ Winner                        
   ----------------------+-------------------------------
   Lowest cognitive load │ Progressive (your approach)
   Fastest               │ Predictive pre-fetch
   Most accurate         │ Test-guided
   Most flexible         │ Composable pipeline
   Best for LLMs         │ Session memory + your approach

   My recommendation: Start with your 5-phase progressive approach, then add:
   1. Session memory (remembers context across queries)
   2. Predictive pre-fetch (silently cache related entities)
   3. Composable hints (let LLM hint at depth/format constraints)



# some ideas

Great prompt. Keep the **same single API** and create **workflow profiles** on top of it.

Use one endpoint:
`POST /context-query-converge`

Then vary behavior by `profile`.

| Profile | What user sees | Best for | Tradeoff |
|---|---|---|---|
| `balanced` | 2-4 labeled clusters, then pick | Default use | One extra step |
| `auto_fast` | Auto-resolve if score gap is high | Experienced users | Wrong auto-pick risk if threshold too low |
| `safe_strict` | Always disambiguate, never auto-resolve | High-stakes edits | Slightly slower |
| `debug_error` | Starts from compiler/test error, then cluster | Bug fixing | Needs logs/errors input |
| `refactor_impact` | Shows “what breaks” cluster first | API changes | More graph compute |
| `learn_codebase` | Shows architecture/module cluster summaries first | New contributors | Less immediate code detail |
| `pr_review` | Biases to changed files + callers/callees | Reviewers | Can miss older hidden coupling |
| `symbol_direct` | If exact symbol hit, jump to deep dive | Power users | Less guidance |
| `compare_two` | Returns side-by-side clusters for top 2 | Ambiguous queries | More tokens upfront |
| `agent_json` | Strict JSON state machine (`resolved/disambiguate/no_match`) | LLM/automation | Human readability lower |

Concrete examples:

1. `pt q "authentication" --profile auto_fast`  
Returns deep dive immediately if top cluster is clearly dominant.

2. `pt q "authentication" --profile safe_strict`  
Always shows:
- API handler cluster
- Auth abstraction cluster  
Then asks for choice.

3. `pt q --from-error build.log --profile debug_error`  
Extracts symbols from error, anchors to public entity, returns minimal fix context first.

4. `pt q "rename AuthProvider.login" --profile refactor_impact`  
Starts with blast radius + affected impls + tests before code dump.

5. `pt q "auth" --profile learn_codebase`  
Shows module map + key interfaces + 1-hop neighbors before deep code.

If you want, I can draft the exact `profile` enum + default thresholds (auto-resolve gap, candidate count, token budgets).



**Moment-To-Moment UX Blueprint (Primary Segment: OSS Rust Contributors, Large Repos)**

You’re right. The value is in the *felt flow*, not just architecture.  
Here is the concrete user experience.

### 1) Core “Ask → Navigate → Decide → Deep Dive” Experience

| Moment | User Sees | User Does | Tool Does (Behind the scenes) | User Feels |
|---|---|---|---|---|
| 0 | Chat box + current branch + repo scope | Types: “How does authentication work?” | Intent classify (`explain-architecture`) | “It understands my goal” |
| 1 | “I interpreted your intent as: Explain architecture (84%). Change?” | Accepts or edits intent | Query decomposition into semantic anchors | “I’m in control” |
| 2 | 3-5 **Option Cards** (not raw hits) | Scans cards | Search + entity-wrap + graph clustering | “I can compare quickly” |
| 3 | Each card shows breadcrumb path, anchor entity, top neighbors, confidence, token cost | Clicks one card | Prepares high-ROI context packet | “Low risk to try” |
| 4 | “Preview context (900 tokens): signature map + call slice + type links” | Clicks “Deep dive” or “Compare” | Expands chosen cluster by intent profile | “I only read what matters” |
| 5 | Deep view: execution path, data movement, related tests, likely invariants | Asks follow-up | Pulls selective code spans + verified graph facts | “This is trustworthy” |
| 6 | Action bar: “Generate patch plan / Run blast radius / Review checklist” | Picks next action | Switches mode from explain to change/risk | “Momentum, no context reset” |
| 7 | Final answer includes exact files/lines + confidence + unknowns | Proceeds to edit/review | Persists session memory for thread | “I can act immediately” |

---

### 2) What the Option Card Must Show (Critical)

For each candidate cluster, user sees:

1. **Title**: `auth::service::authenticate_user`  
2. **Breadcrumb**: `crates/auth/src/service.rs`  
3. **Why this card exists**: lexical/semantic/graph reasons  
4. **Top related entities**: 3-4 named neighbors  
5. **Risk badge**: low/medium/high change impact  
6. **Confidence score**  
7. **Token budget estimate** (`~700`, `~1400`)  
8. **Freshness badge** (up-to-date vs stale)

This removes the “black box search” feeling.

---

### 3) Segment x Workflow (Detailed Experience)

| Segment | First 30 seconds | Next 2 minutes | Success moment |
|---|---|---|---|
| New OSS contributor | Gets “read order” card: where to start, what to ignore | Guided deep dive with glossary + key entities + tests | Opens first PR without reading entire repo |
| Repeat contributor | Gets task-focused cards tied to issue/keyword | Chooses one cluster, receives patch plan + blast radius | Makes correct change with fewer review cycles |
| Maintainer/reviewer | Drops PR diff, gets changed-entity map | Gets hidden impact edges + control/data-flow warnings | Catches non-obvious regressions quickly |

---

### 4) High-Quality “Feel” Targets (PMF Level)

1. User reaches a trustworthy target cluster in <60s.  
2. User gets actionable context (not docs dump) in <120s.  
3. User always sees **why** result was chosen.  
4. User can pivot intent without restarting from scratch.  
5. User sees uncertainty explicitly instead of false precision.

---

### 5) Product Behavior in Ambiguity (Important)

When confidence is low, user sees:

1. “Top candidates are close; pick one:”  
2. 2-3 disambiguation prompts (“Auth login path or token validation path?”)  
3. Side-by-side mini previews

So the tool feels honest, not hallucinated.

---

If you want, I can now write this exact moment-to-moment UX spec into the top section of [ES-V200-Decision-log-01.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-Reference/ES-V200-Decision-log-01.md) as a formal “User Experience Contract” and push.

# Code Search Workflow

Use **intent-specific workflow variants**, not one universal flow. For OSS Rust in large repos, this is a strong segment x workflow map:

| Segment | Workflow Variant | How LLM Navigates | Output |
|---|---|---|---|
| New contributor | `Map-first onboarding` | Query -> crate/module map -> key public APIs -> “read order” path | 30-min learning route |
| Returning contributor | `Task-first patch path` | Issue text -> candidate entities -> enclosing owners -> minimal edit set | patch plan with files/lines |
| Core maintainer | `Blast-radius before edit` | Proposed change -> callers/callees/trait impls/SCC neighbors | risk-ranked impact report |
| Reviewer | `PR truth-check` | Diff -> changed entities -> dependency + control-flow slices | review checklist + hidden break risks |
| Bug fixer | `Symptom-to-path` | Error/log tokens -> likely execution paths -> dataflow checkpoints | top 3 root-cause hypotheses |
| API migrator | `Signature migration graph` | Old API refs -> wrapper/adapters -> downstream breakpoints | ordered migration sequence |
| Unsafe code owner | `Soundness audit mode` | `unsafe` blocks -> aliasing/lifetime neighbors -> call paths | invariant map + violation candidates |
| Perf contributor | `Hotpath context mode` | perf clue -> hot functions -> transitive callee chain + alloc points | optimization targets with confidence |

Top 3 to build first for your segment:
1. `Blast-radius before edit` (maintainers feel this pain daily)  
2. `PR truth-check` (review is the bottleneck in large OSS)  
3. `Task-first patch path` (contributor throughput)

Key creative variations on your current pipeline:
1. **Auto-route by intent** before retrieval (`bug`, `refactor`, `review`, `migrate`, `unsafe`, `perf`).  
2. **Two-pass choice model**: machine picks when confidence is high; ask user/LLM only when ambiguous.  
3. **Cluster cards, not raw results**: each option card includes scope path, key entities, risk, confidence, estimated tokens.  
4. **Depth budget profiles**: `skim`, `balanced`, `deep`, `proof` modes.  
5. **Counterfactual card** for every deep dive: “if we change X, what likely breaks first?”

If you want, I can turn this into a formal “Segment x Workflow Operating Model” section at the top of the V200 decision log.

===


   The Meta-Pattern: Iterative Refinement Through Disambiguation

     ┌─────────────────────────────────────────────────────────────────┐
     │                    PHASE 1: SEARCH                              │
     │   "authentication" → [auth, login, authenticate, oauth]        │
     │   (fuzzy keyword → 3-4 semantic candidates)                     │
     └─────────────────────────────────────────────────────────────────┘
                                   ↓
     ┌─────────────────────────────────────────────────────────────────┐
     │                    PHASE 2: ANCHOR                              │
     │   For each candidate:                                           │
     │   - Find where it appears in code                               │
     │   - Bubble up to ENCAPSULATING PUBLIC ENTITY                    │
     │   - (private impl → public function that calls it)              │
     └─────────────────────────────────────────────────────────────────┘
                                   ↓
     ┌─────────────────────────────────────────────────────────────────┐
     │                    PHASE 3: CLUSTER                             │
     │   For each anchored entity:                                     │
     │   - Top 3-4 connected entities (callers/callees/impls)         │
     │   - Module path (crate::module::submodule::entity)             │
     │   - Signature snippet                                           │
     └─────────────────────────────────────────────────────────────────┘
                                   ↓
     ┌─────────────────────────────────────────────────────────────────┐
     │                    PHASE 4: DISAMBIGUATE                        │
     │   Present N clusters to LLM:                                    │
     │   "Which of these are you interested in?"                       │
     │   LLM chooses based on context clues in clusters               │
     └─────────────────────────────────────────────────────────────────┘
                                   ↓
     ┌─────────────────────────────────────────────────────────────────┐
     │                    PHASE 5: DEEP DIVE                           │
     │   For chosen cluster:                                           │
     │   - Full code                                                   │
     │   - Complete relationship graph                                 │
     │   - Type information                                            │
     │   - Control flow highlights                                     │
     └─────────────────────────────────────────────────────────────────┘

   ──────────────────────────────────────────

   Key Insight: The "Encapsulating Public Entity" Pattern

   This is the crucial observation:

     User searches: "validate_credentials"
     Found in: src/auth/internal/validator.rs:47 (PRIVATE function)

     Instead of returning the private function directly:
       → Bubble up to: pub fn login() at src/auth/mod.rs:23
       → This is the PUBLIC interface that contains the search hit

   Why this matters:
   1. Public interfaces are stable - less likely to change than internals
   2. Public interfaces are the API boundary - what you actually interact with
   3. LLMs get context through the public contract, not lost in implementation details
   4. Private code is accessed through its public interface in real usage

   ──────────────────────────────────────────

   The Pattern in Abstract Terms

   Phase            │ Input       │ Output              │ Key Operation                
   -----------------+-------------+---------------------+------------------------------
   **Search**       │ Fuzzy query │ Semantic candidates │ Cosine/embedding similarity
   **Anchor**       │ Candidates  │ Public entities     │ Bubble-up to public interface
   **Cluster**      │ Entity      │ Context packet      │ Graph expansion (3-4 hops)
   **Disambiguate** │ N packets   │ 1 chosen packet     │ LLM selects based on context
   **Deep Dive**    │ 1 packet    │ Full context        │ Complete analysis

   ──────────────────────────────────────────

   What Makes This Pattern Smart

   1. Progressive disclosure - Don't dump everything at once
   2. Structured ambiguity - When uncertain, present options WITH context
   3. Public-first - Surface through APIs, not implementation
   4. Clustered context - Enough to choose, not so much it overwhelms
   5. Human-in-the-loop (or LLM-in-the-loop) - Let the chooser decide

   ──────────────────────────────────────────

   One Refinement to Consider

   The Cluster phase could use different "expansion profiles" based on query intent:

   Query Type                   │ Cluster By                   
   -----------------------------+------------------------------
   "How does X work?"           │ Callees (down the call graph)
   "Where is X used?"           │ Callers (up the call graph)
   "What implements X?"         │ Trait implementations
   "What breaks if I change X?" │ Both callers AND callees

   This would make the disambiguation clusters more meaningful for the specific question.

   ──────────────────────────────────────────

   Bottom line: Yes, this pattern is exactly right. It's a conversation, not a search. Search → Anchor → Cluster → Choose → Deep Dive
**Moment-To-Moment UX Blueprint (Primary Segment: OSS Rust Contributors, Large Repos)**

You’re right. The value is in the *felt flow*, not just architecture.  
Here is the concrete user experience.

### 1) Core “Ask → Navigate → Decide → Deep Dive” Experience

| Moment | User Sees | User Does | Tool Does (Behind the scenes) | User Feels |
|---|---|---|---|---|
| 0 | Chat box + current branch + repo scope | Types: “How does authentication work?” | Intent classify (`explain-architecture`) | “It understands my goal” |
| 1 | “I interpreted your intent as: Explain architecture (84%). Change?” | Accepts or edits intent | Query decomposition into semantic anchors | “I’m in control” |
| 2 | 3-5 **Option Cards** (not raw hits) | Scans cards | Search + entity-wrap + graph clustering | “I can compare quickly” |
| 3 | Each card shows breadcrumb path, anchor entity, top neighbors, confidence, token cost | Clicks one card | Prepares high-ROI context packet | “Low risk to try” |
| 4 | “Preview context (900 tokens): signature map + call slice + type links” | Clicks “Deep dive” or “Compare” | Expands chosen cluster by intent profile | “I only read what matters” |
| 5 | Deep view: execution path, data movement, related tests, likely invariants | Asks follow-up | Pulls selective code spans + verified graph facts | “This is trustworthy” |
| 6 | Action bar: “Generate patch plan / Run blast radius / Review checklist” | Picks next action | Switches mode from explain to change/risk | “Momentum, no context reset” |
| 7 | Final answer includes exact files/lines + confidence + unknowns | Proceeds to edit/review | Persists session memory for thread | “I can act immediately” |

---

### 2) What the Option Card Must Show (Critical)

For each candidate cluster, user sees:

1. **Title**: `auth::service::authenticate_user`  
2. **Breadcrumb**: `crates/auth/src/service.rs`  
3. **Why this card exists**: lexical/semantic/graph reasons  
4. **Top related entities**: 3-4 named neighbors  
5. **Risk badge**: low/medium/high change impact  
6. **Confidence score**  
7. **Token budget estimate** (`~700`, `~1400`)  
8. **Freshness badge** (up-to-date vs stale)

This removes the “black box search” feeling.

---

### 3) Segment x Workflow (Detailed Experience)

| Segment | First 30 seconds | Next 2 minutes | Success moment |
|---|---|---|---|
| New OSS contributor | Gets “read order” card: where to start, what to ignore | Guided deep dive with glossary + key entities + tests | Opens first PR without reading entire repo |
| Repeat contributor | Gets task-focused cards tied to issue/keyword | Chooses one cluster, receives patch plan + blast radius | Makes correct change with fewer review cycles |
| Maintainer/reviewer | Drops PR diff, gets changed-entity map | Gets hidden impact edges + control/data-flow warnings | Catches non-obvious regressions quickly |

---

### 4) High-Quality “Feel” Targets (PMF Level)

1. User reaches a trustworthy target cluster in <60s.  
2. User gets actionable context (not docs dump) in <120s.  
3. User always sees **why** result was chosen.  
4. User can pivot intent without restarting from scratch.  
5. User sees uncertainty explicitly instead of false precision.

---

### 5) Product Behavior in Ambiguity (Important)

When confidence is low, user sees:

1. “Top candidates are close; pick one:”  
2. 2-3 disambiguation prompts (“Auth login path or token validation path?”)  
3. Side-by-side mini previews

So the tool feels honest, not hallucinated.

---

If you want, I can now write this exact moment-to-moment UX spec into the top section of [ES-V200-Decision-log-01.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-Reference/ES-V200-Decision-log-01.md) as a formal “User Experience Contract” and push.

----

Graph Algorithms + Compiler Information = Superpower

   Let me map each algorithm to what compiler information enables it:

   ──────────────────────────────────────────

   The Key Insight: Compiler Gives You EDGES

     Without compiler:         With compiler:
     ─────────────────         ───────────────
     "Maybe this calls that"   "This DEFINITELY calls that"
     Text similarity           Semantic relationship
     Guesswork                 Ground truth

   The graph algorithms need a graph. The compiler GIVES you the real graph.

   ──────────────────────────────────────────

   Algorithm-by-Algorithm Analysis

   1. Circular Dependency Detection

     Compiler provides edges:
     ├── use statements (module level)
     ├── function calls (code level)
     ├── trait bounds (type level)
     └── generic instantiations (param level)

     What you can detect:
     ├── Module cycles: A uses B uses A
     ├── Call cycles: f() → g() → h() → f()
     ├── Trait cycles: T1: T2, T2: T1
     └── Type cycles: struct A { b: B }, struct B { a: A }

   Why it matters for LLMs:
   •  "This code has circular dependencies" → warn before refactoring
   •  "Breaking this cycle requires changing these 3 modules"

   ──────────────────────────────────────────

   2. Complexity Hotspots Ranking

     Compiler provides:
     ├── Call graph depth (who calls deep call chains)
     ├── Fan-out (how many functions does this call)
     ├── Fan-in (how many callers depend on this)
     ├── Type complexity (generics, traits, lifetimes)
     └── Trait bound chains (T: A + B + C where A: D)

     Hotspot = High fan-in + High fan-out + Complex types

   Example output:

     Top 3 Hotspots:
     1. process_request()
        - 47 callers (fan-in)
        - calls 23 functions (fan-out)
        - 4 generic params, 6 trait bounds

     2. authenticate()
        - 31 callers
        - calls 15 functions
        - 3 lifetime params

   Why it matters for LLMs:
   •  "Be careful changing this - it's a hotspot"
   •  "This function touches many parts of the system"

   ──────────────────────────────────────────

   3. Semantic Cluster Grouping

     Compiler provides:
     ├── Module tree (actual hierarchy)
     ├── Trait implementations (group by capability)
     ├── Type relationships (group by data)
     └── Call density (group by interaction)

     Clusters emerge from:
     ├── "All functions that implement Auth trait"
     ├── "All types that contain User"
     ├── "All functions in call chain from main()"

   Example:

     Cluster: Authentication
     ├── login()
     ├── logout()
     ├── refresh_token()
     ├── validate_session()
     └── [All implement SessionHandler trait]

     Cluster: User Management
     ├── create_user()
     ├── delete_user()
     ├── update_profile()
     └── [All operate on User type]

   Why it matters for LLMs:
   •  "You asked about auth - here's the WHOLE auth cluster"
   •  "These 5 functions are semantically related"

   ──────────────────────────────────────────

   4. Strongly Connected Components (Tarjan SCC)

     Compiler provides:
     ├── Call graph edges
     ├── Trait impl edges
     ├── Type dependency edges

     SCC = set of nodes where every node reaches every other

     What SCCs reveal:
     ├── Mutual recursion groups
     ├── Tightly coupled modules
     ├── Potential refactoring boundaries

   Example:

     SCC #1: {parse, tokenize, lex}
       → These 3 are mutually recursive

     SCC #2: {validate, sanitize, normalize}
       → These 3 call each other in cycles

     Isolated: {format_output}
       → No cycles, easy to extract

   Why it matters for LLMs:
   •  "These functions must be understood together"
   •  "This SCC is a natural module boundary"

   ──────────────────────────────────────────

   5. Technical Debt (SQALE Scoring)

     Compiler provides ACTUAL metrics:
     ├── Cyclomatic complexity (from MIR branches)
     ├── Cognitive complexity (nested control flow)
     ├── Coupling (CBO - types used)
     ├── Cohesion (LCOM - field usage)
     └── Documentation coverage (doc comments)

     SQALE = sum of remediation effort

   Example:

     Function: process_request()
     ├── Cyclomatic: 23 (high - many branches)
     ├── Nesting depth: 5 (medium)
     ├── Coupling: 12 types (high)
     ├── Debt score: 4.2 hours to clean up

     Function: format_date()
     ├── Cyclomatic: 2 (low)
     ├── Nesting depth: 1 (low)
     ├── Coupling: 2 types (low)
     ├── Debt score: 0.1 hours

   Why it matters for LLMs:
   •  "This function is debt-heavy - suggest refactoring"
   •  "Prioritize cleaning high-debt areas"

   ──────────────────────────────────────────

   6. k-Core Decomposition (Layering)

     Compiler provides call graph:
     ├── k=1: Peripheral (called by 1 or none)
     ├── k=2: Bridge (connects periphery to core)
     ├── k=3+: Core (highly interconnected)

     k-core = subgraph where every node has ≥k connections

   Example:

     k=3 Core:
     ├── dispatch_request()
     ├── route_handler()
     ├── middleware_chain()
     └── [These are the "engine" - most connected]

     k=2 Bridge:
     ├── auth_middleware()
     ├── logging_middleware()
     └── [Connect core to periphery]

     k=1 Periphery:
     ├── health_check()
     ├── metrics_endpoint()
     └── [Leaf functions - easy to change]

   Why it matters for LLMs:
   •  "Core functions are critical - changes ripple everywhere"
   •  "Peripheral functions are safe to modify"

   ──────────────────────────────────────────

   7. Centrality Measures (PageRank/Betweenness)

     Compiler provides call graph weights:
     ├── Call frequency (static analysis of call sites)
     ├── Type usage frequency
     ├── Trait implementation count

     PageRank: "Which functions are most important?"
     Betweenness: "Which functions are on the most paths?"

   Example:

     PageRank Top 3:
     1. handle_request() - 0.15 (entry point, many call chains)
     2. authenticate() - 0.12 (security critical, many dependents)
     3. database_query() - 0.09 (data layer, widely used)

     Betweenness Top 3:
     1. middleware() - 0.23 (on path between entry and handlers)
     2. router() - 0.19 (on path between URL and handlers)
     3. serializer() - 0.14 (on path between data and response)

   Why it matters for LLMs:
   •  "These are the most important functions to understand"
   •  "Bugs in high-centrality functions cause widespread issues"

   ──────────────────────────────────────────

   8. Entropy Complexity

     Compiler provides:
     ├── Type variety per function
     ├── Call target variety (for dynamic dispatch)
     ├── Control flow entropy (branch distribution)

     High entropy = unpredictable, complex
     Low entropy = predictable, simple

   Example:

     Function: handle_websocket()
     ├── Handles 12 different message types
     ├── Type entropy: 3.58 bits (high)
     ├── "Complex - many code paths"

     Function: add_numbers()
     ├── Handles 1 type pair (i32, i32) → i32
     ├── Type entropy: 0 bits (zero)
     ├── "Simple - single code path"

   Why it matters for LLMs:
   •  "High entropy = needs more test cases"
   •  "This function is hard to reason about"

   ──────────────────────────────────────────

   9. Coupling/Cohesion (CK Metrics)

     Compiler provides PRECISE counts:

     CBO (Coupling Between Objects):
       → Count of distinct types used
       → From type inference, not text search

     LCOM (Lack of Cohesion of Methods):
       → Do methods use the same fields?
       → From field access analysis

     RFC (Response For Class):
       → Count of methods that can be called
       → From call graph

     WMC (Weighted Methods per Class):
       → Sum of method complexities
       → From MIR cyclomatic

   Example:

     Struct: UserService
     ├── CBO: 8 (uses 8 other types) - highly coupled
     ├── LCOM: 0.7 (methods don't share fields) - low cohesion
     ├── RFC: 45 (can trigger 45 method calls) - high response
     ├── WMC: 34 (total complexity) - complex class

     Diagnosis: "God class - consider splitting"

   Why it matters for LLMs:
   •  "This struct is doing too much"
   •  "These metrics suggest refactoring opportunities"

   ──────────────────────────────────────────

   10. Leiden Community Detection

     Compiler provides edge weights:
     ├── Call frequency
     ├── Type sharing (both use same types)
     ├── Module proximity
     ├── Trait co-implementation

     Leiden finds: "Natural groupings that minimize cross-group edges"

   Example:

     Community A (auth): 23 functions
     ├── High internal calls (89%)
     ├── Low external calls (11%)
     └── Cohesive "auth" concept

     Community B (database): 18 functions
     ├── High internal calls (92%)
     ├── Low external calls (8%)
     └── Cohesive "data access" concept

     Cross-community edges:
     ├── auth → database: 2 (validate_credentials, log_audit)
     └── [These are the "seams" between modules]

   Why it matters for LLMs:
   •  "Here's how the codebase naturally clusters"
   •  "These communities match domain boundaries"

   ──────────────────────────────────────────

   The Combined Power

   When you run ALL these algorithms on compiler-derived graphs:

     You can answer:
     ├── "What's the architecture?" → Leiden + k-core
     ├── "Where's the technical debt?" → SQALE + Complexity
     ├── "What's critical?" → PageRank + Centrality
     ├── "What's fragile?" → SCC + Cycles + Coupling
     ├── "What should I refactor first?" → Debt + Centrality + k-core
     └── "What's easy to change?" → Periphery + Low coupling

   ──────────────────────────────────────────

   For the LLM Navigation Workflow

   In your 5-phase workflow:

   Phase            │ Graph Algorithm Role                                        
   -----------------+-------------------------------------------------------------
   **Search**       │ -
   **Anchor**       │ Find entity in graph
   **Cluster**      │ **Leiden communities** - what group is this in?
   **Disambiguate** │ Show **centrality** - which is more important?
   **Deep Dive**    │ Show **SCC**, **coupling**, **complexity** for chosen entity

   ──────────────────────────────────────────

   Bottom Line

   Graph algorithms without compiler info: Guessing from text
   Graph algorithms with compiler info: Mathematical precision on real structure

   The compiler gives you the TRUE graph. The algorithms reveal its HIDDEN structure.

**Moment-To-Moment UX Blueprint (Primary Segment: OSS Rust Contributors, Large Repos)**

You’re right. The value is in the *felt flow*, not just architecture.  
Here is the concrete user experience.

### 1) Core “Ask → Navigate → Decide → Deep Dive” Experience

| Moment | User Sees | User Does | Tool Does (Behind the scenes) | User Feels |
|---|---|---|---|---|
| 0 | Chat box + current branch + repo scope | Types: “How does authentication work?” | Intent classify (`explain-architecture`) | “It understands my goal” |
| 1 | “I interpreted your intent as: Explain architecture (84%). Change?” | Accepts or edits intent | Query decomposition into semantic anchors | “I’m in control” |
| 2 | 3-5 **Option Cards** (not raw hits) | Scans cards | Search + entity-wrap + graph clustering | “I can compare quickly” |
| 3 | Each card shows breadcrumb path, anchor entity, top neighbors, confidence, token cost | Clicks one card | Prepares high-ROI context packet | “Low risk to try” |
| 4 | “Preview context (900 tokens): signature map + call slice + type links” | Clicks “Deep dive” or “Compare” | Expands chosen cluster by intent profile | “I only read what matters” |
| 5 | Deep view: execution path, data movement, related tests, likely invariants | Asks follow-up | Pulls selective code spans + verified graph facts | “This is trustworthy” |
| 6 | Action bar: “Generate patch plan / Run blast radius / Review checklist” | Picks next action | Switches mode from explain to change/risk | “Momentum, no context reset” |
| 7 | Final answer includes exact files/lines + confidence + unknowns | Proceeds to edit/review | Persists session memory for thread | “I can act immediately” |

---

### 2) What the Option Card Must Show (Critical)

For each candidate cluster, user sees:

1. **Title**: `auth::service::authenticate_user`  
2. **Breadcrumb**: `crates/auth/src/service.rs`  
3. **Why this card exists**: lexical/semantic/graph reasons  
4. **Top related entities**: 3-4 named neighbors  
5. **Risk badge**: low/medium/high change impact  
6. **Confidence score**  
7. **Token budget estimate** (`~700`, `~1400`)  
8. **Freshness badge** (up-to-date vs stale)

This removes the “black box search” feeling.

---

### 3) Segment x Workflow (Detailed Experience)

| Segment | First 30 seconds | Next 2 minutes | Success moment |
|---|---|---|---|
| New OSS contributor | Gets “read order” card: where to start, what to ignore | Guided deep dive with glossary + key entities + tests | Opens first PR without reading entire repo |
| Repeat contributor | Gets task-focused cards tied to issue/keyword | Chooses one cluster, receives patch plan + blast radius | Makes correct change with fewer review cycles |
| Maintainer/reviewer | Drops PR diff, gets changed-entity map | Gets hidden impact edges + control/data-flow warnings | Catches non-obvious regressions quickly |

---

### 4) High-Quality “Feel” Targets (PMF Level)

1. User reaches a trustworthy target cluster in <60s.  
2. User gets actionable context (not docs dump) in <120s.  
3. User always sees **why** result was chosen.  
4. User can pivot intent without restarting from scratch.  
5. User sees uncertainty explicitly instead of false precision.

---

### 5) Product Behavior in Ambiguity (Important)

When confidence is low, user sees:

1. “Top candidates are close; pick one:”  
2. 2-3 disambiguation prompts (“Auth login path or token validation path?”)  
3. Side-by-side mini previews

So the tool feels honest, not hallucinated.

---

If you want, I can now write this exact moment-to-moment UX spec into the top section of [ES-V200-Decision-log-01.md](/Users/amuldotexe/Desktop/parseltongue-rust-LLM-companion/docs/ACTIVE-Reference/ES-V200-Decision-log-01.md) as a formal “User Experience Contract” and push.



# Parseltongue

>  Now understanding this workflow - The LLM comes and asks you: "Tell me about suppose authentication."First thing you would do is probablySearch for non-keywords in the codebase which are 
   similar to the given keyword. You might come up with, say, two or three options due to cosine similarity or something.Ideally, if you had LLMs checking this, this would have been better, but 
   because they cannot, because it might be too much, I don't know. Maybe you will give them some similar words and ask them which ones they are looking for.You will say, yeah, these 3, 4 look 
   like good search options. Then, for each of them, you run 1 by 1 to find them in the code base. When you find them in the code base, they might not be at the interface level or a public 
   interface level, which is captured by any entity, because the entity is only public interfaces, right? Only if a function is exposed publicly shall you be able to look at it. It will be an 
   entity; otherwise it won't be an entity.So what you would do hence is, in each of them, one by one, you will find the encapsulating public entity, the public function or whatever, which is 
   having this keyword in the code.And then you will try to find the top 3 or 4 connected entities to this. You will also mention this is under this line of the tree, which is to say that, under
    this big folder, then big folder, then big folder, under this, this is there.Now, because you will do it for all the four keywords there, or code search results, you will give it to the lm 
   again as an option, saying that these are the four options that we have. Each of them, giving them a cluster of entities that they can look atLLM will then choose which one they are more 
   interested in. The one they are more interested in, you will do a deep dive analysis and come back with actual code and as much information as possible.--- try to create the meta-pattern of 
   this kind of workflow - do not write - just deeply think and tell me if this makes sense

### Rust Analyzer Superset Mapping
Assessment:
1. Rust Analyzer inputs are a practical superset of our read-pointer format.
2. RA carries richer identity/range context (file identity + typed symbol context + ranges), which can be projected to our compact read key.

Canonical layers:
1. Stable identity layer:
   - `language|||kind|||scope|||name|||file_path|||discriminator`
2. Read-pointer layer (for code fetch):
   - `file_path|||entity_key|||start_line|||end_line`

Projection contract:
1. Resolve RA symbol/definition to canonical `entity_key`.
2. Convert RA range/offsets to line-based span.
3. Emit read pointer exactly as:
   - `filepath|||entity|||StartLine|||EndLine`
4. Use full entity span for default code read; keep narrower selection ranges optional for focused context.

Example:
1. `entity_key = rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3`
2. `read_pointer = src/auth/service.rs|||rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3|||40|||96`

### Options Under Evaluation
**Option PK-A: Local sidecar-first**
1. Keep graph canonical in Parseltongue.
2. Use local retriever sidecar for candidate spans.
3. Resolve spans to `entity_key` before analysis.
4. Lower infrastructure overhead.

**Option PK-B: Turso/libSQL retrieval index**
1. Persist pointer metadata + ranking features in Turso/libSQL.
2. Keep source-on-read model (no full body persistence).
3. Better for shared/team retrieval at scale.
4. Higher operational complexity.

**Option PK-C: Hybrid**
1. Local-first default.
2. Turso mode as explicit opt-in.
3. Same canonical key schema in both modes.

### Open Questions
1. `OQ-PK-1`: What is the final schema and uniqueness contract for `entity_key` and `chunk_key`?
2. `OQ-PK-2`: How should overlaps be represented when one chunk maps to multiple entities?
3. `OQ-PK-3`: Should `entity_version_hash` be mandatory or optional in V216?
4. `OQ-PK-4`: Is Turso mode in V216 core scope or deferred behind a capability flag?
5. `OQ-PK-5`: What p95 freshness and retrieval latency SLOs are required for default enablement?

### Acceptance Criteria
1. 100% of queryable entities have canonical `entity_key`.
2. 100% of retriever hits can resolve to `entity_key` or explicit unresolved state.
3. No stale row can produce `verified` truth-grade output.
4. Pointer-based source retrieval returns explicit stale/missing errors, never silent fallback.
5. Both local and Turso modes (if enabled) preserve identical key semantics.

### Happy Path Example (End-to-End)
User ask:
1. "Where is authentication implemented and what breaks if I change it?"

System flow:
1. Retriever returns top chunks with pointers:
   - `chunk_key=ch_01`, `file_path=src/auth/service.rs`, `start_line=40`, `end_line=96`, `score=0.91`
   - `chunk_key=ch_17`, `file_path=src/auth/token.rs`, `start_line=10`, `end_line=58`, `score=0.87`
2. Resolve chunks to canonical entities:
   - `ch_01 -> entity_key=rust|||fn|||auth::service|||authenticate_user|||src/auth/service.rs|||sig_v3`
   - `ch_17 -> entity_key=rust|||fn|||auth::token|||verify_token|||src/auth/token.rs|||sig_v1`
3. Freshness check passes (`file_hash` matches current file state).
4. Graph analysis runs on resolved entity key:
   - callers
   - callees
   - blast radius (hops=2)
5. Response returns:
   - ranked entities
   - trusted pointer ranges
   - dependency impact summary
   - confidence/truth-grade annotations

### Classification Model (Code + Non-Code)
Every discovered file must terminate in one classification:
1. `code-graph`
   - parseable source files used for entities/edges
2. `identifiable-tests`
   - test-only files (unit/integration/e2e/fixtures)
   - for mixed files, test tagging moves to entity-level (see below)
3. `docs`
   - markdown/rst/adoc and other documentation artifacts
4. `non-eligible-text`
   - unsupported language/extensions, generated blobs, binaries, or irrelevant text

Entity-level sub-classification in `code-graph`:
1. `implementation-entity`
   - functions/classes/types participating in runtime behavior
2. `test-entity`
   - test blocks inside otherwise non-test files (e.g., inline module tests, cfg(test), test functions)
   - excluded from default production blast-radius unless explicitly included
3. `comment-entity` (optional, low-trust)
   - extracted comments/docstrings as advisory metadata only
   - never promoted to dependency truth
4. `unparsable-entity`
   - file is eligible but parse failed/partial; store with degrade reason and non-verified grade

Handling rules:
1. Incompatible/unsupported files -> `non-eligible-text` with reason code.
2. Parse failures in supported files -> keep ledger row, mark `partial` or `failed`, never silently drop.
3. Mixed files stay `code-graph` at file level; only test nodes are marked `test-entity`.
4. Tests are visible and queryable with explicit filter flags, but excluded from default blast-radius unless requested.
5. Comments/docstrings can improve retrieval ranking, but cannot create verified edges without parser/LSP evidence.


===

