# Web UI TDD Test Plan

**Document**: Test-Driven Development plan for Parseltongue 3D CodeCity Web UI

**Created**: 2025-01-11 09:35 America/Los_Angeles

---

## Overview

This document outlines the TDD approach for building `web-ui/`, following Parseltongue's established STUB → RED → GREEN → REFACTOR cycle.

**Key Principle**: Start from tests and work backwards. Every feature begins with a failing test.

---

## Part 1: Test Coverage Targets

| Test Type | Target | Tool | Runtime |
|-----------|--------|------|---------|
| Unit Tests | >80% coverage | Vitest | <50ms each |
| Integration Tests | All user flows | Vitest + MSW | <500ms each |
| E2E Tests | Critical paths | Playwright | <5s each |
| Visual Regression | Component-level | Playwright | CI only |

---

## Part 2: Test Pyramid (Bottom-Up)

```
                    ┌─────────────────────┐
                    │   E2E Visual Tests  │ ← Slow, expensive
                    │   (Playwright)      │
                    └─────────────────────┘
                  ┌─────────────────────────┐
                  │   Integration Tests     │ ← Medium speed
                  │   (API + Components)    │
                  └─────────────────────────┘
               ┌──────────────────────────────┐
               │      Unit Tests              │ ← Fast, many
               │      (Vitest)                │
               └──────────────────────────────┘
```

**Strategy**: Start at the bottom (unit tests), move up only when foundation is solid.

---

## Part 3: Test Suite Structure

### Suite 1: Data Transformation Tests

**Location**: `src/data/__tests__/`

| Test File | Test Name | Description | Priority |
|-----------|-----------|-------------|----------|
| `entity_parser.test.ts` | `test_parses_api_response_to_entity_list` | API JSON → internal types | P0 |
| `entity_parser.test.ts` | `test_validates_required_entity_fields` | Missing fields rejected | P0 |
| `entity_parser.test.ts` | `test_handles_malformed_response_gracefully` | Error handling | P1 |
| `entity_filters.test.ts` | `test_filters_entities_by_language` | Language filter logic | P0 |
| `entity_filters.test.ts` | `test_filters_entities_by_type` | Entity type filter | P0 |
| `entity_filters.test.ts` | `test_filters_entities_by_package` | Package filter | P1 |
| `entity_filters.test.ts` | `test_combines_multiple_filters` | Filter composition | P1 |
| `complexity_sorter.test.ts` | `test_sorts_entities_by_complexity_descending` | Complexity sorting | P1 |
| `complexity_sorter.test.ts` | `test_handles_ties_stably` | Stable sort | P2 |
| `color_mapper.test.ts` | `test_maps_entity_to_color` | Color assignment | P0 |
| `color_mapper.test.ts` | `test_color_mapper_handles_unknown_types` | Fallback color | P1 |

### Suite 2: State Management Tests

**Location**: `src/state/__tests__/`

| Test File | Test Name | Description | Priority |
|-----------|-----------|-------------|----------|
| `store.test.ts` | `test_initializes_empty_state` | Store initialization | P0 |
| `store.test.ts` | `test_dispatches_load_entities_action` | Action dispatch | P0 |
| `store.test.ts` | `test_replays_actions_from_history` | Time travel debug | P2 |
| `selectors.test.ts` | `test_selects_filtered_entities` | Selector logic | P0 |
| `selectors.test.ts` | `test_selects_visible_buildings` | Visible subset | P0 |
| `selectors.test.ts` | `test_selects_selected_entity_details` | Current selection | P0 |
| `actions.test.ts` | `test_select_entity_action_updates_state` | Selection state | P0 |
| `actions.test.ts` | `test_set_camera_target_action` | Camera state | P1 |

### Suite 3: 3D Layout Tests

**Location**: `src/scene/__tests__/`

| Test File | Test Name | Description | Priority |
|-----------|-----------|-------------|----------|
| `layout_engine.test.ts` | `test_calculates_building_position_from_coordinates` | Position algorithm | P0 |
| `layout_engine.test.ts` | `test_calculates_building_dimensions_from_metrics` | Size mapping | P0 |
| `layout_engine.test.ts` | `test_arranges_buildings_by_cluster` | Cluster layout | P1 |
| `layout_engine.test.ts` | `test_prevents_building_overlap` | Collision detection | P1 |
| `layout_engine.test.ts` | `test_layout_handles_single_entity` | Edge case | P1 |
| `layout_engine.test.ts` | `test_layout_handles_empty_dataset` | Edge case | P1 |
| `building_builder.test.ts` | `test_creates_instanced_mesh_with_correct_count` | Mesh creation | P0 |
| `building_builder.test.ts` | `test_sets_building_height_from_loc` | Height mapping | P0 |
| `building_builder.test.ts` | `test_disposes_old_mesh_on_rebuild` | Memory cleanup | P0 |
| `camera_controller.test.ts` | `test_initializes_with_default_position` | Camera setup | P0 |
| `camera_controller.test.ts` | `test_animates_to_target_position` | Animation | P1 |

