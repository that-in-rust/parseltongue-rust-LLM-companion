; Go Dependency Queries (v0.9.0)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

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
