/**
 * Connection Status Indicator Tests
 *
 * REQ-WS-004: Connection Status Indicator
 * Tests for WebSocket connection status display.
 */

import { describe, test, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import {
  ConnectionStatusIndicator,
  CONNECTION_STATUS_CONFIG,
} from '../ConnectionStatusIndicator';
import type { ConnectionStatus } from '@/types/api';

// =============================================================================
// REQ-WS-004: Connection Status Indicator
// =============================================================================

describe('REQ-WS-004: Connection Status Indicator', () => {
  /**
   * REQ-WS-004.1: Display Connection Status - Connecting
   *
   * WHEN connectionStatus is 'connecting'
   * THEN SHALL display yellow dot and "Connecting..." text
   */
  test('displays connecting status with yellow indicator', () => {
    render(<ConnectionStatusIndicator connectionStatus="connecting" />);

    expect(screen.getByText('Connecting...')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-yellow-500');
  });

  /**
   * REQ-WS-004.1: Display Connection Status - Connected
   *
   * WHEN connectionStatus is 'connected'
   * THEN SHALL display green dot and "Connected" text
   */
  test('displays connected status with green indicator', () => {
    render(<ConnectionStatusIndicator connectionStatus="connected" />);

    expect(screen.getByText('Connected')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-green-500');
  });

  /**
   * REQ-WS-004.1: Display Connection Status - Disconnected
   *
   * WHEN connectionStatus is 'disconnected'
   * THEN SHALL display gray dot and "Disconnected" text
   */
  test('displays disconnected status with gray indicator', () => {
    render(<ConnectionStatusIndicator connectionStatus="disconnected" />);

    expect(screen.getByText('Disconnected')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-gray-500');
  });

  /**
   * REQ-WS-004.1: Display Connection Status - Error
   *
   * WHEN connectionStatus is 'error'
   * THEN SHALL display red dot and "Connection error" text
   */
  test('displays error status with red indicator', () => {
    render(<ConnectionStatusIndicator connectionStatus="error" />);

    expect(screen.getByText('Connection error')).toBeInTheDocument();
    expect(screen.getByTestId('status-dot')).toHaveClass('bg-red-500');
  });

  /**
   * REQ-WS-004.3: Show Reconnection Progress
   *
   * WHEN connectionStatus is 'error' and reconnection is in progress
   * THEN SHALL display "Reconnecting... (attempt N/5)"
   */
  test('shows reconnection progress when reconnecting', () => {
    render(
      <ConnectionStatusIndicator
        connectionStatus="error"
        reconnectAttempt={2}
        maxReconnectAttempts={5}
      />
    );

    expect(screen.getByText('Reconnecting... (attempt 2/5)')).toBeInTheDocument();
  });

  /**
   * REQ-WS-004.3: Show Reconnection Progress - Different Attempts
   *
   * WHEN reconnection attempt changes
   * THEN SHALL update the attempt counter display
   */
  test('updates attempt counter on retry', () => {
    const { rerender } = render(
      <ConnectionStatusIndicator
        connectionStatus="error"
        reconnectAttempt={1}
        maxReconnectAttempts={5}
      />
    );

    expect(screen.getByText('Reconnecting... (attempt 1/5)')).toBeInTheDocument();

    rerender(
      <ConnectionStatusIndicator
        connectionStatus="error"
        reconnectAttempt={3}
        maxReconnectAttempts={5}
      />
    );

    expect(screen.getByText('Reconnecting... (attempt 3/5)')).toBeInTheDocument();
  });
});

// =============================================================================
// Configuration Tests
// =============================================================================

describe('CONNECTION_STATUS_CONFIG', () => {
  /**
   * Configuration has all required status types
   */
  test('has configuration for all connection statuses', () => {
    const statuses: ConnectionStatus[] = ['connecting', 'connected', 'disconnected', 'error'];

    statuses.forEach((status) => {
      expect(CONNECTION_STATUS_CONFIG[status]).toBeDefined();
      expect(CONNECTION_STATUS_CONFIG[status].color).toBeDefined();
      expect(CONNECTION_STATUS_CONFIG[status].text).toBeDefined();
      expect(CONNECTION_STATUS_CONFIG[status].dotClass).toBeDefined();
    });
  });
});
