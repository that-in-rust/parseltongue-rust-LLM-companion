/**
 * Workspace Store - Zustand store for workspace state management.
 *
 * REQ-STORE-001: Workspace Store
 * Manages global state for workspace list and selection.
 */

import { create } from 'zustand';
import type { WorkspaceMetadata } from '@/types/api';

/**
 * Workspace store state shape.
 */
export interface WorkspaceState {
  workspaces: WorkspaceMetadata[];
  selectedWorkspaceId: string | null;
  isLoading: boolean;
  error: string | null;
  actions: {
    fetchWorkspaceListData: () => Promise<void>;
    selectWorkspaceById: (id: string) => void;
    toggleWorkspaceWatchState: (id: string, enabled: boolean) => Promise<void>;
    createWorkspaceFromPath: (path: string, name?: string) => Promise<void>;
    clearSelectedWorkspace: () => void;
  };
}

/**
 * Main workspace store.
 */
export const useWorkspaceStore = create<WorkspaceState>((set, get) => ({
  workspaces: [],
  selectedWorkspaceId: null,
  isLoading: false,
  error: null,
  actions: {
    fetchWorkspaceListData: async () => {
      set({ isLoading: true, error: null });
      try {
        const response = await fetch('/workspace-list-all');
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        const data = await response.json();
        set({ workspaces: data.workspaces, isLoading: false });
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        set({ error: errorMessage, isLoading: false });
      }
    },

    selectWorkspaceById: (id: string) => {
      set({ selectedWorkspaceId: id });
    },

    toggleWorkspaceWatchState: async (id: string, enabled: boolean) => {
      try {
        const response = await fetch('/workspace-watch-toggle', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            workspace_identifier_target_value: id,
            watch_enabled_desired_state: enabled,
          }),
        });

        if (!response.ok) {
          throw new Error(`Failed to toggle watch state: ${response.statusText}`);
        }

        // Refresh workspace list to get updated state
        await get().actions.fetchWorkspaceListData();
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        set({ error: errorMessage });
        throw error;
      }
    },

    createWorkspaceFromPath: async (path: string, name?: string) => {
      try {
        const response = await fetch('/workspace-create-from-path', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            source_path_directory_value: path,
            workspace_display_name_option: name,
          }),
        });

        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.error || 'Failed to create workspace');
        }

        // Refresh workspace list and select the new workspace
        await get().actions.fetchWorkspaceListData();
        const data = await response.json();
        if (data.workspace) {
          set({ selectedWorkspaceId: data.workspace.workspace_identifier_value });
        }
      } catch (error) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        set({ error: errorMessage });
        throw error;
      }
    },

    clearSelectedWorkspace: () => {
      set({ selectedWorkspaceId: null });
    },
  },
}));

// =============================================================================
// Selector Hooks (REQ-STORE-001.2)
// =============================================================================

/**
 * Selector hook for workspace list.
 */
export const useWorkspaceList = (): WorkspaceMetadata[] => {
  return useWorkspaceStore((state) => state.workspaces);
};

/**
 * Selector hook for selected workspace ID.
 */
export const useSelectedWorkspaceId = (): string | null => {
  return useWorkspaceStore((state) => state.selectedWorkspaceId);
};

/**
 * Selector hook for loading state.
 */
export const useWorkspaceLoading = (): boolean => {
  return useWorkspaceStore((state) => state.isLoading);
};

/**
 * Selector hook for error state.
 */
export const useWorkspaceError = (): string | null => {
  return useWorkspaceStore((state) => state.error);
};

/**
 * Selector hook for actions.
 */
export const useWorkspaceActions = (): WorkspaceState['actions'] => {
  return useWorkspaceStore((state) => state.actions);
};
