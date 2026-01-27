# REQ-APP-INTEGRATION: App.tsx Integration Specification

## Document Metadata

| Field | Value |
|-------|-------|
| Requirement ID | REQ-APP-INTEGRATION |
| Title | App.tsx Root Component Integration |
| Version | 1.0.0 |
| Status | Draft |
| Created | 2026-01-23 |
| Author | Executable Specs Agent |

---

## 1. Problem Statement

### 1.1 What Pain Exists

The Parseltongue frontend has individual components (WorkspaceListSidebar, DiffGraphCanvasView, ConnectionStatusIndicator, EntityDetailPanel, DiffSummaryStats), stores (workspaceStore, diffVisualizationStore), and a WebSocket hook (useWebsocketDiffStream) that are implemented and tested in isolation. However, there is no cohesive integration that:

1. Composes these components into a functional layout
2. Wires data flow between stores and components
3. Orchestrates WebSocket connection lifecycle based on user actions
4. Handles responsive layout for mobile/desktop
5. Coordinates user interactions across the application

### 1.2 Who Feels This Pain

- **End users** who cannot visualize dependency diffs because the application does not render a functional UI
- **Developers** who need a clear contract for how components interact
- **QA engineers** who need testable integration requirements

### 1.3 What Would Success Look Like

A fully functional single-page application where:
- Users can select a workspace and immediately see real-time diff updates
- The 3D graph visualizes dependency changes with proper color coding
- Clicking nodes reveals detailed entity information
- Connection status is always visible
- The layout adapts gracefully to different screen sizes

---

## 2. Component Inventory

### 2.1 Components to Integrate

| Component | Location | Purpose |
|-----------|----------|---------|
| `WorkspaceListSidebar` | `src/components/WorkspaceListSidebar.tsx` | Left sidebar for workspace selection and management |
| `DiffGraphCanvasView` | `src/components/DiffGraphCanvasView.tsx` | Main 3D force graph visualization canvas |
| `ConnectionStatusIndicator` | `src/components/ConnectionStatusIndicator.tsx` | WebSocket connection state display |
| `EntityDetailPanel` | `src/components/EntityDetailPanel.tsx` | Right slide-out panel for selected node details |
| `DiffSummaryStats` | `src/components/DiffSummaryStats.tsx` | Top bar showing change counts |

### 2.2 Stores to Wire

| Store | Location | State Managed |
|-------|----------|---------------|
| `workspaceStore` | `src/stores/workspaceStore.ts` | Workspace list, selection, loading, errors |
| `diffVisualizationStore` | `src/stores/diffVisualizationStore.ts` | Graph data, selected node, diff summary |

### 2.3 Hooks to Integrate

| Hook | Location | Purpose |
|------|----------|---------|
| `useWebsocketDiffStream` | `src/hooks/useWebsocketDiffStream.ts` | WebSocket connection lifecycle and event handling |

---

## 3. REQ-APP-001: Layout Composition

### 3.1 Problem Statement

The application needs a responsive three-column layout (sidebar, canvas, detail panel) with a header bar that degrades gracefully on mobile devices.

### 3.2 Specification

#### REQ-APP-001.1: Desktop Layout Structure

```
WHEN App component renders
  WITH viewport width >= 768px (md breakpoint)
THEN SHALL render a grid layout with:
  - Left column: WorkspaceListSidebar (fixed width 256px / w-64)
  - Center column: Main content area (flex-grow)
  - Right column: EntityDetailPanel (conditional, fixed width 320px / w-80)
AND SHALL render header bar at top of center column containing:
  - DiffSummaryStats (left-aligned)
  - ConnectionStatusIndicator (right-aligned)
AND SHALL fill entire viewport height (h-screen)
AND SHALL use dark theme background (bg-gray-900)
```

#### REQ-APP-001.2: Mobile Layout Structure

```
WHEN App component renders
  WITH viewport width < 768px (below md breakpoint)
THEN SHALL hide WorkspaceListSidebar by default
AND SHALL render sidebar toggle button in header
AND SHALL render EntityDetailPanel as bottom sheet (max-height 60vh)
AND SHALL render DiffSummaryStats in compact 2x2 grid layout
```

#### REQ-APP-001.3: Sidebar Toggle Behavior

```
WHEN user clicks sidebar toggle button on mobile
  WITH sidebar currently hidden
THEN SHALL display WorkspaceListSidebar as overlay
AND SHALL add semi-transparent backdrop (bg-black/50)
AND SHALL trap focus within sidebar until dismissed

WHEN user clicks backdrop OR presses Escape
  WITH sidebar currently visible on mobile
THEN SHALL hide WorkspaceListSidebar
AND SHALL remove backdrop
AND SHALL restore focus to toggle button
```

#### REQ-APP-001.4: Layout Test IDs

