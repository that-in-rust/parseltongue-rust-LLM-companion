/**
 * Entity Detail Panel Tests
 *
 * REQ-DETAIL-001: EntityDetailPanel Component
 * Tests for REQ-DETAIL-001.1 through REQ-DETAIL-001.9
 *
 * These tests verify the slide-out panel that displays selected node details
 * including entity identity, file location, and dependency relationships.
 */

import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EntityDetailPanel } from '../EntityDetailPanel';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';
import type { GraphNode, ForceGraphData } from '@/types/api';

// =============================================================================
// Test Fixtures
// =============================================================================

const mockNode: GraphNode = {
  id: 'rust:fn:handle_auth',
  name: 'handle_auth',
  nodeType: 'function',
  changeType: 'added',
  filePath: 'src/auth.rs',
  lineStart: 10,
  lineEnd: 45,
};

const mockNodeRemoved: GraphNode = {
  id: 'rust:fn:old_handler',
  name: 'old_handler',
  nodeType: 'function',
  changeType: 'removed',
  filePath: 'src/legacy.rs',
  lineStart: 5,
  lineEnd: 20,
};

const mockNodeModified: GraphNode = {
  id: 'rust:fn:process_data',
  name: 'process_data',
  nodeType: 'function',
  changeType: 'modified',
  filePath: 'src/processor.rs',
  lineStart: 100,
  lineEnd: 150,
};

const mockNodeAffected: GraphNode = {
  id: 'rust:fn:downstream',
  name: 'downstream',
  nodeType: 'function',
  changeType: 'affected',
  filePath: 'src/downstream.rs',
  lineStart: 1,
  lineEnd: 10,
};

const mockNodeUnchanged: GraphNode = {
  id: 'rust:fn:validate',
  name: 'validate',
  nodeType: 'function',
  changeType: null,
};

const mockNodeWithoutPath: GraphNode = {
  id: 'rust:fn:unknown_location',
  name: 'unknown_location',
  nodeType: 'function',
  changeType: 'added',
  filePath: undefined,
  lineStart: undefined,
  lineEnd: undefined,
};

const mockNodeSingleLine: GraphNode = {
  id: 'rust:fn:single_line',
  name: 'single_line',
  nodeType: 'function',
  changeType: 'modified',
  filePath: 'src/utils.rs',
  lineStart: 42,
  lineEnd: 42,
};

const mockGraphData: ForceGraphData = {
  nodes: [
    mockNode,
    { id: 'rust:fn:validate', name: 'validate', nodeType: 'function', changeType: null },
    { id: 'rust:fn:login', name: 'login', nodeType: 'function', changeType: 'modified' },
    { id: 'rust:mod:auth', name: 'auth', nodeType: 'module', changeType: null },
    { id: 'rust:struct:User', name: 'User', nodeType: 'struct', changeType: 'added' },
  ],
  links: [
    { source: 'rust:fn:login', target: 'rust:fn:handle_auth', edgeType: 'Calls' },
    { source: 'rust:mod:auth', target: 'rust:fn:handle_auth', edgeType: 'Contains' },
    { source: 'rust:fn:handle_auth', target: 'rust:fn:validate', edgeType: 'Calls' },
    { source: 'rust:fn:handle_auth', target: 'rust:struct:User', edgeType: 'Uses' },
  ],
};

const mockGraphDataWithManyDeps: ForceGraphData = {
  nodes: [
    mockNode,
    ...Array.from({ length: 10 }, (_, i) => ({
      id: `rust:fn:caller_${i}`,
      name: `caller_${i}`,
      nodeType: 'function',
      changeType: null as const,
    })),
    ...Array.from({ length: 8 }, (_, i) => ({
      id: `rust:fn:callee_${i}`,
      name: `callee_${i}`,
      nodeType: 'function',
      changeType: null as const,
    })),
  ],
  links: [
    ...Array.from({ length: 10 }, (_, i) => ({
      source: `rust:fn:caller_${i}`,
      target: 'rust:fn:handle_auth',
      edgeType: 'Calls',
    })),
    ...Array.from({ length: 8 }, (_, i) => ({
      source: 'rust:fn:handle_auth',
      target: `rust:fn:callee_${i}`,
      edgeType: 'Calls',
    })),
  ],
};

