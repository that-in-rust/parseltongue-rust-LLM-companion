; Java Dependency Queries (v1.4.9)
; Comprehensive coverage: constructors, fields, streams, generics, annotations

; ============================================================================
; LEGACY PATTERNS (v0.9.0 - KEEP)
; ============================================================================

; Method calls
(method_invocation
  name: (identifier) @reference.call) @dependency.call

; Import statements
(import_declaration
  (scoped_identifier) @reference.import) @dependency.import

; Class extension
(class_declaration
  name: (identifier) @definition.class
  (superclass
    (type_identifier) @reference.extends)) @dependency.extends

; Interface implementation
(class_declaration
  name: (identifier) @definition.class
  (super_interfaces
    (type_list
      (type_identifier) @reference.implements))) @dependency.implements

; ============================================================================
; PATTERN A: Constructor Calls (v1.4.9)
; ============================================================================

; Simple constructor: new Person()
(object_creation_expression
  type: (type_identifier) @reference.constructor) @dependency.constructor

; Generic constructor: new ArrayList<String>()
(object_creation_expression
  type: (generic_type
    (type_identifier) @reference.constructor_generic)) @dependency.constructor_generic

; ============================================================================
; PATTERN B: Field Access (v1.4.9)
; ============================================================================

; Field access: obj.field
(field_access
  field: (identifier) @reference.field_access) @dependency.field_access

; ============================================================================
; PATTERN C: Stream Operations (v1.4.9)
; ============================================================================

; Stream methods: list.stream().map()
; Note: Already captured by method_invocation pattern above

; ============================================================================
; PATTERN D: Generic Types (v1.4.9)
; ============================================================================

; Generic type in variable declaration: List<String>
(local_variable_declaration
  type: (generic_type
    (type_identifier) @reference.generic_type)) @dependency.generic_type

; Generic type in field declaration
(field_declaration
  type: (generic_type
    (type_identifier) @reference.generic_type)) @dependency.generic_type

; Generic type in parameter: void process(List<T> items)
(formal_parameter
  type: (generic_type
    (type_identifier) @reference.generic_type)) @dependency.generic_type

; ============================================================================
; PATTERN E: Annotations (v1.4.9)
; ============================================================================

; Annotation usage: @Override, @Entity
(marker_annotation
  name: (identifier) @reference.annotation) @dependency.annotation

; Annotation with arguments: @Table(name = "users")
(annotation
  name: (identifier) @reference.annotation) @dependency.annotation