```
WHEN App component renders
THEN SHALL assign data-testid attributes:
  - "app-container" to root container
  - "app-header" to header bar
  - "app-sidebar" to sidebar container
  - "app-main-canvas" to main canvas area
  - "sidebar-toggle-button" to mobile toggle button
```

### 3.3 Test Template

```typescript
// File: src/__tests__/App.layout.test.tsx

import { render, screen, fireEvent } from '@testing-library/react';
import { App } from '../App';

describe('REQ-APP-001: Layout Composition', () => {
  describe('REQ-APP-001.1: Desktop Layout Structure', () => {
    beforeEach(() => {
      // Set viewport to desktop width
      Object.defineProperty(window, 'innerWidth', { value: 1024, writable: true });
      window.dispatchEvent(new Event('resize'));
    });

    test('GIVEN desktop viewport WHEN App renders THEN sidebar is visible', () => {
      // GIVEN
      render(<App />);

      // WHEN (rendered)

      // THEN
      expect(screen.getByTestId('workspace-list-sidebar')).toBeVisible();
    });

    test('GIVEN desktop viewport WHEN App renders THEN header contains DiffSummaryStats and ConnectionStatusIndicator', () => {
      // GIVEN
      render(<App />);

      // WHEN (rendered)

      // THEN
      const header = screen.getByTestId('app-header');
      expect(header).toContainElement(screen.getByTestId('diff-summary-stats'));
      expect(header).toContainElement(screen.getByTestId('connection-status-indicator'));
    });

    test('GIVEN desktop viewport WHEN App renders THEN uses full viewport height', () => {
      // GIVEN
      render(<App />);

      // WHEN (rendered)

      // THEN
      const container = screen.getByTestId('app-container');
      expect(container).toHaveClass('h-screen');
    });
  });

  describe('REQ-APP-001.2: Mobile Layout Structure', () => {
    beforeEach(() => {
      // Set viewport to mobile width
      Object.defineProperty(window, 'innerWidth', { value: 375, writable: true });
      window.dispatchEvent(new Event('resize'));
    });

    test('GIVEN mobile viewport WHEN App renders THEN sidebar is hidden by default', () => {
      // GIVEN
      render(<App />);

      // WHEN (rendered)

      // THEN
      expect(screen.getByTestId('workspace-list-sidebar')).not.toBeVisible();
    });

    test('GIVEN mobile viewport WHEN App renders THEN toggle button is visible', () => {
      // GIVEN
      render(<App />);

      // WHEN (rendered)

      // THEN
      expect(screen.getByTestId('sidebar-toggle-button')).toBeVisible();
    });
  });

  describe('REQ-APP-001.3: Sidebar Toggle Behavior', () => {
    beforeEach(() => {
      Object.defineProperty(window, 'innerWidth', { value: 375, writable: true });
      window.dispatchEvent(new Event('resize'));
    });

    test('GIVEN mobile viewport with sidebar hidden WHEN toggle clicked THEN sidebar becomes visible', () => {
      // GIVEN
      render(<App />);
      const toggleButton = screen.getByTestId('sidebar-toggle-button');

      // WHEN
      fireEvent.click(toggleButton);

      // THEN
      expect(screen.getByTestId('workspace-list-sidebar')).toBeVisible();
    });

    test('GIVEN mobile viewport with sidebar visible WHEN Escape pressed THEN sidebar hides', () => {
      // GIVEN
      render(<App />);
      fireEvent.click(screen.getByTestId('sidebar-toggle-button'));
      expect(screen.getByTestId('workspace-list-sidebar')).toBeVisible();

      // WHEN
      fireEvent.keyDown(document, { key: 'Escape' });

      // THEN
      expect(screen.getByTestId('workspace-list-sidebar')).not.toBeVisible();
    });
  });
});
```

### 3.4 Acceptance Criteria

- [ ] Desktop layout displays sidebar, canvas, and header correctly
- [ ] Mobile layout hides sidebar by default
- [ ] Sidebar toggle button appears only on mobile
- [ ] Toggle button opens/closes sidebar
- [ ] Escape key closes mobile sidebar
- [ ] All test IDs are present and correct
- [ ] Dark theme applied consistently (bg-gray-900)

---

## 4. REQ-APP-002: Component Wiring

### 4.1 Problem Statement

Components need to receive correct props from stores and have their callbacks wired to store actions to enable data flow throughout the application.

### 4.2 Specification

#### REQ-APP-002.1: WorkspaceListSidebar Wiring

```
WHEN WorkspaceListSidebar is rendered
THEN SHALL receive workspace list from useWorkspaceList() selector
AND SHALL receive selected workspace ID from useSelectedWorkspaceId() selector
AND SHALL receive loading state from useWorkspaceLoading() selector
AND SHALL receive error state from useWorkspaceError() selector
AND SHALL have onSelect callback wired to selectWorkspaceById action
AND SHALL have onToggleWatch callback wired to toggleWorkspaceWatchState action
AND SHALL call fetchWorkspaceListData on mount
```

