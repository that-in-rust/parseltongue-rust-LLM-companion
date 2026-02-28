# Idiomatic Rust Patterns: Proc Macro Server
> Source: rust-analyzer proc-macro crates (proc-macro-api, proc-macro-srv, proc-macro-srv-cli)

## Pattern 1: Process Pool with Load Balancing for External Code
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/pool.rs
**Category:** IPC Design, Concurrency

**Code Example:**
```rust
#[derive(Debug, Clone)]
pub(crate) struct ProcMacroServerPool {
    workers: Arc<[ProcMacroServerProcess]>,
    version: u32,
}

impl ProcMacroServerPool {
    pub(crate) fn pick_process(&self) -> Result<&ProcMacroServerProcess, ServerError> {
        let mut best: Option<&ProcMacroServerProcess> = None;
        let mut best_load = u32::MAX;

        for w in self.workers.iter().filter(|w| w.exited().is_none()) {
            let load = w.number_of_active_req();

            if load == 0 {
                return Ok(w);
            }

            if load < best_load {
                best = Some(w);
                best_load = load;
            }
        }

        best.ok_or_else(|| ServerError {
            message: "all proc-macro server workers have exited".into(),
            io: None,
        })
    }
}
```

**Why This Matters for Contributors:** Shows how to implement a worker pool with intelligent load balancing. The pattern uses fast-path optimization (return immediately if idle worker found) combined with fallback to least-loaded worker. This is critical when external processes (proc-macros) may block or have varying execution times. The use of `Arc<[T]>` instead of `Vec<T>` signals that the pool size is fixed after creation.

---

### Expert Rust Commentary: Pattern 1

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Worker Pool Load Balancing (L2 - std collections, Arc)
- **Complexity:** Intermediate
- **Rust Idioms:** Arc<[T]> for immutable shared slices, fast-path optimization, Option combinators

**Rust-Specific Insights:**
1. **Arc<[T]> vs Arc<Vec<T>>**: Using `Arc<[T]>` signals the pool size is immutable after construction, enabling compiler optimizations and preventing accidental mutations
2. **Fast-path with early return**: The `if load == 0 { return Ok(w); }` pattern avoids unnecessary iteration when an idle worker is found
3. **Filter-map composition**: `workers.iter().filter(|w| w.exited().is_none())` elegantly skips dead workers without explicit conditionals
4. **Option::ok_or_else**: Lazy error construction only when all workers are dead - zero allocation in the common case

**Contribution Tips:**
- When adding new load metrics, maintain the fast-path pattern for zero-load workers
- If implementing worker health checks, consider adding them to the filter predicate
- For bounded pools, prefer `Arc<[T; N]>` for compile-time size checking
- Document the load balancing algorithm choice (least-loaded vs round-robin vs random)

**Common Pitfalls:**
- ❌ Using `Arc<Vec<T>>` and exposing `push()` methods that can't actually mutate
- ❌ Removing the fast-path and always scanning all workers (O(n) in hot path)
- ❌ Not filtering exited workers, leading to errors instead of graceful degradation
- ❌ Using `unwrap()` instead of `ok_or_else()`, panicking when all workers exit

**Related Ecosystem Patterns:**
- **tokio::sync::Semaphore**: For bounded concurrency with backpressure
- **crossbeam-channel bounded**: For explicit load shedding via channel capacity
- **rayon ThreadPool**: For CPU-bound work-stealing pools
- **tower::load::PeakEwma**: For exponentially weighted load tracking

**Design Wisdom:**
This pattern exemplifies the "make the common case fast" principle. By returning immediately for idle workers (common case) and only scanning when system is under load, it optimizes for throughput. The use of `Arc<[T]>` over `Arc<Vec<T>>` is a subtle type-system documentation that the pool is immutable - a pattern seen in high-performance Rust where type choices communicate invariants.

---

## Pattern 2: Multi-Protocol IPC with Version Negotiation
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/process.rs
**Category:** Protocol Design, Version Compatibility

**Code Example:**
```rust
pub(crate) fn run(
    spawn: impl Fn(Option<ProtocolFormat>) -> io::Result<(...)>,
    version: Option<&Version>,
    binary_server_version: impl Fn() -> String,
) -> io::Result<ProcMacroServerProcess> {
    const VERSION: Version = Version::new(1, 93, 0);
    let has_working_format_flag = version.map_or(false, |v| {
        if v.pre.as_str() == "nightly" { *v > VERSION } else { *v >= VERSION }
    });

    let formats: &[_] = if std::env::var_os("RUST_ANALYZER_USE_POSTCARD").is_some()
        && has_working_format_flag
    {
        &[Some(ProtocolFormat::BidirectionalPostcardPrototype), Some(ProtocolFormat::JsonLegacy)]
    } else {
        &[None]
    };

    let mut err = None;
    for &format in formats {
        let mut srv = create_srv()?;
        match srv.version_check(Some(&reject_subrequests)) {
            Ok(v) if v > version::CURRENT_API_VERSION => {
                err = Some(io::Error::other("server too new for client"));
            }
            Ok(v) => {
                srv.version = v;
                return Ok(srv);
            }
            Err(e) => err = Some(io::Error::other(format!("version check failed: {e}"))),
        }
    }
    Err(err.unwrap())
}
```

**Why This Matters for Contributors:** Demonstrates graceful protocol negotiation with fallback. The pattern tries multiple protocols in order of preference (newer postcard format, then legacy JSON). It handles version mismatch by checking both forward and backward compatibility. The nightly vs stable version comparison shows how to handle different release channels. Critical for maintaining compatibility across rustc versions.

---

### Expert Rust Commentary: Pattern 2

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Protocol Negotiation with Fallback (L2 - std, L3 - semver)
- **Complexity:** Advanced
- **Rust Idioms:** Slice iteration with fallthrough, Option combinators, env var guards

**Rust-Specific Insights:**
1. **Slice iteration for fallback chain**: Using `&[Some(Protocol1), Some(Protocol2)]` enables clean fallback iteration without Vec allocation
2. **Nightly vs stable version comparison**: `v.pre.as_str() == "nightly"` shows real-world rustc compatibility handling
3. **Option::map_or pattern**: `version.map_or(false, |v| ...)` provides safe default when version is unknown
4. **Error accumulation**: `err = Some(...)` pattern captures last error for user-friendly diagnostics
5. **Environment-driven protocol selection**: Feature flag via `std::env::var_os` enables A/B testing in production

**Contribution Tips:**
- When adding new protocols, append to the fallback array (never insert at beginning)
- Always test with `version = None` case (unknown rustc version)
- Document protocol compatibility matrix in comments
- Use `tracing::debug!` to log which protocol was selected for debugging
- Consider adding telemetry to track protocol usage in production

**Common Pitfalls:**
- ❌ Returning first error instead of trying all protocols (defeats fallback purpose)
- ❌ Not handling nightly versions specially (they compare strangely with stable)
- ❌ Hardcoding protocol choice instead of environment-driven selection
- ❌ Forgetting to validate server version before using features

**Related Ecosystem Patterns:**
- **tonic gRPC version negotiation**: ALPN-based protocol selection
- **quinn QUIC handshake**: Version negotiation packets
- **rustc metadata versioning**: Binary format evolution with `SVH` hashes
- **postcard wire format evolution**: Discriminant-based backward compatibility

**Design Wisdom:**
This pattern handles the hard problem of distributed system upgrades - clients and servers may be at different versions. The key insight is trying newer protocols first with graceful degradation. The nightly special-casing (`*v > VERSION` vs `*v >= VERSION`) shows attention to Rust's release model. Environment-based feature flags allow safe rollout of new protocols without code changes.

---

## Pattern 3: Dylib Loading with ABI Version Validation
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/dylib.rs
**Category:** Unsafe Code, Dynamic Loading

**Code Example:**
```rust
impl ProcMacroLibrary {
    fn open(path: &Utf8Path) -> Result<Self, LoadProcMacroDylibError> {
        let file = fs::File::open(path)?;
        // SAFETY: Mapping file for reading binary format
        let file = unsafe { memmap2::Mmap::map(&file) }?;
        let obj = object::File::parse(&*file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let version_info = version::read_dylib_info(&obj)?;
        if version_info.version_string != crate::RUSTC_VERSION_STRING {
            return Err(LoadProcMacroDylibError::AbiMismatch(version_info.version_string));
        }

        let symbol_name = find_registrar_symbol(&obj)
            .map_err(invalid_data_err)?
            .ok_or_else(|| invalid_data_err(format!("Cannot find registrar symbol")))?;

        // SAFETY: We have verified the validity of the dylib as a proc-macro library
        let lib = unsafe { load_library(path) }.map_err(invalid_data_err)?;
        // SAFETY: We have verified the validity of the dylib as a proc-macro library
        // The 'static lifetime is a lie, it's actually the lifetime of the library but unavoidable
        // due to self-referentiality. We ensure we do not drop it before the symbol is dropped
        let proc_macros = unsafe { lib.get::<&'static &'static ProcMacros>(symbol_name.as_bytes()) };
        match proc_macros {
            Ok(proc_macros) => Ok(ProcMacroLibrary { proc_macros: *proc_macros, _lib: lib }),
            Err(e) => Err(e.into()),
        }
    }
}
```

**Why This Matters for Contributors:** Shows the correct pattern for safe dynamic library loading: 1) Parse the binary file, 2) Extract and validate version info BEFORE loading, 3) Find required symbols, 4) Only then load the library. The extensive SAFETY comments explain the lifetime lie required by self-referential structs. The field ordering (`proc_macros` before `_lib`) ensures correct drop order. Essential for anyone working with FFI or dynamic loading.

---

### Expert Rust Commentary: Pattern 3

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Safe Dynamic Library Loading (L3 - unsafe, memmap2, object)
- **Complexity:** Expert
- **Rust Idioms:** Multi-stage validation, explicit SAFETY comments, drop order via field position

**Rust-Specific Insights:**
1. **Validation before loading**: Parse binary → validate version → find symbols → THEN load. This prevents loading incompatible code into address space
2. **Self-referential struct with lifetime lie**: `proc_macros: &'static &'static ProcMacros` is actually bounded by `_lib` lifetime, but type system can't express it
3. **Drop order guarantee**: `proc_macros` field comes before `_lib` field, ensuring symbols are dropped before library unloads (Rust drops fields in declaration order)
4. **SAFETY comment discipline**: Each `unsafe` block has a preceding comment explaining the invariant being upheld
5. **Error propagation in validation chain**: Using `?` operator even in unsafe context maintains clean error handling

**Contribution Tips:**
- Never reorder struct fields - `proc_macros` MUST come before `_lib`
- When adding new validation steps, insert them before `load_library()` call
- Always validate ABI version matches exactly (no "compatible versions" - FFI is fragile)
- Use `tracing::warn!` if you must skip version check (for testing only)
- Consider adding symbol signature validation beyond just name matching

