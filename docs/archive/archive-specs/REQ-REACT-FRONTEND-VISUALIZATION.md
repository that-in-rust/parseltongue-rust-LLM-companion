# React Frontend Visualization Specification

## Phase 2.3-2.4 - React + react-force-graph-3d Frontend

**Document Version**: 1.0.0
**Created**: 2026-01-23
**Status**: Specification Complete
**Phase**: 2.3 (React Core) + 2.4 (3D Visualization)
**Dependency**: Phase 2.1-2.2 (Backend) Complete

---

## Overview

### Problem Statement

Developers using Parseltongue need a visual interface to:

1. **Manage workspaces** - Create, select, and configure workspaces without CLI commands
2. **Visualize dependency changes** - See added/removed/modified entities in 3D space
3. **Understand blast radius** - Identify affected entities when code changes
4. **Monitor in real-time** - Watch changes flow through the dependency graph as files are edited

Currently, the Rust backend provides HTTP and WebSocket APIs, but there is no user interface. Developers must use curl commands and cannot visualize the dependency graph.

### Solution

A React frontend with:

| Component | Technology | Purpose |
|-----------|------------|---------|
| **WorkspaceListSidebar** | React + Zustand | Workspace CRUD and selection |
| **DiffGraphCanvasView** | react-force-graph-3d | 3D force-directed graph visualization |
| **ConnectionStatusIndicator** | React | WebSocket connection state display |
| **EntityDetailPanel** | React | Details for selected node |
| **DiffSummaryStats** | React | Summary of changes in current diff |

### Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| Framework | React | 18.x |
| Language | TypeScript | 5.x |
| 3D Visualization | react-force-graph-3d | 1.24.x |
| State Management | zustand | 4.x |
| Data Fetching | @tanstack/react-query | 5.x |
| Styling | tailwindcss | 3.x |
| Build Tool | vite | 5.x |
| E2E Testing | Playwright | latest |

---

## TypeScript Types (Match Rust Backend)

### API Response Types

```typescript
// types/api.ts

// Workspace types (match parseltongue-core/src/workspace/types.rs)
export interface WorkspaceMetadata {
  workspace_identifier_value: string;
  workspace_display_name: string;
  source_directory_path_value: string;
  base_database_path_value: string;
  live_database_path_value: string;
  watch_enabled_flag_status: boolean;
  created_timestamp_utc_value: string;
  last_indexed_timestamp_option: string | null;
}

export interface WorkspaceCreateRequest {
  source_path_directory_value: string;
  workspace_display_name_option?: string;
}

export interface WorkspaceWatchToggleRequest {
  workspace_identifier_target_value: string;
  watch_enabled_desired_state: boolean;
}

export interface WorkspaceListResponse {
  success: boolean;
  endpoint: string;
  workspaces: WorkspaceMetadata[];
  total_workspace_count_value: number;
  token_estimate: number;
}

export interface WorkspaceOperationResponse {
  success: boolean;
  endpoint: string;
  workspace: WorkspaceMetadata;
  token_estimate: number;
}

export interface WorkspaceErrorResponse {
  error: string;
  code: string;
  existing_workspace_id?: string;
}
```

### WebSocket Event Types

```typescript
// types/websocket.ts

// Match websocket_streaming_module/message_types.rs

export interface LineRangeData {
  start: number;
  end: number;
}

export interface DiffSummaryData {
  total_before_count: number;
  total_after_count: number;
  added_entity_count: number;
  removed_entity_count: number;
  modified_entity_count: number;
  unchanged_entity_count: number;
  relocated_entity_count: number;
}

export type WebSocketClientMessage =
  | { action: 'subscribe'; workspace_id: string }
  | { action: 'unsubscribe' }
  | { action: 'ping' };

export type WebSocketServerEvent =
  | { event: 'subscribed'; workspace_id: string; workspace_name: string; timestamp: string }
  | { event: 'unsubscribed'; timestamp: string }
  | { event: 'pong'; timestamp: string }
  | { event: 'diff_started'; workspace_id: string; files_changed: number; triggered_by: string; timestamp: string }
  | { event: 'entity_added'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; line_range: LineRangeData | null; timestamp: string }
  | { event: 'entity_removed'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; timestamp: string }
  | { event: 'entity_modified'; workspace_id: string; entity_key: string; entity_type: string; file_path: string; before_line_range: LineRangeData | null; after_line_range: LineRangeData | null; timestamp: string }
  | { event: 'edge_added'; workspace_id: string; from_entity_key: string; to_entity_key: string; edge_type: string; timestamp: string }
  | { event: 'edge_removed'; workspace_id: string; from_entity_key: string; to_entity_key: string; edge_type: string; timestamp: string }
  | { event: 'diff_completed'; workspace_id: string; summary: DiffSummaryData; blast_radius_count: number; duration_ms: number; timestamp: string }
  | { event: 'error'; code: string; message: string; timestamp: string };
```

### Visualization Types

```typescript
// types/visualization.ts

export type ChangeType = 'added' | 'removed' | 'modified' | 'affected' | null;

export interface GraphNode {
  id: string;
  name: string;
  nodeType: string;
  changeType: ChangeType;
  filePath?: string;
  lineStart?: number;
  lineEnd?: number;
}

export interface GraphLink {
  source: string;
  target: string;
  edgeType: string;
}

export interface ForceGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export const CHANGE_TYPE_COLORS: Record<ChangeType | 'null', string> = {
  added: '#22c55e',     // green-500
  removed: '#ef4444',   // red-500
  modified: '#f59e0b',  // amber-500
  affected: '#3b82f6',  // blue-500
  null: '#6b7280',      // gray-500
};
```

---

# Section 1: Workspace Sidebar Requirements

## REQ-SIDEBAR-001: Workspace List Display

### Problem Statement

Users need to see all configured workspaces in a sidebar to select which one to visualize.

### Specification

#### REQ-SIDEBAR-001.1: Render Workspace List on Mount

```
WHEN WorkspaceListSidebar component mounts
THEN SHALL call GET /workspace-list-all endpoint
  AND SHALL display loading indicator while fetching
  AND SHALL render workspace list within 500ms of response
  AND SHALL display each workspace with:
    - workspace_display_name as primary text
    - source_directory_path_value as secondary text
    - watch_enabled_flag_status as toggle indicator
  AND SHALL order workspaces by created_timestamp_utc_value (newest first)
```

#### REQ-SIDEBAR-001.2: Empty State Display

```
WHEN WorkspaceListSidebar receives empty workspaces array
THEN SHALL display empty state message:
  - Primary: "No workspaces configured"
  - Secondary: "Add a workspace to get started"
  AND SHALL display "Add Workspace" button prominently
  AND SHALL NOT display any workspace list items
```

#### REQ-SIDEBAR-001.3: Error State Display

```
WHEN GET /workspace-list-all request fails
THEN SHALL display error message with:
  - Error icon
  - Message: "Failed to load workspaces"
  - Retry button
  AND SHALL log error to console with details
  AND SHALL allow retry by clicking retry button
```

### Verification Test Template

