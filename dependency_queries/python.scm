; Python Dependency Queries (v0.9.0)
;
; Captures two main types of dependencies:
; 1. Import statements (Uses edge)
; 2. Function/method calls (Calls edge)
;
; Python's dynamic nature makes complete static analysis challenging,
; but we can capture the most common patterns.

; ============================================================================
; DEPENDENCY TYPE 1: Import Statements (A imports B)
; ============================================================================

; Simple import: import os
(import_statement
  name: (dotted_name
    (identifier) @reference.import)) @dependency.import

; From import: from pathlib import Path
(import_from_statement
  module_name: (dotted_name) @reference.import_module
  name: (dotted_name
    (identifier) @reference.import_name)) @dependency.import_from

; From import with alias: from pathlib import Path as P
(import_from_statement
  module_name: (dotted_name) @reference.import_module_alias
  name: (aliased_import
    name: (dotted_name
      (identifier) @reference.import_alias_name))) @dependency.import_from_alias

; Import with alias: import numpy as np
(import_statement
  name: (aliased_import
    name: (dotted_name) @reference.import_with_alias)) @dependency.import_alias

; ============================================================================
; DEPENDENCY TYPE 2: Function Calls (A calls B)
; ============================================================================

; Direct function call: foo()
(call
  function: (identifier) @reference.call) @dependency.call

; Method call: obj.method()
(call
  function: (attribute
    attribute: (identifier) @reference.method_call)) @dependency.method_call

; Chained method call: obj.method1().method2()
(call
  function: (attribute
    object: (call)
    attribute: (identifier) @reference.chained_call)) @dependency.chained_call

; ============================================================================
; DEPENDENCY TYPE 3: Class Inheritance (A inherits B)
; ============================================================================

; Class with base classes: class Child(Parent):
(class_definition
  name: (identifier) @definition.class
  superclasses: (argument_list
    (identifier) @reference.base_class)) @dependency.inherits

; ============================================================================
; CONTEXT CAPTURE: Containing functions for scoping
; ============================================================================

; Capture function context for calls
(function_definition
  name: (identifier) @context.function_name
  body: (block
    (expression_statement
      (call) @context.call_in_function)))

; Capture class context
(class_definition
  name: (identifier) @context.class_name)

; ============================================================================
; DEPENDENCY TYPE 4: Attribute Access (v1.4.9)
; ============================================================================

; Attribute access: obj.name (property access)
(attribute
  attribute: (identifier) @reference.attribute_access) @dependency.attribute_access

; ============================================================================
; DEPENDENCY TYPE 5: Decorators (v1.4.9)
; ============================================================================

; Simple decorator: @property
(decorator
  (identifier) @reference.decorator) @dependency.decorator

; Decorator call: @app.route("/")
(decorator
  (call
    function: (attribute
      attribute: (identifier) @reference.decorator_call))) @dependency.decorator_call

; Decorator with dotted name: @app.route
(decorator
  (attribute
    attribute: (identifier) @reference.decorator_dotted)) @dependency.decorator_dotted

; ============================================================================
; DEPENDENCY TYPE 6: Async/Await (v1.4.9)
; ============================================================================

; Await expression: await fetch_data()
(await
  (call
    function: (identifier) @reference.await_call)) @dependency.await_call

; Await method call: await obj.method()
(await
  (call
    function: (attribute
      attribute: (identifier) @reference.await_method))) @dependency.await_method

; ============================================================================
; DEPENDENCY TYPE 7: Type Hints (v1.4.9)
; ============================================================================

; Generic type annotation: List[str]
(type
  (subscript
    value: (identifier) @reference.type_generic)) @dependency.type_generic

; Simple type annotation: name: str
(type
  (identifier) @reference.type_simple) @dependency.type_simple

; ============================================================================
; NOTES
; ============================================================================
;
; Python-specific considerations:
; - Dynamic imports (importlib) not captured (runtime only)
; - Decorators: captured as dependencies (v1.4.9)
; - Type hints: captured for generic types (v1.4.9)
; - Attribute access: distinguishes property access from method calls (v1.4.9)
;
; Parseltongue EdgeType mapping:
; - @dependency.import* → EdgeType::Uses
; - @dependency.call → EdgeType::Calls
; - @dependency.method_call → EdgeType::Calls
; - @dependency.inherits → EdgeType::Implements (semantic equivalence)
; - @dependency.attribute_access → EdgeType::Uses (v1.4.9)
; - @dependency.decorator* → EdgeType::Uses (v1.4.9)
; - @dependency.await_* → EdgeType::Calls (v1.4.9)
; - @dependency.type_* → EdgeType::Uses (v1.4.9)
