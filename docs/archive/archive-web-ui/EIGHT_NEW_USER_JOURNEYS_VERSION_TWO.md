# Eight NEW User Journeys for Parseltongue Visualization (Version 2)

**Complementary Research Distinct from Version 1**

**Date**: 2026-01-14
**Status**: Research Complete
**Context**: Parseltongue Dependency Graph Generator - 239 entities, 211 dependency arcs
**Version**: 2.0 (New journeys, new personas, new metaphors)

---

## Executive Summary

This document presents **eight completely NEW user journeys** that complement but DO NOT duplicate the first version. While Version 1 covered general developer workflows (Onboarding, Bug Investigation, Refactoring, Code Review), Version 2 explores **specialized roles, runtime concerns, and advanced use cases**.

**Critical**: These 8 journeys are fundamentally different from Version 1 in:

| Dimension | Version 1 (Original) | Version 2 (This Document) |
|-----------|---------------------|---------------------------|
| **Target Persona** | General developers | Specialized roles (QA, DevOps, Security, SRE) |
| **Primary Focus** | Code-level concerns | Runtime, performance, security, compliance |
| **Trigger Context** | Development workflow | Production incidents, migrations, audits |
| **Visualization Metaphor** | Static code structure | Dynamic behavior, temporal patterns |
| **Time Horizon** | Current code state | Historical, predictive, comparative |

---

## Part 1: The Eight New User Journeys

### Journey 1: Performance Profiler's Bottleneck Hunt

**Persona**: Performance engineer or senior developer optimizing runtime behavior

**Trigger**: Application slow-down, high latency alerts, memory leaks, CPU spikes

**Mental State**: Analytical, urgent, measurement-driven, hypothesis-testing mode

**Goals**:
- Identify runtime bottlenecks in call paths
- Understand which functions are called most frequently
- Detect N+1 query patterns and excessive iterations
- Optimize hot paths and critical sections
- Measure impact of performance optimizations

**Success Criteria**:
- Bottleneck identified with execution frequency data
- Hot path visualized with cumulative cost
- Optimization opportunities ranked by ROI
- Performance regression detected before deployment
- A/B comparison of optimization results

**Pain Points**:
- Code complexity ≠ runtime cost (deceptive functions)
- Hidden hot paths (function called millions of times)
- Cascade failures (small delay amplifies downstream)
- Profiling tools lack architectural context
- Optimization in wrong place (premature optimization)

**Time Pressure**: High (production issues require immediate attention)

**Frequency**: Weekly performance reviews, ad-hoc incident response

**Questions They Ask**:
- "Why is this endpoint so slow?"
- "Which functions are called the most?"
- "What's the cumulative cost of this call path?"
- "Are there N+1 query patterns?"
- "Did my optimization actually help?"

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=50` - **Core**: High fan-in = frequently called
- `/forward-callees-query-graph?entity=X` - **Core**: Trace call paths
- `/reverse-callers-query-graph?entity=X` - **Core**: See what's calling hot code
- `/blast-radius-impact-analysis?entity=X&hops=3` - Understand optimization impact
- `/code-entity-detail-view?key=X` - Check implementation for optimization opportunities
- `/dependency-edges-list-all` - Identify potential N+1 patterns (loops calling external services)

**Design Voice**: Jay Doblin's systems thinking - understanding feedback loops and system behavior

---

### Journey 2: Security Auditor's Attack Surface Analysis

**Persona**: Security engineer or application security analyst

**Trigger**: Security audit, penetration testing preparation, compliance review, vulnerability assessment

**Mental State**: Vigilant, adversarial mindset, risk-assessment mode, thoroughness-focused

**Goals**:
- Map attack surface (user input flow to sensitive operations)
- Identify security-sensitive functions (auth, crypto, data access)
- Trace data flows from untrusted inputs
- Find code that calls risky external dependencies
- Ensure security boundaries are enforced

**Success Criteria**:
- All entry points (HTTP handlers, CLI commands) mapped
- Data flow from input to sensitive operations traced
- Security-critical functions identified and categorized
- Risky dependencies catalogued with call sites
- Security validation points (checks) located

**Pain Points**:
- Indirect attack paths (input → complex chain → sensitive op)
- Hidden data flows (serialization, reflection)
- Unknown dependency vulnerabilities
- Scattered security controls
- Missing context on "why is this secure?"

**Time Pressure**: Medium (audits have deadlines, but thoroughness matters)

**Frequency**: Quarterly audits, pre-release security reviews

**Questions They Ask**:
- "What are all the entry points an attacker can reach?"
- "Where does user input go?"
- "Which functions handle authentication/authorization?"
- "What calls does this security-critical function make?"
- "Are there vulnerable dependencies in the call path?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy?q=handler|http|route|cli` - **Entry Points**
- `/forward-callees-query-graph?entity=ENTRY_POINT` - **Data Flow Tracing**
- `/code-entities-search-fuzzy?q=auth|crypto|password|token` - **Security Functions**
- `/reverse-callers-query-graph?entity=SECURITY_FN` - **Who calls security code**
- `/dependency-edges-list-all` - **Filter for unknown:0-0** (external deps)
- `/semantic-cluster-grouping-list` - Identify security clusters

**Design Voice**: Bruce Schneier's security mindset - "Think like an attacker"

---

### Journey 3: Incident Responder's Failure Cascade Mapping

**Persona**: Site Reliability Engineer (SRE) or on-call engineer

**Trigger**: Production incident, PagerDuty alert, customer report of failure

**Mental State**: Urgent, focused, time-pressured, systematic elimination

**Goals**:
- Understand what changed recently
- Trace failure propagation through system
- Identify all components affected by failing service
- Find potential fix locations
- Communicate impact to stakeholders

