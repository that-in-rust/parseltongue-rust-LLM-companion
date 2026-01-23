/**
 * Root Application Component
 *
 * REQ-APP-INTEGRATION: App.tsx Root Component Integration
 * Main layout component that orchestrates the application structure,
 * wires components to stores, and manages WebSocket lifecycle.
 */

import { useEffect, useState, useCallback, useMemo } from 'react';
import { WorkspaceListSidebar } from './components/WorkspaceListSidebar';
import { DiffGraphCanvasView } from './components/DiffGraphCanvasView';
import { ConnectionStatusIndicator } from './components/ConnectionStatusIndicator';
import { EntityDetailPanel } from './components/EntityDetailPanel';
import { DiffSummaryStats } from './components/DiffSummaryStats';
import {
  useGraphData,
  useSelectedNode,
  useDiffVisualizationActions,
} from './stores/diffVisualizationStore';
import {
  useSelectedWorkspaceId,
  useWorkspaceList,
  useWorkspaceError,
  useWorkspaceActions,
} from './stores/workspaceStore';
import { useWebsocketDiffStream } from './hooks/useWebsocketDiffStream';
import type { GraphNode } from './types/api';

/**
 * Main App component.
 *
 * REQ-APP-001: Layout Composition
 * REQ-APP-002: Component Wiring
 * REQ-APP-003: WebSocket Integration
 * REQ-APP-004: User Flow
 */
