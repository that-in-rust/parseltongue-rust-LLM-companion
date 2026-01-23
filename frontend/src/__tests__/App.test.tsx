/**
 * App.tsx Integration Tests
 *
 * REQ-APP-INTEGRATION: App.tsx Root Component Integration
 * Executable specification tests for layout composition, component wiring,
 * WebSocket integration, and user flows.
 *
 * These tests are in RED phase (test.skip) - implementation required.
 *
 * Test IDs expected:
 * - app-container
 * - app-header
 * - app-sidebar
 * - app-main-canvas
 * - sidebar-toggle-button
 */

import { describe, test, expect, beforeEach, vi, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App } from '../App';
import { useWorkspaceStore } from '@/stores/workspaceStore';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';
import type { WorkspaceMetadata, GraphNode, ForceGraphData } from '@/types/api';

// =============================================================================
// Mock Setup
// =============================================================================

// Mock subscribe/unsubscribe functions we can spy on
const mockSubscribe = vi.fn();
const mockUnsubscribe = vi.fn();

vi.mock('@/hooks/useWebsocketDiffStream', () => ({
  useWebsocketDiffStream: () => ({
    connectionStatus: 'connected',
    lastDiffEvent: null,
    reconnectAttempt: 0,
    maxReconnectAttempts: 5,
    subscribe: mockSubscribe,
    unsubscribe: mockUnsubscribe,
  }),
}));

// =============================================================================
// Test Fixtures
// =============================================================================

const mockWorkspace: WorkspaceMetadata = {
  workspace_identifier_value: 'ws_001',
  workspace_display_name: 'Test Workspace',
  source_directory_path_value: '/path/to/project',
  base_database_path_value: '/path/to/base.db',
  live_database_path_value: '/path/to/live.db',
  watch_enabled_flag_status: true,
  created_timestamp_utc_value: '2026-01-23T00:00:00Z',
  last_indexed_timestamp_option: null,
};

const mockWorkspaceWatchDisabled: WorkspaceMetadata = {
  ...mockWorkspace,
  workspace_identifier_value: 'ws_002',
  workspace_display_name: 'Unwatched Workspace',
  watch_enabled_flag_status: false,
};

const mockGraphNode: GraphNode = {
  id: 'rust:fn:handle_request',
  name: 'handle_request',
  nodeType: 'function',
  changeType: 'modified',
  filePath: '/src/handler.rs',
  lineStart: 10,
  lineEnd: 50,
};

const mockGraphData: ForceGraphData = {
  nodes: [mockGraphNode],
  links: [],
};

// =============================================================================
// Test Setup / Teardown
// =============================================================================

const getInitialWorkspaceState = () => ({
  workspaces: [] as WorkspaceMetadata[],
  selectedWorkspaceId: null as string | null,
  isLoading: false,
  error: null as string | null,
});

const getInitialDiffVisualizationState = () => ({
  graphData: { nodes: [], links: [] } as ForceGraphData,
  selectedNode: null as GraphNode | null,
  summary: null,
  isDiffInProgress: false,
});

beforeEach(() => {
  vi.clearAllMocks();
  useWorkspaceStore.setState(getInitialWorkspaceState());
  useDiffVisualizationStore.setState(getInitialDiffVisualizationState());
});

afterEach(() => {
  vi.resetAllMocks();
});

// =============================================================================
// REQ-APP-001: Layout Composition
// =============================================================================

