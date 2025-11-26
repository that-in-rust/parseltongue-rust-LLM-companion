; Swift Dependency Queries (v0.9.0)
;
; Simplified to avoid invalid node types in tree-sitter-swift v0.7
; Protocol conformance and class inheritance queries removed due to
; incompatible node type: type_inheritance_clause doesn't exist in v0.7

; Function calls
(call_expression
  (simple_identifier) @reference.call) @dependency.call

; Import statements
(import_declaration
  (identifier) @reference.import) @dependency.import
