; JavaScript Dependency Queries (v0.9.0)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

(call_expression
  function: (member_expression
    property: (property_identifier) @reference.method_call)) @dependency.method_call

; Import statements
(import_statement
  source: (string) @reference.import) @dependency.import

(import_statement
  (import_clause
    (identifier) @reference.import_name)) @dependency.import_name

; Require (CommonJS)
(call_expression
  function: (identifier) @_require (#eq? @_require "require")
  arguments: (arguments (string) @reference.require)) @dependency.require

; Class inheritance
(class_declaration
  name: (identifier) @definition.class
  (class_heritage
    (identifier) @reference.extends)) @dependency.extends