### Suite 4: API Integration Tests

**Location**: `src/api/__tests__/`

| Test File | Test Name | Description | Priority |
|-----------|-----------|-------------|----------|
| `api_client.test.ts` | `test_fetch_health_check` | Health endpoint | P0 |
| `api_client.test.ts` | `test_fetch_all_entities` | List entities | P0 |
| `api_client.test.ts` | `test_fetch_entity_details` | Single entity | P0 |
| `api_client.test.ts` | `test_fetch_blast_radius` | Impact analysis | P0 |
| `api_client.test.ts` | `test_fetch_semantic_clusters` | Clustering | P1 |
| `api_client.test.ts` | `test_handles_network_errors` | Error handling | P0 |
| `api_client.test.ts` | `test_handles_timeout` | Timeout handling | P1 |
| `api_client.test.ts` | `test_retries_on_failure` | Retry logic | P1 |

### Suite 5: E2E Tests

**Location**: `e2e/`

| Test File | Test Name | Description | Priority |
|-----------|-----------|-------------|----------|
| `app.spec.ts` | `test_loads_initial_view` | Application loads | P0 |
| `app.spec.ts` | `test_displays_building_count` | Stats display | P0 |
| `interaction.spec.ts` | `test_clicking_entity_shows_details` | Entity selection | P0 |
| `interaction.spec.ts` | `test_hovering_highlights_building` | Hover effect | P1 |
| `blast_radius.spec.ts` | `test_blast_radius_displays_on_selection` | Impact visualization | P0 |
| `filters.spec.ts` | `test_language_filter_updates_view` | Filter interaction | P0 |
| `filters.spec.ts` | `test_type_filter_updates_view` | Type filter | P1 |
| `comparison.spec.ts` | `test_snapshot_comparison_loads` | Diff view | P1 |
| `comparison.spec.ts` | `test_diff_highlights_added_buildings` | Diff visualization | P1 |

---

## Part 4: TDD Cycle Examples

### Example 1: Entity Color Mapper

```typescript
// STUB: Write failing test
describe('assignBuildingColor', () => {
  test('assigns_blue_to_rust_structs', () => {
    const entity = {
      language: 'rust',
      entity_type: 'struct'
    };
    expect(assignBuildingColor(entity)).toBe('#3b82f6');
  });

  test('assigns_green_to_rust_functions', () => {
    const entity = {
      language: 'rust',
      entity_type: 'fn'
    };
    expect(assignBuildingColor(entity)).toBe('#10b981');
  });

  test('assigns_gray_to_unknown_types', () => {
    const entity = {
      language: 'unknown',
      entity_type: 'unknown'
    };
    expect(assignBuildingColor(entity)).toBe('#6b7280');
  });
});

// RED: Run test, verify failure
// Output: assignBuildingColor is not defined

// GREEN: Minimal implementation
const assignBuildingColor = (entity: Entity): string => {
  const colorMap: Record<string, string> = {
    'rust:struct': '#3b82f6',
    'rust:fn': '#10b981',
  };
  const key = `${entity.language}:${entity.entity_type}`;
  return colorMap[key] ?? '#6b7280';
};

// REFACTOR: Improve structure, add docs
```

### Example 2: Layout Calculation

```typescript
// STUB: Write failing test
describe('calculateBuildingPosition', () => {
  test('places_first_building_at_origin', () => {
    const entities = [{ name: 'A', package: 'pkg1' }];
    const positions = calculateBuildingPositions(entities);
    expect(positions[0]).toEqual({ x: 0, y: 0, z: 0 });
  });

  test('places_buildings_in_grid_pattern', () => {
    const entities = [
      { name: 'A', package: 'pkg1' },
      { name: 'B', package: 'pkg1' },
      { name: 'C', package: 'pkg1' }
    ];
    const positions = calculateBuildingPositions(entities);
    expect(positions[0]).toEqual({ x: 0, y: 0, z: 0 });
    expect(positions[1]).toEqual({ x: 10, y: 0, z: 0 });
    expect(positions[2]).toEqual({ x: 20, y: 0, z: 0 });
  });
});

// RED: Run test, verify failure

// GREEN: Minimal implementation
function calculateBuildingPositions(entities: Entity[]): Position[] {
  const spacing = 10;
  return entities.map((_, i) => ({
    x: i * spacing,
    y: 0,
    z: 0
  }));
}

// REFACTOR: Add package grouping, variable spacing, etc.
```

---

## Part 5: Test Execution Commands

