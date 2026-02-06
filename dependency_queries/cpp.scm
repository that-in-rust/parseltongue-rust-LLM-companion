; C++ Dependency Queries (v1.4.10)
; REQ-CPP-001.0 through REQ-CPP-004.0

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

(call_expression
  function: (field_expression
    field: (field_identifier) @reference.method_call)) @dependency.method_call

; Include statements
(preproc_include
  path: (string_literal) @reference.include) @dependency.include

(preproc_include
  path: (system_lib_string) @reference.include_system) @dependency.include_system

; Class inheritance
(class_specifier
  name: (type_identifier) @definition.class
  (base_class_clause
    (type_identifier) @reference.inherits)) @dependency.inherits

; ============================================================================
; REQ-CPP-001.0: Constructor Calls (new_expression)
; ============================================================================

; Simple new expressions: new Type()
(new_expression
  type: (type_identifier) @reference.constructor) @dependency.constructor

; Template new expressions: new std::vector<int>()
(new_expression
  type: (template_type
    name: (type_identifier) @reference.template_new)) @dependency.template_constructor

; ============================================================================
; REQ-CPP-002.0: Field Access (field_expression)
; ============================================================================

; Field access via -> or .
(field_expression
  field: (field_identifier) @reference.field_access) @dependency.field_access

; ============================================================================
; REQ-CPP-003.0: Template Types
; ============================================================================

; Template type - capture the base type name (vector, map, string, etc.)
(template_type
  name: (type_identifier) @reference.template_type) @dependency.template_use

; Template argument types (User, Config, int in templates)
(template_type
  arguments: (template_argument_list
    (type_descriptor
      type: (type_identifier) @reference.template_param))) @dependency.template_parameter

; Qualified identifiers used as types (std::string, std::vector, User, etc.)
(declaration
  type: (qualified_identifier) @reference.qualified_type) @dependency.qualified_type_use

; ============================================================================
; REQ-CPP-004.0: Smart Pointers
; ============================================================================

; Smart pointer calls: make_unique and make_shared
(call_expression
  function: (qualified_identifier) @reference.smart_pointer) @dependency.smart_pointer_call