// =============================================================================
// Test Setup
// =============================================================================

beforeEach(() => {
  useDiffVisualizationStore.setState({
    graphData: mockGraphData,
    selectedNode: null,
    summary: null,
    isDiffInProgress: false,
  });
});

// =============================================================================
// REQ-DETAIL-001.1: Panel Visibility on Node Selection
// =============================================================================

describe('REQ-DETAIL-001.1: Panel Visibility on Node Selection', () => {
  /**
   * REQ-DETAIL-001.1a: Panel renders when node selected
   *
   * WHEN selectedNode in diffVisualizationStore is non-null
   * THEN EntityDetailPanel SHALL render as visible
   */
  test('renders panel when node is selected', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('entity-detail-panel')).toBeVisible();
  });

  /**
   * REQ-DETAIL-001.1b: Panel slides in from right
   *
   * WHEN selectedNode in diffVisualizationStore is non-null
   * THEN EntityDetailPanel SHALL slide in from the right edge of the screen
   *   AND SHALL have translate-x-0 class when visible
   */
  test('panel slides in from right with correct transform class', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('translate-x-0');
  });

  /**
   * REQ-DETAIL-001.1c: Panel width on desktop
   *
   * WHEN selectedNode is non-null AND viewport width >= 768px
   * THEN EntityDetailPanel SHALL occupy 320px width
   */
  test('panel has correct width on desktop', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('w-80'); // 320px = w-80 in Tailwind
  });

  /**
   * REQ-DETAIL-001.1d: Panel has correct z-index
   *
   * WHEN EntityDetailPanel is visible
   * THEN SHALL have z-index higher than graph canvas (z-50)
   */
  test('panel has correct z-index above canvas', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('z-50');
  });
});

// =============================================================================
// REQ-DETAIL-001.2: Panel Hidden When No Selection
// =============================================================================

describe('REQ-DETAIL-001.2: Panel Hidden When No Selection', () => {
  /**
   * REQ-DETAIL-001.2a: Panel not visible when no selection
   *
   * WHEN selectedNode in diffVisualizationStore is null
   * THEN EntityDetailPanel SHALL NOT be visible
   */
  test('hides panel when no node selected', () => {
    useDiffVisualizationStore.setState({ selectedNode: null });

    render(<EntityDetailPanel />);

    expect(screen.queryByTestId('entity-detail-panel')).not.toBeVisible();
  });

  /**
   * REQ-DETAIL-001.2b: Panel slides out to right
   *
   * WHEN selectedNode is null
   * THEN EntityDetailPanel SHALL slide out to the right
   *   AND SHALL have translate-x-full class
   */
  test('panel slides out with correct transform class', () => {
    useDiffVisualizationStore.setState({ selectedNode: null });

    render(<EntityDetailPanel />);

    const panel = screen.queryByTestId('entity-detail-panel');
    expect(panel).toHaveClass('translate-x-full');
  });

  /**
   * REQ-DETAIL-001.2c: Panel does not occupy layout space when hidden
   *
   * WHEN selectedNode is null
   * THEN EntityDetailPanel SHALL NOT occupy layout space when hidden
   */
  test('panel does not occupy layout space when hidden', () => {
    useDiffVisualizationStore.setState({ selectedNode: null });

    render(<EntityDetailPanel />);

    const panel = screen.queryByTestId('entity-detail-panel');
    expect(panel).toHaveClass('fixed');
  });
});

// =============================================================================
// REQ-DETAIL-001.3: Display Node Identity Information
// =============================================================================

