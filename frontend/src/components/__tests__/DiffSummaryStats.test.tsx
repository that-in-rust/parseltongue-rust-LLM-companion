/**
 * Diff Summary Stats Tests
 *
 * REQ-SUMMARY-001: DiffSummaryStats Component
 * Tests for REQ-SUMMARY-001.1 through REQ-SUMMARY-001.8
 *
 * These tests verify the badge bar that displays change counts
 * (added, removed, modified, affected) for diff summary.
 */

import { describe, test, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { DiffSummaryStats } from '../DiffSummaryStats';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';
import type { DiffSummaryData } from '@/types/api';

// =============================================================================
// Test Fixtures
// =============================================================================

const mockSummary: DiffSummaryData = {
  total_before_count: 100,
  total_after_count: 115,
  added_entity_count: 20,
  removed_entity_count: 5,
  modified_entity_count: 12,
  unchanged_entity_count: 78,
  relocated_entity_count: 3,
};

const mockSummaryLargeNumbers: DiffSummaryData = {
  total_before_count: 50000,
  total_after_count: 62345,
  added_entity_count: 12345,
  removed_entity_count: 1000,
  modified_entity_count: 5678,
  unchanged_entity_count: 45000,
  relocated_entity_count: 100,
};

const mockSummaryZeros: DiffSummaryData = {
  total_before_count: 100,
  total_after_count: 100,
  added_entity_count: 0,
  removed_entity_count: 0,
  modified_entity_count: 0,
  unchanged_entity_count: 100,
  relocated_entity_count: 0,
};

const mockSummaryWithNegative: DiffSummaryData = {
  total_before_count: 100,
  total_after_count: 100,
  added_entity_count: -5, // Invalid negative
  removed_entity_count: 10,
  modified_entity_count: 5,
  unchanged_entity_count: 90,
  relocated_entity_count: 0,
};

// =============================================================================
// Test Setup
// =============================================================================

beforeEach(() => {
  useDiffVisualizationStore.setState({
    summary: null,
    isDiffInProgress: false,
  });
});

// =============================================================================
// REQ-SUMMARY-001.1: Display Change Counts
// =============================================================================

describe('REQ-SUMMARY-001.1: Display Change Counts', () => {
  /**
   * REQ-SUMMARY-001.1a: Display added count badge
   *
   * WHEN DiffSummaryStats receives summary from useDiffSummary()
   * THEN SHALL display Added badge with "+{count}" format
   *   AND badge SHALL have bg-green-500/20 text-green-400 styling
   */
  test('displays added count badge with correct styling', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveTextContent('+20');
    expect(addedBadge).toHaveClass('bg-green-500/20');
    expect(addedBadge).toHaveClass('text-green-400');
  });

  /**
   * REQ-SUMMARY-001.1b: Display removed count badge
   *
   * WHEN DiffSummaryStats receives summary
   * THEN SHALL display Removed badge with "-{count}" format
   *   AND badge SHALL have bg-red-500/20 text-red-400 styling
   */
  test('displays removed count badge with correct styling', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const removedBadge = screen.getByTestId('badge-removed-count');
    expect(removedBadge).toHaveTextContent('-5');
    expect(removedBadge).toHaveClass('bg-red-500/20');
    expect(removedBadge).toHaveClass('text-red-400');
  });

  /**
   * REQ-SUMMARY-001.1c: Display modified count badge
   *
   * WHEN DiffSummaryStats receives summary
   * THEN SHALL display Modified badge with "~{count}" format
   *   AND badge SHALL have bg-amber-500/20 text-amber-400 styling
   */
  test('displays modified count badge with correct styling', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const modifiedBadge = screen.getByTestId('badge-modified-count');
    expect(modifiedBadge).toHaveTextContent('~12');
    expect(modifiedBadge).toHaveClass('bg-amber-500/20');
    expect(modifiedBadge).toHaveClass('text-amber-400');
  });

  /**
   * REQ-SUMMARY-001.1d: Display affected/blast radius count badge
   *
   * WHEN DiffSummaryStats receives blastRadiusCount prop
   * THEN SHALL display Affected badge with lightning icon and count
   *   AND badge SHALL have bg-blue-500/20 text-blue-400 styling
   */
  test('displays affected count badge with correct styling', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const affectedBadge = screen.getByTestId('badge-affected-count');
    expect(affectedBadge).toHaveTextContent('45');
    expect(affectedBadge).toHaveClass('bg-blue-500/20');
    expect(affectedBadge).toHaveClass('text-blue-400');
  });

  /**
   * REQ-SUMMARY-001.1e: Badges have consistent padding
   *
   * WHEN badges are rendered
   * THEN each badge SHALL have consistent padding (px-2 py-1)
   */
  test('badges have consistent padding', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    const removedBadge = screen.getByTestId('badge-removed-count');
    const modifiedBadge = screen.getByTestId('badge-modified-count');
    const affectedBadge = screen.getByTestId('badge-affected-count');

    [addedBadge, removedBadge, modifiedBadge, affectedBadge].forEach((badge) => {
      expect(badge).toHaveClass('px-2', 'py-1');
    });
  });

  /**
   * REQ-SUMMARY-001.1f: Badges have rounded corners
   *
   * WHEN badges are rendered
   * THEN badges SHALL have rounded corners (rounded-md)
   */
  test('badges have rounded corners', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveClass('rounded-md');
  });

  /**
   * REQ-SUMMARY-001.1g: Badges have subtle border
   *
   * WHEN badges are rendered
   * THEN badges SHALL have subtle border matching their color family
   */
  test('badges have border matching color family', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveClass('border', 'border-green-500/50');

    const removedBadge = screen.getByTestId('badge-removed-count');
    expect(removedBadge).toHaveClass('border', 'border-red-500/50');
  });
});

