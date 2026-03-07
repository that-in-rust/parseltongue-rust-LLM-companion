# Idiomatic Rust Patterns: Library Crates
> Source: rust-analyzer/lib/ (la-arena, line-index, lsp-server, smol_str, text-size, ungrammar)

## Pattern 1: Type-Safe Arena with PhantomData Type Parameters
**File:** lib/la-arena/src/lib.rs
**Category:** Arena Design, Type-Safe Indexing
**Code Example:**
```rust
/// The raw index of a value in an arena.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawIdx(u32);

/// The index of a value allocated in an arena that holds `T`s.
pub struct Idx<T> {
    raw: RawIdx,
    _ty: PhantomData<fn() -> T>,
}

/// Yet another index-based arena.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Arena<T> {
    data: Vec<T>,
}

impl<T> Arena<T> {
    pub fn alloc(&mut self, value: T) -> Idx<T> {
        let idx = self.next_idx();
        self.data.push(value);
        idx
    }

    fn next_idx(&self) -> Idx<T> {
        Idx::from_raw(RawIdx(self.data.len() as u32))
    }
}

impl<T> Index<Idx<T>> for Arena<T> {
    type Output = T;
    fn index(&self, idx: Idx<T>) -> &T {
        let idx = idx.into_raw().0 as usize;
        &self.data[idx]
    }
}
```
**Why This Matters for Contributors:**
This pattern uses PhantomData with `fn() -> T` (not just `T`) to ensure Idx is covariant over T while remaining Copy. The arena stores values in a Vec but returns type-safe indices that can't be mixed between different arenas. Using u32 indices instead of pointers saves memory (8 bytes vs 16 on 64-bit) and makes indices stable across reallocations. The `fn() -> T` trick is crucial: it makes the phantom data not own T while maintaining proper variance.

---

### Expert Rust Commentary: Pattern 1

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Exemplary)

**Pattern Classification:** L2 Standard Library Idioms + Type-Level Programming
- **Primary Category:** Arena allocation with type-safe indexing
- **Related Patterns:** Newtype (A.5), PhantomData for variance (A.109), Index trait implementation (9.5)

**Rust-Specific Insight:**
The `PhantomData<fn() -> T>` pattern here is a masterclass in variance control. Using `fn() -> T` instead of just `T` accomplishes three critical goals:
1. **Covariance over T** - Enables `Idx<&'static T>` to be compatible with `Idx<&'a T>`
2. **Zero-size** - PhantomData adds no runtime cost
3. **No ownership** - `fn() -> T` doesn't own T, so Idx remains `Copy` even if T isn't

The u32 index choice is deliberate: on 64-bit systems, this saves 4 bytes per index compared to usize, and 8 bytes compared to raw pointers. For a codebase with millions of AST nodes (rust-analyzer's scale), this translates to significant memory savings. The stability guarantee (indices remain valid across Vec reallocations) is crucial for incremental compilation scenarios.

**Contribution Tip:**
When extending this pattern, preserve the PhantomData invariants. If you need a mutable arena API, consider using `Cell<Vec<T>>` or interior mutability patterns. For concurrent access, explore `RwLock<Arena<T>>` with `Idx<T>` shared across threads (Idx is Send+Sync if T is).

**Common Pitfalls:**
- **Anti-pattern:** Using `PhantomData<T>` directly makes Idx non-Copy if T isn't Copy
- **Anti-pattern:** Using `*const T` or `*mut T` in PhantomData breaks variance (invariant over T)
- **Memory leak risk:** Arena never drops T until the entire Arena is dropped - not suitable for long-lived incremental scenarios without periodic cleanup

**Related Patterns in Ecosystem:**
- **typed-arena**: Uses bump allocation for faster allocation, but indices aren't stable
- **generational-arena**: Adds generation counter to detect use-after-free
- **slotmap**: Similar but with packed storage and generation checks
- **rustc's Arena**: Uses typed-arena internally but with different indexing strategies

**Further Reading:**
- Rustonomicon: PhantomData and variance (https://doc.rust-lang.org/nomicon/phantom-data.html)
- Rust API Guidelines: Newtype pattern (C-NEWTYPE)
- "Understanding Rust's PhantomData" - variance implications for phantom type parameters

---

## Pattern 2: Sparse Arena Map with Vec<Option<V>>
**File:** lib/la-arena/src/map.rs
**Category:** Sparse Data Structures, Arena Companion
**Code Example:**
```rust
/// A map from arena indexes to some other type.
/// Space requirement is O(highest index).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArenaMap<IDX, V> {
    v: Vec<Option<V>>,
    _ty: PhantomData<IDX>,
}

impl<T, V> ArenaMap<Idx<T>, V> {
    pub fn insert(&mut self, idx: Idx<T>, t: V) -> Option<V> {
        let idx = Self::to_idx(idx);
        self.v.resize_with((idx + 1).max(self.v.len()), || None);
        self.v[idx].replace(t)
    }

    pub fn get(&self, idx: Idx<T>) -> Option<&V> {
        self.v.get(Self::to_idx(idx)).and_then(|it| it.as_ref())
    }

    pub fn shrink_to_fit(&mut self) {
        let min_len = self.v.iter().rposition(|slot| slot.is_some()).map_or(0, |i| i + 1);
        self.v.truncate(min_len);
        self.v.shrink_to_fit();
    }
}
```
**Why This Matters for Contributors:**
ArenaMap provides O(1) access to auxiliary data indexed by arena indices. Unlike HashMap, it doesn't hash and has better cache locality. The `Vec<Option<V>>` representation is sparse (empty slots are None) but grows to accommodate any index. The `shrink_to_fit` implementation finds the rightmost Some value using `rposition`, demonstrating proper sparse data cleanup. This pattern is ideal when you have arena indices and need to attach optional metadata.

---

### Expert Rust Commentary: Pattern 2

**Idiomatic Rating:** ⭐⭐⭐⭐ (4/5 - Production Quality)

**Pattern Classification:** L2 Standard Library Idioms (Collections)
- **Primary Category:** Sparse data structures optimized for arena indices
- **Related Patterns:** HashMap alternative (A.39), memory optimization (8.1-8.10), Option combinators (31.1-31.10)

**Rust-Specific Insight:**
This is a perfect example of choosing the right data structure for access patterns. ArenaMap trades space efficiency for time efficiency: O(n) space (where n is highest index) but O(1) access with perfect cache locality. The key insight is that `Vec<Option<V>>` outperforms `HashMap<Idx<T>, V>` when:
1. Indices are relatively dense (few gaps)
2. Access patterns are sequential or clustered
3. You need predictable, hash-free performance

The `shrink_to_fit` implementation using `rposition` is elegant - it finds the rightmost Some value in O(n) and truncates. This is crucial for long-running processes that build up ArenaMap data temporarily (e.g., per-query caches in rust-analyzer).

The `resize_with(|| None)` pattern ensures sparse growth: only the Vec capacity grows, not the actual Option allocations. Memory usage is `sizeof(Option<V>) * highest_index`, not `sizeof(V) * highest_index`.

**Contribution Tip:**
Consider implementing `retain` method for bulk filtering, or `iter/iter_mut` that skip None entries. For very sparse data (>90% None), profile against FxHashMap - the crossover point depends on V's size and access patterns.

**Common Pitfalls:**
- **Performance trap:** Using ArenaMap for truly sparse data (e.g., only 100 entries with max index 1_000_000) wastes 3.8MB per entry type
- **API confusion:** The API accepts `Idx<T>` but stores `V` - type parameters can be confusing (T is phantom in ArenaMap)
- **Memory leak:** Forgetting to call shrink_to_fit after removing high-index entries leaves large Vec allocated

**Related Patterns in Ecosystem:**
- **FxHashMap**: Faster hashing for integer keys, use when data is truly sparse
- **SmallVec**: Could be used for `Vec<Option<V>>` when most maps are small
- **dashmap**: Concurrent alternative when parallel access needed
- **indexmap**: Maintains insertion order, different use case

**Performance Characteristics:**
```rust
// Sparse case (10% filled, max_idx=10000):
// ArenaMap: ~80KB (10000 * sizeof(Option<V>))
// HashMap:   ~1KB (1000 entries * ~1 byte overhead)

// Dense case (90% filled, max_idx=10000):
// ArenaMap: ~80KB, O(1) access, cache-friendly
// HashMap:   ~20KB, O(1) access, pointer chasing
```

---

## Pattern 3: Entry API for Arena Maps
**File:** lib/la-arena/src/map.rs
**Category:** Ergonomic APIs, Entry Pattern
**Code Example:**
```rust
pub enum Entry<'a, IDX, V> {
    Vacant(VacantEntry<'a, IDX, V>),
    Occupied(OccupiedEntry<'a, IDX, V>),
}

impl<'a, IDX, V> Entry<'a, IDX, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Self::Vacant(ent) => ent.insert(default),
            Self::Occupied(ent) => ent.into_mut(),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Self::Vacant(ent) => ent.insert(default()),
            Self::Occupied(ent) => ent.into_mut(),
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Self::Occupied(ent) = &mut self {
            f(ent.get_mut());
        }
        self
    }
}

pub struct VacantEntry<'a, IDX, V> {
    slot: &'a mut Option<V>,
    _ty: PhantomData<IDX>,
}

pub struct OccupiedEntry<'a, IDX, V> {
    slot: &'a mut Option<V>,
    _ty: PhantomData<IDX>,
}
```
**Why This Matters for Contributors:**
This replicates HashMap's entry API for ArenaMap, enabling patterns like `map.entry(idx).or_insert(default)` to avoid double lookups. The VacantEntry and OccupiedEntry both hold `&mut Option<V>`, exploiting that ArenaMap pre-allocates slots. The `and_modify` method returns Self (not &mut Self), enabling chaining. This pattern shows how to adapt standard library APIs to custom collections.

---

### Expert Rust Commentary: Pattern 3

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Exemplary API Design)

**Pattern Classification:** L2 Standard Library Idioms (Ergonomic APIs)
- **Primary Category:** Entry API pattern from HashMap adapted to custom collections
- **Related Patterns:** HashMap entry API (A.39), builder pattern (3.1-3.10), Option ergonomics (31.1-31.10)

**Rust-Specific Insight:**
This is a textbook example of adapting std patterns to custom types. The Entry API eliminates double-lookups: instead of `if map.get(k).is_none() { map.insert(k, v) }` (two lookups), you write `map.entry(k).or_insert(v)` (one lookup).

The critical implementation detail is that both VacantEntry and OccupiedEntry hold `&mut Option<V>` - they're just different *interpretations* of the same slot. This is possible because ArenaMap pre-allocates slots with None. The lifetime `'a` ensures the borrow checker prevents use-after-free: you can't modify the ArenaMap while holding an Entry.

The `and_modify` returns `Self` (not `&mut Self`) to enable chaining:
```rust
map.entry(idx)
    .and_modify(|v| v.count += 1)
    .or_insert(Data { count: 1 });
```

**Contribution Tip:**
When implementing entry APIs for custom collections, follow this structure:
1. Entry enum with Vacant/Occupied variants
2. Both variants hold mutable references to the storage location
3. Lifetime `'a` ties entry to the collection's borrow
4. Implement or_insert, or_default, and_modify for ergonomics

Consider adding `or_insert_with_key` (takes `FnOnce(&K) -> V`) for cases where key is needed to construct value.

**Common Pitfalls:**
- **Lifetime confusion:** Entry borrows the map mutably - can't access map until Entry is dropped
- **API misuse:** Calling `entry(idx).or_insert(expensive())` evaluates expensive() unconditionally - use `or_insert_with`
- **Performance:** `and_modify` always returns `self`, so chaining creates moves - acceptable for Copy types, consider for large enums

**Related Patterns in Ecosystem:**
- **HashMap/BTreeMap**: Standard library entry APIs this pattern mimics
- **hashbrown::HashMap**: Raw entry API (unstable) for even lower-level control
- **IndexMap**: Entry API with index-based ordering
- **cache patterns**: Entry API natural for caches (compute-if-absent)

**Advanced Pattern - Raw Entry API:**
```rust
// Not shown in pattern, but could add:
pub fn raw_entry_mut(&mut self, idx: Idx<T>)
    -> RawEntryMut<'_, IDX, V> {
    // Expose &mut Option<V> directly for unsafe optimization
}
```

---

## Pattern 4: LSP Connection as Channel Pair
**File:** lib/lsp-server/src/lib.rs
**Category:** LSP Protocol, Channel-Based Architecture
**Code Example:**
```rust
/// Connection is just a pair of channels of LSP messages.
pub struct Connection {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

impl Connection {
    /// Create connection over standard in/standard out.
    pub fn stdio() -> (Connection, IoThreads) {
        let (sender, receiver, io_threads) = stdio_transport();
        (Connection { sender, receiver }, io_threads)
    }

    /// Creates a pair of connected connections.
    /// Use this for testing.
    pub fn memory() -> (Connection, Connection) {
        let (s1, r1) = crossbeam_channel::unbounded();
        let (s2, r2) = crossbeam_channel::unbounded();
        (Connection { sender: s1, receiver: r2 }, Connection { sender: s2, receiver: r1 })
    }
}
```
**Why This Matters for Contributors:**
The Connection abstraction reduces LSP transport to a channel pair, making the protocol transport-agnostic. stdio() creates real I/O threads, while memory() creates in-memory connections for testing. This separation of concerns (protocol logic vs transport) is crucial for testable LSP servers. The public sender/receiver fields enable direct channel operations while maintaining type safety through the Message enum.

---

### Expert Rust Commentary: Pattern 4

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Architectural Excellence)

**Pattern Classification:** L2 Standard Library Idioms + Concurrency Patterns
- **Primary Category:** Transport abstraction via message-passing channels
- **Related Patterns:** Channel patterns (5.5), actor pattern (5.1), dependency injection (A.3), testability patterns (11.1-11.10)

**Rust-Specific Insight:**
This pattern demonstrates the power of reducing complex I/O to simple channels. The Connection type is transport-agnostic - it's just a sender/receiver pair. The beauty is in the constructor pattern:

1. **stdio()** - Production: spawns real I/O threads reading/writing stdin/stdout
2. **memory()** - Testing: creates in-memory channel pairs with no I/O

This separation enables testing LSP servers without mocking - just use memory() connections. The public fields (sender, receiver) provide escape hatches for advanced usage while maintaining type safety through Message enum.

