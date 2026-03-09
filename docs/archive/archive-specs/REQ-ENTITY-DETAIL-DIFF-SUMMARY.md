# Entity Detail Panel and Diff Summary Stats Specification

## Phase 2.4 - Additional React Components

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete
**Phase**: 2.4 (Entity Detail + Summary Stats)
**Dependency**: REQ-REACT-FRONTEND-VISUALIZATION (Phase 2.3-2.4 Core)

---

## Overview

### Problem Statement

Users viewing the 3D dependency graph need:

1. **Entity Details** - When clicking a node, users need to see detailed information about that entity (function, class, module) including its name, type, file location, and relationships to other entities.

2. **Change Summary** - Users need an at-a-glance summary of the current diff showing counts of added, removed, modified, and affected entities to understand the scope of code changes.

Currently, the DiffGraphCanvasView renders the 3D graph and handles node selection, but the selected node information is stored in the diffVisualizationStore without a corresponding UI panel to display it. Similarly, DiffSummaryData is tracked but not rendered.

### Solution

Two new React components:

| Component | Purpose | Data Source |
|-----------|---------|-------------|
| **EntityDetailPanel** | Slide-out panel showing selected node details | `useSelectedNode()` from diffVisualizationStore |
| **DiffSummaryStats** | Badge bar showing change counts | `useDiffSummary()` from diffVisualizationStore |

### Integration Points

```typescript
// From diffVisualizationStore.ts
useSelectedNode(): GraphNode | null;
useDiffSummary(): DiffSummaryData | null;
useDiffVisualizationActions().clearSelectedNode(): void;

// From types/api.ts
interface GraphNode {
  id: string;
  name: string;
  nodeType: string;
  changeType: 'added' | 'removed' | 'modified' | 'affected' | null;
  filePath?: string;
  lineStart?: number;
  lineEnd?: number;
}

interface DiffSummaryData {
  total_before_count: number;
  total_after_count: number;
  added_entity_count: number;
  removed_entity_count: number;
  modified_entity_count: number;
  unchanged_entity_count: number;
  relocated_entity_count: number;
}
```

---

# Section 1: Entity Detail Panel Requirements

## REQ-DETAIL-001: EntityDetailPanel Component

### Problem Statement

When a user clicks on a node in the 3D graph, they need to see detailed information about that entity in a panel. This panel must show the entity's identity, location, and relationships while matching the graph's color coding for the entity's change type.

### Specification

#### REQ-DETAIL-001.1: Panel Visibility on Node Selection

```
WHEN selectedNode in diffVisualizationStore is non-null
THEN EntityDetailPanel SHALL render as visible
  AND SHALL slide in from the right edge of the screen
  AND SHALL occupy 320px width on desktop (min-width: 768px)
  AND SHALL occupy 100% width on mobile (max-width: 767px)
  AND animation duration SHALL be 200ms with ease-out timing
  AND SHALL have z-index higher than graph canvas (z-50)
```

#### REQ-DETAIL-001.2: Panel Hidden When No Selection

```
WHEN selectedNode in diffVisualizationStore is null
THEN EntityDetailPanel SHALL NOT be visible
  AND SHALL slide out to the right
  AND SHALL NOT occupy layout space when hidden
  AND animation duration SHALL be 150ms with ease-in timing
```

#### REQ-DETAIL-001.3: Display Node Identity Information

```
WHEN EntityDetailPanel renders with selectedNode
THEN SHALL display entity identity section containing:
  - Primary: node.name in text-lg font weight semibold
  - Secondary: node.nodeType in parentheses with text-sm text-gray-400
  - Badge: node.changeType with color matching CHANGE_TYPE_COLORS:
    - 'added': bg-green-500/20 text-green-400 border-green-500/50
    - 'removed': bg-red-500/20 text-red-400 border-red-500/50
    - 'modified': bg-amber-500/20 text-amber-400 border-amber-500/50
    - 'affected': bg-blue-500/20 text-blue-400 border-blue-500/50
    - null: bg-gray-500/20 text-gray-400 border-gray-500/50 (label: "Unchanged")
  AND badge text SHALL be capitalized (e.g., "Added", "Modified")
```

#### REQ-DETAIL-001.4: Display File Location