describe('REQ-DETAIL-001.3: Display Node Identity Information', () => {
  /**
   * REQ-DETAIL-001.3a: Display entity name
   *
   * WHEN EntityDetailPanel renders with selectedNode
   * THEN SHALL display node.name in text-lg font weight semibold
   */
  test('displays entity name with correct styling', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const entityName = screen.getByTestId('entity-name');
    expect(entityName).toHaveTextContent('handle_auth');
    expect(entityName).toHaveClass('text-lg', 'font-semibold');
  });

  /**
   * REQ-DETAIL-001.3b: Display entity type
   *
   * WHEN EntityDetailPanel renders with selectedNode
   * THEN SHALL display node.nodeType in parentheses with text-sm text-gray-400
   */
  test('displays entity type in parentheses', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const entityType = screen.getByTestId('entity-type');
    expect(entityType).toHaveTextContent('(function)');
    expect(entityType).toHaveClass('text-sm', 'text-gray-400');
  });

  /**
   * REQ-DETAIL-001.3c: Display change type badge - Added
   *
   * WHEN selectedNode.changeType === 'added'
   * THEN badge SHALL have bg-green-500/20 text-green-400 border-green-500/50
   *   AND text SHALL be "Added"
   */
  test('displays added change type badge with correct styling', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const badge = screen.getByTestId('change-type-badge');
    expect(badge).toHaveTextContent('Added');
    expect(badge).toHaveClass('bg-green-500/20');
    expect(badge).toHaveClass('text-green-400');
  });

  /**
   * REQ-DETAIL-001.3d: Display change type badge - Removed
   *
   * WHEN selectedNode.changeType === 'removed'
   * THEN badge SHALL have bg-red-500/20 text-red-400 border-red-500/50
   *   AND text SHALL be "Removed"
   */
  test('displays removed change type badge with correct styling', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        ...mockGraphData,
        nodes: [...mockGraphData.nodes, mockNodeRemoved],
      },
      selectedNode: mockNodeRemoved,
    });

    render(<EntityDetailPanel />);

    const badge = screen.getByTestId('change-type-badge');
    expect(badge).toHaveTextContent('Removed');
    expect(badge).toHaveClass('bg-red-500/20');
    expect(badge).toHaveClass('text-red-400');
  });

  /**
   * REQ-DETAIL-001.3e: Display change type badge - Modified
   *
   * WHEN selectedNode.changeType === 'modified'
   * THEN badge SHALL have bg-amber-500/20 text-amber-400 border-amber-500/50
   *   AND text SHALL be "Modified"
   */
  test('displays modified change type badge with correct styling', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        ...mockGraphData,
        nodes: [...mockGraphData.nodes, mockNodeModified],
      },
      selectedNode: mockNodeModified,
    });

    render(<EntityDetailPanel />);

    const badge = screen.getByTestId('change-type-badge');
    expect(badge).toHaveTextContent('Modified');
    expect(badge).toHaveClass('bg-amber-500/20');
    expect(badge).toHaveClass('text-amber-400');
  });

  /**
   * REQ-DETAIL-001.3f: Display change type badge - Affected
   *
   * WHEN selectedNode.changeType === 'affected'
   * THEN badge SHALL have bg-blue-500/20 text-blue-400 border-blue-500/50
   *   AND text SHALL be "Affected"
   */
  test('displays affected change type badge with correct styling', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        ...mockGraphData,
        nodes: [...mockGraphData.nodes, mockNodeAffected],
      },
      selectedNode: mockNodeAffected,
    });

    render(<EntityDetailPanel />);

    const badge = screen.getByTestId('change-type-badge');
    expect(badge).toHaveTextContent('Affected');
    expect(badge).toHaveClass('bg-blue-500/20');
    expect(badge).toHaveClass('text-blue-400');
  });

  /**
   * REQ-DETAIL-001.3g: Display change type badge - Unchanged (null)
   *
   * WHEN selectedNode.changeType === null
   * THEN badge SHALL have bg-gray-500/20 text-gray-400 border-gray-500/50
   *   AND text SHALL be "Unchanged"
   */
  test('displays unchanged badge when changeType is null', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNodeUnchanged });

    render(<EntityDetailPanel />);

    const badge = screen.getByTestId('change-type-badge');
    expect(badge).toHaveTextContent('Unchanged');
    expect(badge).toHaveClass('bg-gray-500/20');
    expect(badge).toHaveClass('text-gray-400');
  });
});