#### REQ-APP-002.2: DiffGraphCanvasView Wiring

```
WHEN DiffGraphCanvasView is rendered
THEN SHALL receive graphData prop from useGraphData() selector
AND SHALL have onNodeClick callback that:
  - Calls selectNodeById action with clicked node ID
AND SHALL have onBackgroundClick callback that:
  - Calls clearSelectedNode action
```

#### REQ-APP-002.3: EntityDetailPanel Wiring

```
WHEN EntityDetailPanel is rendered
  WITH selectedNode in diffVisualizationStore being non-null
THEN SHALL be visible (visibility: visible, translate-x-0)
AND SHALL display selected node information

WHEN EntityDetailPanel is rendered
  WITH selectedNode in diffVisualizationStore being null
THEN SHALL be hidden (visibility: hidden, translate-x-full)
```

#### REQ-APP-002.4: DiffSummaryStats Wiring

```
WHEN DiffSummaryStats is rendered
THEN SHALL receive summary from useDiffSummary() selector
AND SHALL receive isDiffInProgress from useIsDiffInProgress() selector
AND SHALL compute blastRadiusCount from graph data:
  - Count nodes where changeType === 'affected'
```

#### REQ-APP-002.5: ConnectionStatusIndicator Wiring

```
WHEN ConnectionStatusIndicator is rendered
THEN SHALL receive connectionStatus from useWebsocketDiffStream hook
AND SHALL receive reconnectAttempt from useWebsocketDiffStream hook
AND SHALL receive maxReconnectAttempts from useWebsocketDiffStream hook
```

### 4.3 Test Template

```typescript
// File: src/__tests__/App.wiring.test.tsx

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { App } from '../App';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useDiffVisualizationStore } from '../stores/diffVisualizationStore';

// Mock the WebSocket hook
jest.mock('../hooks/useWebsocketDiffStream', () => ({
  useWebsocketDiffStream: () => ({
    connectionStatus: 'connected',
    lastDiffEvent: null,
    reconnectAttempt: 0,
    maxReconnectAttempts: 5,
    subscribe: jest.fn(),
    unsubscribe: jest.fn(),
  }),
}));

describe('REQ-APP-002: Component Wiring', () => {
  beforeEach(() => {
    // Reset stores
    useWorkspaceStore.setState({
      workspaces: [],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
    useDiffVisualizationStore.setState({
      graphData: { nodes: [], links: [] },
      selectedNode: null,
      summary: null,
      isDiffInProgress: false,
    });
  });

  describe('REQ-APP-002.2: DiffGraphCanvasView Wiring', () => {
    test('GIVEN graph data in store WHEN node clicked THEN selectNodeById called', async () => {
      // GIVEN
      const testNode = {
        id: 'test-node-1',
        name: 'testFunction',
        nodeType: 'function',
        changeType: 'added' as const,
      };
      useDiffVisualizationStore.setState({
        graphData: { nodes: [testNode], links: [] },
      });
      render(<App />);

      // WHEN - simulate node click (via store action directly in integration test)
      useDiffVisualizationStore.getState().actions.selectNodeById('test-node-1');

      // THEN
      await waitFor(() => {
        expect(useDiffVisualizationStore.getState().selectedNode).toEqual(testNode);
      });
    });

    test('GIVEN selected node WHEN background clicked THEN selectedNode cleared', async () => {
      // GIVEN
      const testNode = {
        id: 'test-node-1',
        name: 'testFunction',
        nodeType: 'function',
        changeType: 'added' as const,
      };
      useDiffVisualizationStore.setState({
        graphData: { nodes: [testNode], links: [] },
        selectedNode: testNode,
      });
      render(<App />);

      // WHEN
      useDiffVisualizationStore.getState().actions.clearSelectedNode();

      // THEN
      await waitFor(() => {
        expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
      });
    });
  });

  describe('REQ-APP-002.3: EntityDetailPanel Wiring', () => {
    test('GIVEN selectedNode is non-null WHEN rendered THEN panel is visible', async () => {
      // GIVEN
      const testNode = {
        id: 'test-node-1',
        name: 'testFunction',
        nodeType: 'function',
        changeType: 'modified' as const,
        filePath: '/src/test.ts',
        lineStart: 10,
        lineEnd: 20,
      };
      useDiffVisualizationStore.setState({
        graphData: { nodes: [testNode], links: [] },
        selectedNode: testNode,
      });

      // WHEN
      render(<App />);

      // THEN
      const panel = screen.getByTestId('entity-detail-panel');
      expect(panel).toHaveStyle({ visibility: 'visible' });
    });

    test('GIVEN selectedNode is null WHEN rendered THEN panel is hidden', () => {
      // GIVEN
      useDiffVisualizationStore.setState({
        graphData: { nodes: [], links: [] },
        selectedNode: null,
      });

      // WHEN
      render(<App />);

      // THEN
      const panel = screen.getByTestId('entity-detail-panel');
      expect(panel).toHaveStyle({ visibility: 'hidden' });
    });
  });

  describe('REQ-APP-002.4: DiffSummaryStats Wiring', () => {
    test('GIVEN diff summary in store WHEN rendered THEN displays counts', () => {
      // GIVEN
      useDiffVisualizationStore.setState({
        summary: {
          total_before_count: 100,
          total_after_count: 105,
          added_entity_count: 10,
          removed_entity_count: 5,
          modified_entity_count: 3,
          unchanged_entity_count: 87,
          relocated_entity_count: 0,
        },
        graphData: {
          nodes: [
            { id: '1', name: 'a', nodeType: 'fn', changeType: 'affected' },
            { id: '2', name: 'b', nodeType: 'fn', changeType: 'affected' },
          ],
          links: [],
        },
      });

      // WHEN
      render(<App />);

      // THEN
      expect(screen.getByTestId('badge-added-count')).toHaveTextContent('+10');
      expect(screen.getByTestId('badge-removed-count')).toHaveTextContent('-5');
      expect(screen.getByTestId('badge-modified-count')).toHaveTextContent('~3');
      expect(screen.getByTestId('badge-affected-count')).toHaveTextContent('2');
    });

    test('GIVEN diff in progress WHEN rendered THEN shows loading state', () => {
      // GIVEN
      useDiffVisualizationStore.setState({
        isDiffInProgress: true,
        summary: null,
      });

      // WHEN
      render(<App />);

      // THEN
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    });
  });

  describe('REQ-APP-002.5: ConnectionStatusIndicator Wiring', () => {
    test('GIVEN WebSocket connected WHEN rendered THEN shows connected status', () => {
      // GIVEN (mock returns 'connected')

      // WHEN
      render(<App />);

      // THEN
      const indicator = screen.getByTestId('connection-status-indicator');
      expect(indicator).toHaveTextContent('Connected');
    });
  });
});
```

