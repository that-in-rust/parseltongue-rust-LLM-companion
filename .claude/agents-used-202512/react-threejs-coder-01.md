# React + Three.js + TypeScript Coder Agent

> Idiomatic patterns for Parseltongue frontend development

---

## Tech Stack

| Layer | Technology | Version |
|-------|------------|---------|
| **Framework** | React | 18.x |
| **Language** | TypeScript | 5.x |
| **3D Visualization** | react-force-graph-3d | 1.24.x |
| **State Management** | zustand | 4.x |
| **Data Fetching** | @tanstack/react-query | 5.x |
| **Styling** | tailwindcss | 3.x |
| **Build Tool** | vite | 5.x |
| **E2E Testing** | Playwright | latest |

---

## Project Structure

```
frontend/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── playwright.config.ts
├── index.html
├── src/
│   ├── main.tsx                    # Entry point
│   ├── App.tsx                     # Root component
│   ├── stores/                     # Zustand stores
│   │   ├── workspaceStore.ts
│   │   └── diffVisualizationStore.ts
│   ├── hooks/                      # Custom React hooks
│   │   ├── useWorkspaceListData.ts
│   │   ├── useWebsocketDiffStream.ts
│   │   └── useForcegraphNodeRenderer.ts
│   ├── components/                 # React components
│   │   ├── Layout/
│   │   │   ├── AppLayoutContainer.tsx
│   │   │   ├── WorkspaceListSidebar.tsx
│   │   │   └── MainContentArea.tsx
│   │   ├── Visualization/
│   │   │   ├── DiffGraphCanvasView.tsx
│   │   │   └── ConnectionStatusIndicator.tsx
│   │   └── Details/
│   │       ├── EntityDetailPanel.tsx
│   │       └── DiffSummaryStats.tsx
│   ├── utils/                      # Helper functions
│   │   ├── transformDiffToForcegraph.ts
│   │   └── colorMappingHelpers.ts
│   └── types/                      # TypeScript types
│       ├── api.ts
│       └── visualization.ts
└── e2e/                            # Playwright tests
    ├── workspace.spec.ts
    └── visualization.spec.ts
```

---

## 4-Word Naming Convention (TypeScript Adaptation)

### Functions (camelCase, 4 words)
```typescript
// Good
transformDiffToForcegraph()
handleWorkspaceWatchToggle()
createWebsocketConnectionManager()
fetchWorkspaceListData()
updateSelectedWorkspaceState()

// Bad
transformDiff()           // Too short
handleToggle()            // Too short
createManager()           // Too short
```

### Components (PascalCase, 3-4 words)
```typescript
// Good
WorkspaceListSidebar
DiffGraphCanvasView
ConnectionStatusIndicator
EntityDetailPanel
MainContentArea

// Bad
Sidebar                   // Too short
Graph                     // Too short
```

### Hooks (use prefix + 3 words)
```typescript
// Good
useWorkspaceListData()
useWebsocketDiffStream()
useForcegraphNodeRenderer()
useSelectedEntityDetails()

// Bad
useWorkspaces()           // Too short
useWebsocket()            // Too short
```

### Stores (noun + Store or State)
```typescript
// Good
workspaceStore
diffVisualizationStore
connectionStateStore

// Bad
store                     // Too short
state                     // Too short
```

---

## Core Patterns

### 1. Zustand Store Pattern

```typescript
// stores/workspaceStore.ts
import { create } from 'zustand';

interface WorkspaceMetadata {
  workspace_identifier_value: string;
  workspace_display_name: string;
  source_directory_path_value: string;
  watch_enabled_flag_status: boolean;
  created_timestamp_utc_value: string;
}

interface WorkspaceState {
  // State
  workspaces: WorkspaceMetadata[];
  selectedWorkspaceId: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions (grouped under 'actions' for clarity)
  actions: {
    fetchWorkspaceListData: () => Promise<void>;
    selectWorkspaceById: (id: string) => void;
    toggleWorkspaceWatchState: (id: string, enabled: boolean) => Promise<void>;
    createWorkspaceFromPath: (path: string, displayName?: string) => Promise<void>;
  };
}

export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  workspaces: [],
  selectedWorkspaceId: null,
  isLoading: false,
  error: null,

  actions: {
    fetchWorkspaceListData: async () => {
      set({ isLoading: true, error: null });
      try {
        const response = await fetch('/workspace-list-all');
        const data = await response.json();
        set({ workspaces: data.workspaces, isLoading: false });
      } catch (error) {
        set({ error: String(error), isLoading: false });
      }
    },

    selectWorkspaceById: (id) => {
      set({ selectedWorkspaceId: id });
    },

    toggleWorkspaceWatchState: async (id, enabled) => {
      const response = await fetch('/workspace-watch-toggle', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          workspace_identifier_value: id,
          watch_enabled_flag_status: enabled,
        }),
      });
      if (response.ok) {
        await get().actions.fetchWorkspaceListData();
      }
    },

    createWorkspaceFromPath: async (path, displayName) => {
      const response = await fetch('/workspace-create-from-path', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          source_path_directory_value: path,
          workspace_display_name_option: displayName,
        }),
      });
      if (response.ok) {
        await get().actions.fetchWorkspaceListData();
      }
    },
  },
}));

// Selector hooks for specific pieces of state
export const useWorkspaceList = () => useWorkspaceStore((s) => s.workspaces);
export const useSelectedWorkspaceId = () => useWorkspaceStore((s) => s.selectedWorkspaceId);
export const useWorkspaceActions = () => useWorkspaceStore((s) => s.actions);
```