```
WHEN EntityDetailPanel renders with selectedNode
  WITH node.filePath defined
THEN SHALL display file location section containing:
  - Label: "File" in text-xs text-gray-500 uppercase
  - Value: node.filePath in text-sm font-mono truncated with title tooltip
  AND IF node.lineStart is defined
  THEN SHALL append ":${lineStart}" to file path display
  AND IF node.lineEnd is defined AND node.lineEnd !== node.lineStart
  THEN SHALL append "-${lineEnd}" after lineStart (e.g., "src/auth.ts:10-25")

WHEN EntityDetailPanel renders with selectedNode
  WITH node.filePath undefined
THEN SHALL display "Location unknown" in text-gray-500 italic
```

#### REQ-DETAIL-001.5: Display Incoming Dependencies

```
WHEN EntityDetailPanel renders with selectedNode
THEN SHALL compute incoming dependencies from graphData.links:
  - Filter links WHERE link.target === selectedNode.id
  - Resolve source node from graphData.nodes for each link
  - Group by edgeType
THEN SHALL display "Incoming Dependencies" section:
  - Header: "Incoming" with count badge showing total count
  - IF count === 0: Display "No incoming dependencies" in text-gray-500
  - IF count > 0: Display grouped list:
    - Group header: edgeType (e.g., "Calls", "Imports")
    - List items: source node names, max 5 visible initially
    - IF group has > 5 items: Show "+N more" expandable link
  AND each dependency item SHALL be clickable to select that node
  AND incoming dependencies SHALL have left border color matching source changeType
```

#### REQ-DETAIL-001.6: Display Outgoing Dependencies

```
WHEN EntityDetailPanel renders with selectedNode
THEN SHALL compute outgoing dependencies from graphData.links:
  - Filter links WHERE link.source === selectedNode.id
  - Resolve target node from graphData.nodes for each link
  - Group by edgeType
THEN SHALL display "Outgoing Dependencies" section:
  - Header: "Outgoing" with count badge showing total count
  - IF count === 0: Display "No outgoing dependencies" in text-gray-500
  - IF count > 0: Display grouped list:
    - Group header: edgeType (e.g., "Calls", "Imports")
    - List items: target node names, max 5 visible initially
    - IF group has > 5 items: Show "+N more" expandable link
  AND each dependency item SHALL be clickable to select that node
  AND outgoing dependencies SHALL have right border color matching target changeType
```

#### REQ-DETAIL-001.7: Close Panel with Escape Key

```
WHEN EntityDetailPanel is visible
  AND user presses Escape key
THEN SHALL call clearSelectedNode() from diffVisualizationStore
  AND panel SHALL animate closed
  AND focus SHALL return to graph canvas
```

#### REQ-DETAIL-001.8: Close Button

```
WHEN EntityDetailPanel is visible
THEN SHALL display close button (X icon) in top-right corner
  WITH aria-label="Close entity details"
  AND button SHALL have hover state (bg-gray-700)
  AND clicking SHALL call clearSelectedNode()
```

#### REQ-DETAIL-001.9: Responsive Layout

```
WHEN viewport width < 768px (mobile)
THEN EntityDetailPanel SHALL:
  - Render as bottom sheet instead of side panel
  - Occupy 100% width
  - Max height: 60vh
  - Include drag handle for swipe-to-close gesture
  - Swipe down > 50% height SHALL close panel

WHEN viewport width >= 768px (desktop)
THEN EntityDetailPanel SHALL:
  - Render as right-side panel
  - Fixed width: 320px
  - Full viewport height minus header
```

### Error Conditions

```
WHEN selectedNode references a node not in graphData.nodes
THEN SHALL display error state:
  - Message: "Entity not found in current graph"
  - "Close" button to clear selection
  AND SHALL log warning to console

WHEN computing dependencies encounters malformed link data
THEN SHALL skip malformed entries
  AND SHALL NOT crash component
  AND SHALL log warning with link details to console
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Panel open animation | < 200ms | CSS transition duration |
| Dependency computation | < 50ms for 1000 links | Performance.measure() |
| Re-render on selection | < 16ms (60fps) | React DevTools Profiler |
| Memory for dependency list | < 10KB | Computed from node count |

### Verification Test Template

```typescript
// __tests__/components/EntityDetailPanel.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EntityDetailPanel } from '@/components/EntityDetailPanel';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';