**Success Criteria**:
- Root component identified
- Failure cascade mapped (what breaks if X fails)
- Blast radius quantified (how many users affected)
- Fix candidates identified with risk assessment
- Status update drafted with architectural context

**Pain Points**:
- Symptom-cause gap (error manifests far from source)
- Cascading failures (failure propagates unpredictably)
- Missing recent change context
- Hidden dependencies (config/environment not in code)
- Pressure to fix quickly vs. understand fully

**Time Pressure**: Critical (every minute of downtime matters)

**Frequency**: Incident-driven (could be daily, weekly, or monthly depending on maturity)

**Questions They Ask**:
- "What's calling this failing service?"
- "What else breaks if this goes down?"
- "What changed recently that could cause this?"
- "Which services are in the blast radius?"
- "What's the safest fix path?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy?q=ERROR_SERVICE_NAME` - **Locate failing component**
- `/reverse-callers-query-graph?entity=FAILING_COMPONENT` - **Who's affected**
- `/blast-radius-impact-analysis?entity=FAILING_COMPONENT&hops=2` - **Failure cascade**
- `/circular-dependency-detection-scan` - **Check for deadlock risk**
- `/semantic-cluster-grouping-list` - **Understand component context**
- `/code-entity-detail-view?key=X` - **Read implementation**

**Design Voice**: John Allspaw's blameless postmortem culture - focus on system, not individuals

---

### Journey 4: Legacy Migrator's Dependency Unraveling

**Persona**: Senior engineer or architect leading legacy system migration

**Trigger**: Legacy codebase sunset, framework upgrade, platform migration, technology stack refresh

**Mental State**: Strategic, cautious, overwhelmed by complexity, planning-mode

**Goals**:
- Understand legacy codebase structure before migration
- Identify migration order (dependencies first)
- Find coupling points that complicate migration
- Assess migration risk for each component
- Plan incremental migration strategy

**Success Criteria**:
- Dependency graph of legacy system mapped
- Migration order determined (leaves first, roots last)
- High-risk coupling points identified
- Migration phasing plan created
- Rollback strategy prepared for each phase

**Pain Points**:
- Spaghetti dependencies (everything depends on everything)
- God classes touching all parts of system
- Implicit dependencies (config, shared state)
- No tests to verify migration correctness
- Business pressure to migrate quickly vs. technical reality

**Time Pressure**: Medium (migration is multi-month project, but deadlines exist)

**Frequency**: Once per major migration (rare but critical)

**Questions They Ask**:
- "What depends on this legacy module?"
- "What's the right order to migrate these components?"
- "Are there circular dependencies blocking incremental migration?"
- "What's the blast radius if we migrate this first?"
- "Which components are isolated enough to migrate safely?"

**Parseltongue Endpoints**:
- `/circular-dependency-detection-scan` - **Critical**: Find blockers
- `/reverse-callers-query-graph?entity=X` - **Fan-in analysis**
- `/semantic-cluster-grouping-list` - **Module identification**
- `/complexity-hotspots-ranking-view?top=20` - **Identify god classes**
- `/blast-radius-impact-analysis?entity=X&hops=2` - **Coupling assessment**
- `/dependency-edges-list-all` - **Full dependency mapping**

**Design Voice**: Martin Fowler's refactoring patterns - Strangler Fig pattern for legacy migration

---

### Journey 5: API Contract Validator's Interface Mapping

**Persona**: Backend developer maintaining API contracts or integrating with external services

**Trigger**: API version upgrade, breaking change detection, integration testing, contract validation

**Mental State**: Detail-oriented, verification-focused, concerned about backward compatibility

**Goals**:
- Map all public API entry points
- Understand which consumers call which endpoints
- Detect breaking changes in API contracts
- Identify deprecated functions still in use
- Validate API documentation matches code

**Success Criteria**:
- All API entry points catalogued
- Consumer → endpoint mapping created
- Breaking changes detected with consumer impact
- Deprecated code usage quantified
- Contract validation checklist completed

**Pain Points**:
- Undocumented APIs (shadow endpoints)
- Unknown consumers (internal teams, external integrations)
- Implicit contracts (behavior not documented)
- Version skew (consumers on old API versions)
- Breaking changes discovered in production

**Time Pressure**: Medium (release cycles drive urgency)

**Frequency**: Per release (bi-weekly to monthly)

**Questions They Ask**:
- "What are all the public API functions?"
- "Who's calling this deprecated endpoint?"
- "What breaks if we change this signature?"
- "Are there undocumented endpoints being called?"
- "Which consumers need notification before we change this?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy?q=handler|api|route|endpoint|public` - **API Discovery**
- `/reverse-callers-query-graph?entity=API_FN` - **Consumer mapping**
- `/blast-radius-impact-analysis?entity=API_FN&hops=3` - **Change impact**
- `/forward-callees-query-graph?entity=API_FN` - **Implementation dependencies**
- `/code-entity-detail-view?key=API_FN` - **Signature inspection**
- `/semantic-cluster-grouping-list` - **API version clusters**

**Design Voice**: Michal Batory's pragmatic API design - contracts matter, documentation is code

---

### Journey 6: Test Coverage Explorer's Gap Analysis

**Persona**: QA engineer or developer improving test coverage

**Trigger**: Low test coverage metrics, escaped bugs, quality initiative, pre-release testing

**Mental State**: Quality-focused, systematic, concerned about risk coverage

**Goals**:
- Find untested critical functions
- Identify code paths lacking test coverage
- Prioritize testing efforts by risk
- Detect redundant tests (testing same thing)
- Map test entities to production code