The crossbeam_channel::unbounded() choice for memory() is deliberate - tests shouldn't deadlock on backpressure. In contrast, stdio() uses bounded(0) (rendezvous) channels for production backpressure.

**Contribution Tip:**
When building protocol handlers, follow this pattern:
1. Define a message type (enum or struct)
2. Encapsulate transport as sender/receiver pair
3. Provide multiple constructors for different transports
4. Keep protocol logic separate from I/O logic

For production, consider:
```rust
pub fn tcp(addr: SocketAddr) -> io::Result<(Connection, IoThreads)> {
    // TCP-based LSP server (VSCode remote scenarios)
}
pub fn websocket(url: Url) -> Result<(Connection, IoThreads)> {
    // Browser-based LSP (Monaco editor)
}
```

**Common Pitfalls:**
- **Channel capacity mismatch:** Using unbounded in production can cause OOM under load
- **Shutdown handling:** Channels don't auto-close on sender drop - need explicit exit protocol
- **Message ordering:** Channels guarantee FIFO but async processing can reorder responses

**Related Patterns in Ecosystem:**
- **tower::Service**: More generic request/response abstraction
- **tonic**: gRPC using similar channel-based approach for streaming
- **actix**: Actor framework where each actor has a channel-based mailbox
- **tarpc**: RPC framework using channels for transport abstraction

**Testing Strategy:**
```rust
#[test]
fn test_lsp_completion() {
    let (server, client) = Connection::memory();

    // Send request from client
    client.sender.send(Message::Request(/* ... */)).unwrap();

    // Server receives and processes
    let msg = server.receiver.recv().unwrap();
    // ... handle request ...

    // Client receives response
    let resp = client.receiver.recv().unwrap();
    assert_eq!(/* expected response */);
}
```

---

## Pattern 5: LSP Initialization State Machine
**File:** lib/lsp-server/src/lib.rs
**Category:** Protocol State Management, LSP Handshake
**Code Example:**
```rust
impl Connection {
    pub fn initialize_start(&self) -> Result<(RequestId, serde_json::Value), ProtocolError> {
        self.initialize_start_while(|| true)
    }

    pub fn initialize_start_while<C>(
        &self,
        running: C,
    ) -> Result<(RequestId, serde_json::Value), ProtocolError>
    where
        C: Fn() -> bool,
    {
        while running() {
            let msg = match self.receiver.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(msg) => msg,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Err(ProtocolError::disconnected()),
            };

            match msg {
                Message::Request(req) if req.is_initialize() => return Ok((req.id, req.params)),
                // Respond to non-initialize requests with ServerNotInitialized
                Message::Request(req) => {
                    let resp = Response::new_err(
                        req.id.clone(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {req:?}"),
                    );
                    self.sender.send(resp.into()).unwrap();
                    continue;
                }
                Message::Notification(n) if !n.is_exit() => continue,
                msg => return Err(ProtocolError::new(format!("expected initialize request, got {msg:?}"))),
            };
        }
        Err(ProtocolError::new("Initialization aborted"))
    }

    pub fn initialize_finish(
        &self,
        initialize_id: RequestId,
        initialize_result: serde_json::Value,
    ) -> Result<(), ProtocolError> {
        let resp = Response::new_ok(initialize_id, initialize_result);
        self.sender.send(resp.into()).unwrap();
        match &self.receiver.recv() {
            Ok(Message::Notification(n)) if n.is_initialized() => Ok(()),
            Ok(msg) => Err(ProtocolError::new(format!(r#"expected initialized notification, got: {msg:?}"#))),
            Err(RecvError) => Err(ProtocolError::disconnected()),
        }
    }
}
```
**Why This Matters for Contributors:**
The LSP initialization is split into start/finish phases, enabling custom capability negotiation between them. The `_while` variants accept a running predicate for graceful shutdown via signals (e.g., Ctrl+C). Before initialization completes, non-initialize requests are rejected with ServerNotInitialized per LSP spec. This state machine ensures protocol compliance while remaining flexible for application-specific initialization logic.

---

### Expert Rust Commentary: Pattern 5

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Protocol Compliance Mastery)

**Pattern Classification:** L2 Standard Library + Protocol State Management
- **Primary Category:** LSP initialization handshake state machine
- **Related Patterns:** Error handling (2.1-2.10), async patterns (14.1-14.10), cancellation (A.25, A.52)

**Rust-Specific Insight:**
This pattern implements the LSP initialization handshake as a two-phase commit: `initialize_start` receives the initialize request, application negotiates capabilities, `initialize_finish` sends response and waits for initialized notification. The split enables custom capability logic between phases.

The `_while` variant accepting a `running` predicate is brilliant - it enables graceful shutdown via signal handlers:
```rust
let running = Arc::new(AtomicBool::new(true));
let r = running.clone();
ctrlc::set_handler(move || r.store(false, Ordering::SeqCst)).unwrap();

connection.initialize_start_while(|| running.load(Ordering::SeqCst))?;
```

The timeout loop (`recv_timeout(1s)`) prevents blocking forever, checking running() each second. The protocol compliance is meticulous:
- Rejects non-initialize requests with ServerNotInitialized (-32002)
- Ignores non-exit notifications
- Returns ProtocolError for unexpected messages

**Contribution Tip:**
When implementing protocol handshakes, follow this state machine pattern:
1. Loop until expected message arrives
2. Reject invalid messages with protocol-specific errors
3. Provide cancellation hooks via predicates or tokens
4. Use timeouts to avoid hanging on malformed input

For complex protocols, consider encoding states as types:
```rust
struct Uninitialized(Connection);
struct Initialized(Connection);

impl Uninitialized {
    fn initialize(self) -> Result<Initialized> { /* ... */ }
}
// Can't call methods requiring initialization on Uninitialized
```

**Common Pitfalls:**
- **Blocking forever:** Not using timeout or cancellation in initialization loop
- **Protocol violations:** Accepting non-initialize requests before handshake completes
- **Resource leaks:** Not handling Disconnected errors (client crashed during init)
- **Race conditions:** Processing messages before initialized notification confirmed

**Related Patterns in Ecosystem:**
- **tokio::sync::watch**: Could be used for running flag with change notifications
- **tokio::select!**: Async alternative to timeout loop
- **tower::Service::ready()**: Similar readiness check pattern
- **State machine crates (typestate)**: Compile-time state enforcement

**LSP Specification Compliance:**
```
Client                          Server
  |                               |
  |  initialize request      -->  |
  |                               | (negotiate capabilities)
  |  <-- initialize response      |
  |                               |
  |  initialized notification --> |
  |                               | (now ready)
```

**Error Handling Excellence:**
The pattern properly distinguishes:
- `ProtocolError::disconnected()` - transport failure
- `ProtocolError::new(msg)` - protocol violation
- `ServerNotInitialized` - LSP error code -32002

---

## Pattern 6: Untagged Serde Enum for LSP Messages
**File:** lib/lsp-server/src/msg.rs
**Category:** LSP Protocol, Serde Patterns
**Code Example:**
```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub id: RequestId,
    pub method: String,
    #[serde(default = "serde_json::Value::default")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct RequestId(IdRepr);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(untagged)]
enum IdRepr {
    I32(i32),
    String(String),
}
```
**Why This Matters for Contributors:**
The `#[serde(untagged)]` on Message enables parsing LSP messages without explicit type tags - serde tries each variant until one matches. RequestId supports both i32 and String IDs per JSON-RPC spec. The `#[serde(skip_serializing_if)]` ensures null params/results are omitted from serialized JSON, matching LSP wire format. The `#[serde(default)]` on params allows missing fields to parse as null. This pattern demonstrates production-grade serde configuration for protocol implementations.

---

### Expert Rust Commentary: Pattern 6

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Serde Mastery)

**Pattern Classification:** L3 External Dependencies (Serde) + Serialization Patterns
- **Primary Category:** JSON-RPC protocol serialization with serde attributes
- **Related Patterns:** Serde patterns (17.1-17.10), enum optimization (A.4), error handling (2.1-2.10)

**Rust-Specific Insight:**
This pattern showcases advanced serde configuration for protocol compliance. Each attribute serves a specific purpose:

**`#[serde(untagged)]` on Message:**
Serde tries each variant until one matches, no discriminator field. This enables parsing `{"id":1,"method":"..."}` as Request vs `{"id":1,"result":...}` as Response based on field presence. Performance cost: O(n) variant attempts, but n=3 here.

**`#[serde(default)]` and `#[serde(skip_serializing_if)]` on params:**
JSON-RPC allows omitting null params. `default` makes missing fields parse as `Value::Null`. `skip_serializing_if = "Value::is_null"` omits null when serializing. Round-trip correctness: `{} == {"params":null}`.

**`#[serde(transparent)]` on RequestId:**
Unwraps the newtype - `RequestId(IdRepr)` serializes as just the IdRepr. Combined with `#[serde(untagged)]` on IdRepr, this handles both `"id":123` and `"id":"abc"` per JSON-RPC spec.

**RequestId with untagged enum:**
JSON-RPC allows both integer and string IDs. The untagged enum tries i32 first (common case), then String. Implements Hash/Ord for use in HashMaps/BTreeMaps.

**Contribution Tip:**
When implementing protocol types with serde:
1. Use `#[serde(untagged)]` sparingly - it's O(n) and error messages are poor
2. Combine `default` and `skip_serializing_if` for optional fields to match wire format
3. Implement `transparent` for newtypes that shouldn't appear in JSON
4. Test round-trip: `assert_eq!(msg, serde_json::from_str(&serde_json::to_string(&msg)?)?)`

**Common Pitfalls:**
- **Untagged variant ordering:** Order matters! Most specific variant first. If String variant came before i32, "123" would parse as String
- **Default values:** `#[serde(default)]` uses Default::default(), not serde_json::Value::Null explicitly
- **Error messages:** Untagged enums produce confusing errors ("expected X or Y or Z")
- **Performance:** Untagged enums with many variants are slow - profile deserialization

**Related Patterns in Ecosystem:**
- **serde_json::value::RawValue**: Zero-copy deserialization for nested JSON
- **serde_repr**: For C-like enums with explicit discriminants
- **serde_with**: Additional attribute helpers for edge cases
- **typetag**: Polymorphic deserialization for trait objects

**Advanced Testing:**
```rust
#[test]
fn test_id_formats() {
    let int_id: RequestId = serde_json::from_str("42").unwrap();
    let str_id: RequestId = serde_json::from_str("\"abc\"").unwrap();
    assert_ne!(int_id, str_id);

    // Verify serialization round-trip
    assert_eq!(serde_json::to_string(&int_id).unwrap(), "42");
    assert_eq!(serde_json::to_string(&str_id).unwrap(), "\"abc\"");
}
```

---

## Pattern 7: LSP Message I/O with Content-Length Headers
**File:** lib/lsp-server/src/msg.rs
**Category:** LSP Transport, Protocol Framing
**Code Example:**
```rust
impl Message {
    pub fn read(r: &mut impl BufRead) -> io::Result<Option<Message>> {
        let text = match read_msg_text(r)? {
            None => return Ok(None),
            Some(text) => text,
        };
        let msg = serde_json::from_str(&text)
            .map_err(|e| invalid_data!("malformed LSP payload `{e:?}`: {text:?}"))?;
        Ok(Some(msg))
    }

    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        #[derive(Serialize)]
        struct JsonRpc<'a> {
            jsonrpc: &'static str,
            #[serde(flatten)]
            msg: &'a Message,
        }
        let text = serde_json::to_string(&JsonRpc { jsonrpc: "2.0", msg: self })?;
        write_msg_text(w, &text)
    }
}

fn read_msg_text(inp: &mut dyn BufRead) -> io::Result<Option<String>> {
    let mut size = None;
    let mut buf = String::new();
    loop {
        buf.clear();
        if inp.read_line(&mut buf)? == 0 {
            return Ok(None);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data!("malformed header: {:?}", buf));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().unwrap();
        let header_value = parts.next().ok_or_else(|| invalid_data!("malformed header: {:?}", buf))?;
        if header_name.eq_ignore_ascii_case("Content-Length") {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data!("no Content-Length"))?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf).map_err(invalid_data)?;
    Ok(Some(buf))
}

fn write_msg_text(out: &mut dyn Write, msg: &str) -> io::Result<()> {
    write!(out, "Content-Length: {}\r\n\r\n", msg.len())?;
    out.write_all(msg.as_bytes())?;
    out.flush()?;
    Ok(())
}
```
**Why This Matters for Contributors:**
LSP uses HTTP-style Content-Length headers to frame JSON messages. The read logic parses headers line-by-line, validates \r\n endings, case-insensitively matches "Content-Length", then reads exactly that many bytes. The write logic injects `"jsonrpc": "2.0"` via serde flatten. This pattern shows correct LSP framing, including edge cases like header validation and UTF-8 conversion errors.

---

### Expert Rust Commentary: Pattern 7

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Protocol Implementation Excellence)

**Pattern Classification:** L2 Standard Library (I/O) + Protocol Framing
- **Primary Category:** HTTP-style message framing for LSP transport
- **Related Patterns:** Error handling (2.1-2.10), I/O patterns (26.1-26.10), buffered I/O (26.2)

**Rust-Specific Insight:**
This pattern implements the LSP wire format: HTTP-style headers + JSON body. The read logic is exceptionally robust:

**Header Parsing:**
- Reads line-by-line until empty line (end of headers)
- Validates `\r\n` line endings (not just `\n`)
- Case-insensitive header matching (`eq_ignore_ascii_case`)
- Accumulates Content-Length, ignores other headers (extensibility)

**Body Reading:**
- Converts header buffer to bytes (`into_bytes()`) to reuse allocation
- `resize(size, 0)` ensures exact size, then `read_exact` fills it
- `String::from_utf8` validates UTF-8 per LSP spec

**Write Logic:**
The `JsonRpc` wrapper struct with `#[serde(flatten)]` is elegant:
```rust
{"jsonrpc":"2.0","id":1,"method":"..."} // flattened
```
Without flatten, you'd get nested structure. The macro `invalid_data!` presumably creates io::ErrorKind::InvalidData consistently.

**Contribution Tip:**
When implementing framed protocols:
1. **Validate framing strictly** - reject malformed input early
2. **Reuse allocations** - reusing buf String across reads avoids allocation
3. **Exact-size reads** - use `read_exact` after reading Content-Length
4. **Flush writes** - ensure `out.flush()` after each message

For production, consider:
```rust
fn read_msg_text_with_limit(
    inp: &mut dyn BufRead,
    max_size: usize
) -> io::Result<Option<String>> {
    // Prevent DoS via huge Content-Length
}
```

