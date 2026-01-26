; TypeScript Dependency Queries (v0.9.0)

; Function calls
(call_expression
  function: (identifier) @reference.call) @dependency.call

; Method calls (e.g., obj.method())
(call_expression
  function: (member_expression
    property: (property_identifier) @reference.method_call)) @dependency.method_call

; Import statements - capture source path
(import_statement
  source: (string) @reference.import) @dependency.import

; Import names (default imports)
(import_statement
  (import_clause
    (identifier) @reference.import_name)) @dependency.import_name

; Named imports (e.g., import { foo, bar } from 'module')
(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @reference.named_import)))) @dependency.named_import

; Type imports (TypeScript-specific: import type { Foo } from 'module')
(import_statement
  "type"
  source: (string) @reference.type_import) @dependency.type_import

; Namespace imports (import * as ns from 'module')
(import_statement
  (import_clause
    (namespace_import
      (identifier) @reference.namespace_import))) @dependency.namespace_import

; Dynamic imports (const x = await import('./module'))
(call_expression
  function: (import)
  arguments: (arguments (string) @reference.dynamic_import)) @dependency.dynamic_import

; Require (CommonJS compatibility)
(call_expression
  function: (identifier) @_require (#eq? @_require "require")
  arguments: (arguments (string) @reference.require)) @dependency.require

; Class inheritance (extends) - simple identifier
(class_declaration
  name: (type_identifier) @definition.class
  (class_heritage
    (extends_clause
      (identifier) @reference.extends))) @dependency.extends

; Class inheritance (extends) - member expression (e.g., React.Component)
(class_declaration
  name: (type_identifier) @definition.class
  (class_heritage
    (extends_clause
      (member_expression) @reference.extends))) @dependency.extends_member

; Interface inheritance (extends)
(interface_declaration
  name: (type_identifier) @definition.interface
  (extends_type_clause
    (type_identifier) @reference.interface_extends)) @dependency.interface_extends

; Class implements interface
(class_declaration
  name: (type_identifier) @definition.class
  (class_heritage
    (implements_clause
      (type_identifier) @reference.implements))) @dependency.implements
