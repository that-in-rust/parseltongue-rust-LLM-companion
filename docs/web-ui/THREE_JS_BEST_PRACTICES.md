# Three.js & CodeCity Visualization Best Practices

**Research Document**: Industry best practices for building a 3D CodeCity visualization using Three.js and TypeScript.

**Created**: 2025-01-11 09:30 America/Los_Angeles

---

## Executive Summary

This document compiles industry best practices for building a large-scale 3D code visualization system. The recommendations are based on:

- Original CodeCity research papers
- Three.js community best practices
- Performance optimization patterns
- Testing strategies for WebGL applications

**Key Recommendation**: Use **InstancedMesh** + **React Three Fiber** for scalable, maintainable code.

---

## Part 1: Three.js Performance for Large-Scale Data

### 1.1 InstancedMesh (Critical)

**Why**: Rendering thousands of individual meshes kills performance. Each mesh = one draw call.

**Solution**: `THREE.InstancedMesh` renders thousands of similar objects with a single draw call.

```typescript
// Correct: InstancedMesh for buildings
const geometry = new THREE.BoxGeometry(1, 1, 1);
const material = new THREE.MeshStandardMaterial();
const instancedMesh = new THREE.InstancedMesh(geometry, material, entityCount);

const matrix = new THREE.Matrix4();
const color = new THREE.Color();

for (let i = 0; i < entityCount; i++) {
  matrix.setPosition(x[i], y[i], z[i]);
  matrix.scale(w[i], h[i], d[i]);
  instancedMesh.setMatrixAt(i, matrix);
  instancedMesh.setColorAt(i, color[i]);
}

instancedMesh.instanceMatrix.needsUpdate = true;
instancedMesh.instanceColor.needsUpdate = true;
```

**Performance**: Can scale to **3 million+ instances** with proper optimization.