**Common Pitfalls:**
- **Line ending mismatch:** Accepting `\n` instead of `\r\n` breaks protocol compliance
- **Partial reads:** Not using `read_exact` can leave partial message in buffer
- **UTF-8 panics:** Malformed UTF-8 should return Error, not panic
- **Buffer reuse bugs:** Forgetting `buf.clear()` causes header parsing to accumulate

**Related Patterns in Ecosystem:**
- **tokio::io::BufReader**: Async equivalent for tokio-based servers
- **hyper::HeaderMap**: More sophisticated header parsing
- **tower-http::content_length**: Middleware for HTTP Content-Length
- **httparse**: Zero-copy HTTP header parsing (overkill for LSP)

**Security Considerations:**
```rust
// Missing in pattern - should add:
const MAX_MESSAGE_SIZE: usize = 128 * 1024 * 1024; // 128 MB

if size > MAX_MESSAGE_SIZE {
    return Err(invalid_data!("message too large: {}", size));
}
```

**Performance Optimization:**
The pattern reuses the String buffer across reads in the loop:
```rust
let mut buf = String::new();
loop {
    buf.clear(); // Reuse allocation
    inp.read_line(&mut buf)?;
    // ...
}
```

---

## Pattern 8: Request Queue Pattern for LSP
**File:** lib/lsp-server/src/req_queue.rs
**Category:** Request/Response Tracking, LSP State
**Code Example:**
```rust
/// Manages the set of pending requests, both incoming and outgoing.
#[derive(Debug)]
pub struct ReqQueue<I, O> {
    pub incoming: Incoming<I>,
    pub outgoing: Outgoing<O>,
}

#[derive(Debug)]
pub struct Incoming<I> {
    pending: HashMap<RequestId, I>,
}

#[derive(Debug)]
pub struct Outgoing<O> {
    next_id: i32,
    pending: HashMap<RequestId, O>,
}

impl<I> Incoming<I> {
    pub fn register(&mut self, id: RequestId, data: I) {
        self.pending.insert(id, data);
    }

    pub fn cancel(&mut self, id: RequestId) -> Option<Response> {
        let _data = self.complete(&id)?;
        let error = ResponseError {
            code: ErrorCode::RequestCanceled as i32,
            message: "canceled by client".to_owned(),
            data: None,
        };
        Some(Response { id, result: None, error: Some(error) })
    }

    pub fn complete(&mut self, id: &RequestId) -> Option<I> {
        self.pending.remove(id)
    }
}

impl<O> Outgoing<O> {
    pub fn register<P: serde::Serialize>(&mut self, method: String, params: P, data: O) -> Request {
        let id = RequestId::from(self.next_id);
        self.pending.insert(id.clone(), data);
        self.next_id += 1;
        Request::new(id, method, params)
    }

    pub fn complete(&mut self, id: RequestId) -> Option<O> {
        self.pending.remove(&id)
    }
}
```
**Why This Matters for Contributors:**
ReqQueue tracks in-flight requests bidirectionally. Incoming stores server-generated metadata (like cancellation tokens), while Outgoing auto-generates request IDs and stores client-side context. The cancel() method constructs a proper LSP RequestCanceled error. This pattern is essential for LSP servers that need to correlate responses with request context or implement cancellation.

---

### Expert Rust Commentary: Pattern 8

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Essential LSP Pattern)

**Pattern Classification:** L2 Standard Library + LSP-Specific State Management
- **Primary Category:** Request/response correlation and lifecycle tracking
- **Related Patterns:** HashMap for tracking (A.39), error handling (2.1-2.10), LSP cancellation (A.25)

**Rust-Specific Insight:**
ReqQueue solves a fundamental problem in bidirectional RPC: correlating responses with their originating requests while tracking application-specific context. The dual-direction design is elegant:

**Incoming (server handling client requests):**
- `register(id, data)` stores server-side context (e.g., cancellation token, start time)
- `cancel(id)` removes pending request and generates proper LSP error response
- `complete(id)` removes context when response is sent

**Outgoing (client sending requests to server):**
- `register(method, params, data)` auto-generates monotonic IDs and stores client context
- `complete(id)` retrieves context when response arrives

The `next_id: i32` counter ensures unique IDs. Using i32 (not u32/u64) matches LSP spec (JSON-RPC allows negative IDs, though rarely used). The generic parameters I and O enable type-safe context storage - no need for `Any` or downcasting.

**Contribution Tip:**
When building RPC systems with request/response correlation:
1. **Separate incoming/outgoing** - different ID generation strategies
2. **Generic context types** - avoid `Box<dyn Any>` if possible
3. **Cancellation support** - include in incoming queue handling
4. **Metrics/timeouts** - store request start time in context

Example context types:
```rust
struct IncomingContext {
    cancel_token: CancellationToken,
    start_time: Instant,
}

struct OutgoingContext {
    sender: oneshot::Sender<Response>,
}

type MyReqQueue = ReqQueue<IncomingContext, OutgoingContext>;
```

**Common Pitfalls:**
- **ID reuse:** Don't reuse IDs immediately after completion - wrap-around at i32::MAX could collide
- **Memory leaks:** If client disconnects, pending requests stay in HashMap - need timeout mechanism
- **Cancellation race:** cancel() might fire after complete() - order matters

**Related Patterns in Ecosystem:**
- **tonic::Request**: gRPC request metadata storage
- **tower::Service**: Similar request/response tracking in middleware
- **actix::Addr**: Actor mailbox with request tracking
- **tarpc::context**: RPC context with deadlines and trace IDs

**Advanced Pattern - Request Lifecycle:**
```rust
impl<I> Incoming<I> {
    pub fn is_active(&self, id: &RequestId) -> bool {
        self.pending.contains_key(id)
    }

    pub fn active_count(&self) -> usize {
        self.pending.len()
    }

    // Timeout old requests
    pub fn timeout_old_requests(&mut self, max_age: Duration)
        -> Vec<Response>
    where
        I: HasTimestamp,
    {
        let now = Instant::now();
        let mut timeouts = Vec::new();
        self.pending.retain(|id, data| {
            if now - data.started_at() > max_age {
                timeouts.push(timeout_response(id.clone()));
                false
            } else {
                true
            }
        });
        timeouts
    }
}
```

---

## Pattern 9: Three-Thread LSP I/O Architecture
**File:** lib/lsp-server/src/stdio.rs
**Category:** Concurrency, LSP Transport
**Code Example:**
```rust
pub(crate) fn stdio_transport() -> (Sender<Message>, Receiver<Message>, IoThreads) {
    let (drop_sender, drop_receiver) = bounded::<Message>(0);
    let (writer_sender, writer_receiver) = bounded::<Message>(0);

    let writer = thread::Builder::new()
        .name("LspServerWriter".to_owned())
        .spawn(move || {
            let stdout = stdout();
            let mut stdout = stdout.lock();
            writer_receiver.into_iter().try_for_each(|it| {
                let result = it.write(&mut stdout);
                let _ = drop_sender.send(it);
                result
            })
        })
        .unwrap();

    let dropper = thread::Builder::new()
        .name("LspMessageDropper".to_owned())
        .spawn(move || drop_receiver.into_iter().for_each(drop))
        .unwrap();

    let (reader_sender, reader_receiver) = bounded::<Message>(0);
    let reader = thread::Builder::new()
        .name("LspServerReader".to_owned())
        .spawn(move || {
            let stdin = stdin();
            let mut stdin = stdin.lock();
            while let Some(msg) = Message::read(&mut stdin)? {
                let is_exit = matches!(&msg, Message::Notification(n) if n.is_exit());
                if let Err(e) = reader_sender.send(msg) {
                    return Err(io::Error::other(e));
                }
                if is_exit {
                    break;
                }
            }
            Ok(())
        })
        .unwrap();

    let threads = IoThreads { reader, writer, dropper };
    (writer_sender, reader_receiver, threads)
}

pub struct IoThreads {
    reader: thread::JoinHandle<io::Result<()>>,
    writer: thread::JoinHandle<io::Result<()>>,
    dropper: thread::JoinHandle<()>,
}

impl IoThreads {
    pub fn join(self) -> io::Result<()> {
        match self.reader.join() {
            Ok(r) => r?,
            Err(err) => std::panic::panic_any(err),
        }
        match self.dropper.join() {
            Ok(_) => (),
            Err(err) => std::panic::panic_any(err),
        }
        match self.writer.join() {
            Ok(r) => r,
            Err(err) => std::panic::panic_any(err),
        }
    }
}
```
**Why This Matters for Contributors:**
The three-thread architecture separates concerns: reader reads from stdin, writer writes to stdout, dropper drops messages after writing (to avoid blocking the writer). Using bounded(0) channels (rendezvous channels) provides backpressure. The reader exits on the "exit" notification. The dropper thread is necessary because Message owns heap allocations that need to be dropped off the writer thread. This pattern demonstrates proper concurrent I/O for LSP servers.

---

### Expert Rust Commentary: Pattern 9

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Concurrency Architecture Mastery)

**Pattern Classification:** L2 Standard Library + Concurrency Patterns
- **Primary Category:** Multi-threaded I/O with channel-based communication
- **Related Patterns:** Channel patterns (5.5, 32.1-32.10), RAII (4.1), thread safety (12.5), backpressure (A.12)

**Rust-Specific Insight:**
This three-thread architecture is a masterclass in concurrent I/O design. The separation of concerns is surgical:

**Reader Thread:**
- Reads from stdin (blocking I/O)
- Sends messages to main thread via bounded(0) rendezvous channel
- Exits on "exit" notification per LSP spec
- Errors propagate via `JoinHandle<io::Result<()>>`

**Writer Thread:**
- Receives messages from main thread via bounded(0) channel
- Writes to stdout (blocking I/O)
- **Critical:** Sends message to dropper after writing (doesn't drop in-thread)

**Dropper Thread:**
- Receives messages that have been written
- Drops them on a dedicated thread
- **Why?** Message owns heap allocations (Strings, Vecs) - dropping large messages blocks the writer thread

The `bounded(0)` rendezvous channel is brilliant - it provides natural backpressure. If the main thread can't keep up with incoming messages, the reader blocks. If writing is slow, the main thread blocks trying to send.

**Contribution Tip:**
When building concurrent I/O systems:
1. **Separate I/O threads from processing** - blocking I/O shouldn't block business logic
2. **Dedicated dropper thread** - for large allocations, drop on separate thread
3. **Rendezvous channels for backpressure** - bounded(0) prevents unbounded queueing
4. **Propagate errors via JoinHandle** - don't swallow thread panics

For async alternatives:
```rust
// Tokio-based LSP I/O
pub(crate) async fn tokio_stdio_transport()
    -> (mpsc::Sender<Message>, mpsc::Receiver<Message>)
{
    let (tx, rx) = mpsc::channel(1); // Bounded async channel

    tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();
        while let Some(msg) = Message::read_async(&mut lines).await? {
            tx.send(msg).await?;
        }
    });
    // ...
}
```

**Common Pitfalls:**
- **Thread leak:** Not joining IoThreads causes graceful shutdown to fail
- **Deadlock risk:** Using bounded channels requires careful ordering (this pattern avoids it)
- **Drop performance:** Without dropper thread, large message drops block writer thread
- **Panic handling:** Thread panic propagation via `panic_any` in join()

**Related Patterns in Ecosystem:**
- **tokio::io::split()**: Async reader/writer separation
- **crossbeam::scope()**: Scoped threads with guaranteed join
- **rayon::ThreadPool**: Work-stealing for CPU-bound tasks
- **parking_lot**: Faster primitives than std::sync

**Performance Analysis:**
```rust
// Memory overhead per thread:
// - Stack: ~2MB default (can configure with Builder)
// - Channel: bounded(0) = no queueing, minimal overhead
// - Total: ~6MB for 3 threads

// Latency characteristics:
// - Rendezvous: max 2 context switches per message
// - No queueing delay (messages processed immediately)
// - Writer thread prevents stdout lock contention
```

**Advanced Pattern - Graceful Shutdown:**
```rust
impl IoThreads {
    pub fn shutdown(self) -> io::Result<()> {
        // Drop sender to signal reader to exit
        drop(sender);

        // Wait for threads to finish
        self.join()
    }
}
```

---

## Pattern 10: Line Index with SIMD Newline Detection
**File:** lib/line-index/src/lib.rs
**Category:** Performance, SIMD Optimization, Text Processing
**Code Example:**
```rust
/// Maps flat [`TextSize`] offsets to/from `(line, column)` representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    /// Offset the beginning of each line (except the first, which always has offset 0).
    newlines: Box<[TextSize]>,
    /// List of non-ASCII characters on each line.
    line_wide_chars: IntMap<u32, Box<[WideChar]>>,
    /// The length of the entire text.
    len: TextSize,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn analyze_source_file_dispatch(
    src: &str,
    lines: &mut Vec<TextSize>,
    multi_byte_chars: &mut IntMap<u32, Vec<WideChar>>,
) {
    if is_x86_feature_detected!("sse2") {
        // SAFETY: SSE2 support was checked
        unsafe {
            analyze_source_file_sse2(src, lines, multi_byte_chars);
        }
    } else {
        analyze_source_file_generic(src, src.len(), TextSize::from(0), lines, multi_byte_chars);
    }
}

#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn analyze_source_file_sse2(
    src: &str,
    lines: &mut Vec<TextSize>,
    multi_byte_chars: &mut IntMap<u32, Vec<WideChar>>,
) {
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 16;
    let src_bytes = src.as_bytes();
    let chunk_count = src.len() / CHUNK_SIZE;
    let mut intra_chunk_offset = 0;

    for chunk_index in 0..chunk_count {
        let ptr = src_bytes.as_ptr() as *const __m128i;
        let chunk = unsafe { _mm_loadu_si128(ptr.add(chunk_index)) };

        // Check for multibyte chars (byte < 0)
        let multibyte_test = _mm_cmplt_epi8(chunk, _mm_set1_epi8(0));
        let multibyte_mask = _mm_movemask_epi8(multibyte_test);

        if multibyte_mask == 0 {
            // Pure ASCII - check for newlines
            let newlines_test = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
            let newlines_mask = _mm_movemask_epi8(newlines_test);

            if newlines_mask != 0 {
                let mut newlines_mask = 0xFFFF0000 | newlines_mask as u32;
                let output_offset = TextSize::from((chunk_index * CHUNK_SIZE + 1) as u32);

                loop {
                    let index = newlines_mask.trailing_zeros();
                    if index >= CHUNK_SIZE as u32 {
                        break;
                    }
                    lines.push(TextSize::from(index) + output_offset);
                    newlines_mask &= (!1) << index;
                }
            }
            continue;
        }

        // Fallback to generic for non-ASCII
        let scan_start = chunk_index * CHUNK_SIZE + intra_chunk_offset;
        intra_chunk_offset = analyze_source_file_generic(
            &src[scan_start..],
            CHUNK_SIZE - intra_chunk_offset,
            TextSize::from(scan_start as u32),
            lines,
            multi_byte_chars,
        );
    }
}
```
**Why This Matters for Contributors:**
LineIndex uses SSE2 intrinsics to process 16 bytes at once, detecting newlines and multibyte characters in parallel. The `_mm_cmplt_epi8` checks for UTF-8 continuation bytes (< 0), while `_mm_cmpeq_epi8` finds newlines. When multibyte chars are detected, it falls back to generic parsing. The ARM NEON variant follows similar patterns. This demonstrates platform-specific SIMD optimization for text processing, a pattern relevant for high-performance compiler infrastructure.

---

### Expert Rust Commentary: Pattern 10

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Performance Engineering Masterpiece)

