/**
 * 3D CodeCity Scene Manager - Enhanced
 *
 * Implements CodeCity visualization metaphor:
 * - Buildings = code entities (height = LOC, color = type)
 * - Circular periphery layout for dependency visualization
 * - OrbitControls for navigation
 * - Hover labels and click selection
 *
 * # 4-Word Name: code_city_scene_manager
 *
 * # Contract
 * - Preconditions: Container element exists, entities array non-empty
 * - Postconditions: Scene initialized with buildings in circular layout, camera controllable
 * - Error Conditions: Throws if container invalid or WebGL unavailable
 * - Performance: Target 60 FPS with 10k buildings
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { EntitySummaryListItem } from '../types/parseltongue_api_types';
import type { DependencyEdge } from '../api/parseltongue_api_client';

/**
 * Entity visual specification
 */
interface BuildingSpec {
  entity: EntitySummaryListItem;
  position: THREE.Vector3;
  dimensions: THREE.Vector3;
  color: number;
}

/**
 * Selection data for UI display
 */
export interface SelectedEntity {
  entity: EntitySummaryListItem;
  position: THREE.Vector3;
}

/**
 * Callback type for selection events
 */
export type SelectionCallback = (selected: SelectedEntity | null) => void;

/**
 * Enhanced CodeCity scene manager with full interactivity
 *
 * Features:
 * - Circular periphery layout for dependency visualization
 * - OrbitControls for smooth camera navigation
 * - Hover tooltips showing entity names
 * - Click selection with highlight
 * - Ambient lighting (natural light, no directional shadows)
 * - Dependency edges with straight lines and end markers
 */
export class CodeCitySceneManager {
  private container: HTMLElement;
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private controls: OrbitControls;
  private buildings: THREE.Mesh[] = [];
  private raycaster: THREE.Raycaster;
  private pointer: THREE.Vector2;
  private hoveredMesh: THREE.Mesh | null = null;
  private selectedMesh: THREE.Mesh | null = null;
  private highlightMesh: THREE.Mesh | null = null;
  private tooltip: HTMLElement | null = null;
  private onSelectionCallback: SelectionCallback | null = null;

  // Dependency edges visualization
  private edgeGroup: THREE.Group | null = null;
  private buildingByKey: Map<string, THREE.Mesh> = new Map();

  // Color palette - vibrant neon for dark theme (cyberpunk code city)
  private readonly colors: Record<string, number> = {
    // Entity type colors - vibrant neon palette
    // Functions/Methods - Cyan Neon
    function: 0x00fff5,
    fn: 0x00fff5,
    method: 0x00d4ff,

    // Structs/Classes - Neon Green
    struct: 0x39ff14,
    class: 0x39ff14,
    impl: 0x5fff5f,

    // Traits/Interfaces - Neon Purple
    trait: 0xbf00ff,
    interface: 0xbf00ff,

    // Enums - Neon Orange
    enum: 0xff6b00,

    // Modules/Namespaces - Neon Pink
    module: 0xff00aa,
    namespace: 0xff00aa,

    // Constants/Variables - Neon Yellow
    const: 0xffea00,
    variable: 0xffff55,

    // Default for unknown types - Gray
    default: 0x6b7280,

    // UI colors
    highlight: 0x39ff14,
    hover: 0xffea00,
    selection: 0x00fff5,

    // === RELATIONSHIP-TYPE BASED EDGE COLORS ===
    // Developer journey: visually distinguish relationship types

    // "Calls" relationship - Electric Blue (function invocation)
    edgeCalls: 0x00d4ff,

    // "Implements" relationship - Neon Purple (inheritance)
    edgeImplements: 0xbf00ff,

    // "Imports" relationship - Neon Green (module dependency)
    edgeImports: 0x39ff14,

    // "References" relationship - Neon Orange (loose coupling)
    edgeReferences: 0xff6b00,

    // "Contains" relationship - Neon Pink (ownership)
    edgeContains: 0xff00aa,

    // "Defines" relationship - Neon Yellow (declaration)
    edgeDefines: 0xffea00,

    // Default edge color
    edgeDefault: 0x00d4ff,
  };

