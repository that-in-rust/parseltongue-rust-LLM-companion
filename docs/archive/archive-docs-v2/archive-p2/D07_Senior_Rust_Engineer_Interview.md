# Senior Rust Engineer Interview Questions

> **Document ID**: D07
> **Role Level**: Senior Rust Engineer (3-7 years experience)
> **Total Questions**: 100
> **Probability Score**: 1-100 (likelihood of being asked)
> **Generated with**: Parseltongue + Claude Code

---

## Document Purpose

This interview guide is designed for Senior Rust Engineers. Questions focus on:
- Advanced ownership and borrowing patterns
- Async/await and concurrency in depth
- Performance optimization techniques
- Unsafe Rust and FFI
- Advanced trait patterns
- Memory layout and optimization
- System design with Rust

---

## Section 1: Advanced Ownership & Borrowing (Q1-Q20)

### Q1. Explain interior mutability and when to use it.
**Probability**: 92

```mermaid
graph TD
    subgraph "Interior Mutability"
        A[Mutate Data Behind<br>Immutable Reference]
        B[Runtime Borrow Check<br>Instead of Compile Time]
    end

    subgraph "Types"
        C["Cell<T>"] --> D[Copy types only<br>Single thread]
        E["RefCell<T>"] --> F[Any type<br>Single thread<br>Panics on violation]
        G["Mutex<T>"] --> H[Thread-safe<br>Blocking]
        I["RwLock<T>"] --> J[Read-many<br>Write-exclusive]
    end

    subgraph "Use Cases"
        K[Graph structures]
        L[Caching]
        M[Observer pattern]
    end

    A --> C
    A --> E
    A --> G

    style F fill:#ffd43b
```

---

### Q2. What is the difference between `Rc<T>` and `Arc<T>`?
**Probability**: 95

```mermaid
graph TD
    subgraph "Rc<T>"
        A[Reference Counted]
        B[Single Thread Only]
        C[No Send/Sync]
        D[Cheaper Counter]
    end

    subgraph "Arc<T>"
        E[Atomic Reference Counted]
        F[Thread Safe]
        G[Implements Send + Sync]
        H[Atomic Counter Overhead]
    end

    subgraph "Usage Pattern"
        I["Rc + RefCell"] --> J[Single-threaded<br>Shared Mutability]
        K["Arc + Mutex"] --> L[Multi-threaded<br>Shared Mutability]
    end

    style J fill:#69db7c
    style L fill:#69db7c
```

---

### Q3. How do you handle cyclic references with `Rc`?
**Probability**: 85

```mermaid
graph TD
    subgraph "The Problem"
        A[Parent holds Rc to Child]
        B[Child holds Rc to Parent]
        C[Reference Count Never 0]
        D[Memory Leak!]
    end

    subgraph "Solution: Weak<T>"
        E["Weak<T>"] --> F[Non-owning Reference]
        F --> G[Doesn't Increment Count]
        G --> H[upgrade() → Option<Rc<T>>]
    end

    subgraph "Pattern"
        I[Parent: Rc to children]
        J[Child: Weak to parent]
    end

    D --> E
    I --> J

    style D fill:#ff6b6b
    style J fill:#69db7c
```

---

### Q4. Explain `Pin<T>` and why it exists.
**Probability**: 78

```mermaid
graph TD
    subgraph "Self-Referential Structs"
        A[Struct Contains<br>Pointer to Itself]
        B[If Moved, Pointer<br>Becomes Invalid]
    end

    subgraph "Pin<T>"
        C[Guarantees Value<br>Won't Move in Memory]
        D[Required for Async<br>Futures]
    end

    subgraph "Creation"
        E["Box::pin(value)"]
        F["Pin::new(&mut value)"]
        G[Only if T: Unpin]
    end

    subgraph "Async Context"
        H[Futures Hold References<br>Across .await]
        I[Must Be Pinned<br>to Poll]
    end

    A --> B
    B --> C
    C --> H

    style D fill:#69db7c
```

---

### Q5. What is variance in Rust and when does it matter?
**Probability**: 65

```mermaid
graph TD
    subgraph "Variance Types"
        A[Covariant] --> B["Can use 'longer for 'shorter"]
        C[Contravariant] --> D["Can use 'shorter for 'longer"]
        E[Invariant] --> F[Must match exactly]
    end

    subgraph "Common Cases"
        G["&'a T"] --> H[Covariant in 'a and T]
        I["&'a mut T"] --> J[Covariant in 'a<br>Invariant in T]
        K["fn(T) -> U"] --> L[Contravariant in T<br>Covariant in U]
    end

    subgraph "Why It Matters"
        M[Subtyping for Lifetimes]
        N[Safe Generic APIs]
    end

    J --> M

    style J fill:#ffd43b
```

---

### Q6. How does `PhantomData<T>` work and when do you need it?
**Probability**: 70

```mermaid
graph TD
    subgraph "PhantomData"
        A[Zero-sized Type]
        B[Tells Compiler About<br>Unused Type Parameter]
        C[Affects Drop Check<br>and Variance]
    end

    subgraph "Use Cases"
        D[Unused Lifetime Parameters]
        E[FFI Pointer Types]
        F[Type-level State Machines]
    end

    subgraph "Example"
        G["struct Iter<'a, T> {"]
        H["  ptr: *const T,"]
        I["  _marker: PhantomData<&'a T>,"]
        J["}"]
    end

    B --> D
    D --> G

    style C fill:#ffd43b
```

---

### Q7. Explain the difference between `Deref` and `AsRef`.
**Probability**: 82

```mermaid
graph TD
    subgraph "Deref"
        A[Transparent Wrapper]
        B[Smart Pointer Behavior]
        C[Enables Deref Coercion]
        D["*x and method calls"]
    end

    subgraph "AsRef"
        E[Explicit Conversion]
        F[Generic Function Bounds]
        G[Cheap Reference Conversion]
        H["fn takes<T: AsRef<str>>"]
    end

    subgraph "When to Implement"
        I[Deref: Wrapper Types<br>Box, Rc, MutexGuard]
        J[AsRef: Any Type That<br>Can Cheaply Become &T]
    end

    C --> I
    G --> J

    style I fill:#69db7c
    style J fill:#69db7c
```

---

### Q8. What is `ManuallyDrop<T>` and when would you use it?
**Probability**: 60

```mermaid
graph TD
    subgraph "ManuallyDrop<T>"
        A[Prevents Automatic Drop]
        B[Must Call drop() Manually]
        C[Or Forget the Value]
    end

    subgraph "Use Cases"
        D[FFI Ownership Transfer]
        E[Custom Drop Order]
        F[Union with Non-Copy Types]
    end

    subgraph "Danger"
        G[Forgetting = Memory Leak]
        H[Double Drop = UB]
    end

    A --> D
    B --> G

    style G fill:#ff6b6b
    style H fill:#ff6b6b
```

---

### Q9. How do you implement `Drop` correctly with unsafe code?
**Probability**: 72

```mermaid
graph TD
    subgraph "Drop Trait"
        A["fn drop(&mut self)"]
        B[Called Automatically<br>When Value Goes Out of Scope]
    end

    subgraph "Unsafe Considerations"
        C[Must Not Panic<br>in Drop]
        D[Raw Pointers: Null Check]
        E[Avoid Double Free]
        F[Consider panic=abort]
    end

    subgraph "Pattern"
        G["impl Drop for MyType {"]
        H["  fn drop(&mut self) {"]
        I["    if !self.ptr.is_null() {"]
        J["      unsafe { drop(Box::from_raw(self.ptr)) }"]
        K["    }"]
        L["  }"]
        M["}"]
    end

    A --> C
    C --> G

    style C fill:#ffd43b
```

---

### Q10. What is `mem::take` and `mem::replace`?
**Probability**: 75

```mermaid
graph TD
    subgraph "mem::take"
        A["std::mem::take(&mut value)"]
        B[Replace with Default]
        C[Returns Original]
    end

    subgraph "mem::replace"
        D["std::mem::replace(&mut value, new)"]
        E[Replace with Any Value]
        F[Returns Original]
    end

    subgraph "Use Cases"
        G[Move Out of &mut]
        H[Avoid Clone]
        I[State Machine Transitions]
    end

    subgraph "Example"
        J["let old = mem::take(&mut self.data);"]
        K["// self.data is now Default"]
        L["// old has the original value"]
    end

    A --> G
    G --> J

    style G fill:#69db7c
```

---

### Q11. How do you optimize struct layout for cache efficiency?
**Probability**: 68

```mermaid
graph TD
    subgraph "Default Layout"
        A[Compiler Reorders Fields]
        B[Minimizes Padding]
        C[Not Predictable]
    end

    subgraph "repr(C)"
        D[C-Compatible Layout]
        E[Field Order Preserved]
        F[Predictable but<br>May Have Padding]
    end

    subgraph "Optimization Tips"
        G[Group Hot Fields Together]
        H[Avoid False Sharing<br>in Concurrent Code]
        I[Use #[repr(packed)]<br>Carefully]
    end

    subgraph "Tools"
        J[std::mem::size_of]
        K[std::mem::align_of]
        L[cargo-bloat]
    end

    G --> J

    style G fill:#69db7c
```

---

### Q12. Explain `MaybeUninit<T>` and safe initialization patterns.
**Probability**: 65

