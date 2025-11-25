; TypeScript Dependency Queries (v0.9.0)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Import statements (simplified - TypeScript uses same as JavaScript)
(import_statement) @dependency.import