**Success Criteria**:
- Untested hotspots identified
- Test gap quantified by risk (high fan-in + no tests = priority)
- Redundant tests detected
- Test coverage plan prioritized
- Production code → test mapping created

**Pain Points**:
- Coverage metrics lie (lines covered vs. scenarios tested)
- Untested critical paths (high fan-in functions with no tests)
- Test redundancy (same path tested multiple times)
- Missing edge cases in tests
- Hard to know what's actually risky

**Time Pressure**: Low (quality is ongoing, but releases drive urgency)

**Frequency**: Per sprint (weekly to bi-weekly)

**Questions They Ask**:
- "Which critical functions have no tests?"
- "What code paths are completely untested?"
- "Are we testing the same thing multiple times?"
- "Where should we focus testing efforts for maximum risk reduction?"
- "Which production code entities do these tests cover?"

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=50` - **Critical paths** (high fan-in)
- `/code-entities-list-all?entity_type=function` - **All functions**
- `/semantic-cluster-grouping-list` - **Module-level coverage**
- `/blast-radius-impact-analysis?entity=X&hops=1` - **Risk assessment**
- `/code-entities-search-fuzzy?q=test` - **Find test entities**
- `/dependency-edges-list-all` - **Filter for TEST → CODE edges** (if tagged)

**Note**: Current Parseltongue excludes TEST entities during ingestion (see README line 113: "TEST entities: 982 (excluded for optimal LLM context)"). This journey would require a new flag to include tests or a separate test-mode ingestion.

**Design Voice**: Google's Testing on the Toilet - pragmatic, risk-based testing strategy

---

### Journey 7: Capability Discoverer's Feature Finder

**Persona**: Developer joining existing codebase, technical PM, or solution architect

**Trigger**: Need to understand "what can this system do?", feature discovery for integration, capability assessment

**Mental State**: Exploratory, curious, pattern-seeking, vocabulary-building

**Goals**:
- Discover all features/capabilities in codebase
- Understand feature relationships and dependencies
- Find reusable components and utilities
- Identify feature boundaries (what's implemented vs. not)
- Build mental dictionary of system capabilities

**Success Criteria**:
- Feature inventory created (what the system can do)
- Feature clusters identified (related capabilities)
- Reusable utilities catalogued
- Feature dependencies mapped
- Capability gaps identified (what's NOT implemented)

**Pain Points**:
- Undocumented features (code has capabilities no one knows about)
- Scattered implementations (same feature in multiple places)
- Discovery by word-of-mouth (inefficient)
- No clear feature boundaries
- Reinventing the wheel (not knowing existing capability exists)

**Time Pressure**: Low (exploration takes time, but initial orientation matters)

**Frequency**: Once per team/codebase, then periodic re-discovery

**Questions They Ask**:
- "What features does this system actually have?"
- "Is there already an implementation for X?"
- "What's the difference between these similar functions?"
- "What utilities can I reuse?"
- "Where is feature X implemented?"

**Parseltongue Endpoints**:
- `/code-entities-list-all` - **Browse everything**
- `/code-entities-search-fuzzy?q=KEYWORD` - **Keyword search**
- `/semantic-cluster-grouping-list` - **Feature clusters**
- `/code-entity-detail-view?key=X` - **Understand implementation**
- `/forward-callees-query-graph?entity=X` - **See dependencies**
- `/complexity-hotspots-ranking-view?top=20` - **Find core capabilities**

**Design Voice**: Don Norman's affordance theory - understanding what actions are possible

---

### Journey 8: Dead Code Eliminator's Pruning Plan

**Persona**: Senior developer or tech lead reducing technical debt

**Trigger**: Codebase cleanup initiative, repository size concerns, maintenance burden reduction

**Mental State**: Cautious, analytical, reduction-focused, concerned about breaking things

**Goals**:
- Identify unused functions, modules, and dependencies
- Quantify risk of removing code
- Find code that appears unused but is critical (reflection, dynamic calls)
- Create safe deletion plan
- Verify deletion doesn't break tests

**Success Criteria**:
- Unused code candidates identified with confidence levels
- Deletion risk assessed (fan-in = 0 = safe candidate)
- Dynamic usage patterns detected (plugins, reflection)
- Pruning plan created with rollback strategy
- Tests verify deletion safety

**Pain Points**:
- False positives (code looks unused but is called dynamically)
- Reflection and dynamic calls (not visible in static analysis)
- Configuration-driven features (enabled/disabled at runtime)
- Fear of breaking things
- Lack of test coverage to verify safety

**Time Pressure**: Low (cleanup is ongoing work, not urgent)

**Frequency**: Quarterly cleanup cycles

**Questions They Ask**:
- "What code is completely unused?"
- "Is it safe to delete this function?"
- "Are there dynamic calls to this code?"
- "What's the risk of removing this module?"
- "How much can we reduce the codebase safely?"

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=1000` - **Filter for fan-in = 0** (unused)
- `/reverse-callers-query-graph?entity=X` - **Verify: should return empty**
- `/code-entities-search-fuzzy?q=ENTITY_NAME` - **Check for dynamic usage**
- `/semantic-cluster-grouping-list` - **Module-level usage**
- `/dependency-edges-list-all` - **Find all edges to/from candidate**
- `/code-entity-detail-view?key=X` - **Read code for hints of dynamic usage**

**Limitation**: Static analysis can't detect runtime dynamic calls (reflection, plugins). This journey requires combining static analysis with runtime profiling or grep-based heuristics.

**Design Voice**: Ed Catmull's creativity principle - "constraints foster creativity" (less code = more focus)

---

## Part 2: Eight NEW Visualization Modes

### Mode 1: Hotspot Mode (Execution Frequency Heatmap)

**User Journey**: Performance Profiler's Bottleneck Hunt