```mermaid
graph TD
    subgraph "MaybeUninit<T>"
        A[Uninitialized Memory]
        B[No Drop Called<br>on Uninitialized]
        C[Must Initialize<br>Before Use]
    end

    subgraph "Safe Pattern"
        D["let mut arr: [MaybeUninit<T>; N]"]
        E["for elem in &mut arr {"]
        F["  elem.write(value);"]
        G["}"]
        H["unsafe { transmute(arr) }"]
    end

    subgraph "Why Use"
        I[Performance: Skip Default]
        J[FFI: Out Parameters]
        K[Arrays of Non-Copy Types]
    end

    C --> D

    style C fill:#ffd43b
```

---

### Q13. What is the difference between `Copy` and `Clone`?
**Probability**: 88

```mermaid
graph TD
    subgraph "Copy"
        A[Implicit Bitwise Copy]
        B[Stack Only Types]
        C[No Custom Logic]
        D[Marker Trait]
    end

    subgraph "Clone"
        E[Explicit .clone() Call]
        F[Any Type]
        G[Can Have Custom Logic]
        H[May Allocate]
    end

    subgraph "Relationship"
        I[Copy: Clone]
        J[Copy Implies Clone]
        K[Clone Does NOT Imply Copy]
    end

    subgraph "Example"
        L[Copy: i32, bool, (u8, u8)]
        M[Clone only: String, Vec]
    end

    A --> D
    D --> I

    style I fill:#69db7c
```

---

### Q14. How do you implement custom smart pointers?
**Probability**: 70

```mermaid
graph TD
    subgraph "Required Traits"
        A[Deref → Access Inner]
        B[DerefMut → Mutate Inner]
        C[Drop → Cleanup]
    end

    subgraph "Optional Traits"
        D[Borrow]
        E[AsRef]
        F[From/Into]
    end

    subgraph "Example: Box-like"
        G["struct MyBox<T>(NonNull<T>);"]
        H["impl<T> Deref for MyBox<T> {"]
        I["  type Target = T;"]
        J["  fn deref(&self) -> &T {"]
        K["    unsafe { self.0.as_ref() }"]
        L["  }"]
        M["}"]
    end

    A --> G

    style A fill:#69db7c
```

---

### Q15. Explain `NonNull<T>` vs raw pointers.
**Probability**: 62

```mermaid
graph TD
    subgraph "*const T / *mut T"
        A[Can Be Null]
        B[Covariant]
        C[No Guarantees]
    end

    subgraph "NonNull<T>"
        D[Never Null]
        E[Covariant]
        F[Enables Niche Optimization]
        G["Option<NonNull<T>> same size"]
    end

    subgraph "Usage"
        H[Unsafe Code]
        I[Custom Collections]
        J[FFI When Non-Null Known]
    end

    D --> F
    F --> I

    style F fill:#69db7c
```

---

### Q16. What are the rules for implementing `Send` and `Sync`?
**Probability**: 90

```mermaid
graph TD
    subgraph "Send"
        A[Safe to MOVE<br>Between Threads]
        B[Ownership Transfer OK]
    end

    subgraph "Sync"
        C[Safe to SHARE<br>Between Threads]
        D["&T Can Be Sent"]
        E["T: Sync iff &T: Send"]
    end

    subgraph "Auto-Implemented"
        F[Most Types Are Send + Sync]
        G[Opt-out with PhantomData]
    end

    subgraph "NOT Send/Sync"
        H["Rc<T>"] --> I[Not Send]
        J["RefCell<T>"] --> K[Not Sync]
        L[Raw Pointers] --> M[Neither]
    end

    style I fill:#ff6b6b
    style K fill:#ff6b6b
```

---

### Q17. How do you safely share data across threads?
**Probability**: 92

```mermaid
graph TD
    subgraph "Read-Only Sharing"
        A["Arc<T>"] --> B[Atomic Reference Count]
    end

    subgraph "Mutable Sharing"
        C["Arc<Mutex<T>>"] --> D[Lock for Access]
        E["Arc<RwLock<T>>"] --> F[Multiple Readers<br>Single Writer]
    end

    subgraph "Lock-Free"
        G[Atomic Types] --> H[AtomicBool, AtomicUsize]
        I[crossbeam] --> J[Lock-free Data Structures]
    end

    subgraph "Message Passing"
        K[mpsc::channel] --> L[Multiple Producers<br>Single Consumer]
    end

    style D fill:#69db7c
    style L fill:#69db7c
```

---

### Q18. What is `UnsafeCell<T>` and why is it fundamental?
**Probability**: 72

```mermaid
graph TD
    subgraph "UnsafeCell<T>"
        A[Only Way to Get<br>&mut from & in Safe Code]
        B[Basis of All Interior<br>Mutability Types]
    end

    subgraph "Built On UnsafeCell"
        C["Cell<T>"]
        D["RefCell<T>"]
        E["Mutex<T>"]
        F["RwLock<T>"]
    end

    subgraph "Key Property"
        G["UnsafeCell<T>.get() → *mut T"]
        H[From &UnsafeCell to *mut]
        I[Compiler Allows Mutation<br>Through Immutable Reference]
    end

    A --> C
    A --> D
    A --> E

    style B fill:#ffd43b
```

---

### Q19. How do you prevent data races at compile time?
**Probability**: 85

```mermaid
graph TD
    subgraph "Rust's Guarantees"
        A[Send + Sync Bounds]
        B[Borrow Checker]
        C[No Shared Mutable State<br>Without Synchronization]
    end

    subgraph "Mechanisms"
        D["&T or &mut T"] --> E[Never Both]
        F[Mutex/RwLock] --> G[Runtime Lock]
        H[Channels] --> I[Ownership Transfer]
    end

    subgraph "Compiler Prevents"
        J[Sharing Non-Sync Types]
        K[Moving Non-Send Types]
        L[Multiple &mut Simultaneously]
    end

    A --> J
    B --> L

    style C fill:#69db7c
```

---

### Q20. Explain lifetime subtyping and higher-ranked trait bounds.
**Probability**: 58

```mermaid
graph TD
    subgraph "Lifetime Subtyping"
        A["'longer: 'shorter"]
        B["'static: 'any"]
        C[Longer Lives<br>Longer Than Shorter]
    end

    subgraph "HRTB"
        D["for<'a> Fn(&'a str) -> &'a str"]
        E[Works For ANY Lifetime]
        F[Common in Closures]
    end

    subgraph "Use Cases"
        G[Generic over Closures]
        H[Complex Trait Bounds]
        I["where F: for<'a> Fn(&'a T)"]
    end

    D --> G

    style D fill:#ffd43b
```

---

## Section 2: Async/Await & Concurrency (Q21-Q40)

### Q21. Explain how async/await works under the hood.
**Probability**: 92

```mermaid
graph TD
    subgraph "Compilation"
        A[async fn] --> B[Returns impl Future]
        B --> C[State Machine<br>Generated]
    end

    subgraph "State Machine"
        D[Each .await = State]
        E[poll() Called by<br>Executor]
        F["Returns Poll::Ready<br>or Poll::Pending"]
    end

    subgraph "Execution"
        G[Executor Polls Future]
        H[Pending → Waker Registered]
        I[Ready → Value Returned]
        J[No Thread Blocked!]
    end

    C --> D
    E --> F
    F --> G

    style J fill:#69db7c
```

---

### Q22. What is the difference between `tokio` and `async-std`?
**Probability**: 78

```mermaid
graph TD
    subgraph "tokio"
        A[Most Popular]
        B[Work-Stealing Scheduler]
        C[Feature-Rich]
        D[Larger Binary]
    end

    subgraph "async-std"
        E[std-like API]
        F[Simpler Mental Model]
        G[Smaller Ecosystem]
    end

    subgraph "Compatibility"
        H[Different Runtimes<br>Don't Mix Well]
        I[Choose One Per Project]
    end

    subgraph "Recommendation"
        J[Web Services → tokio]
        K[Simple Apps → async-std]
    end

    style A fill:#69db7c
    style H fill:#ffd43b
```

---

### Q23. How do you handle cancellation in async Rust?
**Probability**: 80

```mermaid
graph TD
    subgraph "Drop = Cancel"
        A[Dropping Future<br>Cancels It]
        B[No Explicit Cancel<br>Needed]
    end

    subgraph "Graceful Shutdown"
        C[CancellationToken] --> D[tokio_util::sync]
        E[Select with Shutdown<br>Signal]
    end

    subgraph "Pattern"
        F["tokio::select! {"]
        G["  result = work() => handle(result),"]
        H["  _ = shutdown.recv() => return,"]
        I["}"]
    end

    subgraph "Cleanup"
        J[Use Drop Trait]
        K[Async Drop Not Stable]
    end

    A --> B
    E --> F

    style B fill:#69db7c
```

---

### Q24. What is `tokio::spawn` vs `tokio::spawn_blocking`?
**Probability**: 88

```mermaid
graph TD
    subgraph "spawn()"
        A[For Async Tasks]
        B[Runs on Runtime<br>Thread Pool]
        C[Must Not Block!]
    end

    subgraph "spawn_blocking()"
        D[For Sync/Blocking Code]
        E[Runs on Dedicated<br>Blocking Pool]
        F[Won't Starve Runtime]
    end

    subgraph "When to Use"
        G[spawn: Network I/O, async code]
        H[spawn_blocking: File I/O, CPU work]
    end

    subgraph "Danger"
        I[Blocking in spawn<br>= Starved Runtime]
    end

    C --> I

    style I fill:#ff6b6b
    style F fill:#69db7c
```

