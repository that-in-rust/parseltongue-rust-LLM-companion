/**
 * Transform Diff to Forcegraph - Data transformation utilities.
 *
 * REQ-VIZ-004: Data Transformation
 * Transforms API response format to react-force-graph-3d expected format.
 */

import type { ApiDiffVisualization, ForceGraphData, ChangeType, GraphNode } from '@/types/api';
import { getChangeTypeColor } from '@/types/api';

/**
 * Transforms API diff visualization response to ForceGraph format.
 *
 * REQ-VIZ-004.1: Transform API Response to ForceGraph Format
 *
 * WHEN transformDiffToForcegraph receives API response
 * THEN SHALL return ForceGraphData with transformed nodes and links
 */
export function transformDiffToForcegraph(
  apiResponse: ApiDiffVisualization | null | undefined
): ForceGraphData {
  // REQ-VIZ-004.3: Handle null/undefined input
  if (!apiResponse || !apiResponse.nodes || !apiResponse.edges) {
    return { nodes: [], links: [] };
  }

  // REQ-VIZ-004.1: Transform nodes from API format to ForceGraph format
  const nodes: GraphNode[] = apiResponse.nodes.map((node) => ({
    id: node.id,
    name: node.label,
    nodeType: node.node_type,
    changeType: node.change_type,
    filePath: node.file_path,
    lineStart: node.line_start,
    lineEnd: node.line_end,
  }));

  // REQ-VIZ-004.1: Transform edges from API format to ForceGraph format
  const links = apiResponse.edges.map((edge) => ({
    source: edge.source,
    target: edge.target,
    edgeType: edge.edge_type,
  }));

  return { nodes, links };
}

/**
 * Gets node color based on change type.
 *
 * REQ-VIZ-002.1: Node Color Mapping
 *
 * WHEN rendering graph nodes
 * THEN SHALL apply color based on node.changeType
 */
export function getNodeColorByChangeType(changeType: ChangeType): string {
  return getChangeTypeColor(changeType);
}

/**
 * Gets node size based on change type.
 *
 * REQ-VIZ-002.2: Node Size by Change Type
 *
 * WHEN rendering graph nodes
 * THEN SHALL apply size based on node.changeType:
 *   - Changed nodes (added/removed/modified/affected): val = 15
 *   - Unchanged nodes (null): val = 5
 */
export function getNodeSizeByChangeType(changeType: ChangeType): number {
  return changeType !== null ? 15 : 5;
}

/**
 * Formats node label for display.
 *
 * REQ-VIZ-002.3: Node Labels
 *
 * WHEN rendering graph nodes
 * THEN SHALL display label showing: name (nodeType)
 */
export function getNodeLabelFormatted(node: GraphNode): string {
  return `${node.name} (${node.nodeType})`;
}