**Pattern Classification:** L3 Platform-Specific + SIMD Optimization
- **Primary Category:** SIMD-accelerated text processing for compiler infrastructure
- **Related Patterns:** Platform abstraction (29.1-29.10), SIMD (13.7), zero-cost abstractions (22.1-22.10)

**Rust-Specific Insight:**
This pattern demonstrates production-grade SIMD optimization in Rust. The dispatch pattern uses runtime CPU feature detection to choose the best implementation:

**x86/x86_64 with SSE2:**
- Processes 16 bytes per iteration using 128-bit SIMD registers
- `_mm_cmplt_epi8(chunk, 0)` detects UTF-8 continuation bytes (high bit set)
- `_mm_cmpeq_epi8(chunk, '\n')` finds newlines in parallel
- `_mm_movemask_epi8` converts comparison results to bitmask
- `trailing_zeros()` finds first set bit for newline positions

**Fallback to generic:**
When multibyte characters detected, falls back to scalar code. This is correct - UTF-8 variable-length encoding makes SIMD complex for non-ASCII.

**Safety guarantees:**
The `#[target_feature(enable = "sse2")]` attribute ensures function only called when SSE2 available. The SAFETY comment documents the runtime check. The `unsafe` block is minimal and well-justified.

**Contribution Tip:**
When adding SIMD optimizations:
1. **Runtime dispatch** - use `is_x86_feature_detected!` for portable binaries
2. **Target feature attribute** - mark SIMD functions with `#[target_feature]`
3. **Fallback path** - always provide scalar implementation
4. **Safety documentation** - explain why unsafe is sound
5. **Benchmark** - profile to ensure SIMD actually helps

For modern CPUs, consider AVX2 (32 bytes) or AVX-512 (64 bytes):
```rust
#[target_feature(enable = "avx2")]
unsafe fn analyze_source_file_avx2(...) {
    use std::arch::x86_64::*;
    const CHUNK_SIZE: usize = 32; // _mm256_*
    // ...
}
```

**Common Pitfalls:**
- **Alignment:** SIMD loads require aligned data - use `_mm_loadu_si128` (unaligned) not `_mm_load_si128`
- **Partial reads:** Last chunk might be <16 bytes - need scalar fallback
- **Platform assumptions:** Code only works on x86/ARM - needs feature gates
- **Performance regression:** SIMD overhead can hurt small inputs - benchmark threshold

**Related Patterns in Ecosystem:**
- **memchr crate**: Production-quality SIMD string searching
- **bytecount**: SIMD byte counting
- **simd-json**: SIMD JSON parsing
- **Highway library**: Portable SIMD abstraction (C++ but relevant)

**Performance Characteristics:**
```rust
// ASCII newline detection (SSE2):
// - 16 bytes per iteration
// - ~2-3 CPU cycles per iteration (pipelined)
// - ~5-8 GB/s throughput on modern CPUs

// Scalar fallback:
// - 1 byte per iteration
// - ~0.5-1 GB/s throughput
// - 8-16x slower than SIMD
```

**Advanced Pattern - Multi-Platform SIMD:**
```rust
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn dispatch() { /* SSE2/AVX2 */ }

#[cfg(target_arch = "aarch64")]
fn dispatch() { /* ARM NEON */ }

#[cfg(target_arch = "wasm32")]
fn dispatch() { /* WASM SIMD */ }

#[cfg(not(any(/* ... */)))]
fn dispatch() { /* Scalar fallback */ }
```

---

## Pattern 11: UTF-8/UTF-16/UTF-32 Offset Conversion
**File:** lib/line-index/src/lib.rs
**Category:** Encoding Handling, LSP Interop
**Code Example:**
```rust
/// A kind of wide character encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WideEncoding {
    Utf16,
    Utf32,
}

impl WideEncoding {
    pub fn measure(&self, text: &str) -> usize {
        match self {
            WideEncoding::Utf16 => text.encode_utf16().count(),
            WideEncoding::Utf32 => text.chars().count(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WideChar {
    start: TextSize,
    end: TextSize,
}

impl WideChar {
    fn wide_len(&self, enc: WideEncoding) -> u32 {
        match enc {
            WideEncoding::Utf16 => {
                if self.len() == TextSize::from(4) {
                    2  // Surrogate pair
                } else {
                    1
                }
            }
            WideEncoding::Utf32 => 1,
        }
    }
}

impl LineIndex {
    /// Transforms the `LineCol` with the given `WideEncoding` into a `WideLineCol`.
    pub fn to_wide(&self, enc: WideEncoding, line_col: LineCol) -> Option<WideLineCol> {
        let mut col = line_col.col;
        if let Some(wide_chars) = self.line_wide_chars.get(&line_col.line) {
            for c in wide_chars.iter() {
                if u32::from(c.end) <= line_col.col {
                    col = col.checked_sub(u32::from(c.len()) - c.wide_len(enc))?;
                } else {
                    break;
                }
            }
        }
        Some(WideLineCol { line: line_col.line, col })
    }

    /// Transforms the `WideLineCol` with the given `WideEncoding` into a `LineCol`.
    pub fn to_utf8(&self, enc: WideEncoding, line_col: WideLineCol) -> Option<LineCol> {
        let mut col = line_col.col;
        if let Some(wide_chars) = self.line_wide_chars.get(&line_col.line) {
            for c in wide_chars.iter() {
                if col > u32::from(c.start) {
                    col = col.checked_add(u32::from(c.len()) - c.wide_len(enc))?;
                } else {
                    break;
                }
            }
        }
        Some(LineCol { line: line_col.line, col })
    }
}
```
**Why This Matters for Contributors:**
LSP clients may use UTF-16 (VSCode) or UTF-32 offsets, while Rust uses UTF-8. LineIndex tracks multibyte characters per line and converts between encodings. For UTF-16, 4-byte chars become surrogate pairs (length 2). The conversion logic iterates through wide chars, adjusting column offsets by the difference between UTF-8 and target encoding lengths. This pattern is essential for any tool interfacing with editors that use non-UTF-8 encoding.

---

### Expert Rust Commentary: Pattern 11

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Critical LSP Interop Pattern)

**Pattern Classification:** L2 Standard Library + LSP Encoding Interop
- **Primary Category:** Multi-encoding offset conversion for editor integration
- **Related Patterns:** Newtype (A.5), encoding handling (18.5-18.10), LSP protocol compliance

**Rust-Specific Insight:**
This pattern solves a fundamental LSP problem: Rust uses UTF-8 byte offsets, but editors use different encodings. VSCode uses UTF-16 (JavaScript heritage), some editors use UTF-32 (Unicode codepoint count). The conversion logic is subtle:

**UTF-16 Encoding:**
- 1-3 byte UTF-8 chars → 1 UTF-16 code unit
- 4-byte UTF-8 chars → 2 UTF-16 code units (surrogate pair)
- Example: "𝕊" (U+1D54A) is 4 bytes in UTF-8, 2 in UTF-16

**UTF-32 Encoding:**
- Always 1 codepoint = 1 UTF-32 code unit
- Simpler: just count chars with `.chars().count()`

**Conversion Algorithm:**
The to_wide/to_utf8 methods iterate through tracked WideChar entries (multibyte chars on that line) and adjust column offsets by the encoding difference:
```rust
// UTF-8 -> UTF-16 adjustment
col -= (utf8_len - utf16_len)

// UTF-16 -> UTF-8 reverse
col += (utf8_len - utf16_len)
```

The `checked_sub/checked_add` prevents underflow/overflow for invalid positions.

**Contribution Tip:**
When implementing multi-encoding support:
1. **Track multibyte chars** - only these need conversion
2. **Per-line tracking** - enables O(wide_chars_on_line) conversion, not O(all_chars)
3. **Checked arithmetic** - return Option for invalid positions
4. **Test edge cases** - emoji (4-byte), CJK (3-byte), BMP chars (1-2 byte)

Test cases to include:
```rust
#[test]
fn test_encodings() {
    let text = "a𝕊b🦀c";  // a=1B, 𝕊=4B, b=1B, 🦀=4B, c=1B
    let idx = LineIndex::new(text);

    // UTF-8 offsets: 0, 1, 5, 6, 10, 11
    // UTF-16 offsets: 0, 1, 3, 4, 6, 7  (𝕊 and 🦀 are 2 units)
    // UTF-32 offsets: 0, 1, 2, 3, 4, 5

    let col_utf8 = 5; // Start of 'b'
    let col_utf16 = idx.to_wide(Utf16, LineCol { line: 0, col: col_utf8 });
    assert_eq!(col_utf16.unwrap().col, 3);
}
```

**Common Pitfalls:**
- **Off-by-one:** Column offsets are 0-indexed, line numbers might be 1-indexed in LSP
- **BMP assumption:** Assuming all UTF-16 chars are 1 unit (fails for emoji/math symbols)
- **Grapheme clusters:** This handles codepoints, not graphemes ("👨‍👩‍👧" is multiple codepoints)
- **Performance:** Iterating wide_chars is O(n) - cache conversions if hot path

**Related Patterns in Ecosystem:**
- **ropey crate**: Rope with multi-encoding support
- **unicode-segmentation**: Grapheme cluster iteration
- **encoding_rs**: General charset conversion
- **tower-lsp**: LSP framework handling encoding automatically

**LSP Position Encoding:**
```rust
// LSP InitializeParams includes positionEncoding:
// ["utf-8", "utf-16", "utf-32"]

// Server declares support, client picks one
pub enum PositionEncodingKind {
    UTF8,   // Byte offsets (rare, rust-analyzer uses)
    UTF16,  // VSCode, most editors
    UTF32,  // Some legacy editors
}
```

---

## Pattern 12: Newtype Pattern with Transparent Serde
**File:** lib/text-size/src/size.rs
**Category:** Type Safety, Newtype Pattern
**Code Example:**
```rust
/// A measure of text length. Also, equivalently, an index into text.
///
/// This is a UTF-8 bytes offset stored as `u32`, but
/// most clients should treat it as an opaque measure.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextSize {
    pub(crate) raw: u32,
}

impl fmt::Debug for TextSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl From<u32> for TextSize {
    #[inline]
    fn from(raw: u32) -> Self {
        TextSize { raw }
    }
}

impl From<TextSize> for u32 {
    #[inline]
    fn from(value: TextSize) -> Self {
        value.raw
    }
}

impl From<TextSize> for usize {
    #[inline]
    fn from(value: TextSize) -> Self {
        value.raw as usize
    }
}

impl TryFrom<usize> for TextSize {
    type Error = TryFromIntError;
    #[inline]
    fn try_from(value: usize) -> Result<Self, TryFromIntError> {
        Ok(u32::try_from(value)?.into())
    }
}

macro_rules! ops {
    (impl $Op:ident for TextSize by fn $f:ident = $op:tt) => {
        impl $Op<TextSize> for TextSize {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: TextSize) -> TextSize {
                TextSize { raw: self.raw $op other.raw }
            }
        }
        impl $Op<&TextSize> for TextSize {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: &TextSize) -> TextSize {
                self $op *other
            }
        }
        impl<T> $Op<T> for &TextSize
        where
            TextSize: $Op<T, Output=TextSize>,
        {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: T) -> TextSize {
                *self $op other
            }
        }
    };
}

ops!(impl Add for TextSize by fn add = +);
ops!(impl Sub for TextSize by fn sub = -);
```
**Why This Matters for Contributors:**
TextSize wraps u32 to provide type safety for text offsets. It prevents mixing byte offsets with line numbers or other integers. The newtype is u32 (not usize) to save memory on 64-bit systems. The macro generates operator overloads for &TextSize to avoid clones. The From implementations enable ergonomic conversions while TryFrom<usize> catches overflow. This pattern demonstrates proper newtype design for domain-specific integer types.

---

### Expert Rust Commentary: Pattern 12

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Textbook Newtype)

**Pattern Classification:** L2 Standard Library + Type Safety
- **Primary Category:** Newtype pattern for domain-specific integer types
- **Related Patterns:** Newtype (A.5), type safety (7.1), operator overloading (19.6), From/Into (9.1)

**Rust-Specific Insight:**
TextSize is the platonic ideal of the newtype pattern. It wraps u32 to provide:

1. **Type Safety:** Can't accidentally mix byte offsets with line numbers
2. **Memory Efficiency:** u32 saves 4 bytes vs usize on 64-bit (critical for large ASTs)
3. **Ergonomics:** Operator overloads make it feel like a primitive

**Design Decisions:**

**u32 not usize:**
Files larger than 4GB are vanishingly rare in source code. Using u32 saves memory without practical limits. The TryFrom<usize> handles overflow explicitly.

**Debug format:**
Custom Debug impl prints just the number (not "TextSize(42)"), improving debug output readability.

**Operator overloads via macro:**
The ops! macro generates impls for `T op T`, `T op &T`, `&T op T` combinations. This enables:
```rust
let a: TextSize = 10.into();
let b: TextSize = 20.into();
let c = a + b;        // T op T
let d = a + &b;       // T op &T
let e = &a + b;       // &T op T
let f = &a + &b;      // &T op &T
```

Without `&T` impls, you'd need explicit clones: `*&a + *&b`.

**AddAssign reuses Add:**
```rust
impl<A> AddAssign<A> for TextSize
where TextSize: Add<A, Output = TextSize>
```
This generic impl means any type A that works with Add automatically works with AddAssign. No code duplication.

