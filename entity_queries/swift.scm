; Swift entity extraction queries
; Based on tree-sitter-swift grammar v0.7
;
; CRITICAL: Swift grammar uses `class_declaration` for ALL type declarations
; (class, struct, enum), NOT separate node types like `struct_declaration`.
; The first child keyword ("class", "struct", "enum") differentiates them,
; but for entity extraction we treat them all as class-like entities.

; Functions
(function_declaration
  name: (simple_identifier) @name) @definition.function

; Classes, Structs, Enums - ALL use class_declaration node
; Note: Cannot distinguish between class/struct/enum at query level
; without predicates, so all are tagged as @definition.class
(class_declaration
  name: (type_identifier) @name) @definition.class

; Protocols (has its own node type)
(protocol_declaration
  name: (type_identifier) @name) @definition.interface