```typescript
// __tests__/components/WorkspaceListSidebar.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import { WorkspaceListSidebar } from '@/components/Layout/WorkspaceListSidebar';
import { server } from '@/mocks/server';
import { rest } from 'msw';

describe('REQ-SIDEBAR-001: Workspace List Display', () => {
  // REQ-SIDEBAR-001.1
  test('renders workspace list after successful fetch', async () => {
    // GIVEN mock API returns workspaces
    server.use(
      rest.get('*/workspace-list-all', (req, res, ctx) => {
        return res(ctx.json({
          success: true,
          workspaces: [
            { workspace_identifier_value: 'ws_1', workspace_display_name: 'Project A' },
            { workspace_identifier_value: 'ws_2', workspace_display_name: 'Project B' },
          ],
          total_workspace_count_value: 2,
        }));
      })
    );

    // WHEN component renders
    render(<WorkspaceListSidebar />);

    // THEN should show loading then workspaces
    expect(screen.getByTestId('workspace-list-loading')).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText('Project A')).toBeInTheDocument();
      expect(screen.getByText('Project B')).toBeInTheDocument();
    });
  });

  // REQ-SIDEBAR-001.2
  test('displays empty state when no workspaces exist', async () => {
    // GIVEN mock API returns empty list
    server.use(
      rest.get('*/workspace-list-all', (req, res, ctx) => {
        return res(ctx.json({
          success: true,
          workspaces: [],
          total_workspace_count_value: 0,
        }));
      })
    );

    // WHEN component renders
    render(<WorkspaceListSidebar />);

    // THEN should show empty state
    await waitFor(() => {
      expect(screen.getByText('No workspaces configured')).toBeInTheDocument();
      expect(screen.getByText('Add Workspace')).toBeInTheDocument();
    });
  });

  // REQ-SIDEBAR-001.3
  test('displays error state when fetch fails', async () => {
    // GIVEN mock API fails
    server.use(
      rest.get('*/workspace-list-all', (req, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    // WHEN component renders
    render(<WorkspaceListSidebar />);

    // THEN should show error state
    await waitFor(() => {
      expect(screen.getByText('Failed to load workspaces')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });
  });
});
```

---

## REQ-SIDEBAR-002: Workspace Selection

### Problem Statement

Users must be able to select a workspace to view its dependency graph.

### Specification

#### REQ-SIDEBAR-002.1: Click to Select Workspace

```
WHEN user clicks on a workspace item in the list
THEN SHALL update selectedWorkspaceId in workspaceStore
  AND SHALL apply selected visual styling (bg-blue-600)
  AND SHALL trigger WebSocket subscription to selected workspace
  AND SHALL update DiffGraphCanvasView with workspace data
  AND SHALL complete selection within 100ms
```

#### REQ-SIDEBAR-002.2: Keyboard Navigation Support

```
WHEN user presses Enter or Space on focused workspace item
THEN SHALL select that workspace (same as click)
  AND SHALL maintain focus on selected item
  AND SHALL announce selection to screen readers
```

#### REQ-SIDEBAR-002.3: Only One Workspace Selected

```
WHEN user selects a new workspace
  WITH another workspace already selected
THEN SHALL deselect previous workspace
  AND SHALL unsubscribe from previous workspace WebSocket
  AND SHALL subscribe to new workspace WebSocket
  AND SHALL clear previous graph data before loading new
```

### Verification Test Template

```typescript
// __tests__/components/WorkspaceListSidebar.selection.test.tsx
describe('REQ-SIDEBAR-002: Workspace Selection', () => {
  // REQ-SIDEBAR-002.1
  test('clicking workspace updates store and applies styling', async () => {
    // GIVEN rendered sidebar with workspaces
    const { getByTestId } = render(<WorkspaceListSidebar />);
    await waitFor(() => expect(getByTestId('workspace-item-ws_1')).toBeInTheDocument());

    // WHEN user clicks workspace
    fireEvent.click(getByTestId('workspace-item-ws_1'));

    // THEN store should update and styling should apply
    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_1');
    expect(getByTestId('workspace-item-ws_1')).toHaveClass('bg-blue-600');
  });

  // REQ-SIDEBAR-002.2
  test('keyboard Enter selects workspace', async () => {
    // GIVEN focused workspace item
    const { getByTestId } = render(<WorkspaceListSidebar />);
    const item = getByTestId('workspace-item-ws_1');
    item.focus();

    // WHEN user presses Enter
    fireEvent.keyDown(item, { key: 'Enter' });

    // THEN workspace should be selected
    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_1');
  });

  // REQ-SIDEBAR-002.3
  test('selecting new workspace deselects previous', async () => {
    // GIVEN ws_1 is selected
    useWorkspaceStore.setState({ selectedWorkspaceId: 'ws_1' });
    const { getByTestId } = render(<WorkspaceListSidebar />);

    // WHEN user clicks ws_2
    fireEvent.click(getByTestId('workspace-item-ws_2'));

    // THEN ws_1 should be deselected, ws_2 selected
    expect(getByTestId('workspace-item-ws_1')).not.toHaveClass('bg-blue-600');
    expect(getByTestId('workspace-item-ws_2')).toHaveClass('bg-blue-600');
  });
});
```

---

## REQ-SIDEBAR-003: Create Workspace

### Problem Statement

Users need to add new workspaces by specifying a directory path.

### Specification

#### REQ-SIDEBAR-003.1: Open Create Workspace Dialog

```
WHEN user clicks "Add Workspace" button
THEN SHALL open modal dialog with:
  - Title: "Add Workspace"
  - Input field for source_path_directory_value (required)
  - Input field for workspace_display_name_option (optional)
  - "Create" button (disabled until path entered)
  - "Cancel" button
  AND SHALL focus on path input field
```

#### REQ-SIDEBAR-003.2: Submit Create Request

```
WHEN user enters valid path and clicks "Create"
THEN SHALL call POST /workspace-create-from-path with:
  {
    "source_path_directory_value": "{entered_path}",
    "workspace_display_name_option": "{entered_name}" | null
  }
  AND SHALL show loading state on Create button
  AND SHALL disable form inputs during request
  AND SHALL complete request within 30000ms (indexing may be slow)
```

#### REQ-SIDEBAR-003.3: Handle Create Success

```
WHEN POST /workspace-create-from-path returns 200 OK
THEN SHALL close the dialog
  AND SHALL add new workspace to workspaceStore.workspaces
  AND SHALL select the newly created workspace
  AND SHALL display success toast: "Workspace created: {name}"
  AND SHALL NOT require page refresh
```

#### REQ-SIDEBAR-003.4: Handle Create Error

```
WHEN POST /workspace-create-from-path returns error
  WITH code in [PATH_NOT_FOUND, PATH_NOT_DIRECTORY, WORKSPACE_ALREADY_EXISTS]
THEN SHALL display error message below path input
  AND SHALL NOT close the dialog
  AND SHALL allow user to correct and retry
  AND SHALL re-enable form inputs
```

### Verification Test Template

```typescript
// __tests__/components/CreateWorkspaceDialog.test.tsx
describe('REQ-SIDEBAR-003: Create Workspace', () => {
  // REQ-SIDEBAR-003.1
  test('clicking Add Workspace opens dialog', async () => {
    render(<WorkspaceListSidebar />);

    fireEvent.click(screen.getByTestId('create-workspace-button'));

    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByLabelText(/directory path/i)).toHaveFocus();
  });

  // REQ-SIDEBAR-003.2
  test('submitting form calls API with correct payload', async () => {
    const mockCreate = jest.fn();
    server.use(
      rest.post('*/workspace-create-from-path', async (req, res, ctx) => {
        mockCreate(await req.json());
        return res(ctx.json({ success: true, workspace: mockWorkspace }));
      })
    );

    render(<WorkspaceListSidebar />);
    fireEvent.click(screen.getByTestId('create-workspace-button'));

    await userEvent.type(screen.getByLabelText(/directory path/i), '/path/to/project');
    await userEvent.type(screen.getByLabelText(/display name/i), 'My Project');
    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledWith({
        source_path_directory_value: '/path/to/project',
        workspace_display_name_option: 'My Project',
      });
    });
  });

  // REQ-SIDEBAR-003.4
  test('displays validation error when path not found', async () => {
    server.use(
      rest.post('*/workspace-create-from-path', (req, res, ctx) => {
        return res(ctx.status(400), ctx.json({
          error: 'Source path does not exist',
          code: 'PATH_NOT_FOUND',
        }));
      })
    );

    render(<WorkspaceListSidebar />);
    fireEvent.click(screen.getByTestId('create-workspace-button'));
    await userEvent.type(screen.getByLabelText(/directory path/i), '/invalid/path');
    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(screen.getByText(/path does not exist/i)).toBeInTheDocument();
      expect(screen.getByRole('dialog')).toBeInTheDocument(); // Still open
    });
  });
});
```