describe('REQ-APP-001: Layout Composition', () => {
  /**
   * REQ-APP-001.1: Renders header with app title
   *
   * WHEN App component renders
   * THEN SHALL render a header bar with data-testid="app-header"
   *   AND SHALL contain app title or branding
   */
  test('renders header with app title', () => {
    // GIVEN
    render(<App />);

    // WHEN (rendered)

    // THEN
    const header = screen.getByTestId('app-header');
    expect(header).toBeInTheDocument();
    expect(header).toBeVisible();
  });

  /**
   * REQ-APP-001.2: Renders sidebar with WorkspaceListSidebar
   *
   * WHEN App component renders
   * THEN SHALL render sidebar container with data-testid="app-sidebar"
   *   AND sidebar SHALL contain WorkspaceListSidebar component
   */
  test('renders sidebar with WorkspaceListSidebar', () => {
    // GIVEN
    render(<App />);

    // WHEN (rendered)

    // THEN
    const sidebar = screen.getByTestId('app-sidebar');
    expect(sidebar).toBeInTheDocument();
    expect(screen.getByTestId('workspace-list-sidebar')).toBeInTheDocument();
  });

  /**
   * REQ-APP-001.3: Renders main canvas area
   *
   * WHEN App component renders
   * THEN SHALL render main content area with data-testid="app-main-canvas"
   *   AND SHALL contain DiffGraphCanvasView component
   */
  test('renders main canvas area', () => {
    // GIVEN
    render(<App />);

    // WHEN (rendered)

    // THEN
    const mainCanvas = screen.getByTestId('app-main-canvas');
    expect(mainCanvas).toBeInTheDocument();
    expect(mainCanvas).toBeVisible();
  });

  /**
   * REQ-APP-001.4: Responsive toggle button on mobile
   *
   * WHEN App component renders with viewport width < 768px
   * THEN SHALL render sidebar toggle button with data-testid="sidebar-toggle-button"
   *   AND button SHALL be visible
   */
  test('renders responsive toggle button on mobile', () => {
    // GIVEN - simulate mobile viewport
    Object.defineProperty(window, 'innerWidth', { value: 375, writable: true });
    window.dispatchEvent(new Event('resize'));

    // WHEN
    render(<App />);

    // THEN
    const toggleButton = screen.getByTestId('sidebar-toggle-button');
    expect(toggleButton).toBeInTheDocument();
    expect(toggleButton).toBeVisible();
  });

  /**
   * REQ-APP-001.5: Grid layout on desktop
   *
   * WHEN App component renders with viewport width >= 768px
   * THEN SHALL apply grid or flex layout
   *   AND app container SHALL have data-testid="app-container"
   *   AND container SHALL use full viewport height (h-screen)
   *   AND container SHALL have dark background (bg-gray-900)
   */
  test('uses grid layout on desktop', () => {
    // GIVEN - simulate desktop viewport
    Object.defineProperty(window, 'innerWidth', { value: 1024, writable: true });
    window.dispatchEvent(new Event('resize'));

    // WHEN
    render(<App />);

    // THEN
    const container = screen.getByTestId('app-container');
    expect(container).toBeInTheDocument();
    expect(container).toHaveClass('h-screen');
    expect(container).toHaveClass('bg-gray-900');
  });
});

// =============================================================================
// REQ-APP-002: Component Wiring
// =============================================================================

