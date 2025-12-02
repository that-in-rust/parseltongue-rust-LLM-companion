; Python entity extraction queries
; Based on tree-sitter-python grammar

; Classes
(class_definition
  name: (identifier) @name) @definition.class

; Methods (functions inside classes) - must come before general functions
(class_definition
  body: (block
    (function_definition
      name: (identifier) @name) @definition.method))

; Functions (top-level only - but tree-sitter will match all, so we rely on dedup in code)
(function_definition
  name: (identifier) @name) @definition.function
