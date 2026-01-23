/**
 * Workspace List Sidebar Component.
 *
 * REQ-SIDEBAR-001: Workspace List Display
 * REQ-SIDEBAR-002: Workspace Selection
 * REQ-SIDEBAR-003: Create Workspace
 * REQ-SIDEBAR-004: Watch Toggle Control
 *
 * Displays workspace list, selection, creation, and watch toggle controls.
 */

import { useEffect, useState } from 'react';
import type { WorkspaceMetadata } from '@/types/api';
import {
  useWorkspaceList,
  useSelectedWorkspaceId,
  useWorkspaceLoading,
  useWorkspaceError,
  useWorkspaceActions,
} from '@/stores/workspaceStore';

/**
 * Props for WorkspaceListSidebar component.
 */
export interface WorkspaceListSidebarProps {
  className?: string;
}

/**
 * Workspace List Sidebar component.
 *
 * REQ-SIDEBAR-001: Workspace List Display
 * REQ-SIDEBAR-002: Workspace Selection
 * REQ-SIDEBAR-003: Create Workspace
 */
export function WorkspaceListSidebar({
  className = '',
}: WorkspaceListSidebarProps): JSX.Element {
  const workspaces = useWorkspaceList();
  const selectedWorkspaceId = useSelectedWorkspaceId();
  const isLoading = useWorkspaceLoading();
  const error = useWorkspaceError();
  const { fetchWorkspaceListData, selectWorkspaceById, toggleWorkspaceWatchState, createWorkspaceFromPath } =
    useWorkspaceActions();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

  // REQ-SIDEBAR-001.1: Fetch workspace list on mount
  useEffect(() => {
    fetchWorkspaceListData();
  }, [fetchWorkspaceListData]);

  // REQ-SIDEBAR-001.1: Display loading indicator while fetching
  if (isLoading && workspaces.length === 0) {
    return (
      <aside
        className={`w-64 bg-gray-800 p-4 border-r border-gray-700 ${className}`}
        data-testid="workspace-list-sidebar"
      >
        <div data-testid="workspace-list-loading" className="text-gray-400">
          Loading workspaces...
        </div>
      </aside>
    );
  }

  // REQ-SIDEBAR-001.3: Display error state
  if (error && workspaces.length === 0) {
    return (
      <aside
        className={`w-64 bg-gray-800 p-4 border-r border-gray-700 ${className}`}
        data-testid="workspace-list-sidebar"
      >
        <div className="text-red-500 mb-2">Failed to load workspaces</div>
        <button
          onClick={() => fetchWorkspaceListData()}
          className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm"
        >
          Retry
        </button>
      </aside>
    );
  }

  // REQ-SIDEBAR-001.2: Display empty state
  if (workspaces.length === 0) {
    return (
      <aside
        className={`w-64 bg-gray-800 p-4 border-r border-gray-700 ${className}`}
        data-testid="workspace-list-sidebar"
      >
        <h2 className="text-lg font-semibold mb-4">Workspaces</h2>
        <div className="text-gray-400 mb-4">
          <div className="mb-2">No workspaces configured</div>
          <div className="text-sm">Add a workspace to get started</div>
        </div>
        <button
          onClick={() => setIsCreateDialogOpen(true)}
          data-testid="create-workspace-button"
          className="w-full py-2 bg-blue-600 hover:bg-blue-700 rounded"
        >
          Add Workspace
        </button>
        <CreateWorkspaceDialog
          isOpen={isCreateDialogOpen}
          onClose={() => setIsCreateDialogOpen(false)}
          onSubmit={async (path, name) => {
            await createWorkspaceFromPath(path, name);
            setIsCreateDialogOpen(false);
          }}
        />
      </aside>
    );
  }

  // REQ-SIDEBAR-001.1: Display workspace list
  return (
    <aside
      className={`w-64 bg-gray-800 border-r border-gray-700 flex flex-col h-full ${className}`}
      data-testid="workspace-list-sidebar"
    >
      <div className="p-4 pb-2">
        <div className="flex items-center justify-between mb-3">
          <h2 className="text-lg font-semibold">Workspaces</h2>
          <button
            onClick={() => setIsCreateDialogOpen(true)}
            data-testid="create-workspace-button"
            className="px-3 py-1 bg-blue-600 hover:bg-blue-700 rounded text-sm"
          >
            + Add
          </button>
        </div>
      </div>

      <ul className="space-y-2 flex-1 overflow-y-auto px-4 pb-4">
        {workspaces.map((workspace) => (
          <WorkspaceListItem
            key={workspace.workspace_identifier_value}
            workspace={workspace}
            isSelected={selectedWorkspaceId === workspace.workspace_identifier_value}
            onSelect={() => selectWorkspaceById(workspace.workspace_identifier_value)}
            onToggleWatch={(enabled) =>
              toggleWorkspaceWatchState(workspace.workspace_identifier_value, enabled)
            }
          />
        ))}
      </ul>

      <CreateWorkspaceDialog
        isOpen={isCreateDialogOpen}
        onClose={() => setIsCreateDialogOpen(false)}
        onSubmit={async (path, name) => {
          await createWorkspaceFromPath(path, name);
          setIsCreateDialogOpen(false);
        }}
      />
    </aside>
  );
}

