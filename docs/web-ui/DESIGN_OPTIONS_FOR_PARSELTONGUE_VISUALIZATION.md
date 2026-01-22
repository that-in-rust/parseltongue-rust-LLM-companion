# Design Options for Parseltongue Visualization

**Applying 20 Years of Design Thinking to Code Dependency Visualization**

**Date**: 2025-01-13
**Status**: Design Exploration
**Context**: Parseltongue 3D CodeCity - 239 entities, 211 dependency arcs

---

## Executive Summary

This document presents **four distinct visualization approaches** for Parseltongue, each inspired by different influential design voices from the past 20 years. Each option represents a unique philosophy for solving the core problem: **How do developers effectively understand and navigate code architecture?**

| Option | Design Voice Inspiration | Philosophy | Best For |
|--------|--------------------------|------------|----------|
| **1. Competence Journey** | Kathy Sierra + Teresa Torres | Build user skill progressively | Onboarding, learning |
| **2. Question Workspace** | Shreyas Doshi + Julie Zhuo | Solve specific user problems | Task completion |
| **3. CodeCity Explorer** | Jakob Nielsen + Dan Saffer | Usability through progressive disclosure | Exploration, navigation |
| **4. Atomic Code City** | Brad Frost + Nathan Curtis | Systematic component design | Consistency, scale |

---

## Current State Analysis

### What We Have
- **239 entities** displayed in circular layout
- **211 dependency arcs** as neon tubes
- **Dark theme** with neon colors by entity type
- **15 Parseltongue API endpoints** available
- **Click-to-select** with details panel

### The Core Problem
The current visualization shows **everything at once** - a noun-heavy interface presenting 239 entities with no guidance on where to start. This creates:

1. **Cognitive overload** for new users
2. **No progressive disclosure** for complex codebases
3. **One-size-fits-all** regardless of user intent
4. **No learning path** from novice to expert

### What Design Voices Tell Us

| Designer | Key Insight | Application |
|----------|-------------|-------------|
| **Kathy Sierra** | "Make users awesome, not just happy" | Focus on building competence |
| **Shreyas Doshi** | "Users care about their problems, not your product" | Intent-driven design |
| **Jakob Nielsen** | "Recognition rather than recall" | Progressive disclosure |
| **Brad Frost** | "Design systems are human relationships" | Atomic, systematic approach |

---

## Option 1: Competence Journey

> *"The best way to create passionate users is to help them become experts."* — Kathy Sierra

### Design Voice Inspiration
- **Kathy Sierra**: User competence and skill-building
- **Teresa Torres**: Opportunity Solution Tree framework

### Problem Statement
Developers opening a dependency visualization are rarely exploring for its own sake. They need to **become productive quickly**. Current visualization shows everything equally, creating cognitive overload rather than building competence.

### Mental Model Applied

**Kathy Sierra's Competence Curve**: Design for the path from unconscious incompetence → conscious competence → unconscious competence.

**Teresa Torres' Framework**:
- **Outcome**: User understands codebase architecture
- **Opportunity**: User needs progressive disclosure, not data dump
- **Solution**: Journey-based visualization that reveals complexity gradually

### Core Metaphor: "The Architect's Tour"

Instead of showing everything at once, follow a **guided journey**:

```
┌─────────────────────────────────────────────────────────────┐
│  Welcome Plaza (Entry Point)                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │ HTTP    │  │ Auth    │  │ Storage │  │ Parser  │       │
│  │ Handler │  │ Module  │  │ Layer   │  │         │       │
│  └────┬────┘  └────┬────┘  └────┬────┘  └─────────┘       │
│       │            │            │                          │
│       ▼            ▼            ▼                          │
│  [Explore Path] [Explore Path] [Explore Path]              │
└─────────────────────────────────────────────────────────────┘
```

**Progressive Unfolding**:

| Stage | What User Sees | User Action | Result |
|-------|----------------|-------------|--------|
| **Welcome Plaza** | 5-7 landmark entities | Click "Explore" | Path unfolds, related entities appear |
| **Skill Paths** | Direct dependencies (hop=1) | Click "Deeper" | Hop=2 entities revealed |
| **Mastery Zones** | Context-appropriate detail | Toggle view | Novice/Apprentice/Expert modes |