**Contribution Tip:**
When creating domain-specific integer types:
1. Choose appropriate size (u32 for offsets, u8 for enum tags, etc.)
2. Implement Debug to print the value, not the wrapper
3. Use macros for operator boilerplate
4. Implement From for common conversions (u32, usize)
5. Use TryFrom for fallible conversions
6. Make it Copy if appropriate (zero-cost cloning)

**Common Pitfalls:**
- **Forgetting Copy:** TextSize is Copy, so comparisons like `if size == other` work without .clone()
- **Arithmetic overflow:** Add/Sub don't check overflow in release mode (intentional for performance)
- **From<usize> panic:** Using From instead of TryFrom on 64-bit can panic
- **Comparison with primitives:** Can't do `size == 42u32` without `.into()`

**Related Patterns in Ecosystem:**
- **bytesize crate**: Similar for byte counts with formatting (e.g., "1.5 MB")
- **time crate**: Duration type wrapping i64/u64
- **rust-decimal**: Precise decimal arithmetic newtype
- **uuid crate**: 128-bit identifier newtype

**Advanced Pattern - Checked Arithmetic:**
```rust
impl TextSize {
    pub fn checked_add(self, other: TextSize) -> Option<TextSize> {
        self.raw.checked_add(other.raw).map(TextSize::from)
    }

    pub fn saturating_sub(self, other: TextSize) -> TextSize {
        TextSize::from(self.raw.saturating_sub(other.raw))
    }
}
```

---

## Pattern 13: TextRange with Const Invariants
**File:** lib/text-size/src/range.rs
**Category:** Const Correctness, Range Types
**Code Example:**
```rust
/// A range in text, represented as a pair of [`TextSize`].
///
/// It is a logic error for `start` to be greater than `end`.
#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TextRange {
    // Invariant: start <= end
    start: TextSize,
    end: TextSize,
}

impl TextRange {
    /// Creates a new `TextRange` with the given `start` and `end` (`start..end`).
    ///
    /// # Panics
    /// Panics if `end < start`.
    #[inline]
    pub const fn new(start: TextSize, end: TextSize) -> TextRange {
        assert!(start.raw <= end.raw);
        TextRange { start, end }
    }

    /// Create a new `TextRange` with the given `offset` and `len` (`offset..offset + len`).
    #[inline]
    pub const fn at(offset: TextSize, len: TextSize) -> TextRange {
        TextRange::new(offset, TextSize::new(offset.raw + len.raw))
    }

    /// Create a zero-length range at the specified offset (`offset..offset`).
    #[inline]
    pub const fn empty(offset: TextSize) -> TextRange {
        TextRange { start: offset, end: offset }
    }

    /// The size of this range.
    #[inline]
    pub const fn len(self) -> TextSize {
        // HACK for const fn: math on primitives only
        TextSize { raw: self.end().raw - self.start().raw }
    }

    /// Check if this range is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        // HACK for const fn: math on primitives only
        self.start().raw == self.end().raw
    }
}

impl Index<TextRange> for str {
    type Output = str;
    #[inline]
    fn index(&self, index: TextRange) -> &str {
        &self[Range::<usize>::from(index)]
    }
}

impl RangeBounds<TextSize> for TextRange {
    fn start_bound(&self) -> Bound<&TextSize> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> Bound<&TextSize> {
        Bound::Excluded(&self.end)
    }
}
```
**Why This Matters for Contributors:**
TextRange enforces start <= end via const assertions, preventing invalid ranges at compile time where possible. All constructors are const fn, enabling compile-time range construction. The len() and is_empty() implementations directly access raw fields to remain const (const fn restrictions). Implementing Index for str and RangeBounds enables ergonomic usage like `text[range]` and range syntax. This pattern shows how to build const-correct, ergonomic range types.

---

### Expert Rust Commentary: Pattern 13

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Const Correctness Excellence)

**Pattern Classification:** L2 Standard Library + Const Generics
- **Primary Category:** Const-correct range type with compile-time validation
- **Related Patterns:** Const evaluation (37.1-37.10), RangeBounds (A.36), Index trait (9.5)

**Rust-Specific Insight:**
TextRange demonstrates the power of const fn for compile-time validation and zero-cost abstractions. Every constructor is const, enabling compile-time range construction:

**Const Constructors:**
```rust
const RANGE: TextRange = TextRange::new(
    TextSize::new(0),
    TextSize::new(100)
); // Validated at compile time!
```

The `assert!(start.raw <= end.raw)` in `new` is a const assertion - if violated at compile time, you get a compile error, not a runtime panic.

**Const Method Restrictions:**
The `len()` and `is_empty()` implementations directly access `raw` fields due to const fn limitations (at the time of writing). The comment "HACK for const fn" is honest - ideally you'd use `end - start`, but that requires const trait impls which weren't stable.

**Index Implementation:**
```rust
impl Index<TextRange> for str
```
This enables `text[range]` syntax, converting TextRange to `Range<usize>` under the hood. Extremely ergonomic.

**RangeBounds Implementation:**
Implementing RangeBounds means TextRange works with any API expecting range bounds:
```rust
let slice = &vec[TextRange::new(10.into(), 20.into())]; // Works!
```

**Contribution Tip:**
When building range types:
1. **Const constructors** - enable compile-time validation
2. **Invariants in type** - document "start <= end" as type invariant
3. **Index for relevant types** - make it work like built-in ranges
4. **RangeBounds trait** - enables usage with slice/vec/string APIs
5. **Zero-size for empty** - empty() returns ZST-optimized range

For advanced validation:
```rust
impl TextRange {
    pub const fn checked_new(start: TextSize, end: TextSize)
        -> Option<Self>
    {
        if start.raw <= end.raw {
            Some(TextRange { start, end })
        } else {
            None
        }
    }
}
```

**Common Pitfalls:**
- **Unchecked construction:** Don't expose constructors that skip validation
- **Overflow arithmetic:** `end - start` can underflow if validation missed
- **Empty range confusion:** `new(10, 10)` is valid (empty range at offset 10)
- **Comparison semantics:** PartialOrd/Ord compare lexicographically (start, then end)

**Related Patterns in Ecosystem:**
- **std::ops::Range**: Standard library range, but not const-correct
- **rowan::TextRange**: Used in rust-analyzer's CST (concrete syntax tree)
- **logos::Span**: Similar range type for lexer output
- **proc-macro2::Span**: Token span in proc-macros

**Const Validation Pattern:**
```rust
// This works at compile time:
const fn validate_ranges() {
    const R1: TextRange = TextRange::new(
        TextSize::new(0),
        TextSize::new(100)
    );

    // This would fail at compile time:
    // const R2: TextRange = TextRange::new(
    //     TextSize::new(100),
    //     TextSize::new(0)  // ERROR: start > end
    // );
}
```

**Advanced Pattern - Range Algebra:**
```rust
impl TextRange {
    pub const fn contains(&self, offset: TextSize) -> bool {
        self.start.raw <= offset.raw && offset.raw < self.end.raw
    }

    pub const fn intersect(&self, other: TextRange) -> Option<TextRange> {
        let start = if self.start.raw > other.start.raw {
            self.start
        } else {
            other.start
        };
        let end = if self.end.raw < other.end.raw {
            self.end
        } else {
            other.end
        };

        if start.raw <= end.raw {
            Some(TextRange { start, end })
        } else {
            None
        }
    }
}
```

---

## Pattern 14: SmolStr - Inline String Optimization
**File:** lib/smol_str/src/lib.rs
**Category:** Memory Optimization, Small String Optimization
**Code Example:**
```rust
/// A `SmolStr` is a string type with O(1) clone and stack allocation for <=23 bytes.
pub struct SmolStr(Repr);

const INLINE_CAP: usize = 23;

#[derive(Clone, Debug)]
enum Repr {
    Inline { len: InlineSize, buf: [u8; INLINE_CAP] },
    Static(&'static str),
    Heap(Arc<str>),
}

impl SmolStr {
    #[inline]
    pub const fn new_inline(text: &str) -> SmolStr {
        assert!(text.len() <= INLINE_CAP);
        let text = text.as_bytes();
        let mut buf = [0; INLINE_CAP];
        let mut i = 0;
        while i < text.len() {
            buf[i] = text[i];
            i += 1;
        }
        SmolStr(Repr::Inline {
            len: unsafe { InlineSize::transmute_from_u8(text.len() as u8) },
            buf,
        })
    }

    #[inline(always)]
    pub const fn new_static(text: &'static str) -> SmolStr {
        SmolStr(Repr::Static(text))
    }

    #[inline(always)]
    pub fn new(text: impl AsRef<str>) -> SmolStr {
        SmolStr(Repr::new(text.as_ref()))
    }
}

impl Clone for SmolStr {
    #[inline]
    fn clone(&self) -> Self {
        #[cold]
        #[inline(never)]
        fn cold_clone(v: &SmolStr) -> SmolStr {
            SmolStr(v.0.clone())
        }

        if self.is_heap_allocated() {
            return cold_clone(self);
        }

        // SAFETY: Inline/Static variants are POD
        unsafe { core::ptr::read(self as *const SmolStr) }
    }
}

impl Repr {
    fn new(text: &str) -> Self {
        Self::new_on_stack(text).unwrap_or_else(|| Repr::Heap(Arc::from(text)))
    }

    fn new_on_stack<T>(text: T) -> Option<Self>
    where
        T: AsRef<str>,
    {
        let text = text.as_ref();
        let len = text.len();

        if len <= INLINE_CAP {
            let mut buf = [0; INLINE_CAP];
            buf[..len].copy_from_slice(text.as_bytes());
            return Some(Repr::Inline {
                len: unsafe { InlineSize::transmute_from_u8(len as u8) },
                buf,
            });
        }

        // Special case: whitespace-only strings (newlines + spaces)
        if len <= N_NEWLINES + N_SPACES {
            let bytes = text.as_bytes();
            let possible_newline_count = cmp::min(len, N_NEWLINES);
            let newlines = bytes[..possible_newline_count].iter().take_while(|&&b| b == b'\n').count();
            let possible_space_count = len - newlines;
            if possible_space_count <= N_SPACES && bytes[newlines..].iter().all(|&b| b == b' ') {
                let spaces = possible_space_count;
                let substring = &WS[N_NEWLINES - newlines..N_NEWLINES + spaces];
                return Some(Repr::Static(substring));
            }
        }
        None
    }
}
```
**Why This Matters for Contributors:**
SmolStr uses tagged enum with inline storage for strings <=23 bytes, static references for literals, and Arc<str> for large strings. Clone is O(1) for all variants: inline/static are memcpy'd, heap uses Arc::clone. The whitespace optimization stores common indentation patterns (newlines + spaces) as static string slices into a constant WS. The InlineSize enum with 24 variants provides niches for Option<SmolStr> optimization. This pattern demonstrates sophisticated small-value optimization for immutable strings.

---

### Expert Rust Commentary: Pattern 14

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Memory Optimization Masterpiece)

**Pattern Classification:** L2 Standard Library + Memory Optimization
- **Primary Category:** Small string optimization (SSO) with Arc fallback
- **Related Patterns:** SSO (A.55), Clone-on-Write (A.3), Arc for sharing (1.3), enum optimization (A.4)

**Rust-Specific Insight:**
SmolStr is a tour de force in memory optimization for immutable strings. The three-variant enum covers all cases:

**Inline (≤23 bytes):**
- Stored directly in the struct (no heap allocation)
- 23 bytes chosen to fit in 24 bytes with length byte (common cache line fraction)
- Clone is memcpy (O(1) copy of 24 bytes)

