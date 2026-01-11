# Parseltongue 3D CodeCity - Proof of Concept

**Status**: RED → GREEN phase complete. Ready for testing.

**Created**: 2025-01-11 11:15 America/Los_Angeles

---

## What This Is

A minimal proof-of-concept 3D visualization that:
1. Connects to the Parseltongue HTTP API (pt08)
2. Fetches code entities with lines of code
3. Renders them as buildings in a 3D city using Three.js
4. Validates performance with real data

**This is NOT the full web-ui** - it's a minimal POC to validate the approach before building the complete application.

---

## Prerequisites

1. **Rust toolchain** - to build the Parseltongue server
2. **Node.js 18+** - to run the frontend
3. **A codebase** - any codebase analyzed by Parseltongue

---

## Quick Start

### Step 1: Build and Start Parseltongue Server

```bash
# From repository root
cargo build --release

# Analyze a codebase (e.g., the Parseltongue codebase itself)
./target/release/parseltongue pt01-folder-to-cozodb .

# Start the HTTP server (now with CORS and LOC support!)
./target/release/parseltongue pt08 --db "rocksdb:parseltongue*/analysis.db" --port 7777
```

You should see:
```
Parseltongue HTTP Server
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

HTTP Server running at: http://localhost:7777
CORS: Enabled (allows browser-based applications)
```

### Step 2: Start the Web UI

```bash
cd web-ui-poc
npm install
npm run dev
```

Open http://localhost:3000

---

## What You'll See

| Element | Description |
|---------|-------------|
| **Buildings** | Each code entity rendered as a 3D box |
| **Height** | Proportional to lines of code (clamped 1-20 units) |
| **Width/Depth** | Based on entity type (classes wider, functions narrower) |
| **Color** | Based on programming language (Rust = orange, Python = blue, etc.) |
| **Status Bar** | Server status, entity count, FPS, building count |

---

## TDD Approach

This POC follows the STUB → RED → GREEN → REFACTOR cycle:

1. **STUB**: Wrote failing tests in `src/api/client.test.ts`
2. **RED**: Tests fail (client not implemented)
3. **GREEN**: Minimal implementation in `src/api/client.ts`
4. **REFACTOR**: Will improve after tests pass

To run tests:
```bash
cd web-ui-poc
npm install
npm test
```

---

## File Structure

```
web-ui-poc/
├── index.html              # HTML entry point
├── package.json            # Dependencies
├── tsconfig.json           # TypeScript config (strict mode)
├── vite.config.ts          # Build config with proxy fallback
├── vitest.config.ts        # Test config
└── src/
    ├── main.ts             # Application entry point
    ├── api/
    │   ├── client.ts       # API client implementation
    │   └── client.test.ts  # API client tests
    ├── scene/
    │   └── scene.ts        # Three.js scene management
    └── types/
        └── api.ts          # TypeScript types matching real API
```

---

## API Changes Required

The POC requires these minimal changes to pt08 (already implemented):

1. **`lines_of_code` field** added to `/code-entities-list-all` response
2. **CORS support** added to pt08 server

See `docs/web-ui/API_CHANGES_2025-01-11.md` for details.

---

## Performance Validation

The POC includes:

- **FPS counter** - monitor frame rate in real-time
- **Entity count display** - see how many entities are loaded
- **Building count** - verify all entities are rendered

**Goals**:
- < 3s initial load time
- 60 FPS with 100-500 buildings
- < 500ms interaction response

---

## Next Steps (If POC Succeeds)

If performance is acceptable:
1. Add OrbitControls for camera navigation
2. Implement entity selection on click
3. Add dependency edge rendering
4. Implement district/grouping by package
5. Add filter panel (by language, type)
6. Consider React Three Fiber for component architecture

---

## Known Limitations (Intentional for POC)

| Limitation | Reason |
|------------|--------|
| No camera controls | Testing basic rendering first |
| No selection/interaction | Performance validation first |
| Simple grid layout | Will improve to district-based layout |
| No edges | Buildings are the priority |
| No filters | Rendering all entities first |

---

## Troubleshooting

**CORS errors**: Make sure you built the latest pt08 with CORS support

**No entities showing**: Check browser console for errors, verify pt08 is running

**Build errors**: Run `npm install` to ensure dependencies are installed

**Three.js errors**: Check that browser supports WebGL
