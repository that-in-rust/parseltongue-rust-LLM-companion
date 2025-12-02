; C++ Dependency Extraction Queries - v1.2.1
; MINIMAL version - only using verified tree-sitter-cpp node types
; All patterns validated via AST dump testing

; ============================================================================
; FUNCTION CALLS
; ============================================================================

; Simple function call
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Method call (obj.method())
(call_expression
  function: (field_expression
    field: (field_identifier) @reference.method_call)) @dependency.method_call

; Qualified function call (namespace::function)
(call_expression
  function: (qualified_identifier
    scope: (namespace_identifier) @namespace
    name: (identifier) @reference.qualified_call)) @dependency.qualified_call

; ============================================================================
; INCLUDES
; ============================================================================

; Local include (#include "file.h")
(preproc_include
  path: (string_literal) @reference.include) @dependency.include

; System include (#include <header>)
(preproc_include
  path: (system_lib_string) @reference.include_system) @dependency.include_system

; ============================================================================
; INHERITANCE
; ============================================================================

; Base class reference
(base_class_clause
  (type_identifier) @reference.base_type) @dependency.inherits

; Template base class
(base_class_clause
  (template_type
    name: (type_identifier) @reference.template_base)) @dependency.template_inherits

; ============================================================================
; TYPE REFERENCES
; ============================================================================

; Template type instantiation (vector<int>)
(template_type
  name: (type_identifier) @reference.template_name) @dependency.template_instantiation

; Variable type
(declaration
  type: (type_identifier) @reference.type_usage) @dependency.variable_type

; Parameter type
(parameter_declaration
  type: (type_identifier) @reference.parameter_type) @dependency.parameter_type

; Return type
(function_definition
  type: (type_identifier) @reference.return_type) @dependency.return_type

; Field type
(field_declaration
  type: (type_identifier) @reference.field_type) @dependency.field_type

; ============================================================================
; OBJECT CREATION
; ============================================================================

; New expression (new ClassName)
(new_expression
  type: (type_identifier) @reference.constructor) @dependency.new_expression

; ============================================================================
; USING DECLARATIONS
; ============================================================================

; using namespace std; (captures the identifier)
(using_declaration
  (identifier) @reference.using_namespace) @dependency.using_namespace

; using std::cout;
(using_declaration
  (qualified_identifier
    scope: (namespace_identifier) @namespace
    name: (identifier) @reference.using_symbol)) @dependency.using_symbol

; ============================================================================
; MEMBER ACCESS
; ============================================================================

; Direct member access (obj.member)
(field_expression
  field: (field_identifier) @reference.member_access) @dependency.member_access

; ============================================================================
; RANGE-BASED FOR
; ============================================================================

; for (auto x : container)
(for_range_loop
  right: (identifier) @reference.range_container) @dependency.range_for
