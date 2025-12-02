; JavaScript entity extraction queries
; Based on tree-sitter-javascript grammar

; Functions
(function_declaration
  name: (identifier) @name) @definition.function

; Generator functions
(generator_function_declaration
  name: (identifier) @name) @definition.function

; Function expressions assigned to variables
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(function_expression) (arrow_function)])) @definition.function

(variable_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(function_expression) (arrow_function)])) @definition.function

; Classes
(class_declaration
  name: (identifier) @name) @definition.class

; Methods
(method_definition
  name: (property_identifier) @name) @definition.method
