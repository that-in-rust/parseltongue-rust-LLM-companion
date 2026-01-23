/**
 * Entity Detail Panel Component
 *
 * REQ-DETAIL-001: EntityDetailPanel Component
 *
 * Slide-out panel displaying selected node details including:
 * - Entity identity (name, type, change type badge)
 * - File location with line numbers
 * - Incoming and outgoing dependencies
 */

import { useEffect, useMemo } from 'react';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';
import type { GraphNode, GraphLink, ChangeType } from '@/types/api';

/**
 * Grouped dependencies by edge type.
 */
interface DependencyGroup {
  edgeType: string;
  dependencies: Array<{
    node: GraphNode;
    link: GraphLink;
  }>;
}

/**
 * Get badge classes for change type.
 */
function getChangeTypeBadgeClasses(changeType: ChangeType): string {
  const baseClasses = 'px-2 py-1 rounded-md text-xs font-medium border';

  switch (changeType) {
    case 'added':
      return `${baseClasses} bg-green-500/20 text-green-400 border-green-500/50`;
    case 'removed':
      return `${baseClasses} bg-red-500/20 text-red-400 border-red-500/50`;
    case 'modified':
      return `${baseClasses} bg-amber-500/20 text-amber-400 border-amber-500/50`;
    case 'affected':
      return `${baseClasses} bg-blue-500/20 text-blue-400 border-blue-500/50`;
    default:
      return `${baseClasses} bg-gray-500/20 text-gray-400 border-gray-500/50`;
  }
}

/**
 * Get display label for change type.
 */
function getChangeTypeLabel(changeType: ChangeType): string {
  if (changeType === null) {
    return 'Unchanged';
  }
  return changeType.charAt(0).toUpperCase() + changeType.slice(1);
}

/**
 * Format file location with line numbers.
 */
function formatFileLocationText(node: GraphNode): string {
  if (!node.filePath) {
    return '';
  }

  let location = node.filePath;

  if (node.lineStart !== undefined) {
    if (node.lineEnd !== undefined && node.lineEnd !== node.lineStart) {
      location += `:${node.lineStart}-${node.lineEnd}`;
    } else {
      location += `:${node.lineStart}`;
    }
  }

  return location;
}

/**
 * Group dependencies by edge type.
 */
function groupDependenciesByEdgeType(
  dependencies: Array<{ node: GraphNode; link: GraphLink }>
): DependencyGroup[] {
  const groups = new Map<string, DependencyGroup>();

  for (const dep of dependencies) {
    const edgeType = dep.link.edgeType;
    if (!groups.has(edgeType)) {
      groups.set(edgeType, {
        edgeType,
        dependencies: [],
      });
    }
    groups.get(edgeType)!.dependencies.push(dep);
  }

  return Array.from(groups.values());
}

/**
 * Get border color class for change type (left border for incoming).
 */
function getBorderColorClassLeft(changeType: ChangeType): string {
  switch (changeType) {
    case 'added':
      return 'border-l-2 border-l-green-500';
    case 'removed':
      return 'border-l-2 border-l-red-500';
    case 'modified':
      return 'border-l-2 border-l-amber-500';
    case 'affected':
      return 'border-l-2 border-l-blue-500';
    default:
      return 'border-l-2 border-l-gray-500';
  }
}

/**
 * Get border color class for change type (right border for outgoing).
 */
function getBorderColorClassRight(changeType: ChangeType): string {
  switch (changeType) {
    case 'added':
      return 'border-r-2 border-r-green-500';
    case 'removed':
      return 'border-r-2 border-r-red-500';
    case 'modified':
      return 'border-r-2 border-r-amber-500';
    case 'affected':
      return 'border-r-2 border-r-blue-500';
    default:
      return 'border-r-2 border-r-gray-500';
  }
}

/**
 * EntityDetailPanel - Displays detailed information about a selected graph node.
 */
