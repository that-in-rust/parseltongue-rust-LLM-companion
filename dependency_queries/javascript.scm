; JavaScript Dependency Queries (v1.4.9)
; Comprehensive pattern coverage: constructors, properties, collections, async, promises

; ============================================================================
; LEGACY PATTERNS (v0.9.0 - KEEP)
; ============================================================================

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Method calls
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.method_call)) @dependency.method_call

; Import statements
(import_statement
  source: (string) @reference.import) @dependency.import

(import_statement
  (import_clause
    (identifier) @reference.import_name)) @dependency.import_name

; Require (CommonJS)
(call_expression
  function: (identifier) @_require (#eq? @_require "require")
  arguments: (arguments (string) @reference.require)) @dependency.require

; Class inheritance
(class_declaration
  name: (identifier) @definition.class
  (class_heritage
    (identifier) @reference.extends)) @dependency.extends

; ============================================================================
; PATTERN A: Constructor Calls (v1.4.9 - NEW)
; ============================================================================

; Simple constructor: new User()
(new_expression
  constructor: (identifier) @reference.constructor) @dependency.constructor

; Qualified constructor: new Models.User()
(new_expression
  constructor: (member_expression
    property: (property_identifier) @reference.constructor_qualified)) @dependency.constructor_qualified

; ============================================================================
; PATTERN B: Property Access (v1.4.9 - NEW)
; ============================================================================

; Property access (non-call context): obj.property
; This pattern captures all property access, including in method calls
; The query_extractor will deduplicate with method_call pattern
(member_expression
  property: (property_identifier) @reference.property_access) @dependency.property_access

; ============================================================================
; PATTERN C: Async/Await (v1.4.9 - NEW)
; ============================================================================

; Await expression with function call: await fetch()
(await_expression
  (call_expression
    function: (identifier) @reference.async_call)) @dependency.async_call

; Await expression with method call: await obj.method()
(await_expression
  (call_expression
    function: (member_expression
      property: (property_identifier) @reference.async_method_call))) @dependency.async_method_call

; ============================================================================
; PATTERN D: Array Methods / Collection Operations (v1.4.9 - NEW)
; ============================================================================

; Array methods: items.map(), items.filter(), items.reduce(), etc.
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.array_method
    (#match? @reference.array_method "^(map|filter|reduce|forEach|find|findIndex|some|every|includes|indexOf)$"))) @dependency.array_method

; ============================================================================
; PATTERN E: Promise Chains (v1.4.9 - NEW)
; ============================================================================

; Promise methods: promise.then(), promise.catch(), promise.finally()
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.promise_method
    (#match? @reference.promise_method "^(then|catch|finally)$"))) @dependency.promise_method