**Common Pitfalls:**
- ❌ Loading library first, then validating (too late - incompatible code already in memory)
- ❌ Reordering struct fields and breaking drop order safety
- ❌ Using lifetime `'a` instead of `'static` for dylib symbols (doesn't compile but tempting)
- ❌ Not checking symbol exists before loading (panics in `lib.get()`)
- ❌ Forgetting memmap is `unsafe` even for read-only files

**Related Ecosystem Patterns:**
- **rustc_driver dylib loading**: Similar ABI validation for compiler plugins
- **wasmer/wasmtime module loading**: Validation before instantiation
- **dlopen-rs**: Higher-level safe wrapper over libloading
- **shared-library**: Platform abstraction for dynamic libraries

**Design Wisdom:**
This pattern showcases defense-in-depth for unsafe code: validate everything possible before crossing the unsafe boundary. The "lifetime lie" is documented honestly - acknowledging when type system limitations force unsafe code. The field ordering trick for drop order is a subtle but critical Rust pattern seen in FFI code. The multi-stage validation (binary parse → version → symbols → load) prevents many classes of dynamic loading errors.

---

## Pattern 4: Platform-Specific Dylib Loading with RTLD_DEEPBIND
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/dylib.rs
**Category:** Platform Abstraction, Unsafe Code

**Code Example:**
```rust
/// Loads dynamic library in platform dependent manner.
/// For unix, you have to use RTLD_DEEPBIND flag to escape problems with
/// symbol conflicts in proc-macros
#[cfg(unix)]
unsafe fn load_library(file: &Utf8Path) -> Result<Library, libloading::Error> {
    #[cfg(target_env = "gnu")]
    use libc::RTLD_DEEPBIND;
    use libloading::os::unix::Library as UnixLibrary;
    use libloading::os::unix::RTLD_NOW;

    // MUSL and bionic don't have it..
    #[cfg(not(target_env = "gnu"))]
    const RTLD_DEEPBIND: std::os::raw::c_int = 0x0;

    // SAFETY: The caller is responsible for ensuring that the path is valid proc-macro library
    unsafe { UnixLibrary::open(Some(file), RTLD_NOW | RTLD_DEEPBIND).map(|lib| lib.into()) }
}

#[cfg(windows)]
unsafe fn load_library(file: &Utf8Path) -> Result<Library, libloading::Error> {
    // SAFETY: The caller is responsible for ensuring that the path is valid proc-macro library
    unsafe { Library::new(file) }
}
```

**Why This Matters for Contributors:** Shows how to handle platform-specific requirements while maintaining a unified interface. On Unix, `RTLD_DEEPBIND` isolates the proc-macro's symbols from the main process to prevent conflicts. The pattern gracefully handles platforms without `RTLD_DEEPBIND` by defining it as 0. The SAFETY comments push responsibility to the caller, establishing a safety contract. Important for cross-platform FFI work.

---

### Expert Rust Commentary: Pattern 4

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Platform-Specific FFI (L3 - libloading, libc)
- **Complexity:** Advanced
- **Rust Idioms:** Conditional compilation, unified interface, const fallback

**Rust-Specific Insights:**
1. **cfg-based platform dispatch**: Same function signature across platforms, but different implementations via `#[cfg(unix)]` / `#[cfg(windows)]`
2. **Target environment granularity**: `#[cfg(target_env = "gnu")]` shows finer-grained conditional compilation beyond OS level
3. **Const fallback for missing symbols**: Defining `RTLD_DEEPBIND = 0` on non-GNU targets shows graceful degradation pattern
4. **Safety contract pushing**: SAFETY comments state "caller responsible" - establishes safety invariant boundary
5. **Library type conversion**: `UnixLibrary::open(...).map(|lib| lib.into())` unifies platform types

**Contribution Tips:**
- When adding platform support, maintain the unified function signature
- Document why platform-specific code is needed (symbol isolation, performance, etc.)
- Test on all target environments (gnu, musl, bionic for Linux; windows-msvc vs windows-gnu)
- Consider adding runtime detection instead of compile-time for some platforms
- Use `#[cfg(all(unix, not(target_env = "gnu")))]` for explicit multi-platform logic

**Common Pitfalls:**
- ❌ Using `#[cfg(target_os = "linux")]` when you need `#[cfg(unix)]` (excludes macOS/BSD)
- ❌ Forgetting to define fallback constants (compile errors on non-GNU)
- ❌ Not testing on MUSL/Alpine Linux (common Docker base image)
- ❌ Assuming all Unix systems have same libc (they don't)
- ❌ Duplicating logic instead of abstracting common parts

**Related Ecosystem Patterns:**
- **rustix platform abstractions**: Unified syscall interface across Unix variants
- **winapi vs windows-sys**: Windows API bindings evolution
- **libc crate**: Platform constant definitions with cfg attributes
- **nix crate**: Higher-level Unix system calls with platform handling

**Design Wisdom:**
This pattern demonstrates sophisticated platform abstraction. The key insight is `RTLD_DEEPBIND` prevents symbol conflicts when loading multiple proc-macro dylibs that might link different versions of the same dependencies. The graceful degradation (defining it as 0 on unsupported platforms) is pragmatic - symbol isolation is nice-to-have, not required. The unified interface hides platform complexity from callers.

---

## Pattern 5: Copy-on-Windows for Lock-Free DLL Access
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/dylib.rs
**Category:** Platform Workarounds, File System

**Code Example:**
```rust
#[cfg(windows)]
fn ensure_file_with_lock_free_access(
    temp_dir: &TempDir,
    path: &Utf8Path,
) -> io::Result<Utf8PathBuf> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    if std::env::var("RA_DONT_COPY_PROC_MACRO_DLL").is_ok() {
        return Ok(path.to_path_buf());
    }

    let mut to = Utf8Path::from_path(temp_dir.path()).unwrap().to_owned();
    let file_name = path.file_stem().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("File path is invalid: {path}"))
    })?;

    to.push({
        // Generate a unique number by abusing `HashMap`'s hasher.
        // Maybe this will also "inspire" a libs team member to finally put `rand` in libstd.
        let unique_name = RandomState::new().build_hasher().finish();
        format!("{file_name}-{unique_name}.dll")
    });
    fs::copy(path, &to)?;
    Ok(to)
}

#[cfg(unix)]
fn ensure_file_with_lock_free_access(_temp_dir: &TempDir, path: &Utf8Path) -> io::Result<Utf8PathBuf> {
    Ok(path.to_owned())
}
```

**Why This Matters for Contributors:** Solves Windows-specific DLL locking issues by copying to temp directory with unique name. The creative use of `HashMap`'s internal hasher for uniqueness (with tongue-in-cheek comment about stdlib) shows pragmatic problem-solving. The escape hatch via environment variable allows disabling the workaround. On Unix, it's a no-op. Excellent example of handling OS-specific constraints transparently.

---

### Expert Rust Commentary: Pattern 5

**Idiomatic Rating: ⭐⭐⭐⭐ (4/5)**

**Pattern Classification:**
- **Category:** Platform Workaround (L2 - std fs, collections)
- **Complexity:** Intermediate
- **Rust Idioms:** cfg-based no-op, creative stdlib usage, escape hatch via env var

**Rust-Specific Insights:**
1. **Unix no-op pattern**: `#[cfg(unix)] fn ensure(...) -> PathBuf { Ok(path.to_owned()) }` shows zero-cost abstraction on platforms that don't need workarounds
2. **Creative hasher usage**: Abusing `RandomState::new().build_hasher().finish()` for random number generation (with tongue-in-cheek stdlib comment)
3. **Escape hatch environment variable**: `RA_DONT_COPY_PROC_MACRO_DLL` allows disabling workaround for debugging or special deployments
4. **File stem extraction**: `path.file_stem().ok_or_else(|| ...)` with custom error message shows defensive path handling
5. **Windows file locking knowledge**: Comment documents the OS-level problem being solved (exclusive locks on loaded DLLs)

**Contribution Tips:**
- When debugging DLL issues on Windows, set `RA_DONT_COPY_PROC_MACRO_DLL=1` to see original errors
- Consider using `temp_dir.path().join()` instead of manual `to.push()` for clarity
- Add cleanup logic for orphaned temp DLLs (they accumulate on crashes)
- Document the Windows locking behavior in top-level comments
- Profile if `fs::copy()` becomes a bottleneck on large dylibs

**Common Pitfalls:**
- ❌ Using `rand` crate dependency just for unique name generation (overkill)
- ❌ Not handling temp directory cleanup on long-running processes
- ❌ Assuming Unix doesn't need this (it doesn't, but could change)
- ❌ Not providing escape hatch for testing/debugging
- ❌ Using timestamp instead of random for uniqueness (race conditions)

**Related Ecosystem Patterns:**
- **tempfile crate**: More robust temporary file management
- **uuid crate**: Standard unique identifier generation
- **shadow-rs**: Binary metadata embedding
- **Windows DLL loading**: LoadLibraryEx with LOAD_LIBRARY_AS_DATAFILE

**Design Wisdom:**
This pattern is a prime example of OS-specific workarounds in cross-platform code. Windows locks DLLs exclusively when loaded, preventing cargo from overwriting them during rebuild. The solution: copy to temp with unique name before loading. The hasher abuse is clever but fragile - a dependency on `uuid` or `tempfile` would be more robust. The Unix no-op shows the right abstraction - platform differences hidden behind same interface.

**Rating Rationale:** Deducted one star for the hasher abuse (creative but not idiomatic) and lack of temp file cleanup (resource leak on long runs).

---

## Pattern 6: Reading Rustc Version from Binary Metadata
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/dylib/version.rs
**Category:** Binary Format Parsing, Metadata Extraction

**Code Example:**
```rust
pub fn read_version(obj: &object::File<'_>) -> io::Result<String> {
    let dot_rustc = read_section(obj, ".rustc")?;

    // check if magic is valid
    if &dot_rustc[0..4] != b"rust" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown metadata magic, expected `rust`, found `{:?}`", &dot_rustc[0..4]),
        ));
    }
    let version = u32::from_be_bytes([dot_rustc[4], dot_rustc[5], dot_rustc[6], dot_rustc[7]]);

    let (mut metadata_portion, bytes_before_version) = match version {
        8 => {
            let len_bytes = &dot_rustc[8..12];
            let data_len = u32::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
            (&dot_rustc[12..data_len + 12], 13)
        }
        9 | 10 => {
            let len_bytes = &dot_rustc[8..16];
            let data_len = u64::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
            (&dot_rustc[16..data_len + 12], 17)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported metadata version {version}"),
            ));
        }
    };

    let mut bytes = [0u8; 17];
    metadata_portion.read_exact(&mut bytes[..bytes_before_version])?;
    let length = bytes[bytes_before_version - 1];

    let mut version_string_utf8 = vec![0u8; length as usize];
    metadata_portion.read_exact(&mut version_string_utf8)?;
    let version_string = String::from_utf8(version_string_utf8);
    version_string.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

**Why This Matters for Contributors:** Shows how to parse binary formats with evolving schemas. The pattern handles multiple metadata versions (8, 9, 10) with different layout rules. Magic bytes validation prevents processing invalid files. The detailed comments explain the binary layout. This is crucial for ABI compatibility checking between rust-analyzer and proc-macro dylibs. Demonstrates defensive binary parsing with clear error messages.

---

### Expert Rust Commentary: Pattern 6

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Binary Format Parsing with Schema Evolution (L3 - object crate)
- **Complexity:** Expert
- **Rust Idioms:** Match on version, explicit endianness, defensive validation

**Rust-Specific Insights:**
1. **Magic byte validation**: Checking `&dot_rustc[0..4] != b"rust"` prevents processing garbage data early
2. **Explicit endianness handling**: `u32::from_be_bytes` vs `u64::from_le_bytes` documents binary format precisely (version 8 uses big-endian, 9/10 use little-endian)
3. **Exhaustive version matching**: Match on exact versions (8, 9, 10) with explicit error for unknown - forces conscious decision when format changes
4. **TryInto for array slices**: `len_bytes.try_into().unwrap()` converts `&[u8]` to `[u8; N]` - safe because slice size is validated
5. **Multi-stage parsing**: Extract metadata portion → read length byte → read version string - each stage validates its preconditions

**Contribution Tips:**
- When rustc metadata format changes, add new version arm to match statement
- Document endianness choice for each version in comments
- Use `tracing::debug!` to log version and metadata offset for debugging
- Consider extracting version-specific parsing to separate functions
- Add fuzzing targets for this function (high-risk parsing code)

**Common Pitfalls:**
- ❌ Using host endianness instead of explicit big/little endian conversion
- ❌ Not validating magic bytes before accessing rest of data
- ❌ Using `unsafe` slice indexing instead of safe range operators
- ❌ Returning generic error messages (hard to debug for users)
- ❌ Not handling future metadata versions gracefully

**Related Ecosystem Patterns:**
- **rustc_metadata crate**: Full implementation of rustc metadata parsing
- **object crate**: Generic binary format parsing (ELF, Mach-O, PE)
- **goblin crate**: Alternative binary parser with zero-copy focus
- **binrw crate**: Derive macros for binary reading/writing

**Design Wisdom:**
This pattern showcases defensive binary parsing. The progression (magic check → version → length → data) validates assumptions at each stage. The explicit version matching forces maintainers to consciously handle new formats. The different endianness across versions shows real-world format evolution messiness. Error messages include context (expected vs actual), making debugging easier. This is production-grade parsing - no shortcuts, no assumptions.

---

## Pattern 7: Newline-Delimited JSON with Ill-Behaved Stdout Filtering
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/transport/json.rs
**Category:** IPC Protocol, Error Recovery

**Code Example:**
```rust
pub(crate) fn read<'a, R: BufRead + ?Sized>(
    inp: &mut R,
    buf: &'a mut String,
) -> io::Result<Option<&'a mut String>> {
    loop {
        buf.clear();

        inp.read_line(buf)?;
        buf.pop(); // Remove trailing '\n'

        if buf.is_empty() {
            return Ok(None);
        }

        // Some ill behaved macro try to use stdout for debugging
        // We ignore it here
        if !buf.starts_with('{') {
            tracing::error!("proc-macro tried to print : {}", buf);
            continue;
        }

        return Ok(Some(buf));
    }
}