---

## REQ-SIDEBAR-004: Watch Toggle Control

### Problem Statement

Users need to enable/disable file watching for a workspace to control when live updates occur.

### Specification

#### REQ-SIDEBAR-004.1: Display Watch Toggle

```
WHEN rendering a workspace item
THEN SHALL display toggle button showing current watch_enabled_flag_status
  WITH visual state:
    - Enabled: Green background, "Watching" text
    - Disabled: Gray background, "Watch" text
  AND SHALL be accessible with aria-pressed attribute
```

#### REQ-SIDEBAR-004.2: Toggle Watch State

```
WHEN user clicks watch toggle button
THEN SHALL call POST /workspace-watch-toggle with:
  {
    "workspace_identifier_target_value": "{workspace_id}",
    "watch_enabled_desired_state": !current_state
  }
  AND SHALL show loading spinner on toggle button
  AND SHALL prevent multiple rapid clicks (debounce 300ms)
```

#### REQ-SIDEBAR-004.3: Handle Toggle Success

```
WHEN POST /workspace-watch-toggle returns 200 OK
THEN SHALL update workspace in store with new watch_enabled_flag_status
  AND SHALL update toggle button visual state
  AND SHALL display toast:
    - If enabled: "Watching: {name}"
    - If disabled: "Stopped watching: {name}"
```

#### REQ-SIDEBAR-004.4: Handle Toggle Error

```
WHEN POST /workspace-watch-toggle returns error
THEN SHALL revert toggle visual state to previous
  AND SHALL display error toast with message
  AND SHALL NOT update store state
```

### Verification Test Template

```typescript
// __tests__/components/WorkspaceWatchToggle.test.tsx
describe('REQ-SIDEBAR-004: Watch Toggle Control', () => {
  // REQ-SIDEBAR-004.1
  test('displays correct visual state for watch enabled', () => {
    const workspace = { ...mockWorkspace, watch_enabled_flag_status: true };
    render(<WorkspaceListItem workspace={workspace} />);

    const toggle = screen.getByTestId(`watch-toggle-${workspace.workspace_identifier_value}`);
    expect(toggle).toHaveClass('bg-green-600');
    expect(toggle).toHaveTextContent('Watching');
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
  });

  // REQ-SIDEBAR-004.2
  test('calls API when toggle clicked', async () => {
    const mockToggle = jest.fn();
    server.use(
      rest.post('*/workspace-watch-toggle', async (req, res, ctx) => {
        mockToggle(await req.json());
        return res(ctx.json({ success: true, workspace: mockWorkspace }));
      })
    );

    render(<WorkspaceListItem workspace={{ ...mockWorkspace, watch_enabled_flag_status: false }} />);
    fireEvent.click(screen.getByTestId('watch-toggle-ws_1'));

    await waitFor(() => {
      expect(mockToggle).toHaveBeenCalledWith({
        workspace_identifier_target_value: 'ws_1',
        watch_enabled_desired_state: true,
      });
    });
  });

  // REQ-SIDEBAR-004.4
  test('reverts visual state on error', async () => {
    server.use(
      rest.post('*/workspace-watch-toggle', (req, res, ctx) => {
        return res(ctx.status(500));
      })
    );

    render(<WorkspaceListItem workspace={{ ...mockWorkspace, watch_enabled_flag_status: false }} />);
    const toggle = screen.getByTestId('watch-toggle-ws_1');

    fireEvent.click(toggle);

    await waitFor(() => {
      expect(toggle).toHaveClass('bg-gray-600'); // Reverted
      expect(screen.getByText(/failed/i)).toBeInTheDocument();
    });
  });
});
```

---

# Section 2: 3D Visualization Requirements

## REQ-VIZ-001: Graph Rendering

### Problem Statement

Users need to visualize the dependency graph in 3D space to understand codebase structure.

### Specification

#### REQ-VIZ-001.1: Render ForceGraph3D Component

```
WHEN DiffGraphCanvasView receives graphData with nodes and links
THEN SHALL render ForceGraph3D component with:
  - graphData prop containing nodes[] and links[]
  - backgroundColor: "#111827" (gray-900)
  - controlType: "orbit"
  AND SHALL complete initial render within 1000ms
  AND SHALL achieve 60fps for graphs up to 500 nodes
```

#### REQ-VIZ-001.2: Empty Graph State

```
WHEN DiffGraphCanvasView receives empty graphData
  WITH nodes.length === 0
THEN SHALL display centered message:
  - "No graph data available"
  - "Select a workspace and enable watching to see changes"
  AND SHALL NOT render ForceGraph3D component
```

#### REQ-VIZ-001.3: Large Graph Performance

```
WHEN graphData contains more than 1000 nodes
THEN SHALL apply Level of Detail (LOD) rendering:
  - Distant nodes rendered as simple spheres
  - Nearby nodes rendered with labels
  - Edges culled beyond camera frustum
  AND SHALL maintain minimum 30fps
  AND SHALL display node count indicator: "Showing 1000+ nodes"
```

### Performance Contract

| Metric | Target | Measurement Method |
|--------|--------|--------------------|
| Initial Render | < 1000ms | Performance.measure() |
| Frame Rate (< 500 nodes) | >= 60fps | requestAnimationFrame delta |
| Frame Rate (> 1000 nodes) | >= 30fps | requestAnimationFrame delta |
| Memory (1000 nodes) | < 200MB | Performance.memory |

### Verification Test Template

```typescript
// __tests__/components/DiffGraphCanvasView.test.tsx
describe('REQ-VIZ-001: Graph Rendering', () => {
  // REQ-VIZ-001.1
  test('renders ForceGraph3D with provided data', async () => {
    const graphData = {
      nodes: [
        { id: 'n1', name: 'FunctionA', nodeType: 'function', changeType: null },
        { id: 'n2', name: 'FunctionB', nodeType: 'function', changeType: 'added' },
      ],
      links: [
        { source: 'n1', target: 'n2', edgeType: 'Calls' },
      ],
    };

    render(<DiffGraphCanvasView graphData={graphData} />);

    expect(screen.getByTestId('diff-graph-canvas')).toBeInTheDocument();
    // Canvas should be rendered
    expect(document.querySelector('canvas')).toBeInTheDocument();
  });

  // REQ-VIZ-001.2
  test('displays empty state when no graph data', () => {
    render(<DiffGraphCanvasView graphData={{ nodes: [], links: [] }} />);

    expect(screen.getByText('No graph data available')).toBeInTheDocument();
    expect(document.querySelector('canvas')).not.toBeInTheDocument();
  });
});
```

---

## REQ-VIZ-002: Node Styling by Change Type

### Problem Statement

Users need to visually distinguish nodes by their change status in the diff.

### Specification

#### REQ-VIZ-002.1: Node Color Mapping

```
WHEN rendering graph nodes
THEN SHALL apply color based on node.changeType:
  - 'added': #22c55e (green-500)
  - 'removed': #ef4444 (red-500)
  - 'modified': #f59e0b (amber-500)
  - 'affected': #3b82f6 (blue-500)
  - null (unchanged): #6b7280 (gray-500)
  AND colors SHALL be consistent across all views
```

#### REQ-VIZ-002.2: Node Size by Change Type

```
WHEN rendering graph nodes
THEN SHALL apply size based on node.changeType:
  - Changed nodes (added/removed/modified/affected): val = 15
  - Unchanged nodes (null): val = 5
  AND size difference SHALL be visually apparent
```

#### REQ-VIZ-002.3: Node Labels

```
WHEN rendering graph nodes
THEN SHALL display label on hover showing:
  - node.name (primary)
  - node.nodeType in parentheses
  - Example: "handleAuth (function)"
  AND label SHALL be readable against background
  AND label SHALL disappear on mouseout
```