describe('REQ-DETAIL-001: EntityDetailPanel Component', () => {
  const mockNode = {
    id: 'rust:fn:handle_auth',
    name: 'handle_auth',
    nodeType: 'function',
    changeType: 'added' as const,
    filePath: 'src/auth.rs',
    lineStart: 10,
    lineEnd: 45,
  };

  const mockGraphData = {
    nodes: [
      mockNode,
      { id: 'rust:fn:validate', name: 'validate', nodeType: 'function', changeType: null },
      { id: 'rust:fn:login', name: 'login', nodeType: 'function', changeType: 'modified' as const },
    ],
    links: [
      { source: 'rust:fn:login', target: 'rust:fn:handle_auth', edgeType: 'Calls' },
      { source: 'rust:fn:handle_auth', target: 'rust:fn:validate', edgeType: 'Calls' },
    ],
  };

  beforeEach(() => {
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: null,
    });
  });

  // REQ-DETAIL-001.1
  test('renders panel when node is selected', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('entity-detail-panel')).toBeVisible();
    expect(screen.getByTestId('entity-detail-panel')).toHaveClass('translate-x-0');
  });

  // REQ-DETAIL-001.2
  test('hides panel when no node selected', () => {
    useDiffVisualizationStore.setState({ selectedNode: null });

    render(<EntityDetailPanel />);

    expect(screen.queryByTestId('entity-detail-panel')).not.toBeVisible();
  });

  // REQ-DETAIL-001.3
  test('displays node identity with correct change type styling', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('entity-name')).toHaveTextContent('handle_auth');
    expect(screen.getByTestId('entity-type')).toHaveTextContent('(function)');
    expect(screen.getByTestId('change-type-badge')).toHaveTextContent('Added');
    expect(screen.getByTestId('change-type-badge')).toHaveClass('bg-green-500/20');
  });

  // REQ-DETAIL-001.4
  test('displays file location with line numbers', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('file-location')).toHaveTextContent('src/auth.rs:10-45');
  });

  // REQ-DETAIL-001.4 (no file path)
  test('displays "Location unknown" when filePath is undefined', () => {
    const nodeWithoutPath = { ...mockNode, filePath: undefined, lineStart: undefined };
    useDiffVisualizationStore.setState({ selectedNode: nodeWithoutPath });

    render(<EntityDetailPanel />);

    expect(screen.getByText('Location unknown')).toBeInTheDocument();
  });

  // REQ-DETAIL-001.5
  test('displays incoming dependencies grouped by edge type', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('incoming-deps-header')).toHaveTextContent('Incoming');
    expect(screen.getByTestId('incoming-deps-count')).toHaveTextContent('1');
    expect(screen.getByText('login')).toBeInTheDocument();
  });

  // REQ-DETAIL-001.6
  test('displays outgoing dependencies grouped by edge type', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('outgoing-deps-header')).toHaveTextContent('Outgoing');
    expect(screen.getByTestId('outgoing-deps-count')).toHaveTextContent('1');
    expect(screen.getByText('validate')).toBeInTheDocument();
  });

  // REQ-DETAIL-001.5/006 (clicking dependency)
  test('clicking dependency item selects that node', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    await userEvent.click(screen.getByText('validate'));

    expect(useDiffVisualizationStore.getState().selectedNode?.id).toBe('rust:fn:validate');
  });

  // REQ-DETAIL-001.7
  test('Escape key closes panel', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    fireEvent.keyDown(document, { key: 'Escape' });

    await waitFor(() => {
      expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
    });
  });

  // REQ-DETAIL-001.8
  test('close button clears selection', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    await userEvent.click(screen.getByRole('button', { name: /close/i }));

    expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
  });

  // REQ-DETAIL-001.5 (empty state)
  test('displays "No incoming dependencies" when none exist', () => {
    const isolatedNode = { ...mockNode, id: 'isolated' };
    const graphWithIsolated = {
      nodes: [...mockGraphData.nodes, isolatedNode],
      links: mockGraphData.links,
    };
    useDiffVisualizationStore.setState({
      graphData: graphWithIsolated,
      selectedNode: isolatedNode,
    });

    render(<EntityDetailPanel />);

    expect(screen.getByText('No incoming dependencies')).toBeInTheDocument();
  });
});
```

### Acceptance Criteria

- [ ] Panel slides in from right when node selected
- [ ] Panel slides out when selection cleared
- [ ] Node name and type displayed correctly
- [ ] Change type badge matches CHANGE_TYPE_COLORS
- [ ] File path with line numbers displayed
- [ ] "Location unknown" shown when no file path
- [ ] Incoming dependencies listed and grouped
- [ ] Outgoing dependencies listed and grouped
- [ ] Clicking dependency selects that node
- [ ] Escape key closes panel
- [ ] Close button closes panel
- [ ] Mobile responsive as bottom sheet
- [ ] Accessible with screen readers

---

# Section 2: Diff Summary Stats Requirements

## REQ-SUMMARY-001: DiffSummaryStats Component

### Problem Statement

Users need to quickly understand the scope and nature of changes in the current diff without examining individual nodes. A compact summary bar showing counts by change type provides this at-a-glance overview.

### Specification

#### REQ-SUMMARY-001.1: Display Change Counts

```
WHEN DiffSummaryStats receives summary from useDiffSummary()
  WITH summary.added_entity_count >= 0
  AND summary.removed_entity_count >= 0
  AND summary.modified_entity_count >= 0
