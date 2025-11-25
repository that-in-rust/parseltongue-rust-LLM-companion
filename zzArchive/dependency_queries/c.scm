; C Dependency Queries (v0.9.0)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Include statements
(preproc_include
  path: (string_literal) @reference.include) @dependency.include

(preproc_include
  path: (system_lib_string) @reference.include_system) @dependency.include_system