### Verification Test Template

```typescript
// __tests__/utils/colorMappingHelpers.test.ts
describe('REQ-VIZ-002: Node Styling', () => {
  // REQ-VIZ-002.1
  test('getNodeColor returns correct colors for change types', () => {
    expect(getNodeColorByChangeType('added')).toBe('#22c55e');
    expect(getNodeColorByChangeType('removed')).toBe('#ef4444');
    expect(getNodeColorByChangeType('modified')).toBe('#f59e0b');
    expect(getNodeColorByChangeType('affected')).toBe('#3b82f6');
    expect(getNodeColorByChangeType(null)).toBe('#6b7280');
  });

  // REQ-VIZ-002.2
  test('getNodeSize returns correct sizes for change types', () => {
    expect(getNodeSizeByChangeType('added')).toBe(15);
    expect(getNodeSizeByChangeType('removed')).toBe(15);
    expect(getNodeSizeByChangeType('modified')).toBe(15);
    expect(getNodeSizeByChangeType('affected')).toBe(15);
    expect(getNodeSizeByChangeType(null)).toBe(5);
  });

  // REQ-VIZ-002.3
  test('getNodeLabel formats correctly', () => {
    const node = { id: 'n1', name: 'handleAuth', nodeType: 'function', changeType: 'added' };
    expect(getNodeLabelFormatted(node)).toBe('handleAuth (function)');
  });
});
```

---

## REQ-VIZ-003: Node Click Interactions

### Problem Statement

Users need to click nodes to see detailed information about that entity.

### Specification

#### REQ-VIZ-003.1: Node Click Handler

```
WHEN user clicks on a graph node
THEN SHALL call onNodeClick callback with node data
  AND SHALL update selectedNode in diffVisualizationStore
  AND SHALL open EntityDetailPanel with node details
  AND SHALL apply selected visual styling to clicked node
```

#### REQ-VIZ-003.2: Camera Focus on Click

```
WHEN user clicks on a graph node
THEN SHALL animate camera to center on clicked node
  WITH duration 1000ms
  AND SHALL zoom camera to level 2
  AND animation SHALL use easing function for smooth transition
```

#### REQ-VIZ-003.3: Background Click Deselection

```
WHEN user clicks on graph background (not a node)
THEN SHALL call onBackgroundClick callback
  AND SHALL clear selectedNode in diffVisualizationStore
  AND SHALL close EntityDetailPanel
  AND SHALL NOT change camera position
```

### Verification Test Template

```typescript
// __tests__/components/DiffGraphCanvasView.interactions.test.tsx
describe('REQ-VIZ-003: Node Click Interactions', () => {
  // REQ-VIZ-003.1
  test('clicking node updates store and shows detail panel', async () => {
    const onNodeClick = jest.fn();
    const graphData = {
      nodes: [{ id: 'n1', name: 'TestFunc', nodeType: 'function', changeType: 'added' }],
      links: [],
    };

    render(
      <DiffGraphCanvasView
        graphData={graphData}
        onNodeClick={onNodeClick}
      />
    );

    // Simulate node click (would need ForceGraph3D mock)
    // fireEvent.click on node

    expect(onNodeClick).toHaveBeenCalledWith(expect.objectContaining({ id: 'n1' }));
    expect(useDiffVisualizationStore.getState().selectedNode).toEqual(
      expect.objectContaining({ id: 'n1' })
    );
  });

  // REQ-VIZ-003.3
  test('clicking background clears selection', async () => {
    const onBackgroundClick = jest.fn();
    useDiffVisualizationStore.setState({ selectedNode: { id: 'n1' } });

    render(
      <DiffGraphCanvasView
        graphData={{ nodes: [], links: [] }}
        onBackgroundClick={onBackgroundClick}
      />
    );

    // Simulate background click
    fireEvent.click(screen.getByTestId('diff-graph-canvas'));

    expect(onBackgroundClick).toHaveBeenCalled();
    expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
  });
});
```

---

## REQ-VIZ-004: Data Transformation

### Problem Statement

API response format differs from react-force-graph-3d expected format and must be transformed.

### Specification

#### REQ-VIZ-004.1: Transform API Response to ForceGraph Format

```
WHEN transformDiffToForcegraph receives API response
  WITH format:
    {
      nodes: [{ id, label, node_type, change_type, file_path, line_start }],
      edges: [{ source, target, edge_type }]
    }
THEN SHALL return ForceGraphData:
    {
      nodes: [{ id, name (from label), nodeType (from node_type),
                changeType (from change_type), filePath, lineStart }],
      links: [{ source, target, edgeType (from edge_type) }]
    }
  AND SHALL preserve all node IDs for link resolution
  AND transformation SHALL complete in < 10ms for 1000 nodes
```

#### REQ-VIZ-004.2: Handle Missing Optional Fields

```
WHEN API response node has null/undefined optional fields
  WITH file_path, line_start, change_type potentially missing
THEN SHALL set corresponding output fields to undefined
  AND SHALL NOT throw error
  AND SHALL NOT set to empty string (use undefined)
```

#### REQ-VIZ-004.3: Handle Empty Input

```
WHEN transformDiffToForcegraph receives null, undefined, or empty object
THEN SHALL return { nodes: [], links: [] }
  AND SHALL NOT throw error
```

### Verification Test Template

```typescript
// __tests__/utils/transformDiffToForcegraph.test.ts
describe('REQ-VIZ-004: Data Transformation', () => {
  // REQ-VIZ-004.1
  test('transforms API response to ForceGraph format', () => {
    const apiResponse = {
      nodes: [
        { id: 'n1', label: 'handleAuth', node_type: 'function', change_type: 'added', file_path: 'src/auth.ts', line_start: 10 },
        { id: 'n2', label: 'validate', node_type: 'function', change_type: null },
      ],
      edges: [
        { source: 'n1', target: 'n2', edge_type: 'Calls' },
      ],
    };

    const result = transformDiffToForcegraph(apiResponse);

    expect(result).toEqual({
      nodes: [
        { id: 'n1', name: 'handleAuth', nodeType: 'function', changeType: 'added', filePath: 'src/auth.ts', lineStart: 10 },
        { id: 'n2', name: 'validate', nodeType: 'function', changeType: null, filePath: undefined, lineStart: undefined },
      ],
      links: [
        { source: 'n1', target: 'n2', edgeType: 'Calls' },
      ],
    });
  });

  // REQ-VIZ-004.2
  test('handles missing optional fields gracefully', () => {
    const apiResponse = {
      nodes: [{ id: 'n1', label: 'test', node_type: 'fn' }], // missing optional fields
      edges: [],
    };

    const result = transformDiffToForcegraph(apiResponse);

    expect(result.nodes[0].changeType).toBeUndefined();
    expect(result.nodes[0].filePath).toBeUndefined();
    expect(result.nodes[0].lineStart).toBeUndefined();
  });

  // REQ-VIZ-004.3
  test('returns empty structure for null input', () => {
    expect(transformDiffToForcegraph(null)).toEqual({ nodes: [], links: [] });
    expect(transformDiffToForcegraph(undefined)).toEqual({ nodes: [], links: [] });
    expect(transformDiffToForcegraph({})).toEqual({ nodes: [], links: [] });
  });
});
```

---

## REQ-VIZ-005: Diff Summary Display

### Problem Statement

Users need a summary of changes to understand the overall impact of code modifications.

### Specification

#### REQ-VIZ-005.1: Display Diff Summary Stats

```
WHEN DiffSummaryStats receives summary data
THEN SHALL display:
  - Added entities count with green indicator (+N)
  - Removed entities count with red indicator (-N)
  - Modified entities count with amber indicator (~N)
  - Blast radius count with blue indicator
  AND SHALL format numbers with thousands separator for > 999
```