THEN SHALL display four badges in a horizontal row:
  - Added badge: "+{count}" with bg-green-500/20 text-green-400
  - Removed badge: "-{count}" with bg-red-500/20 text-red-400
  - Modified badge: "~{count}" with bg-amber-500/20 text-amber-400
  - Affected badge: lightning icon + "{blast_radius_count}" with bg-blue-500/20 text-blue-400
  AND each badge SHALL have consistent padding (px-2 py-1)
  AND badges SHALL have rounded corners (rounded-md)
  AND badges SHALL have subtle border matching their color family
```

#### REQ-SUMMARY-001.2: Format Large Numbers

```
WHEN any count exceeds 999
THEN SHALL format with thousands separator:
  - 1000 -> "1,000"
  - 12345 -> "12,345"
  AND SHALL use locale-aware formatting (Intl.NumberFormat)
  AND SHALL NOT truncate to "1K" format
```

#### REQ-SUMMARY-001.3: Empty/Zero State

```
WHEN summary is null
  OR (added_entity_count === 0 AND removed_entity_count === 0 AND modified_entity_count === 0)
THEN SHALL display:
  - Text: "No changes detected"
  - Style: text-gray-500 bg-gray-800 rounded-lg px-3 py-2
  AND SHALL NOT display individual count badges
```

#### REQ-SUMMARY-001.4: Auto-Update on WebSocket Events

```
WHEN diffVisualizationStore.summary updates
  (via updateSummaryData action from diff_completed WebSocket event)
THEN DiffSummaryStats SHALL re-render with new counts
  AND re-render SHALL complete within 100ms
  AND SHALL animate count changes with CountUp animation (300ms)
```

#### REQ-SUMMARY-001.5: Collapsible Detail View

```
WHEN user clicks on DiffSummaryStats bar
THEN SHALL expand to show additional details:
  - Total entities before: {summary.total_before_count}
  - Total entities after: {summary.total_after_count}
  - Unchanged entities: {summary.unchanged_entity_count}
  - Relocated entities: {summary.relocated_entity_count}
  AND expanded view SHALL have smooth height transition (200ms)
  AND chevron icon SHALL rotate 180 degrees when expanded

WHEN user clicks again on expanded DiffSummaryStats
THEN SHALL collapse back to summary bar
```

#### REQ-SUMMARY-001.6: Diff In Progress Indicator

```
WHEN isDiffInProgress in diffVisualizationStore is true
THEN SHALL display loading state:
  - Pulsing animation on container (animate-pulse)
  - Text: "Analyzing changes..." in text-gray-400
  - Spinner icon next to text
  AND SHALL replace count badges with loading skeleton

WHEN isDiffInProgress changes from true to false
THEN SHALL transition from loading to populated state
  WITH fade animation (150ms)
