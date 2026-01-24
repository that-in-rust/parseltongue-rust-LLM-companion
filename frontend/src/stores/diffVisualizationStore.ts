/**
 * Diff Visualization Store - Zustand store for graph visualization state.
 *
 * REQ-STORE-002: Diff Visualization Store
 * Manages graph data and selection state for 3D visualization.
 */

import { create } from 'zustand';
import type {
  ForceGraphData,
  GraphNode,
  DiffSummaryData,
  ApiDiffVisualization,
  WebSocketServerEvent,
} from '@/types/api';
import { transformDiffToForcegraph } from '@/utils/transformDiffToForcegraph';

/**
 * Diff visualization store state shape.
 */
export interface DiffVisualizationState {
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

/**
 * Initial empty graph data.
 */
const EMPTY_GRAPH_DATA: ForceGraphData = {
  nodes: [],
  links: [],
};

/**
 * Main diff visualization store.
 */
export const useDiffVisualizationStore = create<DiffVisualizationState>((set, get) => ({
  graphData: EMPTY_GRAPH_DATA,
  selectedNode: null,
  summary: null,
  isDiffInProgress: false,
  actions: {
    setGraphDataFromApi: (apiResponse: ApiDiffVisualization) => {
      const graphData = transformDiffToForcegraph(apiResponse);
      set({ graphData });
    },

    selectNodeById: (nodeId: string) => {
      const { graphData } = get();
      const selectedNode = graphData.nodes.find((node) => node.id === nodeId) || null;
      set({ selectedNode });
    },

    clearSelectedNode: () => {
      set({ selectedNode: null });
    },

    updateSummaryData: (summary: DiffSummaryData) => {
      set({ summary });
    },

    setDiffInProgress: (inProgress: boolean) => {
      set({ isDiffInProgress: inProgress });
    },

    applyEntityEvent: (event: WebSocketServerEvent) => {
      const { graphData } = get();

      if (event.event === 'entity_added') {
        // Add new node to graph
        const newNode: GraphNode = {
          id: event.entity_key,
          name: event.entity_key,
          nodeType: event.entity_type,
          changeType: 'added',
          filePath: event.file_path,
          lineStart: event.line_range?.start,
          lineEnd: event.line_range?.end,
        };
        set({
          graphData: {
            ...graphData,
            nodes: [...graphData.nodes, newNode],
          },
        });
      } else if (event.event === 'entity_removed') {
        // Mark node as removed
        const updatedNodes = graphData.nodes.map((node) =>
          node.id === event.entity_key ? { ...node, changeType: 'removed' as const } : node
        );
        set({
          graphData: {
            ...graphData,
            nodes: updatedNodes,
          },
        });
      } else if (event.event === 'entity_modified') {
        // Mark node as modified
        const updatedNodes = graphData.nodes.map((node) =>
          node.id === event.entity_key
            ? {
                ...node,
                changeType: 'modified' as const,
                lineStart: event.after_line_range?.start,
                lineEnd: event.after_line_range?.end,
              }
            : node
        );
        set({
          graphData: {
            ...graphData,
            nodes: updatedNodes,
          },
        });
      } else if (event.event === 'edge_added') {
        // Add new edge
        const newLink = {
          source: event.from_entity_key,
          target: event.to_entity_key,
          edgeType: event.edge_type,
        };
        set({
          graphData: {
            ...graphData,
            links: [...graphData.links, newLink],
          },
        });
      } else if (event.event === 'edge_removed') {
        // Remove edge
        const updatedLinks = graphData.links.filter(
          (link) =>
            !(
              link.source === event.from_entity_key &&
              link.target === event.to_entity_key &&
              link.edgeType === event.edge_type
            )
        );
        set({
          graphData: {
            ...graphData,
            links: updatedLinks,
          },
        });
      }
    },

    clearAllGraphData: () => {
      set({
        graphData: EMPTY_GRAPH_DATA,
        selectedNode: null,
        summary: null,
        isDiffInProgress: false,
      });
    },
  },
}));

// =============================================================================
// Selector Hooks
// =============================================================================

/**
 * Selector hook for graph data.
 */
export const useGraphData = (): ForceGraphData => {
  return useDiffVisualizationStore((state) => state.graphData);
};

/**
 * Selector hook for selected node.
 */
export const useSelectedNode = (): GraphNode | null => {
  return useDiffVisualizationStore((state) => state.selectedNode);
};

/**
 * Selector hook for diff summary.
 */
export const useDiffSummary = (): DiffSummaryData | null => {
  return useDiffVisualizationStore((state) => state.summary);
};

/**
 * Selector hook for diff in progress state.
 */
export const useIsDiffInProgress = (): boolean => {
  return useDiffVisualizationStore((state) => state.isDiffInProgress);
};

/**
 * Selector hook for actions.
 */
export const useDiffVisualizationActions = (): DiffVisualizationState['actions'] => {
  return useDiffVisualizationStore((state) => state.actions);
};
