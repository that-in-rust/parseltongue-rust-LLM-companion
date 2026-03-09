# Multi-Repository Port Management: Research & Solution

**Date**: 2025-01-18
**Status**: Research Complete, Implementation Proposed
**Context**: Parseltongue HTTP Server - Multi-codebase workflow support

---

## Executive Summary

Developers working across multiple codebases need to run multiple Parseltongue instances simultaneously. The current implementation has **two critical bugs** that prevent this:

1. **`--port` flag bypasses availability check** - binds directly, fails if port occupied
2. **`find_available_port_number` has a race condition** - checks port, releases it, then server tries to bind (too late!)

**Proposed Solution**: Implement "smart port selection with retry" that works whether or not `--port` is specified.

---

## The Problem

### User Scenario: Working with 5 Codebases

```bash
# Developer runs on project 1:
parseltongue pt08 --db "rocksdb:project1/analysis.db"
# → Runs on port 7777

# Developer runs on project 2:
parseltongue pt08 --db "rocksdb:project2/analysis.db"
# → ERROR: Address already in use (os error 48)

# Developer tries manual port management:
parseltongue pt08 --db "rocksdb:project2/analysis.db" --port 7778
# → What if 7778 is taken? Manually track... tedious!
```

### Current Behavior Analysis

| Command | What Happens | Result |
|---------|--------------|--------|
| `parseltongue pt08` | Calls `find_available_port_number(7777)` | **Race condition** - may still fail |
| `parseltongue pt08 --port 7777` | Binds directly to 7777 | **No availability check** - fails if occupied |

### Bug #1: Race Condition in `find_available_port_number`

**Location**: `crates/pt08-http-code-query-server/src/command_line_argument_parser.rs:120-133`

```rust
pub fn find_available_port_number(starting_port: u16) -> Result<u16> {
    use std::net::TcpListener;

    for port in starting_port..starting_port + 100 {
        if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
            return Ok(port);  // ❌ BUG: Listener dropped here, port released!
        }
    }
    anyhow::bail!("No available ports found...")
}
```

**The Race:**
```
1. find_available_port_number(7777) called
2. TcpListener::bind("127.0.0.1:7777") succeeds ✅
3. Function returns Ok(7777)
4. Listener DROP → port 7777 RELEASED
5. Another process grabs port 7777 (or it was never truly free)
6. tokio::net::TcpListener::bind("0.0.0.0:7777") → FAILS ❌
```

**Empirical Evidence:**
```bash
$ parseltongue pt08 --db "rocksdb:db/analysis.db"
Running Tool 8: HTTP Code Query Server
HTTP Server running at: http://localhost:7777
Error: Address already in use (os error 48)
```

### Bug #2: `--port` Flag Bypasses Availability Check

**Location**: `crates/pt08-http-code-query-server/src/http_server_startup_runner.rs:192-193`

```rust
let port = config.http_port_override_option
    .unwrap_or_else(|| find_available_port_number(7777).unwrap_or(7777));
//                                    ^^^^^^^^^^^ Only called if NO --port
```

When user specifies `--port 7777`:
- No availability check performed
- Direct bind attempt
- Fails if port occupied

---

## Research: Industry Best Practices

### How Popular Tools Solve This

| Tool | Strategy |
|------|----------|
| **Next.js** | Try default (3000), increment on failure, log actual port |
| **Vite** | Try default (5173), increment on failure, log actual port |
| **VS Code Live Server** | Try 5500, increment to 5501, 5502... |
| **Create React App** | Try 3000, increment on failure |

**Common Pattern**: `try-bind → fail → try-next → repeat`

### Safe Port Ranges (Cross-Platform)

| Range | Availability | Use Case |
|-------|--------------|----------|
| `3000-3999` | Universally safe | Development servers (Next.js, CRA) |
| `4000-4999` | Very safe | Less commonly used |
| `5000-5999` | Mostly safe | Vite uses 5173 |
| `7777-7877` | Parseltongue traditional range | Current default |

**Why 7777+ is fine**: Above privileged port range (1024), unlikely to conflict with system services.

---

## Proposed Solution

### Architecture: Smart Port Selection with Retry

**Core Principle**: Whether `--port` is specified or not, **always check availability and retry on failure**.

