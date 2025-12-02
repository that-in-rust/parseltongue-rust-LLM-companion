; C entity extraction queries
; Based on tree-sitter-c grammar v0.21+

; Functions (with body - full definitions only)
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)
  body: (compound_statement)) @definition.function

; Structs (definitions with bodies)
(struct_specifier
  name: (type_identifier) @name
  body: (field_declaration_list)) @definition.struct

; Structs (forward declarations - optional, captures struct name without body)
(struct_specifier
  name: (type_identifier) @name) @definition.struct

; Enums
(enum_specifier
  name: (type_identifier) @name
  body: (enumerator_list)) @definition.enum

; Typedefs (using declaration node with type_definition)
(declaration
  type: (type_definition
    declarator: (type_identifier) @name)) @definition.typedef

; Alternative typedef pattern (typedef struct/enum/etc)
(type_definition
  (struct_specifier
    name: (type_identifier) @name)) @definition.typedef