#### REQ-VIZ-005.2: Update on WebSocket Events

```
WHEN WebSocket receives 'diff_completed' event
  WITH summary: DiffSummaryData
THEN SHALL update diffVisualizationStore.summary
  AND SHALL animate counter changes
  AND SHALL complete update within 100ms
```

#### REQ-VIZ-005.3: Empty State

```
WHEN summary has all zero counts
THEN SHALL display "No changes detected"
  AND SHALL NOT show individual counters
```

### Verification Test Template

```typescript
// __tests__/components/DiffSummaryStats.test.tsx
describe('REQ-VIZ-005: Diff Summary Display', () => {
  // REQ-VIZ-005.1
  test('displays formatted summary statistics', () => {
    const summary = {
      added_entity_count: 1234,
      removed_entity_count: 5,
      modified_entity_count: 42,
      blast_radius_count: 89,
    };

    render(<DiffSummaryStats summary={summary} />);

    expect(screen.getByText('+1,234')).toHaveClass('text-green-500');
    expect(screen.getByText('-5')).toHaveClass('text-red-500');
    expect(screen.getByText('~42')).toHaveClass('text-amber-500');
    expect(screen.getByText('Blast radius: 89')).toBeInTheDocument();
  });

  // REQ-VIZ-005.3
  test('displays empty state when no changes', () => {
    const summary = {
      added_entity_count: 0,
      removed_entity_count: 0,
      modified_entity_count: 0,
      blast_radius_count: 0,
    };

    render(<DiffSummaryStats summary={summary} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();
    expect(screen.queryByText('+0')).not.toBeInTheDocument();
  });
});
```

---

# Section 3: WebSocket Integration Requirements

## REQ-WS-001: WebSocket Connection Hook

### Problem Statement

Components need to establish and manage WebSocket connections with automatic lifecycle handling.

### Specification

#### REQ-WS-001.1: Connection Establishment

```
WHEN useWebsocketDiffStream hook mounts
THEN SHALL create WebSocket connection to:
  - ws://localhost:7777/websocket-diff-stream (HTTP)
  - wss://localhost:7777/websocket-diff-stream (HTTPS)
  AND SHALL set connectionStatus to 'connecting'
  AND SHALL handle protocol detection based on window.location.protocol
```

#### REQ-WS-001.2: Connection Success

```
WHEN WebSocket 'open' event fires
THEN SHALL set connectionStatus to 'connected'
  AND SHALL re-subscribe to previous workspace if subscribedWorkspaceRef.current exists
  AND SHALL start heartbeat interval (30 seconds)
```

#### REQ-WS-001.3: Connection Failure

```
WHEN WebSocket 'error' event fires
THEN SHALL set connectionStatus to 'error'
  AND SHALL log error to console
  AND SHALL attempt reconnection with exponential backoff:
    - Initial delay: 1000ms
    - Max delay: 30000ms
    - Factor: 2
    - Max attempts: 5
```

#### REQ-WS-001.4: Connection Cleanup

```
WHEN useWebsocketDiffStream hook unmounts
THEN SHALL close WebSocket connection
  AND SHALL cancel any pending reconnection attempts
  AND SHALL clear heartbeat interval
  AND SHALL set connectionStatus to 'disconnected'
```

### Verification Test Template

```typescript
// __tests__/hooks/useWebsocketDiffStream.test.tsx
import { renderHook, act, waitFor } from '@testing-library/react';
import { useWebsocketDiffStream } from '@/hooks/useWebsocketDiffStream';
import WS from 'jest-websocket-mock';

describe('REQ-WS-001: WebSocket Connection Hook', () => {
  let mockServer: WS;

  beforeEach(() => {
    mockServer = new WS('ws://localhost:7777/websocket-diff-stream');
  });

  afterEach(() => {
    WS.clean();
  });

  // REQ-WS-001.1
  test('establishes connection on mount', async () => {
    const { result } = renderHook(() => useWebsocketDiffStream());

    expect(result.current.connectionStatus).toBe('connecting');
    await mockServer.connected;
    expect(result.current.connectionStatus).toBe('connected');
  });

  // REQ-WS-001.3
  test('handles connection error with reconnection', async () => {
    const { result } = renderHook(() => useWebsocketDiffStream());
    await mockServer.connected;

    mockServer.error();

    expect(result.current.connectionStatus).toBe('error');
    // Should attempt reconnection
  });

  // REQ-WS-001.4
  test('closes connection on unmount', async () => {
    const { result, unmount } = renderHook(() => useWebsocketDiffStream());
    await mockServer.connected;

    unmount();

    expect(result.current.connectionStatus).toBe('disconnected');
    await mockServer.closed;
  });
});
```

---

## REQ-WS-002: Subscription Management

### Problem Statement

Clients must be able to subscribe to specific workspace updates.

### Specification

#### REQ-WS-002.1: Subscribe to Workspace

```
WHEN subscribe(workspaceId) is called
  WITH connectionStatus === 'connected'
THEN SHALL send message:
  { "action": "subscribe", "workspace_id": "{workspaceId}" }
  AND SHALL store workspaceId in subscribedWorkspaceRef
  AND SHALL update lastDiffEvent when events arrive
```

#### REQ-WS-002.2: Subscribe When Disconnected

```
WHEN subscribe(workspaceId) is called
  WITH connectionStatus !== 'connected'
THEN SHALL store workspaceId in subscribedWorkspaceRef
  AND SHALL NOT send message immediately
  AND SHALL send subscription when connection established
```

#### REQ-WS-002.3: Unsubscribe from Workspace

```
WHEN unsubscribe() is called
THEN SHALL send message:
  { "action": "unsubscribe" }
  AND SHALL clear subscribedWorkspaceRef
  AND SHALL clear lastDiffEvent
```

### Verification Test Template

```typescript
// __tests__/hooks/useWebsocketDiffStream.subscription.test.tsx
describe('REQ-WS-002: Subscription Management', () => {
  // REQ-WS-002.1
  test('subscribe sends correct message when connected', async () => {
    const { result } = renderHook(() => useWebsocketDiffStream());
    await mockServer.connected;

    act(() => {
      result.current.subscribe('ws_123');
    });

    await expect(mockServer).toReceiveMessage(
      JSON.stringify({ action: 'subscribe', workspace_id: 'ws_123' })
    );
  });

  // REQ-WS-002.2
  test('subscribe queues when disconnected', async () => {
    // Don't connect the server yet
    const { result } = renderHook(() => useWebsocketDiffStream());

    act(() => {
      result.current.subscribe('ws_123');
    });

    // Now connect
    await mockServer.connected;

    // Should send queued subscription
    await expect(mockServer).toReceiveMessage(
      JSON.stringify({ action: 'subscribe', workspace_id: 'ws_123' })
    );
  });

  // REQ-WS-002.3
  test('unsubscribe clears state', async () => {
    const { result } = renderHook(() => useWebsocketDiffStream());
    await mockServer.connected;

    act(() => {
      result.current.subscribe('ws_123');
    });

    act(() => {
      result.current.unsubscribe();
    });

    await expect(mockServer).toReceiveMessage(
      JSON.stringify({ action: 'unsubscribe' })
    );
    expect(result.current.lastDiffEvent).toBeNull();
  });
});
```

---

## REQ-WS-003: Event Processing

### Problem Statement

WebSocket events must be parsed and dispatched to update application state.

### Specification

#### REQ-WS-003.1: Parse Incoming Messages

```
WHEN WebSocket receives message
THEN SHALL parse JSON
  AND SHALL validate event type
  AND SHALL update lastDiffEvent with parsed data
  AND SHALL dispatch to appropriate handler based on event type
```

#### REQ-WS-003.2: Handle Diff Lifecycle Events