```
┌─────────────────────────────────────────────────────────────┐
│  Smart Port Selection Flow                                  │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  User: parseltongue pt08 --port 7777                        │
│           or                                                 │
│  User: parseltongue pt08                                    │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  preferred = user_specified OR 7777                  │   │
│  │  for port in preferred..preferred+100:               │   │
│  │      print "Trying {port}..."                        │   │
│  │      if bind_succeeds(port):                         │   │
│  │          return port  // Found it!                   │   │
│  │      else:                                           │   │
│  │          continue  // Try next port                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Output:                                                    │
│    "Port 7777 is in use. Trying 7778..."                   │
│    "Port 7778 is in use. Trying 7779..."                   │
│    "✓ Parseltongue running on http://localhost:7779"       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **`--port` becomes a preference, not requirement** | User intent respected, but system adapts |
| **Retry loop with logging** | User sees what's happening |
| **100 port range** | Reasonable上限 for concurrent instances |
| **Bind check AND server bind use same logic** | No race condition |
| **Clear success message with actual port** | Web UI can parse this for auto-connect |

### Implementation Sketch

```rust
/// Find and bind to an available port, starting from preference
///
/// # 4-Word Name: find_and_bind_to_available_port
pub async fn find_and_bind_to_available_port(
    preferred_port: Option<u16>,
    max_attempts: u16,
) -> Result<TcpListener> {
    let start = preferred_port.unwrap_or(7777);

    for port in start..start + max_attempts {
        eprint!("Trying {}...", port);

        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(listener) => {
                eprintln!(" ✓");
                return Ok(listener);
            }
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                eprintln!(" in use, trying next...");
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    anyhow::bail!("No available ports in range {}-{}", start, start + max_attempts)
}

// Usage in http_server_startup_runner.rs:
pub async fn start_http_server_blocking_loop(config: HttpServerStartupConfig) -> Result<()> {
    let preferred = config.http_port_override_option;
    let listener = find_and_bind_to_available_port(preferred, 100).await?;
    let port = listener.local_addr()?.port();

    // ... rest of setup ...

    println!("✓ Parseltongue running on http://localhost:{}", port);

    axum::serve(listener, router).await?;
}
```

---

## User Experience Improvements

### Before: Broken Multi-Repo Workflow

```bash
# Terminal 1:
$ parseltongue pt08 --db "rocksdb:auth-service/db"
✓ Running on http://localhost:7777

# Terminal 2:
$ parseltongue pt08 --db "rocksdb:payment-service/db"
Error: Address already in use (os error 48)

# Developer has to:
$ parseltongue pt08 --db "rocksdb:payment-service/db" --port 7778
# But what if 7778 is taken? Try 7779? This doesn't scale.
```

### After: Seamless Multi-Repo Support

```bash
# Terminal 1:
$ parseltongue pt08 --db "rocksdb:auth-service/db"
Trying 7777... ✓
✓ Parseltongue running on http://localhost:7777

# Terminal 2:
$ parseltongue pt08 --db "rocksdb:payment-service/db"
Trying 7777... in use, trying next...
Trying 7778... ✓
✓ Parseltongue running on http://localhost:7778

# Terminal 3:
$ parseltongue pt08 --db "rocksdb:user-service/db"
Trying 7777... in use, trying next...
Trying 7778... in use, trying next...
Trying 7779... ✓
✓ Parseltongue running on http://localhost:7779

# With explicit preference (still works):
$ parseltongue pt08 --db "rocksdb:admin/db" --port 8000
Trying 8000... ✓
✓ Parseltongue running on http://localhost:8000
```

---

## Web UI Integration

### Auto-Discovery Pattern

The web UI can discover the running Parseltongue instance by:

1. **Parsing stdout** for the success message pattern
2. **Trying common ports sequentially** (7777, 7778, 7779...)
3. **Reading a PID file** with port mapping

```javascript
// Web UI auto-connect logic
async function findParseltongueServer() {
  for (let port = 7777; port < 7877; port++) {
    try {
      const response = await fetch(`http://localhost:${port}/server-health-check-status`);
      if (response.ok) {
        return port;  // Found it!
      }
    } catch {
      continue;
    }
  }
  throw new Error('No Parseltongue server found');
}
```

---

## LLM Integration Benefits

### Before: Complex Instructions

```
"Run Parseltongue on port 7777 with the auth database,
 then run another instance on port 7778 with the payment database,
 make sure 7778 isn't already in use..."
```

### After: Simple Intent

```
"Run Parseltongue on the auth service and payment service"
```

The LLM just invokes:
```bash
parseltongue pt08 --db "rocksdb:auth/db"
parseltongue pt08 --db "rocksdb:payment/db"
```

Both work. No port management needed.

---

## Implementation Checklist

- [ ] Fix `find_available_port_number` race condition
- [ ] Implement `find_and_bind_to_available_port` with retry loop
- [ ] Update `start_http_server_blocking_loop` to use new function
- [ ] Make `--port` flag a preference, not hard requirement
- [ ] Add clear logging: "Trying X... in use... trying Y... ✓"
- [ ] Update CLI output to show actual bound port prominently
- [ ] Consider PID file or port registry for multi-instance tracking
- [ ] Update README with multi-repository workflow examples
- [ ] Test: 5 simultaneous instances
- [ ] Test: Explicit `--port` with retry behavior

---

## Testing Strategy

```bash
# Test 1: Default port selection
parseltongue pt08 --db "db1" &
sleep 1
parseltongue pt08 --db "db2" &
sleep 1
parseltongue pt08 --db "db3" &
# Expect: Ports 7777, 7778, 7779

# Test 2: Explicit port with retry
parseltongue pt08 --db "db4" --port 7777 &
# Expect: "Trying 7777... in use... trying 7778... ✓"

# Test 3: Port exhaustion
for i in {1..100}; do
  parseltongue pt08 --db "db$i" &
done
# Expect: First 100 succeed, 101st fails with clear error
```

---

## Future Enhancements

1. **Port Registry Service**: Track which database is on which port
2. **Auto-shutdown on idle**: Cleanup unused instances
3. **Named instances**: `--name auth-service` for easier identification
4. **Port persistence**: Remember port preferences per database
5. **Zero-conf discovery**: mDNS/bonjour for local network discovery

---

## Conclusion

The current implementation has two bugs preventing multi-repository workflows:

1. **Race condition** in `find_available_port_number`
2. **No retry** when `--port` is specified

The solution is simple: **unify the code paths** so that both scenarios use the same "try-bind-retry" logic. This makes:

- **Multi-repo workflows** seamless
- **LLM integration** simpler (no port management)
- **User experience** better (clear feedback, automatic adaptation)
- **`--port` flag** a preference, not a source of errors

**Implementation effort**: ~2-3 hours
**Impact**: High (enables core multi-codebase workflow)

---

**Generated**: 2025-01-18
**Agent**: Claude Opus 4.5
**Related Documents**:
- `/docs/web-ui/THREE_JS_INNOVATION_RESEARCH.md`
- `/docs/web-ui/DESIGN_OPTIONS_FOR_PARSELTONGUE_VISUALIZATION.md`