// =============================================================================
// REQ-SUMMARY-001.2: Format Large Numbers
// =============================================================================

describe('REQ-SUMMARY-001.2: Format Large Numbers', () => {
  /**
   * REQ-SUMMARY-001.2a: Format thousands with separator
   *
   * WHEN any count exceeds 999
   * THEN SHALL format with thousands separator (e.g., 1000 -> "1,000")
   */
  test('formats numbers with thousands separator', () => {
    useDiffVisualizationStore.setState({ summary: mockSummaryLargeNumbers });

    render(<DiffSummaryStats blastRadiusCount={9999} />);

    expect(screen.getByTestId('badge-added-count')).toHaveTextContent('+12,345');
    expect(screen.getByTestId('badge-removed-count')).toHaveTextContent('-1,000');
    expect(screen.getByTestId('badge-modified-count')).toHaveTextContent('~5,678');
    expect(screen.getByTestId('badge-affected-count')).toHaveTextContent('9,999');
  });

  /**
   * REQ-SUMMARY-001.2b: Use locale-aware formatting
   *
   * WHEN formatting numbers
   * THEN SHALL use locale-aware formatting (Intl.NumberFormat)
   *   AND SHALL NOT truncate to "1K" format
   */
  test('uses locale-aware number formatting without K abbreviation', () => {
    useDiffVisualizationStore.setState({ summary: mockSummaryLargeNumbers });

    render(<DiffSummaryStats blastRadiusCount={100000} />);

    // Should not use "100K" abbreviation
    expect(screen.getByTestId('badge-affected-count')).toHaveTextContent('100,000');
    expect(screen.getByTestId('badge-affected-count')).not.toHaveTextContent('100K');
  });

  /**
   * REQ-SUMMARY-001.2c: Format in detail view
   *
   * WHEN expanded detail view shows totals
   * THEN large numbers SHALL also be formatted
   */
  test('formats large numbers in detail view', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummaryLargeNumbers });

    render(<DiffSummaryStats blastRadiusCount={1000} />);

    await userEvent.click(screen.getByTestId('diff-summary-stats'));

    expect(screen.getByText(/Total before: 50,000/)).toBeInTheDocument();
    expect(screen.getByText(/Total after: 62,345/)).toBeInTheDocument();
  });
});

// =============================================================================
// REQ-SUMMARY-001.3: Empty/Zero State
// =============================================================================