```

#### REQ-SUMMARY-001.7: Badge Tooltips

```
WHEN user hovers over a count badge
THEN SHALL display tooltip after 300ms delay:
  - Added badge: "{count} entities added"
  - Removed badge: "{count} entities removed"
  - Modified badge: "{count} entities modified"
  - Affected badge: "{count} entities in blast radius"
  AND tooltip SHALL be positioned above badge
  AND tooltip SHALL have arrow pointing to badge
```

#### REQ-SUMMARY-001.8: Responsive Layout

```
WHEN viewport width < 640px (mobile)
THEN badges SHALL stack in 2x2 grid
  AND text size SHALL be text-xs
  AND padding SHALL be reduced (px-1.5 py-0.5)

WHEN viewport width >= 640px (desktop)
THEN badges SHALL display in horizontal row
  AND text size SHALL be text-sm
  AND padding SHALL be standard (px-2 py-1)
```

### Error Conditions

```
WHEN summary contains negative count values
THEN SHALL treat as 0
  AND SHALL log warning to console: "Invalid negative count in diff summary"

WHEN summary contains NaN or non-numeric values
THEN SHALL display "--" instead of count
  AND SHALL log warning with field name
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Initial render | < 16ms | React DevTools Profiler |
| Count animation | 300ms total | CSS transition measurement |
| Re-render on update | < 16ms (60fps) | requestAnimationFrame delta |
| Expand/collapse animation | 200ms | CSS transition duration |

### Verification Test Template

```typescript
// __tests__/components/DiffSummaryStats.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DiffSummaryStats } from '@/components/DiffSummaryStats';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';
import type { DiffSummaryData } from '@/types/api';

describe('REQ-SUMMARY-001: DiffSummaryStats Component', () => {
  const mockSummary: DiffSummaryData = {
    total_before_count: 100,
    total_after_count: 115,
    added_entity_count: 20,
    removed_entity_count: 5,
    modified_entity_count: 12,
    unchanged_entity_count: 78,
    relocated_entity_count: 3,
  };

  beforeEach(() => {
    useDiffVisualizationStore.setState({
      summary: null,
      isDiffInProgress: false,
    });
  });

  // REQ-SUMMARY-001.1
  test('displays change counts with correct styling', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    expect(screen.getByTestId('added-count')).toHaveTextContent('+20');
    expect(screen.getByTestId('added-count')).toHaveClass('bg-green-500/20');
    expect(screen.getByTestId('added-count')).toHaveClass('text-green-400');

    expect(screen.getByTestId('removed-count')).toHaveTextContent('-5');
    expect(screen.getByTestId('removed-count')).toHaveClass('bg-red-500/20');
    expect(screen.getByTestId('removed-count')).toHaveClass('text-red-400');

    expect(screen.getByTestId('modified-count')).toHaveTextContent('~12');
    expect(screen.getByTestId('modified-count')).toHaveClass('bg-amber-500/20');
    expect(screen.getByTestId('modified-count')).toHaveClass('text-amber-400');

    expect(screen.getByTestId('affected-count')).toHaveTextContent('45');
    expect(screen.getByTestId('affected-count')).toHaveClass('bg-blue-500/20');
  });

  // REQ-SUMMARY-001.2
  test('formats large numbers with thousands separator', () => {
    const largeSummary: DiffSummaryData = {
      ...mockSummary,
      added_entity_count: 12345,
      removed_entity_count: 1000,
    };
    useDiffVisualizationStore.setState({ summary: largeSummary });

    render(<DiffSummaryStats blastRadiusCount={9999} />);

    expect(screen.getByTestId('added-count')).toHaveTextContent('+12,345');
    expect(screen.getByTestId('removed-count')).toHaveTextContent('-1,000');
    expect(screen.getByTestId('affected-count')).toHaveTextContent('9,999');
  });

  // REQ-SUMMARY-001.3 (null summary)
  test('displays empty state when summary is null', () => {
    useDiffVisualizationStore.setState({ summary: null });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();
    expect(screen.queryByTestId('added-count')).not.toBeInTheDocument();
  });

  // REQ-SUMMARY-001.3 (all zeros)
  test('displays empty state when all counts are zero', () => {
    const zeroSummary: DiffSummaryData = {
      ...mockSummary,
      added_entity_count: 0,
      removed_entity_count: 0,
      modified_entity_count: 0,
    };
    useDiffVisualizationStore.setState({ summary: zeroSummary });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();
  });

  // REQ-SUMMARY-001.4
  test('re-renders when summary updates', async () => {
    const { rerender } = render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();

    useDiffVisualizationStore.setState({ summary: mockSummary });
    rerender(<DiffSummaryStats blastRadiusCount={45} />);

    await waitFor(() => {
      expect(screen.getByTestId('added-count')).toHaveTextContent('+20');
    });
  });

  // REQ-SUMMARY-001.5
  test('expands to show details on click', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    // Initially collapsed
    expect(screen.queryByTestId('detail-view')).not.toBeVisible();

    // Click to expand
    await userEvent.click(screen.getByTestId('summary-bar'));

    expect(screen.getByTestId('detail-view')).toBeVisible();
    expect(screen.getByText('Total before: 100')).toBeInTheDocument();
    expect(screen.getByText('Total after: 115')).toBeInTheDocument();
    expect(screen.getByText('Unchanged: 78')).toBeInTheDocument();
    expect(screen.getByText('Relocated: 3')).toBeInTheDocument();
  });

  // REQ-SUMMARY-001.5 (collapse)
  test('collapses detail view on second click', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    // Expand
    await userEvent.click(screen.getByTestId('summary-bar'));
    expect(screen.getByTestId('detail-view')).toBeVisible();

    // Collapse
    await userEvent.click(screen.getByTestId('summary-bar'));
    await waitFor(() => {
      expect(screen.queryByTestId('detail-view')).not.toBeVisible();
    });
  });

  // REQ-SUMMARY-001.6
  test('shows loading state when diff in progress', () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('Analyzing changes...')).toBeInTheDocument();
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.getByTestId('summary-container')).toHaveClass('animate-pulse');
  });

  // REQ-SUMMARY-001.7
  test('shows tooltip on badge hover', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('added-count'));

    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toHaveTextContent('20 entities added');
      },
      { timeout: 500 }
    );
  });

  // Error handling
  test('handles negative counts gracefully', () => {
    const invalidSummary: DiffSummaryData = {
      ...mockSummary,
      added_entity_count: -5,
    };
    useDiffVisualizationStore.setState({ summary: invalidSummary });
    const consoleSpy = jest.spyOn(console, 'warn').mockImplementation();

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByTestId('added-count')).toHaveTextContent('+0');
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('Invalid negative count')
    );

    consoleSpy.mockRestore();
  });
});
```

