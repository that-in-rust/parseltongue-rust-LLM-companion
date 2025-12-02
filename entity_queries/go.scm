; Go entity extraction queries
; Based on tree-sitter-go grammar

; Functions
(function_declaration
  name: (identifier) @name) @definition.function

; Methods
(method_declaration
  name: (field_identifier) @name) @definition.method

; Structs
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type))) @definition.struct

; Interfaces
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type))) @definition.interface