**Core Metaphor**: Thermal Camera with Heat-Induced Node Size

```
    [Ice Cold Function]          [Lukewarm Utility]
    fan-in: 1                    fan-in: 12

         [🔥🔥🔥 HOT Function 🔥🔥🔥]
              fan-in: 215
              called millions of times
```

**Layout Strategy**:
- Primary: Node size = fan-in (frequency indicator)
- Secondary: Color scale = execution cost (blue → red)
- Tertiary: Glow effect = cumulative cost (self + descendants)

**Key Features**:
- "Thermal Slider" to adjust frequency threshold
- Hover shows estimated call count
- Click to trace execution path forward
- "Hot Path" highlight (red path through frequently-called chain)
- Toggle: "Show Only Hotspots" (fan-in > threshold)

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=100`
- `/forward-callees-query-graph`
- `/reverse-callers-query-graph`
- `/blast-radius-impact-analysis`

**Time to Insight**: 45 seconds

**Visual Innovation**: Nodes pulse with animation frequency proportional to call rate

---

### Mode 2: Attack Surface Mode (Security Terrain Map)

**User Journey**: Security Auditor's Attack Surface Analysis

**Core Metaphor**: Fortress Map with Perimeter Breach Paths

```
    ┌─────────────────────────────────────┐
    │  EXTERIOR (Untrusted Zone)          │
    │                                     │
    │  [🚪 Entry Points]                  │
    │   handler   handler   CLI           │
    │      │         │         │          │
    │      ▼         ▼         ▼          │
    │  ═════════════════════════════════  │  ← Security Boundary
    │  INTERIOR (Trusted Zone)            │
    │                                     │
    │  [🔐 Security Functions]            │
    │   auth    crypto   data_access      │
    └─────────────────────────────────────┘
```

**Layout Strategy**:
- Primary: Zone-based (exterior vs. interior)
- Secondary: Entry points on perimeter, security functions in center
- Tertiary: Red lines = data flow from entry to security-critical

**Key Features**:
- "Attack Path" visualization (untrusted input → sensitive operation)
- Color code by risk level (red = auth/crypto, orange = data access, yellow = validation)
- Click entry point to trace all data flows
- "Unknown Dependency" overlay (external deps highlighted in purple)
- Export security report with entry point inventory

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy`
- `/forward-callees-query-graph`
- `/reverse-callers-query-graph`
- `/semantic-cluster-grouping-list`
- `/dependency-edges-list-all` (filter unknown:0-0)

**Time to Insight**: 2 minutes

**Visual Innovation**: Animated "intruder" dot tracing attack paths from entry points to sensitive operations

---

### Mode 3: Incident Mode (Failure Cascade Visualization)

**User Journey**: Incident Responder's Failure Cascade Mapping

**Core Metaphor**: Dominos with Knock-On Effect Prediction

```
    [Push] ──▶ [🎯 Failing Service]
                  │
                  ▼
             [Dependent A] ──▶ [User-Facing Feature 1]
                  │
                  ▼
             [Dependent B] ──▶ [User-Facing Feature 2]
                  │
                  ▼
             [Dependent C] ──▶ [Critical Payment Flow]
```

**Layout Strategy**:
- Primary: Vertical flow (failure propagates downward)
- Secondary: Color = severity (red = critical, orange = degraded, yellow = warning)
- Tertiary: Node size = user impact (how many people affected)

**Key Features**:
- Select failing component → auto-generate cascade tree
- "Time to Failure" annotation (estimated downtime per component)
- "Fix Priority" ranking (critical user features first)
- Single-point-of-failure detection (nodes whose failure affects many)
- Export incident summary with blast radius

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy`
- `/reverse-callers-query-graph`
- `/blast-radius-impact-analysis?hops=3`
- `/complexity-hotspots-ranking-view`
- `/semantic-cluster-grouping-list`

**Time to Insight**: 60 seconds

**Visual Innovation**: Animated domino toppling effect to visualize failure propagation

---

### Mode 4: Migration Mode (Dependency Unraveling Tree)

**User Journey**: Legacy Migrator's Dependency Unraveling

**Core Metaphor**: Untangling Yarn with Migration Order Guidance

```
    [🟢 Ready to Migrate]         (no dependents)
         leaf_fn_1   leaf_fn_2

    [🟡 Migrate After Above]     (depends on green)
         mid_component

    [🔴 Migrate Last]             (many dependents)
         legacy_god_class
```

**Layout Strategy**:
- Primary: Bottom-up tree (dependents above, dependencies below)
- Secondary: Color = migration readiness (green = ready, yellow = depends on green, red = blocked)
- Tertiary: Node size = migration complexity (LOC, cyclomatic complexity)

**Key Features**:
- "Migration Order" overlay (numbered steps: 1, 2, 3...)
- Circular dependency warning (red ring around blocked components)
- "Island Detection" (isolated components safe to migrate first)
- Effort estimation per component
- Export migration plan with phasing

**Parseltongue Endpoints**:
- `/circular-dependency-detection-scan`
- `/reverse-callers-query-graph`
- `/semantic-cluster-grouping-list`
- `/complexity-hotspots-ranking-view`
- `/blast-radius-impact-analysis`

**Time to Insight**: 3 minutes

**Visual Innovation**: Animated "unraveling" transition where tangled knot smoothly unwinds into ordered tree

---

### Mode 5: Contract Mode (API Interface Catalog)

**User Journey**: API Contract Validator's Interface Mapping

**Core Metaphor**: Library Card Catalog with Consumer Check-Out Cards

```
┌─────────────────────────────────────────┐
│  PUBLIC API CARD CATALOG                │
├─────────────────────────────────────────┤
│  📇 POST /users/create                  │
│      └── Consumers: 3                   │
│          ├── Frontend App              │
│          ├── Mobile App                │
│          └── External Partner          │
│                                         │
│  📇 GET /users/:id                      │
│      └── Consumers: 5                   │
│                                         │
│  📇 POST /auth/login                    │
│      └── Consumers: 2                   │
└─────────────────────────────────────────┘
```

**Layout Strategy**:
- Primary: Card catalog grid (one card per API function)
- Secondary: Card size = consumer count (bigger = more widely used)
- Tertiary: Color = contract stability (green = stable, yellow = recently changed, red = deprecated)

**Key Features**:
- Click card to show all consumers
- "Breaking Change" simulator (modify signature → see affected consumers)
- "Deprecated API" detection (still has consumers)
- "Undocumented API" warning (no consumers = likely internal or unlisted)
- Export API inventory with consumer mapping

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy`
- `/reverse-callers-query-graph`
- `/blast-radius-impact-analysis`
- `/code-entity-detail-view`
- `/semantic-cluster-grouping-list`

