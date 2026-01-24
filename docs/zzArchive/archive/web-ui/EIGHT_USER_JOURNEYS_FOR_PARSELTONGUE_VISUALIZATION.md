# Eight User Journeys for Parseltongue Visualization

**A Comprehensive Design Document Applying 20 Years of Design Thinking to Code Dependency Visualization**

**Date**: 2025-01-14
**Status**: Design Research Complete
**Context**: Parseltongue Dependency Graph Generator - 239 entities, 211 dependency arcs

---

## Executive Summary

This document presents **eight distinctly different developer user journeys** that drive visualization design for the Parseltongue code dependency graph tool. Each journey represents a unique mental state, goal, time pressure, and success criteria.

Based on research from 50+ influential design voices (2005-2025) and academic studies on developer cognition, we define:
- **8 user personas** with specific contexts and goals
- **8 visualization modes** optimized for each journey
- **8 core metaphors** that make the complex understandable
- **Implementation plans** with Parseltongue API endpoint mappings

---

## Part 1: The Eight User Journeys

### Journey 1: Newcomer Onboarding

**Persona**: Junior to mid-level developer joining a new team

**Trigger**: First day on the job, need to understand unfamiliar codebase

**Mental State**: Overwhelmed, curious, anxious to prove value, afraid of breaking things

**Goals**:
- Build mental model of codebase structure
- Identify where different features live
- Understand architectural patterns
- Find first safe area to contribute

**Success Criteria**:
- Can navigate to relevant code without basic questions
- Understands main modules and relationships
- Identifies 2-3 safe areas for initial contributions
- Can explain architecture to another new joiner

**Pain Points**:
- Information overload: Documentation outdated/scattered
- Fear of "stupid questions"
- Lack of spatial orientation
- Missing context on why code exists

**Time Pressure**: Medium (first 2-4 weeks critical)

**Frequency**: One-time per team/role

**Questions They Ask**:
- "What does this codebase actually do?"
- "Where is the authentication logic?"
- "How do modules connect?"
- "What's the safest place for my first change?"

**Parseltongue Endpoints**:
- `/codebase-statistics-overview-summary` - Quick scale assessment
- `/semantic-cluster-grouping-list` - Module boundaries
- `/complexity-hotspots-ranking-view` - Areas to avoid initially
- `/code-entities-search-fuzzy` - Find functionality

**Design Voice**: Kathy Sierra's competence curve - progressive skill building

---

### Journey 2: Cross-Team Feature Exploration

**Persona**: Senior developer or tech lead working across teams

**Trigger**: Need to integrate with another team's system

**Mental State**: Strategic, curious, time-constrained, focused on integration

**Goals**:
- Understand how another team's code works
- Find integration points and APIs
- Assess complexity and risk
- Identify who to contact

**Success Criteria**:
- Identify all integration points
- Understand data flow across boundaries
- Know team ownership
- Assessed integration complexity

**Pain Points**:
- Siloed knowledge (other teams = black box)
- Outdated documentation
- Hidden dependencies
- Coordination overhead

**Time Pressure**: High (integration deadlines aggressive)

**Frequency**: Weekly to monthly

**Questions They Ask**:
- "What does Team X's system do?"
- "Where are the APIs?"
- "What's the blast radius if this service goes down?"
- "Are there circular dependencies between systems?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy` - Find API endpoints
- `/reverse-callers-query-graph` - See who calls which functions
- `/blast-radius-impact-analysis` - Cross-team dependencies
- `/circular-dependency-detection-scan` - Problematic coupling

**Design Voice**: Shreyas Doshi's intent-driven design

---

### Journey 3: Bug Hunter's Root Cause Investigation

**Persona**: Developer debugging production issue

**Trigger**: Error report, failed test, unexpected behavior

**Mental State**: Frustrated but focused, narrowing hypothesis space, urgent

**Goals**:
- Trace from symptom to root cause
- Understand execution path
- Identify all code that could cause issue
- Find similar patterns with same bug

