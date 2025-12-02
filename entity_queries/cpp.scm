; C++ entity extraction queries
; Based on tree-sitter-cpp grammar

; Functions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function

; Classes
(class_specifier
  name: (type_identifier) @name) @definition.class

; Structs
(struct_specifier
  name: (type_identifier) @name) @definition.struct

; Enums
(enum_specifier
  name: (type_identifier) @name) @definition.enum

; Namespaces
(namespace_definition
  name: (identifier) @name) @definition.namespace