**Time to Insight**: 90 seconds

**Visual Innovation**: 3D card flip animation - front shows API signature, back shows consumer list

---

### Mode 6: Test Gap Mode (Coverage Heatmap with Risk Overlay)

**User Journey**: Test Coverage Explorer's Gap Analysis

**Core Metaphor**: Radar Map with Blind Spot Detection

```
    [🟢 Fully Tested]  [🟡 Partially Tested]  [🔴 Untested Risk]

    Complexity vs. Coverage Scatter Plot:

    High    ●  🔴 HOTSPOT
            ●     (fan-in=50, tests=0)
    Risk
            ●  🟡 Some coverage
    Low     ●  🟢 Tested
            └────────────────────────
                Low    Test Coverage    High
```

**Layout Strategy**:
- Primary: Scatter plot (x-axis = test coverage, y-axis = complexity/fan-in)
- Secondary: Color = risk level (red = high complexity + no tests)
- Tertiary: Node size = blast radius (impact if broken)

**Key Features**:
- "Untested Hotspots" quadrant (top-left = high complexity, low coverage)
- Click node to see what tests exist (if any)
- "Test This" button generates test recommendation
- "Redundant Tests" detection (multiple tests covering same path)
- Export test gap prioritized list

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=100`
- `/code-entities-list-all`
- `/semantic-cluster-grouping-list`
- `/blast-radius-impact-analysis`
- `/code-entities-search-fuzzy?q=test`

**Time to Insight**: 2 minutes

**Visual Innovation**: "Radar sweep" animation that reveals untested areas as dark shadows

---

### Mode 7: Discovery Mode (Feature Explorer with Affordance Cards)

**User Journey**: Capability Discoverer's Feature Finder

**Core Metaphor**: Museum Exhibit with Interactive Discovery Kiosks

```
┌─────────────────────────────────────────┐
│  FEATURE DISCOVERY HALL                 │
├─────────────────────────────────────────┤
│                                         │
│  🏛️  [Authentication Wing]              │
│      ├── login_handler                  │
│      ├── password_validator             │
│      └── token_manager                  │
│                                         │
│  🏛️  [Data Processing Wing]            │
│      ├── parser_engine                  │
│      ├── transformer_pipeline           │
│      └── export_formatter               │
│                                         │
│  🔍 [Search: _______________]           │
└─────────────────────────────────────────┘
```

**Layout Strategy**:
- Primary: Museum wings by semantic cluster (feature groups)
- Secondary: Exhibit cases (functions) within wings
- Tertiary: "What Can I Do?" affordance labels on each exhibit

**Key Features**:
- Keyword search with fuzzy matching
- Click exhibit to see "What this does" (code preview)
- "Similar Features" suggestion (find related capabilities)
- "Reuse This" button (copies code pattern)
- "Feature Graph" (show relationships between features)

**Parseltongue Endpoints**:
- `/code-entities-list-all`
- `/code-entities-search-fuzzy`
- `/semantic-cluster-grouping-list`
- `/code-entity-detail-view`
- `/forward-callees-query-graph`

**Time to Insight**: Variable (exploration mode)

**Visual Innovation**: Spotlight effect on hover, illuminating related features like museum lighting

---

### Mode 8: Pruning Mode (Dead Code Detection with Confidence Levels)

**User Journey**: Dead Code Eliminator's Pruning Plan

**Core Metaphor**: Garden Pruning with Regrowth Risk Assessment

```
    🌳 [Healthy Code]         (fan-in > 0, actively used)
        └── Keep and maintain

    🍂 [Dead Leaves]          (fan-in = 0, safe to prune)
        ├── orphan_function   ✅ Safe to delete
        └── unused_utility    ✅ Safe to delete

    ⚠️ [Dormant Seeds]        (fan-in = 0, but dynamic usage suspected)
        ├── plugin_hook       ⚠️  Verify reflection/plugins first
        └── config_driven     ⚠️  Check runtime configuration
```

**Layout Strategy**:
- Primary: Tree structure (alive vs. dead branches)
- Secondary: Color = confidence level (green = safe, yellow = verify, red = critical)
- Tertiary: Withered appearance for dead code (faded, desaturated)

**Key Features**:
- "Fan-In = 0" filter (show only unused code)
- "Dynamic Usage" warning (heuristic: words like "plugin", "hook", "reflect")
- "Prune This" action with confirmation dialog
- "Deletion Risk Score" (0 = safe, 100 = dangerous)
- Export pruning plan with rollback strategy

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=1000` (filter fan-in = 0)
- `/reverse-callers-query-graph?entity=X` (verify empty)
- `/code-entities-search-fuzzy?q=ENTITY_NAME` (dynamic usage check)
- `/dependency-edges-list-all`
- `/code-entity-detail-view`