### Acceptance Criteria

- [ ] Four badges displayed with correct colors
- [ ] Numbers formatted with thousands separators
- [ ] Empty state shown when no changes
- [ ] Auto-updates when summary changes via WebSocket
- [ ] Collapsible detail view works
- [ ] Diff in progress shows loading state
- [ ] Tooltips appear on hover
- [ ] Mobile responsive 2x2 grid layout
- [ ] Negative counts handled gracefully

---

# Section 3: Integration Requirements

## REQ-INTEGRATION-001: Layout Composition

### Problem Statement

EntityDetailPanel and DiffSummaryStats must integrate seamlessly with the existing application layout containing WorkspaceListSidebar and DiffGraphCanvasView.

### Specification

#### REQ-INTEGRATION-001.1: Main Layout Structure

```
WHEN application renders main layout
THEN layout SHALL follow this structure:
  <div class="h-screen flex">
    <!-- Left sidebar -->
    <WorkspaceListSidebar class="w-64 flex-shrink-0" />

    <!-- Main content area -->
    <div class="flex-1 relative">
      <!-- Top bar with summary -->
      <DiffSummaryStats class="absolute top-4 left-4 right-4 z-10" />

      <!-- 3D Graph -->
      <DiffGraphCanvasView class="w-full h-full" />

      <!-- Right panel overlay -->
      <EntityDetailPanel class="absolute right-0 top-0 bottom-0 z-20" />

      <!-- Connection status -->
      <ConnectionStatusIndicator class="absolute bottom-4 left-4 z-10" />
    </div>
  </div>
```

#### REQ-INTEGRATION-001.2: Z-Index Layering