// =============================================================================
// REQ-DETAIL-001.4: Display File Location
// =============================================================================

describe('REQ-DETAIL-001.4: Display File Location', () => {
  /**
   * REQ-DETAIL-001.4a: Display file path with line range
   *
   * WHEN selectedNode has filePath, lineStart, and lineEnd defined
   *   AND lineEnd !== lineStart
   * THEN SHALL display as "filePath:lineStart-lineEnd"
   */
  test('displays file location with line range', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const fileLocation = screen.getByTestId('file-location');
    expect(fileLocation).toHaveTextContent('src/auth.rs:10-45');
  });

  /**
   * REQ-DETAIL-001.4b: Display file path with single line
   *
   * WHEN selectedNode has lineEnd === lineStart
   * THEN SHALL display as "filePath:lineStart" (no range)
   */
  test('displays file location with single line number', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        ...mockGraphData,
        nodes: [...mockGraphData.nodes, mockNodeSingleLine],
      },
      selectedNode: mockNodeSingleLine,
    });

    render(<EntityDetailPanel />);

    const fileLocation = screen.getByTestId('file-location');
    expect(fileLocation).toHaveTextContent('src/utils.rs:42');
    expect(fileLocation).not.toHaveTextContent('42-42');
  });

  /**
   * REQ-DETAIL-001.4c: File label styling
   *
   * WHEN EntityDetailPanel renders file location section
   * THEN label "File" SHALL have text-xs text-gray-500 uppercase
   */
  test('displays file label with correct styling', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const fileLabel = screen.getByTestId('file-label');
    expect(fileLabel).toHaveTextContent('File');
    expect(fileLabel).toHaveClass('text-xs', 'text-gray-500', 'uppercase');
  });

  /**
   * REQ-DETAIL-001.4d: File path styling
   *
   * WHEN EntityDetailPanel renders file location
   * THEN value SHALL have text-sm font-mono truncated with title tooltip
   */
  test('displays file path with monospace font', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const fileLocation = screen.getByTestId('file-location');
    expect(fileLocation).toHaveClass('text-sm', 'font-mono', 'truncate');
    expect(fileLocation).toHaveAttribute('title', 'src/auth.rs:10-45');
  });

  /**
   * REQ-DETAIL-001.4e: Location unknown when filePath undefined
   *
   * WHEN selectedNode.filePath is undefined
   * THEN SHALL display "Location unknown" in text-gray-500 italic
   */
  test('displays "Location unknown" when filePath is undefined', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        ...mockGraphData,
        nodes: [...mockGraphData.nodes, mockNodeWithoutPath],
      },
      selectedNode: mockNodeWithoutPath,
    });

    render(<EntityDetailPanel />);

    const unknownText = screen.getByText('Location unknown');
    expect(unknownText).toBeInTheDocument();
    expect(unknownText).toHaveClass('text-gray-500', 'italic');
  });
});

// =============================================================================
// REQ-DETAIL-001.5: Display Incoming Dependencies
// =============================================================================