### Key Interactions

| Interaction | Current | Competence Journey |
|-------------|---------|-------------------|
| First view | All 239 entities | 5 landmarks with "Explore" buttons |
| Click entity | Shows details panel | Unlocks related entities, extends graph |
| Search | Basic text filter | Guided questions: "What handles HTTP?" |
| Navigation | Free 3D orbit | Follow paths, unlock regions, earn badges |

### Implementation Sketch

**Phase 1: Landmark Detection**
```typescript
// Use complexity hotspots to identify landmarks
const hotspots = await api.getComplexityHotspots();
const clusters = await api.getSemanticClusters();
const landmarks = clusterRepresentatives; // Top 5-7
```

**Phase 2: Progressive Path Unfolding**
```typescript
async function expandPathFromEntity(entityKey: string) {
  const deps = await api.fetch_both_entity_dependencies(entityKey);
  // Show hop=1 first, then offer "Deeper" for hop=2
  return rankByRelevance(deps);
}
```

**Phase 3: Competence Indicators**
- Progress bar: "You've explored 12% of authentication module"
- Badge system: "Dependency Detective", "Architecture Explorer"
- Save state to localStorage

### Success Metrics (Kathy Sierra Style)
- **Time to first competent action**: Can user make meaningful change within 10 minutes?
- **Exploration coverage**: What % explored before first commit?
- **Return usage**: Do users come back?
- **Confidence survey**: "How confident in your understanding?" (1-10, target: 7+)

---

## Option 2: Question Workspace

> *"Users don't care about your product. They care about their problems."* — Shreyas Doshi

### Design Voice Inspiration
- **Shreyas Doshi**: Mental models, product philosophy
- **Julie Zhuo**: Start with the problem
- **Ryan Singer**: Shape Up methodology

### Problem Statement
When developers open Parseltongue, they have specific problems:
- "I need to change authentication logic and I'm scared I'll break something"
- "Why is this function so slow?"
- "Where should I add this new feature?"

Current visualization is **noun-heavy** (entities, relationships, modules). Developers need a **verb-heavy interface** that helps them complete their job.

### Mental Model Applied

**Shreyas Doshi's Mental Models**:
1. Reduce cognitive friction
2. Design for the user's actual workflow
3. Product adapts to user, not vice versa

**Ryan Singer's Shape Up**: Design with appetite, not backlogs. Each "question" is a shaped project with clear boundaries.

### Core Metaphor: "The Investigation Workbench"

```
┌─────────────────────────────────────────────────────────────┐
│  What are you trying to do?                                  │
│  ● Refactor existing code                                    │
│  ● Add a new feature                                         │
│  ● Fix a bug                                                 │
│  ● Understand how something works                            │
└─────────────────────────────────────────────────────────────┘
```

Based on selection, workspace adapts:

**For "Refactor existing code"**:
```
┌─────────────────────────────────────────────────────────────┐
│  Which function/module are you modifying?                   │
│  [Search: Authentication handler...]                        │
│                                                              │
│  Impact Preview:                                             │
│  ├── Direct dependents: 12 functions                        │
│  ├── Transitive impact: 47 functions in 3 modules           │
│  ├── Risk level: HIGH (circular dependency detected)        │
│  └── Suggested approach: Read these 3 files first           │
│                                                              │
│  [Show Blast Radius] [Read Related Files] [Test Impact]     │
└─────────────────────────────────────────────────────────────┘
```

### Key Interactions

| Intent | Job To Be Done | Success Criteria |
|--------|----------------|------------------|
| Refactor | Know what breaks | List of affected tests/functions |
| Add feature | Find right place | Suggested module + file path |
| Fix bug | Understand why | Call chain from entry to error |
| Learn | Build mental model | Annotated reading list |

### Implementation Sketch

**Phase 1: Intent Router**
```typescript
type UserIntent = 'refactor' | 'add-feature' | 'fix-bug' | 'learn';

class IntentRouter {
  async onIntentSelected(intent: UserIntent) {
    switch(intent) {
      case 'refactor': return new RefactorWorkspace();
      case 'add-feature': return new FeatureWorkspace();
      case 'fix-bug': return new BugFixWorkspace();
      case 'learn': return new LearningWorkspace();
    }
  }
}
```