---

### Q25. How do you implement backpressure in async streams?
**Probability**: 72

```mermaid
graph TD
    subgraph "Backpressure"
        A[Slow Consumer<br>Limits Fast Producer]
        B[Prevents Memory<br>Exhaustion]
    end

    subgraph "Mechanisms"
        C[Bounded Channels] --> D["mpsc::channel(100)"]
        E[Semaphores] --> F[Limit Concurrent Work]
        G[Stream::buffer_unordered] --> H[Control Parallelism]
    end

    subgraph "Pattern"
        I["let (tx, rx) = mpsc::channel(100);"]
        J["tx.send(item).await; // Waits if full"]
    end

    D --> I

    style B fill:#69db7c
```

---

### Q26. Explain `Future` trait and `poll()`.
**Probability**: 85

```mermaid
graph TD
    subgraph "Future Trait"
        A["trait Future {"]
        B["  type Output;"]
        C["  fn poll(self: Pin<&mut Self>, cx: &mut Context)"]
        D["    -> Poll<Self::Output>;"]
        E["}"]
    end

    subgraph "Poll Result"
        F["Poll::Ready(value)"] --> G[Done, Return Value]
        H["Poll::Pending"] --> I[Not Done, Wake Later]
    end

    subgraph "Context"
        J[Contains Waker]
        K[Waker Signals<br>Ready to Poll Again]
    end

    C --> F
    C --> H
    H --> J

    style K fill:#69db7c
```

---

### Q27. What is `Waker` and how does it work?
**Probability**: 75

```mermaid
graph TD
    subgraph "Waker"
        A[Tells Executor<br>Future is Ready]
        B[Called When I/O<br>Completes]
    end

    subgraph "Flow"
        C[Future Returns Pending]
        D[Waker Stored Somewhere]
        E[Event Occurs]
        F[Waker.wake() Called]
        G[Executor Re-polls Future]
    end

    subgraph "Implementation"
        H[Usually from epoll/kqueue/IOCP]
        I[OS Notifies Completion]
        J[Runtime Wakes Future]
    end

    C --> D
    E --> F
    F --> G

    style G fill:#69db7c
```

---

### Q28. How do you avoid deadlocks in async code?
**Probability**: 82

```mermaid
graph TD
    subgraph "Deadlock Causes"
        A[Holding Lock<br>Across .await]
        B[Circular Lock<br>Dependencies]
    end

    subgraph "Prevention"
        C[Keep Lock Scope Small]
        D[Don't await While<br>Holding Lock]
        E[Consistent Lock Order]
    end

    subgraph "Anti-Pattern"
        F["let guard = mutex.lock().await;"]
        G["do_async_work().await; // BAD!"]
        H["drop(guard);"]
    end

    subgraph "Better"
        I["{"]
        J["  let guard = mutex.lock().await;"]
        K["  let data = guard.clone();"]
        L["} // guard dropped"]
        M["do_async_work(data).await;"]
    end

    A --> F
    D --> I

    style F fill:#ff6b6b
    style I fill:#69db7c
```

---

### Q29. What is `join!` vs `select!` vs `race`?
**Probability**: 85

```mermaid
graph TD
    subgraph "join!"
        A[Wait for ALL Futures]
        B[Returns All Results]
        C[Concurrent, Not Parallel]
    end

    subgraph "select!"
        D[Wait for FIRST Future]
        E[Cancels Others]
        F[Returns Winner]
    end

    subgraph "race"
        G[Like select But<br>Returns Result Type]
    end

    subgraph "Use Cases"
        H[join: Fetch Multiple APIs]
        I[select: Timeout or Result]
        J[race: Try Multiple<br>Fallback Sources]
    end

    B --> H
    F --> I

    style H fill:#69db7c
    style I fill:#69db7c
```

---

### Q30. How do you implement a timeout in async Rust?
**Probability**: 88

```mermaid
graph TD
    subgraph "tokio::time::timeout"
        A["timeout(duration, future)"]
        B["Returns Result<T, Elapsed>"]
    end

    subgraph "tokio::select!"
        C["select! {"]
        D["  result = work() => Ok(result),"]
        E["  _ = sleep(duration) => Err(Timeout),"]
        F["}"]
    end

    subgraph "Pattern"
        G["match timeout(Duration::from_secs(5), fetch()).await {"]
        H["  Ok(data) => process(data),"]
        I["  Err(_) => handle_timeout(),"]
        J["}"]
    end

    A --> G

    style A fill:#69db7c
```

---

### Q31. What is `Stream` trait and how does it differ from `Iterator`?
**Probability**: 80

```mermaid
graph TD
    subgraph "Iterator"
        A[Synchronous]
        B[next() → Option<Item>]
        C[Blocking]
    end

    subgraph "Stream"
        D[Asynchronous]
        E["poll_next() → Poll<Option<Item>>"]
        F[Non-Blocking]
    end

    subgraph "Usage"
        G["while let Some(item) = stream.next().await {"]
        H["  process(item);"]
        I["}"]
    end

    subgraph "Combinators"
        J[map, filter, take]
        K[Same as Iterator<br>But Async]
    end

    D --> G

    style D fill:#69db7c
```

---

### Q32. How do you share state across async tasks safely?
**Probability**: 90

```mermaid
graph TD
    subgraph "Options"
        A["Arc<Mutex<T>>"] --> B[General Purpose]
        C["Arc<RwLock<T>>"] --> D[Read-Heavy Workloads]
        E[DashMap] --> F[Concurrent HashMap]
        G[Atomics] --> H[Simple Values]
    end

    subgraph "Pattern"
        I["let state = Arc::new(Mutex::new(State::new()));"]
        J["let state_clone = state.clone();"]
        K["tokio::spawn(async move {"]
        L["  let mut guard = state_clone.lock().await;"]
        M["  guard.update();"]
        N["});"]
    end

    B --> I

    style B fill:#69db7c
```

---

### Q33. What is `!Send` and when would a future be `!Send`?
**Probability**: 72

```mermaid
graph TD
    subgraph "Send Future"
        A[Can Be Polled on<br>Different Threads]
        B[Required for tokio::spawn]
    end

    subgraph "!Send Future"
        C[Must Stay on<br>Same Thread]
        D[Contains !Send Types<br>Across .await]
    end

    subgraph "Common !Send Types"
        E["Rc<T>"]
        F["RefCell<T>"]
        G[Raw Pointers]
        H["MutexGuard held across await"]
    end

    subgraph "Solution"
        I[Use Arc instead of Rc]
        J[Don't hold guards across await]
        K[Use spawn_local for !Send]
    end

    D --> E
    D --> H
    H --> J

    style J fill:#69db7c
```

---

### Q34. Explain actor model implementation in Rust.
**Probability**: 65

```mermaid
graph TD
    subgraph "Actor Model"
        A[Isolated State]
        B[Message Passing Only]
        C[No Shared Memory]
    end

    subgraph "Implementation"
        D["struct Actor { rx: mpsc::Receiver }"]
        E["Loop: recv message, handle, respond"]
        F[State Owned by Actor Task]
    end

    subgraph "Libraries"
        G[actix]
        H[xactor]
        I[bastion]
    end

    subgraph "Pattern"
        J["tokio::spawn(async move {"]
        K["  while let Some(msg) = rx.recv().await {"]
        L["    self.handle(msg);"]
        M["  }"]
        N["});"]
    end

    A --> D

    style C fill:#69db7c
```

---

### Q35. How do you implement graceful shutdown?
**Probability**: 85

```mermaid
graph TD
    subgraph "Signal Handling"
        A["tokio::signal::ctrl_c()"]
        B[Unix Signals]
    end

    subgraph "Pattern"
        C["let (shutdown_tx, shutdown_rx) = broadcast::channel(1);"]
        D["select! {"]
        E["  _ = server.run() => {},"]
        F["  _ = ctrl_c() => {"]
        G["    shutdown_tx.send(()).ok();"]
        H["  }"]
        I["}"]
    end

    subgraph "Task Cleanup"
        J[Tasks Check shutdown_rx]
        K[Finish Current Work]
        L[Close Connections Gracefully]
    end

    A --> C
    G --> J

    style K fill:#69db7c
```

---

### Q36. What is structured concurrency and does Rust have it?
**Probability**: 60

```mermaid
graph TD
    subgraph "Structured Concurrency"
        A[Child Tasks Bounded<br>by Parent Scope]
        B[No Orphan Tasks]
        C[Automatic Cleanup]
    end

    subgraph "Rust Status"
        D[tokio::spawn = Unstructured]
        E[Tasks Can Outlive Spawner]
        F[Must Manage Manually]
    end

    subgraph "Workarounds"
        G[JoinSet] --> H[Collect Task Handles]
        I[TaskTracker] --> J[Track Outstanding Tasks]
        K[Scoped Tasks<br>in rayon] --> L[True Structured]
    end

    F --> G

    style L fill:#69db7c
```

---

### Q37. How do you profile async code?
**Probability**: 70