**Static (&'static str):**
- Zero allocation for string literals
- Clone is O(1) pointer copy
- Common for keywords, built-in names, etc.

**Heap (Arc<str>):**
- Reference-counted heap string for large strings
- Clone is O(1) Arc::clone (increment ref count)
- Multiple SmolStr can share the same heap allocation

**The Whitespace Trick:**
The pattern detects strings that are pure newlines + spaces (common for indentation) and stores them as static slices into a pre-allocated constant `WS`. This is genius - 1000 instances of "    " (4 spaces) share one static allocation.

**Clone Optimization:**
```rust
if self.is_heap_allocated() {
    return cold_clone(self);
}
unsafe { core::ptr::read(self as *const SmolStr) }
```

For inline/static, clone is an unsafe memcpy. The `#[cold]` and `#[inline(never)]` on heap clone path help branch prediction - the common case (inline/static) is hot, heap is cold.

**InlineSize Enum:**
Not shown in pattern, but InlineSize has 24 variants (0-23). This provides niche optimization - `Option<SmolStr>` is the same size as `SmolStr` (no extra discriminant).

**Contribution Tip:**
When implementing small-value optimization:
1. **Profile first** - measure string size distribution
2. **Choose inline size** - typically 23, 31, or 63 bytes
3. **Static detection** - dedupe common patterns (whitespace, keywords)
4. **Arc for large** - enables cheap cloning
5. **Optimize clone** - inline should be memcpy, not field-by-field

For different domains:
```rust
// SmolVec: similar for small vectors
pub struct SmolVec<T>(SmolVecRepr<T>);
enum SmolVecRepr<T> {
    Inline { len: u8, buf: [MaybeUninit<T>; N] },
    Heap(Arc<[T]>),
}
```

**Common Pitfalls:**
- **Inline size too large:** 64+ bytes makes struct unwieldy to pass around
- **No Arc for large:** Using Box instead means clone is O(n) copy
- **Forgetting static:** Missing optimization for string literals
- **Unsafe clone bugs:** memcpy only safe for POD types (Inline/Static are)

**Related Patterns in Ecosystem:**
- **smartstring**: Similar SSO string (inline size 23 or 24)
- **compact_str**: Another SSO string with different tradeoffs
- **arcstr**: Pure Arc<str> without inline (simpler but always heap)
- **String**: Standard library, no SSO (always heap for non-empty)

**Memory Layout:**
```rust
// Size analysis (64-bit):
// Repr enum discriminant: 1 byte
// Inline { len: 1 byte, buf: 23 bytes } = 24 bytes
// Static: 8 bytes (pointer)
// Heap: 8 bytes (Arc pointer)
// Total: max(24, 8, 8) + 1 = ~24 bytes

// With niche optimization:
// Option<SmolStr>: 24 bytes (no extra byte)
```

**Performance Characteristics:**
```rust
// Creation cost:
// "hello" (5 bytes): inline, 0 allocations, ~10ns
// "a".repeat(100) (100 bytes): Arc, 1 allocation, ~50ns
// "    " (spaces): static, 0 allocations, ~5ns

// Clone cost:
// Inline: 24-byte memcpy, ~1ns
// Static: pointer copy, ~1ns
// Heap: Arc increment, ~2ns (atomic op)
```

---

## Pattern 15: Builder Pattern with Inline Optimization
**File:** lib/smol_str/src/lib.rs
**Category:** Builder Pattern, Performance
**Code Example:**
```rust
/// A builder that can be used to efficiently build a [`SmolStr`].
/// This won't allocate if the final string fits into the inline buffer.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SmolStrBuilder(SmolStrBuilderRepr);

#[derive(Clone, Debug, PartialEq, Eq)]
enum SmolStrBuilderRepr {
    Inline { len: usize, buf: [u8; INLINE_CAP] },
    Heap(String),
}

impl SmolStrBuilder {
    pub const fn new() -> Self {
        Self(SmolStrBuilderRepr::Inline { buf: [0; INLINE_CAP], len: 0 })
    }

    pub fn push_str(&mut self, s: &str) {
        match &mut self.0 {
            SmolStrBuilderRepr::Inline { len, buf } => {
                let old_len = *len;
                *len += s.len();

                if *len <= INLINE_CAP {
                    buf[old_len..*len].copy_from_slice(s.as_bytes());
                    return;
                }

                // Overflow to heap
                let mut heap = String::with_capacity(*len);
                unsafe { heap.as_mut_vec().extend_from_slice(&buf[..old_len]) };
                heap.push_str(s);
                self.0 = SmolStrBuilderRepr::Heap(heap);
            }
            SmolStrBuilderRepr::Heap(heap) => heap.push_str(s),
        }
    }

    pub fn finish(&self) -> SmolStr {
        SmolStr(match &self.0 {
            &SmolStrBuilderRepr::Inline { len, buf } => {
                Repr::Inline {
                    len: unsafe { InlineSize::transmute_from_u8(len as u8) },
                    buf,
                }
            }
            SmolStrBuilderRepr::Heap(heap) => Repr::new(heap),
        })
    }
}

impl fmt::Write for SmolStrBuilder {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! format_smolstr {
    ($($tt:tt)*) => {{
        let mut w = $crate::SmolStrBuilder::new();
        ::core::fmt::Write::write_fmt(&mut w, format_args!($($tt)*))
            .expect("a formatting trait implementation returned an error");
        w.finish()
    }};
}
```
**Why This Matters for Contributors:**
SmolStrBuilder starts with inline storage and transitions to heap only when necessary. The push_str implementation copies to inline buffer until overflow, then allocates a String with exact capacity and transitions state. Implementing fmt::Write enables usage with format_args! and write! macros. The format_smolstr! macro provides format!-like syntax that avoids allocation for small formatted strings. This pattern shows how to build efficient, allocation-minimizing builders.

---

### Expert Rust Commentary: Pattern 15

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Builder Pattern Excellence)

**Pattern Classification:** L2 Standard Library + Builder Patterns
- **Primary Category:** Allocation-minimizing builder with state transition
- **Related Patterns:** Builder pattern (3.1-3.10), SSO (A.55), fmt::Write (10.7), macro design (10.1)

**Rust-Specific Insight:**
SmolStrBuilder extends the SSO pattern to incremental construction. The genius is in the state transition:

**Start in Inline:**
- Allocates 23-byte inline buffer on stack
- All push_str operations copy to this buffer
- Zero heap allocations if final size ≤23 bytes

**Transition to Heap:**
When overflow occurs:
1. Allocate String with exact capacity (old_len + new_len)
2. Copy inline buffer to heap using `unsafe { heap.as_mut_vec().extend_from_slice(&buf[..old_len]) }`
3. Push new string
4. Replace state with Heap variant

**The unsafe block** is necessary because `heap.as_mut_vec()` bypasses UTF-8 validation - we know the buffer is valid UTF-8 because it came from the inline buffer.

**fmt::Write Integration:**
```rust
impl fmt::Write for SmolStrBuilder
```
This enables usage with `write!` and `format_args!`:
```rust
let mut builder = SmolStrBuilder::new();
write!(&mut builder, "x = {}", 42).unwrap();
let s = builder.finish(); // SmolStr
```

**format_smolstr! Macro:**
The macro provides `format!`-like syntax:
```rust
let s = format_smolstr!("x = {}", 42);
// Allocates inline if result fits, heap otherwise
```

This avoids the intermediate String allocation that `format!().into()` would cause.

**Contribution Tip:**
When building efficient string builders:
1. **Start inline** - stack buffer for common case
2. **Transition once** - don't oscillate between inline/heap
3. **Exact capacity** - allocate precisely when transitioning
4. **Reuse allocations** - consider push_str that reuses existing heap
5. **Implement fmt::Write** - enables write! macro ecosystem

For mutable scenarios:
```rust
impl SmolStrBuilder {
    pub fn clear(&mut self) {
        match &mut self.0 {
            SmolStrBuilderRepr::Inline { len, .. } => *len = 0,
            SmolStrBuilderRepr::Heap(heap) => heap.clear(),
        }
    }
}
```

**Common Pitfalls:**
- **Multiple transitions:** Transitioning heap→inline on truncate adds complexity
- **Capacity waste:** Using String::new() instead of with_capacity on transition
- **UTF-8 bugs:** Manual buffer manipulation can break UTF-8 invariants
- **Clone cost:** Builder clone is cheap for inline, expensive for heap

**Related Patterns in Ecosystem:**
- **String**: Standard library, always heap (even empty allocates)
- **bumpalo::String**: Arena-allocated builder (no individual drops)
- **bytes::BytesMut**: For binary data builders
- **std::fmt::Formatter**: Similar buffered writing

**Performance Comparison:**
```rust
// Building "hello":
// String::new() + push_str: 1 allocation, ~50ns
// SmolStrBuilder: 0 allocations, ~10ns
// Speedup: 5x

// Building "a".repeat(100):
// String::with_capacity(100) + push_str: 1 allocation, ~100ns
// SmolStrBuilder: 1 allocation + 1 transition, ~120ns
// Cost: 20% overhead for transition
```

**Advanced Pattern - String Interning:**
```rust
pub struct StringInterner {
    map: HashMap<SmolStr, ()>,
}

impl StringInterner {
    pub fn intern(&mut self, s: &str) -> SmolStr {
        if let Some(existing) = self.map.get_key_value(s) {
            existing.0.clone() // O(1) Arc clone
        } else {
            let smol = SmolStr::new(s);
            self.map.insert(smol.clone(), ());
            smol
        }
    }
}
```

---

## Pattern 16: Ungrammar DSL with Simple Index Types
**File:** lib/ungrammar/src/lib.rs
**Category:** DSL Design, Newtype Indices
**Code Example:**
```rust
/// A node, like `A = 'b' | 'c'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node(usize);

/// A token, denoted with single quotes, like `'+'` or `'struct'`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token(usize);

/// An Ungrammar grammar.
#[derive(Default, Debug)]
pub struct Grammar {
    nodes: Vec<NodeData>,
    tokens: Vec<TokenData>,
}

impl ops::Index<Node> for Grammar {
    type Output = NodeData;
    fn index(&self, Node(index): Node) -> &NodeData {
        &self.nodes[index]
    }
}

impl ops::Index<Token> for Grammar {
    type Output = TokenData;
    fn index(&self, Token(index): Token) -> &TokenData {
        &self.tokens[index]
    }
}

/// A production rule.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Rule {
    Labeled { label: String, rule: Box<Rule> },
    Node(Node),
    Token(Token),
    Seq(Vec<Rule>),
    Alt(Vec<Rule>),
    Opt(Box<Rule>),
    Rep(Box<Rule>),
}

impl FromStr for Grammar {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        let tokens = lexer::tokenize(s)?;
        parser::parse(tokens)
    }
}
```
**Why This Matters for Contributors:**
Ungrammar defines a simple grammar DSL for specifying syntax trees. Node and Token are newtype indices (not PhantomData-based) since there's only one grammar per parse. The Rule enum recursively defines grammar productions: Seq for concatenation, Alt for alternatives, Opt/Rep for ?, *. Implementing FromStr enables parsing with `.parse()`. This pattern shows minimalist DSL design for compiler tooling, where simplicity beats generality.

---

### Expert Rust Commentary: Pattern 16

**Idiomatic Rating:** ⭐⭐⭐⭐ (4/5 - Pragmatic DSL Design)

**Pattern Classification:** L2 Standard Library + DSL Design
- **Primary Category:** Minimalist grammar DSL for specifying syntax trees
- **Related Patterns:** Newtype indices (A.5), recursive types (A.67), FromStr (9.1)

**Rust-Specific Insight:**
Ungrammar demonstrates the "simplest thing that works" philosophy. Unlike la-arena's type-safe Idx<T>, this uses bare newtype indices (Node(usize), Token(usize)) because:
1. Only one grammar per parse (no cross-grammar index confusion)
2. Simpler implementation (no PhantomData complexity)
3. Adequate type safety for the domain

**The Rule enum** is the heart of the DSL:
```rust
pub enum Rule {
    Labeled { label: String, rule: Box<Rule> },  // name:Rule
    Node(Node),                                   // NonTerminal
    Token(Token),                                 // 'terminal'
    Seq(Vec<Rule>),                              // A B C
    Alt(Vec<Rule>),                              // A | B | C
    Opt(Box<Rule>),                              // A?
    Rep(Box<Rule>),                              // A*
}
```

This directly models EBNF grammar notation. The Box<Rule> for Opt/Rep/Labeled prevents infinite-size recursive types.

**Index Trait:**
```rust
impl ops::Index<Node> for Grammar {
    type Output = NodeData;
    fn index(&self, Node(index): Node) -> &NodeData {
        &self.nodes[index]
    }
}
```

This enables `grammar[node]` syntax for accessing node data. Clean ergonomics.

**FromStr Implementation:**
```rust
impl FromStr for Grammar {
    fn from_str(s: &str) -> Result<Self> {
        let tokens = lexer::tokenize(s)?;
        parser::parse(tokens)
    }
}
```

Enables parsing with `.parse()`:
```rust
let grammar: Grammar = grammar_text.parse()?;
```

**Contribution Tip:**
When designing DSLs for tool internals:
1. **Simplicity over generality** - don't add features you don't need
2. **Recursive enum for AST** - natural representation of recursive grammars
3. **Index trait for ergonomics** - `grammar[node]` beats `grammar.get(node)`
4. **FromStr for parsing** - integrates with Rust ecosystem

For external DSLs (user-facing), consider:
- Better error messages (spans, suggestions)
- Pretty-printing (Display trait)
- Validation (well-formedness checks)

**Common Pitfalls:**
- **Infinite recursion:** Box<Rule> prevents it in type, but logic can still loop
- **Memory leaks:** If grammar has cycles (shouldn't but could), no detection
- **Poor error messages:** FromStr error is opaque (just String)

**Related Patterns in Ecosystem:**
- **pest**: Parser generator from PEG grammars
- **nom**: Parser combinator library
- **lalrpop**: LR parser generator
- **tree-sitter**: Incremental parser with error recovery

**Example Grammar:**
```rust
// In ungrammar notation:
// Expr = Literal | BinOp
// BinOp = lhs:Expr op:'+' rhs:Expr
// Literal = 'num'

let grammar: Grammar = r#"
Expr = Literal | BinOp
BinOp = lhs:Expr op:'+' rhs:Expr
Literal = 'num'
"#.parse()?;
```

---

## Pattern 17: Hand-Written Recursive Descent Parser
**File:** lib/ungrammar/src/parser.rs
**Category:** Parsing, Recursive Descent
**Code Example:**
```rust
#[derive(Default)]
struct Parser {
    grammar: Grammar,
    tokens: Vec<lexer::Token>,
    node_table: HashMap<String, Node>,
    token_table: HashMap<String, Token>,
}

const DUMMY_RULE: Rule = Rule::Node(Node(!0));

impl Parser {
    fn new(mut tokens: Vec<lexer::Token>) -> Parser {
        tokens.reverse();  // Use as stack
        Parser { tokens, ..Parser::default() }
    }

    fn peek(&self) -> Option<&lexer::Token> {
        self.peek_n(0)
    }

    fn peek_n(&self, n: usize) -> Option<&lexer::Token> {
        self.tokens.iter().nth_back(n)
    }

    fn bump(&mut self) -> Result<lexer::Token> {
        self.tokens.pop().ok_or_else(|| format_err!("unexpected EOF"))
    }

    fn intern_node(&mut self, name: String) -> Node {
        let len = self.node_table.len();
        let grammar = &mut self.grammar;
        *self.node_table.entry(name.clone()).or_insert_with(|| {
            grammar.nodes.push(NodeData { name, rule: DUMMY_RULE });
            Node(len)
        })
    }
}

fn rule(p: &mut Parser) -> Result<Rule> {
    let lhs = seq_rule(p)?;
    let mut alt = vec![lhs];

    while let Some(token) = p.peek() {
        if token.kind != TokenKind::Pipe {
            break;
        }
        p.bump()?;
        let rule = seq_rule(p)?;
        alt.push(rule)
    }

    let res = if alt.len() == 1 { alt.pop().unwrap() } else { Rule::Alt(alt) };
    Ok(res)
}

fn seq_rule(p: &mut Parser) -> Result<Rule> {
    let lhs = atom_rule(p)?;
    let mut seq = vec![lhs];

    while let Some(rule) = opt_atom_rule(p)? {
        seq.push(rule)
    }

    let res = if seq.len() == 1 { seq.pop().unwrap() } else { Rule::Seq(seq) };
    Ok(res)
}