**Phase 2: Refactor Workspace (Highest Value)**
```typescript
class RefactorWorkspace {
  async onSelectEntity(entityKey: string) {
    const blast = await api.getBlastRadius(entityKey, 2);
    const cycles = await api.detectCircularDependencies();
    const affectedTests = await this.findAffectedTests(blast);

    this.displayImpactAnalysis({
      directImpact: blast.direct,
      transitiveImpact: blast.transitive,
      riskLevel: cycles.includes(entityKey) ? 'HIGH' : 'MEDIUM',
      affectedTests,
      recommendedReading: this.topologicalSort(blast),
    });
  }
}
```

### Success Metrics (Shreyas Doshi Style)
- **Time to answer**: Actionable answer within 30 seconds?
- **Decision confidence**: Does user proceed with changes? (Yes/No)
- **Correction rate**: How often are changes reverted? (Lower = better)
- **Task completion**: % of users who complete intended task

---

## Option 3: CodeCity Explorer

> *"Recognition rather than recall."* — Jakob Nielsen

### Design Voice Inspiration
- **Jakob Nielsen**: 10 Usability Heuristics
- **Dan Saffer**: Microinteractions
- **Luke Wroblewski**: Progressive enhancement

### Problem Statement
Current circular layout shows all entities equally with no hierarchy. Users struggle with:
- **Information overload**: 239 entities at once
- **No spatial organization**: Modules not grouped visually
- **Missing context**: Don't know where things are

### Mental Model Applied

**Jakob Nielsen's Heuristics**:
1. **Visibility of system status**: Always know where you are
2. **Recognition rather than recall**: Make patterns obvious
3. **User control and freedom**: Easy to zoom, filter, explore
4. **Consistency and standards**: Familiar interaction patterns

**Dan Saffer's Microinteractions**: Small moments that delight and guide.

### Core Metaphor: "Hierarchical Urban Planning"

Replace circular layout with **district-based urban planning**:

```
┌─────────────────────────────────────────────────────────────┐
│                   City View (Zoomed Out)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   HTTP      │  │   AUTH      │  │   STORAGE   │        │
│  │  District   │  │  District   │  │  District   │        │
│  │   ( teal )  │  │  (amber )   │  │ (emerald)   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────┘
```

**3-Tier Zoom System**:
- **City View** (zoomed out): Colored districts for major modules
- **District View** (medium): Buildings grouped by functionality
- **Building View** (zoomed in): Individual entities with connections

### Progressive Disclosure Approach

| Level | View | Entity Count | What's Shown |
|-------|------|--------------|--------------|
| **Level 1** | City View | 5-8 districts | Module groupings only |
| **Level 2** | District View | 20-50 buildings | Entities within selected district |
| **Level 3** | Building View | Full detail | All entities with dependencies |
| **Level 4** | Detail View | Single entity | Code snippets, full metadata |

### Key Interactions

**Smart Lens System**: Hover over district to see summary, click to "enter"

**Microinteractions**:
- **Building Pulse**: Gentle pulse on hover
- **District Highlight**: District glows when containing selected entity
- **Connection Sparkles**: Sparkle effects on active dependency lines
- **Smooth Transitions**: Easing functions for natural movement

### Visual Design

**Color System**:
- **District Base Colors**: Soft, muted (teal, amber, emerald, violet, orange)
- **Entity Accent Colors**: Bright, saturated for entity types
- **Connection Gradients**: Smooth gradients between related entities
- **Status Indicators**: Green (low coupling) → Red (high coupling)

**Building Shapes by Entity Type**:
- **Functions**: Tall towers
- **Modules**: Large domes
- **Structs**: Rectangular buildings
- **Traits**: Hexagonal structures
- **Enums**: Small cubes

### Implementation Sketch

**Phase 1: District Detection**
```typescript
const clusters = await api.getSemanticClusters();
const districts = clusters.map(cluster => ({
  name: cluster.name,
  entities: cluster.entities,
  color: assignDistrictColor(cluster.name),
  bounds: calculateDistrictBounds(cluster.entities)
}));
```