**Success Criteria**:
- Root cause identified with evidence
- Full call stack understood
- All affected paths identified
- Fix proposed with confidence

**Pain Points**:
- Symptom-cause gap (error appears in one place, caused elsewhere)
- Hidden dependencies (config/external services not obvious)
- Complex call chains
- Temporal coupling (files change together but no code edge)

**Time Pressure**: Critical (production bugs need immediate attention)

**Frequency**: Daily for some, weekly for others

**Questions They Ask**:
- "Where is this error actually coming from?"
- "What calls this function?"
- "What changed recently?"
- "Are there other places with this bug pattern?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy` - Locate error-related code
- `/code-entity-detail-view` - See actual code
- `/reverse-callers-query-graph` - Trace backwards from error
- `/temporal-coupling-hidden-deps` - **Killer feature for bugs**
- `/blast-radius-impact-analysis` - Check if fix affects other things

**Design Voice**: Julie Zhuo's "start with the problem"

---

### Journey 4: Safe Refactorer's Impact Analysis

**Persona**: Experienced developer planning code changes

**Trigger**: Need to improve code quality, update dependencies, restructure

**Mental State**: Cautious, analytical, risk-averse, thorough

**Goals**:
- Understand complete impact of planned changes
- Identify all code needing updates
- Assess risk level
- Plan test coverage
- Find hidden coupling

**Success Criteria**:
- Full list of affected functions/modules
- Risk score quantified (low/medium/high)
- Test coverage plan created
- Hidden dependencies surfaced
- Rollback strategy prepared

**Pain Points**:
- Unknown ripple effects
- Hidden coupling (files must change together)
- Cycle involvement (high risk)
- Testing blind spots

**Time Pressure**: Medium (need to get it right)

**Frequency**: Weekly to monthly

**Questions They Ask**:
- "What breaks if I change this?"
- "How many places call this?"
- "Is this in a circular dependency?"
- "What files change together with this?"

**Parseltongue Endpoints**:
- `/reverse-callers-query-graph` - Direct fan-in
- `/blast-radius-impact-analysis` - Transitive impact
- `/circular-dependency-detection-scan` - Check cycle involvement
- `/temporal-coupling-hidden-deps` - Files that must change together

**Design Voice**: Ryan Singer's Shape Up - clear appetite, bounded scope

---

### Journey 5: Feature Developer's Placement Decision

**Persona**: Developer implementing new feature

**Trigger**: Product requirement ready for implementation

**Mental State**: Creative, exploratory, uncertain, seeking patterns

**Goals**:
- Find right place to add new code
- Identify similar patterns to follow
- Understand existing conventions
- Locate related code to avoid duplication
- Assess impact on existing modules

**Success Criteria**:
- New code in architecturally appropriate location
- Follows existing patterns
- Reuses existing code appropriately
- Doesn't introduce unwanted dependencies
- Fits naturally into module structure