  // Map edge_type strings to colors
  private readonly edgeColorMap: Record<string, number> = {
    'Calls': 0x00d4ff,           // Electric Blue
    'CALLS': 0x00d4ff,
    'calls': 0x00d4ff,
    'Implements': 0xbf00ff,      // Neon Purple
    'IMPLEMENTS': 0xbf00ff,
    'implements': 0xbf00ff,
    'Imports': 0x39ff14,         // Neon Green
    'IMPORTS': 0x39ff14,
    'imports': 0x39ff14,
    'References': 0xff6b00,      // Neon Orange
    'REFERENCES': 0xff6b00,
    'references': 0xff6b00,
    'Contains': 0xff00aa,        // Neon Pink
    'CONTAINS': 0xff00aa,
    'contains': 0xff00aa,
    'Defines': 0xffea00,         // Neon Yellow
    'DEFINES': 0xffea00,
    'defines': 0xffea00,
  };

  constructor(container: HTMLElement) {
    this.container = container;

    // Validate container
    if (!container) {
      throw new Error('Container element is required');
    }

    // Create scene with darker background for better contrast
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x1a1a2e); // Dark blue-gray
    this.scene.fog = new THREE.Fog(0x1a1a2e, 150, 600);

    // Create camera
    const aspect = container.clientWidth / container.clientHeight;
    this.camera = new THREE.PerspectiveCamera(60, aspect, 0.1, 2000);
    this.camera.position.set(80, 80, 80);