```bash
# Unit tests (watch mode)
npm run test:watch

# Unit tests (single run)
npm test

# Coverage report
npm run test:coverage

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Visual regression
npm run test:visual

# All tests
npm run test:all
```

---

## Part 6: Mocks and Fixtures

### MSW Handlers

**Location**: `test/mocks/handlers.ts`

```typescript
import { rest } from 'msw';

export const handlers = [
  // Health check
  rest.get('http://localhost:7777/server-health-check-status',
    (req, res, ctx) => {
      return res(
        ctx.status(200),
        ctx.json({
          success: true,
          endpoint: '/server-health-check-status',
          status: 'ok'
        })
      );
    }
  ),

  // All entities
  rest.get('http://localhost:7777/code-entities-list-all',
    (req, res, ctx) => {
      return res(
        ctx.status(200),
        ctx.json({
          success: true,
          endpoint: '/code-entities-list-all',
          data: mockEntityList
        })
      );
    }
  ),

  // Entity details
  rest.get('http://localhost:7777/code-entity-detail-view',
    (req, res, ctx) => {
      const key = req.url.searchParams.get('key');
      return res(
        ctx.status(200),
        ctx.json({
          success: true,
          endpoint: '/code-entity-detail-view',
          data: mockEntities[key] || mockEntityDetail
        })
      );
    }
  ),

  // Blast radius
  rest.get('http://localhost:7777/blast-radius-impact-analysis',
    (req, res, ctx) => {
      return res(
        ctx.status(200),
        ctx.json({
          success: true,
          endpoint: '/blast-radius-impact-analysis',
          data: mockBlastRadius
        })
      );
    }
  ),
];
```

### Test Fixtures

**Location**: `test/fixtures/entities.ts`

```typescript
export const mockEntityList = {
  entities: [
    {
      key: 'rust:struct:Main:src_main_rs:1-50',
      language: 'rust',
      entity_type: 'struct',
      name: 'Main',
      file_path: 'src/main.rs',
      line_range: { start: 1, end: 50 },
      lines_of_code: 50,
      complexity: 5
    },
    // ... more entities
  ]
};

export const mockBlastRadius = {
  focus_entity: 'rust:fn:process',
  impact_set: [
    'rust:fn:helper1',
    'rust:fn:helper2'
  ],
  hop_count: 2
};
```

---

## Part 7: Implementation Order (Test-First)

### Phase 1: Foundation (Days 1-2)

1. **Setup**
   - Configure Vite + TypeScript + Vitest
   - Create directory structure
   - Write first test: `test_project_config_loads`

2. **Types**
   - Define `Entity`, `Edge`, `ApiError` types
   - Write type validation tests

3. **API Client Stub**
   - Create `fetchWithAuth` wrapper
   - Write MSW handlers
   - Test: `test_fetch_wraps_errors_appropriately`

### Phase 2: Data Layer (Days 3-5)

1. **Parser Tests**
   - `test_parses_api_response_to_entity_list`
   - `test_validates_required_entity_fields`
   - Implement parser

2. **Filter Tests**
   - `test_filters_entities_by_language`
   - `test_filters_entities_by_type`
   - Implement filters

3. **Color Mapper Tests**
   - `test_maps_entity_to_color`
   - Implement color mapper

### Phase 3: State Management (Days 6-7)

1. **Store Tests**
   - `test_initializes_empty_state`
   - `test_dispatches_load_entities_action`
   - Implement Zustand store

2. **Selector Tests**
   - `test_selects_filtered_entities`
   - `test_selects_selected_entity_details`
   - Implement selectors

### Phase 4: 3D Scene (Days 8-12)

1. **Layout Tests**
   - `test_calculates_building_position_from_coordinates`
   - `test_prevents_building_overlap`
   - Implement layout engine

2. **Building Renderer Tests**
   - `test_creates_instanced_mesh_with_correct_count`
   - `test_sets_building_height_from_loc`
   - Implement building renderer

3. **Camera Tests**
   - `test_initializes_with_default_position`
   - `test_animates_to_target_position`
   - Implement camera controller

### Phase 5: Integration (Days 13-15)

1. **API Integration Tests**
   - All endpoint tests with MSW

2. **Component Integration**
   - React + Three.js working together

3. **E2E Tests**
   - Playwright tests for critical paths

---

## Part 8: Success Criteria

A test is considered complete when:

1. **STUB**: Test file exists, test is written
2. **RED**: Test fails descriptively
3. **GREEN**: Minimal implementation passes
4. **REFACTOR**: Code is clean, documented

**No commits without all tests passing.**

---

## Sources

- Parseltongue TDD Principles: `S06-design101-tdd-architecture-principles.md`
- Vitest Documentation: https://vitest.dev/
- Playwright Documentation: https://playwright.dev/
- MSW Documentation: https://mswjs.io/