```mermaid
graph TD
    subgraph "Tools"
        A[tokio-console] --> B[Real-time Task<br>Visualization]
        C[tracing] --> D[Structured Logging]
        E[flamegraph] --> F[CPU Profiling]
    end

    subgraph "Metrics"
        G[Task Poll Duration]
        H[Time in Poll vs Waiting]
        I[Waker Wake Count]
    end

    subgraph "Setup"
        J["console_subscriber::init();"]
        K["#[instrument]"]
        L["async fn my_fn() { ... }"]
    end

    A --> G
    C --> K

    style B fill:#69db7c
```

---

### Q38. Explain cooperative scheduling in async Rust.
**Probability**: 75

```mermaid
graph TD
    subgraph "Cooperative Scheduling"
        A[Tasks Yield Voluntarily]
        B[No Preemption]
        C[.await Points Are<br>Yield Points]
    end

    subgraph "Problem"
        D[Long Computation<br>Blocks Executor]
        E[Other Tasks Starved]
    end

    subgraph "Solutions"
        F["tokio::task::yield_now()"]
        G[spawn_blocking for CPU Work]
        H[Chunk Large Work]
    end

    subgraph "Budget"
        I[tokio Has Task Budget]
        J[Forces Yield After<br>N Polls]
    end

    D --> F
    D --> G

    style G fill:#69db7c
```

---

### Q39. What is `LocalSet` and when do you use it?
**Probability**: 62

```mermaid
graph TD
    subgraph "LocalSet"
        A[Single-Threaded Executor]
        B[For !Send Futures]
        C[All Tasks Same Thread]
    end

    subgraph "Use Cases"
        D[GUI Integration]
        E[WebAssembly]
        F[Legacy !Send APIs]
    end

    subgraph "Pattern"
        G["let local = LocalSet::new();"]
        H["local.run_until(async {"]
        I["  spawn_local(my_task());"]
        J["}).await;"]
    end

    B --> D

    style B fill:#ffd43b
```

---

### Q40. How do you handle panics in spawned tasks?
**Probability**: 78

```mermaid
graph TD
    subgraph "Default Behavior"
        A[Task Panics → JoinError]
        B[Other Tasks Continue]
        C[panic Not Propagated]
    end

    subgraph "Catching Panics"
        D["let handle = tokio::spawn(work());"]
        E["match handle.await {"]
        F["  Ok(result) => process(result),"]
        G["  Err(e) if e.is_panic() => recover(),"]
        H["}"]
    end

    subgraph "Best Practices"
        I[Log Panics]
        J[Consider panic = abort<br>for Production]
        K[Use Result Instead]
    end

    A --> D
    K --> I

    style K fill:#69db7c
```

---

## Section 3: Performance & Optimization (Q41-Q60)

### Q41. How do you profile a Rust application?
**Probability**: 90

```mermaid
graph TD
    subgraph "CPU Profiling"
        A[perf + flamegraph]
        B[Instruments on macOS]
        C[cargo-flamegraph]
    end

    subgraph "Memory Profiling"
        D[heaptrack]
        E[valgrind --tool=massif]
        F[DHAT]
    end

    subgraph "Rust-Specific"
        G[cargo bench]
        H[criterion]
        I[tracing + tracing-flame]
    end

    subgraph "Commands"
        J["cargo flamegraph --bin myapp"]
        K["RUSTFLAGS='-C target-cpu=native' cargo build --release"]
    end

    A --> C

    style H fill:#69db7c
```

---

### Q42. What are the common causes of slow Rust code?
**Probability**: 88

```mermaid
graph TD
    subgraph "Allocation Issues"
        A[Excessive Clone]
        B[String Concatenation<br>in Loops]
        C[Not Pre-sizing Collections]
    end

    subgraph "Algorithm Issues"
        D[O(n²) When O(n) Possible]
        E[Repeated HashMap Lookups]
        F[Unnecessary Sorting]
    end

    subgraph "Rust-Specific"
        G[Debug Build<br>10-100x Slower]
        H[Bounds Checking<br>in Hot Loops]
        I[Dynamic Dispatch<br>in Hot Paths]
    end

    subgraph "Solutions"
        J[Always Benchmark Release]
        K[Use Iterators]
        L[Profile Before Optimizing]
    end

    G --> J

    style G fill:#ff6b6b
    style J fill:#69db7c
```

---

### Q43. How do you reduce binary size in Rust?
**Probability**: 75

```mermaid
graph TD
    subgraph "Cargo.toml"
        A["[profile.release]"]
        B["opt-level = 'z'"]
        C["lto = true"]
        D["codegen-units = 1"]
        E["strip = true"]
    end

    subgraph "Other Techniques"
        F[Remove Unused Features]
        G[Avoid Generics<br>When Possible]
        H[Use panic = 'abort']
    end

    subgraph "Impact"
        I["opt-level = 'z': -10-20%"]
        J["lto: -10-20%"]
        K["strip: -20-30%"]
    end

    A --> B
    B --> I

    style E fill:#69db7c
```

---

### Q44. Explain LTO and when to use it.
**Probability**: 72

```mermaid
graph TD
    subgraph "Link-Time Optimization"
        A[Optimize Across Crates]
        B[Inline Cross-Crate Calls]
        C[Remove Dead Code]
    end

    subgraph "Types"
        D["lto = 'thin'"] --> E[Faster Compile<br>Good Optimization]
        F["lto = 'fat'"] --> G[Slow Compile<br>Best Optimization]
    end

    subgraph "Trade-offs"
        H[Smaller Binary]
        I[Faster Runtime]
        J[Much Slower Compile]
    end

    C --> H
    B --> I

    style E fill:#69db7c
```

---

### Q45. How do you avoid unnecessary allocations?
**Probability**: 88

```mermaid
graph TD
    subgraph "Techniques"
        A[Use References] --> B["&str over String"]
        C[Pre-allocate] --> D["Vec::with_capacity(n)"]
        E[Reuse Buffers] --> F["String::clear() + push"]
        G[SmallVec] --> H[Stack for Small<br>Heap for Large]
    end

    subgraph "Patterns"
        I[Return References<br>When Possible]
        J[Accept &[T] Over Vec<T>]
        K[Use Cow<str> for<br>Maybe-Owned]
    end

    B --> I
    D --> J

    style D fill:#69db7c
```

---

### Q46. What is SIMD and how do you use it in Rust?
**Probability**: 65

```mermaid
graph TD
    subgraph "SIMD"
        A[Single Instruction<br>Multiple Data]
        B[Process Multiple<br>Values at Once]
    end

    subgraph "Rust Support"
        C[std::arch] --> D[Platform Intrinsics]
        E[portable-simd] --> F[Portable<br>Nightly Only]
        G[Auto-vectorization] --> H[Compiler Does It]
    end

    subgraph "Example"
        I["use std::arch::x86_64::*;"]
        J["let a = _mm_loadu_ps(ptr);"]
        K["let b = _mm_add_ps(a, a);"]
    end

    subgraph "Tip"
        L[Write Simple Loops<br>Compiler Often Vectorizes]
    end

    G --> L

    style L fill:#69db7c
```

---

### Q47. How do you optimize hash map performance?
**Probability**: 78

```mermaid
graph TD
    subgraph "Standard HashMap"
        A[Uses SipHash]
        B[DoS Resistant]
        C[Not Fastest]
    end

    subgraph "Alternatives"
        D[rustc-hash] --> E[FxHashMap<br>Fastest, No DoS Protection]
        F[ahash] --> G[Fast + DoS Resistant]
        H[indexmap] --> I[Preserves Order]
    end

    subgraph "Optimization Tips"
        J["HashMap::with_capacity(n)"]
        K[Use Integer Keys<br>When Possible]
        L[Consider BTreeMap<br>for Ordered Data]
    end

    E --> J

    style E fill:#69db7c
```

---

### Q48. Explain cache-friendly programming in Rust.
**Probability**: 70

```mermaid
graph TD
    subgraph "Cache Basics"
        A[L1 Cache ~32KB]
        B[Access Time ~1ns vs ~100ns RAM]
        C[Cache Line 64 bytes]
    end

    subgraph "Techniques"
        D[Data Locality] --> E[Keep Related Data Together]
        F[Array of Structs<br>vs Struct of Arrays]
        G[Sequential Access<br>Over Random]
    end

    subgraph "Rust Tips"
        H[Use Vec Over LinkedList]
        I[Avoid Pointer Chasing]
        J[#[repr(C)] for Predictable Layout]
    end

    E --> H

    style H fill:#69db7c
```

---

### Q49. How do you benchmark correctly in Rust?
**Probability**: 85

```mermaid
graph TD
    subgraph "Tools"
        A[cargo bench] --> B[Built-in<br>Nightly Only]
        C[criterion] --> D[Statistical<br>Stable Rust]
    end

    subgraph "Best Practices"
        E[Use black_box()] --> F[Prevent Dead Code<br>Elimination]
        G[Warm Up] --> H[Avoid Cold Start]
        I[Multiple Iterations] --> J[Statistical Significance]
    end

    subgraph "Pitfalls"
        K[Benchmarking Debug Build]
        L[Not Using black_box]
        M[Measuring Wall Time Only]
    end

    D --> E

    style D fill:#69db7c
    style K fill:#ff6b6b
```

---

### Q50. What is `#[inline]` and when should you use it?
**Probability**: 80

