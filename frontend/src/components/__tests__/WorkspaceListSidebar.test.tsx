/**
 * Workspace List Sidebar Tests
 *
 * REQ-SIDEBAR-001: Workspace List Display
 * REQ-SIDEBAR-002: Workspace Selection
 * REQ-SIDEBAR-003: Create Workspace
 * REQ-SIDEBAR-004: Watch Toggle Control
 *
 * Tests for workspace sidebar component functionality.
 */

import { describe, test, expect, beforeEach, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {
  WorkspaceListSidebar,
  WorkspaceListItem,
  CreateWorkspaceDialog,
} from '../WorkspaceListSidebar';
import { useWorkspaceStore } from '@/stores/workspaceStore';
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

const mockWorkspaceWatching: WorkspaceMetadata = {
  ...mockWorkspace,
  workspace_identifier_value: 'ws_2',
  workspace_display_name: 'Watching Project',
  watch_enabled_flag_status: true,
};

// =============================================================================
// REQ-SIDEBAR-001: Workspace List Display
// =============================================================================

// Helper function to get initial store state
const getInitialWorkspaceState = () => ({
  workspaces: [] as WorkspaceMetadata[],
  selectedWorkspaceId: null as string | null,
  isLoading: false,
  error: null as string | null,
});

describe('REQ-SIDEBAR-001: Workspace List Display', () => {
  beforeEach(() => {
    useWorkspaceStore.setState(getInitialWorkspaceState());
  });

  /**
   * REQ-SIDEBAR-001.1: Render Workspace List on Mount
   *
   * WHEN WorkspaceListSidebar component mounts
   * THEN SHALL call GET /workspace-list-all endpoint
   *   AND SHALL display loading indicator while fetching
   *   AND SHALL render workspace list within 500ms of response
   */
  test('renders workspace list after successful fetch', async () => {
    // Mock API response
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace, mockWorkspaceWatching],
      isLoading: false,
    });

    render(<WorkspaceListSidebar />);

    await waitFor(() => {
      expect(screen.getByText('Test Project')).toBeInTheDocument();
      expect(screen.getByText('Watching Project')).toBeInTheDocument();
    });
  });

  /**
   * REQ-SIDEBAR-001.1: Display loading indicator
   *
   * WHEN WorkspaceListSidebar is fetching data
   * THEN SHALL display loading indicator
   */
  test('displays loading indicator while fetching', () => {
    useWorkspaceStore.setState({ isLoading: true, workspaces: [] });

    render(<WorkspaceListSidebar />);

    expect(screen.getByTestId('workspace-list-loading')).toBeInTheDocument();
  });

  /**
   * REQ-SIDEBAR-001.2: Empty State Display
   *
   * WHEN WorkspaceListSidebar receives empty workspaces array
   * THEN SHALL display empty state message
   *   AND SHALL display "Add Workspace" button prominently
   */
  test('displays empty state when no workspaces exist', async () => {
    useWorkspaceStore.setState({ workspaces: [], isLoading: false, error: null });

    render(<WorkspaceListSidebar />);

    await waitFor(() => {
      expect(screen.getByText('No workspaces configured')).toBeInTheDocument();
      expect(screen.getByText('Add Workspace')).toBeInTheDocument();
    });
  });

  /**
   * REQ-SIDEBAR-001.3: Error State Display
   *
   * WHEN GET /workspace-list-all request fails
   * THEN SHALL display error message with retry button
   */
  test('displays error state when fetch fails', async () => {
    // Make fetch fail
    vi.spyOn(global, 'fetch').mockImplementation(async () => ({
      ok: false,
      statusText: 'Server Error',
      json: async () => ({}),
    }) as Response);

    useWorkspaceStore.setState({
      workspaces: [],
      error: 'Failed to load workspaces',
      isLoading: false,
    });

    render(<WorkspaceListSidebar />);

    await waitFor(() => {
      expect(screen.getByText('Failed to load workspaces')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument();
    });
  });
});