fn opt_atom_rule(p: &mut Parser) -> Result<Option<Rule>> {
    let token = match p.peek() {
        Some(it) => it,
        None => return Ok(None),
    };

    let mut res = match &token.kind {
        TokenKind::Node(name) => {
            if let Some(lookahead) = p.peek_n(1) {
                match lookahead.kind {
                    TokenKind::Eq => return Ok(None),
                    TokenKind::Colon => {
                        let label = name.clone();
                        p.bump()?;
                        p.bump()?;
                        let rule = atom_rule(p)?;
                        return Ok(Some(Rule::Labeled { label, rule: Box::new(rule) }));
                    }
                    _ => (),
                }
            }
            let name = name.clone();
            p.bump()?;
            let node = p.intern_node(name);
            Rule::Node(node)
        }
        TokenKind::Token(name) => {
            let name = name.clone();
            p.bump()?;
            let token = p.intern_token(name);
            Rule::Token(token)
        }
        TokenKind::LParen => {
            p.bump()?;
            let rule = rule(p)?;
            p.expect(TokenKind::RParen, ")")?;
            rule
        }
        _ => return Ok(None),
    };

    // Handle postfix operators
    if let Some(token) = p.peek() {
        match &token.kind {
            TokenKind::QMark => {
                p.bump()?;
                res = Rule::Opt(Box::new(res));
            }
            TokenKind::Star => {
                p.bump()?;
                res = Rule::Rep(Box::new(res));
            }
            _ => (),
        }
    }

    Ok(Some(res))
}
```
**Why This Matters for Contributors:**
This is a classic recursive descent parser for ungrammar syntax. Tokens are stored reversed and popped like a stack. The parser uses interning (intern_node/intern_token) to deduplicate grammar symbols. Forward references are handled with DUMMY_RULE placeholders. peek_n enables lookahead for disambiguation (e.g., `Node:` vs `Node =`). The rule -> seq_rule -> atom_rule hierarchy encodes operator precedence. This pattern demonstrates hand-written parser implementation suitable for simple DSLs.

---

### Expert Rust Commentary: Pattern 17

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Recursive Descent Mastery)

**Pattern Classification:** L2 Standard Library + Parsing Patterns
- **Primary Category:** Hand-written recursive descent parser for ungrammar DSL
- **Related Patterns:** Recursive descent (A.94), error handling (2.1-2.10), interning (A.22)

**Rust-Specific Insight:**
This parser demonstrates classic recursive descent with Rust idioms:

**Token Stack Pattern:**
```rust
tokens.reverse();  // Use as stack
```
Reversing the token vector allows using `pop()` for parsing (O(1) from end). The `peek()` / `peek_n()` / `bump()` API is standard for recursive descent:
- `peek()` - lookahead without consuming
- `peek_n(1)` - multi-token lookahead
- `bump()` - consume and return token

**Interning with Dummy Values:**
```rust
const DUMMY_RULE: Rule = Rule::Node(Node(!0));
```
When forward references are encountered (e.g., `Expr = ... OtherNode ...` before OtherNode is defined), the parser allocates a node with DUMMY_RULE, then backfills later. The `!0` (max value) acts as a sentinel.

**Operator Precedence via Recursion Depth:**
```rust
rule -> seq_rule -> atom_rule
```
The function call hierarchy encodes precedence:
- `rule` handles `|` (alternation, lowest precedence)
- `seq_rule` handles concatenation (medium precedence)
- `atom_rule` handles atoms and postfix `?`/`*` (highest precedence)

**Lookahead for Disambiguation:**
```rust
if let Some(lookahead) = p.peek_n(1) {
    match lookahead.kind {
        TokenKind::Eq => return Ok(None),      // Node =
        TokenKind::Colon => { /* label:Node */ }
        _ => { /* bare Node */ }
    }
}
```
Two-token lookahead distinguishes node references from definitions.

**Contribution Tip:**
When writing recursive descent parsers:
1. **Token stack** - reverse and use pop() for O(1) consumption
2. **Peek/bump API** - standard interface for recursive functions
3. **Interning for forward refs** - allocate with dummy, backfill later
4. **Precedence via recursion** - function depth = operator precedence
5. **Minimal lookahead** - typically 1-2 tokens, more suggests ambiguous grammar

For better errors:
```rust
fn expect(&mut self, kind: TokenKind, expected: &str) -> Result<Token> {
    match self.peek() {
        Some(tok) if tok.kind == kind => self.bump(),
        Some(tok) => Err(format_err!(
            "expected {}, found {:?} at {:?}",
            expected, tok.kind, tok.span
        )),
        None => Err(format_err!("expected {}, found EOF", expected)),
    }
}
```

**Common Pitfalls:**
- **Left recursion:** `Expr = Expr '+' Num` causes infinite loop - rewrite as iteration
- **Ambiguous grammar:** Conflicts need lookahead or grammar refactoring
- **Poor error recovery:** Parser aborts on first error - consider panic mode
- **Memory leaks:** Forgetting to pop tokens leaves them in Vec

**Related Patterns in Ecosystem:**
- **nom**: Parser combinator alternative (functional style)
- **pest**: PEG parser generator
- **syn**: Production-quality Rust parser (recursive descent)
- **chumsky**: Error-recovery parser combinator

**Left Recursion Example:**
```rust
// BAD: infinite recursion
// Expr = Expr '+' Num | Num

// GOOD: rewrite as iteration
// Expr = Num ('+' Num)*

fn expr(p: &mut Parser) -> Result<Rule> {
    let mut lhs = atom(p)?;
    while matches!(p.peek(), Some(tok) if tok.kind == TokenKind::Plus) {
        p.bump()?;
        let rhs = atom(p)?;
        lhs = Rule::BinOp(Box::new(lhs), Box::new(rhs));
    }
    Ok(lhs)
}
```

**Advanced Pattern - Error Recovery:**
```rust
fn atom_rule_with_recovery(p: &mut Parser) -> Result<Rule> {
    match atom_rule(p) {
        Ok(rule) => Ok(rule),
        Err(e) => {
            eprintln!("parse error: {}", e);
            // Panic mode: skip until sync point
            while let Some(tok) = p.peek() {
                if matches!(tok.kind, TokenKind::Semicolon | TokenKind::RBrace) {
                    break;
                }
                p.bump()?;
            }
            Ok(Rule::Error) // Placeholder
        }
    }
}
```

---

## Pattern 18: Sealed Trait Pattern for Extension Traits
**File:** lib/text-size/src/traits.rs
**Category:** API Design, Sealed Traits
**Code Example:**
```rust
use priv_in_pub::Sealed;
mod priv_in_pub {
    pub trait Sealed {}
}

/// Primitives with a textual length that can be passed to [`TextSize::of`].
pub trait TextLen: Copy + Sealed {
    fn text_len(self) -> TextSize;
}

impl Sealed for &'_ str {}
impl TextLen for &'_ str {
    #[inline]
    fn text_len(self) -> TextSize {
        self.len().try_into().unwrap()
    }
}

impl Sealed for &'_ String {}
impl TextLen for &'_ String {
    #[inline]
    fn text_len(self) -> TextSize {
        self.as_str().text_len()
    }
}

impl Sealed for char {}
impl TextLen for char {
    #[inline]
    fn text_len(self) -> TextSize {
        (self.len_utf8() as u32).into()
    }
}
```
**Why This Matters for Contributors:**
The sealed trait pattern prevents external crates from implementing TextLen. The Sealed supertrait lives in a private module (priv_in_pub), making it impossible to name outside the crate. This enables API evolution: new methods can be added to TextLen without breaking downstream code (no risk of method name conflicts). The pattern is essential for public traits that should have a closed set of implementations, common in library crates that want to control trait semantics.

---

### Expert Rust Commentary: Pattern 18

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - API Stability Pattern)

**Pattern Classification:** L2 Standard Library + API Design
- **Primary Category:** Sealed trait pattern for controlled trait implementations
- **Related Patterns:** Sealed traits (A.46, 6.6), API stability (41.1-41.10), coherence (A.47)

**Rust-Specific Insight:**
The sealed trait pattern is the gold standard for preventing external trait implementations while allowing public methods. The mechanism is elegant:

**Private Sealed Supertrait:**
```rust
mod priv_in_pub {
    pub trait Sealed {}  // Public visibility but private module
}

pub trait TextLen: Copy + Sealed {  // Public trait
    fn text_len(self) -> TextSize;
}
```

The Sealed trait is in a private module, so:
- External crates can see it (it's `pub`)
- External crates can't name it (module is private)
- External crates can't implement it (can't satisfy trait bound)

**Why Seal Traits:**
1. **API Evolution:** Add methods to TextLen without breaking downstream
2. **Exhaustive Matching:** Know all impls exist, optimize accordingly
3. **Semantic Control:** Prevent nonsensical impls (e.g., `impl TextLen for File`)

**Controlled Implementations:**
```rust
impl Sealed for &'_ str {}
impl TextLen for &'_ str { /* ... */ }

impl Sealed for char {}
impl TextLen for char { /* ... */ }
```

The crate controls exactly which types implement TextLen. Adding new impls is non-breaking.

**Contribution Tip:**
Use sealed traits when:
1. **Exhaustive set** - you know all valid implementations
2. **Add methods later** - want to extend without breaking changes
3. **Semantic guarantees** - trait has invariants hard to verify externally

Pattern template:
```rust
mod private {
    pub trait Sealed {}
}

pub trait MyTrait: private::Sealed {
    fn method(&self);
    // Can add methods later without SemVer break
}

impl private::Sealed for MyType {}
impl MyTrait for MyType {
    fn method(&self) { /* ... */ }
}
```

**Common Pitfalls:**
- **Over-sealing:** Not all traits should be sealed - consider if third-party impls are useful
- **Documentation:** Explain why trait is sealed (users will wonder)
- **Module naming:** `priv_in_pub` or `private` are conventional names

**Related Patterns in Ecosystem:**
- **serde::Serialize:** NOT sealed (user types can derive it)
- **std::io::Read:** NOT sealed (custom streams common)
- **std::error::Error:** NOT sealed (custom error types)
- **std::slice::SliceIndex:** Sealed (only valid impls are usize, Range, etc.)

**Rust RFC History:**
The pattern emerged from RFC discussions about trait evolution. Before sealed traits, adding methods to public traits was a breaking change (downstream impls wouldn't compile).

**Advanced Pattern - Conditional Sealing:**
```rust
// Sealed in stable, open in unstable
#[cfg(not(feature = "unstable"))]
mod seal {
    pub trait Sealed {}
}

#[cfg(feature = "unstable")]
pub trait Sealed {}  // Public in unstable

pub trait MyTrait: Sealed { /* ... */ }
```

**API Guidelines Compliance:**
This pattern follows Rust API Guidelines:
- **C-SEALED**: Seal traits that users shouldn't implement
- **C-EVOLUTION**: Design for future extension
- **C-STABLE**: Prevent accidental breaking changes

---

## Pattern 19: Macro-Based Operator Overloading
**File:** lib/text-size/src/size.rs, lib/text-size/src/range.rs
**Category:** Macro Design, Operator Overloading
**Code Example:**
```rust
macro_rules! ops {
    (impl $Op:ident for TextSize by fn $f:ident = $op:tt) => {
        impl $Op<TextSize> for TextSize {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: TextSize) -> TextSize {
                TextSize { raw: self.raw $op other.raw }
            }
        }
        impl $Op<&TextSize> for TextSize {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: &TextSize) -> TextSize {
                self $op *other
            }
        }
        impl<T> $Op<T> for &TextSize
        where
            TextSize: $Op<T, Output=TextSize>,
        {
            type Output = TextSize;
            #[inline]
            fn $f(self, other: T) -> TextSize {
                *self $op other
            }
        }
    };
}

ops!(impl Add for TextSize by fn add = +);
ops!(impl Sub for TextSize by fn sub = -);

impl<A> AddAssign<A> for TextSize
where
    TextSize: Add<A, Output = TextSize>,
{
    #[inline]
    fn add_assign(&mut self, rhs: A) {
        *self = *self + rhs
    }
}
```
**Why This Matters for Contributors:**
The ops! macro generates operator overloads for T, &T, and &T op T combinations, reducing boilerplate. The macro accepts operator tokens ($op:tt) like + or -, enabling compile-time operator dispatch. The &T implementations are generic to leverage already-defined T operations. The AddAssign implementation is generic over any A where Add is defined, reusing the Add logic. This pattern demonstrates effective use of macros to reduce repetitive trait implementations while maintaining full type safety.

---

### Expert Rust Commentary: Pattern 19

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Macro Design Excellence)

**Pattern Classification:** L2 Standard Library + Macro Patterns
- **Primary Category:** Declarative macro for reducing operator overload boilerplate
- **Related Patterns:** Macro design (10.1-10.10, A.101-A.108), operator overloading (A.7)

**Rust-Specific Insight:**
This macro demonstrates the DRY principle applied to operator overloading. Without it, you'd write 9+ impls per operator:

**The Problem:**
```rust
// Without macro:
impl Add<TextSize> for TextSize { /* ... */ }
impl Add<&TextSize> for TextSize { /* ... */ }
impl<T> Add<T> for &TextSize where TextSize: Add<T> { /* ... */ }
impl AddAssign<TextSize> for TextSize { /* ... */ }
// ... repeat for Sub, Mul, Div, etc.
```

**The Solution:**
```rust
ops!(impl Add for TextSize by fn add = +);
ops!(impl Sub for TextSize by fn sub = -);
```

**Macro Breakdown:**
```rust
macro_rules! ops {
    (impl $Op:ident for TextSize by fn $f:ident = $op:tt) => {
        // $Op = Add, Sub, etc.
        // $f = add, sub, etc.
        // $op:tt = token tree (+ or -)

        impl $Op<TextSize> for TextSize { /* ... */ }
        impl $Op<&TextSize> for TextSize { /* ... */ }
        impl<T> $Op<T> for &TextSize
        where TextSize: $Op<T, Output=TextSize>
        { /* ... */ }
    };
}
```

**The `$op:tt` trick** allows passing operators as tokens. Using `$op:tt` (token tree) instead of specific tokens makes the macro flexible.

**The generic `&T` impl** is elegant:
```rust
impl<T> Add<T> for &TextSize
where TextSize: Add<T, Output=TextSize>
```
This says: if `TextSize + T` works, then `&TextSize + T` also works by dereferencing. Avoids duplicating logic.

**AddAssign Reuse:**
```rust
impl<A> AddAssign<A> for TextSize
where TextSize: Add<A, Output = TextSize>
{
    fn add_assign(&mut self, rhs: A) {
        *self = *self + rhs  // Reuse Add impl
    }
}
```

**Contribution Tip:**
When writing operator macros:
1. **Token trees for operators** - `$op:tt` accepts +, -, *, / etc.
2. **Generate &T variants** - enables borrowing without explicit clones
3. **Generic implementations** - reduce duplication with where clauses
4. **Reuse for compound ops** - AddAssign can delegate to Add

For more complex operators:
```rust
macro_rules! checked_ops {
    (impl $Op:ident for $T:ty by fn $f:ident = $op:tt) => {
        impl $T {
            pub fn $f(self, rhs: Self) -> Option<Self> {
                self.0.$f(rhs.0).map($T)
            }
        }
    };
}

