; Ruby Dependency Queries (v0.9.0)

; Method calls
(call
  method: (identifier) @reference.call) @dependency.call

; Require statements
(call
  method: (identifier) @_require (#eq? @_require "require")
  arguments: (argument_list (string) @reference.require)) @dependency.require

; Class inheritance
(class
  name: (constant) @definition.class
  superclass: (superclass (constant) @reference.inherits)) @dependency.inherits

; Module inclusion
(call
  method: (identifier) @_include (#eq? @_include "include")
  arguments: (argument_list (constant) @reference.include_module)) @dependency.include_module