export function App(): JSX.Element {
  // =============================================================================
  // State Management - Stores
  // =============================================================================
  const graphData = useGraphData();
  const selectedNode = useSelectedNode();
  const selectedWorkspaceId = useSelectedWorkspaceId();
  const workspaces = useWorkspaceList();
  const workspaceError = useWorkspaceError();
  const { selectNodeById, clearSelectedNode, applyEntityEvent, clearAllGraphData, setDiffInProgress, updateSummaryData } = useDiffVisualizationActions();
  const { fetchWorkspaceListData } = useWorkspaceActions();

  // =============================================================================
  // State Management - WebSocket
  // =============================================================================
  const { connectionStatus, lastDiffEvent, reconnectAttempt, maxReconnectAttempts, subscribe, unsubscribe } = useWebsocketDiffStream();

  // =============================================================================
  // State Management - Local UI State
  // =============================================================================
  const [isSidebarOpenMobile, setIsSidebarOpenMobile] = useState(false);

  // Check if we're on mobile viewport (for visibility logic)
  const [isMobileViewport, setIsMobileViewport] = useState(
    typeof window !== 'undefined' ? window.innerWidth < 768 : false
  );

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handleResize = () => {
      setIsMobileViewport(window.innerWidth < 768);
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // =============================================================================
  // REQ-APP-003: WebSocket Integration - Workspace Subscription
  // =============================================================================
  useEffect(() => {
    if (!selectedWorkspaceId) {
      // No workspace selected, unsubscribe if needed
      unsubscribe();
      clearAllGraphData();
      return;
    }

    // Find selected workspace
    const selectedWorkspace = workspaces.find(
      (ws) => ws.workspace_identifier_value === selectedWorkspaceId
    );

    if (!selectedWorkspace) {
      return;
    }

    // REQ-APP-003.2: Only subscribe if watch is enabled
    if (selectedWorkspace.watch_enabled_flag_status) {
      // Clear previous graph data when switching workspaces
      clearAllGraphData();
      subscribe(selectedWorkspaceId);
    } else {
      unsubscribe();
    }

    // Cleanup on workspace change
    return () => {
      unsubscribe();
    };
  }, [selectedWorkspaceId, workspaces, subscribe, unsubscribe, clearAllGraphData]);

  // =============================================================================
  // REQ-APP-003.3: Diff Event Processing
  // =============================================================================
  useEffect(() => {
    if (!lastDiffEvent) {
      return;
    }

    // Process WebSocket events and update store
    if (lastDiffEvent.event === 'diff_started') {
      setDiffInProgress(true);
    } else if (lastDiffEvent.event === 'diff_completed') {
      setDiffInProgress(false);
      if ('summary' in lastDiffEvent) {
        updateSummaryData(lastDiffEvent.summary);
      }
    } else if (
      lastDiffEvent.event === 'entity_added' ||
      lastDiffEvent.event === 'entity_removed' ||
      lastDiffEvent.event === 'entity_modified' ||
      lastDiffEvent.event === 'edge_added' ||
      lastDiffEvent.event === 'edge_removed'
    ) {
      applyEntityEvent(lastDiffEvent);
    }
  }, [lastDiffEvent, applyEntityEvent, setDiffInProgress, updateSummaryData]);

  // =============================================================================
  // REQ-APP-002: Component Wiring - Callbacks
  // =============================================================================
  const handleNodeClickSelection = useCallback(
    (node: GraphNode) => {
      selectNodeById(node.id);
    },
    [selectNodeById]
  );

  const handleBackgroundClickDeselect = useCallback(() => {
    clearSelectedNode();
  }, [clearSelectedNode]);

  const handleSidebarToggleClick = useCallback(() => {
    setIsSidebarOpenMobile((prev) => !prev);
  }, []);

  const handleSidebarCloseAction = useCallback(() => {
    setIsSidebarOpenMobile(false);
  }, []);

  // =============================================================================
  // REQ-APP-004.2: Keyboard Navigation - Escape Key
  // =============================================================================
  const handleEscapeKeyPress = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        // Close detail panel if open
        if (selectedNode) {
          clearSelectedNode();
          event.preventDefault();
          return;
        }
        // Close mobile sidebar if open
        if (isSidebarOpenMobile) {
          handleSidebarCloseAction();
          event.preventDefault();
        }
      }
    },
    [selectedNode, isSidebarOpenMobile, clearSelectedNode, handleSidebarCloseAction]
  );

  useEffect(() => {
    document.addEventListener('keydown', handleEscapeKeyPress);
    return () => {
      document.removeEventListener('keydown', handleEscapeKeyPress);
    };
  }, [handleEscapeKeyPress]);

  // =============================================================================
  // REQ-APP-002.4: Compute Blast Radius Count
  // =============================================================================
  const blastRadiusCount = useMemo(() => {
    return graphData.nodes.filter((node) => node.changeType === 'affected').length;
  }, [graphData.nodes]);

  // =============================================================================
  // REQ-APP-001: Layout Composition - Render
  // =============================================================================
  return (
    <div className="h-screen bg-gray-900 text-white flex flex-col" data-testid="app-container">
      {/* REQ-APP-001.1: Header Bar */}
      <header
        className="h-16 bg-gray-800 border-b border-gray-700 flex items-center justify-between px-4"
        data-testid="app-header"
        role="banner"
      >
        {/* Left: Mobile Toggle + DiffSummaryStats */}
        <div className="flex items-center gap-4">
          {/* REQ-APP-001.4: Mobile Toggle Button */}
          <button
            data-testid="sidebar-toggle-button"
            className="md:hidden p-2 rounded hover:bg-gray-700"
            onClick={handleSidebarToggleClick}
            aria-label="Toggle workspace sidebar"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 12h16M4 18h16"
              />
            </svg>
          </button>

          {/* REQ-APP-002.5: DiffSummaryStats */}
          <DiffSummaryStats blastRadiusCount={blastRadiusCount} />
        </div>

        {/* Right: ConnectionStatusIndicator */}
        <ConnectionStatusIndicator
          connectionStatus={connectionStatus}
          reconnectAttempt={reconnectAttempt}
          maxReconnectAttempts={maxReconnectAttempts}
        />
      </header>

      {/* Main Layout */}
      <div className="flex-1 flex overflow-hidden">
        {/* REQ-APP-001.2: Sidebar */}
        <aside
          data-testid="app-sidebar"
          className={`
            w-64 bg-gray-800 border-r border-gray-700
            fixed md:relative inset-y-0 left-0 z-50
            transform transition-transform duration-300 ease-in-out
            ${isSidebarOpenMobile ? 'translate-x-0 md:translate-x-0' : '-translate-x-full md:translate-x-0'}
          `}
          role="navigation"
          style={{
            top: '64px',
            height: 'calc(100vh - 64px)',
            // REQ-APP-004.6: Hide sidebar on mobile when closed
            visibility: isMobileViewport && !isSidebarOpenMobile ? 'hidden' : 'visible'
          }}
        >
          <WorkspaceListSidebar />
        </aside>

        {/* Mobile Sidebar Backdrop */}
        {isSidebarOpenMobile && (
          <div
            className="fixed inset-0 bg-black/50 z-40 md:hidden"
            onClick={handleSidebarCloseAction}
            aria-hidden="true"
          />
        )}

        {/* REQ-APP-001.3: Main Canvas Area */}
        <main
          className="flex-1 relative overflow-hidden"
          data-testid="app-main-canvas"
          role="main"
        >
          {/* REQ-APP-ERROR.1: Error State */}
          {workspaceError && (
            <div className="absolute top-4 left-1/2 transform -translate-x-1/2 z-10 bg-red-500/20 border border-red-500 text-red-400 px-4 py-2 rounded-lg">
              <p className="font-medium">Failed to load workspaces</p>
              <button
                onClick={() => fetchWorkspaceListData()}
                className="mt-2 px-3 py-1 bg-red-500 hover:bg-red-600 text-white rounded text-sm"
                role="button"
                aria-label="Retry loading workspaces"
              >
                Retry
              </button>
            </div>
          )}

          {/* REQ-APP-002.2: DiffGraphCanvasView */}
          <DiffGraphCanvasView
            graphData={graphData}
            onNodeClick={handleNodeClickSelection}
            onBackgroundClick={handleBackgroundClickDeselect}
          />

          {/* REQ-APP-002.3: EntityDetailPanel (Conditional) */}
          <EntityDetailPanel />
        </main>
      </div>
    </div>
  );
}
