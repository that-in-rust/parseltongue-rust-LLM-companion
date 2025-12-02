; C# entity extraction queries
; Based on tree-sitter-c-sharp grammar

; Classes
(class_declaration
  name: (identifier) @name) @definition.class

; Interfaces
(interface_declaration
  name: (identifier) @name) @definition.interface

; Structs
(struct_declaration
  name: (identifier) @name) @definition.struct

; Enums
(enum_declaration
  name: (identifier) @name) @definition.enum

; Methods
(method_declaration
  name: (identifier) @name) @definition.method

; Properties
(property_declaration
  name: (identifier) @name) @definition.method
