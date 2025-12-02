; C++ Entity Extraction Queries - v1.2.1
; MINIMAL version - only using verified tree-sitter-cpp node types
; All patterns validated via AST dump testing

; ============================================================================
; CLASSES & STRUCTS
; ============================================================================

; Class definitions
(class_specifier
  name: (type_identifier) @name) @definition.class

; Struct definitions
(struct_specifier
  name: (type_identifier) @name) @definition.struct

; ============================================================================
; FUNCTIONS
; ============================================================================

; Free function
(function_declarator
  declarator: (identifier) @name) @definition.function

; Method (field_identifier)
(function_declarator
  declarator: (field_identifier) @name) @definition.function

; Qualified function (namespace::function or class::method)
(function_declarator
  declarator: (qualified_identifier
    scope: (namespace_identifier) @scope
    name: (identifier) @name)) @definition.method

; ============================================================================
; NAMESPACES
; ============================================================================

; Namespace definition
(namespace_definition
  name: (namespace_identifier) @name) @definition.namespace

; ============================================================================
; ENUMS
; ============================================================================

; Enum definition
(enum_specifier
  name: (type_identifier) @name) @definition.enum

; Enum constant
(enumerator
  name: (identifier) @name) @definition.enum_constant

; ============================================================================
; TYPE ALIASES
; ============================================================================

; Using alias (modern C++)
(alias_declaration
  name: (type_identifier) @name) @definition.type_alias

; ============================================================================
; UNIONS
; ============================================================================

; Union definition
(union_specifier
  name: (type_identifier) @name) @definition.union
