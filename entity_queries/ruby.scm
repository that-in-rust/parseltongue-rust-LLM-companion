; Ruby entity extraction queries
; Based on tree-sitter-ruby grammar

; Classes
(class
  name: (constant) @name) @definition.class

; Modules
(module
  name: (constant) @name) @definition.module

; Methods
(method
  name: (identifier) @name) @definition.method

; Singleton methods (class methods)
(singleton_method
  name: (identifier) @name) @definition.method
