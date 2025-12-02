; Java entity extraction queries
; Based on tree-sitter-java grammar

; Classes
(class_declaration
  name: (identifier) @name) @definition.class

; Interfaces
(interface_declaration
  name: (identifier) @name) @definition.interface

; Enums
(enum_declaration
  name: (identifier) @name) @definition.enum

; Methods
(method_declaration
  name: (identifier) @name) @definition.method

; Constructors
(constructor_declaration
  name: (identifier) @name) @definition.method
