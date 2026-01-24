/**
 * Workspace Store Tests
 *
 * REQ-STORE-001: Workspace Store
 * Tests for workspace state management.
 */

import { describe, test, expect, beforeEach } from 'vitest';
import { renderHook } from '@testing-library/react';
import {
  useWorkspaceStore,
  useWorkspaceList,
  useSelectedWorkspaceId,
  useWorkspaceLoading,
  useWorkspaceError,
} from '../workspaceStore';
import type { WorkspaceMetadata } from '@/types/api';

// =============================================================================
// Test Fixtures
// =============================================================================

const mockWorkspace: WorkspaceMetadata = {
  workspace_identifier_value: 'ws_1',
  workspace_display_name: 'Test Project',
  source_directory_path_value: '/path/to/project',
  base_database_path_value: '/path/to/base.db',
  live_database_path_value: '/path/to/live.db',
  watch_enabled_flag_status: false,
  created_timestamp_utc_value: '2026-01-23T00:00:00Z',
  last_indexed_timestamp_option: null,
};

// =============================================================================
// REQ-STORE-001: Workspace Store
// =============================================================================

describe('REQ-STORE-001: Workspace Store', () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useWorkspaceStore.setState({
      workspaces: [],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
  });

  /**
   * REQ-STORE-001.1: Store Shape
   *
   * WHEN workspaceStore is created
   * THEN SHALL have correct initial shape with:
   *   - workspaces: []
   *   - selectedWorkspaceId: null
   *   - isLoading: false
   *   - error: null
   *   - actions object with required methods
   */
  test('store has correct initial shape', () => {
    const state = useWorkspaceStore.getState();

    expect(state).toEqual(
      expect.objectContaining({
        workspaces: [],
        selectedWorkspaceId: null,
        isLoading: false,
        error: null,
      })
    );
    expect(state.actions).toBeDefined();
    expect(typeof state.actions.fetchWorkspaceListData).toBe('function');
    expect(typeof state.actions.selectWorkspaceById).toBe('function');
    expect(typeof state.actions.toggleWorkspaceWatchState).toBe('function');
    expect(typeof state.actions.createWorkspaceFromPath).toBe('function');
    expect(typeof state.actions.clearSelectedWorkspace).toBe('function');
  });

  /**
   * REQ-STORE-001.2: Selector Hooks - useWorkspaceList
   *
   * WHEN useWorkspaceList hook is called
   * THEN SHALL return only the workspaces array from store
   */
  test('useWorkspaceList selector returns workspaces array', () => {
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace],
    });

    const { result } = renderHook(() => useWorkspaceList());

    expect(result.current).toEqual([mockWorkspace]);
  });

  /**
   * REQ-STORE-001.2: Selector Hooks - useSelectedWorkspaceId
   *
   * WHEN useSelectedWorkspaceId hook is called
   * THEN SHALL return only the selectedWorkspaceId from store
   */
  test('useSelectedWorkspaceId selector returns selected ID', () => {
    useWorkspaceStore.setState({
      selectedWorkspaceId: 'ws_123',
    });

    const { result } = renderHook(() => useSelectedWorkspaceId());

    expect(result.current).toBe('ws_123');
  });

  /**
   * REQ-STORE-001.2: Selector Hooks - useWorkspaceLoading
   *
   * WHEN useWorkspaceLoading hook is called
   * THEN SHALL return only the isLoading state from store
   */
  test('useWorkspaceLoading selector returns loading state', () => {
    useWorkspaceStore.setState({
      isLoading: true,
    });

    const { result } = renderHook(() => useWorkspaceLoading());

    expect(result.current).toBe(true);
  });

  /**
   * REQ-STORE-001.2: Selector Hooks - useWorkspaceError
   *
   * WHEN useWorkspaceError hook is called
   * THEN SHALL return only the error state from store
   */
  test('useWorkspaceError selector returns error state', () => {
    useWorkspaceStore.setState({
      error: 'Test error message',
    });

    const { result } = renderHook(() => useWorkspaceError());

    expect(result.current).toBe('Test error message');
  });

  /**
   * REQ-STORE-001.3: selectWorkspaceById updates immediately
   *
   * WHEN selectWorkspaceById action is called
   * THEN SHALL immediately update selectedWorkspaceId in store
   */
  test('selectWorkspaceById updates store immediately', () => {
    const { actions } = useWorkspaceStore.getState();

    actions.selectWorkspaceById('ws_123');

    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_123');
  });

  /**
   * REQ-STORE-001.3: clearSelectedWorkspace clears selection
   *
   * WHEN clearSelectedWorkspace action is called
   * THEN SHALL set selectedWorkspaceId to null
   */
  test('clearSelectedWorkspace clears the selection', () => {
    useWorkspaceStore.setState({ selectedWorkspaceId: 'ws_123' });
    const { actions } = useWorkspaceStore.getState();

    actions.clearSelectedWorkspace();

    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBeNull();
  });
});
