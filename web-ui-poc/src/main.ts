/**
 * Parseltongue 3D CodeCity - Proof of Concept
 * Main entry point
 */

import { ParseltongueApiClient } from './api/client';
import { CodeCityScene } from './scene/scene';

// Configuration
const API_BASE = 'http://localhost:7777';

// DOM elements
const loadingEl = document.getElementById('loading')!;
const loadingTextEl = document.getElementById('loading-text')!;
const serverStatusEl = document.getElementById('server-status')!;
const entityCountEl = document.getElementById('entity-count')!;
const fpsCounterEl = document.getElementById('fps-counter')!;
const buildingCountEl = document.getElementById('building-count')!;

// State
let scene: CodeCityScene | null = null;
let lastFrameTime = performance.now();
let frameCount = 0;
let fpsUpdateTime = 0;

/**
 * Initialize the application
 */
async function init() {
  try {
    // Step 1: Connect to Parseltongue server
    loadingTextEl.textContent = 'Connecting to Parseltongue server...';
    const client = new ParseltongueApiClient(API_BASE);

    // Step 2: Health check
    const health = await client.getHealthCheck();
    serverStatusEl.textContent = `OK (${health.endpoint})`;
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
    loadingTextEl.textContent = 'Building 3D scene...';
    scene = new CodeCityScene(document.getElementById('canvas-container')!);
    await scene.initialize(entities);

    buildingCountEl.textContent = entities.length.toString();

    // Step 6: Hide loading, start render loop
    loadingEl.style.display = 'none';
    requestAnimationFrame(renderLoop);

  } catch (error) {
    console.error('Initialization failed:', error);
    loadingTextEl.textContent = `Error: ${error instanceof Error ? error.message : 'Unknown error'}`;
    serverStatusEl.textContent = 'Failed';
    serverStatusEl.style.color = '#ef4444';
  }
}

/**
 * Main render loop
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

// Start the app
init();