describe('REQ-SUMMARY-001.3: Empty/Zero State', () => {
  /**
   * REQ-SUMMARY-001.3a: Display empty state when summary is null
   *
   * WHEN summary is null
   * THEN SHALL display "No changes detected"
   *   AND SHALL NOT display individual count badges
   */
  test('displays empty state when summary is null', () => {
    useDiffVisualizationStore.setState({ summary: null });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();
    expect(screen.queryByTestId('badge-added-count')).not.toBeInTheDocument();
    expect(screen.queryByTestId('badge-removed-count')).not.toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.3b: Display empty state when all counts are zero
   *
   * WHEN added_entity_count === 0 AND removed_entity_count === 0 AND modified_entity_count === 0
   * THEN SHALL display "No changes detected"
   */
  test('displays empty state when all change counts are zero', () => {
    useDiffVisualizationStore.setState({ summary: mockSummaryZeros });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();
    expect(screen.queryByTestId('badge-added-count')).not.toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.3c: Empty state has correct styling
   *
   * WHEN empty state is displayed
   * THEN SHALL have text-gray-500 bg-gray-800 rounded-lg px-3 py-2 styling
   */
  test('empty state has correct styling', () => {
    useDiffVisualizationStore.setState({ summary: null });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    const emptyState = screen.getByText('No changes detected').closest('div');
    expect(emptyState).toHaveClass('text-gray-500', 'bg-gray-800', 'rounded-lg');
  });
});

// =============================================================================
// REQ-SUMMARY-001.4: Auto-Update on WebSocket Events
// =============================================================================

describe('REQ-SUMMARY-001.4: Auto-Update on WebSocket Events', () => {
  /**
   * REQ-SUMMARY-001.4a: Re-renders when summary updates
   *
   * WHEN diffVisualizationStore.summary updates
   * THEN DiffSummaryStats SHALL re-render with new counts
   */
  test('re-renders when summary updates in store', async () => {
    const { rerender } = render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();

    useDiffVisualizationStore.setState({ summary: mockSummary });
    rerender(<DiffSummaryStats blastRadiusCount={45} />);

    await waitFor(() => {
      expect(screen.getByTestId('badge-added-count')).toHaveTextContent('+20');
    });
  });

  /**
   * REQ-SUMMARY-001.4b: Transition from empty to populated
   *
   * WHEN summary changes from null to valid data
   * THEN SHALL transition from empty state to badge display
   */
  test('transitions from empty to populated state', async () => {
    useDiffVisualizationStore.setState({ summary: null });

    const { rerender } = render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('No changes detected')).toBeInTheDocument();

    useDiffVisualizationStore.setState({ summary: mockSummary });
    rerender(<DiffSummaryStats blastRadiusCount={45} />);

    await waitFor(() => {
      expect(screen.queryByText('No changes detected')).not.toBeInTheDocument();
      expect(screen.getByTestId('badge-added-count')).toBeInTheDocument();
    });
  });
});

// =============================================================================
// REQ-SUMMARY-001.5: Collapsible Detail View
// =============================================================================

describe('REQ-SUMMARY-001.5: Collapsible Detail View', () => {
  /**
   * REQ-SUMMARY-001.5a: Click expands detail view
   *
   * WHEN user clicks on DiffSummaryStats bar
   * THEN SHALL expand to show additional details
   */
  test('expands to show details on click', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    // Initially collapsed
    expect(screen.queryByTestId('summary-detail-view')).not.toBeVisible();

    // Click to expand
    await userEvent.click(screen.getByTestId('diff-summary-stats'));

    expect(screen.getByTestId('summary-detail-view')).toBeVisible();
  });

  /**
   * REQ-SUMMARY-001.5b: Detail view shows totals
   *
   * WHEN detail view is expanded
   * THEN SHALL show total_before_count, total_after_count, unchanged, relocated
   */
  test('expanded view shows additional details', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    await userEvent.click(screen.getByTestId('diff-summary-stats'));

    expect(screen.getByText(/Total before: 100/)).toBeInTheDocument();
    expect(screen.getByText(/Total after: 115/)).toBeInTheDocument();
    expect(screen.getByText(/Unchanged: 78/)).toBeInTheDocument();
    expect(screen.getByText(/Relocated: 3/)).toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.5c: Chevron rotates on expand
   *
   * WHEN detail view is expanded
   * THEN chevron icon SHALL rotate 180 degrees
   */
  test('chevron rotates when expanded', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const chevron = screen.getByTestId('expand-chevron');
    expect(chevron).not.toHaveClass('rotate-180');

    await userEvent.click(screen.getByTestId('diff-summary-stats'));

    expect(chevron).toHaveClass('rotate-180');
  });

  /**
   * REQ-SUMMARY-001.5d: Second click collapses
   *
   * WHEN user clicks again on expanded DiffSummaryStats
   * THEN SHALL collapse back to summary bar
   */
  test('collapses on second click', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    // Expand
    await userEvent.click(screen.getByTestId('diff-summary-stats'));
    expect(screen.getByTestId('summary-detail-view')).toBeVisible();

    // Collapse
    await userEvent.click(screen.getByTestId('diff-summary-stats'));

    await waitFor(() => {
      expect(screen.queryByTestId('summary-detail-view')).not.toBeVisible();
    });
  });

  /**
   * REQ-SUMMARY-001.5e: Smooth height transition
   *
   * WHEN expanding/collapsing
   * THEN SHALL have smooth height transition (200ms)
   */
  test('has transition class for smooth animation', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const container = screen.getByTestId('diff-summary-stats');
    expect(container).toHaveClass('transition-all', 'duration-200');
  });
});