```
WHEN receiving 'diff_started' event
THEN SHALL update diffVisualizationStore.isDiffInProgress = true
  AND SHALL display "Analyzing changes..." indicator

WHEN receiving 'diff_completed' event
THEN SHALL update diffVisualizationStore.isDiffInProgress = false
  AND SHALL update diffVisualizationStore.summary
  AND SHALL hide "Analyzing changes..." indicator
```

#### REQ-WS-003.3: Handle Entity Events

```
WHEN receiving 'entity_added' event
THEN SHALL add node to graphData.nodes
  WITH changeType = 'added'
  AND SHALL trigger graph re-render

WHEN receiving 'entity_removed' event
THEN SHALL mark node in graphData.nodes
  WITH changeType = 'removed'
  AND SHALL trigger graph re-render

WHEN receiving 'entity_modified' event
THEN SHALL update node in graphData.nodes
  WITH changeType = 'modified'
  AND SHALL trigger graph re-render
```

#### REQ-WS-003.4: Handle Error Events

```
WHEN receiving 'error' event
THEN SHALL display error toast with message
  AND SHALL log to console with code
  AND SHALL NOT disconnect (maintain connection)
```

### Verification Test Template

```typescript
// __tests__/hooks/useWebsocketDiffStream.events.test.tsx
describe('REQ-WS-003: Event Processing', () => {
  // REQ-WS-003.1
  test('parses and stores incoming events', async () => {
    const { result } = renderHook(() => useWebsocketDiffStream());
    await mockServer.connected;

    mockServer.send(JSON.stringify({
      event: 'subscribed',
      workspace_id: 'ws_123',
      workspace_name: 'Test',
      timestamp: '2026-01-23T00:00:00Z',
    }));

    await waitFor(() => {
      expect(result.current.lastDiffEvent).toEqual(
        expect.objectContaining({ event: 'subscribed' })
      );
    });
  });

  // REQ-WS-003.2
  test('updates diff progress state on lifecycle events', async () => {
    const { result } = renderHook(() => {
      const ws = useWebsocketDiffStream();
      const store = useDiffVisualizationStore();
      return { ws, store };
    });
    await mockServer.connected;

    mockServer.send(JSON.stringify({
      event: 'diff_started',
      workspace_id: 'ws_123',
      files_changed: 3,
      triggered_by: 'file_watcher',
      timestamp: '2026-01-23T00:00:00Z',
    }));

    await waitFor(() => {
      expect(result.current.store.isDiffInProgress).toBe(true);
    });

    mockServer.send(JSON.stringify({
      event: 'diff_completed',
      workspace_id: 'ws_123',
      summary: { added_entity_count: 5 },
      blast_radius_count: 10,
      duration_ms: 500,
      timestamp: '2026-01-23T00:00:01Z',
    }));

    await waitFor(() => {
      expect(result.current.store.isDiffInProgress).toBe(false);
      expect(result.current.store.summary.added_entity_count).toBe(5);
    });
  });
});
```

---

## REQ-WS-004: Connection Status Indicator

### Problem Statement

Users need visual feedback about WebSocket connection state.

### Specification

#### REQ-WS-004.1: Display Connection Status

```
WHEN ConnectionStatusIndicator renders
THEN SHALL display visual indicator based on connectionStatus:
  - 'connecting': Yellow dot + "Connecting..."
  - 'connected': Green dot + "Connected"
  - 'disconnected': Gray dot + "Disconnected"
  - 'error': Red dot + "Connection error"
  AND indicator SHALL be visible in bottom-right corner
```

#### REQ-WS-004.2: Animate Status Transitions

```
WHEN connectionStatus changes
THEN SHALL animate dot color transition (200ms ease)
  AND SHALL fade text change (150ms)
```

#### REQ-WS-004.3: Show Reconnection Progress

```
WHEN connectionStatus === 'error'
  AND reconnection is in progress
THEN SHALL display "Reconnecting... (attempt N/5)"
  AND SHALL update attempt counter on each retry
```

### Verification Test Template

```typescript
// __tests__/components/ConnectionStatusIndicator.test.tsx
describe('REQ-WS-004: Connection Status Indicator', () => {
  // REQ-WS-004.1
  test('displays correct status for each state', () => {
    const { rerender } = render(
      <ConnectionStatusIndicator connectionStatus="connecting" />
    );
    expect(screen.getByText('Connecting...')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-yellow-500');

    rerender(<ConnectionStatusIndicator connectionStatus="connected" />);
    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-green-500');

    rerender(<ConnectionStatusIndicator connectionStatus="disconnected" />);
    expect(screen.getByText('Disconnected')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-gray-500');

    rerender(<ConnectionStatusIndicator connectionStatus="error" />);
    expect(screen.getByText('Connection error')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-red-500');
  });

  // REQ-WS-004.3
  test('shows reconnection progress', () => {
    render(
      <ConnectionStatusIndicator
        connectionStatus="error"
        reconnectAttempt={2}
        maxReconnectAttempts={5}
      />
    );

    expect(screen.getByText('Reconnecting... (attempt 2/5)')).toBeInTheDocument();
  });
});
```

---

# Section 4: State Management Requirements

## REQ-STORE-001: Workspace Store

### Problem Statement

Global state for workspace list and selection must be accessible across components.

### Specification

#### REQ-STORE-001.1: Store Shape

```typescript
WHEN workspaceStore is created
THEN SHALL have shape:
  {
    workspaces: WorkspaceMetadata[];
    selectedWorkspaceId: string | null;
    isLoading: boolean;
    error: string | null;
    actions: {
      fetchWorkspaceListData: () => Promise<void>;
      selectWorkspaceById: (id: string) => void;
      toggleWorkspaceWatchState: (id: string, enabled: boolean) => Promise<void>;
      createWorkspaceFromPath: (path: string, name?: string) => Promise<void>;
      clearSelectedWorkspace: () => void;
    };
  }
```

#### REQ-STORE-001.2: Selector Hooks

```
WHEN component needs specific store data
THEN SHALL use selector hooks to prevent unnecessary re-renders:
  - useWorkspaceList() -> workspaces[]
  - useSelectedWorkspaceId() -> string | null
  - useWorkspaceLoading() -> boolean
  - useWorkspaceError() -> string | null
  - useWorkspaceActions() -> actions object
```

#### REQ-STORE-001.3: Optimistic Updates

```
WHEN action modifies state (toggle watch, create)
THEN SHALL apply optimistic update immediately
  AND SHALL revert if API call fails
  AND SHALL NOT block UI during API call
```

### Verification Test Template

```typescript
// __tests__/stores/workspaceStore.test.ts
describe('REQ-STORE-001: Workspace Store', () => {
  beforeEach(() => {
    useWorkspaceStore.setState({
      workspaces: [],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
  });

  // REQ-STORE-001.1
  test('store has correct initial shape', () => {
    const state = useWorkspaceStore.getState();

    expect(state).toEqual(expect.objectContaining({
      workspaces: [],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    }));
    expect(state.actions).toBeDefined();
    expect(typeof state.actions.fetchWorkspaceListData).toBe('function');
    expect(typeof state.actions.selectWorkspaceById).toBe('function');
  });

  // REQ-STORE-001.2
  test('selector hooks return specific slices', () => {
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      selectedWorkspaceId: 'ws_1',
      isLoading: true,
      error: 'test error',
    });

    const { result: list } = renderHook(() => useWorkspaceList());
    const { result: selected } = renderHook(() => useSelectedWorkspaceId());
    const { result: loading } = renderHook(() => useWorkspaceLoading());
    const { result: error } = renderHook(() => useWorkspaceError());

    expect(list.current).toEqual([mockWorkspace]);
    expect(selected.current).toBe('ws_1');
    expect(loading.current).toBe(true);
    expect(error.current).toBe('test error');
  });

  // REQ-STORE-001.3
  test('selectWorkspaceById updates immediately', () => {
    const { actions } = useWorkspaceStore.getState();

    actions.selectWorkspaceById('ws_123');

    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_123');
  });
});
```

---

