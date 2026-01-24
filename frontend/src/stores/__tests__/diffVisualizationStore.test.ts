/**
 * Diff Visualization Store Tests
 *
 * REQ-STORE-002: Diff Visualization Store
 * Tests for graph data and selection state management.
 */

import { describe, test, expect, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import {
  useDiffVisualizationStore,
  useGraphData,
  useSelectedNode,
  useDiffSummary,
  useIsDiffInProgress,
} from '../diffVisualizationStore';
import type { ApiDiffVisualization, WebSocketServerEvent } from '@/types/api';

// =============================================================================
// Test Fixtures
// =============================================================================

const mockApiResponse: ApiDiffVisualization = {
  nodes: [
    {
      id: 'n1',
      label: 'TestFunction',
      node_type: 'function',
      change_type: 'added',
      file_path: 'src/test.ts',
      line_start: 10,
    },
    {
      id: 'n2',
      label: 'HelperFunction',
      node_type: 'function',
      change_type: null,
    },
  ],
  edges: [
    { source: 'n1', target: 'n2', edge_type: 'Calls' },
  ],
};

const mockEntityAddedEvent: WebSocketServerEvent = {
  event: 'entity_added',
  workspace_id: 'ws_1',
  entity_key: 'n3',
  entity_type: 'function',
  file_path: 'src/new.ts',
  line_range: { start: 20, end: 30 },
  timestamp: '2026-01-23T00:00:00Z',
};

// =============================================================================
// REQ-STORE-002: Diff Visualization Store
// =============================================================================

describe('REQ-STORE-002: Diff Visualization Store', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useDiffVisualizationStore.setState({
      graphData: { nodes: [], links: [] },
      selectedNode: null,
      summary: null,
      isDiffInProgress: false,
    });
  });

  /**
   * REQ-STORE-002.1: Store Shape
   *
   * WHEN diffVisualizationStore is created
   * THEN SHALL have correct initial shape with:
   *   - graphData: { nodes: [], links: [] }
   *   - selectedNode: null
   *   - summary: null
   *   - isDiffInProgress: false
   *   - actions object with required methods
   */
  test('store has correct initial shape', () => {
    const state = useDiffVisualizationStore.getState();

    expect(state).toEqual(
      expect.objectContaining({
        graphData: { nodes: [], links: [] },
        selectedNode: null,
        summary: null,
        isDiffInProgress: false,
      })
    );
    expect(state.actions).toBeDefined();
    expect(typeof state.actions.setGraphDataFromApi).toBe('function');
    expect(typeof state.actions.selectNodeById).toBe('function');
    expect(typeof state.actions.clearSelectedNode).toBe('function');
    expect(typeof state.actions.updateSummaryData).toBe('function');
    expect(typeof state.actions.setDiffInProgress).toBe('function');
    expect(typeof state.actions.applyEntityEvent).toBe('function');
    expect(typeof state.actions.clearAllGraphData).toBe('function');
  });

  /**
   * REQ-STORE-002.1: Selector - useGraphData
   *
   * WHEN useGraphData hook is called
   * THEN SHALL return only the graphData from store
   */
  test('useGraphData selector returns graph data', () => {
    const testData = {
      nodes: [{ id: 'n1', name: 'Test', nodeType: 'fn', changeType: null as const }],
      links: [],
    };
    useDiffVisualizationStore.setState({ graphData: testData });

    const { result } = renderHook(() => useGraphData());

    expect(result.current).toEqual(testData);
  });

  /**
   * REQ-STORE-002.2: setGraphDataFromApi transforms API response
   *
   * WHEN setGraphDataFromApi is called with API response format
   * THEN SHALL transform to ForceGraphData format
   *   AND SHALL store in graphData
   *   AND SHALL NOT mutate original response
   */
  test('setGraphDataFromApi transforms API response correctly', () => {
    const { actions } = useDiffVisualizationStore.getState();

    actions.setGraphDataFromApi(mockApiResponse);

    const { graphData } = useDiffVisualizationStore.getState();
    expect(graphData.nodes[0]).toEqual(
      expect.objectContaining({
        id: 'n1',
        name: 'TestFunction',
        nodeType: 'function',
        changeType: 'added',
        filePath: 'src/test.ts',
        lineStart: 10,
      })
    );
    expect(graphData.links[0]).toEqual(
      expect.objectContaining({
        source: 'n1',
        target: 'n2',
        edgeType: 'Calls',
      })
    );
  });

  /**
   * REQ-STORE-002.3: applyEntityEvent updates nodes incrementally
   *
   * WHEN applyEntityEvent is called with entity_added event
   * THEN SHALL add node to graphData.nodes incrementally
   *   AND SHALL NOT replace entire array
   */
  test('applyEntityEvent adds node incrementally for entity_added', () => {
    // Setup initial state with one node
    useDiffVisualizationStore.setState({
      graphData: {
        nodes: [{ id: 'n1', name: 'Existing', nodeType: 'fn', changeType: null }],
        links: [],
      },
    });

    const { actions } = useDiffVisualizationStore.getState();
    actions.applyEntityEvent(mockEntityAddedEvent);

    const { graphData } = useDiffVisualizationStore.getState();
    expect(graphData.nodes).toHaveLength(2);
    expect(graphData.nodes[1]).toEqual(
      expect.objectContaining({
        id: 'n3',
        changeType: 'added',
      })
    );
  });

  /**
   * REQ-STORE-002: selectNodeById updates selectedNode
   *
   * WHEN selectNodeById is called with valid node ID
   * THEN SHALL update selectedNode in store
   */
  test('selectNodeById updates selected node', () => {
    useDiffVisualizationStore.setState({
      graphData: {
        nodes: [{ id: 'n1', name: 'Test', nodeType: 'fn', changeType: 'added' }],
        links: [],
      },
    });

    const { actions } = useDiffVisualizationStore.getState();
    actions.selectNodeById('n1');

    const { selectedNode } = useDiffVisualizationStore.getState();
    expect(selectedNode).toEqual(
      expect.objectContaining({ id: 'n1' })
    );
  });

  /**
   * REQ-STORE-002: clearSelectedNode clears selection
   *
   * WHEN clearSelectedNode is called
   * THEN SHALL set selectedNode to null
   */
  test('clearSelectedNode clears the selected node', () => {
    useDiffVisualizationStore.setState({
      selectedNode: { id: 'n1', name: 'Test', nodeType: 'fn', changeType: 'added' },
    });

    const { actions } = useDiffVisualizationStore.getState();
    actions.clearSelectedNode();

    expect(useDiffVisualizationStore.getState().selectedNode).toBeNull();
  });
});