describe('REQ-DETAIL-001.5: Display Incoming Dependencies', () => {
  /**
   * REQ-DETAIL-001.5a: Display incoming dependencies header with count
   *
   * WHEN EntityDetailPanel renders with selectedNode
   * THEN SHALL display "Incoming" header with count badge
   */
  test('displays incoming dependencies header with count', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('incoming-deps-header')).toHaveTextContent('Incoming');
    expect(screen.getByTestId('incoming-deps-count')).toHaveTextContent('2');
  });

  /**
   * REQ-DETAIL-001.5b: Display incoming dependency names
   *
   * WHEN selectedNode has incoming dependencies
   * THEN SHALL display source node names for each dependency
   */
  test('displays incoming dependency source names', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByText('login')).toBeInTheDocument();
    expect(screen.getByText('auth')).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.5c: Group incoming dependencies by edge type
   *
   * WHEN selectedNode has incoming dependencies of multiple types
   * THEN SHALL group by edgeType (e.g., "Calls", "Contains")
   */
  test('groups incoming dependencies by edge type', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('incoming-group-Calls')).toBeInTheDocument();
    expect(screen.getByTestId('incoming-group-Contains')).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.5d: Display "No incoming dependencies" when empty
   *
   * WHEN selectedNode has no incoming dependencies
   * THEN SHALL display "No incoming dependencies" in text-gray-500
   */
  test('displays "No incoming dependencies" when none exist', () => {
    const isolatedNode: GraphNode = {
      id: 'isolated',
      name: 'isolated',
      nodeType: 'function',
      changeType: null,
    };
    const graphWithIsolated: ForceGraphData = {
      nodes: [...mockGraphData.nodes, isolatedNode],
      links: mockGraphData.links,
    };
    useDiffVisualizationStore.setState({
      graphData: graphWithIsolated,
      selectedNode: isolatedNode,
    });

    render(<EntityDetailPanel />);

    const emptyText = screen.getByText('No incoming dependencies');
    expect(emptyText).toBeInTheDocument();
    expect(emptyText).toHaveClass('text-gray-500');
  });

  /**
   * REQ-DETAIL-001.5e: "+N more" expandable for > 5 items
   *
   * WHEN an edge type group has > 5 items
   * THEN SHALL show "+N more" expandable link
   */
  test('shows "+N more" link when group has more than 5 items', () => {
    useDiffVisualizationStore.setState({
      graphData: mockGraphDataWithManyDeps,
      selectedNode: mockNode,
    });

    render(<EntityDetailPanel />);

    // 10 callers, showing 5, so "+5 more"
    expect(screen.getByText('+5 more')).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.5f: Clicking dependency selects that node
   *
   * WHEN user clicks on a dependency item
   * THEN SHALL call selectNodeById with that node's ID
   */
  test('clicking incoming dependency selects that node', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    await userEvent.click(screen.getByText('login'));

    expect(useDiffVisualizationStore.getState().selectedNode?.id).toBe('rust:fn:login');
  });
});

// =============================================================================
// REQ-DETAIL-001.6: Display Outgoing Dependencies
// =============================================================================

describe('REQ-DETAIL-001.6: Display Outgoing Dependencies', () => {
  /**
   * REQ-DETAIL-001.6a: Display outgoing dependencies header with count
   *
   * WHEN EntityDetailPanel renders with selectedNode
   * THEN SHALL display "Outgoing" header with count badge
   */
  test('displays outgoing dependencies header with count', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('outgoing-deps-header')).toHaveTextContent('Outgoing');
    expect(screen.getByTestId('outgoing-deps-count')).toHaveTextContent('2');
  });

  /**
   * REQ-DETAIL-001.6b: Display outgoing dependency names
   *
   * WHEN selectedNode has outgoing dependencies
   * THEN SHALL display target node names for each dependency
   */
  test('displays outgoing dependency target names', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByText('validate')).toBeInTheDocument();
    expect(screen.getByText('User')).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.6c: Group outgoing dependencies by edge type
   *
   * WHEN selectedNode has outgoing dependencies of multiple types
   * THEN SHALL group by edgeType (e.g., "Calls", "Uses")
   */
  test('groups outgoing dependencies by edge type', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    expect(screen.getByTestId('outgoing-group-Calls')).toBeInTheDocument();
    expect(screen.getByTestId('outgoing-group-Uses')).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.6d: Display "No outgoing dependencies" when empty
   *
   * WHEN selectedNode has no outgoing dependencies
   * THEN SHALL display "No outgoing dependencies" in text-gray-500
   */
  test('displays "No outgoing dependencies" when none exist', () => {
    const leafNode: GraphNode = {
      id: 'rust:fn:validate',
      name: 'validate',
      nodeType: 'function',
      changeType: null,
    };
    useDiffVisualizationStore.setState({
      graphData: mockGraphData,
      selectedNode: leafNode,
    });

    render(<EntityDetailPanel />);

    const emptyText = screen.getByText('No outgoing dependencies');
    expect(emptyText).toBeInTheDocument();
    expect(emptyText).toHaveClass('text-gray-500');
  });

  /**
   * REQ-DETAIL-001.6e: Clicking outgoing dependency selects that node
   *
   * WHEN user clicks on an outgoing dependency item
   * THEN SHALL call selectNodeById with that node's ID
   */
  test('clicking outgoing dependency selects that node', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    await userEvent.click(screen.getByText('validate'));

    expect(useDiffVisualizationStore.getState().selectedNode?.id).toBe('rust:fn:validate');
  });
});