### 4.4 Acceptance Criteria

- [ ] WorkspaceListSidebar receives all required store data
- [ ] DiffGraphCanvasView receives graphData and callbacks
- [ ] EntityDetailPanel visibility controlled by selectedNode
- [ ] DiffSummaryStats receives summary and computes blast radius
- [ ] ConnectionStatusIndicator receives WebSocket state
- [ ] All callbacks properly wired to store actions

---

## 5. REQ-APP-003: WebSocket Integration

### 5.1 Problem Statement

The WebSocket connection must be coordinated with workspace selection to ensure users receive real-time diff updates only for the workspace they are viewing.

### 5.2 Specification

#### REQ-APP-003.1: Connection Lifecycle

```
WHEN App component mounts
THEN SHALL call useWebsocketDiffStream hook
AND SHALL establish WebSocket connection automatically
AND SHALL display connection status via ConnectionStatusIndicator
```

#### REQ-APP-003.2: Workspace Subscription

```
WHEN user selects a workspace
  WITH workspace having watch_enabled_flag_status === true
THEN SHALL call subscribe(workspaceId) from useWebsocketDiffStream
  WITHIN 100ms of selection
AND SHALL clear previous graph data via clearAllGraphData action
AND SHALL display "Connecting to workspace..." status

WHEN user selects a workspace
  WITH workspace having watch_enabled_flag_status === false
THEN SHALL NOT call subscribe
AND SHALL display prompt to enable watching
```

#### REQ-APP-003.3: Diff Event Processing

```
WHEN WebSocket receives 'diff_started' event
THEN SHALL set isDiffInProgress to true via setDiffInProgress action
AND SHALL display loading state in DiffSummaryStats

WHEN WebSocket receives 'entity_added' event
THEN SHALL call applyEntityEvent action
AND SHALL add node to graph with changeType 'added'

WHEN WebSocket receives 'entity_removed' event
THEN SHALL call applyEntityEvent action
AND SHALL mark node changeType as 'removed'

WHEN WebSocket receives 'entity_modified' event
THEN SHALL call applyEntityEvent action
AND SHALL mark node changeType as 'modified'
AND SHALL update line range if provided

WHEN WebSocket receives 'edge_added' event
THEN SHALL call applyEntityEvent action
AND SHALL add link to graph

WHEN WebSocket receives 'edge_removed' event
THEN SHALL call applyEntityEvent action
AND SHALL remove link from graph

WHEN WebSocket receives 'diff_completed' event
THEN SHALL set isDiffInProgress to false via setDiffInProgress action
AND SHALL update summary via updateSummaryData action
```

#### REQ-APP-003.4: Subscription Cleanup

```
WHEN user selects a different workspace
  WITH previous workspace subscription active
THEN SHALL call unsubscribe() before subscribing to new workspace
AND SHALL clear graph data

WHEN user clears workspace selection
THEN SHALL call unsubscribe()
AND SHALL clear graph data
AND SHALL set selectedNode to null
```

