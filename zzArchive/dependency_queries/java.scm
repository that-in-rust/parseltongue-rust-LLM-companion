; Java Dependency Queries (v0.9.0)

; Method calls
(method_invocation
  name: (identifier) @reference.call) @dependency.call

; Import statements
(import_declaration
  (scoped_identifier) @reference.import) @dependency.import

; Class extension
(class_declaration
  name: (identifier) @definition.class
  (superclass
    (type_identifier) @reference.extends)) @dependency.extends

; Interface implementation
(class_declaration
  name: (identifier) @definition.class
  (super_interfaces
    (type_list
      (type_identifier) @reference.implements))) @dependency.implements