// =============================================================================
// REQ-SIDEBAR-002: Workspace Selection
// =============================================================================

describe('REQ-SIDEBAR-002: Workspace Selection', () => {
  beforeEach(() => {
    // Mock fetchWorkspaceListData to return our test workspaces
    vi.spyOn(global, 'fetch').mockImplementation(async () => ({
      ok: true,
      json: async () => ({
        workspaces: [mockWorkspace, mockWorkspaceWatching],
        success: true
      }),
    }) as Response);
    useWorkspaceStore.setState({
      workspaces: [mockWorkspace, mockWorkspaceWatching],
      selectedWorkspaceId: null,
      isLoading: false,
      error: null,
    });
  });

  /**
   * REQ-SIDEBAR-002.1: Click to Select Workspace
   *
   * WHEN user clicks on a workspace item
   * THEN SHALL update selectedWorkspaceId in store
   *   AND SHALL apply selected visual styling (bg-blue-600)
   */
  test('clicking workspace updates store and applies styling', async () => {
    render(<WorkspaceListSidebar />);

    await waitFor(() => {
      expect(screen.getByTestId('workspace-item-ws_1')).toBeInTheDocument();
    });

    fireEvent.click(screen.getByTestId('workspace-item-ws_1'));

    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_1');
    expect(screen.getByTestId('workspace-item-ws_1')).toHaveClass('bg-blue-600');
  });

  /**
   * REQ-SIDEBAR-002.2: Keyboard Navigation Support
   *
   * WHEN user presses Enter on focused workspace item
   * THEN SHALL select that workspace
   */
  test('keyboard Enter selects workspace', async () => {
    render(<WorkspaceListSidebar />);

    const item = await screen.findByTestId('workspace-item-ws_1');
    item.focus();

    fireEvent.keyDown(item, { key: 'Enter' });

    expect(useWorkspaceStore.getState().selectedWorkspaceId).toBe('ws_1');
  });

  /**
   * REQ-SIDEBAR-002.3: Only One Workspace Selected
   *
   * WHEN user selects a new workspace with another already selected
   * THEN SHALL deselect previous workspace
   */
  test('selecting new workspace deselects previous', async () => {
    useWorkspaceStore.setState({ selectedWorkspaceId: 'ws_1' });

    render(<WorkspaceListSidebar />);

    fireEvent.click(await screen.findByTestId('workspace-item-ws_2'));

    expect(screen.getByTestId('workspace-item-ws_1')).not.toHaveClass('bg-blue-600');
    expect(screen.getByTestId('workspace-item-ws_2')).toHaveClass('bg-blue-600');
  });
});

// =============================================================================
// REQ-SIDEBAR-003: Create Workspace
// =============================================================================

describe('REQ-SIDEBAR-003: Create Workspace', () => {
  /**
   * REQ-SIDEBAR-003.1: Open Create Workspace Dialog
   *
   * WHEN user clicks "Add Workspace" button
   * THEN SHALL open modal dialog with path and name inputs
   */
  test.skip('clicking Add Workspace opens dialog', async () => {
    render(<WorkspaceListSidebar />);

    fireEvent.click(screen.getByTestId('create-workspace-button'));

    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByLabelText(/directory path/i)).toHaveFocus();
  });

  /**
   * REQ-SIDEBAR-003.2: Submit Create Request
   *
   * WHEN user enters valid path and clicks "Create"
   * THEN SHALL call POST /workspace-create-from-path with payload
   */
  test.skip('submitting form calls API with correct payload', async () => {
    const user = userEvent.setup();
    const mockCreate = vi.fn();

    render(<WorkspaceListSidebar />);
    fireEvent.click(screen.getByTestId('create-workspace-button'));

    await user.type(screen.getByLabelText(/directory path/i), '/path/to/project');
    await user.type(screen.getByLabelText(/display name/i), 'My Project');
    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(mockCreate).toHaveBeenCalledWith({
        source_path_directory_value: '/path/to/project',
        workspace_display_name_option: 'My Project',
      });
    });
  });

  /**
   * REQ-SIDEBAR-003.4: Handle Create Error
   *
   * WHEN POST /workspace-create-from-path returns error
   * THEN SHALL display error message below path input
   *   AND SHALL NOT close the dialog
   */
  test.skip('displays validation error when path not found', async () => {
    const user = userEvent.setup();

    render(<WorkspaceListSidebar />);
    fireEvent.click(screen.getByTestId('create-workspace-button'));

    await user.type(screen.getByLabelText(/directory path/i), '/invalid/path');
    fireEvent.click(screen.getByRole('button', { name: /create/i }));

    await waitFor(() => {
      expect(screen.getByText(/path does not exist/i)).toBeInTheDocument();
      expect(screen.getByRole('dialog')).toBeInTheDocument(); // Still open
    });
  });
});