#### REQ-APP-003.5: Error Handling

```
WHEN WebSocket connection fails
THEN SHALL display error status via ConnectionStatusIndicator
AND SHALL attempt reconnection with exponential backoff
AND SHALL show reconnection attempt count (e.g., "Reconnecting... (attempt 2/5)")

WHEN WebSocket receives 'error' event
THEN SHALL log error to console
AND SHALL display error message to user (toast or inline)
AND SHALL NOT crash the application
```

### 5.3 Test Template

```typescript
// File: src/__tests__/App.websocket.test.tsx

import { render, screen, waitFor, act } from '@testing-library/react';
import { App } from '../App';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useDiffVisualizationStore } from '../stores/diffVisualizationStore';

// Create mock functions we can spy on
const mockSubscribe = jest.fn();
const mockUnsubscribe = jest.fn();

jest.mock('../hooks/useWebsocketDiffStream', () => ({
  useWebsocketDiffStream: () => ({
    connectionStatus: 'connected',
    lastDiffEvent: null,
    reconnectAttempt: 0,
    maxReconnectAttempts: 5,
    subscribe: mockSubscribe,
    unsubscribe: mockUnsubscribe,
  }),
}));

describe('REQ-APP-003: WebSocket Integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    useWorkspaceStore.setState({
      workspaces: [
        {
          workspace_identifier_value: 'ws-001',
          workspace_display_name: 'Test Workspace',
          source_directory_path_value: '/path/to/project',
          base_database_path_value: '/path/to/base.db',
          live_database_path_value: '/path/to/live.db',
          watch_enabled_flag_status: true,
          created_timestamp_utc_value: '2026-01-23T00:00:00Z',
          last_indexed_timestamp_option: null,
        },
      ],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
    useDiffVisualizationStore.setState({
      graphData: { nodes: [], links: [] },
      selectedNode: null,
      summary: null,
      isDiffInProgress: false,
    });
  });

  describe('REQ-APP-003.2: Workspace Subscription', () => {
    test('GIVEN watch-enabled workspace WHEN selected THEN subscribe called', async () => {
      // GIVEN
      render(<App />);

      // WHEN
      act(() => {
        useWorkspaceStore.getState().actions.selectWorkspaceById('ws-001');
      });

      // THEN
      await waitFor(() => {
        expect(mockSubscribe).toHaveBeenCalledWith('ws-001');
      }, { timeout: 200 });
    });

    test('GIVEN watch-disabled workspace WHEN selected THEN subscribe NOT called', async () => {
      // GIVEN
      useWorkspaceStore.setState({
        workspaces: [
          {
            workspace_identifier_value: 'ws-002',
            workspace_display_name: 'Disabled Workspace',
            source_directory_path_value: '/path/to/project',
            base_database_path_value: '/path/to/base.db',
            live_database_path_value: '/path/to/live.db',
            watch_enabled_flag_status: false,
            created_timestamp_utc_value: '2026-01-23T00:00:00Z',
            last_indexed_timestamp_option: null,
          },
        ],
      });
      render(<App />);

      // WHEN
      act(() => {
        useWorkspaceStore.getState().actions.selectWorkspaceById('ws-002');
      });

      // THEN
      await waitFor(() => {
        expect(mockSubscribe).not.toHaveBeenCalled();
      }, { timeout: 200 });
    });
  });

  describe('REQ-APP-003.4: Subscription Cleanup', () => {
    test('GIVEN subscribed workspace WHEN different workspace selected THEN unsubscribe called first', async () => {
      // GIVEN
      useWorkspaceStore.setState({
        workspaces: [
          {
            workspace_identifier_value: 'ws-001',
            workspace_display_name: 'First Workspace',
            source_directory_path_value: '/path/one',
            base_database_path_value: '/path/base1.db',
            live_database_path_value: '/path/live1.db',
            watch_enabled_flag_status: true,
            created_timestamp_utc_value: '2026-01-23T00:00:00Z',
            last_indexed_timestamp_option: null,
          },
          {
            workspace_identifier_value: 'ws-002',
            workspace_display_name: 'Second Workspace',
            source_directory_path_value: '/path/two',
            base_database_path_value: '/path/base2.db',
            live_database_path_value: '/path/live2.db',
            watch_enabled_flag_status: true,
            created_timestamp_utc_value: '2026-01-23T00:00:00Z',
            last_indexed_timestamp_option: null,
          },
        ],
        selectedWorkspaceId: 'ws-001',
      });
      render(<App />);

      // WHEN - select different workspace
      act(() => {
        useWorkspaceStore.getState().actions.selectWorkspaceById('ws-002');
      });

      // THEN
      await waitFor(() => {
        expect(mockUnsubscribe).toHaveBeenCalled();
        expect(mockSubscribe).toHaveBeenCalledWith('ws-002');
      });
    });

    test('GIVEN subscribed workspace WHEN selection cleared THEN unsubscribe called', async () => {
      // GIVEN
      useWorkspaceStore.setState({
        selectedWorkspaceId: 'ws-001',
      });
      render(<App />);

      // WHEN
      act(() => {
        useWorkspaceStore.getState().actions.clearSelectedWorkspace();
      });

      // THEN
      await waitFor(() => {
        expect(mockUnsubscribe).toHaveBeenCalled();
      });
    });
  });
});
```