pub(crate) fn decode<T: DeserializeOwned>(buf: &mut str) -> io::Result<T> {
    let mut deserializer = serde_json::Deserializer::from_str(buf);
    // Note that some proc-macro generate very deep syntax tree
    // We have to disable the current limit of serde here
    deserializer.disable_recursion_limit();
    Ok(T::deserialize(&mut deserializer)?)
}
```

**Why This Matters for Contributors:** Shows defensive IPC design that handles misbehaving external code. The loop filters out non-JSON lines (proc-macros printing to stdout), logging them as errors but continuing. Disabling recursion limit handles deeply nested token trees. This robustness is essential when you can't control the code running in the other process. Pattern demonstrates real-world IPC where you can't trust the other end to be well-behaved.

---

### Expert Rust Commentary: Pattern 7

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Defensive IPC Protocol (L3 - serde_json, tracing)
- **Complexity:** Advanced
- **Rust Idioms:** Loop with continue for filtering, recursion limit control, error logging

**Rust-Specific Insights:**
1. **Loop-based filtering**: `loop { ... if !buf.starts_with('{') { continue; } }` handles arbitrary non-JSON output robustly
2. **EOF detection**: `if buf.is_empty() { return Ok(None); }` after `read_line()` detects clean shutdown
3. **Disabling recursion limit**: `deserializer.disable_recursion_limit()` acknowledges deep token trees are legitimate in this domain
4. **Error logging without failing**: `tracing::error!(...)` logs but continues - important for debugging misbehaving macros
5. **Borrowed string deserialization**: Taking `&'a mut String` and returning `Option<&'a mut String>` avoids allocations

**Contribution Tips:**
- Add rate limiting on error logging (proc-macro could spam stdout infinitely)
- Consider collecting filtered lines in debug mode for later analysis
- Document why recursion limit is disabled (deeply nested macro_rules! expansions)
- Add timeout on `read_line()` to detect hung processes
- Instrument with metrics: count of filtered lines, max recursion depth seen

**Common Pitfalls:**
- ❌ Panicking on non-JSON output instead of filtering (breaks on debug prints)
- ❌ Not disabling recursion limit (fails on legitimate deeply nested syntax)
- ❌ Forgetting to pop newline character (JSON parser fails)
- ❌ Using `eprintln!` instead of `tracing::error!` (doesn't respect logging config)
- ❌ Not handling EOF explicitly (infinite loop on process exit)

**Related Ecosystem Patterns:**
- **newline-delimited JSON (NDJSON)**: Standard for streaming JSON protocol
- **serde_json streaming**: Deserializer from `Read` for larger messages
- **tracing instrumentation**: Structured logging in async/IPC systems
- **tower middleware**: Similar filtering/transformation patterns for HTTP

**Design Wisdom:**
This pattern demonstrates IPC robustness in hostile environments. You cannot trust the other end of the pipe - it might print debug output, panic messages, or garbage. The loop-filter pattern is more robust than trying to prevent bad output. Disabling recursion limit is a domain-specific decision: in proc-macros, deep nesting is legitimate (macro_rules! expansions can be 100+ levels deep). The error logging provides debugging visibility without failing the protocol.

---

## Pattern 8: COBS-Encoded Postcard for Binary IPC
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/transport/postcard.rs
**Category:** Binary Protocol, Serialization

**Code Example:**
```rust
pub(crate) fn read<'a, R: BufRead + ?Sized>(
    inp: &mut R,
    buf: &'a mut Vec<u8>,
) -> io::Result<Option<&'a mut Vec<u8>>> {
    buf.clear();
    let n = inp.read_until(0, buf)?;
    if n == 0 {
        return Ok(None);
    }
    Ok(Some(buf))
}

pub(crate) fn write<W: Write + ?Sized>(out: &mut W, buf: &[u8]) -> io::Result<()> {
    out.write_all(buf)?;
    out.flush()
}