**Sources**:
- [Three.js InstancedMesh Performance](https://discourse.threejs.org/t/performance-optimizing-3m-instanced-grass-in-three-js/81286)
- [Instanced Rendering in Three.js](https://waelyasmina.net/articles/instanced-rendering-in-three-js/)

### 1.2 Level of Detail (LOD)

**Why**: High-detail meshes waste GPU resources when far from camera.

**Solution**: Use `THREE.LOD` to switch detail levels based on distance.

```typescript
const lod = new THREE.LOD();
lod.addLevel(highDetailMesh, 0);      // Close: full detail
lod.addLevel(mediumDetailMesh, 100);  // Medium: reduced detail
lod.addLevel(lowDetailMesh, 300);     // Far: minimal detail
scene.add(lod);
```

**Performance gain**: Up to **300% improvement** in large scenes.

**Sources**:
- [LOD Systems in Three.js](https://discourse.threejs.org/t/optimizing-materials-and-textures-for-lod-systems-in-three-js/67107)
- [Dynamic LOD Techniques](https://www.linkedin.com/pulse/dynamic-lop-techniques-real-time-performance-threejs)

### 1.3 Memory Management (Critical)

**Rule**: WebGL resources are **NOT** garbage collected. You must dispose them manually.

```typescript
function disposeMesh(mesh: THREE.Mesh): void {
  if (mesh.geometry) mesh.geometry.dispose();

  if (mesh.material) {
    if (Array.isArray(mesh.material)) {
      mesh.material.forEach(mat => mat.dispose());
    } else {
      mesh.material.dispose();
    }
  }
}

function disposeScene(scene: THREE.Scene): void {
  scene.traverse((object) => {
    if (object.isMesh) {
      if (object.geometry) object.geometry.dispose();
      if (object.material) object.material.dispose();
    }
  });
  scene.clear();
}
```

**Common Pitfall**: Forgetting to dispose textures. Always check `material.map` and `material.normalMap`.

---

## Part 2: CodeCity Visualization Metaphor

### 2.1 Entity to Building Mapping

| Code Entity | Building Representation |
|-------------|------------------------|
| Class/Struct | Tall, wide building |
| Function/Method | Medium building |
| Variable/Field | Small cube |
| Interface/Trait | Building with distinct color |
| Module/Package | District/platform |

**Physical Properties**:

| Property | Metric | Rationale |
|----------|--------|-----------|
| Height | Lines of code or complexity | Visible metric |
| Width/Depth | Number of members | API surface area |
| Color | Entity type or language | Categorical distinction |
| Position | Package hierarchy | Semantic grouping |

### 2.2 Layout Algorithms

**Containment-Based Layout** (Standard CodeCity approach):

```
City (codebase)
├── District A (package com.example.core)
│   ├── Platform (foundation)
│   └── Buildings (classes within package)
├── District B (package com.example.utils)
│   ├── Platform
│   └── Buildings
└── ...
```

**Algorithm Steps**:

1. **Cluster by package/module**: Group entities by their containing module
2. **Rectangle packing**: Arrange districts using tree map or force-directed layout
3. **Position buildings**: Grid or spiral layout within each district

**Sources**:
- [CODECITY Original Paper](https://wettel.github.io/download/Wettel08b-wasdett.pdf)
- [CodeCity 3D Visualization](https://www.inf.usi.ch/lanza/Downloads/Wett2008a.pdf)

### 2.3 Color Schemes

**Qualitative Colors** (for categories):

| Entity Type | Color | Hex |
|-------------|-------|-----|
| Class/Struct | Blue | `#3b82f6` |
| Interface/Trait | Purple | `#8b5cf6` |
| Function/Method | Green | `#10b981` |
| Variable/Field | Gray | `#6b7280` |
| Test Code | Yellow | `#f59e0b` |

**Sequential Colors** (for metrics):

- Lines of code: Light to dark blue
- Complexity: Green (low) to red (high)

**Accessibility**: Use colorblind-safe palettes. Always provide legends/labels.

---

## Part 3: TypeScript + Three.js Patterns

### 3.1 React Three Fiber (Recommended)

**Why R3F**:
- Declarative component model
- React ecosystem integration
- Built-in state management
- Strong TypeScript support

```typescript
import { Canvas } from '@react-three/fiber'
import { OrbitControls } from '@react-three/drei'

function App() {
  return (
    <Canvas camera={{ position: [0, 50, 100] }}>
      <OrbitControls enableDamping />
      <CityScene entities={entities} />
    </Canvas>
  )
}

function Building({ position, size, color }: BuildingProps) {
  return (
    <mesh position={position} castShadow>
      <boxGeometry args={size} />
      <meshStandardMaterial color={color} />
    </mesh>
  )
}
```

**Alternative**: Class-based component system (vanilla Three.js).

### 3.2 Type-Safe Patterns

```typescript
// Extend Three.js types for metadata
declare module 'three' {
  interface Object3D {
    userData: {
      entityKey?: string;
      entityType?: EntityType;
      linesOfCode?: number;
    }
  }
}

// Type guards
function isCodeBuilding(obj: THREE.Object3D): obj is THREE.Mesh & {
  userData: { entityKey: string; entityType: EntityType }
} {
  return 'entityKey' in obj.userData;
}
```

---

## Part 4: Graph Visualization

### 4.1 Edge Rendering Techniques

| Technique | Use Case | Performance |
|-----------|----------|-------------|
| `THREE.Line` | Many edges, low importance | Fastest |
| `THREE.TubeGeometry` | Important edges | Medium |
| Curved Bezier | Aesthetic emphasis | Slower |

```typescript
// Simple line (fastest)
const line = new THREE.Line(
  new THREE.BufferGeometry().setFromPoints([from, to]),
  new THREE.LineBasicMaterial({ color: 0x888888 })
);

// Tube geometry (visible 3D edge)
const tube = new THREE.Mesh(
  new THREE.TubeGeometry(
    new THREE.LineCurve3(from, to),
    1, 0.1, 8, false
  ),
  new THREE.MeshBasicMaterial({ color: 0x888888 })
);

// Curved path (aesthetic)
const curve = new THREE.CubicBezierCurve3(from, cp1, cp2, to);
const curvedLine = new THREE.Line(
  new THREE.BufferGeometry().setFromPoints(curve.getPoints(50)),
  material
);
```

### 4.2 Edge Visibility in Dense Graphs

**Strategies**:

1. **Edge bundling**: Route similar edges through common paths
2. **Opacity filtering**: Fade less important edges
3. **Progressive disclosure**: Show edges only for selected nodes
4. **Type filtering**: Toggle by edge type (inherits, calls, uses)

```typescript
// Fade edges by importance
const edgeMaterial = new THREE.LineBasicMaterial({
  color: 0x888888,
  transparent: true,
  opacity: edgeImportance // 0.0 to 1.0
});
```

---

## Part 5: Camera Controls

### 5.1 OrbitControls Configuration

```typescript
const controls = new OrbitControls(camera, renderer.domElement);

// Smooth, weighty movement
controls.enableDamping = true;
controls.dampingFactor = 0.04;

// Adjust for large scenes
controls.rotateSpeed = 0.5;
controls.zoomSpeed = 1.0;
controls.panSpeed = 0.5;

// Prevent getting lost
controls.minDistance = 10;
controls.maxDistance = 1000;

// Required when damping enabled
function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}
```

**Key Points**:
- Always `controls.update()` when damping is enabled
- Reduce speeds for large-scale scenes
- Set reasonable distance limits

---

## Part 6: Testing Strategies

### 6.1 Unit Testing Three.js

**Principle**: Separate **calculation** from **rendering**.

```typescript
// Testable: Pure calculation
function calculateBuildingLayout(entity: CodeEntity): LayoutSpec {
  return {
    position: [x, y, z],
    dimensions: [width, height, depth],
    color: assignColor(entity)
  };
}

// Test:
test('calculates building height from LOC', () => {
  const result = calculateBuildingLayout({
    linesOfCode: 100
  });
  expect(result.dimensions[1]).toBe(100); // height
});
```

### 6.2 Visual Regression Testing

```typescript
// Using Playwright
test('scene renders correctly', async ({ page }) => {
  await page.goto('/visualization');
  await page.waitForSelector('#canvas');

  const screenshot = await page.screenshot();
  expect(screenshot).toMatchSnapshot('baseline.png');
});
```

**Best practices**:
- Test at component level, not full page
- Use consistent viewport sizes
- Store baselines in version control

### 6.3 Performance Benchmarking

```typescript
test('renders 10k buildings at 30fps', () => {
  const buildings = generateBuildings(10000);
  renderer.initialize(buildings);

  const fps = measureFPS(() => {
    renderer.render(scene, camera);
  }, 1000);

  expect(fps).toBeGreaterThanOrEqual(30);
});
```

**Track**:
- FPS (target: 60)
- Frame time (target: <16ms)
- Memory usage (watch for leaks)
- Draw calls

---

## Part 7: Technology Stack Recommendations

### 7.1 Framework Comparison

| Category | Choice | Rationale |
|----------|--------|-----------|
| 3D Library | Three.js | Standard, mature ecosystem |
| React Integration | React Three Fiber | Declarative, strong TS support |
| Build Tool | Vite | Fast HMR, native ESM |
| State | Zustand | Simple, type-safe |
| Testing | Vitest + Playwright | Fast unit + reliable E2E |
| Controls | OrbitControls (drei) | Proven, feature-complete |

### 7.2 Recommended Dependencies

```json
{
  "dependencies": {
    "@react-three/fiber": "^8.x",
    "@react-three/drei": "^9.x",
    "three": "^0.160.x",
    "zustand": "^4.x"
  },
  "devDependencies": {
    "vite": "^5.x",
    "vitest": "^1.x",
    "playwright": "^1.x",
    "typescript": "^5.x"
  }
}
```

---

## Part 8: Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| Initial load time | <3s | User attention span |
| Interaction response | <500ms | Feels responsive |
| Frame rate | 60 FPS | Smooth animation |
| Max visible buildings | 10,000 | InstancedMesh limit |
| Memory footprint | <200MB | Browser constraints |

---

## Sources

### Three.js Performance
- [Performance: Optimizing 3M Instanced Grass](https://discourse.threejs.org/t/performance-optimizing-3m-instanced-grass-in-three-js/81286)
- [InstancedMesh Performance Optimizations](https://vrmeup.com/devlog/devlog_10_threejs_instancedmesh_performance_optimizations.html)
- [Memory Management in Three.js](https://discourse.threejs.org/t/when-to-dispose-how-to-completely-clean-up-a-three-js-scene/1549)

### CodeCity Research
- [CODECITY Original Paper](https://wettel.github.io/download/Wettel08b-wasdett.pdf)
- [CodeCity 3D Visualization](https://www.inf.usi.ch/lanza/Downloads/Wett2008a.pdf)
- [Official CodeCity Website](https://wettel.github.io/codecity.html)

### TypeScript & React Three Fiber
- [React Three Fiber Documentation](https://r3f.docs.pmnd.rs/)
- [Three.js TypeScript Boilerplate](https://discourse.threejs.org/t/three-js-typescript-boilerplate/12502)

### Testing
- [Vitest Visual Regression](https://vitest.dev/guide/browser/visual-regression-testing)
- [Three.js Unit Testing Discussion](https://discourse.threejs.org/t/how-to-unit-test-three-js/57736)
