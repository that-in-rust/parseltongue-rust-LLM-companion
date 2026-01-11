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