// =============================================================================
// REQ-DETAIL-001.7: Close Panel with Escape Key
// =============================================================================

describe('REQ-DETAIL-001.7: Close Panel with Escape Key', () => {
  /**
   * REQ-DETAIL-001.7a: Escape key closes panel
   *
   * WHEN EntityDetailPanel is visible AND user presses Escape key
   * THEN SHALL call clearSelectedNode() from diffVisualizationStore
   */
  test('Escape key closes panel', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    fireEvent.keyDown(document, { key: 'Escape' });

    await waitFor(() => {
      expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
    });
  });

  /**
   * REQ-DETAIL-001.7b: Panel animates closed on Escape
   *
   * WHEN Escape is pressed
   * THEN panel SHALL animate closed
   */
  test('panel animates closed on Escape', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('translate-x-0');

    fireEvent.keyDown(document, { key: 'Escape' });

    await waitFor(() => {
      expect(panel).toHaveClass('translate-x-full');
    });
  });
});

// =============================================================================
// REQ-DETAIL-001.8: Close Button
// =============================================================================

describe('REQ-DETAIL-001.8: Close Button', () => {
  /**
   * REQ-DETAIL-001.8a: Close button is present
   *
   * WHEN EntityDetailPanel is visible
   * THEN SHALL display close button (X icon) in top-right corner
   */
  test('displays close button when panel is visible', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const closeButton = screen.getByRole('button', { name: /close/i });
    expect(closeButton).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.8b: Close button has correct aria-label
   *
   * WHEN EntityDetailPanel is visible
   * THEN close button SHALL have aria-label="Close entity details"
   */
  test('close button has correct aria-label', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const closeButton = screen.getByRole('button', { name: 'Close entity details' });
    expect(closeButton).toBeInTheDocument();
  });

  /**
   * REQ-DETAIL-001.8c: Close button hover state
   *
   * WHEN close button is rendered
   * THEN SHALL have hover state (hover:bg-gray-700)
   */
  test('close button has hover styling class', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const closeButton = screen.getByRole('button', { name: /close/i });
    expect(closeButton).toHaveClass('hover:bg-gray-700');
  });

  /**
   * REQ-DETAIL-001.8d: Clicking close button clears selection
   *
   * WHEN user clicks close button
   * THEN SHALL call clearSelectedNode()
   */
  test('clicking close button clears selection', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    await userEvent.click(screen.getByRole('button', { name: /close/i }));

    expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
  });
});

// =============================================================================
// REQ-DETAIL-001.9: Responsive Layout
// =============================================================================