**Phase 2: LOD System**
```typescript
const lod = new THREE.LOD();
lod.addLevel(createCityView(), 300);   // Far
lod.addLevel(createDistrictView(), 100); // Medium
lod.addLevel(createBuildingView(), 0);   // Near
```

**Phase 3: Smart Lens**
```typescript
function onMouseOver(district) {
  showTooltip({
    title: district.name,
    stats: `${district.entities.length} entities`,
    highlights: district.hotspots
  });
}
```

### Success Metrics (Jakob Nielsen Style)
- **Task Success Rate**: % completed without assistance
- **Time to First Insight**: Time to identify meaningful patterns
- **Navigation Efficiency**: Interactions needed to find targets
- **Error Rate**: Selection errors, navigation confusion

---

## Option 4: Atomic Code City

> *"Design systems are about human relationships."* — Brad Frost

### Design Voice Inspiration
- **Brad Frost**: Atomic Design methodology
- **Nathan Curtis**: Design systems strategy
- **Bill Buxton**: Sketching and prototyping

### Problem Statement
Current visualization lacks **systematic structure**. Each entity is displayed similarly without clear component hierarchy or consistent atomic organization.

### Mental Model Applied

**Brad Frost's Atomic Design**:
1. **Atoms**: Basic building blocks (functions, variables)
2. **Molecules**: Groups of related entities (classes, modules)
3. **Organisms**: Complete functional units (components, services)
4. **Templates**: Higher-level abstractions (layers, domains)
5. **Pages**: Complete system overview

**Nathan Curtis's Principles**: Federated teams, systematic planning, atomic documentation.

### Core Metaphor: "Component-Based City"

Every element is part of a systematic hierarchy:

```
┌─────────────────────────────────────────────────────────────┐
│  Atomic Level Selector                                       │
│  [Atoms] [Molecules] [Organisms] [Templates] [Pages]        │
│                                                              │
│  Current: Molecules                                          │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐            │
│  │ AuthService│  │ UserService│  │ AuthModule │            │
│  │  (molecule)│  │  (molecule)│  │ (organism)  │            │
│  └────────────┘  └────────────┘  └────────────┘            │
└─────────────────────────────────────────────────────────────┘
```

### Atomic Disclosure Approach

| Atomic Level | What It Shows | Example |
|--------------|---------------|---------|
| **Atoms** | Individual entities | `parse_header()` function |
| **Molecules** | Related entities | `Auth` struct + its methods |
| **Organisms** | Functional units | Complete `AuthService` |
| **Templates** | Architectural layers | HTTP Layer, Service Layer |
| **Pages** | System overview | Entire application |

### Key Interactions

**Atomic Navigation**: Same gestures work at all levels (consistency principle)

**Component Relationship Maps**: Show how components interact across scales

**Microinteractions**:
- **Atomic Pulse**: Pulse effects scale with atomic importance
- **Component Snap**: Buildings snap to grid when aligned
- **Molecular Attraction**: Related entities gravitate toward each other
- **Atomic Bonds**: Visual bonds form between components

### Visual Design

**Atomic Grid System**: Everything aligned to consistent grid

**Color Tokens**:
```css
--atom-primary: #00fff5;
--molecule-primary: #39ff14;
--organism-primary: #bf00ff;
--template-primary: #ff6b00;
--page-primary: #ff00aa;
```

**Component Identity**: Each atomic level has consistent visual language

### Implementation Sketch

**Phase 1: Atomic Hierarchy**
```typescript
interface AtomicEntity {
  key: string;
  level: 'atom' | 'molecule' | 'organism' | 'template' | 'page';
  parent?: string;
  children: string[];
}

function buildAtomicHierarchy(entities: Entity[]): AtomicEntity[] {
  // Group entities by atomic structure
  return entities.map(e => classifyAtomicLevel(e));
}
```

**Phase 2: Component Library**
```typescript
const AtomicComponents = {
  Atom: (entity) => createSmallCube(entity),
  Molecule: (entities) => createGroupedShape(entities),
  Organism: (entities) => createBuilding(entities),
  Template: (entities) => createDistrict(entities),
  Page: (entities) => createFullCity(entities)
};
```

