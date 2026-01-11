/**
 * Parseltongue 3D CodeCity - Proof of Concept
 * Main entry point
 *
 * # 4-Word Name: parseltongue_poc_main
 *
 * Features:
 * - Connects to Parseltongue API
 * - Initializes 3D scene with entities
 * - Displays entity details on selection
 * - Shows FPS and statistics
 */

import { ParseltongueApiClient, type DependencyEdge } from './api/parseltongue_api_client';
import { CodeCitySceneManager, type SelectedEntity } from './scene/code_city_scene_manager';
import type { EntitySummaryListItem } from './types/parseltongue_api_types';

// Configuration
const API_BASE = 'http://localhost:7777';

// DOM elements
const loadingEl = document.getElementById('loading')!;
const loadingTextEl = document.getElementById('loading-text')!;
const serverStatusEl = document.getElementById('server-status')!;
const entityCountEl = document.getElementById('entity-count')!;
const fpsCounterEl = document.getElementById('fps-counter')!;
const buildingCountEl = document.getElementById('building-count')!;
const detailsPanelEl = document.getElementById('details-panel')!;
const detailsTitleEl = document.getElementById('details-title')!;
const detailsContentEl = document.getElementById('details-content')!;
const closeDetailsBtn = document.getElementById('close-details')!;

// State
let scene: CodeCitySceneManager | null = null;
let client: ParseltongueApiClient | null = null;
let lastFrameTime = performance.now();
let frameCount = 0;
let fpsUpdateTime = 0;

/**
 * Initialize the application
 *
 * # 4-Word Name: initialize_poc_application
 *
 * # Contract
 * - Preconditions: Parseltongue server running at API_BASE
 * - Postconditions: Scene initialized, render loop running
 * - Error Conditions: Shows error message if server unavailable
 */
async function init() {
  try {
    // Step 1: Connect to Parseltongue server
    loadingTextEl.textContent = 'Connecting to Parseltongue server...';
    client = new ParseltongueApiClient(API_BASE);

    // Step 2: Health check
    await client.getHealthCheck();
    serverStatusEl.textContent = 'OK';
    serverStatusEl.style.color = '#22c55e';

    // Step 3: Fetch entities
    loadingTextEl.textContent = 'Loading codebase entities...';
    const entitiesResponse = await client.getEntitiesList();
    const entities = entitiesResponse.data.entities;

    entityCountEl.textContent = entities.length.toString();

    // Step 4: Fetch statistics
    const statsResponse = await client.getStatistics();
    console.log('Codebase stats:', statsResponse.data);

    // Step 5: Initialize Three.js scene
    loadingTextEl.textContent = 'Building 3D city...';
    const container = document.getElementById('canvas-container')!;
    scene = new CodeCitySceneManager(container);

    // Register selection callback
    scene.onSelection(handleSelection);

    await scene.initialize(entities);

    const stats = scene.getStats();
    buildingCountEl.textContent = stats.buildings.toString();
    // Districts removed - using circular layout instead

    // Step 6: Load all dependencies and display them
    loadingTextEl.textContent = 'Loading dependency connections...';
    await loadAllDependencies(entities);

    // Step 7: Hide loading, start render loop
    loadingEl.style.display = 'none';
    requestAnimationFrame(renderLoop);

    console.log(`✅ Initialized with ${entities.length} entities in circular layout`);

  } catch (error) {
    console.error('Initialization failed:', error);
    const message = error instanceof Error ? error.message : 'Unknown error';
    loadingTextEl.textContent = `Error: ${message}`;

    if (message.includes('fetch') || message.includes('ECONNREFUSED')) {
      loadingTextEl.textContent = 'Cannot connect to Parseltongue server. Is pt08 running on port 7777?';
    }

    serverStatusEl.textContent = 'Failed';
    serverStatusEl.style.color = '#ef4444';
  }
}

/**
 * Handle entity selection
 *
 * # 4-Word Name: handle_entity_selection_event
 */
async function handleSelection(selected: SelectedEntity | null): Promise<void> {
  if (selected) {
    showDetails(selected.entity);
    // Fetch and display dependencies for selected entity
    await loadAndShowDependencies(selected.entity.key);
  } else {
    hideDetails();
    // Don't clear dependency edges - keep all connections visible
  }
}

/**
 * Load and display dependency edges for an entity
 *
 * # 4-Word Name: load_and_show_dependencies
 *
 * # Contract
 * - Preconditions: API server running, entity key valid
 * - Postconditions: Dependency edges rendered in 3D scene
 * - Error Conditions: Silently fails on network error
 */
async function loadAndShowDependencies(entityKey: string): Promise<void> {
  if (!client) return;
  try {
    const deps = await client.fetch_both_entity_dependencies(entityKey);
    scene?.showDependencyEdges(deps.forward, deps.backward);

    // Update status with dependency counts
    const forwardCount = deps.forward.length;
    const backwardCount = deps.backward.length;
    console.log(`Dependencies: ${forwardCount} forward, ${backwardCount} backward`);
  } catch (error) {
    console.error('Failed to load dependencies:', error);
  }
}