```
WHEN rendering overlapping components
THEN z-index SHALL follow this hierarchy:
  - z-0: DiffGraphCanvasView (3D canvas)
  - z-10: DiffSummaryStats, ConnectionStatusIndicator, ColorLegend
  - z-20: EntityDetailPanel
  - z-30: Modals/Dialogs
  - z-40: Toast notifications
```

#### REQ-INTEGRATION-001.3: Event Coordination

```
WHEN DiffGraphCanvasView.onNodeClick fires
THEN diffVisualizationStore.selectNodeById SHALL be called
  AND EntityDetailPanel SHALL receive update via useSelectedNode()

WHEN EntityDetailPanel close button clicked
THEN diffVisualizationStore.clearSelectedNode SHALL be called
  AND DiffGraphCanvasView SHALL deselect node visual styling

WHEN WebSocket diff_completed event received
THEN diffVisualizationStore.updateSummaryData SHALL be called
  AND DiffSummaryStats SHALL re-render with new counts
```

### Verification Test Template

```typescript
// __tests__/integration/MainLayout.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MainLayout } from '@/components/MainLayout';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';

describe('REQ-INTEGRATION-001: Layout Composition', () => {
  // REQ-INTEGRATION-001.1
  test('renders all components in correct layout', () => {
    render(<MainLayout />);

    expect(screen.getByTestId('workspace-list-sidebar')).toBeInTheDocument();
    expect(screen.getByTestId('diff-summary-stats')).toBeInTheDocument();
    expect(screen.getByTestId('diff-graph-canvas')).toBeInTheDocument();
    expect(screen.getByTestId('connection-status-indicator')).toBeInTheDocument();
  });

  // REQ-INTEGRATION-001.3
  test('node click updates EntityDetailPanel', async () => {
    render(<MainLayout />);

    // Simulate node click through store
    useDiffVisualizationStore.getState().actions.selectNodeById('test-node-id');

    await waitFor(() => {
      expect(screen.getByTestId('entity-detail-panel')).toBeVisible();
    });
  });

  // REQ-INTEGRATION-001.3
  test('closing panel clears graph selection', async () => {
    // Setup selected node
    useDiffVisualizationStore.setState({
      selectedNode: { id: 'test', name: 'Test', nodeType: 'function', changeType: null },
    });

    render(<MainLayout />);

    // Close panel
    fireEvent.click(screen.getByRole('button', { name: /close/i }));

    await waitFor(() => {
      expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
    });
  });
});
```

---

# Section 4: Accessibility Requirements

## REQ-A11Y-001: Keyboard and Screen Reader Support

### Problem Statement

Both components must be fully accessible to users with disabilities, supporting keyboard navigation and screen readers.

### Specification

#### REQ-A11Y-001.1: EntityDetailPanel Accessibility

```
WHEN EntityDetailPanel renders
THEN SHALL include:
  - role="complementary" on panel container
  - aria-label="Entity details panel"
  - aria-labelledby referencing node name heading
  - Close button with aria-label="Close entity details"
  - Focus trap when panel is open
  - Focus returns to trigger element on close

WHEN navigating dependencies list
THEN SHALL support:
  - Tab navigation between dependency items
  - Enter/Space to select dependency
  - aria-current="true" on selected dependency
  - Announce dependency count to screen readers
```

#### REQ-A11Y-001.2: DiffSummaryStats Accessibility

```
WHEN DiffSummaryStats renders
THEN SHALL include:
  - role="status" on container for live region announcements
  - aria-live="polite" for count updates
  - aria-label on each badge describing the count
  - Expand/collapse button with aria-expanded state
  - Collapsible region with aria-hidden when collapsed

WHEN diff completes (counts update)
THEN screen reader SHALL announce summary:
  - "Diff complete: {added} added, {removed} removed, {modified} modified"
```

### Verification Test Template

