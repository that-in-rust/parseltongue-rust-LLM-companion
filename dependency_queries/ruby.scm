; Ruby Dependency Queries (v1.4.9)
; Comprehensive pattern coverage: constructors, blocks, modules, attributes
;
; REQ-RUBY-001.0: Constructor calls (Class.new)
; REQ-RUBY-002.0: Block methods (.each, .map, .select)
; REQ-RUBY-003.0: Module inclusion (include, extend, prepend)
; REQ-RUBY-004.0: Attribute access

; ============================================================================
; LEGACY PATTERNS (v0.9.0 - KEEP)
; ============================================================================

; Method calls (general)
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

; ============================================================================
; REQ-RUBY-001.0: Constructor Calls (Class.new) - v1.4.9
; ============================================================================

; Simple constructor: User.new
(call
  receiver: (constant) @reference.constructor_class
  method: (identifier) @_new
  (#eq? @_new "new")) @dependency.constructor

; Qualified constructor: Models::User.new
(call
  receiver: (scope_resolution
    name: (constant) @reference.constructor_qualified)
  method: (identifier) @_new_qualified
  (#eq? @_new_qualified "new")) @dependency.constructor_qualified

; Nested qualified: Database::Connection.new
(call
  receiver: (scope_resolution
    scope: (constant)
    name: (constant) @reference.constructor_nested)
  method: (identifier) @_new_nested
  (#eq? @_new_nested "new")) @dependency.constructor_nested

; ============================================================================
; REQ-RUBY-002.0: Block Methods (.each, .map, .select, etc.) - v1.4.9
; ============================================================================

; Block methods with curly braces: items.each { |i| }
(call
  method: (identifier) @reference.block_method
  block: (block)) @dependency.block_call

; Block methods with do...end: items.each do |i| end
(call
  method: (identifier) @reference.block_do_method
  block: (do_block)) @dependency.block_do_call

; Chained method calls: items.map { }.compact.uniq
(call
  receiver: (call)
  method: (identifier) @reference.chained_method) @dependency.chained_call

; ============================================================================
; REQ-RUBY-003.0: Module Inclusion (include, extend, prepend) - v1.4.9
; ============================================================================

; DEPRECATED: Module inclusion patterns not working due to tree-sitter Ruby grammar limitations
; The tree-sitter Ruby parser doesn't expose constant arguments in a queryable way
; for bareword method calls like `include Enumerable`
;
; This is a known limitation - these patterns will be marked as P2 (low priority)
; and require either:
; 1. Tree-sitter Ruby grammar updates
; 2. Custom parsing logic in the extractor
;
; For now, we skip these patterns and focus on working patterns (constructors, blocks, attributes)

; ============================================================================
; REQ-RUBY-004.0: Attribute Access - v1.4.9
; ============================================================================

; Simple attribute access: obj.name
(call
  receiver: (identifier)
  method: (identifier) @reference.attribute) @dependency.attribute_access

; Chained attribute access: obj.parent.child
(call
  receiver: (call
    receiver: (identifier)
    method: (identifier))
  method: (identifier) @reference.chained_attribute) @dependency.chained_attribute_access