### 2. WebSocket Hook Pattern

```typescript
// hooks/useWebsocketDiffStream.ts
import { useState, useEffect, useCallback, useRef } from 'react';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface DiffEvent {
  event: string;
  workspace_id?: string;
  [key: string]: unknown;
}

interface UseWebsocketDiffStreamReturn {
  connectionStatus: ConnectionStatus;
  lastDiffEvent: DiffEvent | null;
  subscribe: (workspaceId: string) => void;
  unsubscribe: () => void;
}

export function useWebsocketDiffStream(): UseWebsocketDiffStreamReturn {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('disconnected');
  const [lastDiffEvent, setLastDiffEvent] = useState<DiffEvent | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const subscribedWorkspaceRef = useRef<string | null>(null);

  // Connect to WebSocket on mount
  useEffect(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const ws = new WebSocket(`${protocol}//${window.location.host}/websocket-diff-stream`);

    ws.onopen = () => {
      setConnectionStatus('connected');
      // Re-subscribe if we had a previous subscription
      if (subscribedWorkspaceRef.current) {
        ws.send(JSON.stringify({
          action: 'subscribe',
          workspace_id: subscribedWorkspaceRef.current,
        }));
      }
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        setLastDiffEvent(data);
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e);
      }
    };

    ws.onclose = () => {
      setConnectionStatus('disconnected');
    };

    ws.onerror = () => {
      setConnectionStatus('error');
    };

    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, []);

  const subscribe = useCallback((workspaceId: string) => {
    subscribedWorkspaceRef.current = workspaceId;
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        action: 'subscribe',
        workspace_id: workspaceId,
      }));
    }
  }, []);

  const unsubscribe = useCallback(() => {
    subscribedWorkspaceRef.current = null;
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ action: 'unsubscribe' }));
    }
  }, []);

  return { connectionStatus, lastDiffEvent, subscribe, unsubscribe };
}
```

### 3. react-force-graph-3d Pattern

```typescript
// components/Visualization/DiffGraphCanvasView.tsx
import { useRef, useCallback } from 'react';
import ForceGraph3D, { ForceGraphMethods } from 'react-force-graph-3d';

// Color mapping for change types
const CHANGE_TYPE_COLORS = {
  added: '#22c55e',     // green-500
  removed: '#ef4444',   // red-500
  modified: '#f59e0b',  // amber-500
  affected: '#3b82f6',  // blue-500
  unchanged: '#6b7280', // gray-500
} as const;

interface GraphNode {
  id: string;
  name: string;
  nodeType: string;
  changeType: keyof typeof CHANGE_TYPE_COLORS | null;
  filePath?: string;
  lineStart?: number;
}

interface GraphLink {
  source: string;
  target: string;
  edgeType: string;
}

interface ForceGraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

interface DiffGraphCanvasViewProps {
  graphData: ForceGraphData;
  onNodeClick?: (node: GraphNode) => void;
  onBackgroundClick?: () => void;
}