```typescript
// __tests__/a11y/EntityDetailPanel.a11y.test.tsx
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';
import { EntityDetailPanel } from '@/components/EntityDetailPanel';

expect.extend(toHaveNoViolations);

describe('REQ-A11Y-001: EntityDetailPanel Accessibility', () => {
  test('has no accessibility violations', async () => {
    const { container } = render(<EntityDetailPanel />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  test('panel has correct ARIA attributes', () => {
    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveAttribute('role', 'complementary');
    expect(panel).toHaveAttribute('aria-label', 'Entity details panel');
  });
});

// __tests__/a11y/DiffSummaryStats.a11y.test.tsx
describe('REQ-A11Y-001: DiffSummaryStats Accessibility', () => {
  test('has no accessibility violations', async () => {
    const { container } = render(<DiffSummaryStats blastRadiusCount={10} />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });

  test('container has live region attributes', () => {
    render(<DiffSummaryStats blastRadiusCount={10} />);

    const container = screen.getByTestId('summary-container');
    expect(container).toHaveAttribute('role', 'status');
    expect(container).toHaveAttribute('aria-live', 'polite');
  });
});
```

---

# Summary

## Requirements Count

| Section | Requirement IDs | Test Count |
|---------|-----------------|------------|
| 1. EntityDetailPanel | REQ-DETAIL-001.1 to 001.9 | 12 tests |
| 2. DiffSummaryStats | REQ-SUMMARY-001.1 to 001.8 | 11 tests |
| 3. Integration | REQ-INTEGRATION-001.1 to 001.3 | 3 tests |
| 4. Accessibility | REQ-A11Y-001.1 to 001.2 | 4 tests |
| **Total** | **22 Sub-requirements** | **30 Tests** |

## Acceptance Criteria Checklist

### EntityDetailPanel
- [ ] REQ-DETAIL-001.1: Panel visible when node selected
- [ ] REQ-DETAIL-001.2: Panel hidden when no selection
- [ ] REQ-DETAIL-001.3: Node identity displayed with styling
- [ ] REQ-DETAIL-001.4: File location displayed
- [ ] REQ-DETAIL-001.5: Incoming dependencies listed
- [ ] REQ-DETAIL-001.6: Outgoing dependencies listed
- [ ] REQ-DETAIL-001.7: Escape key closes panel
- [ ] REQ-DETAIL-001.8: Close button works
- [ ] REQ-DETAIL-001.9: Mobile responsive layout

### DiffSummaryStats
- [ ] REQ-SUMMARY-001.1: Change counts displayed
- [ ] REQ-SUMMARY-001.2: Large numbers formatted
- [ ] REQ-SUMMARY-001.3: Empty state shown
- [ ] REQ-SUMMARY-001.4: Auto-updates on events
- [ ] REQ-SUMMARY-001.5: Collapsible detail view
- [ ] REQ-SUMMARY-001.6: Diff in progress indicator
- [ ] REQ-SUMMARY-001.7: Badge tooltips
- [ ] REQ-SUMMARY-001.8: Responsive layout

### Integration
- [ ] REQ-INTEGRATION-001.1: Correct layout structure
- [ ] REQ-INTEGRATION-001.2: Z-index layering correct
- [ ] REQ-INTEGRATION-001.3: Events coordinate correctly

### Accessibility
- [ ] REQ-A11Y-001.1: EntityDetailPanel accessible
- [ ] REQ-A11Y-001.2: DiffSummaryStats accessible

## Performance Targets

| Component | Metric | Target |
|-----------|--------|--------|
| EntityDetailPanel | Open animation | < 200ms |
| EntityDetailPanel | Dependency computation | < 50ms for 1000 links |
| EntityDetailPanel | Re-render | < 16ms |
| DiffSummaryStats | Initial render | < 16ms |
| DiffSummaryStats | Count animation | 300ms |
| DiffSummaryStats | Expand animation | 200ms |

## File Structure for Implementation

```
frontend/src/
  components/
    EntityDetailPanel.tsx          <- NEW
    EntityDetailPanel.test.tsx     <- NEW
    DiffSummaryStats.tsx           <- NEW
    DiffSummaryStats.test.tsx      <- NEW
  utils/
    computeDependencies.ts         <- NEW (helper for REQ-DETAIL-001.5/6)
    formatNumber.ts                <- NEW (helper for REQ-SUMMARY-001.2)
```

---

*Specification created: 2026-01-23*
*Parent specification: REQ-REACT-FRONTEND-VISUALIZATION*
*Target: 30 testable requirements for EntityDetailPanel and DiffSummaryStats*