**Time to Insight**: 3 minutes

**Visual Innovation**: Falling leaves animation (dead code slowly withers and falls when deleted)

---

## Part 3: Comparison Matrix - Version 1 vs. Version 2

| Dimension | V1: Orientation | V1: Archaeology | V1: Bug Hunt | **V2: Hotspot** |
|-----------|-----------------|-----------------|--------------|-----------------|
| **Primary Question** | What am I looking at? | How does this work? | Why is this broken? | **Why is this slow?** |
| **Persona** | New developer | Senior developer | Bug investigator | **Performance engineer** |
| **Trigger** | First day on job | Cross-team work | Error report | **Latency spike** |
| **Time Pressure** | Medium | Medium | Critical | **High** |
| **Metaphor** | Circular city | Galaxy cluster | Reverse tree | **Thermal camera** |
| **Primary Endpoint** | `/semantic-cluster` | `/forward-callees` | `/reverse-callers` | **`/complexity-hotspots`** |

| Dimension | V1: Refactor | V1: Impact | V1: Audit | **V2: Security** |
|-----------|--------------|------------|----------|-----------------|
| **Primary Question** | What breaks if I change? | Is architecture sound? | Where is tech debt? | **Where are attack surfaces?** |
| **Persona** | Developer | Architect | Tech lead | **Security engineer** |
| **Trigger** | Code improvement | Pre-deployment | Quarterly review | **Security audit** |
| **Time Pressure** | Medium | Low | Low | **Medium** |
| **Metaphor** | Ripple effect | Terrain map | Heatmap grid | **Fortress map** |
| **Primary Endpoint** | `/blast-radius` | `/circular-deps` | `/hotspots` | **`/search + /forward`** |

| Dimension | V1: Review | V1: Teach | **V2: Incident** | **V2: Migration** |
|-----------|------------|-----------|------------------|------------------|
| **Primary Question** | What changed? | How do I explain? | **What's failing?** | **How do we migrate?** |
| **Persona** | Code reviewer | Teacher | **SRE / On-call** | **Legacy migrator** |
| **Trigger** | PR submitted | Documentation needed | **Production alert** | **Tech stack refresh** |
| **Time Pressure** | Medium-High | Low | **Critical** | **Medium** |
| **Metaphor** | Split diff | Story path | **Domino cascade** | **Yarn unraveling** |
| **Primary Endpoint** | `/blast-radius` | `/forward-callees` | **`/reverse-callers`** | **`/circular-deps`** |

| Dimension | **V2: Contract** | **V2: Test Gap** | **V2: Discovery** | **V2: Pruning** |
|-----------|------------------|------------------|-------------------|----------------|
| **Primary Question** | **Who uses this API?** | **What's untested?** | **What can this do?** | **What's dead code?** |
| **Persona** | **API maintainer** | **QA engineer** | **Capability explorer** | **Debt cleaner** |
| **Trigger** | **API version change** | **Quality initiative** | **Feature discovery** | **Codebase cleanup** |
| **Time Pressure** | **Medium** | **Low** | **Low** | **Low** |
| **Metaphor** | **Card catalog** | **Radar map** | **Museum exhibit** | **Garden pruning** |
| **Primary Endpoint** | **`/reverse-callers`** | **`/hotspots + /search`** | **`/list-all + /search`** | **`/hotspots (fan-in=0)`** |

---

## Part 4: Distinctiveness Analysis

### What Makes Version 2 Fundamentally Different?

#### 1. **Role-Specific vs. General**
- **Version 1**: General developers doing common tasks
- **Version 2**: Specialists (SRE, Security, QA, Performance, Legacy Architect)

#### 2. **Runtime vs. Static Code**
- **Version 1**: Static code structure (dependencies, architecture)
- **Version 2**: Runtime behavior (performance, failures, security, production)

#### 3. **Temporal Focus**
- **Version 1**: Current code state
- **Version 2**: Historical (migration), predictive (incident), comparative (test coverage)

#### 4. **Emotional Context**
- **Version 1**: Curiosity, caution, learning
- **Version 2**: Urgency (incident, performance), vigilance (security), reduction (pruning)

#### 5. **Success Metrics**
- **Version 1**: Understanding, safety, knowledge transfer
- **Version 2**: Latency reduced, attack surface secured, downtime minimized, code deleted

#### 6. **Endpoint Emphasis**
- **Version 1**: Broad exploration (semantic clusters, blast radius, circular deps)
- **Version 2**: Targeted queries (hotspots fan-in, reverse callers for specific entities, search patterns)

### Unique Endpoint Combinations

| Journey | Unique Endpoint Combination | Why It's New |
|---------|---------------------------|--------------|
| **Hotspot** | `complexity-hotspots` (fan-in as frequency) | Interpreting coupling as runtime cost |
| **Security** | `search` + `forward-callees` + `edges (unknown:0-0)` | Tracing untrusted input through system |
| **Incident** | `reverse-callers` + `blast-radius (hops=2)` | Understanding failure propagation |
| **Migration** | `circular-deps` as blockers | Using cycles to determine migration order |
| **Contract** | `search` patterns + `reverse-callers` | Mapping consumers to API producers |
| **Test Gap** | `hotspots` (risk) + `search test` | Risk-weighted coverage analysis |
| **Discovery** | `list-all` browsing + `semantic clusters` | Capability inventory, not dependency tracing |
| **Pruning** | `hotspots` (filter fan-in=0) | Inverting complexity metric to find dead code |

---

## Part 5: Implementation Roadmap

### Phase 1: High-Value Modes (Weeks 1-4)