    // Create renderer
    this.renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
      powerPreference: 'high-performance',
    });
    this.renderer.setSize(container.clientWidth, container.clientHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.container.appendChild(this.renderer.domElement);

    // Create OrbitControls
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;
    this.controls.rotateSpeed = 0.5;
    this.controls.zoomSpeed = 1.0;
    this.controls.panSpeed = 0.5;
    this.controls.minDistance = 10;
    this.controls.maxDistance = 500;
    this.controls.maxPolarAngle = Math.PI / 2.1; // Prevent going below ground

    // Setup raycaster for interaction
    this.raycaster = new THREE.Raycaster();
    this.raycaster.params.Points = { threshold: 0.5 };
    this.pointer = new THREE.Vector2();

    // Setup lighting
    this.setupLighting();

    // Setup ground
    this.setupGround();

    // Setup interaction handlers
    this.setupInteraction();

    // Handle window resize
    window.addEventListener('resize', () => this.onResize());
  }

  /**
   * Setup ambient natural lighting (no directional sunlight)
   *
   * # 4-Word Name: setup_scene_lighting_rig
   */
  private setupLighting(): void {
    // Ambient light for overall natural illumination
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.2);
    this.scene.add(ambientLight);
  }

  /**
   * Setup ground plane with district visualization
   *
   * # 4-Word Name: setup_ground_plane_grid
   */
  private setupGround(): void {
    // Main ground plane - darker to match scene
    const groundGeometry = new THREE.PlaneGeometry(500, 500);
    const groundMaterial = new THREE.MeshStandardMaterial({
      color: 0x16213e,
      roughness: 0.9,
      metalness: 0.1,
    });
    const ground = new THREE.Mesh(groundGeometry, groundMaterial);
    ground.rotation.x = -Math.PI / 2;
    this.scene.add(ground);

    // Grid helper with darker colors
    const gridHelper = new THREE.GridHelper(500, 50, 0x4a5568, 0x2d3748);
    this.scene.add(gridHelper);
  }

  /**
   * Setup mouse/touch interaction handlers
   *
   * # 4-Word Name: setup_mouse_interaction_handlers
   */
  private setupInteraction(): void {
    // Mouse move for hover
    this.renderer.domElement.addEventListener('pointermove', (e) => {
      const rect = this.renderer.domElement.getBoundingClientRect();
      this.pointer.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
      this.pointer.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
      this.onHover();
    });

    // Click for selection
    this.renderer.domElement.addEventListener('click', () => {
      this.onClick();
    });
  }

  /**
   * Initialize scene with entities using circular periphery layout
   *
   * # 4-Word Name: initialize_scene_with_entities
   *
   * # Contract
   * - Preconditions: entities array non-empty
   * - Postconditions: Scene populated with buildings in circular layout
   * - Error Conditions: Throws if entities empty
   */
  async initialize(entities: EntitySummaryListItem[]): Promise<void> {
    if (!entities || entities.length === 0) {
      throw new Error('Entities array cannot be empty');
    }

    // Clear existing buildings and edges
    this.clearBuildings();
    this.clearDependencyEdges();
    this.buildingByKey.clear();

    // Create circular layout - buildings on periphery
    const specs = this.layoutEntitiesInCircle(entities);
    console.log(`Created circular layout for ${entities.length} entities`);

    // Create building meshes
    specs.forEach((spec) => {
      const building = this.createBuilding(spec);
      this.buildings.push(building);
      this.scene.add(building);

      // Store in map for edge lookup
      this.buildingByKey.set(spec.entity.key, building);
    });

    console.log(`Created ${this.buildings.length} buildings`);

    // Position camera to view entire city from above
    this.fitCameraToCircularCity();
  }

  /**
   * Layout entities in a circular periphery arrangement grouped by module
   *
   * # 4-Word Name: layout_entities_in_circle
   *
   * Buildings are placed on the circumference of a circle, grouped by module.
   * This makes related entities appear together on the circle.
   */
  private layoutEntitiesInCircle(entities: EntitySummaryListItem[]): BuildingSpec[] {
    const specs: BuildingSpec[] = [];
    const entityCount = entities.length;

    // Group entities by module/package for organized layout
    const moduleMap = new Map<string, EntitySummaryListItem[]>();
    entities.forEach((entity) => {
      const moduleName = this.extractModuleName(entity);
      if (!moduleMap.has(moduleName)) {
        moduleMap.set(moduleName, []);
      }
      moduleMap.get(moduleName)!.push(entity);
    });

    const modules = Array.from(moduleMap.entries());
    console.log(`Found ${modules.length} modules: ${modules.map(([name]) => name).join(', ')}`);

    // Calculate circle radius
    const minSpacing = 4;
    const circumference = entityCount * minSpacing;
    const radius = Math.max(40, circumference / (2 * Math.PI));

    // Place entities on circle, grouped by module
    let currentIndex = 0;
    modules.forEach(([moduleName, moduleEntities]) => {
      console.log(`Placing module "${moduleName}" with ${moduleEntities.length} entities`);
      moduleEntities.forEach((entity) => {
        const angle = (currentIndex / entityCount) * 2 * Math.PI;

        const x = Math.cos(angle) * radius;
        const z = Math.sin(angle) * radius;

        const position = new THREE.Vector3(x, 0, z);
        const dimensions = this.calculateDimensions(entity);
        // Color based on entity type (function, struct, class, etc.)
        const entityType = entity.entity_type.toLowerCase();
        const color = this.colors[entityType] ?? this.colors.default;

        specs.push({ entity, position, dimensions, color });
        currentIndex++;
      });
    });

    console.log(`Circular layout: radius=${radius.toFixed(1)}, ${entityCount} buildings, ${modules.length} modules`);
    return specs;
  }

  /**
   * Extract module name from entity file path
   *
   * # 4-Word Name: extract_module_name_from_path
   */
  private extractModuleName(entity: EntitySummaryListItem): string {
    const parts = entity.file_path.split('/');
    if (parts.length >= 2) {
      const parentDir = parts[parts.length - 2];
      if (parentDir === 'src') return 'main';
      if (parentDir.startsWith('crates') && parts.length >= 3) {
        return parts[parts.length - 2]; // Return crate name
      }
      return parentDir;
    }
    return 'root';
  }

  /**
   * Fit camera to view circular city from above
   *
   * # 4-Word Name: fit_camera_to_circular_city
   */
  private fitCameraToCircularCity(): void {
    if (this.buildings.length === 0) return;

    // Position camera above and back to see the whole circle
    // Circle radius is roughly 30-50 units based on entity count
    const entityCount = this.buildings.length;
    const radius = Math.max(30, (entityCount * 4) / (2 * Math.PI));

    // Position camera high above, looking down at an angle
    this.camera.position.set(0, radius * 2.5, radius * 1.5);
    this.controls.target.set(0, 0, 0);
    this.controls.update();

    console.log(`Camera positioned for circular city: radius=${radius.toFixed(1)}`);
  }

  /**
   * Calculate building dimensions from entity
   *
   * # 4-Word Name: calculate_building_dimensions
   */
  private calculateDimensions(entity: EntitySummaryListItem): THREE.Vector3 {
    // Height based on lines of code (min 1, max 30)
    const loc = entity.lines_of_code ?? 1;
    const height = Math.max(1, Math.min(30, loc * 0.2));

    // Width/depth based on entity type
    let width = 1;
    let depth = 1;

    switch (entity.entity_type.toLowerCase()) {
      case 'module':
      case 'namespace':
      case 'impl':
        width = 4;
        depth = 4;
        break;
      case 'struct':
      case 'class':
      case 'interface':
      case 'trait':
        width = 2.5;
        depth = 2.5;
        break;
      case 'enum':
        width = 2;
        depth = 2;
        break;
      case 'function':
      case 'fn':
      case 'method':
        width = 1.5;
        depth = 1.5;
        break;
      default:
        width = 1;
        depth = 1;
    }

    return new THREE.Vector3(width, height, depth);
  }

  /**
   * Create a single building mesh
   *
   * # 4-Word Name: create_building_mesh_from_spec
   */
  private createBuilding(spec: BuildingSpec): THREE.Mesh {
    const { position, dimensions, color, entity } = spec;

    // Create geometry
    const geometry = new THREE.BoxGeometry(dimensions.x, dimensions.y, dimensions.z);
    // Offset so building grows from ground
    geometry.translate(0, dimensions.y / 2, 0);

    // Create material with slight variation for visual interest
    const material = new THREE.MeshStandardMaterial({
      color,
      roughness: 0.7,
      metalness: 0.1,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.copy(position);

    // Store entity data for interaction
    mesh.userData.entityKey = entity.key;
    mesh.userData.entity = entity;
    mesh.userData.originalColor = color;

    return mesh;
  }

  /**
   * Handle hover over buildings
   *
   * # 4-Word Name: handle_hover_over_building
   */
  private onHover(): void {
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const intersects = this.raycaster.intersectObjects(this.buildings);

    // Reset previous hover
    if (this.hoveredMesh && this.hoveredMesh !== this.selectedMesh) {
      const mat = this.hoveredMesh.material as THREE.MeshStandardMaterial;
      mat.emissive.setHex(0x000000);
    }

    if (intersects.length > 0) {
      const mesh = intersects[0].object as THREE.Mesh;
      this.hoveredMesh = mesh;

      // Highlight hovered building
      if (mesh !== this.selectedMesh) {
        const mat = mesh.material as THREE.MeshStandardMaterial;
        mat.emissive.setHex(0x222222);
      }

      // Show tooltip
      this.showTooltip(mesh, intersects[0].point);
      this.renderer.domElement.style.cursor = 'pointer';
    } else {
      this.hoveredMesh = null;
      this.hideTooltip();
      this.renderer.domElement.style.cursor = 'grab';
    }
  }

  /**
   * Handle click for selection
   *
   * # 4-Word Name: handle_click_for_selection
   */
  private onClick(): void {
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const intersects = this.raycaster.intersectObjects(this.buildings);

    // Reset previous selection
    if (this.selectedMesh) {
      const mat = this.selectedMesh.material as THREE.MeshStandardMaterial;
      mat.emissive.setHex(0x000000);
    }
    if (this.highlightMesh) {
      this.scene.remove(this.highlightMesh);
      this.highlightMesh = null;
    }

    if (intersects.length > 0) {
      const mesh = intersects[0].object as THREE.Mesh;
      this.selectedMesh = mesh;

      // Highlight selected building
      const mat = mesh.material as THREE.MeshStandardMaterial;
      mat.emissive.setHex(this.colors.selection);

      // Add selection ring
      this.addSelectionRing(mesh);

      // Notify callback
      if (this.onSelectionCallback) {
        const entity = mesh.userData.entity as EntitySummaryListItem;
        this.onSelectionCallback({
          entity,
          position: mesh.position.clone(),
        });
      }
    } else {
      this.selectedMesh = null;
      if (this.onSelectionCallback) {
        this.onSelectionCallback(null);
      }
    }
  }

  /**
   * Add selection ring under building
   *
   * # 4-Word Name: add_selection_ring_under_building
   */
  private addSelectionRing(mesh: THREE.Mesh): void {
    const ringGeo = new THREE.RingGeometry(1.5, 2, 32);
    const ringMat = new THREE.MeshBasicMaterial({
      color: this.colors.selection,
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.8,
    });
    this.highlightMesh = new THREE.Mesh(ringGeo, ringMat);
    this.highlightMesh.rotation.x = -Math.PI / 2;
    this.highlightMesh.position.set(mesh.position.x, 0.1, mesh.position.z);
    this.scene.add(this.highlightMesh);
  }

  /**
   * Show tooltip for hovered building
   *
   * # 4-Word Name: show_tooltip_for_entity
   */
  private showTooltip(mesh: THREE.Mesh, position: THREE.Vector3): void {
    const entity = mesh.userData.entity as EntitySummaryListItem;

    // Create tooltip if not exists
    if (!this.tooltip) {
      this.tooltip = document.createElement('div');
      this.tooltip.className = 'codecity-tooltip';
      document.body.appendChild(this.tooltip);
    }

    // Set tooltip content
    const entityName = this.extractEntityName(entity);
    const loc = entity.lines_of_code ?? 0;

    this.tooltip.innerHTML = `
      <div class="tooltip-title">${entityName}</div>
      <div class="tooltip-detail">${entity.entity_type} • ${entity.language}</div>
      <div class="tooltip-detail">${loc} LOC</div>
    `;

    // Position tooltip
    const screenPos = position.clone().project(this.camera);
    const x = (screenPos.x * 0.5 + 0.5) * window.innerWidth;
    const y = (-screenPos.y * 0.5 + 0.5) * window.innerHeight;

    this.tooltip.style.left = `${x + 15}px`;
    this.tooltip.style.top = `${y - 10}px`;
    this.tooltip.style.display = 'block';
  }

  /**
   * Hide tooltip
   *
   * # 4-Word Name: hide_tooltip_overlay
   */
  private hideTooltip(): void {
    if (this.tooltip) {
      this.tooltip.style.display = 'none';
    }
  }

  /**
   * Extract readable entity name from key
   *
   * # 4-Word Name: extract_entity_name_from_key
   */
  private extractEntityName(entity: EntitySummaryListItem): string {
    // Parse entity key to get name
    // Format: rust:fn:function_name:path:line-range
    const parts = entity.key.split(':');
    if (parts.length >= 3) {
      const name = parts[2];
      // Clean up path artifacts
      return name.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
    }
    return entity.entity_type;
  }

  /**
   * Register callback for selection events
   *
   * # 4-Word Name: register_selection_callback
   */
  onSelection(callback: SelectionCallback): void {
    this.onSelectionCallback = callback;
  }

  /**
   * Render a frame
   *
   * # 4-Word Name: render_scene_frame_update
   */
  render(_deltaTime: number): void {
    // Update controls for damping
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  /**
   * Handle window resize
   *
   * # 4-Word Name: handle_window_resize_event
   */
  private onResize(): void {
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;

    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();

    this.renderer.setSize(width, height);
  }

  /**
   * Clear all buildings from scene
   *
   * # 4-Word Name: clear_all_buildings_from_scene
   */
  private clearBuildings(): void {
    this.buildings.forEach((building) => {
      this.scene.remove(building);
      building.geometry.dispose();
      if (Array.isArray(building.material)) {
        building.material.forEach((m) => m.dispose());
      } else {
        building.material.dispose();
      }
    });
    this.buildings = [];
  }

  /**
   * Clean up resources
   *
   * # 4-Word Name: dispose_scene_resources
   */
  dispose(): void {
    this.clearBuildings();
    this.clearDependencyEdges();

    if (this.highlightMesh) {
      this.scene.remove(this.highlightMesh);
      this.highlightMesh.geometry.dispose();
      (this.highlightMesh.material as THREE.Material).dispose();
    }

    this.controls.dispose();
    this.renderer.dispose();

    if (this.tooltip) {
      this.tooltip.remove();
    }
  }

  /**
   * Show all dependency edges as curved arcs through circle center
   *
   * # 4-Word Name: show_all_dependency_edges_arcs
   *
   * Creates curved arcs through the center of the circle to show connections.
   * All dependencies shown by default for visual overview.
   *
   * @param allEdges - All dependency edges between entities
   */
  showAllDependencyEdges(allEdges: DependencyEdge[]): void {
    // Clear existing edges
    this.clearDependencyEdges();

    // Create edge group
    this.edgeGroup = new THREE.Group();

    console.log(`Creating ${allEdges.length} dependency arcs`);

    // Calculate circle radius from first building position
    let radius = 40;
    if (this.buildings.length > 0) {
      const firstPos = this.buildings[0].position;
      radius = Math.sqrt(firstPos.x * firstPos.x + firstPos.z * firstPos.z);
    }

    const arcHeight = radius * 0.5; // Arc goes further toward center for visibility

    let created = 0;
    let skipped = 0;

    // Count edge types for debugging
    const edgeTypes = new Map<string, number>();

    allEdges.forEach((edge) => {
      const fromMesh = this.buildingByKey.get(edge.from);
      const toMesh = this.buildingByKey.get(edge.to);

      if (fromMesh && toMesh && this.edgeGroup) {
        // Get color based on relationship type
        const edgeColor = this.getColorForEdgeType(edge.edgeType);
        edgeTypes.set(edge.edgeType, (edgeTypes.get(edge.edgeType) || 0) + 1);

        this.createArcEdge(
          fromMesh.position,
          toMesh.position,
          radius,
          arcHeight,
          this.edgeGroup,
          edgeColor
        );
        created++;
      } else {
        skipped++;
      }
    });

    // Log edge type distribution
    const edgeTypesObj: Record<string, number> = {};
    edgeTypes.forEach((count, type) => {
      edgeTypesObj[type] = count;
    });
    console.log(`Edge types:`, edgeTypesObj);
    console.log(`Rendered ${created} dependency arcs, ${skipped} skipped (missing buildings)`);

    // Add edge group to scene
    this.scene.add(this.edgeGroup);
  }

  /**
   * Get color for edge based on relationship type
   *
   * # 4-Word Name: get_color_for_edge_type
   */
  private getColorForEdgeType(edgeType: string): number {
    // First try exact match in edgeColorMap
    if (this.edgeColorMap[edgeType]) {
      return this.edgeColorMap[edgeType];
    }

    // Fuzzy matching for variations
    const typeLower = edgeType.toLowerCase();
    if (typeLower.includes('call')) return this.colors.edgeCalls;
    if (typeLower.includes('implement')) return this.colors.edgeImplements;
    if (typeLower.includes('import')) return this.colors.edgeImports;
    if (typeLower.includes('reference')) return this.colors.edgeReferences;
    if (typeLower.includes('contain')) return this.colors.edgeContains;
    if (typeLower.includes('define')) return this.colors.edgeDefines;

    // Default - electric blue
    return this.colors.edgeDefault;
  }

  /**
   * Show dependency edges for selected entity (curved arcs)
   *
   * # 4-Word Name: show_dependency_edges_for_entity
   *
   * # Contract
   * - Preconditions: Entity key exists in buildingByKey, edges array valid
   * - Postconditions: Curved arc lines added to scene through circle center
   * - Error Conditions: Silently fails if building not found
   *
   * @param forwardEdges - Dependencies from this entity (what it calls)
   * @param backwardEdges - Dependencies to this entity (what calls it)
   */
  showDependencyEdges(forwardEdges: DependencyEdge[], backwardEdges: DependencyEdge[]): void {
    // Clear existing edges
    this.clearDependencyEdges();

    // Create edge group
    this.edgeGroup = new THREE.Group();

    console.log(`showDependencyEdges called: ${forwardEdges.length} forward, ${backwardEdges.length} backward`);

    // Calculate circle radius from buildings
    let radius = 40;
    if (this.buildings.length > 0) {
      const firstPos = this.buildings[0].position;
      radius = Math.sqrt(firstPos.x * firstPos.x + firstPos.z * firstPos.z);
    }

    const arcHeight = radius * 0.3; // Arc goes partly toward center

    // Combine all edges
    const allEdges = [...forwardEdges.map(e => ({...e, type: 'forward'})),
                     ...backwardEdges.map(e => ({...e, type: 'backward'}))];

    let created = 0;
    allEdges.forEach((edge) => {
      const fromMesh = this.buildingByKey.get(edge.from);
      const toMesh = this.buildingByKey.get(edge.to);

      if (fromMesh && toMesh && this.edgeGroup) {
        const color = edge.type === 'forward' ? this.colors.edgeForward : this.colors.edgeBackward;
        this.createArcEdge(
          fromMesh.position,
          toMesh.position,
          radius,
          arcHeight,
          this.edgeGroup,
          color
        );
        created++;
      }
    });

    this.scene.add(this.edgeGroup);
    console.log(`Rendered ${created} curved arcs`);
  }

  /**
   * Create a curved arc edge between two buildings
   *
   * # 4-Word Name: create_curved_arc_edge
   *
   * Creates a quadratic Bezier curve that arcs toward the center of the circle.
   *
   * @param start - Starting building position
   * @param end - Ending building position
   * @param radius - Circle radius
   * @param arcHeight - How far the arc curves toward center
   * @param group - Group to add arc to
   * @param color - Edge color (optional, auto-detects from colors)
   */
  private createArcEdge(
    start: THREE.Vector3,
    end: THREE.Vector3,
    radius: number,
    arcHeight: number,
    group: THREE.Group,
    color?: number
  ): void {
    // Calculate angles of start and end points on circle
    const startAngle = Math.atan2(start.z, start.x);
    const endAngle = Math.atan2(end.z, end.x);

    // Create curved arc through center
    // Control point is closer to center of circle
    const midAngle = (startAngle + endAngle) / 2;

    // Calculate control point position (toward center)
    const controlDistance = radius - arcHeight;
    const controlX = Math.cos(midAngle) * controlDistance;
    const controlZ = Math.sin(midAngle) * controlDistance;

    // Create quadratic bezier curve - raise for visibility
    const startPoint = new THREE.Vector3(start.x, 8, start.z);
    const controlPoint = new THREE.Vector3(controlX, 18, controlZ);
    const endPoint = new THREE.Vector3(end.x, 8, end.z);

    const curve = new THREE.QuadraticBezierCurve3(startPoint, controlPoint, endPoint);

    // THICK visible tubes - make them very visible
    const tubeRadius = 0.8; // Much thicker for visibility
    const tubularSegments = 32;
    const radialSegments = 8;
    const geometry = new THREE.TubeGeometry(curve, tubularSegments, tubeRadius, radialSegments, false);

    // Bright neon material - no transparency for maximum visibility
    const material = new THREE.MeshBasicMaterial({
      color: color ?? this.colors.edgeDefault,
      transparent: false,
      opacity: 1.0,
    });

    const tube = new THREE.Mesh(geometry, material);
    group.add(tube);
  }

  /**
   * Clear all dependency edges from scene
   *
   * # 4-Word Name: clear_dependency_edges_from_scene
   */
  clearDependencyEdges(): void {
    if (this.edgeGroup) {
      this.scene.remove(this.edgeGroup);
      this.edgeGroup.traverse((obj) => {
        if (obj instanceof THREE.Mesh) {
          obj.geometry.dispose();
          const mat = obj.material as THREE.Material | THREE.Material[];
          if (Array.isArray(mat)) {
            mat.forEach((m) => m.dispose());
          } else {
            mat.dispose();
          }
        }
      });
      this.edgeGroup = null;
    }
  }

  /**
   * Get scene statistics
   *
   * # 4-Word Name: get_scene_statistics_summary
   */
  getStats(): { buildings: number } {
    return {
      buildings: this.buildings.length,
    };
  }
}