// =============================================================================
// REQ-SUMMARY-001.6: Diff In Progress Indicator
// =============================================================================

describe('REQ-SUMMARY-001.6: Diff In Progress Indicator', () => {
  /**
   * REQ-SUMMARY-001.6a: Shows loading state when diff in progress
   *
   * WHEN isDiffInProgress in diffVisualizationStore is true
   * THEN SHALL display loading state with "Analyzing changes..." text
   */
  test('shows loading text when diff in progress', () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('Analyzing changes...')).toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.6b: Shows spinner icon
   *
   * WHEN isDiffInProgress is true
   * THEN SHALL display spinner icon next to text
   */
  test('shows spinner when diff in progress', () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.6c: Pulsing animation on container
   *
   * WHEN isDiffInProgress is true
   * THEN container SHALL have pulsing animation (animate-pulse)
   */
  test('container has pulse animation when loading', () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    const container = screen.getByTestId('diff-summary-stats');
    expect(container).toHaveClass('animate-pulse');
  });

  /**
   * REQ-SUMMARY-001.6d: Loading skeleton replaces badges
   *
   * WHEN isDiffInProgress is true
   * THEN SHALL replace count badges with loading skeleton
   */
  test('shows loading skeleton instead of badges', () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.queryByTestId('badge-added-count')).not.toBeInTheDocument();
    expect(screen.getByTestId('loading-skeleton')).toBeInTheDocument();
  });

  /**
   * REQ-SUMMARY-001.6e: Transition from loading to populated
   *
   * WHEN isDiffInProgress changes from true to false
   * THEN SHALL transition from loading to populated state with fade animation
   */
  test('transitions from loading to populated state', async () => {
    useDiffVisualizationStore.setState({ isDiffInProgress: true });

    const { rerender } = render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByText('Analyzing changes...')).toBeInTheDocument();

    useDiffVisualizationStore.setState({
      isDiffInProgress: false,
      summary: mockSummary,
    });
    rerender(<DiffSummaryStats blastRadiusCount={45} />);

    await waitFor(() => {
      expect(screen.queryByText('Analyzing changes...')).not.toBeInTheDocument();
      expect(screen.getByTestId('badge-added-count')).toBeInTheDocument();
    });
  });
});

// =============================================================================
// REQ-SUMMARY-001.7: Badge Tooltips
// =============================================================================

describe('REQ-SUMMARY-001.7: Badge Tooltips', () => {
  /**
   * REQ-SUMMARY-001.7a: Added badge tooltip
   *
   * WHEN user hovers over added badge
   * THEN SHALL display tooltip "{count} entities added" after 300ms delay
   */
  test('shows tooltip on added badge hover', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('badge-added-count'));

    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toHaveTextContent('20 entities added');
      },
      { timeout: 500 }
    );
  });

  /**
   * REQ-SUMMARY-001.7b: Removed badge tooltip
   *
   * WHEN user hovers over removed badge
   * THEN SHALL display tooltip "{count} entities removed"
   */
  test('shows tooltip on removed badge hover', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('badge-removed-count'));

    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toHaveTextContent('5 entities removed');
      },
      { timeout: 500 }
    );
  });

  /**
   * REQ-SUMMARY-001.7c: Modified badge tooltip
   *
   * WHEN user hovers over modified badge
   * THEN SHALL display tooltip "{count} entities modified"
   */
  test('shows tooltip on modified badge hover', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('badge-modified-count'));

    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toHaveTextContent('12 entities modified');
      },
      { timeout: 500 }
    );
  });

  /**
   * REQ-SUMMARY-001.7d: Affected badge tooltip
   *
   * WHEN user hovers over affected badge
   * THEN SHALL display tooltip "{count} entities in blast radius"
   */
  test('shows tooltip on affected badge hover', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('badge-affected-count'));

    await waitFor(
      () => {
        expect(screen.getByRole('tooltip')).toHaveTextContent('45 entities in blast radius');
      },
      { timeout: 500 }
    );
  });

  /**
   * REQ-SUMMARY-001.7e: Tooltip positioned above badge
   *
   * WHEN tooltip is displayed
   * THEN SHALL be positioned above badge with arrow pointing to badge
   */
  test('tooltip is positioned above badge', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    fireEvent.mouseEnter(screen.getByTestId('badge-added-count'));

    await waitFor(
      () => {
        const tooltip = screen.getByRole('tooltip');
        expect(tooltip).toHaveClass('bottom-full'); // Positioned above
      },
      { timeout: 500 }
    );
  });
});

// =============================================================================
// REQ-SUMMARY-001.8: Responsive Layout
// =============================================================================