describe('REQ-APP-002: Component Wiring', () => {
  /**
   * REQ-APP-002.1: WorkspaceListSidebar is connected to workspaceStore
   *
   * WHEN App renders with workspaces in workspaceStore
   * THEN WorkspaceListSidebar SHALL display workspace list from store
   *   AND selecting a workspace SHALL update store
   */
  test('WorkspaceListSidebar is connected to workspaceStore', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      isLoading: false,
    });

    // WHEN
    render(<App />);

    // THEN
    await waitFor(() => {
      expect(screen.getByText('Test Workspace')).toBeInTheDocument();
    });

    // AND WHEN workspace is clicked
    fireEvent.click(screen.getByTestId(`workspace-item-${mockWorkspace.workspace_identifier_value}`));

    // THEN store is updated
    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe(mockWorkspace.workspace_identifier_value);
  });

  /**
   * REQ-APP-002.2: DiffGraphCanvasView receives graphData from diffVisualizationStore
   *
   * WHEN App renders with graphData in diffVisualizationStore
   * THEN DiffGraphCanvasView SHALL receive and render graph data
   */
  test('DiffGraphCanvasView receives graphData from diffVisualizationStore', async () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
    });

    // WHEN
    render(<App />);

    // THEN - mock ForceGraph3D renders with nodes count
    await waitFor(() => {
      const mockGraph = screen.getByTestId('force-graph-3d-mock');
      expect(mockGraph).toHaveAttribute('data-nodes', '1');
    });
  });

  /**
   * REQ-APP-002.3: EntityDetailPanel shows when selectedNode exists
   *
   * WHEN diffVisualizationStore has selectedNode set
   * THEN EntityDetailPanel SHALL be visible
   *   AND SHALL display selected node information
   */
  test('EntityDetailPanel shows when selectedNode exists', async () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: mockGraphNode,
    });

    // WHEN
    render(<App />);

    // THEN
    await waitFor(() => {
      const panel = screen.getByTestId('entity-detail-panel');
      expect(panel).toBeVisible();
      expect(screen.getByTestId('entity-name')).toHaveTextContent('handle_request');
    });
  });

  /**
   * REQ-APP-002.4: EntityDetailPanel hides when selectedNode is null
   *
   * WHEN diffVisualizationStore has selectedNode as null
   * THEN EntityDetailPanel SHALL be hidden
   */
  test('EntityDetailPanel hides when selectedNode is null', () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: null,
    });

    // WHEN
    render(<App />);

    // THEN
    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).not.toBeVisible();
  });

  /**
   * REQ-APP-002.5: DiffSummaryStats displays in header
   *
   * WHEN App renders
   * THEN DiffSummaryStats component SHALL be present in header
   *   AND SHALL receive summary data from store
   */
  test('DiffSummaryStats displays in header', () => {
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
    });

    // WHEN
    render(<App />);

    // THEN
    const header = screen.getByTestId('app-header');
    expect(header).toContainElement(screen.getByTestId('diff-summary-stats'));
  });

  /**
   * REQ-APP-002.6: ConnectionStatusIndicator shows connection state
   *
   * WHEN App renders with WebSocket connected
   * THEN ConnectionStatusIndicator SHALL display connected state
   */
  test('ConnectionStatusIndicator shows connection state', () => {
    // GIVEN (mock returns 'connected')

    // WHEN
    render(<App />);

    // THEN
    const indicator = screen.getByTestId('connection-status-indicator');
    expect(indicator).toBeInTheDocument();
    expect(indicator).toHaveTextContent(/connected/i);
  });
});

// =============================================================================
// REQ-APP-003: WebSocket Integration
// =============================================================================

describe('REQ-APP-003: WebSocket Integration', () => {
  /**
   * REQ-APP-003.1: Connects WebSocket when watch-enabled workspace selected
   *
   * WHEN user selects a workspace with watch_enabled_flag_status = true
   * THEN App SHALL call subscribe(workspaceId) within 100ms
   */
  test('connects WebSocket when watch-enabled workspace selected', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      selectedWorkspaceId: null,
    });
    render(<App />);

    // WHEN
    act(() => {
      useWorkspaceStore.getState().actions.selectWorkspaceById(mockWorkspace.workspace_identifier_value);
    });

    // THEN
    await waitFor(() => {
      expect(mockSubscribe).toHaveBeenCalledWith(mockWorkspace.workspace_identifier_value);
    }, { timeout: 200 });
  });

  /**
   * REQ-APP-003.2: Does not connect when watch-disabled workspace selected
   *
   * WHEN user selects a workspace with watch_enabled_flag_status = false
   * THEN App SHALL NOT call subscribe
   */
  test('does not connect WebSocket when watch-disabled workspace selected', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [mockWorkspaceWatchDisabled],
      selectedWorkspaceId: null,
    });
    render(<App />);

    // WHEN
    act(() => {
      useWorkspaceStore.getState().actions.selectWorkspaceById(mockWorkspaceWatchDisabled.workspace_identifier_value);
    });

    // THEN
    await waitFor(() => {
      expect(mockSubscribe).not.toHaveBeenCalled();
    }, { timeout: 200 });
  });

  /**
   * REQ-APP-003.3: Disconnects WebSocket when workspace deselected
   *
   * WHEN user clears workspace selection
   * THEN App SHALL call unsubscribe()
   */
  test('disconnects WebSocket when workspace deselected', async () => {
    // GIVEN - workspace is already selected
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      selectedWorkspaceId: mockWorkspace.workspace_identifier_value,
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

  /**
   * REQ-APP-003.4: Updates store on diff events
   *
   * WHEN WebSocket receives diff event (via lastDiffEvent)
   * THEN diffVisualizationStore SHALL be updated accordingly
   */
  test('updates store on diff events', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      selectedWorkspaceId: mockWorkspace.workspace_identifier_value,
    });
    render(<App />);

    // WHEN - simulate diff event via applyEntityEvent
    act(() => {
      useDiffVisualizationStore.getState().actions.applyEntityEvent({
        event: 'entity_added',
        workspace_id: mockWorkspace.workspace_identifier_value,
        entity_key: 'rust:fn:new_function',
        entity_type: 'function',
        file_path: '/src/new.rs',
        line_range: { start: 1, end: 10 },
        timestamp: '2026-01-23T00:00:00Z',
      });
    });

    // THEN
    await waitFor(() => {
      const { graphData } = useDiffVisualizationStore.getState();
      expect(graphData.nodes.some((n) => n.id === 'rust:fn:new_function')).toBe(true);
    });
  });
});