## REQ-STORE-002: Diff Visualization Store

### Problem Statement

Graph data and selection state must be managed for 3D visualization.

### Specification

#### REQ-STORE-002.1: Store Shape

```typescript
WHEN diffVisualizationStore is created
THEN SHALL have shape:
  {
    graphData: ForceGraphData;
    selectedNode: GraphNode | null;
    summary: DiffSummaryData | null;
    isDiffInProgress: boolean;
    actions: {
      setGraphDataFromApi: (apiResponse: ApiDiffVisualization) => void;
      selectNodeById: (nodeId: string) => void;
      clearSelectedNode: () => void;
      updateSummaryData: (summary: DiffSummaryData) => void;
      setDiffInProgress: (inProgress: boolean) => void;
      applyEntityEvent: (event: WebSocketServerEvent) => void;
      clearAllGraphData: () => void;
    };
  }
```

#### REQ-STORE-002.2: Graph Data Transformation

```
WHEN setGraphDataFromApi is called
  WITH API response format
THEN SHALL transform to ForceGraphData format
  AND SHALL store in graphData
  AND SHALL NOT mutate original response
```

#### REQ-STORE-002.3: Incremental Event Updates

```
WHEN applyEntityEvent is called
  WITH entity_added/removed/modified event
THEN SHALL update graphData.nodes incrementally
  AND SHALL NOT replace entire array
  AND SHALL trigger minimal re-renders
```

### Verification Test Template

```typescript
// __tests__/stores/diffVisualizationStore.test.ts
describe('REQ-STORE-002: Diff Visualization Store', () => {
  beforeEach(() => {
    useDiffVisualizationStore.getState().actions.clearAllGraphData();
  });

  // REQ-STORE-002.1
  test('store has correct initial shape', () => {
    const state = useDiffVisualizationStore.getState();

    expect(state).toEqual(expect.objectContaining({
      graphData: { nodes: [], links: [] },
      selectedNode: null,
      summary: null,
      isDiffInProgress: false,
    }));
  });

  // REQ-STORE-002.2
  test('setGraphDataFromApi transforms API response', () => {
    const apiResponse = {
      nodes: [{ id: 'n1', label: 'Test', node_type: 'fn', change_type: 'added' }],
      edges: [{ source: 'n1', target: 'n2', edge_type: 'Calls' }],
    };

    useDiffVisualizationStore.getState().actions.setGraphDataFromApi(apiResponse);

    const { graphData } = useDiffVisualizationStore.getState();
    expect(graphData.nodes[0]).toEqual(expect.objectContaining({
      id: 'n1',
      name: 'Test',
      nodeType: 'fn',
      changeType: 'added',
    }));
    expect(graphData.links[0]).toEqual(expect.objectContaining({
      source: 'n1',
      target: 'n2',
      edgeType: 'Calls',
    }));
  });

  // REQ-STORE-002.3
  test('applyEntityEvent updates nodes incrementally', () => {
    // Setup initial state
    useDiffVisualizationStore.setState({
      graphData: {
        nodes: [{ id: 'n1', name: 'Existing', nodeType: 'fn', changeType: null }],
        links: [],
      },
    });

    const event = {
      event: 'entity_added',
      workspace_id: 'ws_1',
      entity_key: 'n2',
      entity_type: 'function',
      file_path: 'src/new.ts',
      timestamp: '2026-01-23T00:00:00Z',
    };

    useDiffVisualizationStore.getState().actions.applyEntityEvent(event);

    const { graphData } = useDiffVisualizationStore.getState();
    expect(graphData.nodes).toHaveLength(2);
    expect(graphData.nodes[1]).toEqual(expect.objectContaining({
      id: 'n2',
      changeType: 'added',
    }));
  });
});
```

---

# Section 5: E2E Test Requirements

## REQ-E2E-001: Workspace Management Flow

### Problem Statement

End-to-end tests must verify complete user workflows.

### Specification

#### REQ-E2E-001.1: Create and Select Workspace

```
GIVEN user opens application at http://localhost:7777
WHEN user clicks "Add Workspace" button
  AND enters path "/tmp/test-project" in path input
  AND enters "Test Project" in name input
  AND clicks "Create" button
THEN workspace "Test Project" SHALL appear in sidebar
  AND workspace SHALL be automatically selected
  AND graph canvas SHALL display workspace data
```

#### REQ-E2E-001.2: Toggle Watch Mode

```
GIVEN workspace "Test Project" exists and is selected
WHEN user clicks watch toggle button
THEN toggle SHALL show "Watching" state (green)
  AND connection status SHALL show "Connected"
  AND toast SHALL display "Watching: Test Project"
```

#### REQ-E2E-001.3: Receive Live Updates

```
GIVEN workspace is watching
WHEN file changes occur in watched directory
THEN diff_started notification SHALL appear
  AND graph SHALL update with new/removed/modified nodes
  AND diff_completed summary SHALL display updated counts
```

### Verification Test Template

```typescript
// e2e/workspace.spec.ts
import { test, expect } from '@playwright/test';

test.describe('REQ-E2E-001: Workspace Management Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:7777');
  });

  // REQ-E2E-001.1
  test('should create and select workspace', async ({ page }) => {
    // Click Add Workspace
    await page.getByTestId('create-workspace-button').click();

    // Fill form
    await page.getByTestId('workspace-path-input').fill('/tmp/test-project');
    await page.getByTestId('workspace-name-input').fill('Test Project');
    await page.getByTestId('confirm-create-button').click();

    // Verify workspace appears and is selected
    await expect(page.getByText('Test Project')).toBeVisible();
    await expect(page.getByTestId('workspace-item-selected')).toContainText('Test Project');
    await expect(page.getByTestId('diff-graph-canvas')).toBeVisible();
  });

  // REQ-E2E-001.2
  test('should toggle watch mode', async ({ page }) => {
    // Assuming workspace exists
    await page.getByTestId('watch-toggle-ws_test').click();

    await expect(page.getByTestId('watch-toggle-ws_test')).toHaveText('Watching');
    await expect(page.getByTestId('watch-toggle-ws_test')).toHaveClass(/bg-green/);
    await expect(page.getByTestId('connection-status-indicator')).toContainText('Connected');
  });
});
```

---

## REQ-E2E-002: Visualization Interaction Flow

### Problem Statement

E2E tests must verify 3D visualization interactions work correctly.

### Specification

#### REQ-E2E-002.1: Node Click Shows Details

```
GIVEN graph is rendered with nodes
WHEN user clicks on a node
THEN EntityDetailPanel SHALL open
  AND panel SHALL display node name
  AND panel SHALL display node type
  AND panel SHALL display file path if available
  AND clicked node SHALL have selection indicator
```

#### REQ-E2E-002.2: Background Click Clears Selection

```
GIVEN a node is selected
WHEN user clicks on graph background
THEN EntityDetailPanel SHALL close
  AND no node SHALL have selection indicator
```

#### REQ-E2E-002.3: Color Legend Visibility

```
GIVEN graph is rendered with changed nodes
THEN color legend SHALL be visible
  AND legend SHALL show:
    - Green: Added
    - Red: Removed
    - Amber: Modified
    - Blue: Affected
    - Gray: Unchanged
```

### Verification Test Template

