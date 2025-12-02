; Rust Dependency Queries (v0.9.0)
;
; Captures three types of dependencies:
; 1. Function calls (Calls edge)
; 2. Use declarations (Uses edge)
; 3. Trait implementations (Implements edge)
;
; Query structure follows tree-sitter conventions:
; - @definition.* captures for relationship sources
; - @reference.* captures for relationship targets
; - @context.* captures for containing scopes

; ============================================================================
; DEPENDENCY TYPE 1: Function Calls (A calls B)
; ============================================================================

; Direct function calls: foo() or self.method()
(call_expression
  function: [
    (identifier) @reference.call
    (field_expression
      field: (field_identifier) @reference.call)
    (scoped_identifier
      name: (identifier) @reference.call)
  ]) @dependency.call

; Method calls with receiver: obj.method()
(call_expression
  function: (field_expression
    field: (field_identifier) @reference.method_call)) @dependency.method_call

; ============================================================================
; DEPENDENCY TYPE 2: Use Declarations (A uses B)
; ============================================================================

; Simple use: use std::collections::HashMap;
(use_declaration
  argument: (scoped_identifier
    name: (identifier) @reference.use)) @dependency.use

; Use with path: use std::path::PathBuf;
(use_declaration
  argument: (scoped_identifier
    path: (scoped_identifier)? @reference.use_path
    name: (identifier) @reference.use_name)) @dependency.use_with_path

; Use list: use std::{fs, io};
(use_declaration
  argument: (use_list
    (scoped_identifier
      name: (identifier) @reference.use_list_item))) @dependency.use_list

; Use wildcard: use std::prelude::*;
(use_declaration
  argument: (use_wildcard
    (scoped_identifier) @reference.use_wildcard)) @dependency.use_wildcard

; Use as: use std::collections::HashMap as Map;
(use_declaration
  argument: (use_as_clause
    path: (scoped_identifier
      name: (identifier) @reference.use_as))) @dependency.use_as

; ============================================================================
; DEPENDENCY TYPE 3: Trait Implementations (A implements B)
; ============================================================================

; impl Trait for Struct
(impl_item
  trait: [
    (type_identifier) @reference.impl_trait
    (scoped_type_identifier
      name: (type_identifier) @reference.impl_trait_scoped)
  ]
  type: (type_identifier) @definition.impl_type) @dependency.implements

; impl Struct (inherent impl, no trait)
; These create structural relationships but not trait dependencies
(impl_item
  !trait
  type: (type_identifier) @definition.inherent_impl) @dependency.inherent_impl

; ============================================================================
; DEPENDENCY TYPE 4: Type References (A uses type B)
; ============================================================================

; Function parameters with custom types
(function_item
  parameters: (parameters
    (parameter
      type: (type_identifier) @reference.param_type))) @dependency.param_type_ref

; Return types
(function_item
  return_type: (type_identifier) @reference.return_type) @dependency.return_type_ref

; Struct field types
(field_declaration
  type: (type_identifier) @reference.field_type) @dependency.field_type_ref

; ============================================================================
; CONTEXT CAPTURE: Containing functions for scoping
; ============================================================================

; Capture function context for calls
(function_item
  name: (identifier) @context.function_name
  body: (block
    (call_expression) @context.call_in_function))

; Capture impl block context
(impl_item
  type: (type_identifier) @context.impl_type_name)

; ============================================================================
; NOTES
; ============================================================================
;
; Tree-sitter query syntax:
; - [ ... ] = alternation (OR)
; - ( ... ) = grouping
; - @name = capture name
; - ? = optional
; - * = zero or more
; - + = one or more
;
; Parseltongue EdgeType mapping:
; - @dependency.call → EdgeType::Calls
; - @dependency.use* → EdgeType::Uses
; - @dependency.implements → EdgeType::Implements
; - @dependency.*_type_ref → EdgeType::Uses
