/**
 * 3D CodeCity Scene
 *
 * Minimal Three.js scene for rendering code entities as buildings
 */

import * as THREE from 'three';
import type { EntitySummaryListItem } from '../types/api';

export class CodeCityScene {
  private container: HTMLElement;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private buildings: THREE.Mesh[] = [];

  constructor(container: HTMLElement) {
    this.container = container;

    // Create scene
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0a0a0f);
    this.scene.fog = new THREE.Fog(0x0a0a0f, 100, 500);

    // Create camera
    const aspect = container.clientWidth / container.clientHeight;
    this.camera = new THREE.PerspectiveCamera(60, aspect, 0.1, 1000);
    this.camera.position.set(50, 50, 50);
    this.camera.lookAt(0, 0, 0);

    // Create renderer
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(container.clientWidth, container.clientHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.container.appendChild(this.renderer.domElement);

    // Add lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
    this.scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(50, 100, 50);
    this.scene.add(directionalLight);

    // Add grid helper
    const gridHelper = new THREE.GridHelper(200, 20, 0x27272a, 0x1f1f23);
    this.scene.add(gridHelper);

    // Handle window resize
    window.addEventListener('resize', () => this.onResize());
  }

  /**
   * Initialize scene with entities
   */
  async initialize(entities: EntitySummaryListItem[]): Promise<void> {
    // Create a building for each entity
    const gridSize = Math.ceil(Math.sqrt(entities.length));
    const spacing = 3;

    entities.forEach((entity, index) => {
      const building = this.createBuilding(entity);
      const row = Math.floor(index / gridSize);
      const col = index % gridSize;

      building.position.set(
        col * spacing - (gridSize * spacing) / 2,
        0,
        row * spacing - (gridSize * spacing) / 2
      );

      this.buildings.push(building);
      this.scene.add(building);
    });

    console.log(`Created ${this.buildings.length} buildings`);
  }

  /**
   * Create a single building mesh for an entity
   */
  private createBuilding(entity: EntitySummaryListItem): THREE.Mesh {
    // Height based on lines of code (min 1, max 20)
    const loc = entity.lines_of_code ?? 1;
    const height = Math.max(1, Math.min(20, loc * 0.3));

    // Width/depth based on entity type
    let width = 1;
    let depth = 1;

    switch (entity.entity_type.toLowerCase()) {
      case 'module':
      case 'namespace':
      case 'impl':
        width = 3;
        depth = 3;
        break;
      case 'struct':
      case 'class':
      case 'interface':
      case 'trait':
        width = 2;
        depth = 2;
        break;
      default:
        width = 1;
        depth = 1;
    }

    // Color based on language
    const color = this.getColorForLanguage(entity.language);

    // Create geometry
    const geometry = new THREE.BoxGeometry(width, height, depth);
    // Offset so building grows from ground
    geometry.translate(0, height / 2, 0);

    // Create material
    const material = new THREE.MeshStandardMaterial({
      color,
      roughness: 0.7,
      metalness: 0.1,
    });

    const mesh = new THREE.Mesh(geometry, material);

    // Store entity data on mesh for interaction
    mesh.userData.entityKey = entity.key;
    mesh.userData.entity = entity;

    return mesh;
  }

  /**
   * Get color for programming language
   */
  private getColorForLanguage(language: string): number {
    const colors: Record<string, number> = {
      rust: 0xb7410e,      // Rust orange
      python: 0x3572A5,    // Python blue
      javascript: 0xf7df1e, // JS yellow
      typescript: 0x3178c6, // TS blue
      go: 0x00add8,        // Go cyan
      java: 0xb07219,      // Java red-brown
      c: 0x555555,         // C gray
      cpp: 0x00599c,       // C++ dark blue
      ruby: 0xcc342d,      // Ruby red
      php: 0x777bb4,       // PHP purple
      csharp: 0x239120,    // C# green
      swift: 0xfa7343,     // Swift orange
    };

    return colors[language.toLowerCase()] ?? 0x6b7280; // Default gray
  }

  /**
   * Render a frame
   */
  render(_deltaTime: number): void {
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Handle window resize
   */
  private onResize(): void {
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();

    this.renderer.setSize(width, height);
  }

  /**
   * Clean up resources
   */
  dispose(): void {
    this.buildings.forEach((building) => {
      building.geometry.dispose();
      if (Array.isArray(building.material)) {
        building.material.forEach((m) => m.dispose());
      } else {
        building.material.dispose();
      }
    });

    this.renderer.dispose();
  }
}
