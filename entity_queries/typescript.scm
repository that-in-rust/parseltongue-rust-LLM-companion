; TypeScript entity extraction queries
; Extends JavaScript with TypeScript-specific constructs

; Functions (same as JavaScript)
(function_declaration
  name: (identifier) @name) @definition.function

; Arrow functions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: (arrow_function))) @definition.function

; Classes
(class_declaration
  name: (type_identifier) @name) @definition.class

; Interfaces
(interface_declaration
  name: (type_identifier) @name) @definition.interface

; Type aliases
(type_alias_declaration
  name: (type_identifier) @name) @definition.typedef

; Enums
(enum_declaration
  name: (identifier) @name) @definition.enum

; Methods
(method_signature
  name: (property_identifier) @name) @definition.method

(method_definition
  name: (property_identifier) @name) @definition.method
