; Kotlin entity extraction queries
; Based on tree-sitter-kotlin grammar

; Functions
(function_declaration
  (simple_identifier) @name) @definition.function

; Classes
(class_declaration
  (type_identifier) @name) @definition.class

; Interfaces
(interface_declaration
  (type_identifier) @name) @definition.interface

; Objects
(object_declaration
  (type_identifier) @name) @definition.class