// =============================================================================
// REQ-SIDEBAR-004: Watch Toggle Control
// =============================================================================

describe('REQ-SIDEBAR-004: Watch Toggle Control', () => {
  /**
   * REQ-SIDEBAR-004.1: Display Watch Toggle - Enabled State
   *
   * WHEN rendering a workspace with watch_enabled_flag_status = true
   * THEN SHALL display toggle with green background and "Watching" text
   */
  test('displays correct visual state for watch enabled', () => {
    render(
      <WorkspaceListItem
        workspace={mockWorkspaceWatching}
        isSelected={false}
        onSelect={() => {}}
        onToggleWatch={() => {}}
      />
    );

    const toggle = screen.getByTestId(`watch-toggle-${mockWorkspaceWatching.workspace_identifier_value}`);
    expect(toggle).toHaveClass('bg-green-600');
    expect(toggle).toHaveTextContent('Watching');
    expect(toggle).toHaveAttribute('aria-pressed', 'true');
  });

  /**
   * REQ-SIDEBAR-004.1: Display Watch Toggle - Disabled State
   *
   * WHEN rendering a workspace with watch_enabled_flag_status = false
   * THEN SHALL display toggle with gray background and "Watch" text
   */
  test('displays correct visual state for watch disabled', () => {
    render(
      <WorkspaceListItem
        workspace={mockWorkspace}
        isSelected={false}
        onSelect={() => {}}
        onToggleWatch={() => {}}
      />
    );

    const toggle = screen.getByTestId(`watch-toggle-${mockWorkspace.workspace_identifier_value}`);
    expect(toggle).toHaveClass('bg-gray-600');
    expect(toggle).toHaveTextContent('Watch');
    expect(toggle).toHaveAttribute('aria-pressed', 'false');
  });

  /**
   * REQ-SIDEBAR-004.2: Toggle Watch State
   *
   * WHEN user clicks watch toggle button
   * THEN SHALL call onToggleWatch with inverted state
   */
  test('calls onToggleWatch when toggle clicked', async () => {
    const mockToggle = vi.fn();

    render(
      <WorkspaceListItem
        workspace={mockWorkspace}
        isSelected={false}
        onSelect={() => {}}
        onToggleWatch={mockToggle}
      />
    );

    fireEvent.click(screen.getByTestId(`watch-toggle-${mockWorkspace.workspace_identifier_value}`));

    await waitFor(() => {
      expect(mockToggle).toHaveBeenCalledWith(true);
    });
  });

  /**
   * REQ-SIDEBAR-004.4: Handle Toggle Error
   *
   * WHEN toggle API call fails
   * THEN SHALL revert toggle visual state to previous
   */
  test.skip('reverts visual state on toggle error', async () => {
    // This test would require mocking the API to fail
    render(
      <WorkspaceListItem
        workspace={mockWorkspace}
        isSelected={false}
        onSelect={() => {}}
        onToggleWatch={() => Promise.reject(new Error('API error'))}
      />
    );

    const toggle = screen.getByTestId(`watch-toggle-${mockWorkspace.workspace_identifier_value}`);
    fireEvent.click(toggle);

    await waitFor(() => {
      expect(toggle).toHaveClass('bg-gray-600'); // Reverted
    });
  });
});
