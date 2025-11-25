; PHP Dependency Queries (v0.9.0)

; Function calls
(function_call_expression
  function: (name) @reference.call) @dependency.call

; Method calls
(member_call_expression
  name: (name) @reference.method_call) @dependency.method_call

; Use statements (namespace imports)
(use_declaration
  (qualified_name) @reference.use) @dependency.use

; Require/include
(require_expression
  (string) @reference.require) @dependency.require

(include_expression
  (string) @reference.include) @dependency.include

; Class extension
(class_declaration
  name: (name) @definition.class
  (base_clause
    (qualified_name) @reference.extends)) @dependency.extends

; Interface implementation
(class_declaration
  name: (name) @definition.class
  (class_interface_clause
    (qualified_name) @reference.implements)) @dependency.implements