// =============================================================================
// REQ-APP-004: User Flow
// =============================================================================

describe('REQ-APP-004: User Flow', () => {
  const user = userEvent.setup();

  /**
   * REQ-APP-004.1: Initial state shows empty message
   *
   * WHEN App renders with no workspace selected and no graph data
   * THEN SHALL display empty state message
   *   AND message SHALL include "No graph data available" or similar
   */
  test('initial state shows empty message', () => {
    // GIVEN (default empty state)

    // WHEN
    render(<App />);

    // THEN
    expect(screen.getByText(/no graph data available/i)).toBeInTheDocument();
  });

  /**
   * REQ-APP-004.2: Selecting workspace triggers data loading
   *
   * WHEN user selects a workspace
   * THEN App SHALL trigger WebSocket subscription
   *   AND graph data loading SHALL begin
   */
  test('selecting workspace triggers data loading', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
      isLoading: false,
    });
    render(<App />);

    // WHEN
    await user.click(screen.getByTestId(`workspace-item-${mockWorkspace.workspace_identifier_value}`));

    // THEN
    await waitFor(() => {
      expect(mockSubscribe).toHaveBeenCalledWith(mockWorkspace.workspace_identifier_value);
    });
  });

  /**
   * REQ-APP-004.3: Node click opens detail panel
   *
   * WHEN user clicks a node in the graph
   * THEN EntityDetailPanel SHALL become visible
   *   AND SHALL display clicked node details
   */
  test('node click opens detail panel', async () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: null,
    });
    render(<App />);

    // WHEN - simulate node click via store action
    act(() => {
      useDiffVisualizationStore.getState().actions.selectNodeById(mockGraphNode.id);
    });

    // THEN
    await waitFor(() => {
      const panel = screen.getByTestId('entity-detail-panel');
      expect(panel).toBeVisible();
      expect(screen.getByTestId('entity-name')).toHaveTextContent('handle_request');
    });
  });

  /**
   * REQ-APP-004.4: Escape key closes detail panel
   *
   * WHEN EntityDetailPanel is visible AND user presses Escape key
   * THEN EntityDetailPanel SHALL close
   *   AND selectedNode SHALL be cleared
   */
  test('Escape key closes detail panel', async () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: mockGraphNode,
    });
    render(<App />);

    // Verify panel is visible
    expect(screen.getByTestId('entity-detail-panel')).toBeVisible();

    // WHEN
    fireEvent.keyDown(document, { key: 'Escape' });

    // THEN
    await waitFor(() => {
      expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
      expect(screen.getByTestId('entity-detail-panel')).not.toBeVisible();
    });
  });

  /**
   * REQ-APP-004.5: Loading state during diff analysis
   *
   * WHEN isDiffInProgress is true in store
   * THEN DiffSummaryStats SHALL show loading indicator
   */
  test('shows loading state during diff analysis', () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      isDiffInProgress: true,
    });

    // WHEN
    render(<App />);

    // THEN
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.getByText(/analyzing changes/i)).toBeInTheDocument();
  });

  /**
   * REQ-APP-004.6: Sidebar toggle on mobile
   *
   * WHEN user clicks sidebar toggle button on mobile
   * THEN sidebar SHALL become visible
   *
   * WHEN user presses Escape with sidebar visible on mobile
   * THEN sidebar SHALL close
   */
  test('sidebar toggle behavior on mobile', async () => {
    // GIVEN - mobile viewport
    Object.defineProperty(window, 'innerWidth', { value: 375, writable: true });
    window.dispatchEvent(new Event('resize'));

    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
    });

    render(<App />);

    // Verify sidebar is hidden by default on mobile
    expect(screen.getByTestId('workspace-list-sidebar')).not.toBeVisible();

    // WHEN toggle clicked
    await user.click(screen.getByTestId('sidebar-toggle-button'));

    // THEN sidebar is visible
    expect(screen.getByTestId('workspace-list-sidebar')).toBeVisible();

    // WHEN Escape pressed
    fireEvent.keyDown(document, { key: 'Escape' });

    // THEN sidebar is hidden
    await waitFor(() => {
      expect(screen.getByTestId('workspace-list-sidebar')).not.toBeVisible();
    });
  });
});

