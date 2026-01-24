/**
 * WebSocket Diff Stream Hook - Manages WebSocket connection lifecycle.
 *
 * REQ-WS-001: WebSocket Connection Hook
 * Establishes and manages WebSocket connections with automatic lifecycle handling.
 *
 * REQ-WS-002: Subscription Management
 * Handles subscribe/unsubscribe to workspace updates.
 *
 * REQ-WS-003: Event Processing
 * Parses and dispatches WebSocket events to update application state.
 */

import type { ConnectionStatus, WebSocketServerEvent } from '@/types/api';

/**
 * Return type for useWebsocketDiffStream hook.
 */
export interface UseWebsocketDiffStreamReturn {
  connectionStatus: ConnectionStatus;
  lastDiffEvent: WebSocketServerEvent | null;
  reconnectAttempt: number;
  maxReconnectAttempts: number;
  subscribe: (workspaceId: string) => void;
  unsubscribe: () => void;
}

import { useState, useEffect, useCallback, useRef } from 'react';
import { useDiffVisualizationStore } from '@/stores/diffVisualizationStore';

/**
 * WebSocket configuration constants.
 */
const WS_HEARTBEAT_INTERVAL_MS = 30000;
const RECONNECT_INITIAL_DELAY_MS = 1000;
const RECONNECT_MAX_DELAY_MS = 30000;
const RECONNECT_FACTOR = 2;
const MAX_RECONNECT_ATTEMPTS = 5;

/**
 * Hook for managing WebSocket connection to diff stream.
 *
 * REQ-WS-001: WebSocket Connection Hook
 * REQ-WS-002: Subscription Management
 * REQ-WS-003: Event Processing
 */
export function useWebsocketDiffStream(): UseWebsocketDiffStreamReturn {
  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('connecting');
  const [lastDiffEvent, setLastDiffEvent] = useState<WebSocketServerEvent | null>(null);
  const [reconnectAttempt, setReconnectAttempt] = useState(0);

  const wsRef = useRef<WebSocket | null>(null);
  const subscribedWorkspaceRef = useRef<string | null>(null);
  const heartbeatIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const diffActions = useDiffVisualizationStore((state) => state.actions);

  /**
   * Calculate reconnection delay with exponential backoff.
   */
  const getReconnectDelay = useCallback((attempt: number): number => {
    const delay = Math.min(
      RECONNECT_INITIAL_DELAY_MS * Math.pow(RECONNECT_FACTOR, attempt),
      RECONNECT_MAX_DELAY_MS
    );
    return delay;
  }, []);

  /**
   * Start heartbeat ping interval.
   */
  const startHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
    }
    heartbeatIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ action: 'ping' }));
      }
    }, WS_HEARTBEAT_INTERVAL_MS);
  }, []);

  /**
   * Stop heartbeat ping interval.
   */
  const stopHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
  }, []);

  /**
   * Process incoming WebSocket message.
   */
  const handleMessage = useCallback(
    (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as WebSocketServerEvent;
        setLastDiffEvent(data);

        // REQ-WS-003.2: Handle diff lifecycle events
        if (data.event === 'diff_started') {
          diffActions.setDiffInProgress(true);
        } else if (data.event === 'diff_completed') {
          diffActions.setDiffInProgress(false);
          diffActions.updateSummaryData(data.summary);
        } else if (
          data.event === 'entity_added' ||
          data.event === 'entity_removed' ||
          data.event === 'entity_modified' ||
          data.event === 'edge_added' ||
          data.event === 'edge_removed'
        ) {
          // REQ-WS-003.3: Handle entity events
          diffActions.applyEntityEvent(data);
        } else if (data.event === 'error') {
          // REQ-WS-003.4: Handle error events
          console.error(`WebSocket error [${data.code}]: ${data.message}`);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    },
    [diffActions]
  );

  /**
   * Establish WebSocket connection.
   */
  const connect = useCallback(() => {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/websocket-diff-stream`;

    setConnectionStatus('connecting');

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setConnectionStatus('connected');
      setReconnectAttempt(0);
      startHeartbeat();

      // Re-subscribe if we had a previous subscription
      if (subscribedWorkspaceRef.current) {
        ws.send(
          JSON.stringify({
            action: 'subscribe',
            workspace_id: subscribedWorkspaceRef.current,
          })
        );
      }
    };

    ws.onmessage = handleMessage;

    ws.onclose = () => {
      setConnectionStatus('disconnected');
      stopHeartbeat();

      // Attempt reconnection if not at max attempts
      if (reconnectAttempt < MAX_RECONNECT_ATTEMPTS) {
        const delay = getReconnectDelay(reconnectAttempt);
        reconnectTimeoutRef.current = setTimeout(() => {
          setReconnectAttempt((prev) => prev + 1);
          connect();
        }, delay);
      }
    };

    ws.onerror = () => {
      setConnectionStatus('error');
      stopHeartbeat();
    };
  }, [reconnectAttempt, startHeartbeat, stopHeartbeat, handleMessage, getReconnectDelay]);

  /**
   * REQ-WS-002.1: Subscribe to workspace updates.
   */
  const subscribe = useCallback((workspaceId: string) => {
    subscribedWorkspaceRef.current = workspaceId;

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(
        JSON.stringify({
          action: 'subscribe',
          workspace_id: workspaceId,
        })
      );
    }
  }, []);

  /**
   * REQ-WS-002.3: Unsubscribe from workspace updates.
   */
  const unsubscribe = useCallback(() => {
    subscribedWorkspaceRef.current = null;
    setLastDiffEvent(null);

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ action: 'unsubscribe' }));
    }
  }, []);

  /**
   * REQ-WS-001.1: Establish connection on mount.
   * REQ-WS-001.4: Clean up connection on unmount.
   */
  useEffect(() => {
    connect();

    return () => {
      stopHeartbeat();
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
      setConnectionStatus('disconnected');
    };
  }, [connect, stopHeartbeat]);

  return {
    connectionStatus,
    lastDiffEvent,
    reconnectAttempt,
    maxReconnectAttempts: MAX_RECONNECT_ATTEMPTS,
    subscribe,
    unsubscribe,
  };
}