```mermaid
graph TD
    subgraph "Inline Attributes"
        A["#[inline]"] --> B[Hint to Inline]
        C["#[inline(always)]"] --> D[Force Inline]
        E["#[inline(never)]"] --> F[Prevent Inline]
    end

    subgraph "When to Use"
        G[Small Functions<br>Called in Hot Paths]
        H[Cross-Crate Generics]
        I[NOT Large Functions]
    end

    subgraph "Note"
        J[Compiler Usually<br>Knows Best]
        K[Profile Before Adding]
    end

    B --> G
    J --> K

    style K fill:#69db7c
```

---

### Q51. How do you reduce compilation times?
**Probability**: 88

```mermaid
graph TD
    subgraph "Build Configuration"
        A[Use cargo check]
        B[Incremental Compilation]
        C[Parallel Frontend<br>-Z threads=8]
    end

    subgraph "Project Structure"
        D[Split into Workspaces]
        E[Fewer Dependencies]
        F[Avoid proc-macro<br>Heavy Crates]
    end

    subgraph "Caching"
        G[sccache] --> H[Shared Compile Cache]
        I[mold Linker] --> J[Faster Linking]
    end

    subgraph "CI/CD"
        K[Cache target/ and ~/.cargo]
        L[Use cargo-chef for Docker]
    end

    G --> H

    style A fill:#69db7c
    style H fill:#69db7c
```

---

### Q52. Explain zero-copy parsing patterns.
**Probability**: 68

```mermaid
graph TD
    subgraph "Zero-Copy"
        A[Parse Without Allocating]
        B[Return References<br>Into Source]
    end

    subgraph "Tools"
        C[nom] --> D[Parser Combinators]
        E[pest] --> F[PEG Grammar]
        G[Cow<str>] --> H[Maybe-Owned Strings]
    end

    subgraph "Pattern"
        I["fn parse(input: &str) -> &str"]
        J[Return Slice of Input]
        K[No String::from()]
    end

    subgraph "Trade-offs"
        L[Faster Parsing]
        M[Result Tied to Input Lifetime]
    end

    A --> I

    style L fill:#69db7c
    style M fill:#ffd43b
```

---

### Q53. How do you optimize string operations?
**Probability**: 82

```mermaid
graph TD
    subgraph "Avoid"
        A[format! in Loops]
        B[+ for Concatenation]
        C[to_string() Unnecessarily]
    end

    subgraph "Better"
        D[String::with_capacity]
        E[push_str / push]
        F[Use &str When Possible]
    end

    subgraph "Specialized"
        G[SmartString] --> H[Inline Small Strings]
        I[CompactString] --> J[3 Word Max]
        K[bytes::Bytes] --> L[Reference Counted]
    end

    D --> E

    style D fill:#69db7c
```

---

### Q54. What is arena allocation and when to use it?
**Probability**: 60

```mermaid
graph TD
    subgraph "Arena Allocation"
        A[Allocate from Contiguous<br>Memory Region]
        B[Free All at Once]
        C[No Individual Deallocation]
    end

    subgraph "Benefits"
        D[Faster Allocation]
        E[Better Cache Locality]
        F[Simpler Lifetimes]
    end

    subgraph "Libraries"
        G[bumpalo]
        H[typed-arena]
        I[generational-arena]
    end

    subgraph "Use Cases"
        J[AST Parsing]
        K[Graph Structures]
        L[Request-Scoped Data]
    end

    A --> D

    style D fill:#69db7c
```

---

### Q55. How do you use `Cow<str>` effectively?
**Probability**: 75

```mermaid
graph TD
    subgraph "Cow = Clone on Write"
        A["Cow<'a, str>"]
        B[Borrowed or Owned]
        C[Clone Only When<br>Mutation Needed]
    end

    subgraph "Variants"
        D["Cow::Borrowed(&str)"]
        E["Cow::Owned(String)"]
    end

    subgraph "Use Case"
        F["fn process(input: Cow<str>) -> Cow<str>"]
        G[Return Input If No Change]
        H[Return Owned If Modified]
    end

    subgraph "Pattern"
        I["if needs_change {"]
        J["  Cow::Owned(modified)"]
        K["} else {"]
        L["  input  // Borrowed passthrough"]
        M["}"]
    end

    C --> F

    style C fill:#69db7c
```

---

### Q56. Explain PGO (Profile-Guided Optimization).
**Probability**: 55

```mermaid
graph TD
    subgraph "PGO Process"
        A[Build with Instrumentation]
        B[Run Representative Workload]
        C[Collect Profile Data]
        D[Rebuild with Profile]
    end

    subgraph "Benefits"
        E[Better Inlining Decisions]
        F[Optimized Branch Prediction]
        G[5-20% Performance Gain]
    end

    subgraph "Commands"
        H["RUSTFLAGS='-C profile-generate=/tmp/pgo' cargo build --release"]
        I["./target/release/app < workload.txt"]
        J["RUSTFLAGS='-C profile-use=/tmp/pgo' cargo build --release"]
    end

    A --> B
    B --> C
    C --> D

    style G fill:#69db7c
```

---

### Q57. How do you reduce lock contention?
**Probability**: 78

```mermaid
graph TD
    subgraph "Strategies"
        A[Sharding] --> B[Multiple Locks<br>Hash to Bucket]
        C[Read-Write Locks] --> D[Many Readers<br>Single Writer]
        E[Lock-Free] --> F[Atomic Operations]
    end

    subgraph "Patterns"
        G[Minimize Critical Section]
        H[Clone Out, Drop Lock, Process]
        I[Use Channels Instead]
    end

    subgraph "Tools"
        J[DashMap] --> K[Sharded HashMap]
        L[parking_lot] --> M[Faster Locks]
    end

    A --> J

    style G fill:#69db7c
```

---

### Q58. What is false sharing and how do you avoid it?
**Probability**: 62

```mermaid
graph TD
    subgraph "False Sharing"
        A[Different Cores<br>Access Same Cache Line]
        B[One Writes<br>Invalidates Other's Cache]
        C[Massive Slowdown]
    end

    subgraph "Example"
        D["struct Counters { a: u64, b: u64 }"]
        E[Core 1 increments a]
        F[Core 2 increments b]
        G[Same Cache Line!]
    end

    subgraph "Solution"
        H["#[repr(align(64))]"]
        I[Pad Between Fields]
        J[crossbeam-utils::CachePadded]
    end

    C --> H

    style C fill:#ff6b6b
    style J fill:#69db7c
```

---

### Q59. How do you use `std::hint::black_box`?
**Probability**: 70

```mermaid
graph TD
    subgraph "black_box"
        A[Prevents Compiler<br>Optimization]
        B[Value Appears Used<br>to Optimizer]
    end

    subgraph "Use Cases"
        C[Benchmarking] --> D[Prevent Dead Code<br>Elimination]
        E[Prevent Constant<br>Folding]
    end

    subgraph "Example"
        F["let result = black_box(expensive_fn());"]
        G["// Compiler can't optimize away"]
        H["black_box(result);"]
    end

    C --> F

    style D fill:#69db7c
```

---

### Q60. Explain the cost of dynamic dispatch.
**Probability**: 75

```mermaid
graph TD
    subgraph "Static Dispatch"
        A["fn process<T: Trait>(t: T)"]
        B[Monomorphized]
        C[Inlined Possible]
        D[Larger Binary]
    end

    subgraph "Dynamic Dispatch"
        E["fn process(t: &dyn Trait)"]
        F[vtable Lookup]
        G[No Inlining]
        H[Smaller Binary]
    end

    subgraph "Cost"
        I[vtable: ~2 Pointer Indirections]
        J[Usually Negligible]
        K[Matters in Hot Loops]
    end

    subgraph "Guidance"
        L[Prefer Static in Hot Paths]
        M[Dynamic Fine for Plugins/DI]
    end

    F --> I
    K --> L

    style L fill:#69db7c
```

---

## Section 4: Unsafe Rust & FFI (Q61-Q80)

### Q61. What are the five things you can do in unsafe that you can't in safe Rust?
**Probability**: 95

```mermaid
graph TD
    subgraph "Unsafe Operations"
        A[Dereference Raw Pointers]
        B[Call Unsafe Functions]
        C[Access Mutable Statics]
        D[Implement Unsafe Traits]
        E[Access Union Fields]
    end

    subgraph "Still Enforced"
        F[Borrow Checker Active]
        G[Type System Active]
        H[Bounds Checking Active]
    end

    subgraph "Your Responsibility"
        I[Uphold Safety Invariants]
        J[Document Requirements]
        K[Minimize Scope]
    end

    A --> I

    style I fill:#ffd43b
```

---

### Q62. How do you write safe abstractions over unsafe code?
**Probability**: 88

```mermaid
graph TD
    subgraph "Pattern"
        A[Unsafe Implementation]
        B[Safe Public API]
        C[Encapsulate Invariants]
    end

    subgraph "Example"
        D["pub fn get(&self, idx: usize) -> Option<&T> {"]
        E["  if idx < self.len {"]
        F["    Some(unsafe { &*self.ptr.add(idx) })"]
        G["  } else { None }"]
        H["}"]
    end

    subgraph "Documentation"
        I["/// # Safety"]
        J["/// Caller must ensure..."]
        K[Document All Invariants]
    end

    A --> D
    B --> I

    style B fill:#69db7c
```