describe('REQ-DETAIL-001.9: Responsive Layout', () => {
  /**
   * REQ-DETAIL-001.9a: Desktop renders as side panel
   *
   * WHEN viewport width >= 768px
   * THEN EntityDetailPanel SHALL render as right-side panel with fixed width 320px
   */
  test('renders as side panel on desktop', () => {
    // Note: Testing responsive behavior requires mocking window.matchMedia
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('md:w-80'); // 320px on desktop
    expect(panel).toHaveClass('md:right-0');
  });

  /**
   * REQ-DETAIL-001.9b: Mobile renders as bottom sheet
   *
   * WHEN viewport width < 768px
   * THEN EntityDetailPanel SHALL render as bottom sheet
   *   AND SHALL occupy 100% width
   *   AND max height SHALL be 60vh
   */
  test('renders as bottom sheet on mobile', () => {
    // Note: Testing responsive behavior requires mocking window.matchMedia
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveClass('w-full'); // Full width on mobile
    expect(panel).toHaveClass('max-h-[60vh]'); // Max 60vh height
  });

  /**
   * REQ-DETAIL-001.9c: Mobile includes drag handle
   *
   * WHEN viewport width < 768px
   * THEN EntityDetailPanel SHALL include drag handle for swipe-to-close
   */
  test('includes drag handle on mobile', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const dragHandle = screen.getByTestId('drag-handle');
    expect(dragHandle).toBeInTheDocument();
  });
});

// =============================================================================
// Error Conditions
// =============================================================================

describe('EntityDetailPanel Error Conditions', () => {
  /**
   * Error: Node not found in graphData
   *
   * WHEN selectedNode references a node not in graphData.nodes
   * THEN SHALL display error state with message "Entity not found in current graph"
   */
  test('displays error when selected node not in graph data', () => {
    const orphanNode: GraphNode = {
      id: 'orphan:fn:missing',
      name: 'missing',
      nodeType: 'function',
      changeType: null,
    };
    useDiffVisualizationStore.setState({
      graphData: mockGraphData, // Does not contain orphanNode
      selectedNode: orphanNode,
    });
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    render(<EntityDetailPanel />);

    expect(screen.getByText('Entity not found in current graph')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /close/i })).toBeInTheDocument();
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('not found in current graph')
    );

    consoleSpy.mockRestore();
  });

  /**
   * Error: Malformed link data
   *
   * WHEN computing dependencies encounters malformed link data
   * THEN SHALL skip malformed entries and NOT crash component
   */
  test('handles malformed link data gracefully', () => {
    const graphWithBadLinks: ForceGraphData = {
      nodes: mockGraphData.nodes,
      links: [
        ...mockGraphData.links,
        // Malformed: missing target
        { source: 'rust:fn:login', target: '', edgeType: 'Calls' },
        // Malformed: missing source
        { source: '', target: 'rust:fn:handle_auth', edgeType: 'Calls' },
      ],
    };
    useDiffVisualizationStore.setState({
      graphData: graphWithBadLinks,
      selectedNode: mockNode,
    });
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    render(<EntityDetailPanel />);

    // Should still render without crashing
    expect(screen.getByTestId('entity-detail-panel')).toBeVisible();
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('malformed link')
    );

    consoleSpy.mockRestore();
  });
});

// =============================================================================
// Accessibility Tests (REQ-A11Y-001.1)
// =============================================================================

describe('REQ-A11Y-001.1: EntityDetailPanel Accessibility', () => {
  /**
   * Panel has correct ARIA role
   *
   * WHEN EntityDetailPanel renders
   * THEN SHALL have role="complementary"
   */
  test('panel has complementary role', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveAttribute('role', 'complementary');
  });

  /**
   * Panel has correct ARIA label
   *
   * WHEN EntityDetailPanel renders
   * THEN SHALL have aria-label="Entity details panel"
   */
  test('panel has correct aria-label', () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const panel = screen.getByTestId('entity-detail-panel');
    expect(panel).toHaveAttribute('aria-label', 'Entity details panel');
  });

  /**
   * Dependency items are keyboard accessible
   *
   * WHEN navigating dependencies list
   * THEN SHALL support Tab navigation between dependency items
   */
  test('dependency items are keyboard navigable', async () => {
    useDiffVisualizationStore.setState({ selectedNode: mockNode });

    render(<EntityDetailPanel />);

    const firstDep = screen.getByText('login');
    firstDep.focus();

    expect(document.activeElement).toBe(firstDep);
  });
});
