/**
 * TDD: API Client Tests
 *
 * Test-first approach for Parseltongue API client
 * Following STUB -> RED -> GREEN -> REFACTOR cycle
 */

import { describe, it, expect, beforeAll } from 'vitest';
import { ParseltongueApiClient } from './parseltongue_api_client';

describe('ParseltongueApiClient', () => {
  const API_BASE = 'http://localhost:7777';
  let client: ParseltongueApiClient;

  beforeAll(() => {
    client = new ParseltongueApiClient(API_BASE);
  });

  describe('health check', () => {
    // STUB: Write failing test first
    it('should connect to health check endpoint', async () => {
      const response = await client.getHealthCheck();

      // THEN: Verify response structure
      expect(response).toBeDefined();
      expect(response.success).toBe(true);
      expect(response.endpoint).toBe('/server-health-check-status');
      expect(response.status).toBe('ok');
    });

    it('should return within 100ms', async () => {
      const start = performance.now();
      await client.getHealthCheck();
      const elapsed = performance.now() - start;

      // Performance contract
      expect(elapsed).toBeLessThan(100);
    });
  });

  describe('entities list', () => {
    it('should fetch entities list', async () => {
      const response = await client.getEntitiesList();

      // THEN: Verify response structure
      expect(response).toBeDefined();
      expect(response.success).toBe(true);
      expect(response.endpoint).toBe('/code-entities-list-all');
      expect(response.data).toBeDefined();
      expect(response.data.entities).toBeInstanceOf(Array);
      expect(response.data.total_count).toBeGreaterThanOrEqual(0);
    });

    it('should include lines_of_code field in each entity', async () => {
      const response = await client.getEntitiesList();

      // THEN: Verify LOC field exists (our new API addition)
      response.data.entities.forEach((entity) => {
        expect(entity).toHaveProperty('lines_of_code');
        // LOC can be null (no code stored) or a number
        expect(
          entity.lines_of_code === null || typeof entity.lines_of_code === 'number'
        ).toBe(true);
      });
    });

    it('should filter by entity type when specified', async () => {
      const response = await client.getEntitiesList({ entity_type: 'Function' });

      // THEN: Verify filter worked
      response.data.entities.forEach((entity) => {
        expect(entity.entity_type).toBe('Function');
      });
    });
  });

  describe('statistics', () => {
    it('should fetch codebase statistics', async () => {
      const response = await client.getStatistics();

      // THEN: Verify response structure
      expect(response).toBeDefined();
      expect(response.success).toBe(true);
      expect(response.endpoint).toBe('/codebase-statistics-overview-summary');
      expect(response.data).toBeDefined();
      expect(response.data.code_entities_total_count).toBeGreaterThanOrEqual(0);
      expect(response.data.test_entities_total_count).toBeGreaterThanOrEqual(0);
      expect(response.data.dependency_edges_total_count).toBeGreaterThanOrEqual(0);
      expect(response.data.languages_detected_list).toBeInstanceOf(Array);
    });
  });

  describe('error handling', () => {
    it('should handle server unavailable gracefully', async () => {
      const badClient = new ParseltongueApiClient('http://localhost:9999');

      // THEN: Should throw or return error response
      await expect(badClient.getHealthCheck()).rejects.toThrow();
    });
  });
});