### 5.4 Acceptance Criteria

- [ ] WebSocket connection established on App mount
- [ ] Connection status displayed in header
- [ ] Subscribe called when watch-enabled workspace selected
- [ ] Subscribe NOT called for watch-disabled workspaces
- [ ] Unsubscribe called before switching workspaces
- [ ] Graph data cleared when workspace changes
- [ ] Diff events properly update store state
- [ ] Error events logged and displayed without crashing
- [ ] Reconnection attempts shown to user

---

## 6. REQ-APP-004: User Flow

### 6.1 Problem Statement

Users need a clear, intuitive flow from workspace selection through diff visualization to entity inspection, with proper keyboard support.

### 6.2 Specification

#### REQ-APP-004.1: Complete User Flow

```
WHEN user opens application
THEN SHALL display workspace list in sidebar
AND SHALL display empty graph state in canvas
AND SHALL display "No changes detected" in DiffSummaryStats
AND SHALL display connection status

WHEN user selects a workspace with watching enabled
THEN SHALL subscribe to workspace WebSocket stream
AND SHALL update canvas when diff events arrive
AND SHALL update summary stats when diff completes

WHEN user clicks a node in the graph
THEN SHALL animate camera to focus on node (duration 1000ms)
AND SHALL open EntityDetailPanel on right side
AND SHALL display node name, type, and change badge
AND SHALL display file location with line numbers
AND SHALL display incoming and outgoing dependencies

WHEN user clicks a dependency item in EntityDetailPanel
THEN SHALL select that node (call selectNodeById)
AND SHALL update EntityDetailPanel to show new node
AND SHALL animate camera to new node
```

#### REQ-APP-004.2: Keyboard Navigation

```
WHEN user presses Escape key
  WITH EntityDetailPanel visible
THEN SHALL close EntityDetailPanel
AND SHALL clear selectedNode in store
AND SHALL NOT affect other application state

WHEN user presses Escape key
  WITH mobile sidebar visible
THEN SHALL close mobile sidebar
AND SHALL return focus to toggle button
```

#### REQ-APP-004.3: Empty States

```
WHEN no workspace is selected
THEN SHALL display in canvas:
  - Message: "No graph data available"
  - Submessage: "Select a workspace and enable watching to see changes"

WHEN workspace selected but no diff events received
THEN SHALL display in canvas:
  - Message: "Waiting for changes..."
  - Submessage: "Make changes to files in [workspace_name] to see the diff"

WHEN workspace selected and diff completed with no changes
THEN SHALL display in DiffSummaryStats: "No changes detected"
```

#### REQ-APP-004.4: Loading States

```
WHEN diff analysis is in progress
THEN SHALL display in DiffSummaryStats:
  - Spinning indicator
  - Text: "Analyzing changes..."
  - Skeleton placeholders for badge counts
AND SHALL disable interaction with graph (prevent node clicks)
```

### 6.3 Test Template