### Success Metrics (Brad Frost Style)
- **Component Consistency**: % of interactions following expected patterns
- **Learning Efficiency**: Time to understand atomic system
- **Pattern Recognition**: Ability to identify atomic relationships
- **Systematic Navigation**: Success rate of atomic-level transitions

---

## Comparison Matrix

| Dimension | Competence Journey | Question Workspace | CodeCity Explorer | Atomic Code City |
|-----------|-------------------|-------------------|-------------------|------------------|
| **Primary Philosophy** | Build user skill | Solve user problems | Usability first | Systematic design |
| **Entry Metaphor** | "Let me show you around" | "What are you trying to do?" | "Explore the city" | "Navigate the system" |
| **Learning Model** | Gradual discovery | Direct answer | Progressive disclosure | Consistent patterns |
| **User Progression** | Novice → Expert | Expert tool, easy Qs | City → District → Building | Atom → Page |
| **Best For** | Onboarding, learning | Specific tasks | Exploration, navigation | Consistency, scale |
| **Time Horizon** | Weeks (repeated use) | Minutes (quick answers) | Hours (exploration) | Days (system mastery) |
| **3D Visualization** | Core experience | Optional view | Core experience | Core experience |
| **Progressive Disclosure** | Path-based | Intent-based | Zoom-based | Atomic-based |
| **Primary Metric** | Competence growth | Task completion | Navigation efficiency | Pattern recognition |

---

## Implementation Recommendation

### Phased Approach

**Phase 1: Quick Wins (1 week)**
1. Add Jakob Nielsen-style progressive disclosure (3-tier zoom)
2. Implement Dan Saffer's microinteractions (hover effects)
3. Create district grouping using semantic clusters

**Phase 2: MVP (2-3 weeks)**
4. Build Question Workspace with Refactor intent
5. Add blast radius visualization
6. Implement smart search with intent routing

**Phase 3: Enhanced Features (4-6 weeks)**
7. Add Competence Journey mode for onboarding
8. Implement atomic design system
9. Add progressive path unfolding

### Recommended Starting Point

**Begin with Option 2 (Question Workspace)** because:

1. **Shreyas Doshi's principle**: "Ship one intent first, get feedback"
2. **Ryan Singer's Shape Up**: Each intent is a shaped project with clear appetite
3. **Faster value**: Users get actionable answers immediately
4. **Easier MVP**: One workspace vs. entire system redesign

**Then add Option 3 (CodeCity Explorer)** as the visualization layer.

### Hybrid Architecture

```
[Parseltongue Visualization]
├── Mode: Question Workspace (default)
│   ├── Intent: Refactor ○
│   ├── Intent: Add Feature ○
│   ├── Intent: Fix Bug ○
│   └── Intent: Learn ○
└── Mode: Free Exploration
    ├── View: City Explorer
    └── View: Competence Journey
```

---

## Design Principles Summary

| Principle | Source | Application |
|-----------|--------|-------------|
| **Make users awesome** | Kathy Sierra | Build competence, not just show data |
| **Solve user problems** | Shreyas Doshi | Intent-driven, not noun-heavy |
| **Recognition over recall** | Jakob Nielsen | Progressive disclosure |
| **Systematic design** | Brad Frost | Atomic, component-based |
| **Shape work first** | Ryan Singer | Clear appetite per feature |
| **Details matter** | Dan Saffer | Meaningful microinteractions |
| **Mobile first** | Luke Wroblewski | Progressive enhancement |
| **Design for people** | Nathan Curtis | Human relationships in systems |

---

## Sources

- Influential Design Voices Research: `/influential-design-voices-2005-2025.md`
- Interface Signature Graph Thesis: `/docs/web-ui/INTERFACE_SIGNATURE_GRAPH_THESIS.md`
- Visualization Improvements: `/docs/web-ui/VISUALIZATION_IMPROVEMENTS.md`

---

**Document Version**: 1.0
**Generated**: 2025-01-13
**Agent**: Claude Opus 4.5 + Plan Agent + Explore Agent
**Total Options**: 4 distinct visualization approaches