export function EntityDetailPanel(): JSX.Element {
  const selectedNode = useDiffVisualizationStore((state) => state.selectedNode);
  const graphData = useDiffVisualizationStore((state) => state.graphData);
  const clearSelectedNode = useDiffVisualizationStore(
    (state) => state.actions.clearSelectedNode
  );
  const selectNodeById = useDiffVisualizationStore((state) => state.actions.selectNodeById);

  // Compute incoming and outgoing dependencies
  const { incomingDeps, outgoingDeps, nodeNotFound } = useMemo(() => {
    if (!selectedNode) {
      return { incomingDeps: [], outgoingDeps: [], nodeNotFound: false };
    }

    // Check if selected node exists in current graph
    const nodeExists = graphData.nodes.some((n) => n.id === selectedNode.id);
    if (!nodeExists) {
      console.warn(
        `EntityDetailPanel: Selected node "${selectedNode.id}" not found in current graph data.`
      );
      return { incomingDeps: [], outgoingDeps: [], nodeNotFound: true };
    }

    const incoming: Array<{ node: GraphNode; link: GraphLink }> = [];
    const outgoing: Array<{ node: GraphNode; link: GraphLink }> = [];

    for (const link of graphData.links) {
      // Validate link data
      if (!link.source || !link.target) {
        console.warn(`EntityDetailPanel: Encountered malformed link data (source: ${link.source}, target: ${link.target}, edgeType: ${link.edgeType})`);
        continue;
      }

      // Incoming: where this node is the target
      if (link.target === selectedNode.id) {
        const sourceNode = graphData.nodes.find((n) => n.id === link.source);
        if (sourceNode) {
          incoming.push({ node: sourceNode, link });
        }
      }

      // Outgoing: where this node is the source
      if (link.source === selectedNode.id) {
        const targetNode = graphData.nodes.find((n) => n.id === link.target);
        if (targetNode) {
          outgoing.push({ node: targetNode, link });
        }
      }
    }

    return { incomingDeps: incoming, outgoingDeps: outgoing, nodeNotFound: false };
  }, [selectedNode, graphData]);

  // Handle Escape key
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && selectedNode) {
        clearSelectedNode();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [selectedNode, clearSelectedNode]);

  const isVisible = selectedNode !== null;

  const handleCloseButtonClick = () => {
    clearSelectedNode();
  };

  const handleDependencyItemClick = (nodeId: string) => {
    selectNodeById(nodeId);
  };

  const fileLocation = selectedNode ? formatFileLocationText(selectedNode) : '';

  // Group dependencies
  const incomingGroups = useMemo(() => groupDependenciesByEdgeType(incomingDeps), [incomingDeps]);
  const outgoingGroups = useMemo(() => groupDependenciesByEdgeType(outgoingDeps), [outgoingDeps]);

  return (
    <div
      data-testid="entity-detail-panel"
      role="complementary"
      aria-label="Entity details panel"
      style={{ visibility: isVisible ? 'visible' : 'hidden' }}
      className={`fixed top-0 bottom-0 right-0 z-50 bg-gray-800 border-l border-gray-700 overflow-y-auto transition-transform duration-200 ease-out w-full w-80 max-h-[60vh] md:w-80 md:max-h-full md:right-0 ${
        isVisible ? 'translate-x-0' : 'translate-x-full'
      }`}
    >
      {selectedNode && !nodeNotFound && (
        <div className="p-4">
          {/* Close button */}
          <div className="flex justify-end mb-2">
            <button
              onClick={handleCloseButtonClick}
              aria-label="Close entity details"
              className="p-1 rounded hover:bg-gray-700"
            >
              <svg
                className="w-5 h-5 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          {/* Mobile drag handle */}
          <div
            data-testid="drag-handle"
            className="md:hidden w-12 h-1 bg-gray-600 rounded-full mx-auto mb-4"
          />

          {/* Entity identity */}
          <div className="mb-4">
            <h2 data-testid="entity-name" className="text-lg font-semibold text-white">
              {selectedNode.name}
            </h2>
            <p data-testid="entity-type" className="text-sm text-gray-400">
              ({selectedNode.nodeType})
            </p>
            <div className="mt-2">
              <span
                data-testid="change-type-badge"
                className={getChangeTypeBadgeClasses(selectedNode.changeType)}
              >
                {getChangeTypeLabel(selectedNode.changeType)}
              </span>
            </div>
          </div>

          {/* File location */}
          <div className="mb-4">
            <label data-testid="file-label" className="text-xs text-gray-500 uppercase block mb-1">
              File
            </label>
            {selectedNode.filePath ? (
              <p
                data-testid="file-location"
                className="text-sm font-mono text-gray-300 truncate"
                title={fileLocation}
              >
                {fileLocation}
              </p>
            ) : (
              <p className="text-sm text-gray-500 italic">Location unknown</p>
            )}
          </div>

          {/* Incoming dependencies */}
          <div className="mb-4">
            <div className="flex items-center gap-2 mb-2">
              <h3 data-testid="incoming-deps-header" className="text-sm font-semibold text-white">
                Incoming
              </h3>
              <span
                data-testid="incoming-deps-count"
                className="px-2 py-0.5 text-xs bg-gray-700 rounded-full"
              >
                {incomingDeps.length}
              </span>
            </div>
            {incomingDeps.length === 0 ? (
              <p className="text-sm text-gray-500">No incoming dependencies</p>
            ) : (
              <div className="space-y-3">
                {incomingGroups.map((group) => (
                  <div key={group.edgeType} data-testid={`incoming-group-${group.edgeType}`}>
                    <h4 className="text-xs text-gray-400 uppercase mb-1">{group.edgeType}</h4>
                    <ul className="space-y-1">
                      {group.dependencies.slice(0, 5).map(({ node }) => (
                        <li key={node.id}>
                          <button
                            onClick={() => handleDependencyItemClick(node.id)}
                            className={`text-sm text-gray-300 hover:text-white w-full text-left px-2 py-1 rounded hover:bg-gray-700 ${getBorderColorClassLeft(
                              node.changeType
                            )}`}
                          >
                            {node.name}
                          </button>
                        </li>
                      ))}
                      {group.dependencies.length > 5 && (
                        <li>
                          <span className="text-xs text-blue-400">
                            +{group.dependencies.length - 5} more
                          </span>
                        </li>
                      )}
                    </ul>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Outgoing dependencies */}
          <div className="mb-4">
            <div className="flex items-center gap-2 mb-2">
              <h3 data-testid="outgoing-deps-header" className="text-sm font-semibold text-white">
                Outgoing
              </h3>
              <span
                data-testid="outgoing-deps-count"
                className="px-2 py-0.5 text-xs bg-gray-700 rounded-full"
              >
                {outgoingDeps.length}
              </span>
            </div>
            {outgoingDeps.length === 0 ? (
              <p className="text-sm text-gray-500">No outgoing dependencies</p>
            ) : (
              <div className="space-y-3">
                {outgoingGroups.map((group) => (
                  <div key={group.edgeType} data-testid={`outgoing-group-${group.edgeType}`}>
                    <h4 className="text-xs text-gray-400 uppercase mb-1">{group.edgeType}</h4>
                    <ul className="space-y-1">
                      {group.dependencies.slice(0, 5).map(({ node }) => (
                        <li key={node.id}>
                          <button
                            onClick={() => handleDependencyItemClick(node.id)}
                            className={`text-sm text-gray-300 hover:text-white w-full text-left px-2 py-1 rounded hover:bg-gray-700 ${getBorderColorClassRight(
                              node.changeType
                            )}`}
                          >
                            {node.name}
                          </button>
                        </li>
                      ))}
                      {group.dependencies.length > 5 && (
                        <li>
                          <span className="text-xs text-blue-400">
                            +{group.dependencies.length - 5} more
                          </span>
                        </li>
                      )}
                    </ul>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Error state: node not found */}
      {selectedNode && nodeNotFound && (
        <div className="p-4">
          <div className="flex justify-end mb-2">
            <button
              onClick={handleCloseButtonClick}
              aria-label="Close entity details"
              className="p-1 rounded hover:bg-gray-700"
            >
              <svg
                className="w-5 h-5 text-gray-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
          <p className="text-sm text-red-400">Entity not found in current graph</p>
        </div>
      )}
    </div>
  );
}

export default EntityDetailPanel;
