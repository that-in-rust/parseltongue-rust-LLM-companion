/**
 * Vitest test setup file.
 * Configures testing-library and global test utilities.
 */

import '@testing-library/jest-dom';
import React from 'react';
import { vi } from 'vitest';

// Mock react-force-graph-3d to avoid WebGL context errors in jsdom
vi.mock('react-force-graph-3d', () => ({
  default: vi.fn(({ graphData, onNodeClick, onBackgroundClick }) => {
    return (
      <div
        data-testid="force-graph-3d-mock"
        data-nodes={graphData?.nodes?.length ?? 0}
        data-links={graphData?.links?.length ?? 0}
        onClick={onBackgroundClick}
      >
        {graphData?.nodes?.map((node: { id: string }) => (
          <div
            key={node.id}
            data-testid={`node-${node.id}`}
            onClick={() => onNodeClick?.(node)}
          />
        ))}
      </div>
    );
  }),
  ForceGraphMethods: {},
}));

// Mock WebSocket for tests
class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  url: string;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: (() => void) | null = null;

  constructor(url: string) {
    this.url = url;
  }

  send(_data: string): void {
    // Mock implementation
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.();
  }
}

// @ts-expect-error - Replacing global WebSocket with mock
global.WebSocket = MockWebSocket;

// Mock fetch for tests - returns empty workspace list by default
global.fetch = vi.fn().mockImplementation(async (url: string) => {
  if (url === '/workspace-list-all') {
    return {
      ok: true,
      json: async () => ({ workspaces: [], success: true }),
    };
  }
  // Default mock response
  return {
    ok: true,
    json: async () => ({}),
  };
});
