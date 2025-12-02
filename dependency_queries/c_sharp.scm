; C# Dependency Queries (v0.9.0)

; Method calls
(invocation_expression
  function: (identifier) @reference.call) @dependency.call

(invocation_expression
  function: (member_access_expression
    name: (identifier) @reference.method_call)) @dependency.method_call

; Using directives
(using_directive
  (qualified_name) @reference.using) @dependency.using

; Class inheritance
(class_declaration
  name: (identifier) @definition.class
  (base_list
    (identifier) @reference.extends)) @dependency.extends

; Interface implementation
(class_declaration
  name: (identifier) @definition.class
  (base_list
    (identifier) @reference.implements)) @dependency.implements