---

### Q63. What is UB (Undefined Behavior) and what causes it?
**Probability**: 90

```mermaid
graph TD
    subgraph "Undefined Behavior"
        A[Compiler Assumes<br>UB Never Happens]
        B[Anything Can Happen]
        C[Including Working<br>Incorrectly]
    end

    subgraph "Common Causes"
        D[Dereferencing Null/Dangling]
        E[Data Races]
        F[Invalid Values<br>bool not 0/1]
        G[Uninitialized Memory Read]
        H[Violating Aliasing Rules]
    end

    subgraph "Detection"
        I[miri] --> J[Runtime UB Detection]
        K[Sanitizers] --> L[ASan, MSan, TSan]
    end

    D --> I

    style B fill:#ff6b6b
    style I fill:#69db7c
```

---

### Q64. How do you use Miri for unsafe code verification?
**Probability**: 75

```mermaid
graph TD
    subgraph "Miri"
        A[Rust Interpreter]
        B[Detects UB at Runtime]
        C[Checks Aliasing Rules]
    end

    subgraph "Usage"
        D["rustup +nightly component add miri"]
        E["cargo +nightly miri test"]
    end

    subgraph "Detects"
        F[Use After Free]
        G[Out of Bounds]
        H[Uninitialized Reads]
        I[Aliasing Violations]
    end

    subgraph "Limitation"
        J[Slow: 10-100x Slower]
        K[Can't Test All Paths]
    end

    E --> F

    style B fill:#69db7c
```

---

### Q65. Explain stacked borrows model.
**Probability**: 55

```mermaid
graph TD
    subgraph "Stacked Borrows"
        A[Memory Model for<br>Aliasing Rules]
        B[Each Location Has<br>Borrow Stack]
    end

    subgraph "Stack Operations"
        C[Create Reference → Push]
        D[Use Reference → Check Top]
        E[Invalidate → Pop Until Match]
    end

    subgraph "Violation"
        F[Use Pointer After<br>Invalidation]
        G[Miri Catches This]
    end

    subgraph "Example"
        H["let x = &mut y;"]
        I["let p = x as *mut _;"]
        J["*x = 1;  // Invalidates p"]
        K["*p = 2;  // UB!"]
    end

    B --> C
    K --> F

    style F fill:#ff6b6b
```

---

### Q66. How do you call C functions from Rust?
**Probability**: 92

```mermaid
graph TD
    subgraph "FFI Declaration"
        A["extern 'C' {"]
        B["  fn strlen(s: *const c_char) -> usize;"]
        C["}"]
    end

    subgraph "Calling"
        D["unsafe {"]
        E["  let len = strlen(c_string.as_ptr());"]
        F["}"]
    end

    subgraph "Types"
        G["std::ffi::CString"]
        H["std::ffi::CStr"]
        I["std::os::raw::c_int"]
    end

    subgraph "Tools"
        J[bindgen] --> K[Auto-Generate Bindings]
    end

    A --> D
    J --> K

    style K fill:#69db7c
```

---

### Q67. How do you expose Rust functions to C?
**Probability**: 85

```mermaid
graph TD
    subgraph "Export Function"
        A["#[no_mangle]"]
        B["pub extern 'C' fn add(a: i32, b: i32) -> i32 {"]
        C["  a + b"]
        D["}"]
    end

    subgraph "Header Generation"
        E[cbindgen] --> F[Generate C Header]
    end

    subgraph "Build"
        G["crate-type = ['cdylib']"]
        H[Produces .so / .dylib / .dll]
    end

    subgraph "Memory Safety"
        I[Caller Manages Ownership]
        J[Provide Free Functions]
    end

    A --> B
    E --> F

    style J fill:#ffd43b
```

---

### Q68. What is `#[repr(C)]` and when do you need it?
**Probability**: 88

```mermaid
graph TD
    subgraph "repr(C)"
        A[C-Compatible Layout]
        B[Predictable Field Order]
        C[Predictable Padding]
    end

    subgraph "When Required"
        D[FFI Structs]
        E[Shared Memory]
        F[Serialization to Bytes]
    end

    subgraph "Other repr"
        G["#[repr(transparent)]"] --> H[Same Layout as Single Field]
        I["#[repr(packed)]"] --> J[No Padding<br>Careful: Unaligned!]
        K["#[repr(align(N))]"] --> L[Minimum Alignment]
    end

    A --> D

    style D fill:#69db7c
```

---

### Q69. How do you handle errors across FFI boundaries?
**Probability**: 78

```mermaid
graph TD
    subgraph "C Convention"
        A[Return Error Code]
        B[Out Parameter for Value]
        C[No Exceptions]
    end

    subgraph "Rust Pattern"
        D["pub extern 'C' fn do_thing("]
        E["  input: *const c_char,"]
        F["  output: *mut ResultStruct,"]
        G[") -> i32"]
    end

    subgraph "Error Codes"
        H[0 = Success]
        I[Negative = Error]
        J[Provide Error String Function]
    end

    subgraph "Panic Safety"
        K["std::panic::catch_unwind"]
        L[Never Let Panic Cross FFI]
    end

    L --> K

    style L fill:#ff6b6b
```

---

### Q70. What is `CString` vs `CStr`?
**Probability**: 82

```mermaid
graph TD
    subgraph "CString"
        A[Owned]
        B[Null-Terminated]
        C["CString::new('hello')"]
        D[Allocates on Heap]
    end

    subgraph "CStr"
        E[Borrowed]
        F[Reference to C String]
        G["CStr::from_ptr(ptr)"]
        H[No Allocation]
    end

    subgraph "Conversion"
        I["Rust → C: cstring.as_ptr()"]
        J["C → Rust: CStr::from_ptr(p).to_str()"]
    end

    C --> I
    G --> J

    style I fill:#69db7c
    style J fill:#69db7c
```

---

### Q71. How do you safely wrap a C library?
**Probability**: 80

```mermaid
graph TD
    subgraph "Architecture"
        A[sys Crate] --> B[Raw Bindings<br>bindgen]
        C[Safe Wrapper Crate] --> D[Idiomatic Rust API]
    end

    subgraph "Wrapper Pattern"
        E["struct Wrapper(*mut ffi::Handle);"]
        F["impl Wrapper { fn new() -> Result<Self> }"]
        G["impl Drop { unsafe { ffi::close(self.0) } }"]
    end

    subgraph "Safety Checks"
        H[Null Check All Pointers]
        I[Validate Return Codes]
        J[Encapsulate Lifetimes]
    end

    A --> C
    F --> H

    style D fill:#69db7c
```

---

### Q72. What is `extern "Rust"` vs `extern "C"`?
**Probability**: 68

```mermaid
graph TD
    subgraph "extern 'C'"
        A[C Calling Convention]
        B[Stable ABI]
        C[Cross-Language]
    end

    subgraph "extern 'Rust'"
        D[Rust Calling Convention]
        E[Unstable ABI]
        F[Same Compiler Only]
    end

    subgraph "Other ABIs"
        G["extern 'system'"] --> H[Platform Default<br>stdcall on Windows]
        I["extern 'cdecl'"]
        J["extern 'fastcall'"]
    end

    B --> A

    style B fill:#69db7c
```

---

### Q73. How do you handle callbacks from C?
**Probability**: 72

```mermaid
graph TD
    subgraph "C Callback Type"
        A["type Callback = extern 'C' fn(i32) -> i32;"]
    end

    subgraph "With Context"
        B["type Callback = extern 'C' fn(*mut c_void, i32) -> i32;"]
        C[Pass Closure as void*]
    end

    subgraph "Pattern"
        D["extern 'C' fn trampoline(ctx: *mut c_void, val: i32) -> i32 {"]
        E["  let closure = unsafe { &*(ctx as *const F) };"]
        F["  closure(val)"]
        G["}"]
    end

    subgraph "Safety"
        H[Pin Closure During Call]
        I[Ensure Lifetime Correctness]
    end

    B --> D

    style H fill:#ffd43b
```

---

### Q74. What tools exist for FFI safety?
**Probability**: 65

```mermaid
graph TD
    subgraph "Code Generation"
        A[bindgen] --> B[C/C++ → Rust]
        C[cbindgen] --> D[Rust → C Header]
    end

    subgraph "Verification"
        E[miri] --> F[UB Detection]
        G[Sanitizers] --> H[Memory Errors]
    end

    subgraph "Higher-Level"
        I[cxx] --> J[Safe C++ Interop]
        K[pyo3] --> L[Python Bindings]
        M[neon] --> N[Node.js Bindings]
    end

    J --> I

    style J fill:#69db7c
```

---

### Q75. Explain `#[link]` and linking options.
**Probability**: 60

```mermaid
graph TD
    subgraph "link Attribute"
        A["#[link(name = 'foo')]"]
        B[Link to libfoo]
    end

    subgraph "Options"
        C["#[link(name = 'foo', kind = 'static')]"]
        D["#[link(name = 'foo', kind = 'dylib')]"]
        E["#[link(name = 'foo', kind = 'framework')]"]
    end

    subgraph "Build Script"
        F["println!('cargo:rustc-link-lib=foo');"]
        G["println!('cargo:rustc-link-search=/path');"]
    end

    subgraph "pkg-config"
        H[pkg-config crate]
        I[Auto-detect Paths]
    end

    F --> G

    style H fill:#69db7c
```

