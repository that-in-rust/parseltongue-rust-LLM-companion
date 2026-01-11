/**
 * Parseltongue API Client
 *
 * Minimal implementation to satisfy TDD tests
 * Following RED -> GREEN -> REFACTOR cycle
 */

import type {
  HealthCheckResponsePayload,
  EntitiesListResponsePayload,
  EntitiesListQueryParams,
  StatisticsOverviewResponsePayload,
} from '../types/api';

export class ParseltongueApiClient {
  constructor(private readonly baseUrl: string) {}

  /**
   * GET /server-health-check-status
   */
  async getHealthCheck(): Promise<HealthCheckResponsePayload> {
    const response = await fetch(`${this.baseUrl}/server-health-check-status`);
    if (!response.ok) {
      throw new Error(`Health check failed: ${response.status}`);
    }
    return response.json();
  }

  /**
   * GET /code-entities-list-all
   */
  async getEntitiesList(
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
   * GET /codebase-statistics-overview-summary
   */
  async getStatistics(): Promise<StatisticsOverviewResponsePayload> {
    const response = await fetch(
      `${this.baseUrl}/codebase-statistics-overview-summary`
    );
    if (!response.ok) {
      throw new Error(`Statistics failed: ${response.status}`);
    }
    return response.json();
  }
}