export function DiffGraphCanvasView({
  graphData,
  onNodeClick,
  onBackgroundClick,
}: DiffGraphCanvasViewProps) {
  const fgRef = useRef<ForceGraphMethods>();

  const handleNodeClick = useCallback((node: GraphNode) => {
    // Focus on clicked node
    if (fgRef.current) {
      fgRef.current.centerAt(node.x, node.y, 1000);
      fgRef.current.zoom(2, 1000);
    }
    onNodeClick?.(node);
  }, [onNodeClick]);

  const getNodeColor = useCallback((node: GraphNode) => {
    return CHANGE_TYPE_COLORS[node.changeType ?? 'unchanged'];
  }, []);

  const getNodeSize = useCallback((node: GraphNode) => {
    // Changed nodes are larger
    return node.changeType ? 15 : 5;
  }, []);

  return (
    <ForceGraph3D
      ref={fgRef}
      graphData={graphData}
      nodeColor={getNodeColor}
      nodeVal={getNodeSize}
      nodeLabel={(node: GraphNode) => `${node.name} (${node.nodeType})`}
      linkColor={() => '#4b5563'}
      linkWidth={1}
      linkOpacity={0.6}
      onNodeClick={handleNodeClick}
      onBackgroundClick={onBackgroundClick}
      controlType="orbit"
      backgroundColor="#111827" // gray-900
    />
  );
}
```

### 4. Data Transformation Pattern

```typescript
// utils/transformDiffToForcegraph.ts

interface ApiDiffVisualization {
  nodes: Array<{
    id: string;
    label: string;
    node_type: string;
    change_type: 'added' | 'removed' | 'modified' | 'affected' | null;
    file_path?: string;
    line_start?: number;
  }>;
  edges: Array<{
    source: string;
    target: string;
    edge_type: string;
  }>;
}

interface ForceGraphData {
  nodes: Array<{
    id: string;
    name: string;
    nodeType: string;
    changeType: string | null;
    filePath?: string;
    lineStart?: number;
  }>;
  links: Array<{
    source: string;
    target: string;
    edgeType: string;
  }>;
}

export function transformDiffToForcegraph(
  apiResponse: ApiDiffVisualization
): ForceGraphData {
  return {
    nodes: apiResponse.nodes.map((node) => ({
      id: node.id,
      name: node.label,
      nodeType: node.node_type,
      changeType: node.change_type,
      filePath: node.file_path,
      lineStart: node.line_start,
    })),
    links: apiResponse.edges.map((edge) => ({
      source: edge.source,
      target: edge.target,
      edgeType: edge.edge_type,
    })),
  };
}
```

---

## TypeScript Types (Match Rust Backend)

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
  workspace_identifier_value: string;
  watch_enabled_flag_status: boolean;
}

export interface WorkspaceListResponse {
  success: boolean;
  endpoint: string;
  workspaces: WorkspaceMetadata[];
  total_workspace_count_value: number;
  token_estimate: number;
}

// WebSocket event types (match websocket_streaming_module/message_types.rs)
export type WebSocketEvent =
  | { event: 'subscribed'; workspace_id: string; timestamp: string }
  | { event: 'unsubscribed'; timestamp: string }
  | { event: 'pong'; timestamp: string }
  | { event: 'diff_started'; workspace_id: string; files_changed_count: number }
  | { event: 'entity_added'; entity_key: string; entity_name: string; file_path: string }
  | { event: 'entity_removed'; entity_key: string; entity_name: string }
  | { event: 'entity_modified'; entity_key: string; entity_name: string; modification_type: string }
  | { event: 'edge_added'; source_key: string; target_key: string; edge_type: string }
  | { event: 'edge_removed'; source_key: string; target_key: string; edge_type: string }
  | { event: 'diff_completed'; summary: DiffSummary; duration_ms: number }
  | { event: 'error'; code: string; message: string };

export interface DiffSummary {
  entities_added_count: number;
  entities_removed_count: number;
  entities_modified_count: number;
  edges_added_count: number;
  edges_removed_count: number;
  blast_radius_count: number;
}
```

---

## Testing Patterns (Playwright)

```typescript
// e2e/workspace.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Workspace Management', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:7777');
  });

  test('should display workspace list', async ({ page }) => {
    await expect(page.getByTestId('workspace-list-sidebar')).toBeVisible();
  });

  test('should create new workspace', async ({ page }) => {
    await page.getByTestId('create-workspace-button').click();
    await page.getByTestId('workspace-path-input').fill('/tmp/test-project');
    await page.getByTestId('confirm-create-button').click();

    await expect(page.getByText('test-project')).toBeVisible();
  });

  test('should toggle watch mode', async ({ page }) => {
    // Assuming a workspace exists
    await page.getByTestId('watch-toggle-ws_test').click();
    await expect(page.getByTestId('connection-status-indicator')).toHaveText('Connected');
  });

  test('should show 3D visualization', async ({ page }) => {
    await expect(page.getByTestId('diff-graph-canvas')).toBeVisible();
    // Check that canvas has rendered
    await expect(page.locator('canvas')).toBeVisible();
  });
});

// e2e/visualization.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Diff Visualization', () => {
  test('should render nodes with correct colors', async ({ page }) => {
    await page.goto('http://localhost:7777');
    // This requires mocking the API response or having test data
    // The canvas rendering makes direct assertions difficult
    await expect(page.getByTestId('diff-graph-canvas')).toBeVisible();
  });

  test('should show entity details on click', async ({ page }) => {
    await page.goto('http://localhost:7777');
    // Click on canvas (would need coordinates or mock)
    await page.getByTestId('diff-graph-canvas').click({ position: { x: 400, y: 300 } });
    // Check details panel appears
    await expect(page.getByTestId('entity-detail-panel')).toBeVisible();
  });
});
```

