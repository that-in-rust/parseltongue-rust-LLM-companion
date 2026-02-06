; Go Dependency Queries (v1.4.9)
; Comprehensive coverage: composite literals, field access, goroutines, type references

; ============================================================================
; LEGACY PATTERNS (v0.9.0 - KEEP)
; ============================================================================

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Method calls (receiver.Method())
(call_expression
  function: (selector_expression
    field: (field_identifier) @reference.method_call)) @dependency.method_call

; Import statements
(import_spec
  path: (interpreted_string_literal) @reference.import) @dependency.import

; Type embedding (struct composition)
(type_declaration
  (type_spec
    name: (type_identifier) @definition.type
    type: (struct_type
      (field_declaration_list
        (field_declaration
          type: (type_identifier) @reference.embeds))))) @dependency.embeds

; ============================================================================
; PATTERN A: Composite Literals (v1.4.9) - Go's Constructor Equivalent
; ============================================================================

; Simple struct literal: User{Name: "John"}
(composite_literal
  type: (type_identifier) @reference.composite_type) @dependency.constructor

; Qualified struct literal: models.User{Name: "John"}
(composite_literal
  type: (qualified_type
    name: (type_identifier) @reference.composite_qualified)) @dependency.constructor

; Pointer composite literal: &Server{Port: 8080}
(unary_expression
  operator: "&"
  operand: (composite_literal
    type: (type_identifier) @reference.composite_pointer)) @dependency.constructor

; Pointer to qualified type: &models.User{}
(unary_expression
  operator: "&"
  operand: (composite_literal
    type: (qualified_type
      name: (type_identifier) @reference.composite_pointer_qualified))) @dependency.constructor

; Slice literal with type: []Item{{ID: 1}}
(composite_literal
  type: (slice_type
    element: (type_identifier) @reference.slice_type)) @dependency.constructor

; Map literal with type: map[string]User{}
(composite_literal
  type: (map_type
    value: (type_identifier) @reference.map_value_type)) @dependency.constructor

; ============================================================================
; PATTERN B: Field Access (v1.4.9) - Non-Call Context
; ============================================================================

; Note: In Go, selector_expression is used for BOTH method calls and field access.
; Without type information, we cannot distinguish them syntactically.
; The method_call pattern above already captures all selector_expression nodes.
;
; This pattern would create duplicate edges, so it's commented out.
; To enable field access as a separate dependency type, we would need:
; 1. Type information from go/types package
; 2. Or a different EdgeType to distinguish calls from uses
;
; Commented out to avoid duplicates:
; (selector_expression
;   field: (field_identifier) @reference.field_access) @dependency.field_access

; ============================================================================
; PATTERN C: Goroutines (v1.4.9)
; ============================================================================

; Goroutine launch: go processData()
(go_statement
  (call_expression
    function: (identifier) @reference.goroutine_call)) @dependency.goroutine_call

; Goroutine with method call: go obj.Method()
(go_statement
  (call_expression
    function: (selector_expression
      field: (field_identifier) @reference.goroutine_method))) @dependency.goroutine_call

; ============================================================================
; NOTES
; ============================================================================
;
; Go-specific considerations:
; - Composite literals are Go's equivalent of constructors
; - Field access vs method calls: both use selector_expression
; - Goroutines: captured as special call dependencies
; - Type references in slice/map literals: captured for dependency graph
;
; Parseltongue EdgeType mapping:
; - @dependency.call → EdgeType::Calls
; - @dependency.method_call → EdgeType::Calls
; - @dependency.import → EdgeType::Uses
; - @dependency.embeds → EdgeType::Implements
; - @dependency.constructor → EdgeType::Calls (composite literals)
; - @dependency.field_access → EdgeType::Uses
; - @dependency.goroutine_call → EdgeType::Calls
