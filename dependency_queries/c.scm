; C Dependency Queries (v1.4.9)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Include statements
(preproc_include
  path: (string_literal) @reference.include) @dependency.include

(preproc_include
  path: (system_lib_string) @reference.include_system) @dependency.include_system

; Field access (arrow and dot operators)
(field_expression
  field: (field_identifier) @reference.field_access) @dependency.field_access