/**
 * Props for WorkspaceListItem component.
 */
export interface WorkspaceListItemProps {
  workspace: WorkspaceMetadata;
  isSelected: boolean;
  onSelect: () => void;
  onToggleWatch: (enabled: boolean) => void;
}

/**
 * Individual workspace list item.
 *
 * REQ-SIDEBAR-002: Workspace Selection
 * REQ-SIDEBAR-004: Watch Toggle Control
 */
export function WorkspaceListItem({
  workspace,
  isSelected,
  onSelect,
  onToggleWatch,
}: WorkspaceListItemProps): JSX.Element {
  // REQ-SIDEBAR-002.2: Keyboard navigation support
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onSelect();
    }
  };

  return (
    <li
      data-testid={`workspace-item-${workspace.workspace_identifier_value}`}
      className={`p-3 rounded-lg cursor-pointer transition-all duration-150 border-2 ${
        isSelected
          ? 'bg-blue-600 border-blue-400 shadow-lg'
          : 'bg-gray-700/50 border-transparent hover:bg-gray-700 hover:border-gray-600'
      }`}
      onClick={onSelect}
      onKeyDown={handleKeyDown}
      tabIndex={0}
      role="button"
      aria-pressed={isSelected}
    >
      <div className="flex flex-col gap-1">
        <div className="font-medium text-white truncate">{workspace.workspace_display_name}</div>
        <div className="text-xs text-gray-400 truncate" title={workspace.source_directory_path_value}>
          {workspace.source_directory_path_value}
        </div>
        <div className="flex justify-between items-center mt-2">
          <button
            data-testid={`watch-toggle-${workspace.workspace_identifier_value}`}
            className={`px-2 py-1 rounded text-xs font-medium transition-colors ${
              workspace.watch_enabled_flag_status
                ? 'bg-green-600 hover:bg-green-700 text-white'
                : 'bg-gray-600 hover:bg-gray-500 text-gray-200'
            }`}
            onClick={(e) => {
              e.stopPropagation();
              onToggleWatch(!workspace.watch_enabled_flag_status);
            }}
            aria-pressed={workspace.watch_enabled_flag_status}
          >
            {workspace.watch_enabled_flag_status ? 'Watching' : 'Watch'}
          </button>
        </div>
      </div>
    </li>
  );
}

/**
 * Props for CreateWorkspaceDialog component.
 */
export interface CreateWorkspaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (path: string, name?: string) => Promise<void>;
}

/**
 * Create workspace dialog component.
 *
 * REQ-SIDEBAR-003: Create Workspace
 */
export function CreateWorkspaceDialog({
  isOpen,
  onClose,
  onSubmit,
}: CreateWorkspaceDialogProps): JSX.Element | null {
  const [path, setPath] = useState('');
  const [name, setName] = useState('');
  const [error, setError] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Reset form when dialog opens
  useEffect(() => {
    if (isOpen) {
      setPath('');
      setName('');
      setError('');
      setIsSubmitting(false);
    }
  }, [isOpen]);

  if (!isOpen) {
    return null;
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsSubmitting(true);

    try {
      await onSubmit(path, name || undefined);
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
    >
      <div
        className="bg-gray-800 rounded-lg p-6 w-96 border border-gray-700"
        onClick={(e) => e.stopPropagation()}
      >
        <h3 className="text-lg font-semibold mb-4">Add Workspace</h3>

        <form onSubmit={handleSubmit}>
          <div className="mb-4">
            <label htmlFor="workspace-path" className="block text-sm mb-1">
              Directory Path *
            </label>
            <input
              id="workspace-path"
              data-testid="workspace-path-input"
              type="text"
              value={path}
              onChange={(e) => setPath(e.target.value)}
              className={`w-full px-3 py-2 bg-gray-700 border rounded ${
                error ? 'border-red-500' : 'border-gray-600'
              } focus:outline-none focus:border-blue-500`}
              placeholder="/path/to/project"
              required
              autoFocus
              disabled={isSubmitting}
            />
          </div>

          <div className="mb-4">
            <label htmlFor="workspace-name" className="block text-sm mb-1">
              Display Name (optional)
            </label>
            <input
              id="workspace-name"
              data-testid="workspace-name-input"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded focus:outline-none focus:border-blue-500"
              placeholder="My Project"
              disabled={isSubmitting}
            />
          </div>

          {error && (
            <div className="mb-4 text-sm text-red-500" data-testid="create-error-message">
              {error}
            </div>
          )}

          <div className="flex gap-2 justify-end">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded"
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              type="submit"
              data-testid="confirm-create-button"
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded disabled:opacity-50 disabled:cursor-not-allowed"
              disabled={!path || isSubmitting}
            >
              {isSubmitting ? 'Creating...' : 'Create'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