checked_ops!(impl checked_add for TextSize by fn checked_add = checked_add);
```

**Common Pitfalls:**
- **Macro hygiene:** Variables in macro can shadow caller's variables - use gensym
- **Error messages:** Macro errors can be cryptic - document expected usage
- **Over-abstraction:** Not all operators fit this pattern (e.g., Neg, Not)
- **Type inference:** Generic impls can confuse inference - explicit types help

**Related Patterns in Ecosystem:**
- **derive_more**: Derives operator traits automatically
- **auto_ops**: Similar macro crate for operator overloading
- **std::ops traits**: Standard library operator traits

**Advanced Pattern - Commutative Operators:**
```rust
macro_rules! commutative_op {
    (impl $Op:ident for $T:ty, $U:ty by fn $f:ident = $op:tt) => {
        impl $Op<$U> for $T { /* ... */ }
        impl $Op<$T> for $U { /* reverse */ }
    };
}

commutative_op!(impl Add for TextSize, u32 by fn add = +);
// Generates both TextSize + u32 and u32 + TextSize
```

**Macro Expansion Example:**
```rust
// Input:
ops!(impl Add for TextSize by fn add = +);

// Expands to:
impl Add<TextSize> for TextSize {
    type Output = TextSize;
    fn add(self, other: TextSize) -> TextSize {
        TextSize { raw: self.raw + other.raw }
    }
}
impl Add<&TextSize> for TextSize {
    type Output = TextSize;
    fn add(self, other: &TextSize) -> TextSize {
        self + *other
    }
}
impl<T> Add<T> for &TextSize
where TextSize: Add<T, Output=TextSize>
{
    type Output = TextSize;
    fn add(self, other: T) -> TextSize {
        *self + other
    }
}
```

---

## Pattern 20: LSP Server Main Loop Example
**File:** lib/lsp-server/examples/minimal_lsp.rs
**Category:** LSP Server Architecture, Example Code
**Code Example:**
```rust
fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_thread) = Connection::stdio();

    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions::default()),
        definition_provider: Some(OneOf::Left(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
    };

    let init_params = connection.initialize(serde_json::json!({
        "capabilities": caps,
        "offsetEncoding": ["utf-8"],
    }))?;

    main_loop(connection, init_params)?;
    io_thread.join()?;
    Ok(())
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _init: InitializeParams = serde_json::from_value(params)?;
    let mut docs: FxHashMap<Url, String> = FxHashMap::default();

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                handle_request(&connection, &req, &mut docs)?;
            }
            Message::Notification(note) => {
                handle_notification(&connection, &note, &mut docs)?;
            }
            Message::Response(resp) => {
                // Handle server-to-client request responses
            }
        }
    }
    Ok(())
}

fn handle_request(
    conn: &Connection,
    req: &ServerRequest,
    docs: &mut FxHashMap<Url, String>,
) -> Result<()> {
    match req.method.as_str() {
        GotoDefinition::METHOD => {
            send_ok(conn, req.id.clone(), &lsp_types::GotoDefinitionResponse::Array(Vec::new()))?;
        }
        Completion::METHOD => {
            let item = CompletionItem {
                label: "HelloFromLSP".into(),
                kind: Some(CompletionItemKind::FUNCTION),
                ..Default::default()
            };
            send_ok(conn, req.id.clone(), &CompletionResponse::Array(vec![item]))?;
        }
        _ => send_err(conn, req.id.clone(), ErrorCode::MethodNotFound, "unhandled")?,
    }
    Ok(())
}

fn send_ok<T: serde::Serialize>(conn: &Connection, id: RequestId, result: &T) -> Result<()> {
    let resp = Response { id, result: Some(serde_json::to_value(result)?), error: None };
    conn.sender.send(Message::Response(resp))?;
    Ok(())
}
```
**Why This Matters for Contributors:**
This example demonstrates the canonical LSP server structure: initialize capabilities, run main loop receiving messages, dispatch based on message type (Request/Notification/Response), handle shutdown gracefully. The document cache (FxHashMap<Url, String>) is a common pattern for tracking file state. Using METHOD constants from lsp-types ensures type-safe dispatch. The send_ok/send_err helpers abstract response construction. This pattern provides a template for building LSP servers, showing how the lsp-server library is meant to be used.

---

### Expert Rust Commentary: Pattern 20

**Idiomatic Rating:** ⭐⭐⭐⭐⭐ (5/5 - Production Template)

**Pattern Classification:** L3 External + LSP Server Architecture
- **Primary Category:** Complete LSP server implementation template
- **Related Patterns:** Main loop (5.1-5.5), LSP protocol (patterns 4-9), error handling (2.1-2.10)

**Rust-Specific Insight:**
This example is the Rosetta Stone for building LSP servers in Rust. It demonstrates the canonical structure:

**Initialization Phase:**
```rust
let (connection, io_thread) = Connection::stdio();
let caps = ServerCapabilities { /* ... */ };
let init_params = connection.initialize(caps)?;
```

The server declares its capabilities upfront (what features it supports). This is critical for protocol negotiation.

**Main Loop Pattern:**
```rust
for msg in &connection.receiver {
    match msg {
        Message::Request(req) => { /* handle or delegate */ }
        Message::Notification(note) => { /* handle side effects */ }
        Message::Response(resp) => { /* correlate with outgoing */ }
    }
}
```

This is a blocking, message-driven architecture. Each iteration processes one message. The pattern is synchronous but can spawn async tasks internally.

**Shutdown Protocol:**
```rust
if connection.handle_shutdown(&req)? {
    break;  // Exit main loop
}
```

LSP requires graceful shutdown via "shutdown" request followed by "exit" notification. The library handles this boilerplate.

**Document State Management:**
```rust
let mut docs: FxHashMap<Url, String> = FxHashMap::default();

// On didOpen/didChange:
docs.insert(url, content);

// On requests:
let doc = docs.get(&url)?;
```

The server maintains a cache of opened documents. FxHashMap is a faster HashMap variant (no cryptographic hashing needed for Url keys).

**Request Dispatch:**
```rust
match req.method.as_str() {
    GotoDefinition::METHOD => { /* ... */ }
    Completion::METHOD => { /* ... */ }
    _ => send_err(conn, req.id, ErrorCode::MethodNotFound, "unhandled")?,
}
```

Using METHOD constants from lsp-types ensures type safety and prevents typos.

**Contribution Tip:**
When building LSP servers:
1. **Copy this template** - it's the canonical structure
2. **Add state as needed** - extend the main_loop signature with your state
3. **Use lsp-types constants** - METHOD/ErrorCode for type safety
4. **Handle notifications** - didOpen/didChange update document cache
5. **Graceful shutdown** - always handle shutdown request

For production servers, add:
```rust
struct ServerState {
    docs: FxHashMap<Url, String>,
    config: Config,
    diagnostics: DiagnosticEngine,
    // ... domain-specific state
}

fn main_loop(
    connection: Connection,
    state: &mut ServerState,
) -> Result<()> {
    // ...
}
```

**Common Pitfalls:**
- **Forgetting shutdown:** Not handling shutdown request blocks editor exit
- **Blocking operations:** Long computations block message loop - use threading
- **Missing capabilities:** Declaring capabilities you don't implement confuses clients
- **Document sync:** Not updating cache on didChange causes stale data

**Related Patterns in Ecosystem:**
- **tower-lsp**: Async LSP framework (tokio-based)
- **lsp-server**: This library (sync, minimal)
- **rust-analyzer**: Production LSP server (complex state machine)
- **texlab**: LaTeX LSP server (good reference implementation)

**Advanced Pattern - Async Request Handling:**
```rust
use crossbeam_channel::Sender;
use std::thread;

fn handle_request_async(
    conn: &Connection,
    req: Request,
    state: Arc<Mutex<ServerState>>,
) -> Result<()> {
    let sender = conn.sender.clone();
    let id = req.id.clone();

    thread::spawn(move || {
        let result = compute_result(&req, &state);
        let resp = Response::new_ok(id, result);
        sender.send(Message::Response(resp)).ok();
    });

    Ok(())
}
```

**Production Checklist:**
- [ ] Initialize capabilities match implementation
- [ ] Shutdown/exit protocol handled
- [ ] Document sync (didOpen/didChange/didClose)
- [ ] Error handling (invalid requests, missing documents)
- [ ] Logging/tracing (for debugging)
- [ ] Configuration support (workspace/didChangeConfiguration)
- [ ] Progress reporting (for long operations)
- [ ] Cancellation ($/cancelRequest)
- [ ] Memory management (don't leak documents)
- [ ] Performance (diagnostics on background thread)

---

## Expert Summary: rust-analyzer Library Patterns

### Pattern Excellence Overview

**Category Breakdown:**
- **Arena & Memory Management (Patterns 1-3):** ⭐⭐⭐⭐⭐ Exemplary type-safe arena design with PhantomData mastery
- **LSP Protocol (Patterns 4-9):** ⭐⭐⭐⭐⭐ Production-grade protocol implementation, state machines, and concurrency
- **SIMD Optimization (Pattern 10):** ⭐⭐⭐⭐⭐ Platform-specific performance engineering
- **Encoding Handling (Pattern 11):** ⭐⭐⭐⭐⭐ Critical multi-encoding support for editor interop
- **Type Safety (Patterns 12-13):** ⭐⭐⭐⭐⭐ Newtype and const correctness excellence
- **Memory Optimization (Patterns 14-15):** ⭐⭐⭐⭐⭐ Small string optimization masterclass
- **DSL Design (Patterns 16-17):** ⭐⭐⭐⭐ Pragmatic DSL and parser implementation
- **API Design (Patterns 18-19):** ⭐⭐⭐⭐⭐ Sealed traits and macro-based ergonomics
- **Integration (Pattern 20):** ⭐⭐⭐⭐⭐ Production-ready LSP server template

**Overall Rating: 4.9/5.0** - This is production-grade Rust at its finest

### Key Learnings for Contributors

**1. Memory Efficiency is Paramount:**
- u32 indices save 50% memory vs pointers on 64-bit
- SmolStr with 23-byte inline saves millions of allocations
- Vec<Option<V>> vs HashMap tradeoffs are measurable

**2. Type Safety Through PhantomData:**
- `PhantomData<fn() -> T>` for covariance without ownership
- Enables Copy types with generic parameters
- Zero runtime cost for compile-time safety

**3. LSP Protocol Compliance:**
- Multi-encoding support is non-negotiable (UTF-8/16/32)
- State machines for initialization handshake
- Three-thread I/O architecture for backpressure

**4. Platform-Specific Optimization:**
- SIMD for hot paths (8-16x speedup for ASCII)
- Runtime feature detection for portability
- Always provide scalar fallback

**5. Sealed Traits for API Evolution:**
- Prevent external implementations
- Enable adding methods without breaking changes
- Document the sealing decision

### Contribution Readiness Checklist

**For Arena/Memory Patterns (1-3, 14-15):**
- [ ] Understand PhantomData variance implications
- [ ] Profile memory layout with `std::mem::size_of`
- [ ] Test with realistic data sizes (1M+ entries)
- [ ] Consider Option<T> niche optimization
- [ ] Document memory/performance tradeoffs

**For LSP Patterns (4-9, 20):**
- [ ] Read LSP specification (protocol state machines)
- [ ] Understand channel-based concurrency
- [ ] Test with VSCode, Neovim, Emacs clients
- [ ] Handle all encoding variants (UTF-8/16/32)
- [ ] Implement graceful shutdown protocol

**For SIMD/Performance Patterns (10):**
- [ ] Learn x86/ARM intrinsics basics
- [ ] Benchmark against scalar baseline
- [ ] Test on multiple CPU generations
- [ ] Provide portable fallback path
- [ ] Document SAFETY invariants thoroughly

**For Parser/DSL Patterns (16-17):**
- [ ] Understand recursive descent parsing
- [ ] Handle forward references with interning
- [ ] Test with edge cases (empty input, deeply nested)
- [ ] Provide helpful error messages
- [ ] Document grammar formally (EBNF)

**For Macro Patterns (19):**
- [ ] Test macro expansion (`cargo expand`)
- [ ] Document macro invocation examples
- [ ] Handle error cases gracefully
- [ ] Consider macro_rules! vs proc-macro tradeoffs
- [ ] Provide clear hygiene documentation

### Next Steps for Deep Contribution

**Immediate Actions:**
1. Clone rust-analyzer repository
2. Run `cargo test` in lib/ subdirectories
3. Read ARCHITECTURE.md for system overview
4. Pick a "good first issue" labeled with `E-easy`
5. Study existing PRs for code review standards

**Learning Path:**
1. Week 1: Study la-arena implementation (patterns 1-3)
2. Week 2: Build minimal LSP server using lsp-server (pattern 20)
3. Week 3: Implement UTF-16 offset conversion (pattern 11)
4. Week 4: Optimize with SIMD for a hot path (pattern 10)

**Contribution Areas:**
- **Arena optimizations:** Packed arena variants, memory pooling
- **LSP extensions:** New protocol features (3.17 callHierarchy, etc.)
- **SIMD ports:** AVX2, AVX-512, WASM SIMD, RISC-V Vector
- **Documentation:** Add more examples, architectural diagrams
- **Testing:** Property-based tests, fuzzing, stress tests

**Community Resources:**
- rust-analyzer Zulip: https://rust-lang.zulipchat.com/#narrow/stream/185405-t-compiler.2Frust-analyzer
- Weekly sync meetings: Check calendar in README
- Architecture docs: `docs/dev/` in repository
- RFCs for major features: `rust-lang/rfcs`

### Final Assessment

These patterns represent **production Rust engineering at the highest level**. The codebase demonstrates:
- Zero-cost abstractions (PhantomData, const fn)
- Memory efficiency (u32 indices, SmolStr SSO)
- Platform-specific optimization (SIMD)
- Protocol compliance (LSP state machines)
- API stability (sealed traits)

**Contributors should expect:**
- High code review standards
- Performance regression testing
- Multi-platform compatibility requirements
- Comprehensive test coverage expectations
- Detailed commit messages with rationale

**Reward for contributors:**
- World-class Rust engineering experience
- Impact on thousands of Rust developers daily
- Deep understanding of compiler infrastructure
- Networking with Rust core team members
- Portfolio-quality open source contributions

This is the **gold standard** for Rust library design. Study these patterns, internalize them, and apply them to your own projects. The rust-analyzer team has created a masterclass in production Rust engineering.

---