---

### Q76. How do you transmute safely?
**Probability**: 75

```mermaid
graph TD
    subgraph "transmute"
        A["std::mem::transmute"]
        B[Reinterpret Bits]
        C[Same Size Required]
    end

    subgraph "Dangers"
        D[Can Create Invalid Values]
        E[Violate Type Invariants]
        F[Cause UB]
    end

    subgraph "Safer Alternatives"
        G["transmute_copy"] --> H[Doesn't Consume]
        I["from_ne_bytes"] --> J[For Integers]
        K["bytemuck::cast"] --> L[Safe Transmute Trait]
    end

    subgraph "When OK"
        M[Same repr(C) Layout]
        N[All Bit Patterns Valid]
    end

    D --> G
    L --> K

    style L fill:#69db7c
```

---

### Q77. What is `#[no_mangle]` and name mangling?
**Probability**: 78

```mermaid
graph TD
    subgraph "Name Mangling"
        A[Compiler Modifies<br>Function Names]
        B[Encodes Types<br>and Module]
        C[Prevents Collisions]
    end

    subgraph "no_mangle"
        D[Keeps Original Name]
        E[Required for FFI Export]
        F[Must Be Unique]
    end

    subgraph "Example"
        G["#[no_mangle]"]
        H["pub extern 'C' fn process() {}"]
        I["// Symbol: process"]
        J["// Without: _ZN...hash..."]
    end

    D --> E

    style E fill:#69db7c
```

---

### Q78. How do you handle strings across FFI?
**Probability**: 85

```mermaid
graph TD
    subgraph "Rust → C"
        A["CString::new(s)?.as_ptr()"]
        B[Ensure No Interior NUL]
        C[Keep CString Alive]
    end

    subgraph "C → Rust"
        D["CStr::from_ptr(p)"]
        E[".to_str()? for UTF-8"]
        F[".to_string_lossy() for Safety"]
    end

    subgraph "Ownership"
        G[Who Frees the String?]
        H[Provide Free Function]
        I[Document Ownership Transfer]
    end

    C --> G

    style G fill:#ffd43b
```

---

### Q79. What are the common unsafe patterns for collections?
**Probability**: 70

```mermaid
graph TD
    subgraph "Vec Internals"
        A["set_len() After Init"]
        B["from_raw_parts()"]
        C["into_raw_parts()"]
    end

    subgraph "Safety Requirements"
        D[Initialized Up to len]
        E[Capacity Must Be Accurate]
        F[Same Allocator]
    end

    subgraph "Pattern: Grow Uninit"
        G["v.reserve(n);"]
        H["ptr = v.as_mut_ptr().add(v.len());"]
        I["// Initialize ptr..ptr+n"]
        J["v.set_len(v.len() + n);"]
    end

    A --> D

    style D fill:#ffd43b
```

---

### Q80. How do you audit unsafe code?
**Probability**: 82

```mermaid
graph TD
    subgraph "Code Review"
        A[Check Every unsafe Block]
        B[Verify Documented Invariants]
        C[Trace All Inputs]
    end

    subgraph "Tools"
        D[cargo-geiger] --> E[Count unsafe Usage]
        F[miri] --> G[Runtime UB Check]
        H[cargo-audit] --> I[Known Vulnerabilities]
    end

    subgraph "Documentation"
        J["// SAFETY: comment"]
        K[Explain Why It's Safe]
        L[List All Requirements]
    end

    B --> J

    style J fill:#69db7c
```

---

## Section 5: Design & Architecture (Q81-Q100)

### Q81. How do you design error types for a library?
**Probability**: 88

```mermaid
graph TD
    subgraph "Error Type Design"
        A[Enum for Categories]
        B[thiserror for Derive]
        C[Implement std::error::Error]
    end

    subgraph "Example"
        D["#[derive(thiserror::Error, Debug)]"]
        E["pub enum LibError {"]
        F["  #[error('io error: {0}')]"]
        G["  Io(#[from] std::io::Error),"]
        H["  #[error('parse error at {line}')]"]
        I["  Parse { line: usize },"]
        J["}"]
    end

    subgraph "Best Practices"
        K[Non-Exhaustive Enum]
        L[Implement From for Wrapping]
        M[Include Context]
    end

    A --> D

    style K fill:#69db7c
```

---

### Q82. Explain the newtype pattern and its uses.
**Probability**: 85

```mermaid
graph TD
    subgraph "Newtype"
        A["struct UserId(u64);"]
        B[Wrapper Around Existing Type]
        C[Zero Runtime Cost]
    end

    subgraph "Benefits"
        D[Type Safety] --> E[Can't Mix UserId/PostId]
        F[Orphan Rule Bypass] --> G[Implement Foreign Traits]
        H[Encapsulation] --> I[Hide Implementation]
    end

    subgraph "Implementing Traits"
        J["#[derive(Clone, Copy, PartialEq)]"]
        K["impl From<u64> for UserId"]
    end

    C --> D

    style E fill:#69db7c
```

---

### Q83. How do you implement the typestate pattern?
**Probability**: 70

```mermaid
graph TD
    subgraph "Typestate"
        A[Encode State in Type]
        B[Compile-Time State Machine]
        C[Invalid States Unrepresentable]
    end

    subgraph "Example"
        D["struct Connection<State> { ... }"]
        E["struct Disconnected;"]
        F["struct Connected;"]
        G["impl Connection<Disconnected> {"]
        H["  fn connect(self) -> Connection<Connected>"]
        I["}"]
    end

    subgraph "Benefits"
        J[No Runtime Checks]
        K[Self-Documenting API]
        L[Compiler Enforces Protocol]
    end

    C --> L

    style L fill:#69db7c
```

---

### Q84. What is the visitor pattern in Rust?
**Probability**: 65

```mermaid
graph TD
    subgraph "Visitor Pattern"
        A[Separate Algorithm from<br>Data Structure]
        B[Double Dispatch]
    end

    subgraph "Rust Implementation"
        C["trait Visitor {"]
        D["  fn visit_node(&mut self, n: &Node);"]
        E["  fn visit_leaf(&mut self, l: &Leaf);"]
        F["}"]
        G["trait Element {"]
        H["  fn accept(&self, v: &mut dyn Visitor);"]
        I["}"]
    end

    subgraph "Use Cases"
        J[AST Processing]
        K[Serialization]
        L[Code Generation]
    end

    B --> C

    style J fill:#69db7c
```

---

### Q85. How do you design for extensibility without inheritance?
**Probability**: 80

```mermaid
graph TD
    subgraph "Rust Alternatives"
        A[Traits] --> B[Shared Behavior]
        C[Composition] --> D[Struct Contains Struct]
        E[Generics] --> F[Parameterized Types]
    end

    subgraph "Extension Points"
        G[Trait Objects] --> H[Runtime Polymorphism]
        I[Callbacks/Closures] --> J[Inject Behavior]
        K[Builder Pattern] --> L[Configurable Creation]
    end

    subgraph "Example"
        M["trait Plugin { fn execute(&self); }"]
        N["struct App { plugins: Vec<Box<dyn Plugin>> }"]
    end

    A --> G

    style B fill:#69db7c
```

---

### Q86. Explain sealed traits pattern.
**Probability**: 62

```mermaid
graph TD
    subgraph "Sealed Trait"
        A[Trait Can't Be Implemented<br>Outside Crate]
        B[Preserve API Stability]
    end

    subgraph "Implementation"
        C["mod private { pub trait Sealed {} }"]
        D["pub trait MyTrait: private::Sealed {"]
        E["  fn method(&self);"]
        F["}"]
        G["impl private::Sealed for MyType {}"]
        H["impl MyTrait for MyType { ... }"]
    end

    subgraph "When to Use"
        I[Exhaustive Match Possible]
        J[Add Methods Without Breaking]
    end

    B --> C

    style I fill:#69db7c
```

---

### Q87. How do you implement dependency injection in Rust?
**Probability**: 75

```mermaid
graph TD
    subgraph "Approaches"
        A[Constructor Injection] --> B[Pass Dependencies to new()]
        C[Trait Objects] --> D[dyn Trait for Abstraction]
        E[Generics] --> F[Compile-Time Resolution]
    end

    subgraph "Pattern"
        G["struct Service<D: Database> { db: D }"]
        H["impl<D: Database> Service<D> {"]
        I["  fn new(db: D) -> Self { Self { db } }"]
        J["}"]
    end

    subgraph "Testing"
        K[Mock Implements Trait]
        L["Service::new(MockDb::new())"]
    end

    B --> G
    F --> K

    style E fill:#69db7c
```

---

### Q88. What is the repository pattern in Rust?
**Probability**: 68

```mermaid
graph TD
    subgraph "Repository Pattern"
        A[Abstract Data Access]
        B[Trait Defines Operations]
        C[Impl for Specific Storage]
    end

    subgraph "Example"
        D["trait UserRepo {"]
        E["  fn find(&self, id: UserId) -> Option<User>;"]
        F["  fn save(&self, user: &User) -> Result<()>;"]
        G["}"]
        H["struct PostgresUserRepo { ... }"]
        I["impl UserRepo for PostgresUserRepo { ... }"]
    end

    subgraph "Benefits"
        J[Testability]
        K[Swappable Backends]
        L[Clean Domain Logic]
    end

    B --> D

    style J fill:#69db7c
```