```typescript
// e2e/visualization.spec.ts
import { test, expect } from '@playwright/test';

test.describe('REQ-E2E-002: Visualization Interaction Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:7777');
    // Setup: Create workspace and wait for graph
    await setupTestWorkspace(page);
  });

  // REQ-E2E-002.1
  test('should show entity details on node click', async ({ page }) => {
    // Click on a node (canvas coordinates)
    await page.getByTestId('diff-graph-canvas').click({ position: { x: 400, y: 300 } });

    // Verify detail panel opens
    await expect(page.getByTestId('entity-detail-panel')).toBeVisible();
    await expect(page.getByTestId('entity-name')).not.toBeEmpty();
  });

  // REQ-E2E-002.2
  test('should clear selection on background click', async ({ page }) => {
    // First select a node
    await page.getByTestId('diff-graph-canvas').click({ position: { x: 400, y: 300 } });
    await expect(page.getByTestId('entity-detail-panel')).toBeVisible();

    // Click background (corner of canvas)
    await page.getByTestId('diff-graph-canvas').click({ position: { x: 10, y: 10 } });

    // Panel should close
    await expect(page.getByTestId('entity-detail-panel')).not.toBeVisible();
  });

  // REQ-E2E-002.3
  test('should display color legend', async ({ page }) => {
    await expect(page.getByTestId('color-legend')).toBeVisible();
    await expect(page.getByTestId('legend-added')).toContainText('Added');
    await expect(page.getByTestId('legend-removed')).toContainText('Removed');
    await expect(page.getByTestId('legend-modified')).toContainText('Modified');
  });
});
```

---

## REQ-E2E-003: WebSocket Reconnection Flow

### Problem Statement

E2E tests must verify reconnection behavior after network disruption.

### Specification

#### REQ-E2E-003.1: Display Disconnection Status

```
GIVEN connection is established
WHEN WebSocket connection drops
THEN connection indicator SHALL show "Disconnected" (gray)
  AND "Reconnecting..." message SHALL appear
```

#### REQ-E2E-003.2: Automatic Reconnection

```
GIVEN connection was lost
WHEN reconnection succeeds
THEN connection indicator SHALL show "Connected" (green)
  AND previous subscription SHALL be restored
  AND graph data SHALL refresh
```

#### REQ-E2E-003.3: Max Reconnection Attempts

```
GIVEN connection was lost
WHEN 5 reconnection attempts fail
THEN connection indicator SHALL show "Connection failed"
  AND "Retry" button SHALL appear
  AND clicking retry SHALL restart reconnection process
```

### Verification Test Template

```typescript
// e2e/websocket.spec.ts
import { test, expect } from '@playwright/test';

test.describe('REQ-E2E-003: WebSocket Reconnection Flow', () => {
  // REQ-E2E-003.1
  test('should show disconnection status', async ({ page }) => {
    await page.goto('http://localhost:7777');
    await expect(page.getByTestId('connection-status-indicator')).toContainText('Connected');

    // Simulate disconnect (requires backend control or network interception)
    await page.evaluate(() => {
      // Force close WebSocket
      (window as any).__ws?.close();
    });

    await expect(page.getByTestId('connection-status-indicator')).toContainText('Disconnected');
  });

  // REQ-E2E-003.3
  test('should show retry button after max attempts', async ({ page }) => {
    // This test requires mocking or controlling server availability
    await page.goto('http://localhost:7777');

    // After connection failures
    await expect(page.getByTestId('reconnect-retry-button')).toBeVisible({ timeout: 30000 });

    // Click retry
    await page.getByTestId('reconnect-retry-button').click();
    await expect(page.getByTestId('connection-status-indicator')).toContainText('Connecting');
  });
});
```

---

## REQ-E2E-004: Error Handling Flows

### Problem Statement

E2E tests must verify error states are handled gracefully.

### Specification

#### REQ-E2E-004.1: Invalid Path Error

```
GIVEN create workspace dialog is open
WHEN user enters invalid path "/nonexistent/path"
  AND clicks "Create"
THEN error message SHALL display "Source path does not exist"
  AND dialog SHALL remain open
  AND path input SHALL have error styling
```

#### REQ-E2E-004.2: Workspace Already Exists Error

```
GIVEN workspace exists for path "/existing/project"
WHEN user tries to create workspace with same path
THEN error message SHALL display "Workspace already exists"
  AND existing_workspace_id SHALL be shown
  AND user MAY click link to select existing workspace
```

#### REQ-E2E-004.3: API Error Toast

```
GIVEN any API call fails with 500 error
THEN error toast SHALL appear
  AND toast SHALL display error message
  AND toast SHALL auto-dismiss after 5 seconds
  AND user MAY dismiss toast early by clicking X
```

### Verification Test Template

```typescript
// e2e/errors.spec.ts
import { test, expect } from '@playwright/test';

test.describe('REQ-E2E-004: Error Handling Flows', () => {
  // REQ-E2E-004.1
  test('should display error for invalid path', async ({ page }) => {
    await page.goto('http://localhost:7777');
    await page.getByTestId('create-workspace-button').click();
    await page.getByTestId('workspace-path-input').fill('/nonexistent/path');
    await page.getByTestId('confirm-create-button').click();

    await expect(page.getByText('Source path does not exist')).toBeVisible();
    await expect(page.getByRole('dialog')).toBeVisible(); // Still open
    await expect(page.getByTestId('workspace-path-input')).toHaveClass(/border-red/);
  });

  // REQ-E2E-004.3
  test('should show and auto-dismiss error toast', async ({ page }) => {
    // Mock API to return 500
    await page.route('**/workspace-list-all', (route) => {
      route.fulfill({ status: 500, body: JSON.stringify({ error: 'Server error' }) });
    });

    await page.goto('http://localhost:7777');

    // Toast should appear
    await expect(page.getByTestId('error-toast')).toBeVisible();
    await expect(page.getByTestId('error-toast')).toContainText('Server error');

    // Should auto-dismiss after 5 seconds
    await expect(page.getByTestId('error-toast')).not.toBeVisible({ timeout: 6000 });
  });
});
```

---

# Summary

## Requirements Count by Section

| Section | Requirement IDs | Test Count |
|---------|-----------------|------------|
| 1. Workspace Sidebar | REQ-SIDEBAR-001 to 004 | 12 tests |
| 2. 3D Visualization | REQ-VIZ-001 to 005 | 14 tests |
| 3. WebSocket Integration | REQ-WS-001 to 004 | 10 tests |
| 4. State Management | REQ-STORE-001 to 002 | 6 tests |
| 5. E2E Testing | REQ-E2E-001 to 004 | 10 tests |
| **Total** | **19 Requirements** | **52 Tests** |

## Acceptance Criteria Checklist

### Phase 2.3: React Core Frontend
- [ ] REQ-SIDEBAR-001: Workspace list displays correctly
- [ ] REQ-SIDEBAR-002: Workspace selection works
- [ ] REQ-SIDEBAR-003: Workspace creation works
- [ ] REQ-SIDEBAR-004: Watch toggle works
- [ ] REQ-STORE-001: Workspace store implemented
- [ ] REQ-STORE-002: Diff visualization store implemented
- [ ] REQ-WS-001: WebSocket hook establishes connection
- [ ] REQ-WS-002: Subscription management works
- [ ] REQ-WS-003: Event processing works
- [ ] REQ-WS-004: Connection status indicator works

### Phase 2.4: 3D Visualization
- [ ] REQ-VIZ-001: Graph renders correctly
- [ ] REQ-VIZ-002: Node styling by change type works
- [ ] REQ-VIZ-003: Node click interactions work
- [ ] REQ-VIZ-004: Data transformation works
- [ ] REQ-VIZ-005: Diff summary displays correctly

### E2E Validation
- [ ] REQ-E2E-001: Workspace management flow passes
- [ ] REQ-E2E-002: Visualization interaction flow passes
- [ ] REQ-E2E-003: WebSocket reconnection flow passes
- [ ] REQ-E2E-004: Error handling flows pass

## Performance Targets

| Metric | Target |
|--------|--------|
| Initial page load | < 3 seconds |
| Workspace list fetch | < 500ms |
| Graph render (< 500 nodes) | < 1000ms at 60fps |
| Graph render (> 1000 nodes) | < 2000ms at 30fps |
| WebSocket message processing | < 10ms |
| State update propagation | < 100ms |

---

*Specification created: 2026-01-23*
*Phase 2.1-2.2 baseline: Backend complete*
*Target: 52 testable requirements for React frontend*
