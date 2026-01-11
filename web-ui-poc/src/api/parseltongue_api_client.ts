/**
 * Parseltongue API Client
 *
 * Minimal implementation to satisfy TDD tests
 * Following RED -> GREEN -> REFACTOR cycle
 *
 * # 4-Word Name: parseltongue_api_client
 */

import type {
  HealthCheckResponsePayload,
  EntitiesListResponsePayload,
  EntitiesListQueryParams,
  StatisticsOverviewResponsePayload,
} from '../types/parseltongue_api_types';

/**
 * Dependency edge between entities
 */
export interface DependencyEdge {
  from: string;      // Entity key that depends
  to: string;        // Entity key being depended upon
  edgeType: string;  // Type of relationship
}

/**
 * Forward dependencies response (what this entity calls/uses)
 * API returns either "callees" or "forward_callees" field
 */
export interface ForwardDependenciesResponse {
  success: boolean;
  endpoint: string;
  data?: {
    total_count?: number;
    callees?: Array<{
      from_key: string;
      to_key: string;
      edge_type: string;
      source_location?: string;
    }>;
    forward_callees?: Array<{
      entity_key: string;
      edge_type: string;
    }>;
  };
}

/**
 * Backward dependencies response (what calls this entity)
 * API returns either "callers" or "reverse_callers" field
 */
export interface BackwardDependenciesResponse {
  success: boolean;
  endpoint: string;
  data?: {
    total_count?: number;
    callers?: Array<{
      from_key: string;
      to_key: string;
      edge_type: string;
      source_location?: string;
    }>;
    reverse_callers?: Array<{
      entity_key: string;
      edge_type: string;
    }>;
  };
}

/**
 * Combined dependencies for visualization
 */
export interface EntityDependencies {
  entityKey: string;
  forward: DependencyEdge[];  // What this entity calls/uses
  backward: DependencyEdge[]; // What calls this entity
}

export class ParseltongueApiClient {
  constructor(private readonly baseUrl: string) {}

  /**
   * Fetch health check status from Parseltongue server
   *
   * # 4-Word Name: fetch_server_health_check_status
   *
   * # Contract
   * - Preconditions: Server is running at baseUrl
   * - Postconditions: Returns HealthCheckResponsePayload with success=true
   * - Error Conditions: Throws if server unreachable or returns non-OK status
   * - Performance: Must complete in <100ms
   */
  async fetch_server_health_check_status(): Promise<HealthCheckResponsePayload> {
    const response = await fetch(`${this.baseUrl}/server-health-check-status`);
    if (!response.ok) {
      throw new Error(`Health check failed: ${response.status}`);
    }
    return response.json();
  }

  /**
   * Fetch all code entities from Parseltongue server
   *
   * # 4-Word Name: fetch_all_code_entities_list
   *
   * # Contract
   * - Preconditions: Server is running, database is loaded
   * - Postconditions: Returns EntitiesListResponsePayload with entities array
   * - Error Conditions: Throws if server unreachable or database not loaded
   * - Performance: Must complete in <500ms for up to 1000 entities
   *
   * @param params - Optional query parameters for filtering
   */
  async fetch_all_code_entities_list(
    params?: EntitiesListQueryParams
  ): Promise<EntitiesListResponsePayload> {
    const url = new URL(`${this.baseUrl}/code-entities-list-all`);
    if (params?.entity_type) {
      url.searchParams.set('entity_type', params.entity_type);
    }

    const response = await fetch(url.toString());
    if (!response.ok) {
      throw new Error(`Entities list failed: ${response.status}`);
    }
    return response.json();
  }

  /**
   * Fetch codebase statistics from Parseltongue server
   *
   * # 4-Word Name: fetch_codebase_statistics_summary
   *
   * # Contract
   * - Preconditions: Server is running, database is loaded
   * - Postconditions: Returns StatisticsOverviewResponsePayload with counts
   * - Error Conditions: Throws if server unreachable or database not loaded
   * - Performance: Must complete in <100ms
   */
  async fetch_codebase_statistics_summary(): Promise<StatisticsOverviewResponsePayload> {
    const response = await fetch(
      `${this.baseUrl}/codebase-statistics-overview-summary`
    );
    if (!response.ok) {
      throw new Error(`Statistics failed: ${response.status}`);
    }
    return response.json();
  }

