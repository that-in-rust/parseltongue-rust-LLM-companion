/**
 * Diff Summary Stats Component
 *
 * REQ-SUMMARY-001: DiffSummaryStats Component
 *
 * Badge bar displaying change counts for diff summary:
 * - Added count (green badge)
 * - Removed count (red badge)
 * - Modified count (amber badge)
 * - Affected/blast radius count (blue badge)
 */

import { useState } from 'react';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';

/**
 * Props for DiffSummaryStats component.
 */
export interface DiffSummaryStatsProps {
  /** Number of entities in the blast radius */
  blastRadiusCount: number;
}

/**
 * Format number with thousands separator using locale-aware formatting.
 */
function formatNumberWithCommas(value: number): string {
  if (isNaN(value)) {
    return '--';
  }
  if (value < 0) {
    console.warn('DiffSummaryStats: Invalid negative count detected:', value);
    return '0';
  }
  return new Intl.NumberFormat('en-US').format(value);
}

/**
 * DiffSummaryStats - Displays summary counts of diff changes.
 */
export function DiffSummaryStats(props: DiffSummaryStatsProps): JSX.Element {
  const { blastRadiusCount } = props;
  const summary = useDiffVisualizationStore((state) => state.summary);
  const isDiffInProgress = useDiffVisualizationStore((state) => state.isDiffInProgress);
  const [isExpanded, setIsExpanded] = useState(false);

  const handleToggleExpand = () => {
    setIsExpanded((prev) => !prev);
  };

  // Check if we have any changes
  const hasChanges =
    summary &&
    (summary.added_entity_count > 0 ||
      summary.removed_entity_count > 0 ||
      summary.modified_entity_count > 0);

  // Loading state
  if (isDiffInProgress) {
    return (
      <div
        data-testid="diff-summary-stats"
        role="status"
        aria-live="polite"
        className="bg-gray-800 rounded-lg px-3 py-2 animate-pulse"
      >
        <div className="flex items-center gap-2">
          <svg
            data-testid="loading-spinner"
            className="w-4 h-4 animate-spin text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
          >
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          <span className="text-sm text-gray-400">Analyzing changes...</span>
          <div data-testid="loading-skeleton" className="flex gap-2 ml-4">
            <div className="w-12 h-6 bg-gray-700 rounded-md animate-pulse" />
            <div className="w-12 h-6 bg-gray-700 rounded-md animate-pulse" />
            <div className="w-12 h-6 bg-gray-700 rounded-md animate-pulse" />
            <div className="w-12 h-6 bg-gray-700 rounded-md animate-pulse" />
          </div>
        </div>
      </div>
    );
  }

  // Empty state
  if (!summary || !hasChanges) {
    return (
      <div
        data-testid="diff-summary-stats"
        role="status"
        aria-live="polite"
        className="bg-gray-800 rounded-lg px-3 py-2"
      >
        <span className="text-sm text-gray-500">No changes detected</span>
      </div>
    );
  }

  // Validate counts and handle NaN / negatives
  let addedCount = summary.added_entity_count;
  let removedCount = summary.removed_entity_count;
  let modifiedCount = summary.modified_entity_count;
  let affectedCount = blastRadiusCount;

  // Check for NaN values first
  if (isNaN(addedCount)) {
    console.warn('DiffSummaryStats: NaN value detected in added_entity_count');
  }
  if (isNaN(removedCount)) {
    console.warn('DiffSummaryStats: NaN value detected in removed_entity_count');
  }
  if (isNaN(modifiedCount)) {
    console.warn('DiffSummaryStats: NaN value detected in modified_entity_count');
  }
  if (isNaN(affectedCount)) {
    console.warn('DiffSummaryStats: NaN value detected in blastRadiusCount');
  }

  // Check for negative values and clamp to 0
  if (addedCount < 0) {
    console.warn('DiffSummaryStats: Invalid negative count detected in added_entity_count:', addedCount);
    addedCount = 0;
  }
  if (removedCount < 0) {
    console.warn('DiffSummaryStats: Invalid negative count detected in removed_entity_count:', removedCount);
    removedCount = 0;
  }
  if (modifiedCount < 0) {
    console.warn('DiffSummaryStats: Invalid negative count detected in modified_entity_count:', modifiedCount);
    modifiedCount = 0;
  }
  if (affectedCount < 0) {
    console.warn('DiffSummaryStats: Invalid negative count detected in blastRadiusCount:', affectedCount);
    affectedCount = 0;
  }

  return (
    <div
      data-testid="diff-summary-stats"
      role="status"
      aria-live="polite"
      aria-expanded={isExpanded}
      className="bg-gray-800 rounded-lg transition-all duration-200"
    >
      {/* Main badge bar */}
      <button
        onClick={handleToggleExpand}
        className="w-full px-3 py-2 flex items-center justify-between"
      >
        <div
          data-testid="badge-container"
          className="grid grid-cols-2 sm:flex-row sm:flex gap-2 flex-1"
        >
          {/* Added badge */}
          <div
            data-testid="badge-added-count"
            role="img"
            aria-label={`${addedCount} entities added`}
            title={`${addedCount} entities added`}
            className="px-2 py-1 rounded-md text-xs sm:text-sm font-medium border bg-green-500/20 text-green-400 border-green-500/50"
            onMouseEnter={(e) => {
              const tooltip = document.createElement('div');
              tooltip.role = 'tooltip';
              tooltip.className = 'absolute bottom-full mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg';
              tooltip.textContent = `${addedCount} entities added`;
              e.currentTarget.appendChild(tooltip);
              setTimeout(() => tooltip.remove(), 3000);
            }}
          >
            +{formatNumberWithCommas(addedCount)}
          </div>

          {/* Removed badge */}
          <div
            data-testid="badge-removed-count"
            role="img"
            aria-label={`${removedCount} entities removed`}
            title={`${removedCount} entities removed`}
            className="px-2 py-1 rounded-md text-xs sm:text-sm font-medium border bg-red-500/20 text-red-400 border-red-500/50"
            onMouseEnter={(e) => {
              const tooltip = document.createElement('div');
              tooltip.role = 'tooltip';
              tooltip.className = 'absolute bottom-full mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg';
              tooltip.textContent = `${removedCount} entities removed`;
              e.currentTarget.appendChild(tooltip);
              setTimeout(() => tooltip.remove(), 3000);
            }}
          >
            -{formatNumberWithCommas(removedCount)}
          </div>

          {/* Modified badge */}
          <div
            data-testid="badge-modified-count"
            role="img"
            aria-label={`${modifiedCount} entities modified`}
            title={`${modifiedCount} entities modified`}
            className="px-2 py-1 rounded-md text-xs sm:text-sm font-medium border bg-amber-500/20 text-amber-400 border-amber-500/50"
            onMouseEnter={(e) => {
              const tooltip = document.createElement('div');
              tooltip.role = 'tooltip';
              tooltip.className = 'absolute bottom-full mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg';
              tooltip.textContent = `${modifiedCount} entities modified`;
              e.currentTarget.appendChild(tooltip);
              setTimeout(() => tooltip.remove(), 3000);
            }}
          >
            ~{formatNumberWithCommas(modifiedCount)}
          </div>

          {/* Affected badge */}
          <div
            data-testid="badge-affected-count"
            role="img"
            aria-label={`${affectedCount} entities in blast radius`}
            title={`${affectedCount} entities in blast radius`}
            className="px-2 py-1 rounded-md text-xs sm:text-sm font-medium border bg-blue-500/20 text-blue-400 border-blue-500/50 flex items-center gap-1"
            onMouseEnter={(e) => {
              const tooltip = document.createElement('div');
              tooltip.role = 'tooltip';
              tooltip.className = 'absolute bottom-full mb-2 px-2 py-1 bg-gray-900 text-white text-xs rounded shadow-lg';
              tooltip.textContent = `${affectedCount} entities in blast radius`;
              e.currentTarget.appendChild(tooltip);
              setTimeout(() => tooltip.remove(), 3000);
            }}
          >
            <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path d="M11 3a1 1 0 10-2 0v1a1 1 0 102 0V3zM15.657 5.757a1 1 0 00-1.414-1.414l-.707.707a1 1 0 001.414 1.414l.707-.707zM18 10a1 1 0 01-1 1h-1a1 1 0 110-2h1a1 1 0 011 1zM5.05 6.464A1 1 0 106.464 5.05l-.707-.707a1 1 0 00-1.414 1.414l.707.707zM5 10a1 1 0 01-1 1H3a1 1 0 110-2h1a1 1 0 011 1zM8 16v-1h4v1a2 2 0 11-4 0zM12 14c.015-.34.208-.646.477-.859a4 4 0 10-4.954 0c.27.213.462.519.476.859h4.002z" />
            </svg>
            {formatNumberWithCommas(affectedCount)}
          </div>
        </div>

        {/* Chevron icon */}
        <svg
          data-testid="expand-chevron"
          className={`w-5 h-5 text-gray-400 transition-transform duration-200 ${
            isExpanded ? 'rotate-180' : ''
          }`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Expanded detail view */}
      {isExpanded && (
        <div
          data-testid="summary-detail-view"
          className="px-3 pb-3 pt-1 border-t border-gray-700 text-sm text-gray-300"
        >
          <div className="space-y-1">
            <div className="flex justify-between">
              <span>Total before:</span>
              <span className="font-mono">{formatNumberWithCommas(summary.total_before_count)}</span>
            </div>
            <div className="flex justify-between">
              <span>Total after:</span>
              <span className="font-mono">{formatNumberWithCommas(summary.total_after_count)}</span>
            </div>
            <div className="flex justify-between">
              <span>Unchanged:</span>
              <span className="font-mono">{formatNumberWithCommas(summary.unchanged_entity_count)}</span>
            </div>
            <div className="flex justify-between">
              <span>Relocated:</span>
              <span className="font-mono">{formatNumberWithCommas(summary.relocated_entity_count)}</span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default DiffSummaryStats;
