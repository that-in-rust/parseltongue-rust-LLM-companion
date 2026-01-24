/**
 * Connection Status Indicator Component.
 *
 * REQ-WS-004: Connection Status Indicator
 * Displays visual feedback about WebSocket connection state.
 */

import type { ConnectionStatus } from '@/types/api';

/**
 * Props for ConnectionStatusIndicator component.
 */
export interface ConnectionStatusIndicatorProps {
  connectionStatus: ConnectionStatus;
  reconnectAttempt?: number;
  maxReconnectAttempts?: number;
  className?: string;
}

/**
 * Connection Status Indicator component.
 *
 * REQ-WS-004.1: Display Connection Status
 * REQ-WS-004.2: Animate Status Transitions
 * REQ-WS-004.3: Show Reconnection Progress
 */
export function ConnectionStatusIndicator({
  connectionStatus,
  reconnectAttempt,
  maxReconnectAttempts,
  className = '',
}: ConnectionStatusIndicatorProps): JSX.Element {
  const config = CONNECTION_STATUS_CONFIG[connectionStatus];

  // REQ-WS-004.3: Show reconnection progress when reconnecting
  const displayText =
    connectionStatus === 'error' &&
    reconnectAttempt !== undefined &&
    maxReconnectAttempts !== undefined &&
    reconnectAttempt > 0
      ? `Reconnecting... (attempt ${reconnectAttempt}/${maxReconnectAttempts})`
      : config.text;

  return (
    <div
      className={`flex items-center gap-2 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 ${className}`}
      data-testid="connection-status-indicator"
    >
      {/* REQ-WS-004.1: Colored status dot */}
      <div
        className={`w-2 h-2 rounded-full ${config.dotClass} transition-colors duration-200`}
        data-testid="status-dot"
      />
      {/* REQ-WS-004.2: Animated text transition */}
      <span className="text-sm text-gray-300 transition-opacity duration-150">
        {displayText}
      </span>
    </div>
  );
}

/**
 * Status configuration for each connection state.
 */
export const CONNECTION_STATUS_CONFIG: Record<
  ConnectionStatus,
  { color: string; text: string; dotClass: string }
> = {
  connecting: {
    color: 'yellow',
    text: 'Connecting...',
    dotClass: 'bg-yellow-500',
  },
  connected: {
    color: 'green',
    text: 'Connected',
    dotClass: 'bg-green-500',
  },
  disconnected: {
    color: 'gray',
    text: 'Disconnected',
    dotClass: 'bg-gray-500',
  },
  error: {
    color: 'red',
    text: 'Connection error',
    dotClass: 'bg-red-500',
  },
};