// =============================================================================
// Error Handling
// =============================================================================

describe('REQ-APP-ERROR: Error Handling', () => {
  /**
   * REQ-APP-ERROR.1: Workspace fetch error displays retry button
   *
   * WHEN workspace fetch fails
   * THEN App SHALL display error message
   *   AND SHALL provide retry button
   */
  test('displays error state when workspace fetch fails', async () => {
    // GIVEN
    useWorkspaceStore.setState({
      workspaces: [],
      error: 'Failed to load workspaces',
      isLoading: false,
    });

    // WHEN
    render(<App />);

    // THEN
    expect(screen.getByText(/failed to load workspaces/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
  });

  /**
   * REQ-APP-ERROR.2: Connection error displays status
   *
   * WHEN WebSocket connection fails
   * THEN ConnectionStatusIndicator SHALL show error status
   */
  test('displays connection error status', () => {
    // Note: Would need to update mock to return error status
    // GIVEN - mock would return 'error' status

    // WHEN
    render(<App />);

    // THEN
    const indicator = screen.getByTestId('connection-status-indicator');
    expect(indicator).toHaveTextContent(/error|disconnected/i);
  });
});

// =============================================================================
// Accessibility
// =============================================================================

describe('REQ-APP-A11Y: Accessibility', () => {
  /**
   * REQ-APP-A11Y.1: App container has correct ARIA structure
   *
   * WHEN App renders
   * THEN main content areas SHALL have appropriate ARIA roles
   */
  test('app container has correct ARIA structure', () => {
    // WHEN
    render(<App />);

    // THEN
    expect(screen.getByRole('banner')).toBeInTheDocument(); // header
    expect(screen.getByRole('navigation')).toBeInTheDocument(); // sidebar
    expect(screen.getByRole('main')).toBeInTheDocument(); // main content
  });

  /**
   * REQ-APP-A11Y.2: Focus management on panel open/close
   *
   * WHEN EntityDetailPanel opens
   * THEN focus SHALL move to panel
   *
   * WHEN EntityDetailPanel closes
   * THEN focus SHALL return to previously focused element
   */
  test('manages focus on panel open and close', async () => {
    // GIVEN
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: null,
    });
    render(<App />);

    // Focus a node in the graph area
    const canvas = screen.getByTestId('app-main-canvas');
    canvas.focus();

    // WHEN panel opens
    act(() => {
      useDiffVisualizationStore.getState().actions.selectNodeById(mockGraphNode.id);
    });

    // THEN focus moves to panel
    await waitFor(() => {
      const panel = screen.getByTestId('entity-detail-panel');
      expect(panel).toContainElement(document.activeElement as HTMLElement);
    });

    // WHEN panel closes
    fireEvent.keyDown(document, { key: 'Escape' });

    // THEN focus returns (implementation-specific)
    await waitFor(() => {
      expect(document.activeElement).not.toBe(screen.getByTestId('entity-detail-panel'));
    });
  });
});