/**
 * Show entity details panel
 *
 * # 4-Word Name: show_entity_details_panel
 */
function showDetails(entity: EntitySummaryListItem): void {
  // Extract readable name
  const name = extractEntityName(entity);

  // Build details HTML
  const loc = entity.lines_of_code ?? 0;
  const filePath = formatFilePath(entity.file_path);

  detailsContentEl.innerHTML = `
    <div class="detail-row">
      <span class="detail-label">Type:</span>
      <span class="detail-value">${entity.entity_type}</span>
    </div>
    <div class="detail-row">
      <span class="detail-label">Language:</span>
      <span class="detail-value">${entity.language}</span>
    </div>
    <div class="detail-row">
      <span class="detail-label">Class:</span>
      <span class="detail-value">${entity.entity_class}</span>
    </div>
    <div class="detail-row">
      <span class="detail-label">Lines of Code:</span>
      <span class="detail-value">${loc}</span>
    </div>
    <div class="detail-row">
      <span class="detail-label">File:</span>
      <span class="detail-value detail-file">${filePath}</span>
    </div>
    <div class="detail-row">
      <span class="detail-label">Key:</span>
      <span class="detail-value detail-key">${entity.key}</span>
    </div>
  `;

  detailsTitleEl.textContent = name;
  detailsPanelEl.style.display = 'block';
}

/**
 * Hide entity details panel
 *
 * # 4-Word Name: hide_entity_details_panel
 */
function hideDetails(): void {
  detailsPanelEl.style.display = 'none';
}

/**
 * Extract readable entity name from key
 *
 * # 4-Word Name: extract_entity_name_from_key
 */
function extractEntityName(entity: EntitySummaryListItem): string {
  const parts = entity.key.split(':');
  if (parts.length >= 3) {
    const name = parts[2];
    return name.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
  }
  return entity.entity_type;
}

/**
 * Format file path for display
 *
 * # 4-Word Name: format_file_path_for_display
 */
function formatFilePath(path: string): string {
  // Get last 2-3 path components
  const parts = path.split('/');
  if (parts.length > 3) {
    return '.../' + parts.slice(-3).join('/');
  }
  return path;
}

/**
 * Load all dependencies from the API and display them
 *
 * # 4-Word Name: load_all_dependencies_api
 *
 * Fetches dependencies for a subset of entities (to avoid API overload)
 * and displays all connection arcs.
 */
async function loadAllDependencies(entities: EntitySummaryListItem[]): Promise<void> {
  if (!client || !scene) return;

  try {
    const allEdges: DependencyEdge[] = [];
    const entityKeys = new Set(entities.map(e => e.key));

    console.log(`Loading dependencies for ${entities.length} entities...`);

    // Fetch dependencies for each entity (query all to ensure edges exist)
    for (let i = 0; i < entities.length; i++) {
      const entity = entities[i];
      try {
        const deps = await client.fetch_both_entity_dependencies(entity.key);

        // Convert to DependencyEdge format - only include edges where BOTH endpoints exist
        deps.forward.forEach(fwd => {
          if (entityKeys.has(fwd.to)) {
            allEdges.push({ from: fwd.from, to: fwd.to, edgeType: fwd.edgeType });
          }
        });
        deps.backward.forEach(bwd => {
          if (entityKeys.has(bwd.from)) {
            allEdges.push({ from: bwd.from, to: bwd.to, edgeType: bwd.edgeType });
          }
        });
      } catch (err) {
        // Skip failed entities
        console.warn(`Failed to load dependencies for ${entity.key}:`, err);
      }
    }

    console.log(`Loaded ${allEdges.length} dependency edges (filtered to existing entities)`);

    // Show all edges using the arc method
    scene.showAllDependencyEdges(allEdges);
  } catch (error) {
    console.error('Failed to load all dependencies:', error);
  }
}

/**
 * Main render loop
 *
 * # 4-Word Name: main_render_loop_update
 */
function renderLoop(currentTime: number) {
  requestAnimationFrame(renderLoop);

  // FPS calculation
  frameCount++;
  const deltaTime = currentTime - lastFrameTime;

  if (currentTime - fpsUpdateTime >= 1000) {
    const fps = Math.round((frameCount * 1000) / (currentTime - fpsUpdateTime));
    fpsCounterEl.textContent = fps.toString();
    frameCount = 0;
    fpsUpdateTime = currentTime;
  }

  lastFrameTime = currentTime;

  // Render scene
  scene?.render(deltaTime);
}

// Setup close button handler
closeDetailsBtn.addEventListener('click', hideDetails);

// Start the app
init();
