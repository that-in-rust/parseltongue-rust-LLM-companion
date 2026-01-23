/**
 * Transform Diff to Forcegraph Tests
 *
 * REQ-VIZ-004: Data Transformation
 * Tests for API response to ForceGraph format transformation.
 */

import { describe, test, expect } from 'vitest';
import {
  transformDiffToForcegraph,
  getNodeColorByChangeType,
  getNodeSizeByChangeType,
  getNodeLabelFormatted,
} from '../transformDiffToForcegraph';
import type { ApiDiffVisualization, GraphNode } from '@/types/api';
import { CHANGE_TYPE_COLORS } from '@/types/api';

// =============================================================================
// REQ-VIZ-004: Data Transformation
// =============================================================================

describe('REQ-VIZ-004: Data Transformation', () => {
  /**
   * REQ-VIZ-004.1: Transform API Response to ForceGraph Format
   *
   * WHEN transformDiffToForcegraph receives API response
   * THEN SHALL return ForceGraphData with:
   *   - nodes: transformed with name (from label), nodeType (from node_type), etc.
   *   - links: transformed with edgeType (from edge_type)
   *   AND SHALL preserve all node IDs for link resolution
   */
  test('transforms API response to ForceGraph format correctly', () => {
    const apiResponse: ApiDiffVisualization = {
      nodes: [
        {
          id: 'n1',
          label: 'handleAuth',
          node_type: 'function',
          change_type: 'added',
          file_path: 'src/auth.ts',
          line_start: 10,
        },
        {
          id: 'n2',
          label: 'validate',
          node_type: 'function',
          change_type: null,
        },
      ],
      edges: [{ source: 'n1', target: 'n2', edge_type: 'Calls' }],
    };

    const result = transformDiffToForcegraph(apiResponse);

    expect(result).toEqual({
      nodes: [
        {
          id: 'n1',
          name: 'handleAuth',
          nodeType: 'function',
          changeType: 'added',
          filePath: 'src/auth.ts',
          lineStart: 10,
        },
        {
          id: 'n2',
          name: 'validate',
          nodeType: 'function',
          changeType: null,
          filePath: undefined,
          lineStart: undefined,
        },
      ],
      links: [{ source: 'n1', target: 'n2', edgeType: 'Calls' }],
    });
  });

  /**
   * REQ-VIZ-004.2: Handle Missing Optional Fields
   *
   * WHEN API response node has null/undefined optional fields
   * THEN SHALL set corresponding output fields to undefined
   *   AND SHALL NOT throw error
   */
  test('handles missing optional fields gracefully', () => {
    const apiResponse: ApiDiffVisualization = {
      nodes: [
        {
          id: 'n1',
          label: 'test',
          node_type: 'fn',
          change_type: null,
          // file_path, line_start intentionally missing
        },
      ],
      edges: [],
    };

    const result = transformDiffToForcegraph(apiResponse);

    expect(result.nodes[0].changeType).toBeNull();
    expect(result.nodes[0].filePath).toBeUndefined();
    expect(result.nodes[0].lineStart).toBeUndefined();
  });

  /**
   * REQ-VIZ-004.3: Handle Empty Input - null
   *
   * WHEN transformDiffToForcegraph receives null
   * THEN SHALL return { nodes: [], links: [] }
   *   AND SHALL NOT throw error
   */
  test('returns empty structure for null input', () => {
    expect(transformDiffToForcegraph(null)).toEqual({ nodes: [], links: [] });
  });

  /**
   * REQ-VIZ-004.3: Handle Empty Input - undefined
   *
   * WHEN transformDiffToForcegraph receives undefined
   * THEN SHALL return { nodes: [], links: [] }
   *   AND SHALL NOT throw error
   */
  test('returns empty structure for undefined input', () => {
    expect(transformDiffToForcegraph(undefined)).toEqual({ nodes: [], links: [] });
  });

  /**
   * REQ-VIZ-004.3: Handle Empty Input - empty object
   *
   * WHEN transformDiffToForcegraph receives empty object
   * THEN SHALL return { nodes: [], links: [] }
   *   AND SHALL NOT throw error
   */
  test('returns empty structure for empty object input', () => {
    expect(transformDiffToForcegraph({} as ApiDiffVisualization)).toEqual({
      nodes: [],
      links: [],
    });
  });
});

// =============================================================================
// REQ-VIZ-002: Node Styling
// =============================================================================

describe('REQ-VIZ-002: Node Styling', () => {
  /**
   * REQ-VIZ-002.1: Node Color Mapping
   *
   * WHEN getNodeColorByChangeType is called
   * THEN SHALL return correct color for each change type
   */
  test('getNodeColorByChangeType returns correct colors', () => {
    expect(getNodeColorByChangeType('added')).toBe(CHANGE_TYPE_COLORS.added);
    expect(getNodeColorByChangeType('removed')).toBe(CHANGE_TYPE_COLORS.removed);
    expect(getNodeColorByChangeType('modified')).toBe(CHANGE_TYPE_COLORS.modified);
    expect(getNodeColorByChangeType('affected')).toBe(CHANGE_TYPE_COLORS.affected);
    expect(getNodeColorByChangeType(null)).toBe(CHANGE_TYPE_COLORS.unchanged);
  });

  /**
   * REQ-VIZ-002.2: Node Size by Change Type
   *
   * WHEN getNodeSizeByChangeType is called
   * THEN SHALL return 15 for changed nodes, 5 for unchanged
   */
  test('getNodeSizeByChangeType returns correct sizes', () => {
    expect(getNodeSizeByChangeType('added')).toBe(15);
    expect(getNodeSizeByChangeType('removed')).toBe(15);
    expect(getNodeSizeByChangeType('modified')).toBe(15);
    expect(getNodeSizeByChangeType('affected')).toBe(15);
    expect(getNodeSizeByChangeType(null)).toBe(5);
  });

  /**
   * REQ-VIZ-002.3: Node Labels
   *
   * WHEN getNodeLabelFormatted is called
   * THEN SHALL return "name (nodeType)" format
   */
  test('getNodeLabelFormatted returns correct format', () => {
    const node: GraphNode = {
      id: 'n1',
      name: 'handleAuth',
      nodeType: 'function',
      changeType: 'added',
    };

    expect(getNodeLabelFormatted(node)).toBe('handleAuth (function)');
  });
});