**Pain Points**:
- Placement paralysis (uncertainty)
- Pattern blindness (don't know conventions)
- Duplication risk
- Architectural violation
- Missing context

**Time Pressure**: Medium (feature deadlines exist)

**Frequency**: Daily (active development)

**Questions They Ask**:
- "Where should I add this?"
- "How have similar features been implemented?"
- "What code can I reuse?"
- "Which module does this belong in?"

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy` - Find similar functionality
- `/semantic-cluster-grouping-list` - See appropriate module
- `/forward-callees-query-graph` - Understand patterns
- `/code-entity-detail-view` - Study examples
- `/complexity-hotspots-ranking-view` - Avoid over-burdening hotspots

**Design Voice**: Jakob Nielsen's "recognition rather than recall"

---

### Journey 6: Architecture Auditor's Health Assessment

**Persona**: Tech lead, principal engineer, architect

**Trigger**: Quarterly review, tech debt assessment, pre-refactor planning

**Mental State**: Analytical, strategic, concerned about maintainability

**Goals**:
- Assess overall codebase health
- Identify technical debt hotspots
- Find architectural violations
- Prioritize refactor efforts
- Communicate findings to stakeholders

**Success Criteria**:
- Comprehensive health score calculated
- Top 10 technical debt items prioritized
- Architecture violations documented
- Action plan with resource estimates
- Clear communication to stakeholders

**Pain Points**:
- Metric overload (too many metrics)
- Incomplete picture (tools miss relationships)
- Communication gap (hard to explain to non-engineers)
- False positives
- Lack of context (symptoms without root causes)

**Time Pressure**: Low (can be thorough)

**Frequency**: Quarterly to biannually

**Questions They Ask**:
- "What's the overall health?"
- "Where are our biggest risks?"
- "Do we have circular dependencies?"
- "Which modules are too tightly coupled?"
- "What should we refactor first?"

**Parseltongue Endpoints**:
- `/codebase-statistics-overview-summary` - Overall metrics
- `/circular-dependency-detection-scan` - Find cycles
- `/complexity-hotspots-ranking-view` - Identify god classes
- `/semantic-cluster-grouping-list` - Assess module cohesion
- `/blast-radius-impact-analysis` - Ecosystem impact

**Design Voice**: Teresa Torres' Opportunity Solution Tree

---

### Journey 7: Documentation Generator's Knowledge Capture

**Persona**: Developer or tech writer creating/updating documentation

**Trigger**: New hire, team reorg, outdated docs noticed

**Mental State**: Knowledge-transfer focused, thorough, considering future readers

**Goals**:
- Create accurate architectural diagrams
- Document module relationships
- Explain data flow
- Capture implicit knowledge
- Make documentation maintainable

**Success Criteria**:
- Documentation matches current code
- Diagrams show key relationships
- New hires can onboard faster
- Maintenance process established
- Implicit knowledge made explicit

**Pain Points**:
- Documentation drift (docs don't match code)
- Manual diagram maintenance (tedious)
- Implicit knowledge (things "everyone knows")
- Visual overwhelm (too much detail)
- Update friction

**Time Pressure**: Low (quality over speed)

**Frequency**: Monthly to quarterly

**Questions They Ask**:
- "What are the main components?"
- "What's the data flow?"
- "Which dependencies are critical?"
- "How do I make diagrams maintainable?"

**Parseltongue Endpoints**:
- `/semantic-cluster-grouping-list` - Module boundaries
- `/dependency-edges-list-all` - Relationship diagrams
- `/complexity-hotspots-ranking-view` - Focus on critical paths
- `/codebase-statistics-overview-summary` - Overview

**Design Voice**: Brad Frost's Atomic Design - systematic documentation

---

### Journey 8: Pull Request Reviewer's Change Understanding

**Persona**: Developer reviewing teammate's code change

**Trigger**: PR notification or scheduled review time

**Mental State**: Critical but collaborative, time-constrained, safety-focused

**Goals**:
- Understand what change does
- Assess if it breaks anything
- Check if it follows patterns
- Verify test coverage
- Provide constructive feedback quickly

**Success Criteria**:
- Impact of change fully understood
- Potential issues identified
- Architectural violations caught
- Constructive feedback provided
- Review completed quickly

**Pain Points**:
- Context switching (hard to understand complex changes)
- Missing impact (don't see what else affects)
- Pattern blindness (don't know if follows conventions)
- Test adequacy uncertainty
- Review bottleneck (slows team)

**Time Pressure**: Medium-High (team waiting)

**Frequency**: Daily (regular workflow)

**Questions They Ask**:
- "What does this change affect?"
- "Are there unintended consequences?"
- "Does this follow patterns?"
- "What tests are needed?"
- "Can I approve this safely?"

**Parseltongue Endpoints**:
- `/blast-radius-impact-analysis` - See impact of changed functions
- `/reverse-callers-query-graph` - Check what calls changed code
- `/temporal-coupling-hidden-deps` - Files that should also change
- `/circular-dependency-detection-scan` - Ensure no new cycles
- `/smart-context-token-budget` - Full context for AI-assisted review

**Design Voice**: Cap Watkins' design management principles

---

## Part 2: Eight Visualization Modes

### Mode 1: Orientation Mode (Circular CodeCity)

**User Journey**: Newcomer Onboarding

**Core Metaphor**: Circular CodeCity with District Grouping

```
         [Module District] ===arc=== [Module District]
              /                        \
        [Building]                  [Building]
               \                      /
                 =====arc=====
```

**Layout Strategy**:
- Primary: Module/district grouping on circle perimeter
- Secondary: Entity type colors within districts
- Tertiary: Building height = lines of code

**Key Features**:
- Hover shows entity tooltip
- Click selects building and highlights connections
- Module labels on ground
- "District Density" indicator for >50 entities

**Parseltongue Endpoints**:
- `/codebase-statistics-overview-summary`
- `/code-entities-list-all`
- `/semantic-cluster-grouping-list`
- `/dependency-edges-list-all`

**Time to Insight**: 30 seconds

---

### Mode 2: Archaeology Mode (Force-Directed Galaxy)

**User Journey**: Feature Understanding / Cross-Team Exploration

**Core Metaphor**: Force-Directed Galaxy with Gravity Wells

```
        [Trait Node - Gravity Well]
       /      |      \
  [Struct]  [Struct]  [Struct]
      |        |         |
  [Method] [Method]   [Method]
```

**Layout Strategy**:
- Primary: Physics-based attraction (high edge weight = closer)
- Secondary: Repulsion forces prevent overlap
- Tertiary: Pinned anchor nodes (traits, entry points)

**Key Features**:
- Click "Explore" to spawn galaxy from entity
- Drag nodes to adjust layout
- Double-click cluster to collapse
- "Gravity Well" mode for traits/interfaces

**Parseltongue Endpoints**:
- `/semantic-cluster-grouping-list`
- `/forward-callees-query-graph`
- `/reverse-callers-query-graph`
- `/complexity-hotspots-ranking-view`
- `/code-entity-detail-view`
- `/blast-radius-impact-analysis`

**Time to Insight**: 2 minutes

---

### Mode 3: Bug Hunt Mode (Reverse Tree Trace)

**User Journey**: Root Cause Investigation

**Core Metaphor**: Reverse Call Tree (Inverted Org Chart)

```
              [Error Location]
                     |
          ┌──────────┼──────────┐
         [Caller]   [Caller]   [Caller]
             |          |          |
         [Caller]   [Caller]   [Caller]
```

**Layout Strategy**:
- Primary: Inverted tree (error at bottom, callers above)
- Secondary: Horizontal grouping by call depth
- Tertiary: Color by execution likelihood

**Key Features**:
- Paste error trace to spawn tree
- Hover node to see "why this calls"
- Click node to expand subtree
- "Temporal Coupling" overlay shows hidden dependencies

**Parseltongue Endpoints**:
- `/reverse-callers-query-graph` (recursive)
- `/temporal-coupling-hidden-deps`
- `/complexity-hotspots-ranking-view`
- `/code-entity-detail-view`
- `/code-entities-search-fuzzy`

**Time to Insight**: 1 minute

---

### Mode 4: Refactor Mode (Blast Radius Viz)

**User Journey**: Safe Refactoring

**Core Metaphor**: Blast Radius Visualization (Ripple Effect)

```
         [Hop 0: Changed Entity]
        /    |    \
   [Hop 1] [Hop 1] [Hop 1]
    /  |      |      |  \
  [Hop 2] ...      ...   [Hop 2]
```

**Layout Strategy**:
- Primary: Radial (focus at center, dependents expand outward)
- Secondary: Concentric rings by hop distance
- Tertiary: Color by risk (green=hop1, yellow=hop2, red=hop3+)

**Key Features**:
- Select entity to see blast radius
- Slider to adjust hop depth (1-5)
- Risk score (0-100) based on depth and count
- "Test Coverage Gap" shows affected entities without tests

**Parseltongue Endpoints**:
- `/blast-radius-impact-analysis`
- `/reverse-callers-query-graph`
- `/complexity-hotspots-ranking-view`
- `/circular-dependency-detection-scan`
- `/temporal-coupling-hidden-deps`

**Time to Insight**: 30 seconds

---

### Mode 5: Impact Mode (Terrain Map)

**User Journey**: Pre-Deployment Risk Assessment

**Core Metaphor**: Hierarchical Terrain Map (2.5D Topology)

```
    [HTTP Layer] ←── elevation: high
         │
         ↓ (dependency river)
    [Service Layer]
         │
         ↓
    [Repository Layer]
         │
         ↓
    [Storage/DB]
```

**Layout Strategy**:
- Primary: Elevation = abstraction level (inferred from call patterns)
- Secondary: Horizontal grouping by semantic cluster
- Tertiary: Plateaus represent cohesive layers

**Key Features**:
- Toggle layer inference (manual override)
- Click "Violations Only" to filter
- Dependencies as rivers (blue=downhill, red=uphill/violations)
- Cycles rendered as "waterfalls"

**Parseltongue Endpoints**:
- `/code-entities-list-all`
- `/semantic-cluster-grouping-list`
- `/dependency-edges-list-all`
- `/circular-dependency-detection-scan`

**Time to Insight**: 2 minutes

---

### Mode 6: Audit Mode (Heatmap Grid)

**User Journey**: Technical Debt Assessment

**Core Metaphor**: Complexity Heatmap Grid

```
┌─────────────────────────────────────┐
│  [auth.rs]    [user.rs]    [db.rs]  │
│  ████████     ████░░░░     ██████   │
│  (hotspot)    (medium)     (high)   │
└─────────────────────────────────────┘
```

**Layout Strategy**:
- Primary: 2D grid (rows=modules, columns=metrics)
- Secondary: Sortable by any metric
- Tertiary: Color saturation = severity

**Key Features**:
- 5 metrics: Complexity, Coupling, Cohesion, Coverage, Churn
- Trend indicators (improving/worsening)
- "Debt Score" single number per module (0-100)
- Export as CSV/JSON

**Parseltongue Endpoints**:
- `/complexity-hotspots-ranking-view?top=100`
- `/code-entities-list-all`
- `/semantic-cluster-grouping-list`
- `/dependency-edges-list-all`
- `/circular-dependency-detection-scan`

**Time to Insight**: 3 minutes

---

### Mode 7: Review Mode (Split Diff View)

**User Journey**: Pull Code Review

**Core Metaphor**: Split-Diff Visualization

```
┌─────────────────┬─────────────────┐
│  BEFORE         │  AFTER          │
│  (main branch)  │  (PR branch)    │
│                 │                 │
│  [Entity]       │  [Entity MODIFIED]
│  [Entity]       │  [Entity DELETED]
│                 │  [Entity NEW]   │
└─────────────────┴─────────────────┘
```

**Layout Strategy**:
- Primary: Split view (left=before, right=after)
- Secondary: Aligned by entity key
- Tertiary: Color by change type

**Key Features**:
- Enter PR number or select branch
- Hover changed entity to highlight dependents
- "Orphan Detection" warns if removed code still called
- "Update Needed" indicators

**Parseltongue Endpoints**:
- `/code-entities-list-all`
- `/dependency-edges-list-all`
- `/reverse-callers-query-graph`
- `/blast-radius-impact-analysis`

**Time to Insight**: 1 minute

---

### Mode 8: Teach Mode (Story Path)

**User Journey**: Knowledge Transfer / Documentation

**Core Metaphor**: Guided Story Path (Interactive Tour)

```
┌─────────────────────────────────────────┐
│  STOP 1: Entry Point                    │
│  "This is where requests arrive"        │
│           │                             │
│           ▼                             │
│  STOP 2: Authentication                 │
│  "We verify identity here"              │
│           │                             │
│           ▼                             │
│  STOP 3: Business Logic                 │
│  "The real work happens here"           │
└─────────────────────────────────────────┘
```

**Layout Strategy**:
- Primary: Linear path through system
- Secondary: Side branches for alternative flows
- Tertiary: Numbered stops with descriptions

**Key Features**:
- "Next/Previous" navigation
- Auto-play mode with timing
- Narrative text panel with explanations
- "Export to Video" for sharing

**Parseltongue Endpoints**:
- `/code-entities-search-fuzzy`
- `/code-entity-detail-view`
- `/forward-callees-query-graph`
- `/reverse-callers-query-graph`

**Time to Insight**: Variable (story length)

---

## Part 3: Comparison Matrix

| Dimension | Orientation | Archaeology | Bug Hunt | Refactor | Impact | Audit | Review | Teach |
|-----------|-------------|-------------|----------|----------|--------|-------|--------|-------|
| **Primary Question** | What am I looking at? | How does this work? | Why is this broken? | What breaks if I change this? | Is this architecture sound? | Where is the tech debt? | What changed? | How do I explain this? |
| **Entry Metaphor** | Circular city | Galaxy cluster | Reverse tree | Ripple effect | Terrain map | Heatmap grid | Split diff | Story path |
| **Time Pressure** | Medium | Medium | Critical | Medium | Low | Low | Medium-High | Low |
| **Frequency** | One-time | Weekly | Daily | Weekly | Quarterly | Quarterly | Daily | Monthly |
| **Output** | Mental model | Call tree | Root cause | Risk score | Violations | Prioritized list | Approval | Understanding |
| **Social Context** | Solo + mentor | Cross-team | Usually solo | Code review | Leadership | Leadership | Collaborative | Teaching |
| **Time to Insight** | 30 sec | 2 min | 1 min | 30 sec | 2 min | 3 min | 1 min | Variable |

---

## Part 4: Mode Switching

### Mode Selector UI

```
┌─────────────────────────────────────────────────────────┐
│  Parseltongue                                    [Settings] │
├─────────────────────────────────────────────────────────┤
│  [🏙️ Orient] [🔍 Archaeology] [🐛 Bug Hunt] [🔨 Refactor] │
│  [📊 Impact] [📋 Audit] [👁️ Review] [📖 Teach]           │
├─────────────────────────────────────────────────────────┤
│  Search: [_______________]                              │
└─────────────────────────────────────────────────────────┘
```

### Transition Effects

| From | To | Transition | Context Preserved |
|------|-----|------------|-------------------|
| Any | Orientation | Zoom out to full circle | - |
| Orientation | Archaeology | Zoom into entity, explode to galaxy | Selected entity |
| Any | Bug Hunt | Fade to white, tree grows from bottom | Error location |
| Any | Refactor | Ripple animation from selected | Selected entity |
| Any | Impact | Rotate to terrain, elevation rises | - |
| Any | Audit | Flip to 2D grid, cells colorize | - |
| Any | Review | Split screen slide apart | - |
| Any | Teach | Fade to black, spotlight appears | - |

---

## Part 5: Implementation Roadmap

### Phase 1: Foundation (Current)
✅ Circular CodeCity visualization
✅ Basic entity type coloring
✅ Click-to-select details
✅ Parseltongue API integration

### Phase 2: Core Modes (Highest Value)

**Week 1-2: Refactor Mode**
- Extends current view with blast radius
- Risk score calculation
- Test coverage gap detection

**Week 3-4: Bug Hunt Mode**
- Reverse tree layout
- Stack trace parsing
- Temporal coupling overlay

**Week 5-6: Archaeology Mode**
- Force-directed physics using `d3-force-3d`
- Cluster collapse functionality
- Galaxy metaphor

### Phase 3: Advanced Modes

**Week 7-8: Impact Mode**
- Layer inference algorithm
- Terrain map rendering
- Violation detection

**Week 9-10: Audit Mode**
- 2D heatmap grid
- Metric calculations
- Export functionality

### Phase 4: Collaboration Features

**Week 11-12: Review Mode & Teach Mode**
- Split-diff visualization
- Story authoring tools
- Presentation recording

---

## Part 6: Design Principles Summary

| Principle | Source | Application |
|-----------|--------|-------------|
| **Make users awesome** | Kathy Sierra | Build competence through progressive disclosure |
| **Solve their problems** | Shreyas Doshi | Intent-driven, not noun-heavy interfaces |
| **Recognition over recall** | Jakob Nielsen | Visual patterns, consistent metaphors |
| **Systematic design** | Brad Frost | Atomic components, consistent patterns |
| **Shape work first** | Ryan Singer | Clear appetite, bounded scope |
| **Details matter** | Dan Saffer | Meaningful microinteractions |
| **Opportunity → Solution** | Teresa Torres | Clear outcomes for each mode |

---

## Part 7: Success Metrics by Mode

| Mode | Primary Metric | Secondary Metrics |
|------|----------------|-------------------|
| **Orientation** | Time to articulate architecture | Return visit rate |
| **Archaeology** | Can diagram call flow | Questions asked |
| **Bug Hunt** | Root cause identified | Time to hypothesis |
| **Refactor** | Risk score accuracy | Revert rate |
| **Impact** | Violations found | Deploy approval rate |
| **Audit** | Debt prioritized | Refactor completion rate |
| **Review** | Review completion time | Issues caught post-merge |
| **Teach** | Learner can explain | Knowledge retention |

---

## Sources

### Research Documents
- `influential-design-voices-2005-2025.md` - 50+ design voices
- `docs/web-ui/DESIGN_OPTIONS_FOR_PARSELTONGUE_VISUALIZATION.md` - Previous design work
- `docs/web-ui/INTERFACE_SIGNATURE_GRAPH_THESIS.md` - Visualization thesis
- `README.md` - Parseltongue capabilities and 15 HTTP endpoints

### Key Design Voices Referenced
- **Kathy Sierra** - User competence and skill-building
- **Shreyas Doshi** - Mental models and product philosophy
- **Julie Zhuo** - Designing at scale and problem-first thinking
- **Jakob Nielsen** - Usability heuristics and progressive disclosure
- **Teresa Torres** - Opportunity Solution Tree and continuous discovery
- **Brad Frost** - Atomic Design methodology
- **Ryan Singer** - Shape Up and appetite-based development
- **Dan Saffer** - Microinteractions and detail design
- **Don Norman** - Design thinking and human-centered design
- **Tim Brown** - Design thinking in business
- **David Kelley** - Creative confidence and IDEO methodology
- **Luke Wroblewski** - Mobile-first and form design
- **Bill Buxton** - Sketching and prototyping
- **Brenda Laurel** - Interaction design theory
- **Alan Cooper** - Goal-directed design and personas
- **Jared Spool** - Usability testing and research
- **Cap Watkins** - Design management and leadership
- **Gibson Biddle** - Product strategy at Netflix
- **Casey Winters** - Growth and marketplace frameworks
- **Lenny Rachitsky** - Product management education
- **Nathan Curtis** - Design systems strategy
- **Jina Bolton** - Enterprise design systems
- **Cennydd Bowles** - Ethics in design and AI
- **Erika Hall** - Just enough research

---

**Document Version**: 1.0
**Total User Journeys**: 8
**Total Visualization Modes**: 8
**Research Sources**: 50+ design voices + academic research
**Generated**: 2025-01-14
**Branch**: `research/visualization-improvements-20260110-1914`

---

*Each visualization mode is purpose-built for a specific developer journey. No single view serves all needs. The power of Parseltongue lies in offering the right visualization for the right question.*