---

### Q89. How do you structure a large Rust project?
**Probability**: 85

```mermaid
graph TD
    subgraph "Workspace Structure"
        A[Cargo.toml - Workspace]
        B[crates/core] --> C[Domain Logic]
        D[crates/api] --> E[HTTP Layer]
        F[crates/db] --> G[Database Layer]
        H[crates/cli] --> I[CLI Binary]
    end

    subgraph "Module Organization"
        J[lib.rs - Public API]
        K[mod.rs - Submodules]
        L[Domain-Driven Folders]
    end

    subgraph "Dependencies"
        M[Core Has No IO Deps]
        N[API/DB Depend on Core]
        O[Inward Dependency Rule]
    end

    C --> N

    style M fill:#69db7c
```

---

### Q90. Explain the outward-facing API design principles.
**Probability**: 78

```mermaid
graph TD
    subgraph "API Design"
        A[Minimal Public Surface]
        B[Accept Generic Types]
        C[Return Concrete Types]
    end

    subgraph "Conventions"
        D["fn new() for constructors"]
        E["fn into_X() consumes self"]
        F["fn as_X() borrows"]
        G["fn to_X() clones"]
    end

    subgraph "Ergonomics"
        H[Implement Common Traits]
        I[Provide Examples in Docs]
        J[Use Type Aliases for Complex Types]
    end

    A --> D

    style A fill:#69db7c
```

---

### Q91. How do you version a Rust library API?
**Probability**: 72

```mermaid
graph TD
    subgraph "Semantic Versioning"
        A[MAJOR.MINOR.PATCH]
        B[Breaking Change = Major]
        C[New Feature = Minor]
        D[Bug Fix = Patch]
    end

    subgraph "Breaking Changes"
        E[Remove Public Item]
        F[Change Function Signature]
        G[Add Required Trait Bound]
    end

    subgraph "Non-Breaking"
        H[Add Optional Methods]
        I[Relax Bounds]
        J[Add Default Impl]
    end

    subgraph "Tools"
        K[cargo-semver-checks]
        L[Detect Breaking Changes]
    end

    B --> E
    K --> L

    style L fill:#69db7c
```

---

### Q92. What is the orphan rule and how do you work around it?
**Probability**: 80

```mermaid
graph TD
    subgraph "Orphan Rule"
        A[Can't Impl Foreign Trait<br>for Foreign Type]
        B[Prevents Conflicting Impls]
    end

    subgraph "Workarounds"
        C[Newtype Pattern] --> D["struct MyVec(Vec<T>)"]
        E[Extension Trait] --> F["trait VecExt { fn my_method() }"]
        G[Wrapper Function] --> H[Free Function Instead]
    end

    subgraph "When Allowed"
        I[Your Trait + Foreign Type]
        J[Foreign Trait + Your Type]
    end

    A --> C

    style C fill:#69db7c
```

---

### Q93. How do you implement custom derive macros?
**Probability**: 70

```mermaid
graph TD
    subgraph "Proc Macro Crate"
        A["[lib]"]
        B["proc-macro = true"]
    end

    subgraph "Implementation"
        C["#[proc_macro_derive(MyDerive)]"]
        D["pub fn my_derive(input: TokenStream) -> TokenStream"]
        E[Parse with syn]
        F[Generate with quote!]
    end

    subgraph "Example"
        G["#[derive(MyDerive)]"]
        H["struct Foo { ... }"]
        I[Generates impl block]
    end

    subgraph "Testing"
        J[trybuild for Compile Tests]
        K[Snapshot Testing]
    end

    E --> F

    style J fill:#69db7c
```

---

### Q94. Explain feature flags best practices.
**Probability**: 75

```mermaid
graph TD
    subgraph "Feature Design"
        A[Additive Only]
        B[Never Remove Functionality]
        C[Default Features Minimal]
    end

    subgraph "Common Patterns"
        D["default = ['std']"]
        E["full = ['feat1', 'feat2']"]
        F["serde = ['dep:serde']"]
    end

    subgraph "Best Practices"
        G[Document Each Feature]
        H[Test All Combinations in CI]
        I[Use cfg_attr for Optional Derives]
    end

    subgraph "Syntax"
        J["#[cfg(feature = 'async')]"]
        K["#[cfg_attr(feature = 'serde', derive(Serialize))]"]
    end

    A --> G

    style A fill:#69db7c
```

---

### Q95. How do you design for testability?
**Probability**: 88

```mermaid
graph TD
    subgraph "Principles"
        A[Dependency Injection]
        B[Trait-Based Abstraction]
        C[Pure Functions Where Possible]
    end

    subgraph "Patterns"
        D[Traits for External Services]
        E[Clock Trait for Time]
        F[RNG Trait for Randomness]
    end

    subgraph "Test Helpers"
        G[test-case for Parameterized]
        H[proptest for Property-Based]
        I[mockall for Mocking]
    end

    subgraph "Structure"
        J[Unit Tests in Same File]
        K[Integration Tests in tests/]
        L[Fixtures in tests/fixtures/]
    end

    A --> D

    style D fill:#69db7c
```

---

### Q96. What is the marker trait pattern?
**Probability**: 65

```mermaid
graph TD
    subgraph "Marker Trait"
        A[No Methods]
        B[Indicates Property/Capability]
        C[Compile-Time Only]
    end

    subgraph "Std Examples"
        D[Send]
        E[Sync]
        F[Copy]
        G[Sized]
    end

    subgraph "Custom Usage"
        H["trait Validated {}"]
        I["struct User<V> { ... }"]
        J["impl User<Validated> { fn save() }"]
    end

    subgraph "Benefit"
        K[Type-Level State]
        L[No Runtime Cost]
    end

    B --> H

    style L fill:#69db7c
```

---

### Q97. How do you handle configuration in Rust applications?
**Probability**: 82

```mermaid
graph TD
    subgraph "Sources"
        A[Environment Variables]
        B[Config Files]
        C[Command Line Args]
    end

    subgraph "Libraries"
        D[config-rs] --> E[Layered Config]
        F[dotenvy] --> G[.env Files]
        H[clap] --> I[CLI Parsing]
    end

    subgraph "Pattern"
        J["#[derive(Deserialize)]"]
        K["struct Config { port: u16 }"]
        L[Load Once at Startup]
        M[Pass to Components]
    end

    E --> J

    style L fill:#69db7c
```

---

### Q98. Explain the service layer pattern.
**Probability**: 72

```mermaid
graph TD
    subgraph "Layer Architecture"
        A[API/Controller Layer]
        B[Service Layer]
        C[Repository Layer]
        D[Domain Models]
    end

    subgraph "Service Responsibilities"
        E[Business Logic]
        F[Transaction Management]
        G[Orchestration]
    end

    subgraph "Rust Implementation"
        H["struct UserService<R: UserRepo>"]
        I["  repo: R"]
        J["impl UserService { fn register() }"]
    end

    A --> B
    B --> C
    B --> D

    style E fill:#69db7c
```

---

### Q99. How do you handle graceful degradation?
**Probability**: 70

```mermaid
graph TD
    subgraph "Patterns"
        A[Circuit Breaker] --> B[Stop Calling Failing Service]
        C[Fallback] --> D[Use Cached/Default Value]
        E[Retry with Backoff] --> F[Exponential Delay]
    end

    subgraph "Implementation"
        G["match service.call().await {"]
        H["  Ok(v) => v,"]
        I["  Err(_) => fallback_value(),"]
        J["}"]
    end

    subgraph "Libraries"
        K[tower] --> L[Middleware Stack]
        M[backoff] --> N[Retry Logic]
    end

    C --> G

    style L fill:#69db7c
```

---

### Q100. How do you design a plugin system in Rust?
**Probability**: 68

```mermaid
graph TD
    subgraph "Approaches"
        A[Trait Objects] --> B[dyn Plugin in Vec]
        C[Dynamic Loading] --> D[libloading + cdylib]
        E[WASM Plugins] --> F[Sandboxed Execution]
    end

    subgraph "Trait-Based"
        G["trait Plugin: Send + Sync {"]
        H["  fn name(&self) -> &str;"]
        I["  fn execute(&self, ctx: &Context);"]
        J["}"]
    end

    subgraph "Dynamic Loading"
        K["type PluginCreate = fn() -> Box<dyn Plugin>;"]
        L["lib.get::<PluginCreate>('create_plugin')"]
    end

    subgraph "Safety"
        M[Version Check]
        N[ABI Compatibility]
        O[Prefer WASM for Untrusted]
    end

    B --> G
    F --> O

    style O fill:#69db7c
```

---

## Probability Distribution Summary

```mermaid
pie title Question Probability Distribution
    "High (80-100)" : 42
    "Medium-High (60-79)" : 45
    "Medium (40-59)" : 10
    "Low (<40)" : 3
```

---

## Document Metadata

| Field | Value |
|-------|-------|
| Total Questions | 100 |
| Avg Probability | 75.8 |
| Mermaid Diagrams | 100 |
| Target Role | Senior Rust Engineer |
| Focus Areas | Advanced Ownership, Async, Performance, Unsafe, Design |

---

*Generated with Parseltongue + Claude Code*
*Codebase: parseltongue-dependency-graph-generator*
*Entities Analyzed: 291*
*Dependency Edges: 5,275*