pub(crate) fn encode<T: Serialize>(msg: &T) -> io::Result<Vec<u8>> {
    postcard::to_allocvec_cobs(msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub(crate) fn decode<T: DeserializeOwned>(buf: &mut [u8]) -> io::Result<T> {
    postcard::from_bytes_cobs(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

**Why This Matters for Contributors:** Shows alternative to JSON for performance-critical IPC. COBS (Consistent Overhead Byte Stuffing) encoding allows framing binary messages using null bytes as delimiters, avoiding the overhead of length prefixes. The pattern is minimal and symmetric (encode/decode mirror each other). Important when JSON's text overhead becomes a bottleneck, especially for large token streams. The use of `read_until(0, buf)` for framing is elegant.

---

### Expert Rust Commentary: Pattern 8

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Binary IPC Protocol (L3 - postcard with COBS encoding)
- **Complexity:** Intermediate
- **Rust Idioms:** Symmetric encode/decode, delimiter-based framing, zero-copy reads

**Rust-Specific Insights:**
1. **COBS framing elegance**: Consistent Overhead Byte Stuffing allows using null byte (0) as delimiter without escaping overhead of length prefixes
2. **BufRead::read_until pattern**: `inp.read_until(0, buf)` leverages buffered I/O for efficient framing
3. **Symmetric API design**: `encode`/`decode` and `read`/`write` mirror each other - easy to reason about protocol
4. **Zero-allocation reading**: `read()` takes `&'a mut Vec<u8>` and returns reference to same buffer - no copies
5. **Flush guarantee**: `out.flush()` ensures message is on the wire before returning (prevents buffering deadlocks)

**Contribution Tips:**
- When switching from JSON to postcard, benchmark both (postcard is faster but less debuggable)
- Use `postcard::to_allocvec()` for variable-sized messages without COBS if delimiter isn't needed
- Add protocol version field to messages for future evolution
- Consider adding compression for large token streams (after COBS encoding)
- Document max message size to prevent unbounded buffer growth

**Common Pitfalls:**
- ❌ Forgetting to flush write buffer (deadlocks on bidirectional protocols)
- ❌ Not clearing buffer between reads (data corruption)
- ❌ Using postcard without COBS and trying to frame with length prefixes (complex)
- ❌ Assuming postcard is self-describing like JSON (it's not - schema must match)
- ❌ Not handling partial reads (though `read_until` handles this)

**Related Ecosystem Patterns:**
- **bincode**: Alternative binary serialization (more features, slightly slower)
- **COBS encoding**: Used in embedded systems, CAN bus protocols
- **Cap'n Proto / FlatBuffers**: Zero-copy deserialization formats
- **MessagePack**: Self-describing binary format (trade-off vs postcard)

**Design Wisdom:**
This pattern shows the evolution from JSON (pattern 7) to binary protocol. COBS encoding is elegant: it ensures no null bytes in payload by encoding them, then uses null as delimiter. The result: simple framing (`read_until(0)`) with minimal overhead (one extra byte per 254 bytes of payload). The symmetric design makes protocol bugs obvious - if encode and decode don't mirror, it won't compile. Perfect for performance-critical IPC where JSON overhead matters.

---

## Pattern 9: Bidirectional IPC with Sub-Request Callbacks
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/bidirectional_protocol.rs
**Category:** Advanced IPC, Callback Pattern

**Code Example:**
```rust
pub type SubCallback<'a> = &'a dyn Fn(SubRequest) -> Result<SubResponse, ServerError>;

pub fn run_conversation(
    writer: &mut dyn Write,
    reader: &mut dyn BufRead,
    buf: &mut Vec<u8>,
    msg: BidirectionalMessage,
    callback: SubCallback<'_>,
) -> Result<BidirectionalMessage, ServerError> {
    let encoded = postcard::encode(&msg).map_err(wrap_encode)?;
    postcard::write(writer, &encoded).map_err(wrap_io("failed to write initial request"))?;

    loop {
        let msg: BidirectionalMessage = postcard::decode(b).map_err(wrap_decode)?;

        match msg {
            BidirectionalMessage::Response(response) => {
                return Ok(BidirectionalMessage::Response(response));
            }
            BidirectionalMessage::SubRequest(sr) => {
                let resp = match catch_unwind(AssertUnwindSafe(|| callback(sr))) {
                    Ok(Ok(resp)) => BidirectionalMessage::SubResponse(resp),
                    Ok(Err(err)) => BidirectionalMessage::SubResponse(SubResponse::Cancel {
                        reason: err.to_string(),
                    }),
                    Err(_) => BidirectionalMessage::SubResponse(SubResponse::Cancel {
                        reason: "callback panicked or was cancelled".into(),
                    }),
                };

                let encoded = postcard::encode(&resp).map_err(wrap_encode)?;
                postcard::write(writer, &encoded).map_err(wrap_io("failed to write sub-response"))?;
            }
            _ => return Err(ServerError { message: format!("unexpected message {:?}", msg), io: None }),
        }
    }
}
```

**Why This Matters for Contributors:** Implements bidirectional RPC where the server can make callbacks to the client during expansion. The proc-macro server can request span information from the client mid-expansion. Uses `catch_unwind` to handle panics in the callback gracefully, converting them to protocol-level errors. This allows proc-macros to query IDE state (file paths, line/column info) without exposing the entire database. Advanced pattern for interactive IPC.

---

### Expert Rust Commentary: Pattern 9

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Bidirectional RPC with Callbacks (L2 - std panic, L3 - postcard)
- **Complexity:** Expert
- **Rust Idioms:** catch_unwind for panic conversion, trait object callbacks, loop-based state machine

**Rust-Specific Insights:**
1. **Type alias for callback clarity**: `type SubCallback<'a> = &'a dyn Fn(...)` documents the callback contract at type level
2. **Panic boundary with catch_unwind**: `catch_unwind(AssertUnwindSafe(|| callback(...)))` prevents panics from crossing IPC boundary
3. **Loop-based protocol state machine**: `loop { match msg { Response => return, SubRequest => handle, ... } }` elegantly handles multi-turn protocol
4. **Converting panics to protocol errors**: Maps Rust panics to `SubResponse::Cancel` - important for debugging across process boundaries
5. **AssertUnwindSafe usage**: Explicitly states "we've reviewed this closure and panic is okay to catch"

**Contribution Tips:**
- Add timeout on the main loop to detect hung servers
- Consider using `tokio::select!` for async version with cancellation
- Log all SubRequests with timing for performance debugging
- Document which operations can trigger SubRequests (helps API users)
- Add protocol trace logging for debugging complex interactions

**Common Pitfalls:**
- ❌ Not catching panics in callbacks (crashes entire process)
- ❌ Forgetting to write response for SubRequest (deadlocks protocol)
- ❌ Infinite loop on unexpected message instead of erroring
- ❌ Not propagating cancellation reason to caller
- ❌ Blocking on I/O inside callback (should be async or fast)

**Related Ecosystem Patterns:**
- **tower Service trait**: Similar request/response abstraction
- **tarpc RPC framework**: Bidirectional RPC for Rust
- **capnp-rpc**: Capability-based RPC with promise pipelining
- **Language Server Protocol**: Similar bidirectional protocol ($/progress requests)

**Design Wisdom:**
This pattern implements advanced RPC: the server can call back to the client during processing. This is critical for proc-macros that need IDE context (file paths, span information) without exposing the entire database. The panic catching is essential - proc-macro code is untrusted and can panic. Converting panics to protocol-level Cancel messages allows graceful error handling across the boundary. The loop-based state machine is clearer than async/await for this sequential protocol.

---

## Pattern 10: Thread-Scoped Expansion with Stack Size Control
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/lib.rs
**Category:** Thread Management, Resource Control

**Code Example:**
```rust
const EXPANDER_STACK_SIZE: usize = 8 * 1024 * 1024;

pub fn expand<S: ProcMacroSrvSpan>(
    &self,
    lib: impl AsRef<Utf8Path>,
    env: &[(String, String)],
    current_dir: Option<impl AsRef<Path>>,
    macro_name: &str,
    macro_body: token_stream::TokenStream<S>,
    attribute: Option<token_stream::TokenStream<S>>,
    def_site: S,
    call_site: S,
    mixed_site: S,
    callback: Option<ProcMacroClientHandle<'_>>,
) -> Result<token_stream::TokenStream<S>, ExpandError> {
    let prev_env = EnvChange::apply(snapped_env, env, current_dir.as_ref().map(<_>::as_ref));

    // Note, we spawn a new thread here so that thread locals allocation don't accumulate
    // (this includes the proc-macro symbol interner)
    let result = thread::scope(|s| {
        let thread = thread::Builder::new()
            .stack_size(EXPANDER_STACK_SIZE)
            .name(macro_name.to_owned())
            .spawn_scoped(s, move || {
                expander.expand(macro_name, macro_body, attribute, def_site, call_site, mixed_site, callback)
            });
        match thread.unwrap().join() {
            Ok(res) => res.map_err(ExpandError::Panic),
            Err(payload) => {
                if let Some(marker) = payload.downcast_ref::<ProcMacroPanicMarker>() {
                    return match marker {
                        ProcMacroPanicMarker::Cancelled { reason } => {
                            Err(ExpandError::Cancelled { reason: Some(reason.clone()) })
                        }
                        ProcMacroPanicMarker::Internal { reason } => {
                            Err(ExpandError::Internal { reason: Some(reason.clone()) })
                        }
                    };
                }
                std::panic::resume_unwind(payload)
            }
        }
    });
    prev_env.rollback();
    result
}
```

**Why This Matters for Contributors:** Shows how to isolate potentially dangerous external code. Each expansion runs in a fresh thread with controlled stack size (8MB) to prevent stack overflow from deeply nested macros. Thread locals are reset between expansions to prevent memory leaks. Environment variables are restored via RAII (`EnvChange`). Panics are caught and converted to errors, with custom panic markers to distinguish user errors from internal errors. Essential pattern for sandboxing untrusted code.

---

### Expert Rust Commentary: Pattern 10

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Thread Isolation with Resource Control (L2 - std::thread::scope)
- **Complexity:** Advanced
- **Rust Idioms:** Scoped threads, custom stack size, panic recovery, RAII cleanup

**Rust-Specific Insights:**
1. **Scoped threads for non-'static borrows**: `thread::scope(|s| s.spawn_scoped(...))` allows borrowing parent scope without 'static bound
2. **Custom stack size**: `EXPANDER_STACK_SIZE = 8MB` prevents stack overflow from deeply nested macro expansions
3. **Thread naming**: `thread::Builder::new().name(macro_name.to_owned())` aids debugging in profilers/debuggers
4. **Downcast for custom panic markers**: `payload.downcast_ref::<ProcMacroPanicMarker>()` extracts typed panic info
5. **RAII environment restoration**: `prev_env.rollback()` called after thread finishes, even on error/panic
6. **Resume_unwind for unknown panics**: `std::panic::resume_unwind(payload)` re-raises panics we don't recognize

**Contribution Tips:**
- Profile stack usage to tune `EXPANDER_STACK_SIZE` (8MB is conservative)
- Add metrics for expansion time distribution to find slow macros
- Consider adding CPU time limit (not just stack) for runaway macros
- Log panic payloads that aren't our custom markers (debugging unknown panics)
- Test with extremely nested macros (100+ levels) to validate stack size

**Common Pitfalls:**
- ❌ Not using scoped threads, forcing 'static lifetime on all captures
- ❌ Default stack size (2MB) too small for deeply nested expansions
- ❌ Forgetting to restore environment on panic (leaks modified state)
- ❌ Not catching panics at all (crashes IDE on proc-macro panic)
- ❌ Blocking parent thread waiting for expansion (should use async or thread pool)

**Related Ecosystem Patterns:**
- **rayon::spawn** for CPU-bound work stealing
- **tokio::task::spawn_blocking** for blocking work in async context
- **std::thread::Builder** for custom thread configuration
- **thread_local!** for macro-specific state that must reset between expansions

**Design Wisdom:**
This pattern showcases comprehensive sandboxing of untrusted code. Each expansion gets: (1) Fresh thread (resets thread locals), (2) Custom stack (prevents overflow), (3) Panic catching (prevents crashes), (4) Environment isolation (via RAII). The scoped threads pattern is key - it allows borrowing server state without 'static while ensuring cleanup. The typed panic markers enable distinguishing user errors from internal errors across the panic boundary.

---

## Pattern 11: Environment Snapshot and Restoration with Global Lock
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/lib.rs
**Category:** Environment Management, RAII

**Code Example:**
```rust
pub struct EnvSnapshot {
    vars: HashMap<OsString, OsString>,
}

impl Default for EnvSnapshot {
    fn default() -> EnvSnapshot {
        EnvSnapshot { vars: env::vars_os().collect() }
    }
}

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

struct EnvChange<'snap> {
    changed_vars: Vec<&'snap str>,
    prev_working_dir: Option<PathBuf>,
    snap: &'snap EnvSnapshot,
    _guard: std::sync::MutexGuard<'snap, ()>,
}

impl<'snap> EnvChange<'snap> {
    fn apply(
        snap: &'snap EnvSnapshot,
        new_vars: &'snap [(String, String)],
        current_dir: Option<&Path>,
    ) -> EnvChange<'snap> {
        let guard = ENV_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev_working_dir = match current_dir {
            Some(dir) => {
                let prev_working_dir = std::env::current_dir().ok();
                if let Err(err) = std::env::set_current_dir(dir) {
                    eprintln!("Failed to set the current working dir to {}. Error: {err:?}", dir.display())
                }
                prev_working_dir
            }
            None => None,
        };
        EnvChange {
            snap,
            changed_vars: new_vars.iter().map(|(k, v)| {
                // SAFETY: We have acquired the environment lock
                unsafe { env::set_var(k, v) };
                &**k
            }).collect(),
            prev_working_dir,
            _guard: guard,
        }
    }
    fn rollback(self) {}
}

impl Drop for EnvChange<'_> {
    fn drop(&mut self) {
        for name in self.changed_vars.drain(..) {
            // SAFETY: We have acquired the environment lock
            unsafe {
                match self.snap.vars.get::<std::ffi::OsStr>(name.as_ref()) {
                    Some(prev_val) => env::set_var(name, prev_val),
                    None => env::remove_var(name),
                }
            }
        }
        if let Some(dir) = &self.prev_working_dir
            && let Err(err) = std::env::set_current_dir(dir)
        {
            eprintln!("Failed to set the current working dir to {}. Error: {:?}", dir.display(), err)
        }
    }
}
```

**Why This Matters for Contributors:** Demonstrates safe environment variable mutation in a multi-threaded context. The global `ENV_LOCK` ensures only one thread modifies environment at a time (since `std::env` is process-global, not thread-local). The `_guard` field in `EnvChange` holds the lock for the entire scope via RAII. On drop, all environment changes are rolled back. Critical pattern when you need to temporarily modify process state for external code while maintaining safety.

---

### Expert Rust Commentary: Pattern 11

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Safe Global State Mutation (L2 - std::sync::Mutex, env)
- **Complexity:** Advanced
- **Rust Idioms:** Global lock via static Mutex, RAII restoration, poison recovery

**Rust-Specific Insights:**
1. **Global lock for process-global state**: `static ENV_LOCK: Mutex<()>` protects `std::env` calls which affect entire process, not just current thread
2. **Lock guard as RAII member**: `_guard: MutexGuard<'snap, ()>` field ensures lock held for entire scope of `EnvChange`
3. **Poison recovery**: `unwrap_or_else(std::sync::PoisonError::into_inner)` allows continuing after panic in other thread
4. **Snapshot restoration on drop**: Drop impl restores env vars to snapshot state - works even on panic
5. **Tracked mutations**: `changed_vars: Vec<&'snap str>` tracks what was changed for efficient restoration
6. **Empty rollback() method**: Forces users to call it explicitly for clarity, though Drop does the work

**Contribution Tips:**
- Add thread-local cache of environment to avoid repeated `env::var()` calls
- Consider using `parking_lot::Mutex` for better performance (no poisoning)
- Add tracing to log environment changes for debugging
- Document that this blocks all threads trying to modify env concurrently
- Consider scope-based API: `with_env(vars, || { ... })` for safer usage

**Common Pitfalls:**
- ❌ Not using global lock (race conditions on `std::env` calls)
- ❌ Forgetting to restore environment on panic (leaks modified state)
- ❌ Holding lock too long (blocks all other env modifications)
- ❌ Not handling poisoned mutex (panics instead of recovering)
- ❌ Using thread-local storage for env vars (doesn't affect child code)

**Related Ecosystem Patterns:**
- **temp-env crate**: Similar temporary environment modification
- **serial_test crate**: Serialize tests that modify global state
- **ctor/dtor**: Global initialization/cleanup patterns
- **std::sync::Mutex for global state**: Common pattern in FFI wrappers

**Design Wisdom:**
This pattern demonstrates safe mutation of process-global state in multi-threaded code. The key insight: `std::env` is inherently process-global (affects all threads), so we need a process-global lock. The `_guard` field in the struct is elegant - holding the lock for the entire scope of `EnvChange` prevents TOCTOU bugs. The snapshot-restore pattern ensures clean state even after panics. This is textbook RAII for non-Rust resources.

---

## Pattern 12: Token Stream with Arc-Based Sharing and Copy-on-Write
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/token_stream.rs
**Category:** Data Structures, Memory Efficiency

**Code Example:**
```rust
#[derive(Clone)]
pub struct TokenStream<S>(pub(crate) Arc<Vec<TokenTree<S>>>);

impl<S: Copy> TokenStream<S> {
    /// Push `tt` onto the end of the stream, possibly gluing it to the last
    /// token. Uses `make_mut` to maximize efficiency.
    pub(crate) fn push_tree(&mut self, tt: TokenTree<S>) {
        let vec_mut = Arc::make_mut(&mut self.0);
        vec_mut.push(tt);
    }

    /// Push `stream` onto the end of the stream, possibly gluing the first
    /// token tree to the last token. (No other token trees will be glued.)
    /// Uses `make_mut` to maximize efficiency.
    pub(crate) fn push_stream(&mut self, stream: TokenStream<S>) {
        let vec_mut = Arc::make_mut(&mut self.0);
        let stream_iter = stream.0.iter().cloned();
        vec_mut.extend(stream_iter);
    }
}
```

**Why This Matters for Contributors:** Shows Arc-based copy-on-write for token streams. The type is cheap to clone (just increments refcount) but can be mutated efficiently via `Arc::make_mut` which only clones when refcount > 1. This is perfect for token streams which are frequently passed around but rarely modified. The pattern avoids expensive clones while maintaining value semantics. Important optimization for compiler-like workloads where data structures are pervasively shared.

---

### Expert Rust Commentary: Pattern 12

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Copy-on-Write Data Structures (L2 - Arc, std collections)
- **Complexity:** Intermediate
- **Rust Idioms:** Arc::make_mut for CoW, value semantics with shared ownership

**Rust-Specific Insights:**
1. **Arc::make_mut pattern**: Only clones when refcount > 1, giving copy-on-write semantics
2. **Arc<Vec<T>> vs Arc<[T]>**: Using `Vec` inside allows mutation while maintaining shared ownership
3. **Cheap cloning**: `TokenStream::clone()` is just an Arc refcount increment (O(1))
4. **Iterator chaining in push_stream**: `stream.0.iter().cloned()` extends efficiently
5. **Type constraint S: Copy**: Ensures span type is cheap to clone during iteration

**Contribution Tips:**
- Profile memory usage - Arc adds 16 bytes overhead per token stream
- Consider using `im::Vector` for persistent data structure (more CoW-friendly)
- Add `fn is_unique(&self) -> bool` to check if CoW will clone
- Document that `clone()` is cheap but `push_tree()` may trigger expensive clone
- Consider adding `fn into_vec(self) -> Vec<TokenTree<S>>` for owned conversion

**Common Pitfalls:**
- ❌ Using `Rc` instead of `Arc` in multi-threaded context (not Send/Sync)
- ❌ Not realizing `Arc::make_mut` can trigger expensive clone
- ❌ Cloning unnecessarily instead of borrowing (defeats CoW purpose)
- ❌ Using `Arc<Mutex<Vec<T>>>` when `Arc<Vec<T>>` with make_mut suffices
- ❌ Forgetting that S: Copy is required for efficient iteration

**Related Ecosystem Patterns:**
- **im crate**: Persistent data structures with structural sharing
- **Cow<'_, [T]>**: Standard library copy-on-write for slices
- **Arc vs Rc**: Thread-safe vs single-threaded reference counting
- **gc-arena**: Garbage collected arenas for cyclic structures

**Design Wisdom:**
This pattern optimizes for the common case: token streams are frequently passed around but rarely modified. `Arc<Vec<T>>` gives value semantics (cloning is cheap) while deferring actual copying until mutation via `make_mut`. This is perfect for compiler-like workloads where ASTs are pervasively shared. The pattern is more efficient than `Rc<RefCell<Vec<T>>>` (no runtime borrow checking) and safer than raw pointers.

**Performance Note:** The cost of CoW is amortized: many cheap clones, occasional expensive `make_mut`. Profile before optimizing - if you have many modifications, consider owned `Vec<T>` instead.

---

## Pattern 13: Custom Panic Markers for Error Classification
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/lib.rs
**Category:** Error Handling, Panic Recovery

**Code Example:**
```rust
#[derive(Debug)]
pub enum ProcMacroPanicMarker {
    Cancelled { reason: String },
    Internal { reason: String },
}

// In expansion code:
Err(payload) => {
    if let Some(marker) = payload.downcast_ref::<ProcMacroPanicMarker>() {
        return match marker {
            ProcMacroPanicMarker::Cancelled { reason } => {
                Err(ExpandError::Cancelled { reason: Some(reason.clone()) })
            }
            ProcMacroPanicMarker::Internal { reason } => {
                Err(ExpandError::Internal { reason: Some(reason.clone()) })
            }
        };
    }
    std::panic::resume_unwind(payload)
}

// In callback handling:
fn handle_failure(failure: Result<SubResponse, ProcMacroClientError>) -> ! {
    match failure {
        Err(ProcMacroClientError::Cancelled { reason }) => {
            resume_unwind(Box::new(ProcMacroPanicMarker::Cancelled { reason }));
        }
        Err(err) => {
            panic_any(ProcMacroPanicMarker::Internal {
                reason: format!("proc-macro IPC error: {err:?}"),
            });
        }
        Ok(other) => {
            panic_any(ProcMacroPanicMarker::Internal {
                reason: format!("unexpected SubResponse {other:?}"),
            });
        }
    }
}
```

**Why This Matters for Contributors:** Shows how to use typed panic payloads to communicate error semantics across panic boundaries. Instead of just panicking with a string, custom marker types allow recovery code to distinguish between user cancellation vs internal errors. The pattern uses `panic_any` to inject the marker, and `downcast_ref` to inspect it. This enables turning panics back into typed errors while letting unexpected panics propagate. Sophisticated error handling for cross-thread communication.

---

### Expert Rust Commentary: Pattern 13

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Typed Panic Payloads (L2 - std::panic, Any)
- **Complexity:** Advanced
- **Rust Idioms:** panic_any for typed panics, downcast_ref for recovery, resume_unwind

**Rust-Specific Insights:**
1. **Typed panic payloads**: Using custom enum instead of `String` allows recovery code to distinguish error types
2. **panic_any vs panic!**: `panic_any` allows panicking with any `Send + 'static` type, not just strings
3. **Downcast pattern**: `payload.downcast_ref::<T>()` safely checks if panic payload is our custom type
4. **Resume_unwind**: `std::panic::resume_unwind(payload)` re-raises panics we don't recognize (important for not swallowing unexpected panics)
5. **Panic as control flow**: Using panics to communicate across thread boundaries (not for errors, but for protocol-level cancellation)

**Contribution Tips:**
- Add more marker variants as new error classes emerge
- Consider making markers carry structured data (file/line info, macro name)
- Log all panics before converting to markers (debugging unknown panics)
- Document that this is for cross-thread communication, not regular error handling
- Add `#[derive(Debug)]` to improve panic messages

**Common Pitfalls:**
- ❌ Using this pattern for regular error handling (panics should be exceptional)
- ❌ Not calling `resume_unwind` for unknown panics (swallows bugs)
- ❌ Forgetting panic payloads must be `Send + 'static`
- ❌ Not catching panics at thread boundary (crashes process)
- ❌ Over-relying on downcast (brittle to type changes)

**Related Ecosystem Patterns:**
- **std::panic::catch_unwind**: Catching panics at boundaries
- **AssertUnwindSafe**: Marking types as panic-safe
- **PanicInfo**: Panic hook for custom panic handling
- **fehler crate**: Error handling that uses throws/catch

**Design Wisdom:**
This pattern uses panics as a control flow mechanism across thread boundaries - controversial but justified here. The alternative would be returning `Result` from every callback, but proc-macro bridge API uses panics for errors. By using typed panic payloads, we can distinguish "user cancelled operation" from "internal server error" after catching the panic. The downcast pattern is safer than string parsing.

**Important:** This pattern is appropriate for IPC boundaries where the other side uses panics. Don't use this in regular application code - prefer `Result`.

---

## Pattern 14: Protocol Message Versioning with Discriminant Ordering
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/legacy_protocol/msg.rs
**Category:** Protocol Evolution, Serialization

**Code Example:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    // IMPORTANT: Keep this first, otherwise postcard will break as its not a self describing format
    // As such, this is the only request that needs to be supported across all protocol versions
    // and by keeping it first, we ensure it always has the same discriminant encoding in postcard
    ApiVersionCheck {},

    /// Retrieves a list of macros from a given dynamic library.
    ListMacros { dylib_path: Utf8PathBuf },

    /// Expands a procedural macro.
    ExpandMacro(Box<ExpandMacro>),

    /// Sets server-specific configurations.
    SetConfig(ServerConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    // IMPORTANT: Keep this first, otherwise postcard will break as its not a self describing format
    ApiVersionCheck(u32),

    ListMacros(Result<Vec<(String, ProcMacroKind)>, String>),
    ExpandMacro(Result<FlatTree, PanicMessage>),
    SetConfig(ServerConfig),
    ExpandMacroExtended(Result<ExpandMacroExtended, PanicMessage>),
}
```

**Why This Matters for Contributors:** Critical pattern for protocol evolution with binary formats. Postcard (unlike JSON) uses discriminant values, so enum variant order matters. By keeping `ApiVersionCheck` first in both Request and Response, it gets discriminant 0, ensuring it can be decoded even by old/new clients. The comments explain the constraint clearly. This enables the version handshake to work even when enum variants are added/removed. Essential knowledge for wire protocol design.

---

### Expert Rust Commentary: Pattern 14

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Binary Protocol Evolution (L3 - serde, postcard)
- **Complexity:** Expert
- **Rust Idioms:** Discriminant ordering invariant, self-describing format awareness

**Rust-Specific Insights:**
1. **Discriminant ordering matters**: Postcard uses enum variant position as discriminant value, unlike JSON which uses names
2. **Version handshake must be stable**: By keeping `ApiVersionCheck` first (discriminant 0), it can be decoded by any version
3. **Explicit comment as invariant**: The IMPORTANT comment documents a constraint the compiler can't enforce
4. **Boxing large variants**: `ExpandMacro(Box<ExpandMacro>)` prevents enum size being dominated by one large variant
5. **Protocol evolution strategy**: New variants can be added at end without breaking old clients

**Contribution Tips:**
- NEVER reorder enum variants - it breaks wire compatibility
- Add new variants at the end only
- Consider using `#[repr(u32)]` to make discriminants explicit
- Add compile-time assertions to verify discriminant values
- Document which protocol versions support which variants

**Common Pitfalls:**
- ❌ Reordering variants (silently breaks protocol)
- ❌ Using `#[serde(rename)]` without understanding discriminant impact
- ❌ Not boxing large variants (wastes memory for small messages)
- ❌ Assuming postcard behaves like JSON (it doesn't - not self-describing)
- ❌ Removing old variants before all clients upgraded

**Related Ecosystem Patterns:**
- **bincode repr(u32)**: Explicit discriminant control
- **Cap'n Proto schema evolution**: Structural sharing with field IDs
- **protobuf field numbering**: Similar "never reorder" constraint
- **serde_repr crate**: Derive macros for explicit discriminants

**Design Wisdom:**
This pattern showcases a critical constraint of binary serialization: discriminant stability. Postcard is fast because it's not self-describing (no field names on wire), but this means enum variant order becomes part of the ABI. The `ApiVersionCheck` first pattern ensures version negotiation works even when sender and receiver have different enum definitions. The extensive comment prevents well-meaning refactoring from breaking the protocol.

**Protocol Evolution Best Practice:** Treat enum variant order as sacred. Never reorder, only append. Consider code review checklists to catch violations.

---

## Pattern 15: Parallel Dylib Loading with First-Worker Pattern
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/pool.rs
**Category:** Concurrency, Optimization

**Code Example:**
```rust
pub(crate) fn load_dylib(&self, dylib: &MacroDylib) -> Result<Vec<ProcMacro>, ServerError> {
    let _span = tracing::info_span!("ProcMacroServer::load_dylib").entered();

    let dylib_path = Arc::new(dylib.path.clone());
    let dylib_last_modified =
        std::fs::metadata(dylib_path.as_path()).ok().and_then(|m| m.modified().ok());

    let (first, rest) = self.workers.split_first().expect("worker pool must not be empty");

    let macros = first
        .find_proc_macros(&dylib.path)?
        .map_err(|e| ServerError { message: e, io: None })?;

    rest.into_par_iter()
        .map(|worker| {
            worker
                .find_proc_macros(&dylib.path)?
                .map(|_| ())
                .map_err(|e| ServerError { message: e, io: None })
        })
        .collect::<Result<(), _>>()?;

    Ok(macros
        .into_iter()
        .map(|(name, kind)| ProcMacro {
            pool: self.clone(),
            name: name.into(),
            kind,
            dylib_path: dylib_path.clone(),
            dylib_last_modified,
        })
        .collect())
}
```

**Why This Matters for Contributors:** Demonstrates efficient parallel loading: first worker loads the dylib and extracts macro metadata sequentially, while remaining workers load in parallel (using rayon's `par_iter`) but discard their results. This ensures all workers have the dylib loaded (for later expansion requests) while avoiding redundant metadata extraction. The pattern uses early-return error propagation with `?` even in parallel context. Smart optimization when you need side effects (dylib loading) but only one result set.

---

### Expert Rust Commentary: Pattern 15

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Parallel Side-Effect Loading (L3 - rayon)
- **Complexity:** Advanced
- **Rust Idioms:** Split first pattern, parallel iteration with side effects, early return error propagation

**Rust-Specific Insights:**
1. **Split first pattern**: `workers.split_first()` avoids index-based access and clearly separates first worker from rest
2. **Parallel side effects with rayon**: `rest.into_par_iter()` loads dylib in all workers concurrently but discards results (side effect: dylib loaded into process)
3. **Error propagation in parallel context**: `.collect::<Result<(), _>>()?` short-circuits on first error even in parallel iteration
4. **Arc for shared path**: `Arc::new(dylib.path.clone())` allows sharing path across all `ProcMacro` instances without copying
5. **Metadata caching**: First worker extracts metadata once, shared by all resulting `ProcMacro` instances

**Contribution Tips:**
- Profile whether parallel loading actually helps (might be I/O bound)
- Consider using `par_iter().for_each()` instead of `collect()` if you don't need error short-circuit
- Add telemetry to track dylib loading time per worker
- Handle case where some workers fail to load (partial availability)
- Consider pre-loading common dylibs at server startup

**Common Pitfalls:**
- ❌ Loading dylib serially in all workers (slow)
- ❌ Extracting metadata N times instead of once (wasteful)
- ❌ Not using `?` on parallel collect (silently ignores errors)
- ❌ Forgetting to clone path into Arc (each ProcMacro would own path)
- ❌ Not handling empty worker pool (split_first expects non-empty)

**Related Ecosystem Patterns:**
- **rayon par_iter**: Data parallelism patterns
- **Arc for shared ownership**: Common in parallel code
- **split_first/split_at**: Slice partitioning without indexing
- **try_collect**: Fallible iterator collection in parallel

**Design Wisdom:**
This pattern optimizes a common scenario: performing a side effect (loading dylib) across multiple workers while extracting metadata once. The key insight: loading is idempotent and can happen in parallel, but metadata extraction should happen once. The `split_first` pattern cleanly separates the "extract metadata" worker from the "just load" workers. Error handling is sophisticated - using `?` with parallel collect short-circuits on first error.

**Performance Note:** This assumes dylib loading is CPU/memory bound. If it's I/O bound, parallel loading might not help. Profile first.

---

## Pattern 16: Trait-Based Server Abstraction with Associated Types
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv/src/lib.rs
**Category:** Trait Design, Abstraction

**Code Example:**
```rust
pub trait ProcMacroSrvSpan: Copy + Send + Sync {
    type Server<'a>: rustc_proc_macro::bridge::server::Server<
            TokenStream = crate::token_stream::TokenStream<Self>,
        >;
    fn make_server<'a>(
        call_site: Self,
        def_site: Self,
        mixed_site: Self,
        callback: Option<ProcMacroClientHandle<'a>>,
    ) -> Self::Server<'a>;
}

impl ProcMacroSrvSpan for SpanId {
    type Server<'a> = server_impl::token_id::SpanIdServer<'a>;

    fn make_server<'a>(
        call_site: Self,
        def_site: Self,
        mixed_site: Self,
        callback: Option<ProcMacroClientHandle<'a>>,
    ) -> Self::Server<'a> {
        Self::Server { call_site, def_site, mixed_site, callback, /* ... */ }
    }
}

impl ProcMacroSrvSpan for Span {
    type Server<'a> = server_impl::rust_analyzer_span::RaSpanServer<'a>;
    fn make_server<'a>(/* ... */) -> Self::Server<'a> { /* ... */ }
}
```

**Why This Matters for Contributors:** Shows how to abstract over different span representations using associated types. The trait allows the proc-macro server to work with either simple span IDs or full rust-analyzer spans without code duplication. The associated type `Server` is constrained to implement rustc's bridge API, ensuring type safety. The factory method `make_server` constructs the appropriate server type. This pattern enables compile-time specialization of generic code based on span type.

---

### Expert Rust Commentary: Pattern 16

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Trait-Based Abstraction with GATs (L2 - trait, associated types)
- **Complexity:** Advanced
- **Rust Idioms:** Associated types, generic associated types (GATs), trait bounds on associated types

**Rust-Specific Insights:**
1. **GATs for higher-kinded abstraction**: `type Server<'a>` allows the associated type to be parameterized by lifetime
2. **Trait bounds on associated types**: `where Server<'a>: rustc_proc_macro::bridge::server::Server` enforces constraints at trait level
3. **Factory method pattern**: `make_server()` constructs the appropriate server implementation for each span type
4. **Copy + Send + Sync bounds**: Ensures span type can be used safely across threads and copied cheaply
5. **Compile-time dispatch**: Different span types get different server implementations without runtime overhead

**Contribution Tips:**
- When adding new span types, implement this trait for compile-time specialization
- Document which server implementation is appropriate for which use case
- Consider adding `type Client<'a>` associated type if client-side needs similar abstraction
- Add trait methods for span conversion/validation
- Use `#[diagnostic::on_unimplemented]` for better error messages

**Common Pitfalls:**
- ❌ Using `Box<dyn Trait>` instead of associated types (runtime overhead)
- ❌ Not constraining associated Server type (breaks at call sites)
- ❌ Forgetting GAT syntax `type Server<'a>` vs `type Server`
- ❌ Not requiring Copy bound when span needs to be cloned frequently
- ❌ Over-abstracting when only two implementations exist

**Related Ecosystem Patterns:**
- **tower Service trait**: Similar associated type pattern for request/response
- **futures Stream trait**: Associated `Item` type for async iteration
- **Iterator trait**: Classic associated type example
- **rustc trait solver**: Heavy use of associated types for type inference

**Design Wisdom:**
This pattern demonstrates trait-based abstraction over different span representations. The GAT (`Server<'a>`) allows the server type to borrow data with arbitrary lifetime. This enables zero-cost abstraction - each span type gets its own specialized server implementation, chosen at compile time. The factory method pattern (`make_server`) encapsulates construction logic. This is more flexible than enum dispatch and faster than trait objects.

**Advanced Pattern:** This is textbook use of associated types for compile-time specialization - prefer this over runtime dispatch when implementations are known statically.

---

## Pattern 17: OnceLock for Deferred Error Storage
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/process.rs
**Category:** Lazy Initialization, Error Handling

**Code Example:**
```rust
pub(crate) struct ProcMacroServerProcess {
    state: Mutex<ProcessSrvState>,
    version: u32,
    protocol: Protocol,
    /// Populated when the server exits.
    exited: OnceLock<AssertUnwindSafe<ServerError>>,
    active: AtomicU32,
}

impl ProcMacroServerProcess {
    pub(crate) fn exited(&self) -> Option<&ServerError> {
        self.exited.get().map(|it| &it.0)
    }

    fn with_locked_io<R, B>(
        &self,
        mut buf: B,
        f: impl FnOnce(&mut dyn Write, &mut dyn BufRead, &mut B) -> Result<R, ServerError>,
    ) -> Result<R, ServerError> {
        let state = &mut *self.state.lock().unwrap();
        f(&mut state.stdin, &mut state.stdout, &mut buf).map_err(|e| {
            if e.io.as_ref().map(|it| it.kind()) == Some(io::ErrorKind::BrokenPipe) {
                match state.process.exit_err() {
                    None => e,
                    Some(server_error) => {
                        self.exited.get_or_init(|| AssertUnwindSafe(server_error)).0.clone()
                    }
                }
            } else {
                e
            }
        })
    }
}
```

**Why This Matters for Contributors:** Uses `OnceLock` to store the error from a process exit exactly once, even if multiple threads detect it. The `get_or_init` call is atomic - only the first thread stores the error, others get the same reference. Wrapped in `AssertUnwindSafe` to allow Clone (ServerError implements Clone but is not UnwindSafe). Pattern is perfect for caching an error that can be observed from multiple code paths but only happens once. More efficient than Mutex for this use case.

---

### Expert Rust Commentary: Pattern 17

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Lazy Error Caching (L2 - OnceLock, AtomicU32)
- **Complexity:** Intermediate
- **Rust Idioms:** OnceLock for one-time initialization, AssertUnwindSafe wrapper

**Rust-Specific Insights:**
1. **OnceLock for exactly-once storage**: Multiple threads can call `get_or_init`, but only first stores the value
2. **AssertUnwindSafe wrapper**: Wrapping ServerError in AssertUnwindSafe allows it to be stored in OnceLock despite not implementing UnwindSafe
3. **Atomic get pattern**: `exited.get().map(|it| &it.0)` unwraps AssertUnwindSafe and returns reference to inner error
4. **Error caching at detection point**: When BrokenPipe detected, immediately cache the process exit error
5. **Clone for error propagation**: ServerError is Clone, allowing multiple readers to get their own copy

**Contribution Tips:**
- Add `fn has_exited(&self) -> bool` for checking without accessing error
- Consider adding exit status/signal info to ServerError
- Log when first thread stores the exit error (debugging race conditions)
- Document that `get()` returns `None` until first error occurs
- Consider using `std::sync::OnceLock::get_or_try_init` if initialization can fail

**Common Pitfalls:**
- ❌ Using Mutex<Option<T>> instead of OnceLock (unnecessary locking overhead)
- ❌ Not using AssertUnwindSafe (compile error with non-UnwindSafe types)
- ❌ Calling `set()` instead of `get_or_init()` (panics if already set)
- ❌ Not handling poisoned mutex case elsewhere in the code
- ❌ Forgetting that OnceLock is never cleared (process stays exited)

**Related Ecosystem Patterns:**
- **once_cell crate**: Similar lazy initialization (OnceLock is std version as of Rust 1.70)
- **lazy_static macro**: Compile-time initialized statics
- **std::sync::Once**: Lower-level synchronization primitive
- **tokio::sync::OnceCell**: Async version of OnceLock

**Design Wisdom:**
This pattern elegantly solves "multiple threads detect same error" problem. Using OnceLock ensures only first thread stores the error, avoiding races. The AssertUnwindSafe wrapper is interesting - it asserts "we've audited this type and it's safe to use across unwind boundaries despite not implementing UnwindSafe". The pattern is more efficient than `Mutex<Option<ServerError>>` because `get()` doesn't acquire a lock.

**When to Use:** Perfect for caching errors that can only happen once (process exit, file corruption detection, etc.). Don't use for errors that can occur multiple times.

---

## Pattern 18: Recursion Limit Disabling for Deep Structures
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/transport/json.rs
**Category:** Serialization, Workaround

**Code Example:**
```rust
pub(crate) fn decode<T: DeserializeOwned>(buf: &mut str) -> io::Result<T> {
    let mut deserializer = serde_json::Deserializer::from_str(buf);
    // Note that some proc-macro generate very deep syntax tree
    // We have to disable the current limit of serde here
    deserializer.disable_recursion_limit();
    Ok(T::deserialize(&mut deserializer)?)
}
```

**Why This Matters for Contributors:** Shows how to handle pathological cases in deserialization. Serde's default recursion limit (128) prevents stack overflow, but proc-macro token trees can legitimately be deeper (especially for macro_rules generated code). The pattern explicitly disables the limit with a comment explaining why. Important decision: trading stack safety for correctness in this specific domain. Contributors should understand when it's appropriate to disable safety limits and document the reasoning.

---

### Expert Rust Commentary: Pattern 18

**Idiomatic Rating: ⭐⭐⭐⭐ (4/5)**

**Pattern Classification:**
- **Category:** Security vs Correctness Trade-off (L3 - serde_json)
- **Complexity:** Intermediate
- **Rust Idioms:** Explicit safety limit override with documentation

**Rust-Specific Insights:**
1. **Explicit limit disabling**: `deserializer.disable_recursion_limit()` is opt-in, not default - requires conscious decision
2. **Domain-specific correctness**: Comment explains why deep nesting is legitimate (macro_rules! expansions)
3. **Risk documentation**: Pattern acknowledges the stack overflow risk but accepts it for correctness
4. **Deserializer customization**: Shows how to configure serde_json deserializer beyond derive macro
5. **Trade-off visibility**: Code location makes the security/correctness trade-off obvious to reviewers

**Contribution Tips:**
- Add stack overflow handling at a higher layer (thread stack size, as seen in Pattern 10)
- Consider adding optional depth limiting via configuration
- Log warning when deserializing extremely deep structures (>1000 levels)
- Document maximum observed depth in production (helps validate decision)
- Consider adding telemetry to track recursion depth distribution

**Common Pitfalls:**
- ❌ Disabling limit without documenting why (future maintainers will question it)
- ❌ Not having any stack overflow protection (this pattern relies on 8MB stack from Pattern 10)
- ❌ Using this pattern for untrusted input without other safeguards
- ❌ Forgetting that JSON can have circular references via object reuse
- ❌ Not testing with deeply nested input (validate the decision)

**Related Ecosystem Patterns:**
- **serde recursion_limit attribute**: Setting limit at type level
- **serde_stacker**: Automatic stack growth for deep structures
- **Stack size configuration**: Thread stack size control
- **streaming parsers**: Alternative for extremely deep data

**Design Wisdom:**
This pattern demonstrates thoughtful violation of a safety limit. Serde's recursion limit (128) prevents stack overflow from malicious or malformed input. But in the proc-macro domain, deeply nested token trees are legitimate - macro_rules! can generate 100+ levels easily. The key is documenting WHY the limit is disabled and having compensating controls (8MB thread stack from Pattern 10).

**Rating Rationale:** Deducted one star because this pattern relies on external stack size control for safety. Ideally, would use `serde_stacker` for automatic stack growth instead of disabling limit entirely.

**Alternative Approach:** Consider using `serde_stacker::Deserializer::new()` which automatically grows stack instead of disabling limit.

---

## Pattern 19: Active Request Tracking with Atomic Counters
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/process.rs
**Category:** Concurrency, Metrics

**Code Example:**
```rust
pub(crate) struct ProcMacroServerProcess {
    state: Mutex<ProcessSrvState>,
    version: u32,
    protocol: Protocol,
    exited: OnceLock<AssertUnwindSafe<ServerError>>,
    active: AtomicU32,
}

pub(crate) fn expand(
    &self,
    proc_macro: &ProcMacro,
    // ... other params
) -> Result<Result<tt::TopSubtree, String>, ServerError> {
    self.active.fetch_add(1, Ordering::AcqRel);
    let result = match self.protocol {
        Protocol::LegacyJson { .. } => legacy_protocol::expand(/* ... */),
        Protocol::BidirectionalPostcardPrototype { .. } => bidirectional_protocol::expand(/* ... */),
    };

    self.active.fetch_sub(1, Ordering::AcqRel);
    result
}

pub(crate) fn number_of_active_req(&self) -> u32 {
    self.active.load(Ordering::Acquire)
}
```

**Why This Matters for Contributors:** Shows lock-free request tracking for load balancing. The atomic counter tracks in-flight requests without blocking. `AcqRel` ordering on increment/decrement ensures the counter updates are visible to other threads reading with `Acquire`. This enables the pool to pick the least-loaded worker without locks. The RAII-like increment/decrement pattern (with decrement even on error path) ensures count stays accurate. Essential for performant concurrent systems.

---

### Expert Rust Commentary: Pattern 19

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Lock-Free Metrics (L2 - atomic types, memory ordering)
- **Complexity:** Intermediate
- **Rust Idioms:** Atomic counters with proper ordering, RAII-like increment/decrement

**Rust-Specific Insights:**
1. **AcqRel ordering**: `fetch_add(1, AcqRel)` ensures counter updates are visible to other threads reading with Acquire
2. **Acquire ordering for reads**: `load(Acquire)` ensures reading most recent value written by other threads
3. **RAII pattern without Drop**: Increment on entry, decrement on exit - works even if early return or error
4. **Lock-free load balancing**: Reading counter doesn't block, enabling non-blocking worker selection
5. **Relaxed ordering would be wrong**: Need synchronization to ensure worker selection sees latest counts

**Contribution Tips:**
- Consider using RAII guard: `struct ActiveGuard<'a>(&'a AtomicU32)` with Drop impl
- Add assertion that counter doesn't underflow (debug mode only)
- Profile if AcqRel overhead matters (likely not, but measure)
- Add metrics for peak concurrency (track max value seen)
- Consider using `fetch_update` if implementing more complex logic

**Common Pitfalls:**
- ❌ Using Relaxed ordering (counter reads may be stale, breaking load balancing)
- ❌ Not decrementing on error path (counter leaks, affects load balancing)
- ❌ Using Mutex for counter (unnecessary synchronization overhead)
- ❌ Incrementing/decrementing with SeqCst (stronger than needed, slower)
- ❌ Reading counter with Relaxed (may see stale value)

**Related Ecosystem Patterns:**
- **AtomicUsize for counters**: Common pattern in concurrent code
- **metrics crate**: Production-grade metrics collection
- **crossbeam::utils::Backoff**: For more sophisticated atomic patterns
- **tower::load::PeakEwma**: Exponentially weighted load tracking

**Design Wisdom:**
This pattern demonstrates correct use of atomics for metrics. The key insight: use `AcqRel` for modifications and `Acquire` for reads to ensure visibility across threads. This enables lock-free load balancing - workers can read each other's counters without blocking. The manual increment/decrement pattern is error-prone (easy to forget decrement), so an RAII guard would be better.

**Memory Ordering Explained:**
- `AcqRel` on `fetch_add/sub`: Establishes happens-before relationship with other threads
- `Acquire` on `load`: Ensures seeing most recent write from any thread
- Don't use `Relaxed` here - it breaks load balancing correctness

**Improvement:** Wrap in RAII guard to prevent forgetting decrement on error paths.

---

## Pattern 20: Serde Alias for Backward Compatibility
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-api/src/lib.rs
**Category:** API Evolution, Serialization

**Code Example:**
```rust
#[derive(Copy, Clone, Eq, PartialEq, Debug, serde_derive::Serialize, serde_derive::Deserialize)]
pub enum ProcMacroKind {
    CustomDerive,
    Attr,
    // This used to be called FuncLike, so that's what the server expects currently.
    #[serde(alias = "Bang")]
    #[serde(rename(serialize = "FuncLike", deserialize = "FuncLike"))]
    Bang,
}
```

**Why This Matters for Contributors:** Demonstrates graceful protocol evolution via serde attributes. The variant serializes as "FuncLike" (old name) but accepts both "FuncLike" and "Bang" (new name) during deserialization. This allows clients to be updated before servers without breaking compatibility. The comment explains the historical reason. Pattern enables smooth transitions when renaming protocol types. Essential for distributed systems that can't do atomic upgrades.

---

### Expert Rust Commentary: Pattern 20

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** API Evolution with Serde (L3 - serde attributes)
- **Complexity:** Intermediate
- **Rust Idioms:** Serde alias for migration, rename for backward compatibility

**Rust-Specific Insights:**
1. **Serde alias attribute**: `#[serde(alias = "Bang")]` accepts new name during deserialization
2. **Rename for serialization**: `#[serde(rename(serialize = "FuncLike", ...))]` emits old name when sending
3. **Asymmetric compatibility**: Can deserialize both old and new names, but always serializes as old name
4. **Comment as documentation**: Explains why rename exists (historical compatibility)
5. **Non-breaking API change**: Clients can upgrade before servers, or vice versa

**Contribution Tips:**
- Document which version introduced the rename (helps with deprecation planning)
- Add deprecation timeline in comments ("remove FuncLike alias after 2024-Q4")
- Consider adding metrics to track which name is seen in deserialization
- Test with both old and new payloads to validate compatibility
- Use this pattern for any protocol field/variant renames

**Common Pitfalls:**
- ❌ Renaming without alias (breaks old clients)
- ❌ Using alias on serialize side (always sends new name, breaks old servers)
- ❌ Not documenting why alias exists (confusion during cleanup)
- ❌ Removing alias too early (client/server upgrade races)
- ❌ Using `#[serde(rename = "...")]` alone (doesn't provide two-way compat)

**Related Ecosystem Patterns:**
- **serde field aliases**: Similar pattern for struct field renames
- **deprecated attribute**: Rust-level deprecation warnings
- **semver-compatible evolution**: General API evolution strategies
- **Protocol Buffers field numbering**: Alternative approach (never rename, use IDs)

**Design Wisdom:**
This pattern enables smooth protocol evolution. The key insight: during migration, you need asymmetric compatibility - deserialize both names (for rollback), but serialize only old name (for compatibility). This allows rolling upgrades: new clients can talk to old servers (send "FuncLike"), old clients can talk to new servers (receive "FuncLike", even though variant is now called "Bang").

**Migration Strategy:**
1. Add alias, deploy everywhere (both names accepted)
2. Switch code to use new name internally
3. After all clients/servers upgraded, flip serialization to new name
4. Much later, remove alias

**Excellent Example:** This pattern is textbook protocol evolution. Should be standard practice for any distributed system renames.

---

## Pattern 21: Main Loop with Protocol Dispatch
**File:** /Users/amuldotexe/Desktop/Pre20260126/OSS202601/Notes2026/rust-analyzer-patterns/crates/proc-macro-srv-cli/src/main_loop.rs
**Category:** Server Architecture, Protocol Handling

**Code Example:**
```rust
pub fn run(
    stdin: &mut (dyn BufRead + Send + Sync),
    stdout: &mut (dyn Write + Send + Sync),
    format: ProtocolFormat,
) -> io::Result<()> {
    match format {
        ProtocolFormat::JsonLegacy => run_old(stdin, stdout),
        ProtocolFormat::BidirectionalPostcardPrototype => run_new(stdin, stdout),
    }
}

fn run_new(
    stdin: &mut (dyn BufRead + Send + Sync),
    stdout: &mut (dyn Write + Send + Sync),
) -> io::Result<()> {
    let mut buf = Vec::default();
    let env_snapshot = EnvSnapshot::default();
    let srv = proc_macro_srv::ProcMacroSrv::new(&env_snapshot);
    let mut span_mode = legacy::SpanMode::Id;

    'outer: loop {
        let req_opt = bidirectional::BidirectionalMessage::read(stdin, &mut buf)?;
        let Some(req) = req_opt else {
            break 'outer;
        };

        match req {
            bidirectional::BidirectionalMessage::Request(request) => match request {
                bidirectional::Request::ListMacros { dylib_path } => {
                    let res = srv.list_macros(&dylib_path).map(/* ... */);
                    send_response(stdout, bidirectional::Response::ListMacros(res))?;
                }
                bidirectional::Request::ApiVersionCheck {} => {
                    send_response(stdout, bidirectional::Response::ApiVersionCheck(CURRENT_API_VERSION))?;
                }
                bidirectional::Request::SetConfig(config) => {
                    span_mode = config.span_mode;
                    send_response(stdout, bidirectional::Response::SetConfig(config))?;
                }
                bidirectional::Request::ExpandMacro(task) => {
                    handle_expand(&srv, stdin, stdout, &mut buf, span_mode, *task)?;
                }
            },
            _ => continue,
        }
    }
    Ok(())
}
```

**Why This Matters for Contributors:** Shows clean server architecture with protocol dispatch. The main loop reads messages, dispatches to handlers, and sends responses. Configuration (like `span_mode`) is stateful across requests. The loop exits cleanly on EOF. Error handling is simple - `?` propagates IO errors up. Pattern is the foundation of request/response servers. The bidirectional variant handles nested sub-requests within the expand handler, showing protocol layering.

---

### Expert Rust Commentary: Pattern 21

**Idiomatic Rating: ⭐⭐⭐⭐⭐ (5/5)**

**Pattern Classification:**
- **Category:** Server Main Loop (L2 - std I/O, match-based dispatch)
- **Complexity:** Intermediate
- **Rust Idioms:** Trait object I/O, match-based dispatch, stateful loop, EOF detection

**Rust-Specific Insights:**
1. **Trait object I/O**: `&mut (dyn BufRead + Send + Sync)` allows different I/O sources (stdin, socket, test mock)
2. **Protocol dispatch**: `match format` switches between JSON and binary protocol implementations
3. **Let-else for EOF**: `let Some(req) = req_opt else { break; }` cleanly handles stream end
4. **Stateful server**: `span_mode` persists across requests - configuration affects subsequent operations
5. **Labeled break**: `'outer: loop` allows clean exit from nested match
6. **Helper function extraction**: `handle_expand` separates complex logic from dispatch loop

**Contribution Tips:**
- Add request ID to trace each request end-to-end
- Consider adding timeout on read (detect hung clients)
- Add graceful shutdown signal handling (not just EOF)
- Extract dispatch logic to handler trait for testability
- Add metrics for request type distribution and latency

**Common Pitfalls:**
- ❌ Not handling EOF explicitly (infinite loop on client disconnect)
- ❌ Panicking on malformed requests (brings down server)
- ❌ Not resetting state between requests (state leaks between clients)
- ❌ Blocking on I/O without timeout (hung client hangs server)
- ❌ Using unwrap() in main loop (crashes on any error)

**Related Ecosystem Patterns:**
- **tower Service trait**: More sophisticated request/response abstraction
- **actix Actor pattern**: Message-based server architecture
- **JSON-RPC servers**: Similar request/response dispatch
- **Language Server Protocol**: Similar stateful message loop

**Design Wisdom:**
This pattern demonstrates clean server architecture: thin dispatch loop delegating to handlers. The stateful design (`span_mode` configuration) is appropriate - some settings affect multiple requests. The labeled break pattern cleanly exits nested control flow. The use of trait objects for I/O enables testing with in-memory buffers.

**Testing Strategy:** Pass `Cursor<Vec<u8>>` for stdin and `Vec<u8>` for stdout in tests - no actual I/O needed.

---

## Summary: Proc Macro Server Patterns

### Pattern Categories

**IPC & Protocol Design (Patterns 2, 7, 8, 9, 14, 20, 21):**
These patterns showcase sophisticated IPC protocol design: version negotiation with fallback, defensive parsing of misbehaving output, binary protocol evolution, bidirectional RPC with callbacks, and graceful API migration. The progression from JSON to COBS-encoded postcard shows performance optimization while maintaining robustness.

**Dynamic Loading & FFI (Patterns 3, 4, 5, 6):**
Critical patterns for safe dynamic library loading: multi-stage validation before loading, platform-specific symbol isolation, Windows file locking workarounds, and binary metadata parsing. These demonstrate defense-in-depth for unsafe code.

**Concurrency & Thread Safety (Patterns 1, 10, 11, 15, 19):**
Showcase lock-free load balancing, thread sandboxing for untrusted code, global state mutation with locks, parallel initialization with side effects, and atomic counters for metrics. Pattern 11's global lock for environment variables is particularly instructive.

**Error Handling & Recovery (Patterns 13, 17, 18):**
Demonstrate typed panic payloads for cross-thread error classification, lazy error caching with OnceLock, and conscious violation of safety limits with compensation.

**Data Structures (Pattern 12, 16):**
Arc-based copy-on-write for efficient sharing, and trait-based abstraction with GATs for compile-time specialization.

### Key Rust Idioms Demonstrated

1. **Arc<[T]> for immutable shared slices** - Signals ownership model at type level
2. **Arc::make_mut for CoW** - Efficient copy-on-write semantics
3. **OnceLock for one-time initialization** - Lock-free lazy storage
4. **Atomic counters with proper ordering** - AcqRel/Acquire for visibility
5. **Platform-specific cfg** - Unified interface, platform-specific implementation
6. **Scoped threads** - Borrow from parent without 'static
7. **GATs for higher-kinded abstraction** - Compile-time specialization
8. **Panic as control flow** - Typed panics across thread boundaries
9. **RAII for global state** - Lock guard as struct field
10. **Defensive parsing** - Validate before acting, clear error messages

### Contribution Readiness Checklist

**Before Contributing to rust-analyzer Proc-Macro Subsystem:**

- [ ] **Understand IPC robustness requirements** - Other end of pipe may print debug output, panic, or send malformed data
- [ ] **Know platform differences** - Test on Windows (file locking), Linux (RTLD_DEEPBIND), macOS (no DEEPBIND)
- [ ] **Profile before optimizing protocols** - Measure if JSON is actually a bottleneck before switching to binary
- [ ] **Respect discriminant ordering** - Never reorder enum variants in binary protocols
- [ ] **Use proper atomic ordering** - Understand AcqRel/Acquire/Release semantics
- [ ] **Document unsafe code thoroughly** - SAFETY comments for every unsafe block
- [ ] **Test with pathological inputs** - Deeply nested syntax, huge token streams, malformed messages
- [ ] **Handle panics at boundaries** - Catch panics from proc-macro code, convert to errors
- [ ] **Validate before loading dylibs** - Parse metadata, check version BEFORE loading into address space
- [ ] **Add telemetry** - Instrument protocol selection, error rates, expansion times
- [ ] **Test protocol evolution** - Verify old clients talk to new servers and vice versa
- [ ] **Clean up temporary files** - Windows DLL copies accumulate over time
- [ ] **Handle process exit gracefully** - OnceLock for caching exit errors
- [ ] **Use RAII for cleanup** - Environment restoration, lock guards, stack guards
- [ ] **Consider adding RAII guards** - Especially for atomic counter increment/decrement patterns

### Advanced Patterns to Study

1. **Pattern 9 (Bidirectional RPC)** - Most complex: nested protocol with callbacks
2. **Pattern 14 (Discriminant ordering)** - Most subtle: binary protocol evolution pitfall
3. **Pattern 11 (Environment lock)** - Most instructive: safe global state mutation
4. **Pattern 3 (Dylib loading)** - Most safety-critical: multi-stage validation
5. **Pattern 13 (Panic markers)** - Most controversial: using panics for control flow

### Production Readiness

These patterns are production-grade and battle-tested in rust-analyzer, which is used by thousands of developers daily. They demonstrate:
- Robustness to malformed input
- Platform portability with graceful degradation
- Performance optimization without sacrificing safety
- Clear error messages for debugging
- Backward compatibility during upgrades

### Learning Path Recommendation

1. **Start with Pattern 1** (worker pool) - Foundational load balancing
2. **Progress to Pattern 7** (defensive IPC) - Robustness patterns
3. **Study Pattern 10** (thread sandboxing) - Complete isolation example
4. **Master Pattern 3** (dylib loading) - Safety in unsafe code
5. **Tackle Pattern 9** (bidirectional RPC) - Advanced protocol design

### When to Apply These Patterns

- **Worker pools** - When load balancing across processes/threads
- **IPC protocols** - When designing message formats between processes
- **Dynamic loading** - When loading plugins, proc-macros, or native libraries
- **Thread isolation** - When running untrusted code safely
- **Protocol evolution** - When maintaining backward compatibility

These patterns represent years of refinement in a production IDE. Study them carefully - they solve real problems that emerge at scale.

---