```typescript
// File: src/__tests__/App.userflow.test.tsx

import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../App';
import { useWorkspaceStore } from '../stores/workspaceStore';
import { useDiffVisualizationStore } from '../stores/diffVisualizationStore';

const mockSubscribe = jest.fn();
const mockUnsubscribe = jest.fn();

jest.mock('../hooks/useWebsocketDiffStream', () => ({
  useWebsocketDiffStream: () => ({
    connectionStatus: 'connected',
    lastDiffEvent: null,
    reconnectAttempt: 0,
    maxReconnectAttempts: 5,
    subscribe: mockSubscribe,
    unsubscribe: mockUnsubscribe,
  }),
}));

describe('REQ-APP-004: User Flow', () => {
  const user = userEvent.setup();

  beforeEach(() => {
    jest.clearAllMocks();
    useWorkspaceStore.setState({
      workspaces: [
        {
          workspace_identifier_value: 'ws-001',
          workspace_display_name: 'Test Project',
          source_directory_path_value: '/path/to/project',
          base_database_path_value: '/path/to/base.db',
          live_database_path_value: '/path/to/live.db',
          watch_enabled_flag_status: true,
          created_timestamp_utc_value: '2026-01-23T00:00:00Z',
          last_indexed_timestamp_option: null,
        },
      ],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
    useDiffVisualizationStore.setState({
      graphData: { nodes: [], links: [] },
      selectedNode: null,
      summary: null,
      isDiffInProgress: false,
    });
  });

  describe('REQ-APP-004.1: Complete User Flow', () => {
    test('GIVEN initial state WHEN app renders THEN shows empty state', () => {
      // GIVEN/WHEN
      render(<App />);

      // THEN
      expect(screen.getByText('No graph data available')).toBeInTheDocument();
      expect(screen.getByText(/Select a workspace/)).toBeInTheDocument();
    });

    test('GIVEN workspace selected WHEN node clicked THEN panel opens', async () => {
      // GIVEN
      const testNode = {
        id: 'fn-main',
        name: 'main',
        nodeType: 'function',
        changeType: 'modified' as const,
        filePath: '/src/main.rs',
        lineStart: 10,
        lineEnd: 50,
      };
      useDiffVisualizationStore.setState({
        graphData: { nodes: [testNode], links: [] },
      });
      render(<App />);

      // WHEN - simulate node click
      act(() => {
        useDiffVisualizationStore.getState().actions.selectNodeById('fn-main');
      });

      // THEN
      await waitFor(() => {
        const panel = screen.getByTestId('entity-detail-panel');
        expect(panel).toHaveStyle({ visibility: 'visible' });
        expect(screen.getByTestId('entity-name')).toHaveTextContent('main');
        expect(screen.getByTestId('entity-type')).toHaveTextContent('function');
        expect(screen.getByTestId('file-location')).toHaveTextContent('/src/main.rs:10-50');
      });
    });
  });

  describe('REQ-APP-004.2: Keyboard Navigation', () => {
    test('GIVEN EntityDetailPanel visible WHEN Escape pressed THEN panel closes', async () => {
      // GIVEN
      const testNode = {
        id: 'fn-test',
        name: 'testFn',
        nodeType: 'function',
        changeType: 'added' as const,
      };
      useDiffVisualizationStore.setState({
        graphData: { nodes: [testNode], links: [] },
        selectedNode: testNode,
      });
      render(<App />);

      // Verify panel is visible
      expect(screen.getByTestId('entity-detail-panel')).toHaveStyle({ visibility: 'visible' });

      // WHEN
      fireEvent.keyDown(document, { key: 'Escape' });

      // THEN
      await waitFor(() => {
        expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
        expect(screen.getByTestId('entity-detail-panel')).toHaveStyle({ visibility: 'hidden' });
      });
    });
  });

  describe('REQ-APP-004.3: Empty States', () => {
    test('GIVEN no workspace selected WHEN rendered THEN shows selection prompt', () => {
      // GIVEN/WHEN
      render(<App />);

      // THEN
      expect(screen.getByText('No graph data available')).toBeInTheDocument();
      expect(screen.getByText(/Select a workspace and enable watching/)).toBeInTheDocument();
    });
  });

  describe('REQ-APP-004.4: Loading States', () => {
    test('GIVEN diff in progress WHEN rendered THEN shows loading indicator', () => {
      // GIVEN
      useDiffVisualizationStore.setState({
        isDiffInProgress: true,
      });

      // WHEN
      render(<App />);

      // THEN
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
      expect(screen.getByText('Analyzing changes...')).toBeInTheDocument();
    });
  });
});
```

### 6.4 Acceptance Criteria

- [ ] Initial empty state displays correctly
- [ ] Workspace selection triggers subscription
- [ ] Node click opens detail panel with correct data
- [ ] Dependency clicks navigate to related nodes
- [ ] Escape key closes detail panel
- [ ] Escape key closes mobile sidebar
- [ ] Loading state displays during diff analysis
- [ ] All empty states have helpful messages

---

## 7. Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Initial render | < 200ms | Performance.mark/measure in App mount |
| Workspace switch | < 100ms | Time from click to subscription call |
| Node selection | < 50ms | Time from click to panel visible |
| Graph update (100 nodes) | < 16ms | RAF budget for 60fps |
| Graph update (1000 nodes) | < 100ms | WebSocket event to render complete |
| Memory baseline | < 50MB | Chrome DevTools heap snapshot |
| Memory with 1000 nodes | < 150MB | Chrome DevTools heap snapshot |

---

## 8. Error Handling Matrix

| Error Scenario | User-Facing Behavior | Technical Behavior |
|----------------|---------------------|-------------------|
| Workspace fetch fails | "Failed to load workspaces" + Retry button | Log error, set error state |
| WebSocket connection fails | "Connection error" status dot | Exponential backoff reconnection |
| WebSocket reconnecting | "Reconnecting... (2/5)" | Automatic retry up to 5 times |
| Invalid diff event | Silently ignored | Console.warn, skip event |
| Selected node removed | "Entity not found" message in panel | Clear selection gracefully |
| Network timeout | "Connection lost" status | Trigger reconnection |

---

## 9. Component Integration Diagram

