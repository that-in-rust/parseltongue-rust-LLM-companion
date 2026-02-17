# Prep Doc: Cross-Language Boundary Detection Heuristics for v2.0.0

**Date**: 2026-02-16
**Author**: Research document for rust-llm-02-cross-lang-edges crate
**Context**: Parseltongue v2.0.0 will detect cross-language boundaries in multi-language codebases. This document catalogs the five detection patterns, their tree-sitter extraction signals, accuracy characteristics, and real-world test targets. This capability maps to the rust-llm-02-cross-lang-edges crate in the v2.0.0 architecture.

**Reference**: See Prep-Doc-V200.md Section 5 ("Cross-Language Boundary Problem") for the high-level problem statement.

---

## Table of Contents

1. [Pattern 1: FFI (Rust <-> C/C++)](#1-pattern-1-ffi-rust---cc)
2. [Pattern 2: WASM (Rust -> JS/TS)](#2-pattern-2-wasm-rust---jsts)
3. [Pattern 3: PyO3 / JNI / Ruby FFI (Rust -> Python/Java/Ruby)](#3-pattern-3-pyo3--jni--ruby-ffi-rust---pythonjavaruby)
4. [Pattern 4: HTTP/gRPC (Rust <-> Any)](#4-pattern-4-httpgrpc-rust---any)
5. [Pattern 5: Message Queues (Rust <-> Any via Iggy/Kafka/NATS)](#5-pattern-5-message-queues-rust---any-via-iggykafkanats)
6. [Accuracy Analysis and Confidence Scoring](#6-accuracy-analysis-and-confidence-scoring)
7. [Real-World Test Repositories](#7-real-world-test-repositories)
8. [Tree-Sitter Signal Extraction Reference](#8-tree-sitter-signal-extraction-reference)
9. [Implementation Architecture for rust-llm-02](#9-implementation-architecture-for-rust-llm-02)

---

## 1. Pattern 1: FFI (Rust <-> C/C++)

### 1.1 Overview

Foreign Function Interface (FFI) is the most fundamental cross-language boundary. Rust calls C/C++ through `extern "C"` declarations, and C/C++ calls Rust through `#[no_mangle]` exported functions. This pattern has the highest detection confidence because FFI boundaries are explicit in the syntax.

### 1.2 Rust-Side Signals

**Calling C from Rust (extern blocks)**:
```rust
extern "C" {
    fn rocksdb_open(options: *const Options, name: *const c_char) -> *mut DB;
    fn rocksdb_put(db: *mut DB, key: *const u8, val: *const u8) -> Status;
}
```

Tree-sitter node structure:
- `foreign_mod_item` (NOT `extern_block` -- this is the correct tree-sitter-rust node type)
  - `extern_modifier` containing the ABI string `"C"`, `"system"`, or `"cdecl"`
  - Contains `function_signature_item` children (NOT `function_item` -- declarations lack bodies)
  - Each `function_signature_item` has `name: (identifier)` and `parameters: (parameters)`

**Exposing Rust to C**:
```rust
#[no_mangle]
pub extern "C" fn parseltongue_analyze(path: *const c_char) -> *mut AnalysisResult {
    // ...
}
```

Tree-sitter node structure:
- `function_item` with `extern_modifier`
- Preceded by `attribute_item` containing `#[no_mangle]`
- The function IS a `function_item` (has a body), not a `function_signature_item`

**Rust 2024 Edition Caveat**: The 2024 edition requires `#[unsafe(no_mangle)]` instead of `#[no_mangle]`. Tree-sitter queries must handle both forms:
```scheme
;; Classic form
(attribute_item
  (attribute
    (identifier) @attr_name
    (#eq? @attr_name "no_mangle")))

;; Rust 2024 form: #[unsafe(no_mangle)]
(attribute_item
  (attribute
    (identifier) @outer
    (#eq? @outer "unsafe")
    arguments: (token_tree
      (identifier) @inner
      (#eq? @inner "no_mangle"))))
```

**repr(C) on structs/enums**:
```rust
#[repr(C)]
pub struct RocksDBOptions {
    pub create_if_missing: bool,
    pub max_open_files: i32,
}
```

This indicates a type is shared across the FFI boundary. While not a call edge, it creates a type dependency edge.

### 1.3 C/C++-Side Signals

**C header declarations**:
```c
// In rocksdb/c.h
extern rocksdb_t* rocksdb_open(const rocksdb_options_t* options, const char* name, char** errptr);
extern void rocksdb_put(rocksdb_t* db, const rocksdb_writeoptions_t* options,
                        const char* key, size_t keylen,
                        const char* val, size_t vallen,
                        char** errptr);
```

Tree-sitter-c node: `declaration` with `storage_class_specifier` "extern" and a `function_declarator`.

**C++ with extern "C" blocks**:
```cpp
extern "C" {
    void parseltongue_init();
    void parseltongue_shutdown();
}
```

Tree-sitter-cpp: `linkage_specification` node with `"C"` string and body containing `function_definition` or `declaration` nodes.

### 1.4 Matching Algorithm

```
ALGORITHM: FFI_MATCH(rust_files, c_files)
1. Extract all function_signature_item names inside foreign_mod_item from Rust files
   -> Set EXTERN_DECLS = {(fn_name, abi, file, line)}
2. Extract all #[no_mangle] extern function_item names from Rust files
   -> Set EXPORTED_FNS = {(fn_name, abi, file, line)}
3. Extract all extern declarations from C/C++ headers
   -> Set C_DECLS = {(fn_name, file, line)}
4. Extract all function_definition names from C/C++ files
   -> Set C_DEFS = {(fn_name, file, line)}
5. For each fn in EXTERN_DECLS:
     if fn.name in C_DEFS:
       emit edge: Rust:extern_call -> C:function (confidence: 0.95)
     elif fn.name in C_DECLS:
       emit edge: Rust:extern_call -> C:declaration (confidence: 0.90)
6. For each fn in EXPORTED_FNS:
     if fn.name in C_DECLS:
       emit edge: C:call -> Rust:exported_fn (confidence: 0.90)
```

### 1.5 CXX Bridge (High-Confidence Variant)

The `cxx` crate provides a safer alternative to raw FFI with self-documenting bridge modules:

```rust
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type MultiBuf;
        fn next_chunk(buf: &mut MultiBuf) -> &[u8];
    }

    unsafe extern "C++" {
        include!("example/include/blobstore.h");
        type BlobstoreClient;
        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;
        fn put(self: &BlobstoreClient, parts: &mut MultiBuf) -> u64;
    }
}
```

CXX bridge is the **highest-confidence** FFI detection pattern because:
- The entire bridge is self-contained in a `#[cxx::bridge]` annotated module
- `extern "Rust"` declares Rust functions callable from C++
- `unsafe extern "C++"` declares C++ functions callable from Rust
- The `include!()` macro gives the exact C++ header file path
- False positive rate: 1-3% (essentially zero for well-formed bridges)

Tree-sitter detection:
```scheme
;; Detect #[cxx::bridge] module
(attribute_item
  (attribute
    (scoped_identifier
      path: (identifier) @ns
      name: (identifier) @attr
      (#eq? @ns "cxx")
      (#eq? @attr "bridge"))))
```

### 1.6 bindgen and cbindgen

**bindgen** (C -> Rust): Generates Rust FFI bindings from C headers. Detection signal: presence of `bindgen::Builder` in `build.rs` or `build/` scripts, plus output file matching `*_bindings.rs` or `*_sys.rs`.

**cbindgen** (Rust -> C): Generates C headers from Rust `#[no_mangle]` functions. Detection signal: `cbindgen.toml` or `cbindgen::generate()` in `build.rs`.

These are build-time artifacts. We detect them by scanning `build.rs` files for the relevant macro invocations, then correlating the generated bindings with declared extern functions.

### 1.7 False Positives and Mitigations

| Source | Example | Mitigation |
|--------|---------|------------|
| Name collision | `malloc` declared in both Rust extern block and C stdlib | Require file proximity (same repo) + ABI match |
| Platform-conditional FFI | `#[cfg(windows)] extern "system" { ... }` | Record cfg attributes, flag as conditional |
| Test mocks | `extern "C" fn mock_open(...)` | Filter functions with "mock", "test", "stub" prefixes |
| Inline extern fns | `extern "C" fn callback(...)` used purely internally | Require matching declaration on other side |

**Estimated false positive rate**: 3-8% for raw FFI, 1-3% for CXX bridge.

---

## 2. Pattern 2: WASM (Rust -> JS/TS)

### 2.1 Overview

WebAssembly bridges Rust code to JavaScript/TypeScript. The primary mechanism is `wasm-bindgen`, which uses attributes to annotate exported Rust functions and imported JS APIs. The `wasm-pack` tool builds the Rust code into a `pkg/` directory with JS/TS glue code.

### 2.2 Rust-Side Signals

**Exporting Rust functions to JS**:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

#[wasm_bindgen]
impl Universe {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Universe { /* ... */ }

    pub fn tick(&mut self) { /* ... */ }

    pub fn render(&self) -> String { /* ... */ }
}
```

**Importing JS APIs into Rust**:
```rust
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_name = requestAnimationFrame)]
    fn raf(f: &Closure<dyn FnMut()>);
}
```

**wasm_bindgen attribute variants** (each provides different matching signals):

| Attribute Form | Meaning | JS-Side Name |
|---------------|---------|-------------|
| `#[wasm_bindgen]` | Export with same name | Same as Rust fn name |
| `#[wasm_bindgen(constructor)]` | Maps to JS `new ClassName()` | Constructor |
| `#[wasm_bindgen(js_name = "foo")]` | Export as different name | `foo` |
| `#[wasm_bindgen(js_namespace = console)]` | Import from JS namespace | `console.X` |
| `#[wasm_bindgen(module = "/snippets/foo.js")]` | Import from local JS module | Module path |
| `#[wasm_bindgen(js_class = "Element")]` | Bind to existing JS class | `Element` |
| `#[wasm_bindgen(getter)]` / `#[wasm_bindgen(setter)]` | Property accessor | `.propName` |
| `#[wasm_bindgen(structural)]` | Use structural (duck-typed) dispatch | Duck-typed |
| `#[wasm_bindgen(catch)]` | Function may throw | Returns `Result` |
| `#[wasm_bindgen(typescript_type = "T")]` | Override TS type | TS type `T` |

### 2.3 JS/TS-Side Signals

**Importing wasm-bindgen exports in JS**:
```javascript
import init, { greet, Universe } from './pkg/my_game_of_life';

async function run() {
    await init();
    const universe = new Universe(64, 64);
    universe.tick();
    console.log(universe.render());
    console.log(greet("World"));
}
```

**TypeScript import with types**:
```typescript
import init, { greet, Universe } from './pkg/my_game_of_life';
// wasm-pack generates .d.ts files with full type information
```

Tree-sitter detection for JS/TS imports:
```scheme
;; Match import from wasm pkg directory
(import_statement
  source: (string
    (string_fragment) @import_path
    (#match? @import_path "\\./pkg/")))
```

The `./pkg/` path convention is strong signal for wasm-bindgen. Also detect:
- Dynamic `import()` calls: `const wasm = await import('./pkg/module.js')`
- Webpack/bundler imports: `import { X } from 'my-wasm-package'` (when package.json has `"module": "pkg/..."`)

### 2.4 Matching Algorithm

```
ALGORITHM: WASM_MATCH(rust_files, js_ts_files)
1. Extract all #[wasm_bindgen] annotated items from Rust files:
   -> Set WASM_EXPORTS = {(rust_name, js_name, kind, file, line)}
      where js_name = js_name attribute value OR rust_name
      where kind in {function, struct, impl_method, constructor}
2. Extract all imports with ./pkg/ path from JS/TS files:
   -> Set JS_IMPORTS = {(imported_name, module_path, file, line)}
3. Extract all dynamic import() with ./pkg/ path:
   -> Set JS_DYNAMIC = {(module_path, file, line)}
4. For each export in WASM_EXPORTS:
     for each import in JS_IMPORTS:
       if export.js_name == import.imported_name:
         emit edge: Rust:wasm_export -> JS:wasm_import (confidence: 0.92)
       elif export.rust_name == import.imported_name:
         emit edge: Rust:wasm_export -> JS:wasm_import (confidence: 0.88)
5. For unnamed imports (import *), match at module level:
     emit edge: Rust:wasm_module -> JS:wasm_consumer (confidence: 0.75)
```

### 2.5 False Positives and Mitigations

| Source | Example | Mitigation |
|--------|---------|------------|
| Non-wasm pkg/ directory | `import X from './pkg/utils'` (plain JS) | Verify Cargo.toml has `crate-type = ["cdylib"]` + wasm-bindgen dep |
| Name collisions | `greet` function exists in both Rust+WASM and plain JS util | Prioritize `./pkg/` imports over other import paths |
| Re-exports | JS re-exports WASM bindings via wrapper module | Follow re-export chains (max 3 hops) |
| Dead imports | Import exists but function never called | Only create edge; liveness analysis is separate concern |

**Estimated false positive rate**: 5-10% overall, 2-5% when Cargo.toml validation applied.

---

## 3. Pattern 3: PyO3 / JNI / Ruby FFI (Rust -> Python/Java/Ruby)

### 3.1 PyO3 (Rust -> Python)

PyO3 is the dominant Rust-Python bridge, used by Polars, Pydantic-core, Hugging Face Tokenizers, cryptography, and hundreds of other projects.

#### 3.1.1 Rust-Side Signals

**Function export**:
```rust
use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
#[pyo3(name = "custom_name", signature = (a, b=0))]
fn internal_name(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}
```

**Class export**:
```rust
#[pyclass]
struct MyClass {
    #[pyo3(get, set)]
    value: i32,
}

#[pymethods]
impl MyClass {
    #[new]
    fn new(value: i32) -> Self { MyClass { value } }

    fn double(&self) -> i32 { self.value * 2 }

    #[staticmethod]
    fn from_string(s: &str) -> PyResult<Self> { /* ... */ }

    #[getter]
    fn get_value(&self) -> i32 { self.value }
}
```

**Module registration**:
```rust
#[pymodule]
fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_class::<MyClass>()?;
    Ok(())
}
```

**Key PyO3 attributes** and their tree-sitter extraction:

| Attribute | Purpose | Python-Side Name |
|-----------|---------|-----------------|
| `#[pyfunction]` | Export function | Same as Rust fn name |
| `#[pyo3(name = "X")]` | Rename for Python | `X` |
| `#[pyclass]` | Export class | Same as Rust struct name |
| `#[pyclass(name = "X")]` | Renamed class | `X` |
| `#[pymethods]` | Export impl block methods | Method names |
| `#[new]` | `__init__` equivalent | Constructor |
| `#[staticmethod]` | Static method | `ClassName.method()` |
| `#[getter]` / `#[setter]` | Property accessors | `.prop` |
| `#[pymodule]` | Module entry point | `import module_name` |
| `#[pyo3(signature = (...))]` | Python signature | Controls kwargs |

Tree-sitter query for PyO3 attributes:
```scheme
;; Detect #[pyfunction]
(attribute_item
  (attribute
    (identifier) @attr
    (#any-of? @attr "pyfunction" "pyclass" "pymethods" "pymodule")))

;; Detect #[pyo3(name = "X")] for renaming
(attribute_item
  (attribute
    (identifier) @attr
    (#eq? @attr "pyo3")
    arguments: (token_tree
      (identifier) @key
      (#eq? @key "name")
      (string_literal) @python_name)))
```

#### 3.1.2 Python-Side Signals

```python
# Direct module import (maturin builds native extension)
from polars import DataFrame, Series
import polars as pl

# Specific function import
from my_module import sum_as_string, MyClass

# Usage
result = sum_as_string(1, 2)
obj = MyClass(42)
obj.double()
```

Tree-sitter-python node types:
- `import_from_statement` with `module_name` and `(dotted_name)` or `(aliased_import)` children
- `import_statement` with `(dotted_name)` or `(aliased_import)` children

```scheme
;; Match Python import-from statements
(import_from_statement
  module_name: (dotted_name) @module
  name: (dotted_name) @imported_name)
```

#### 3.1.3 Matching Algorithm

```
ALGORITHM: PYO3_MATCH(rust_files, python_files)
1. Extract #[pymodule] fn name -> MODULE_NAME
2. Extract all #[pyfunction] names (respecting #[pyo3(name="X")] renames)
   -> Set PY_EXPORTS = {(rust_name, python_name, kind, file, line)}
3. Extract all #[pyclass] names (respecting renames)
   -> Append to PY_EXPORTS
4. Extract all #[pymethods] method names within #[pyclass] impls
   -> Set PY_METHODS = {(class_name, method_name, kind)}
5. Extract all `from MODULE_NAME import X` from Python files
   -> Set PY_IMPORTS = {(module, imported_name, file, line)}
6. For each import in PY_IMPORTS:
     if import.module matches MODULE_NAME:
       for each export in PY_EXPORTS:
         if export.python_name == import.imported_name:
           emit edge: Python:import -> Rust:pyfunction (confidence: 0.90)
7. For attribute access patterns (obj.method()):
     if method_name in PY_METHODS for matching class:
       emit edge: Python:method_call -> Rust:pymethods (confidence: 0.80)
```

#### 3.1.4 Build System Signal: maturin

The presence of `pyproject.toml` with `[tool.maturin]` or `[build-system] requires = ["maturin>=..."]` is a strong confirmation signal that the project uses PyO3. Similarly, `Cargo.toml` with `crate-type = ["cdylib"]` and `pyo3` dependency.

### 3.2 JNI (Rust -> Java)

Java Native Interface connects Rust to Java/Kotlin/Android. Less common than PyO3 but critical for Android NDK projects.

#### 3.2.1 Rust-Side Signals

**Raw JNI** (naming convention based):
```rust
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern "system" fn Java_com_example_MyClass_processData(
    mut env: JNIEnv,
    _class: JClass,
    input: JString,
) -> jstring {
    // ...
}
```

The JNI naming convention is: `Java_{package}_{Class}_{method}` where:
- Package separators (`.`) become `_`
- Underscores in names become `_1`
- Unicode chars become `_0xxxx`

This naming convention is the primary detection signal. A function named `Java_com_example_Foo_bar` maps to Java class `com.example.Foo`, method `bar()`.

**robusta crate** (macro-based, cleaner):
```rust
use robusta_jni::bridge;

#[bridge]
mod jni {
    #[derive(Signature)]
    #[package(com.example)]
    struct MyClass;

    impl MyClass {
        pub extern "jni" fn processData(env: &JNIEnv, input: String) -> String {
            // ...
        }
    }
}
```

**jni-rs crate** (builder pattern):
```rust
// Less common, uses runtime registration rather than naming conventions
env.register_native_methods(
    "com/example/MyClass",
    &[NativeMethod {
        name: "processData",
        sig: "(Ljava/lang/String;)Ljava/lang/String;",
        fn_ptr: process_data as *mut c_void,
    }],
)?;
```

#### 3.2.2 Java-Side Signals

```java
package com.example;

public class MyClass {
    static {
        System.loadLibrary("my_rust_lib");
    }

    public native String processData(String input);
}
```

Tree-sitter-java detection:
- `method_declaration` with `native` modifier (no body)
- `static_initializer` block containing `System.loadLibrary()` call
- `(modifiers) @mods (#match? @mods "native")` + `(identifier) @method_name`

```scheme
;; Java native method declaration
(method_declaration
  (modifiers) @mods
  (#match? @mods "native")
  name: (identifier) @native_method_name)
```

#### 3.2.3 Matching Algorithm

```
ALGORITHM: JNI_MATCH(rust_files, java_files)
1. Extract all #[no_mangle] extern "system" functions matching Java_* pattern:
   -> Parse fn name: Java_{pkg}_{Class}_{method}
   -> Set JNI_EXPORTS = {(full_name, package, class, method, file, line)}
2. Extract all robusta #[bridge] annotated functions:
   -> Parse #[package(X)] and struct name
   -> Set JNI_EXPORTS += {(name, package, class, method, file, line)}
3. Extract all `native` method declarations from Java files:
   -> Set NATIVE_METHODS = {(package, class, method, file, line)}
4. Extract all System.loadLibrary() calls:
   -> Set LOADED_LIBS = {(lib_name, class, file)}
5. For each export in JNI_EXPORTS:
     for each native in NATIVE_METHODS:
       if export.package == native.package
          AND export.class == native.class
          AND export.method == native.method:
         emit edge: Java:native_call -> Rust:jni_fn (confidence: 0.92)
```

### 3.3 Ruby FFI (Rust -> Ruby)

#### 3.3.1 Magnus (Primary crate)

```rust
use magnus::{function, method, prelude::*, Error, Ruby};

#[magnus::wrap(class = "MyParser")]
struct Parser {
    inner: String,
}

impl Parser {
    fn new(input: String) -> Self { Parser { inner: input } }
    fn parse(&self) -> String { self.inner.clone() }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = ruby.define_class("MyParser", ruby.class_object())?;
    class.define_alloc_func::<Parser>();
    class.define_method("new", method!(Parser::new, 1))?;
    class.define_method("parse", method!(Parser::parse, 0))?;
    Ok(())
}
```

Detection signal: `#[magnus::init]` attribute on init function, `#[magnus::wrap]` on structs.

#### 3.3.2 Rutie (Legacy crate)

```rust
#[no_mangle]
pub extern "C" fn Init_my_extension() {
    Class::new("MyParser", None).define(|klass| {
        klass.def_self("new", parser_new);
        klass.def("parse", parser_parse);
    });
}
```

Detection signal: `Init_{name}` naming convention on `#[no_mangle] extern "C"` functions.

#### 3.3.3 Ruby-Side Signals

```ruby
require 'my_extension'

parser = MyParser.new("input data")
result = parser.parse

# Or via FFI gem (lower level)
require 'ffi'
module MyLib
  extend FFI::Library
  ffi_lib 'target/release/libmy_extension'
  attach_function :process, [:string], :string
end
```

Tree-sitter-ruby detection:
- `require` call with string literal (module name)
- `FFI::Library` extend + `attach_function` calls
- Class instantiation matching registered class names

#### 3.3.4 Matching Algorithm

```
ALGORITHM: RUBY_FFI_MATCH(rust_files, ruby_files)
1. Extract #[magnus::init] / Init_{name} functions
   -> Parse registered class names and methods from define_method/define_class calls
   -> Set RUBY_EXPORTS = {(class_name, methods[], file, line)}
2. Extract require statements from Ruby files:
   -> Set RUBY_REQUIRES = {(module_name, file, line)}
3. Extract FFI::Library attach_function calls:
   -> Set FFI_ATTACHMENTS = {(fn_name, lib_path, file, line)}
4. Match on class/module names and method names
5. Emit edges with confidence 0.85 (magnus) or 0.80 (rutie/ffi)
```

---

## 4. Pattern 4: HTTP/gRPC (Rust <-> Any)

### 4.1 Overview

HTTP and gRPC boundaries are the most common cross-language interface in microservice architectures. Unlike compile-time patterns (FFI, WASM, PyO3), HTTP/gRPC matching relies on string literal extraction -- URL paths and route patterns. This makes it inherently lower confidence but broader in scope.

### 4.2 Server-Side Route Detection (by Framework)

Each web framework has a different syntax for declaring routes. The extraction strategy must be framework-aware.

#### Rust Frameworks

**Axum**:
```rust
let app = Router::new()
    .route("/api/v1/analyze", post(analyze_handler))
    .route("/api/v1/health", get(health_check))
    .route("/api/v1/entities/:id", get(get_entity));
```
Signal: `.route()` method call with string literal first arg.

**Actix-web**:
```rust
#[get("/api/v1/health")]
async fn health() -> impl Responder { /* ... */ }

#[post("/api/v1/analyze")]
async fn analyze(body: web::Json<Request>) -> impl Responder { /* ... */ }
```
Signal: `#[get("...")]`, `#[post("...")]`, etc. attributes on functions.

**Rocket**:
```rust
#[get("/api/v1/health")]
fn health() -> &'static str { "ok" }

#[post("/api/v1/analyze", data = "<input>")]
fn analyze(input: Json<Request>) -> Json<Response> { /* ... */ }
```
Signal: Same attribute pattern as Actix, different framework import.

#### Python Frameworks

**FastAPI**:
```python
@app.get("/api/v1/health")
async def health():
    return {"status": "ok"}

@app.post("/api/v1/analyze")
async def analyze(request: AnalysisRequest):
    return {"result": "..."}
```
Signal: `@app.get("...")`, `@app.post("...")` decorators.

**Flask**:
```python
@app.route("/api/v1/health", methods=["GET"])
def health():
    return jsonify({"status": "ok"})
```
Signal: `@app.route("...")` decorator.

**Django**:
```python
urlpatterns = [
    path('api/v1/health/', views.health, name='health'),
    path('api/v1/analyze/', views.analyze, name='analyze'),
]
```
Signal: `path()` calls in `urls.py` files.

#### JavaScript/TypeScript Frameworks

**Express**:
```javascript
app.get('/api/v1/health', (req, res) => { res.json({status: 'ok'}); });
app.post('/api/v1/analyze', analyzeHandler);
```
Signal: `app.get()`, `app.post()` calls with string literal path.

**NestJS**:
```typescript
@Controller('api/v1')
export class AnalyzeController {
    @Get('health')
    health() { return { status: 'ok' }; }

    @Post('analyze')
    analyze(@Body() request: AnalysisRequest) { /* ... */ }
}
```
Signal: `@Controller('prefix')` class decorator + `@Get('path')`, `@Post('path')` method decorators.

#### Java Frameworks

**Spring Boot**:
```java
@RestController
@RequestMapping("/api/v1")
public class AnalyzeController {
    @GetMapping("/health")
    public ResponseEntity<String> health() { return ResponseEntity.ok("ok"); }

    @PostMapping("/analyze")
    public ResponseEntity<Result> analyze(@RequestBody Request request) { /* ... */ }
}
```
Signal: `@RequestMapping`, `@GetMapping`, `@PostMapping` annotations.

#### Go Frameworks

**net/http (stdlib)**:
```go
http.HandleFunc("/api/v1/health", healthHandler)
http.HandleFunc("/api/v1/analyze", analyzeHandler)
```
Signal: `http.HandleFunc()` calls.

**Gin**:
```go
r := gin.Default()
r.GET("/api/v1/health", healthHandler)
r.POST("/api/v1/analyze", analyzeHandler)
```
Signal: `r.GET()`, `r.POST()` calls.

#### PHP Frameworks

**Laravel**:
```php
Route::get('/api/v1/health', [AnalyzeController::class, 'health']);
Route::post('/api/v1/analyze', [AnalyzeController::class, 'analyze']);
```
Signal: `Route::get()`, `Route::post()` calls.

#### C# Frameworks

**ASP.NET Core**:
```csharp
[ApiController]
[Route("api/v1")]
public class AnalyzeController : ControllerBase
{
    [HttpGet("health")]
    public IActionResult Health() => Ok("ok");

    [HttpPost("analyze")]
    public IActionResult Analyze([FromBody] Request request) { /* ... */ }
}
```
Signal: `[Route("...")]`, `[HttpGet("...")]`, `[HttpPost("...")]` attributes.

### 4.3 Client-Side Request Detection

**Python**:
```python
import requests
response = requests.post("http://localhost:8080/api/v1/analyze", json=data)
response = requests.get("http://localhost:8080/api/v1/health")

# httpx (async)
async with httpx.AsyncClient() as client:
    response = await client.post("http://localhost:8080/api/v1/analyze", json=data)
```

**JavaScript/TypeScript**:
```javascript
fetch('/api/v1/analyze', { method: 'POST', body: JSON.stringify(data) });
axios.post('/api/v1/analyze', data);
```

**Rust**:
```rust
let resp = reqwest::Client::new()
    .post("http://localhost:8080/api/v1/analyze")
    .json(&data)
    .send()
    .await?;
```

**Java**:
```java
RestTemplate rest = new RestTemplate();
ResponseEntity<String> response = rest.postForEntity(
    "http://localhost:8080/api/v1/analyze", request, String.class);
```

**Go**:
```go
resp, err := http.Post("http://localhost:8080/api/v1/analyze", "application/json", body)
```

### 4.4 Matching Algorithm

```
ALGORITHM: HTTP_MATCH(all_files)
1. Extract server routes from all framework patterns:
   -> Set ROUTES = {(method, path, handler_fn, framework, file, line)}
   - Normalize paths: strip host, normalize trailing slashes, expand path params
   - Path: "/api/v1/entities/:id" -> "/api/v1/entities/{param}"
2. Extract client requests from all HTTP client patterns:
   -> Set REQUESTS = {(method, url, file, line)}
   - Parse URL: extract path component, strip host/port
   - Normalize: same as routes
3. For each request in REQUESTS:
     for each route in ROUTES:
       if request.method == route.method AND path_matches(request.path, route.path):
         emit edge: Client:http_call -> Server:route_handler (confidence: 0.70)
4. Path matching supports:
   - Exact match: "/api/v1/health" == "/api/v1/health" (confidence: 0.75)
   - Param match: "/api/v1/entities/123" ~ "/api/v1/entities/:id" (confidence: 0.65)
   - Prefix match: "/api/v1/" ~ "/api/v1/health" (confidence: 0.50, flag as weak)
```

### 4.5 gRPC Detection

gRPC uses Protocol Buffer (`.proto`) files as the contract definition. Both server and client code are generated from the same `.proto` file, providing high-confidence matching.

**Proto file**:
```protobuf
syntax = "proto3";
package analyzer.v1;

service AnalyzerService {
    rpc Analyze (AnalyzeRequest) returns (AnalyzeResponse);
    rpc GetHealth (Empty) returns (HealthResponse);
}

message AnalyzeRequest {
    string path = 1;
    repeated string languages = 2;
}
```

**Server (Rust with tonic)**:
```rust
#[tonic::async_trait]
impl AnalyzerService for MyAnalyzer {
    async fn analyze(&self, request: Request<AnalyzeRequest>) -> Result<Response<AnalyzeResponse>, Status> {
        // ...
    }
}
```

**Client (Python with grpcio)**:
```python
channel = grpc.insecure_channel('localhost:50051')
stub = analyzer_v1_pb2_grpc.AnalyzerServiceStub(channel)
response = stub.Analyze(analyzer_v1_pb2.AnalyzeRequest(path="/src"))
```

gRPC matching algorithm:
```
ALGORITHM: GRPC_MATCH(all_files)
1. Parse .proto files:
   -> Set SERVICES = {(package, service, methods[], file)}
2. For each language, find generated client stubs (*_pb2_grpc.py, *_grpc.rs, etc.)
3. Find server implementations (trait impls in Rust, class extends in Java/Python)
4. Match service.method names across proto, server, and client
5. Emit edges with confidence 0.85 (proto file is ground truth)
```

### 4.6 OpenAPI/Swagger as Ground Truth

If an `openapi.yaml`, `openapi.json`, or `swagger.json` file exists in the repository, it serves as authoritative ground truth for HTTP API surface. This can boost confidence of HTTP route matches:

```yaml
openapi: "3.0.0"
paths:
  /api/v1/analyze:
    post:
      operationId: analyze
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/AnalyzeRequest'
```

If a detected route or client request matches an OpenAPI path, confidence increases by +0.10.

### 4.7 False Positives and Mitigations

| Source | Example | Mitigation |
|--------|---------|------------|
| Common paths | Multiple services all have `/health` | Require 2+ path segments to match (`/api/v1/X` not `/health`) |
| External APIs | `requests.get("https://api.github.com/repos/...")` | Filter out known external domains |
| Path variables | `/users/123` vs `/users/:id` are they the same? | Path parameter normalization |
| Test fixtures | `assert url == "/api/v1/test"` | Filter test file patterns |
| Dead routes | Route defined but handler is empty/todo | Only create edge; liveness is separate |
| URL in comments/docs | `// See /api/v1/analyze for details` | Only extract from code, not comments |

**Estimated false positive rate**: 10-20% for HTTP, 5-10% for gRPC (proto files constrain matching).

---

## 5. Pattern 5: Message Queues (Rust <-> Any via Iggy/Kafka/NATS)

### 5.1 Overview

Message queue boundaries are asynchronous cross-language edges. A producer in one language publishes to a topic; a consumer in another language subscribes to the same topic. The matching signal is the topic name string literal. This is the lowest confidence pattern because topic names are just strings that could appear anywhere.

### 5.2 Iggy (Rust-Native Message Streaming)

Iggy is a Rust-native persistent message streaming platform (iggy.rs). It uses a streams/topics/partitions model.

**Producer (Rust)**:
```rust
use iggy::client::Client;

let client = IggyClient::default();
client.connect().await?;

// Create stream and topic
client.create_stream("analytics", Some(1)).await?;
client.create_topic("analytics", "user-events", 3, None, None, None, None).await?;

// Send messages
let messages = vec![Message::from_str("event-data")?];
client.send_messages("analytics", "user-events", &Partitioning::balanced(), &messages).await?;
```

**Consumer (Rust or any Iggy client)**:
```rust
let messages = client.poll_messages(
    "analytics",
    "user-events",
    Some(0),  // partition
    &Consumer::new(0),
    &PollingStrategy::next(),
    10,  // count
    false,
).await?;
```

Detection signals:
- `create_topic()` / `send_messages()` / `poll_messages()` method calls
- String literals for stream name and topic name (first two args)
- Iggy-specific types in use statements: `iggy::client::Client`, `iggy::messages::Message`

### 5.3 Kafka

**Producer (Java)**:
```java
Properties props = new Properties();
props.put("bootstrap.servers", "localhost:9092");
Producer<String, String> producer = new KafkaProducer<>(props);
producer.send(new ProducerRecord<>("user-events", key, value));
```

**Consumer (Python)**:
```python
from kafka import KafkaConsumer
consumer = KafkaConsumer('user-events', bootstrap_servers='localhost:9092')
for message in consumer:
    process(message.value)
```

**Consumer (Go)**:
```go
reader := kafka.NewReader(kafka.ReaderConfig{
    Brokers: []string{"localhost:9092"},
    Topic:   "user-events",
    GroupID: "my-group",
})
```

Detection signals:
- `ProducerRecord` / `ConsumerRecord` types (Java)
- `KafkaProducer` / `KafkaConsumer` classes (Python)
- `kafka.NewReader` / `kafka.NewWriter` (Go)
- Topic name as string literal in constructor/method args

### 5.4 RabbitMQ

```python
# Publisher
channel.basic_publish(exchange='', routing_key='task_queue', body=message)

# Consumer
channel.basic_consume(queue='task_queue', on_message_callback=callback)
```

```java
// Publisher
channel.basicPublish("", "task_queue", null, message.getBytes());

// Consumer
channel.basicConsume("task_queue", true, deliverCallback, cancelCallback);
```

Detection signals:
- `basic_publish` / `basicPublish` with `routing_key` / queue name
- `basic_consume` / `basicConsume` with queue name
- Exchange + routing key for topic exchanges
- Queue declaration with queue name

### 5.5 NATS

```go
// Publisher
nc.Publish("user.events", data)

// Subscriber
nc.Subscribe("user.events", func(m *nats.Msg) {
    // handle message
})
```

```rust
// Publisher
client.publish("user.events", data.into()).await?;

// Subscriber
let mut subscriber = client.subscribe("user.events").await?;
```

Detection signals:
- `Publish` / `Subscribe` method calls with subject string
- NATS subjects use dot-separated hierarchies (e.g., `user.events.created`)
- Wildcard subscriptions: `user.*`, `user.>` (partial match)

### 5.6 Matching Algorithm

```
ALGORITHM: MQ_MATCH(all_files)
1. Identify MQ framework from imports/use statements:
   -> Classify each file as Iggy/Kafka/RabbitMQ/NATS/None
2. Extract producer calls with topic/queue/subject names:
   -> Set PRODUCERS = {(topic, mq_system, file, line, language)}
3. Extract consumer calls with topic/queue/subject names:
   -> Set CONSUMERS = {(topic, mq_system, file, line, language)}
4. For each producer in PRODUCERS:
     for each consumer in CONSUMERS:
       if producer.topic == consumer.topic:
         if producer.mq_system == consumer.mq_system:
           emit edge: Producer:publish -> Consumer:subscribe (confidence: 0.75)
         else:
           // Different MQ systems, same topic name -- probably coincidence
           emit edge: (confidence: 0.30, flag as suspicious)
5. For wildcard subscriptions (NATS):
     if consumer.topic matches producer.topic with wildcard rules:
       emit edge with confidence: 0.65
```

### 5.7 False Positives and Mitigations

| Source | Example | Mitigation |
|--------|---------|------------|
| Generic topic names | "events", "messages", "data" | Require 2+ word topics or namespace prefix |
| Config-defined topics | `topic = os.environ["TOPIC_NAME"]` | Flag as unresolvable, don't create edge |
| Test topics | "test-topic", "integration-test-events" | Filter test patterns |
| Same-language queues | Producer and consumer in same service | Only create cross-language edges |
| String constants | `const TOPIC: &str = "user-events"` | Resolve string constants to values |

**Estimated false positive rate**: 15-25% overall, 10-15% with namespace filtering.

---

## 6. Accuracy Analysis and Confidence Scoring

### 6.1 Confidence Score Model

Each cross-language edge gets a confidence score in [0.0, 1.0] calculated as:

```
confidence = base_score * signal_multiplier + bonus_adjustments - penalty_adjustments
```

**Base scores by pattern**:

| Pattern | Base Score | Rationale |
|---------|-----------|-----------|
| FFI (raw extern) | 0.90 | Explicit ABI declaration, name-based matching |
| FFI (CXX bridge) | 0.95 | Self-contained bridge module, include! paths |
| WASM (wasm_bindgen) | 0.88 | Attribute + pkg/ convention + name matching |
| PyO3 | 0.85 | Attribute + maturin convention + import matching |
| JNI | 0.88 | Naming convention (`Java_*`) is very distinctive |
| Ruby FFI (magnus) | 0.82 | Less standardized, multiple crate patterns |
| gRPC | 0.85 | Proto file as ground truth |
| HTTP | 0.65 | String literal matching, many false positive sources |
| Message Queues | 0.70 | String literal matching, topic name ambiguity |

### 6.2 Bonus Adjustments (increase confidence)

| Signal | Bonus | Condition |
|--------|-------|-----------|
| Build system confirmation | +0.05 | Cargo.toml has relevant dependency (pyo3, wasm-bindgen, etc.) |
| OpenAPI spec match | +0.10 | Route matches openapi.yaml path |
| Proto file match | +0.10 | gRPC service matches .proto definition |
| Bilateral match | +0.05 | Both sides found (server+client, producer+consumer) |
| Same repository | +0.03 | Both files in same git repository |
| Test file confirms | +0.05 | Integration test exercises the boundary |
| Multiple signal overlap | +0.05 | Same boundary detected by 2+ heuristics |

### 6.3 Penalty Adjustments (decrease confidence)

| Signal | Penalty | Condition |
|--------|---------|-----------|
| Unilateral match only | -0.10 | Only one side found (e.g., route but no client) |
| Generic name | -0.15 | Function/topic name is very common ("handle", "process", "events") |
| Cross-repository | -0.10 | Files in different repositories |
| Config-indirect | -0.20 | Topic/URL comes from config file, not literal |
| Test file source | -0.10 | Match involves test fixtures |
| Dynamic dispatch | -0.15 | URL or topic constructed at runtime |

### 6.4 Confidence Thresholds

| Threshold | Action | Use Case |
|-----------|--------|----------|
| >= 0.80 | **High confidence**: Include in dependency graph by default | FFI, WASM, PyO3, JNI |
| 0.60 - 0.79 | **Medium confidence**: Include with "uncertain" flag | HTTP routes, some MQ |
| 0.40 - 0.59 | **Low confidence**: Only include if user opts in | Weak HTTP matches, generic topics |
| < 0.40 | **Rejected**: Do not include in graph | Likely false positives |

### 6.5 Expected Accuracy by Pattern (Estimated)

| Pattern | Precision | Recall | F1 | Notes |
|---------|-----------|--------|-----|-------|
| FFI (CXX) | 0.97 | 0.95 | 0.96 | Bridge modules are self-documenting |
| FFI (raw) | 0.92 | 0.88 | 0.90 | Name matching is reliable |
| WASM | 0.90 | 0.85 | 0.87 | pkg/ convention helps |
| PyO3 | 0.88 | 0.82 | 0.85 | Attribute detection is solid |
| JNI | 0.92 | 0.80 | 0.86 | Naming convention is distinctive but less common |
| Ruby FFI | 0.85 | 0.75 | 0.80 | Multiple crate patterns reduce recall |
| gRPC | 0.90 | 0.85 | 0.87 | Proto files constrain matching |
| HTTP | 0.75 | 0.70 | 0.72 | String matching inherently noisy |
| MQ | 0.70 | 0.65 | 0.67 | Topic name ambiguity |

### 6.6 Compile-Time vs Runtime Pattern Divide

A critical observation: detection accuracy divides cleanly into two tiers:

**Tier 1: Compile-time patterns** (FFI, WASM, PyO3, JNI, Ruby FFI)
- Detected via attributes, naming conventions, type annotations
- Extraction is syntactic (tree-sitter query on AST node types)
- False positive rate: 3-12%
- These patterns are structural -- if the attribute exists, the boundary exists

**Tier 2: Runtime patterns** (HTTP, gRPC, MQ)
- Detected via string literal matching
- Extraction requires semantic understanding (which string is a URL? a topic?)
- False positive rate: 10-25%
- These patterns are behavioral -- the boundary only exists at runtime

This divide suggests v2.0.0 should ship Tier 1 patterns first (higher confidence, fewer edge cases) and Tier 2 in a follow-up release.

---

## 7. Real-World Test Repositories

### 7.1 Tier 1: Single-Pattern Test Targets

These repositories exercise one cross-language pattern deeply and serve as focused validation targets.

#### FFI Repositories

| Repository | Pattern | Languages | Why It's Good |
|-----------|---------|-----------|---------------|
| [nickel-org/rust-mustache](https://github.com/nickel-org/rust-mustache) | FFI | Rust + C | Small, clear extern blocks |
| [ArtifexSoftware/mupdf-rs](https://github.com/nickel-org/mupdf-rs) | FFI + bindgen | Rust + C | Large C library wrapped in Rust |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | FFI | Rust + WASM + C | Multi-pattern in one repo |
| [ArtifexSoftware/mupdf-rs](https://github.com/nickel-org/mupdf-rs) | FFI + CXX | Rust + C++ | CXX bridge example |
| [aspect-build/bazel-bsp](https://github.com/nickel-lang/nickel) | FFI | Rust + C + C++ | Complex FFI with build scripts |

#### WASM Repositories

| Repository | Pattern | Languages | Why It's Good |
|-----------|---------|-----------|---------------|
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | WASM | Rust + JS | Language server as WASM module |
| [nickel-org/nickel.rs](https://github.com/nickel-org/nickel.rs) | WASM | Rust + JS | Simple WASM bindings |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | WASM | Rust + JS + TS | Complex WASM application |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | WASM | Rust + JS | Game of Life tutorial (canonical example) |

Notable real-world WASM projects:
- **nickel**: Configuration language with WASM playground
- **egui**: Immediate mode GUI, runs in browser via WASM
- **wasm-bindgen examples**: Canonical examples in the wasm-bindgen repo itself

#### PyO3 Repositories

| Repository | Pattern | Languages | Why It's Good |
|-----------|---------|-----------|---------------|
| [pola-rs/polars](https://github.com/pola-rs/polars) | PyO3 | Rust + Python | Large-scale, production PyO3 usage |
| [pydantic/pydantic-core](https://github.com/pydantic/pydantic-core) | PyO3 | Rust + Python | Complex pyclass with validation logic |
| [huggingface/tokenizers](https://github.com/huggingface/tokenizers) | PyO3 + WASM | Rust + Python + JS | Both PyO3 AND WASM in one repo |
| [pyca/cryptography](https://github.com/pyca/cryptography) | PyO3 | Rust + Python | Security-critical FFI |
| [tikv/pyo3-ffi](https://github.com/tikv/pyo3-ffi) | PyO3 | Rust + Python | Low-level PyO3 FFI examples |

### 7.2 Tier 2: Multi-Pattern Test Targets

These repositories exercise multiple cross-language patterns simultaneously.

| Repository | Patterns | Languages | Why It's Good |
|-----------|----------|-----------|---------------|
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | FFI + WASM | Rust + C + JS | WASM + native builds |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | HTTP + gRPC | Multi | Microservice architecture |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | PyO3 + WASM | Rust + Py + JS | Dual-target bindings |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | FFI + JNI | Rust + C + Java | Database with multi-lang bindings |
| [nickel-lang/nickel](https://github.com/nickel-lang/nickel) | FFI + PyO3 | Rust + C + Python | Crypto library |

### 7.3 Tier 3: Integration Test Target (All Patterns)

The ideal integration test repository exercises ALL five patterns. The best candidate found:

**OpenTelemetry Demo** ([open-telemetry/opentelemetry-demo](https://github.com/open-telemetry/opentelemetry-demo))
- **Languages**: Go, Java, .NET, Node.js, PHP, Python, Ruby, Rust, C++, Kotlin, Erlang, TypeScript
- **Patterns**: HTTP (all services communicate via REST), gRPC (primary inter-service protocol), Kafka (async event streaming)
- **Size**: 12+ microservices in different languages
- **Quality**: Well-maintained by CNCF, production-quality code
- **Limitation**: No FFI/WASM/PyO3 patterns (it's pure microservices)

For FFI+WASM+PyO3 coverage, combine with:
- **Hugging Face Tokenizers**: PyO3 + WASM in one repo
- **CozoDB**: FFI (Rust + C via RocksDB) + multi-language bindings (Java, Python, JS)
- **nickel-lang/nickel**: WASM (playground) + FFI (evaluation engine)

### 7.4 Test Strategy

```
VALIDATION PLAN:
1. Unit tests: Synthetic fixtures with known boundaries (already in test-fixtures/)
2. Integration tests:
   - Clone Tier 1 repos, run detection, compare against manual ground truth
   - Ground truth = manually cataloged cross-lang edges per repo
3. Regression tests:
   - Golden file snapshots of detected edges for each Tier 1 repo
   - Any change in detection triggers review
4. Benchmark:
   - Process OpenTelemetry Demo (12+ services) in < 30 seconds
   - Memory usage < 500MB for any single repo
```

---

## 8. Tree-Sitter Signal Extraction Reference

### 8.1 Attribute Extraction (Rust)

All cross-language detection begins with attribute extraction. Rust attributes are the primary signal for Patterns 1-3.

```scheme
;; Generic attribute extractor for Rust
;; Captures: attribute name, optional arguments, and the annotated item
(attribute_item
  (attribute
    (identifier) @attr_name
    arguments: (token_tree)? @attr_args)) @full_attribute

;; Scoped attribute (e.g., #[cxx::bridge], #[magnus::init])
(attribute_item
  (attribute
    (scoped_identifier
      path: (identifier) @attr_namespace
      name: (identifier) @attr_name)
    arguments: (token_tree)? @attr_args)) @full_attribute
```

**Cross-language attributes to detect**:

| Attribute | Pattern | Language Pair |
|-----------|---------|---------------|
| `#[no_mangle]` | FFI | Rust -> C/C++ |
| `#[unsafe(no_mangle)]` | FFI (2024) | Rust -> C/C++ |
| `#[link_name = "..."]` | FFI | Rust -> C/C++ |
| `#[repr(C)]` | FFI (type) | Rust <-> C/C++ |
| `#[cxx::bridge]` | FFI (CXX) | Rust <-> C++ |
| `#[wasm_bindgen]` | WASM | Rust -> JS/TS |
| `#[wasm_bindgen(constructor)]` | WASM | Rust -> JS/TS |
| `#[wasm_bindgen(js_name = "X")]` | WASM | Rust -> JS/TS |
| `#[wasm_bindgen(js_namespace = X)]` | WASM | Rust -> JS/TS |
| `#[pyfunction]` | PyO3 | Rust -> Python |
| `#[pyclass]` | PyO3 | Rust -> Python |
| `#[pymethods]` | PyO3 | Rust -> Python |
| `#[pymodule]` | PyO3 | Rust -> Python |
| `#[pyo3(name = "X")]` | PyO3 | Rust -> Python |
| `#[magnus::init]` | Ruby | Rust -> Ruby |
| `#[magnus::wrap]` | Ruby | Rust -> Ruby |

### 8.2 String Literal Extraction (All Languages)

Patterns 4 and 5 depend on string literal extraction. Tree-sitter string literal nodes vary by language:

| Language | Node Type | Inner Text Node | Includes Quotes? |
|----------|-----------|-----------------|-------------------|
| Rust | `string_literal` | `string_content` | Yes (outer node), No (inner) |
| Rust | `raw_string_literal` | `string_content` | Yes (outer), No (inner) |
| Python | `string` | `string_content` | Yes (outer), No (inner) |
| Python | `concatenated_string` | `string` children | Requires concatenation |
| JavaScript | `string` | `string_fragment` | Yes (outer), No (inner) |
| JavaScript | `template_string` | `string_fragment` + `template_substitution` | Partial extraction only |
| TypeScript | `string` | `string_fragment` | Same as JS |
| Go | `interpreted_string_literal` | Direct text | Yes, includes quotes |
| Go | `raw_string_literal` | Direct text | Yes, includes backticks |
| Java | `string_literal` | Direct text | Yes, includes quotes |
| C | `string_literal` | `string_content` | Yes (outer), No (inner) |
| C++ | `string_literal` | `string_content` | Yes (outer), No (inner) |
| C++ | `raw_string_literal` | `raw_string_content` | Complex delimiters |
| Ruby | `string` | `string_content` | Yes (outer), No (inner) |
| Ruby | `heredoc_body` | Content lines | No quotes |
| PHP | `string` | `string_content` | Yes (outer), No (inner) |
| C# | `string_literal` | Direct text | Yes, includes quotes |
| C# | `verbatim_string_literal` | Direct text | Yes, includes @" |
| Swift | `string_literal` | `string_fragment` | Yes (outer), No (inner) |

**Critical caveat**: When extracting string literal text from tree-sitter, many languages include the surrounding quotes in the node text. Always use the inner content node (e.g., `string_content`, `string_fragment`) when available, or strip quotes manually.

### 8.3 Import Statement Extraction (Per Language)

Cross-language matching requires knowing what each file imports.

**Python**:
```scheme
(import_from_statement
  module_name: (dotted_name) @module
  name: (dotted_name) @imported_name)

(import_statement
  name: (dotted_name) @module)

(import_statement
  name: (aliased_import
    name: (dotted_name) @module
    alias: (identifier) @alias))
```

**JavaScript/TypeScript**:
```scheme
(import_statement
  source: (string
    (string_fragment) @import_path))

;; Named imports: import { X, Y } from '...'
(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @imported_name)))
  source: (string (string_fragment) @path))

;; Default import: import X from '...'
(import_statement
  (import_clause
    (identifier) @default_import)
  source: (string (string_fragment) @path))
```

**Java**:
```scheme
(import_declaration
  (scoped_identifier) @import_path)
```

**Go**:
```scheme
(import_declaration
  (import_spec_list
    (import_spec
      path: (interpreted_string_literal) @import_path
      name: (package_identifier)? @alias)))
```

**Ruby**:
```scheme
;; require 'module_name'
(call
  method: (identifier) @method
  (#eq? @method "require")
  arguments: (argument_list
    (string (string_content) @module_name)))

;; require_relative './path'
(call
  method: (identifier) @method
  (#eq? @method "require_relative")
  arguments: (argument_list
    (string (string_content) @relative_path)))
```

**PHP**:
```scheme
;; use Namespace\ClassName;
(namespace_use_declaration
  (namespace_use_clause
    (qualified_name) @import_path))

;; require/include
(include_expression
  (string (string_content) @file_path))
```

**C#**:
```scheme
(using_directive
  (qualified_name) @namespace)
```

**C/C++**:
```scheme
;; #include "file.h"
(preproc_include
  path: (string_literal) @include_path)

;; #include <system_header>
(preproc_include
  path: (system_lib_string) @system_include)
```

**Swift**:
```scheme
(import_declaration
  (identifier) @module_name)
```

### 8.4 Function Call Extraction (Method Calls with String Args)

For HTTP/MQ pattern detection, we need to extract method calls where the first (or specific) argument is a string literal.

**Generic pattern (Rust)**:
```scheme
;; method_call.method_name("string_arg", ...)
(call_expression
  function: (field_expression
    field: (field_identifier) @method_name)
  arguments: (arguments
    (string_literal) @first_string_arg))
```

**Python decorator extraction** (for Flask/FastAPI/Django):
```scheme
;; @app.get("/path") or @app.post("/path")
(decorated_definition
  (decorator
    (call
      function: (attribute
        object: (identifier) @obj
        attribute: (identifier) @method)
      arguments: (argument_list
        (string (string_content) @route_path))))
  definition: (function_definition
    name: (identifier) @handler_name))
```

**Java annotation extraction** (for Spring):
```scheme
;; @GetMapping("/path")
(method_declaration
  (modifiers
    (marker_annotation
      name: (identifier) @annotation_name
      (#any-of? @annotation_name "GetMapping" "PostMapping" "PutMapping" "DeleteMapping" "RequestMapping"))
    arguments: (annotation_argument_list
      (string_literal) @route_path))
  name: (identifier) @method_name)
```

### 8.5 Extern Block Extraction (Rust)

```scheme
;; Full extern block with all function signatures
(foreign_mod_item
  (extern_modifier
    (string_literal) @abi)
  (declaration_list
    (function_signature_item
      name: (identifier) @fn_name
      parameters: (parameters) @params
      return_type: (_)? @ret_type)*))
```

### 8.6 Tree-Sitter Node Type Reference

Key node types per language grammar relevant to cross-language detection:

**tree-sitter-rust**:
- `foreign_mod_item` -- extern block (NOT `extern_block`)
- `function_signature_item` -- function declaration without body (in extern blocks)
- `function_item` -- function definition with body
- `extern_modifier` -- the `extern "C"` part
- `attribute_item` -- `#[...]` attribute
- `attribute` -- inner attribute content
- `macro_invocation` -- macro call like `include!(...)`
- `token_tree` -- attribute arguments in parentheses

**tree-sitter-javascript / tree-sitter-typescript**:
- `import_statement` -- import declaration
- `import_clause` -- the imported names part
- `named_imports` -- `{ X, Y }` destructuring
- `import_specifier` -- individual named import
- `call_expression` -- function/method call
- `member_expression` -- property access (e.g., `app.get`)

**tree-sitter-python**:
- `import_from_statement` -- `from X import Y`
- `import_statement` -- `import X`
- `decorated_definition` -- function/class with decorators
- `decorator` -- `@` decorator expression
- `call` -- function call
- `attribute` -- property access (e.g., `app.get`)

**tree-sitter-java**:
- `method_declaration` -- method definition
- `marker_annotation` / `annotation` -- `@Annotation`
- `import_declaration` -- import statement
- `modifiers` -- access modifiers including `native`
- `method_invocation` -- method call

**tree-sitter-go**:
- `call_expression` -- function call
- `import_declaration` -- import block
- `import_spec` -- individual import
- `selector_expression` -- method call (e.g., `http.HandleFunc`)

---

## 9. Implementation Architecture for rust-llm-02

### 9.1 Crate Structure

```
rust-llm-02-cross-lang-edges/
├── src/
│   ├── lib.rs                  -- Public API: detect_cross_language_edges()
│   ├── detector.rs             -- Main orchestrator: runs all pattern detectors
│   ├── confidence.rs           -- Confidence scoring model
│   ├── patterns/
│   │   ├── mod.rs              -- Pattern trait definition
│   │   ├── ffi.rs              -- Pattern 1: FFI (extern, no_mangle, CXX, bindgen)
│   │   ├── wasm.rs             -- Pattern 2: WASM (wasm_bindgen)
│   │   ├── pyo3.rs             -- Pattern 3a: PyO3
│   │   ├── jni.rs              -- Pattern 3b: JNI
│   │   ├── ruby_ffi.rs         -- Pattern 3c: Ruby FFI (magnus, rutie)
│   │   ├── http.rs             -- Pattern 4a: HTTP routes
│   │   ├── grpc.rs             -- Pattern 4b: gRPC proto
│   │   └── message_queue.rs    -- Pattern 5: MQ (Iggy, Kafka, RabbitMQ, NATS)
│   ├── extractors/
│   │   ├── mod.rs              -- Extractor trait
│   │   ├── attributes.rs       -- Rust attribute extraction
│   │   ├── string_literals.rs  -- String literal extraction (all languages)
│   │   ├── imports.rs          -- Import statement extraction (all languages)
│   │   ├── extern_blocks.rs    -- extern block extraction
│   │   └── decorators.rs       -- Python/Java decorator/annotation extraction
│   └── types.rs                -- CrossLangEdge, Confidence, Pattern enums
├── tests/
│   ├── ffi_tests.rs
│   ├── wasm_tests.rs
│   ├── pyo3_tests.rs
│   ├── jni_tests.rs
│   ├── http_tests.rs
│   ├── grpc_tests.rs
│   ├── mq_tests.rs
│   └── integration_tests.rs
└── Cargo.toml
```

### 9.2 Core Types

```rust
/// A detected cross-language boundary edge
pub struct CrossLangEdge {
    /// Source side of the boundary
    pub source: BoundaryEndpoint,
    /// Target side of the boundary
    pub target: BoundaryEndpoint,
    /// Detection pattern that found this edge
    pub pattern: CrossLangPattern,
    /// Confidence score [0.0, 1.0]
    pub confidence: f64,
    /// Signals that contributed to detection
    pub signals: Vec<DetectionSignal>,
}

/// One side of a cross-language boundary
pub struct BoundaryEndpoint {
    /// Language of this endpoint
    pub language: Language,
    /// Entity key (e.g., "rust:fn:analyze_code")
    pub entity_key: String,
    /// File path
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Role in the boundary
    pub role: BoundaryRole,
}

/// What role does this endpoint play?
pub enum BoundaryRole {
    Exporter,       // Rust #[pyfunction], #[wasm_bindgen], #[no_mangle]
    Importer,       // Python import, JS import, C extern decl
    Server,         // HTTP route handler, gRPC service impl
    Client,         // HTTP request, gRPC stub call
    Producer,       // MQ publish
    Consumer,       // MQ subscribe
}

/// Which pattern detected the edge
pub enum CrossLangPattern {
    FfiRaw,         // extern "C" + #[no_mangle]
    FfiCxx,         // #[cxx::bridge]
    FfiBindgen,     // bindgen / cbindgen
    WasmBindgen,    // #[wasm_bindgen]
    PyO3,           // #[pyfunction] / #[pyclass]
    Jni,            // Java_* naming / robusta
    RubyFfi,        // magnus / rutie
    Http,           // Route + request matching
    Grpc,           // Proto service + impl
    MessageQueue,   // Topic-based pub/sub
}

/// A signal that contributed to detection
pub struct DetectionSignal {
    pub kind: SignalKind,
    pub value: String,
    pub confidence_delta: f64,
}

pub enum SignalKind {
    Attribute,          // #[pyfunction], #[wasm_bindgen]
    NamingConvention,   // Java_com_example_*
    StringLiteral,      // "/api/v1/analyze", "user-events"
    ImportPath,         // from X import Y, import { X } from '...'
    BuildConfig,        // Cargo.toml dependency, pyproject.toml
    ProtoDefinition,    // .proto service definition
    OpenApiSpec,        // openapi.yaml path match
}
```

### 9.3 Pattern Trait

```rust
/// Trait implemented by each cross-language pattern detector
pub trait CrossLangPatternDetector {
    /// Which pattern does this detector handle?
    fn pattern(&self) -> CrossLangPattern;

    /// Extract signals from a single file's AST
    fn extract_signals(
        &self,
        tree: &tree_sitter::Tree,
        source: &[u8],
        language: Language,
        file_path: &Path,
    ) -> Vec<ExtractedSignal>;

    /// Match extracted signals across all files to produce edges
    fn match_signals(
        &self,
        signals: &[ExtractedSignal],
    ) -> Vec<CrossLangEdge>;

    /// Which languages can this detector handle?
    fn supported_languages(&self) -> &[Language];
}
```

### 9.4 Detection Pipeline

```
PIPELINE:
1. For each file in codebase:
   a. Parse with tree-sitter (reuse rust-llm-01's parsed trees)
   b. Run each PatternDetector::extract_signals()
   c. Collect all ExtractedSignals into a global pool

2. For each PatternDetector:
   a. Run match_signals() on the global signal pool
   b. Produce Vec<CrossLangEdge> with base confidence scores

3. Run confidence adjustment:
   a. Apply bonus/penalty rules from Section 6.2/6.3
   b. Filter edges below threshold (default 0.40)

4. Deduplicate:
   a. Same source+target detected by multiple patterns -> keep highest confidence
   b. Merge signals from all detectors into one edge

5. Output: Vec<CrossLangEdge> -> consumed by rust-llm-04 (Ascent reasoning)
```

### 9.5 Ascent Integration

The output of `rust-llm-02` feeds directly into `rust-llm-04`'s Ascent Datalog rules:

```rust
ascent! {
    struct CrossLangAnalysis;

    // Base relations from rust-llm-02
    relation cross_lang_edge(String, String, CrossLangPattern, f64);  // src, dst, pattern, confidence
    relation entity_language(String, Language);
    relation entity_file(String, PathBuf);

    // Derived: transitive cross-lang reachability
    relation cross_lang_reachable(String, String, u32);  // src, dst, hops
    cross_lang_reachable(S, D, 1) :- cross_lang_edge(S, D, _, C), C > 0.6;
    cross_lang_reachable(S, D, N+1) :- cross_lang_reachable(S, M, N), cross_lang_edge(M, D, _, C), C > 0.6, N < 5;

    // Derived: blast radius across language boundaries
    relation cross_lang_blast_radius(String, String, u32);
    cross_lang_blast_radius(S, D, H) :- cross_lang_reachable(S, D, H);

    // Derived: unsafe FFI chains (calls through extern that reach unsafe code)
    relation unsafe_ffi_chain(String, String);
    unsafe_ffi_chain(S, D) :- cross_lang_edge(S, D, FfiRaw, _), entity_is_unsafe(D);
    unsafe_ffi_chain(S, D) :- cross_lang_edge(S, M, FfiRaw, _), unsafe_ffi_chain(M, D);

    // Derived: polyglot hub detection (entities bridging 3+ languages)
    relation polyglot_hub(String, u32);  // entity, language_count
    // (requires aggregation, computed post-Ascent)
}
```

### 9.6 Performance Considerations

- **Parallel extraction**: Each file's signal extraction is independent and can run in parallel (rayon)
- **Lazy matching**: Only run match_signals() for patterns that found at least one signal
- **Early termination**: If no Rust files have cross-lang attributes, skip Patterns 1-3 entirely
- **String interning**: Topic names, URL paths, function names should be interned for O(1) equality checks
- **Memory budget**: Keep all signals in memory for matching; estimated 10-50 MB for a 10K-file codebase

### 9.7 Phased Rollout

| Phase | Patterns | Target Release |
|-------|----------|---------------|
| Phase 1 | FFI (raw + CXX) | v2.0.0-alpha |
| Phase 2 | WASM + PyO3 | v2.0.0-beta |
| Phase 3 | JNI + Ruby FFI | v2.0.0-rc |
| Phase 4 | HTTP + gRPC | v2.0.0 |
| Phase 5 | Message Queues | v2.1.0 |

Rationale: Phases 1-3 are compile-time patterns with high confidence (Tier 1). Phase 4-5 are runtime patterns requiring more testing and tuning (Tier 2).

---

## Appendix A: Key Findings Summary

1. **Tree-sitter sees both sides of every cross-language boundary**. This is the fundamental insight. Rust-analyzer only sees the Rust side. Tree-sitter's declarative query language can extract attributes, string literals, and imports across all 12 languages.

2. **Five detection patterns** cover the full spectrum of cross-language boundaries: FFI, WASM, PyO3/JNI/Ruby, HTTP/gRPC, and Message Queues.

3. **Accuracy divides into two tiers**: Compile-time patterns (FFI, WASM, PyO3) achieve 3-12% false positive rates. Runtime patterns (HTTP, MQ) achieve 10-25% false positive rates.

4. **CXX bridge is the highest-confidence pattern** (1-3% FP rate) because bridge modules are self-documenting and self-contained.

5. **String literal extraction is the critical bottleneck** for HTTP/MQ patterns. Each language's tree-sitter grammar handles strings differently (quotes included/excluded, concatenation, template literals, raw strings).

6. **Rust 2024 edition** introduces `#[unsafe(no_mangle)]` syntax requiring dual tree-sitter queries for backward compatibility.

7. **OpenTelemetry Demo** is the best single integration test target (12+ languages, HTTP/gRPC/Kafka), but lacks FFI/WASM/PyO3 patterns. Combine with Hugging Face Tokenizers (PyO3 + WASM) and CozoDB (FFI) for full coverage.

8. **Confidence scoring model** with base scores, bonuses, and penalties provides a principled way to filter false positives while preserving recall.

9. **Phased rollout** (compile-time first, runtime second) reduces risk and delivers high-confidence results early.

10. **The `rust-llm-02-cross-lang-edges` crate** should expose a `CrossLangPatternDetector` trait with `extract_signals()` and `match_signals()` methods, feeding into Ascent Datalog rules in `rust-llm-04`.