**Week 1-2: Hotspot Mode**
- Extend existing CodeCity with frequency-based node sizing
- Use `/complexity-hotspots-ranking-view` for data
- Add thermal color scale and pulse animation
- **Value**: Immediate performance insights

**Week 3-4: Incident Mode**
- Reverse tree layout (different from Bug Hunt's focus on root cause)
- Domino animation for failure propagation
- Blast radius integration
- **Value**: Critical for production incidents

### Phase 2: Specialized Roles (Weeks 5-8)

**Week 5-6: Security Mode**
- Zone-based layout (exterior vs. interior)
- Attack path tracing animation
- Unknown dependency overlay
- **Value**: Security audits, compliance

**Week 7-8: Migration Mode**
- Bottom-up dependency tree
- Circular dependency blocking detection
- Migration order numbering
- **Value**: Legacy migrations (high-cost, high-risk projects)

### Phase 3: Quality & Maintenance (Weeks 9-12)

**Week 9-10: Test Gap Mode**
- Scatter plot visualization (new layout type)
- Risk-weighted coverage analysis
- Test recommendation engine
- **Value**: Quality improvement

**Week 11-12: Pruning Mode**
- Filter for fan-in = 0
- Dynamic usage heuristics
- Confidence level indicators
- **Value**: Technical debt reduction

### Phase 4: Discovery & Contracts (Weeks 13-16)

**Week 13-14: Discovery Mode**
- Museum exhibit metaphor
- Feature browsing UI
- Affordance labeling
- **Value**: Onboarding for exploration-focused learners

**Week 15-16: Contract Mode**
- Card catalog interface
- Consumer mapping
- Breaking change simulator
- **Value**: API maintenance, integration planning

---

## Part 6: Design Principles for Version 2

| Principle | Application in V2 | Example |
|-----------|-------------------|---------|
| **Urgency-appropriate visuals** | Critical journeys get faster, simpler interfaces | Incident Mode: single-click cascade visualization |
| **Role-specific language** | Use terminology the persona understands | Security Mode: "attack surface" not "fan-in" |
| **Confidence levels** | Show uncertainty in analysis | Pruning Mode: 3-tier confidence (safe/verify/critical) |
| **Temporal context** | Show time-based patterns | Hotspot Mode: animation speed = call frequency |
| **Risk-weighted priorities** | Rank by risk, not just metrics | Test Gap Mode: complexity × coverage gap = priority |
| **Export-driven** | Specialists need reports for stakeholders | All V2 modes have export functionality |
| **Heuristic overlay** | Combine static analysis with heuristics | Pruning Mode: name-based detection of dynamic usage |
| **Progressive disclosure** | Start simple, reveal complexity | Discovery Mode: wings → exhibits → details |

---

## Part 7: Success Metrics by Journey

| Journey | Primary Metric | Secondary Metrics |
|---------|----------------|-------------------|
| **Hotspot** | Bottlenecks identified and optimized | Latency reduction, throughput increase |
| **Security** | Attack surface documented | Vulnerabilities found, penetration test time |
| **Incident** | Mean Time to Resolution (MTTR) | Incident recurrence, blast radius accuracy |
| **Migration** | Migration completed without rollback | Migration timeline accuracy, risk predictions |
| **Contract** | Breaking changes detected before deploy | API documentation accuracy, consumer notifications |
| **Test Gap** | Untested hotspots covered | Escape rate (bugs in production), coverage ROI |
| **Discovery** | Time to find reusable code | Duplication reduction, capability inventory accuracy |
| **Pruning** | Dead code safely deleted | Codebase size reduction, maintenance burden |

---

## Part 8: Cross-Version Synergies

### Mode Combinations

| Version 1 Mode | Version 2 Mode | Synergy |
|----------------|----------------|---------|
| **Bug Hunt** + **Incident** | Debug mode → Production mode | Development bugs vs. production failures |
| **Refactor** + **Hotspot** | Structural safety → Runtime cost | Safe changes that also improve performance |
| **Audit** + **Security** | General tech debt → Security-specific debt | Holistic health assessment |
| **Review** + **Contract** | Code changes → API contract impact | PR reviews check API compatibility |
| **Orient** + **Discovery** | Architecture learning → Capability learning | Structure + functionality understanding |
| **Teach** + **Migration** | Knowledge transfer → Migration planning | Teaching team about legacy before migration |

### Shared Visual Patterns

| Visual Pattern | Used In V1 | Used In V2 | Meaning |
|----------------|------------|------------|---------|
| **Radial layout** | Refactor (blast radius) | Incident (failure cascade) | Impact propagation |
| **Tree layout** | Bug Hunt (reverse) | Migration (bottom-up) | Dependency hierarchy |
| **Heatmap** | Audit (complexity) | Hotspot (frequency) | Intensity gradient |
| **Cluster grouping** | Orient (modules) | Discovery (features) | Logical grouping |
| **Flow animation** | Archaeology (calls) | Security (attack path) | Directional movement |

---

## Part 9: Technical Considerations

### New Data Requirements

| Journey | Data Need | Current Parseltongue Support | Gap |
|---------|-----------|-----------------------------|-----|
| **Hotspot** | Execution frequency | Partial (fan-in ≈ frequency) | Need actual profiling data |
| **Security** | Security metadata tags | None | Need to classify entities (auth, crypto) |
| **Incident** | Runtime health metrics | None | External monitoring integration needed |
| **Migration** | Component complexity | Partial (LOC via code-entity-detail) | Cyclomatic complexity not exposed |
| **Contract** | API/Public markers | None | Need to tag public interfaces |
| **Test Gap** | Test entities included | **Excluded** (see README line 113) | Need flag to include tests |
| **Discovery** | Search by keywords | Supported via fuzzy search | None |
| **Pruning** | Fan-in = 0 detection | Supported | None |

### Performance Considerations

| Mode | Rendering Load | Data Volume | Optimization Strategy |
|------|----------------|-------------|----------------------|
| **Hotspot** | Medium (pulse animations) | High (all entities) | LOD for distant nodes |
| **Security** | Medium (zone layout) | Medium (entry points) | Lazy load security clusters |
| **Incident** | Low (tree layout) | Low (single component focus) | None needed |
| **Migration** | High (full dep tree) | High (all edges) | Progressive rendering |
| **Contract** | Low (card grid) | Low (API functions only) | None needed |
| **Test Gap** | Medium (scatter plot) | High (all entities) | Canvas rendering |
| **Discovery** | Medium (museum layout) | Medium (browse all) | Virtual scrolling |
| **Pruning** | Medium (wither animation) | High (filter fan-in=0) | Server-side filtering |

---

## Part 10: Research Sources & Design Voices

### Version 2 Specific Design Voices

| Voice | Contribution | Applied In |
|-------|--------------|------------|
| **Jay Doblin** | Systems thinking, feedback loops | Hotspot Mode (performance cascades) |
| **Bruce Schneier** | Security mindset, "think attacker" | Security Mode (attack paths) |
| **John Allspaw** | Blameless postmortems, complex systems | Incident Mode (failure propagation) |
| **Martin Fowler** | Refactoring patterns, Strangler Fig | Migration Mode (incremental migration) |
| **Michal Batory** | Pragmatic API design | Contract Mode (API boundaries) |
| **Google Testing Team** | Test on the Toilet, risk-based testing | Test Gap Mode (coverage prioritization) |
| **Don Norman** | Affordance theory | Discovery Mode (what's possible) |
| **Ed Catmull** | Creativity constraints | Pruning Mode (less = more focus) |

### Academic Research References

| Topic | Key Research | Application |
|-------|--------------|-------------|
| **Performance visualization** | Knight & Munro "Program visualization for debugging" | Hotspot thermal camera |
| **Security attack graphs** | Noel & Jajodia "Managing attack graph complexity" | Security fortress map |
| **Incident response** | Van Der Werff et al. "Visualizing system failures" | Incident domino cascade |
| **Legacy migration** | Brodie & Stonebraker "Migrating legacy systems" | Migration dependency unraveling |
| **API evolution** | Raemaekers et al. "API usage mining" | Contract card catalog |
| **Test coverage** | Gao & Shao "Test adequacy criteria" | Test Gap radar map |
| **Feature discovery** | Robbes & Lanza "Software visualization for feature location" | Discovery museum exhibit |
| **Dead code** | Kanellopoulos et al. "Code smell detection" | Pruning garden metaphor |

---

## Appendix: Quick Reference Card

```
┌─────────────────────────────────────────────────────────────────┐
│  PARSELTONGUE VISUALIZATION - VERSION 2 USER JOURNEYS           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  🔥 HOTSPOT MODE      - Performance bottlenecks                │
│  🛡️ SECURITY MODE     - Attack surface mapping                 │
│  🚨 INCIDENT MODE     - Failure cascade visualization           │
│  🚚 MIGRATION MODE    - Dependency unraveling                  │
│  📋 CONTRACT MODE     - API interface catalog                  │
│  🧪 TEST GAP MODE     - Coverage blind spot detection           │
│  🔍 DISCOVERY MODE    - Feature capability explorer            │
│  ✂️ PRUNING MODE      - Dead code elimination plan             │
│                                                                 │
│  DISTINGUISHING FACTORS:                                        │
│  • Role-specific personas (SRE, Security, QA, Performance)     │
│  • Runtime focus (not just static code)                        │
│  • Production contexts (incidents, migrations, audits)          │
│  • Urgency-appropriate interfaces                               │
│  • Risk-weighted prioritization                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sources

### Version 1 Foundation
- `/docs/web-ui/EIGHT_USER_JOURNEYS_FOR_PARSELTONGUE_VISUALIZATION.md` - Original 8 journeys
- `/docs/web-ui/DESIGN_OPTIONS_FOR_PARSELTONGUE_VISUALIZATION.md` - Design philosophy
- `/README.md` - Parseltongue API capabilities (15 endpoints)

### Research Documents
- `/influential-design-voices-2005-2025.md` - 50+ design voices
- `/docs/RESEARCH_Visualization_Improvements_20260110.md` - Original research
- `/docs/web-ui/INTERFACE_SIGNATURE_GRAPH_THESIS.md` - Visualization thesis

### Version 2 New Sources
- Jay Doblin - "Unifying" design theory and systems thinking
- Bruce Schneier - "Beyond Fear" and security mindset
- John Allspaw - "Web Operations" and blameless postmortems
- Martin Fowler - "Refactoring" and Strangler Fig pattern
- Google Testing Blog - "Testing on the Toilet" series
- Don Norman - "The Design of Everyday Things" (affordances)
- Ed Catmull - "Creativity, Inc." (constraints and focus)

---

**Document Version**: 2.0
**Total User Journeys**: 8 (NEW) + 8 (V1) = 16 Total
**Total Visualization Modes**: 8 (NEW) + 8 (V1) = 16 Total
**New Personas Covered**: SRE, Security Engineer, Performance Engineer, QA Engineer, Legacy Architect, API Maintainer, Capability Explorer, Debt Cleaner
**Research Sources**: 50+ design voices + academic research + industry best practices
**Generated**: 2026-01-14
**Branch**: `research/visualization-improvements-20260110-1914`

---

*Version 1 focuses on what every developer needs. Version 2 focuses on what specialists need. Together, they provide comprehensive coverage of the modern software development lifecycle.*
