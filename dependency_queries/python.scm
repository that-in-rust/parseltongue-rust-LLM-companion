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
; NOTES
; ============================================================================
;
; Python-specific considerations:
; - Dynamic imports (importlib) not captured (runtime only)
; - Decorators could be added if needed
; - Type hints could provide additional dependency information
;
; Parseltongue EdgeType mapping:
; - @dependency.import* → EdgeType::Uses
; - @dependency.call → EdgeType::Calls
; - @dependency.method_call → EdgeType::Calls
; - @dependency.inherits → EdgeType::Implements (semantic equivalence)