```
+-------------------------------------------------------------------+
|                           App.tsx                                   |
+-------------------------------------------------------------------+
|                                                                     |
|  +------------------+  +--------------------------------+           |
|  | useWebsocket     |  |          Header Bar           |           |
|  | DiffStream       |  |  +-------------+ +----------+ |           |
|  |                  |  |  |DiffSummary  | |Connection| |           |
|  | - connectionStatus  |  |Stats        | |Status    | |           |
|  | - subscribe()    |  |  +-------------+ +----------+ |           |
|  | - unsubscribe()  |  +--------------------------------+           |
|  +--------+---------+                                               |
|           |                                                         |
|           v                                                         |
|  +--------+---------+  +------------------------+  +-------------+  |
|  | workspaceStore   |  | diffVisualization      |  | EntityDetail |  |
|  |                  |  | Store                  |  | Panel       |  |
|  | - workspaces     |  |                        |  |             |  |
|  | - selectedId     |  | - graphData            |  | selectedNode |  |
|  +--------+---------+  | - selectedNode         |  | from store  |  |
|           |            | - summary              |  +------+------+  |
|           |            +----------+-------------+         |         |
|           |                       |                       |         |
|           v                       v                       |         |
|  +------------------+  +------------------------+         |         |
|  | WorkspaceList    |  | DiffGraphCanvasView    |<--------+         |
|  | Sidebar          |  |                        |                   |
|  |                  |  | graphData from store   |                   |
|  | workspaces from  |  | onNodeClick -> store   |                   |
|  | store            |  +------------------------+                   |
|  +------------------+                                               |
|                                                                     |
+-------------------------------------------------------------------+
```

---

## 10. Implementation Checklist

### Phase 1: Layout Structure
- [ ] Create App component with grid layout
- [ ] Add responsive breakpoints (mobile/desktop)
- [ ] Implement sidebar toggle for mobile
- [ ] Add all test IDs
- [ ] Verify dark theme styling

### Phase 2: Component Wiring
- [ ] Wire WorkspaceListSidebar to store
- [ ] Wire DiffGraphCanvasView to store
- [ ] Wire EntityDetailPanel visibility
- [ ] Wire DiffSummaryStats with blast radius calculation
- [ ] Wire ConnectionStatusIndicator to WebSocket hook

### Phase 3: WebSocket Integration
- [ ] Add useEffect for workspace subscription
- [ ] Implement subscription cleanup
- [ ] Handle all diff event types
- [ ] Add error handling

### Phase 4: User Flow Polish
- [ ] Implement Escape key handling
- [ ] Add empty state messages
- [ ] Add loading states
- [ ] Test complete user flow

---

## 11. Appendix: Type Definitions Reference

### Store Selectors Used

```typescript
// From workspaceStore
useWorkspaceList(): WorkspaceMetadata[]
useSelectedWorkspaceId(): string | null
useWorkspaceLoading(): boolean
useWorkspaceError(): string | null
useWorkspaceActions(): {
  fetchWorkspaceListData: () => Promise<void>;
  selectWorkspaceById: (id: string) => void;
  toggleWorkspaceWatchState: (id: string, enabled: boolean) => Promise<void>;
  createWorkspaceFromPath: (path: string, name?: string) => Promise<void>;
  clearSelectedWorkspace: () => void;
}

// From diffVisualizationStore
useGraphData(): ForceGraphData
useSelectedNode(): GraphNode | null
useDiffSummary(): DiffSummaryData | null
useIsDiffInProgress(): boolean
useDiffVisualizationActions(): {
  setGraphDataFromApi: (apiResponse: ApiDiffVisualization) => void;
  selectNodeById: (nodeId: string) => void;
  clearSelectedNode: () => void;
  updateSummaryData: (summary: DiffSummaryData) => void;
  setDiffInProgress: (inProgress: boolean) => void;
  applyEntityEvent: (event: WebSocketServerEvent) => void;
  clearAllGraphData: () => void;
}
```

### WebSocket Hook Return Type

```typescript
interface UseWebsocketDiffStreamReturn {
  connectionStatus: ConnectionStatus;
  lastDiffEvent: WebSocketServerEvent | null;
  reconnectAttempt: number;
  maxReconnectAttempts: number;
  subscribe: (workspaceId: string) => void;
  unsubscribe: () => void;
}
```

### Component Props Summary

```typescript
// WorkspaceListSidebar
interface WorkspaceListSidebarProps {
  className?: string;
}

// DiffGraphCanvasView
interface DiffGraphCanvasViewProps {
  graphData: ForceGraphData;
  onNodeClick?: (node: GraphNode) => void;
  onBackgroundClick?: () => void;
  className?: string;
}

// ConnectionStatusIndicator
interface ConnectionStatusIndicatorProps {
  connectionStatus: ConnectionStatus;
  reconnectAttempt?: number;
  maxReconnectAttempts?: number;
  className?: string;
}

// EntityDetailPanel
// (uses store directly, no props)

// DiffSummaryStats
interface DiffSummaryStatsProps {
  blastRadiusCount: number;
}
```

---

## 12. Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2026-01-23 | Executable Specs Agent | Initial specification |
