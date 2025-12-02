; PHP entity extraction queries
; Based on tree-sitter-php grammar

; Functions
(function_definition
  name: (name) @name) @definition.function

; Classes
(class_declaration
  name: (name) @name) @definition.class

; Interfaces
(interface_declaration
  name: (name) @name) @definition.interface

; Traits
(trait_declaration
  name: (name) @name) @definition.trait

; Methods
(method_declaration
  name: (name) @name) @definition.method