describe('REQ-SUMMARY-001.8: Responsive Layout', () => {
  /**
   * REQ-SUMMARY-001.8a: Desktop horizontal layout
   *
   * WHEN viewport width >= 640px
   * THEN badges SHALL display in horizontal row
   *   AND text size SHALL be text-sm
   */
  test('displays badges in horizontal row on desktop', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const badgeContainer = screen.getByTestId('badge-container');
    expect(badgeContainer).toHaveClass('sm:flex-row');
  });

  /**
   * REQ-SUMMARY-001.8b: Mobile 2x2 grid layout
   *
   * WHEN viewport width < 640px
   * THEN badges SHALL stack in 2x2 grid
   *   AND text size SHALL be text-xs
   */
  test('displays badges in grid on mobile', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const badgeContainer = screen.getByTestId('badge-container');
    expect(badgeContainer).toHaveClass('grid', 'grid-cols-2');
  });

  /**
   * REQ-SUMMARY-001.8c: Desktop standard padding
   *
   * WHEN viewport width >= 640px
   * THEN padding SHALL be standard (px-2 py-1)
   */
  test('badges have standard padding on desktop', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveClass('sm:px-2', 'sm:py-1');
  });

  /**
   * REQ-SUMMARY-001.8d: Mobile reduced padding
   *
   * WHEN viewport width < 640px
   * THEN padding SHALL be reduced (px-1.5 py-0.5)
   */
  test('badges have reduced padding on mobile', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveClass('px-1.5', 'py-0.5');
  });
});

// =============================================================================
// Error Conditions
// =============================================================================

describe('DiffSummaryStats Error Conditions', () => {
  /**
   * Error: Negative count values
   *
   * WHEN summary contains negative count values
   * THEN SHALL treat as 0
   *   AND SHALL log warning to console
   */
  test('handles negative counts by treating as zero', () => {
    useDiffVisualizationStore.setState({ summary: mockSummaryWithNegative });
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByTestId('badge-added-count')).toHaveTextContent('+0');
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('Invalid negative count')
    );

    consoleSpy.mockRestore();
  });

  /**
   * Error: NaN values
   *
   * WHEN summary contains NaN or non-numeric values
   * THEN SHALL display "--" instead of count
   *   AND SHALL log warning with field name
   */
  test('handles NaN values by displaying placeholder', () => {
    const summaryWithNaN: DiffSummaryData = {
      ...mockSummary,
      added_entity_count: NaN,
    };
    useDiffVisualizationStore.setState({ summary: summaryWithNaN });
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    render(<DiffSummaryStats blastRadiusCount={0} />);

    expect(screen.getByTestId('badge-added-count')).toHaveTextContent('--');
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('added_entity_count')
    );

    consoleSpy.mockRestore();
  });
});

// =============================================================================
// Accessibility Tests (REQ-A11Y-001.2)
// =============================================================================

describe('REQ-A11Y-001.2: DiffSummaryStats Accessibility', () => {
  /**
   * Container has status role for live region
   *
   * WHEN DiffSummaryStats renders
   * THEN container SHALL have role="status"
   */
  test('container has status role', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const container = screen.getByTestId('diff-summary-stats');
    expect(container).toHaveAttribute('role', 'status');
  });

  /**
   * Container has aria-live for announcements
   *
   * WHEN DiffSummaryStats renders
   * THEN container SHALL have aria-live="polite"
   */
  test('container has aria-live polite', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const container = screen.getByTestId('diff-summary-stats');
    expect(container).toHaveAttribute('aria-live', 'polite');
  });

  /**
   * Badges have aria-label
   *
   * WHEN badges render
   * THEN each badge SHALL have aria-label describing the count
   */
  test('badges have descriptive aria-labels', () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const addedBadge = screen.getByTestId('badge-added-count');
    expect(addedBadge).toHaveAttribute('aria-label', '20 entities added');

    const removedBadge = screen.getByTestId('badge-removed-count');
    expect(removedBadge).toHaveAttribute('aria-label', '5 entities removed');
  });

  /**
   * Expand button has aria-expanded
   *
   * WHEN DiffSummaryStats renders with expand capability
   * THEN expand button SHALL have aria-expanded state
   */
  test('expand button has aria-expanded state', async () => {
    useDiffVisualizationStore.setState({ summary: mockSummary });

    render(<DiffSummaryStats blastRadiusCount={45} />);

    const expandButton = screen.getByTestId('diff-summary-stats');
    expect(expandButton).toHaveAttribute('aria-expanded', 'false');

    await userEvent.click(expandButton);

    expect(expandButton).toHaveAttribute('aria-expanded', 'true');
  });
});
