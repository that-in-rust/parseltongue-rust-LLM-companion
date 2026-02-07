; TypeScript Dependency Queries (v1.4.9)
; Comprehensive coverage: constructors, properties, collections, async, generics

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
(import_statement) @dependency.import

; ============================================================================
; PATTERN A: Constructor Calls (v1.4.9)
; ============================================================================

; Simple constructor: new Person()
(new_expression
  constructor: (identifier) @reference.constructor) @dependency.constructor

; Qualified constructor: new Models.Person()
(new_expression
  constructor: (member_expression
    property: (property_identifier) @reference.constructor_qualified)) @dependency.constructor_qualified

; Generic constructor: new Array<string>()
(new_expression
  constructor: (identifier) @reference.constructor_generic
  type_arguments: (type_arguments)) @dependency.constructor_with_generics

; ============================================================================
; PATTERN B: Property Access (v1.4.9)
; ============================================================================

; Property access: obj.name
; Note: We need to distinguish property access from method calls
; Method calls have arguments, property access doesn't
(member_expression
  property: (property_identifier) @reference.property_access) @dependency.property_access

; ============================================================================
; PATTERN C: Collection Operations (v1.4.9)
; ============================================================================

; Array methods: items.map(), items.filter(), etc.
; These are special method calls we want to track separately
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.collection_op
    (#match? @reference.collection_op "^(map|filter|reduce|forEach|find|some|every|includes)$"))) @dependency.collection_operation

; ============================================================================
; PATTERN D: Async/Await (v1.4.9)
; ============================================================================

; Await expression: await fetchData()
(await_expression
  (call_expression
    function: (identifier) @reference.async_call)) @dependency.async_call

; Await with member expression: await obj.method()
(await_expression
  (call_expression
    function: (member_expression
      property: (property_identifier) @reference.async_method))) @dependency.async_method_call

; Promise operations: promise.then(), promise.catch(), promise.finally()
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.promise_op
    (#match? @reference.promise_op "^(then|catch|finally)$"))) @dependency.promise_operation

; ============================================================================
; PATTERN E: Generic Types (v1.4.9)
; ============================================================================

; Generic type annotation: Array<string>
; This captures type references in variable declarations and parameters
(type_annotation
  (generic_type
    name: (type_identifier) @reference.generic_type)) @dependency.generic_type_ref

; Generic type in type alias
(type_alias_declaration
  value: (generic_type
    name: (type_identifier) @reference.generic_type_alias)) @dependency.generic_type_alias_ref

; ============================================================================
; PATTERN F: Class Inheritance (already exists but documenting)
; ============================================================================

; Class extends: class Child extends Parent
(class_declaration
  name: (type_identifier) @definition.class
  (class_heritage
    (extends_clause
      (identifier) @reference.extends))) @dependency.extends

; Interface extension: interface Child extends Parent
(interface_declaration
  name: (type_identifier) @definition.interface
  (extends_type_clause
    (type_identifier) @reference.extends_interface)) @dependency.interface_extends