  /**
   * Fetch forward dependencies (what this entity calls/uses)
   *
   * # 4-Word Name: fetch_forward_dependencies_callees
   *
   * # Contract
   * - Preconditions: Server is running, entity exists
   * - Postconditions: Returns list of entities that this one calls
   * - Error Conditions: Throws if server unreachable or entity not found
   * - Performance: Must complete in <200ms
   *
   * @param entityKey - The entity key to fetch callees for
   */
  async fetch_forward_dependencies_callees(
    entityKey: string
  ): Promise<ForwardDependenciesResponse> {
    const response = await fetch(
      `${this.baseUrl}/forward-callees-query-graph?entity=${encodeURIComponent(entityKey)}`
    );
    // Parseltongue API returns 404 with valid JSON when no callees found
    // Parse the response regardless of status code
    const data = await response.json();
    return data as ForwardDependenciesResponse;
  }

  /**
   * Fetch backward dependencies (what calls this entity)
   *
   * # 4-Word Name: fetch_backward_dependencies_callers
   *
   * # Contract
   * - Preconditions: Server is running, entity exists
   * - Postconditions: Returns list of entities that call this one
   * - Error Conditions: Throws if server unreachable or entity not found
   * - Performance: Must complete in <200ms
   *
   * @param entityKey - The entity key to fetch callers for
   */
  async fetch_backward_dependencies_callers(
    entityKey: string
  ): Promise<BackwardDependenciesResponse> {
    const response = await fetch(
      `${this.baseUrl}/reverse-callers-query-graph?entity=${encodeURIComponent(entityKey)}`
    );
    // Parseltongue API returns 404 with valid JSON when no callers found
    // Parse the response regardless of status code
    const data = await response.json();
    return data as BackwardDependenciesResponse;
  }

  /**
   * Fetch both forward and backward dependencies for an entity
   *
   * # 4-Word Name: fetch_both_entity_dependencies
   *
   * # Contract
   * - Preconditions: Server is running, entity exists
   * - Postconditions: Returns combined forward and backward dependencies
   * - Error Conditions: Throws if server unreachable or entity not found
   * - Performance: Must complete in <400ms (parallel requests)
   *
   * @param entityKey - The entity key to fetch dependencies for
   */
  async fetch_both_entity_dependencies(
    entityKey: string
  ): Promise<EntityDependencies> {
    const [forward, backward] = await Promise.all([
      this.fetch_forward_dependencies_callees(entityKey),
      this.fetch_backward_dependencies_callers(entityKey),
    ]);

    // Transform to dependency edges
    // Handle both success:false responses (no data field) and empty arrays
    // Note: API returns "callees" and "callers", not "forward_callees"/"reverse_callers"
    const forwardCallees = forward.data?.callees || forward.data?.forward_callees || [];
    const backwardCallers = backward.data?.callers || backward.data?.reverse_callers || [];

    const forwardEdges: DependencyEdge[] = forwardCallees.map((callee: any) => ({
      from: entityKey,
      to: callee.to_key || callee.entity_key,
      edgeType: callee.edge_type || callee.edgeType || 'calls',
    }));

    const backwardEdges: DependencyEdge[] = backwardCallers.map((caller: any) => ({
      from: caller.from_key || caller.entity_key,
      to: entityKey,
      edgeType: caller.edge_type || caller.edgeType || 'calls',
    }));

    return {
      entityKey,
      forward: forwardEdges,
      backward: backwardEdges,
    };
  }

  /**
   * Legacy method: getHealthCheck
   *
   * # 4-Word Name: get_health_check_status_legacy
   *
   * @deprecated Use fetch_server_health_check_status instead
   */
  async getHealthCheck(): Promise<HealthCheckResponsePayload> {
    return this.fetch_server_health_check_status();
  }

  /**
   * Legacy method: getEntitiesList
   *
   * # 4-Word Name: get_entities_list_legacy
   *
   * @deprecated Use fetch_all_code_entities_list instead
   */
  async getEntitiesList(
    params?: EntitiesListQueryParams
  ): Promise<EntitiesListResponsePayload> {
    return this.fetch_all_code_entities_list(params);
  }

  /**
   * Legacy method: getStatistics
   *
   * # 4-Word Name: get_statistics_summary_legacy
   *
   * @deprecated Use fetch_codebase_statistics_summary instead
   */
  async getStatistics(): Promise<StatisticsOverviewResponsePayload> {
    return this.fetch_codebase_statistics_summary();
  }
}