---

## Component Patterns

### Layout Component

```typescript
// components/Layout/AppLayoutContainer.tsx
import { WorkspaceListSidebar } from './WorkspaceListSidebar';
import { MainContentArea } from './MainContentArea';

export function AppLayoutContainer() {
  return (
    <div className="flex h-screen bg-gray-900 text-white">
      <WorkspaceListSidebar />
      <MainContentArea />
    </div>
  );
}
```

### Sidebar Component

```typescript
// components/Layout/WorkspaceListSidebar.tsx
import { useWorkspaceList, useWorkspaceActions, useSelectedWorkspaceId } from '../../stores/workspaceStore';
import { useEffect } from 'react';

export function WorkspaceListSidebar() {
  const workspaces = useWorkspaceList();
  const selectedId = useSelectedWorkspaceId();
  const { fetchWorkspaceListData, selectWorkspaceById, toggleWorkspaceWatchState } = useWorkspaceActions();

  useEffect(() => {
    fetchWorkspaceListData();
  }, [fetchWorkspaceListData]);

  return (
    <aside
      className="w-64 bg-gray-800 p-4 border-r border-gray-700"
      data-testid="workspace-list-sidebar"
    >
      <h2 className="text-lg font-semibold mb-4">Workspaces</h2>

      <ul className="space-y-2">
        {workspaces.map((ws) => (
          <li
            key={ws.workspace_identifier_value}
            className={`p-2 rounded cursor-pointer ${
              selectedId === ws.workspace_identifier_value
                ? 'bg-blue-600'
                : 'hover:bg-gray-700'
            }`}
            onClick={() => selectWorkspaceById(ws.workspace_identifier_value)}
          >
            <div className="flex justify-between items-center">
              <span>{ws.workspace_display_name}</span>
              <button
                data-testid={`watch-toggle-${ws.workspace_identifier_value}`}
                className={`px-2 py-1 rounded text-xs ${
                  ws.watch_enabled_flag_status
                    ? 'bg-green-600'
                    : 'bg-gray-600'
                }`}
                onClick={(e) => {
                  e.stopPropagation();
                  toggleWorkspaceWatchState(
                    ws.workspace_identifier_value,
                    !ws.watch_enabled_flag_status
                  );
                }}
              >
                {ws.watch_enabled_flag_status ? 'Watching' : 'Watch'}
              </button>
            </div>
          </li>
        ))}
      </ul>

      <button
        data-testid="create-workspace-button"
        className="mt-4 w-full py-2 bg-blue-600 hover:bg-blue-700 rounded"
      >
        + Add Workspace
      </button>
    </aside>
  );
}
```

---

## Error Handling

```typescript
// Always handle API errors gracefully
try {
  const response = await fetch('/workspace-list-all');
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }
  const data = await response.json();
  // Process data
} catch (error) {
  console.error('Failed to fetch workspaces:', error);
  // Set error state for UI
}

// WebSocket reconnection
const RECONNECT_DELAY_MS = 3000;
const MAX_RECONNECT_ATTEMPTS = 5;

// In useWebsocketDiffStream hook, implement exponential backoff
```

---

## Performance Tips

1. **Memoize callbacks** passed to ForceGraph3D
2. **Use selectors** with zustand to prevent unnecessary re-renders
3. **Debounce** rapid WebSocket events before updating state
4. **Virtualize** workspace list if it grows large
5. **Use React.lazy** for code splitting the 3D visualization

---

## Summary

This agent reference provides idiomatic patterns for:
- 4-word naming convention adapted for TypeScript/React
- Zustand state management with actions pattern
- WebSocket hook with reconnection logic
- react-force-graph-3d integration
- Data transformation from Rust API to ForceGraph format
- Playwright E2E testing patterns
- TypeScript types matching Rust backend

*Follow these patterns when implementing Phase 2.3-2.4 frontend.*
