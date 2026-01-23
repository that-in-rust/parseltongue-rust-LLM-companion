/**
 * Diff Graph Canvas View Component.
 *
 * REQ-VIZ-001: Graph Rendering
 * REQ-VIZ-002: Node Styling by Change Type
 * REQ-VIZ-003: Node Click Interactions
 *
 * Renders 3D force-directed graph visualization of dependency diff.
 */

import { useRef, useCallback } from 'react';
import ForceGraph3D from 'react-force-graph-3d';
import type { ForceGraphData, GraphNode } from '@/types/api';
import {
  getNodeColorByChangeType,
  getNodeSizeByChangeType,
  getNodeLabelFormatted,
} from '@/utils/transformDiffToForcegraph';

/**
 * Props for DiffGraphCanvasView component.
 */
export interface DiffGraphCanvasViewProps {
  graphData: ForceGraphData;
  onNodeClick?: (node: GraphNode) => void;
  onBackgroundClick?: () => void;
  className?: string;
}

/**
 * Diff Graph Canvas View component.
 *
 * REQ-VIZ-001: Graph Rendering
 * REQ-VIZ-002: Node Styling by Change Type
 * REQ-VIZ-003: Node Click Interactions
 */
export function DiffGraphCanvasView({
  graphData,
  onNodeClick,
  onBackgroundClick,
  className = '',
}: DiffGraphCanvasViewProps): JSX.Element {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const fgRef = useRef<any>(null);

  // REQ-VIZ-003.1: Handle node click with camera focus
  const handleNodeClick = useCallback(
    (node: GraphNode) => {
      if (fgRef.current && node.x !== undefined && node.y !== undefined && node.z !== undefined) {
        // REQ-VIZ-003.2: Animate camera to center on node
        const distance = 200;
        fgRef.current.cameraPosition(
          { x: node.x, y: node.y, z: node.z + distance },
          { x: node.x, y: node.y, z: node.z },
          1000
        );
      }
      onNodeClick?.(node);
    },
    [onNodeClick]
  );

  // REQ-VIZ-001.2: Empty Graph State
  if (graphData.nodes.length === 0) {
    return (
      <EmptyGraphState
        message="No graph data available"
        submessage="Select a workspace and enable watching to see changes"
      />
    );
  }

  // REQ-VIZ-002.1: Get node color based on change type
  const getNodeColor = useCallback((node: GraphNode) => {
    return getNodeColorByChangeType(node.changeType);
  }, []);

  // REQ-VIZ-002.2: Get node size based on change type
  const getNodeSize = useCallback((node: GraphNode) => {
    return getNodeSizeByChangeType(node.changeType);
  }, []);

  // REQ-VIZ-002.3: Format node label
  const getNodeLabel = useCallback((node: GraphNode) => {
    return getNodeLabelFormatted(node);
  }, []);

  return (
    <div
      className={`relative w-full h-full ${className}`}
      data-testid="diff-graph-canvas"
    >
      {/* REQ-VIZ-001.1: Render ForceGraph3D */}
      <ForceGraph3D
        ref={fgRef}
        graphData={graphData}
        nodeColor={getNodeColor}
        nodeVal={getNodeSize}
        nodeLabel={getNodeLabel}
        linkColor={() => '#4b5563'}
        linkWidth={1}
        linkOpacity={0.6}
        onNodeClick={handleNodeClick}
        onBackgroundClick={onBackgroundClick}
        controlType="orbit"
        backgroundColor="#111827"
      />

      {/* Color legend */}
      <ColorLegend />
    </div>
  );
}

/**
 * Props for ColorLegend component.
 */
export interface ColorLegendProps {
  className?: string;
}

/**
 * Color legend component showing change type colors.
 *
 * REQ-E2E-002.3: Color Legend Visibility
 */
export function ColorLegend({ className = '' }: ColorLegendProps): JSX.Element {
  const legendItems = [
    { label: 'Added', color: 'bg-green-500', testId: 'legend-added' },
    { label: 'Removed', color: 'bg-red-500', testId: 'legend-removed' },
    { label: 'Modified', color: 'bg-amber-500', testId: 'legend-modified' },
    { label: 'Affected', color: 'bg-blue-500', testId: 'legend-affected' },
    { label: 'Unchanged', color: 'bg-gray-500', testId: 'legend-unchanged' },
  ];

  return (
    <div
      className={`absolute bottom-4 right-4 bg-gray-800 border border-gray-700 rounded-lg p-3 ${className}`}
      data-testid="color-legend"
    >
      <div className="text-xs font-semibold text-gray-300 mb-2">Legend</div>
      <div className="space-y-1">
        {legendItems.map((item) => (
          <div
            key={item.label}
            className="flex items-center gap-2"
            data-testid={item.testId}
          >
            <div className={`w-3 h-3 rounded-full ${item.color}`} />
            <span className="text-xs text-gray-400">{item.label}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * Props for EmptyGraphState component.
 */
export interface EmptyGraphStateProps {
  message?: string;
  submessage?: string;
}

/**
 * Empty state displayed when no graph data available.
 *
 * REQ-VIZ-001.2: Empty Graph State
 */
export function EmptyGraphState({
  message = 'No graph data available',
  submessage = 'Select a workspace and enable watching to see changes',
}: EmptyGraphStateProps): JSX.Element {
  return (
    <div className="flex items-center justify-center w-full h-full bg-gray-900">
      <div className="text-center">
        <div className="text-lg text-gray-400 mb-2">{message}</div>
        <div className="text-sm text-gray-500">{submessage}</div>
      </div>
    </div>
  );
}
