; C++ Dependency Queries (v0.9.0)

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
