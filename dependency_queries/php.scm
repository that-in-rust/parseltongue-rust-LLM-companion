; PHP Dependency Queries (v1.4.10)
; REQ-PHP-001.0: Constructor calls
; REQ-PHP-002.0: Property access
; REQ-PHP-003.0: Static method calls

; ============================================================================
; LEGACY PATTERNS (v0.9.0 - KEEP)
; ============================================================================

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

; ============================================================================
; REQ-PHP-001.0: Constructor Calls - v1.4.10
; ============================================================================

; Simple constructor: new User()
(object_creation_expression
  (name) @reference.constructor) @dependency.constructor

; Qualified constructor: new \Models\User()
(object_creation_expression
  (qualified_name) @reference.constructor_qualified) @dependency.constructor_qualified

; ============================================================================
; REQ-PHP-002.0: Property Access - v1.4.10
; ============================================================================

; Property access: $user->name, $user->age = 30
; Note: This captures both reads and writes
(member_access_expression
  name: (name) @reference.property_access) @dependency.property_access

; ============================================================================
; REQ-PHP-003.0: Static Method Calls - v1.4.10
; ============================================================================

; Static method call with identifier: Factory::create()
(scoped_call_expression
  scope: (name) @reference.static_class
  name: (name) @reference.static_method) @dependency.static_call

; Static method call with qualified name: \Utils\Factory::create()
(scoped_call_expression
  scope: (qualified_name) @reference.static_qualified_class
  name: (name) @reference.static_qualified_method) @dependency.static_qualified_call

; Static method calls with special keywords: parent::method(), self::method(), static::method()
(scoped_call_expression
  scope: [
    (relative_scope) @reference.relative_scope
  ]
  name: (name) @reference.relative_method) @dependency.relative_static_call
