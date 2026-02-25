# Prep Doc: Tree-Sitter Query Patterns for Parseltongue v2.0.0

**Date**: 2026-02-16
**Context**: Research into tree-sitter's declarative query language for enriched extraction in rust-llm-01-fact-extractor. This document catalogs every query pattern needed to replace v1.x's skeleton extraction with v2.0.0's full-fidelity fact extraction across all 12 languages.

**Companion Doc**: [Prep-Doc-V200.md](./Prep-Doc-V200.md) (Section 3: "What Tree-Sitter Gives Us That We Throw Away")

---

## Table of Contents

1. [Tree-Sitter Query Language Syntax](#1-tree-sitter-query-language-syntax)
2. [Extraction Target Queries](#2-extraction-target-queries)
3. [Per-Language Grammar Differences](#3-per-language-grammar-differences)
4. [tree-sitter-tags System](#4-tree-sitter-tags-system)
5. [Incremental Parsing API](#5-incremental-parsing-api)
6. [Language Injection](#6-language-injection)
7. [Performance: Queries vs Cursor Walking](#7-performance-queries-vs-cursor-walking)
8. [Current v1.x Query Inventory and Gaps](#8-current-v1x-query-inventory-and-gaps)
9. [v2.0.0 Implementation Plan](#9-v200-implementation-plan)

---

## 1. Tree-Sitter Query Language Syntax

Tree-sitter queries use an S-expression pattern matching language. A query consists of one or more patterns, where each pattern is an S-expression that matches nodes in a concrete syntax tree (CST). The query engine walks the tree and reports all matches along with named captures.

### 1.1 Basic S-Expression Patterns

A pattern matches a node by its type, with optional child patterns:

```scheme
; Match any function_item node
(function_item)

; Match function_item containing an identifier child
(function_item
  (identifier))

; Match binary_expression where both children are number_literal
(binary_expression
  (number_literal)
  (number_literal))
```

### 1.2 Field Names

Fields are how tree-sitter grammars name specific children. Use `field_name:` prefix to constrain matching to a specific field:

```scheme
; Match function_item and bind its name field
(function_item
  name: (identifier) @fn_name)

; Match function with parameters and body
(function_item
  name: (identifier) @name
  parameters: (parameters) @params
  body: (block) @body)
```

Field names are critical for v2.0.0 because they let us extract specific sub-parts (return types, parameters, visibility) without navigating the tree imperatively.

### 1.3 Captures (@name)

Captures bind matched nodes to names. They are the primary mechanism for extracting data:

```scheme
; @name captures the identifier text
; @definition.function captures the entire function_item node
(function_item
  name: (identifier) @name) @definition.function
```

**Capture naming convention** (from tree-sitter-tags standard):
- `@definition.function` -- a function definition
- `@definition.class` -- a class definition
- `@reference.call` -- a function call reference
- `@name` -- the identifier text of the entity
- `@doc` -- associated documentation comment

For v2.0.0, we extend this vocabulary with:
- `@visibility` -- visibility modifier (pub, private, etc.)
- `@modifier` -- function modifiers (async, unsafe, const, static)
- `@return_type` -- return type annotation
- `@param_name`, `@param_type` -- parameter names and types
- `@generic_param`, `@generic_bound` -- generic type parameters and bounds
- `@attribute` -- attributes/decorators
- `@doc_comment` -- documentation comments

### 1.4 Wildcards

- `(_)` matches any **named** node (corresponds to a grammar rule)
- `_` matches any node including anonymous nodes (literals, punctuation)

```scheme
; Match function with any return type node
(function_item
  return_type: (_) @ret_type)
```

### 1.5 Quantification Operators

- `?` -- optional (zero or one)
- `*` -- zero or more
- `+` -- one or more

```scheme
; Match function with optional visibility
(function_item
  (visibility_modifier)? @vis
  name: (identifier) @name)

; Match impl with one or more methods
(impl_item
  body: (declaration_list
    (function_item)+ @methods))
```

### 1.6 Alternation (OR)

Square brackets `[...]` match any one of the listed alternatives:

```scheme
; Match either identifier or field_expression in call position
(call_expression
  function: [
    (identifier) @fn_name
    (field_expression
      field: (field_identifier) @method_name)
  ])
```

### 1.7 Negation (Field Absence)

Prefix a field name with `!` to match nodes where that field is absent:

```scheme
; Match impl blocks WITHOUT a trait (inherent impls only)
(impl_item
  !trait
  type: (type_identifier) @name) @definition.impl

; Match functions WITHOUT a return type annotation
(function_definition
  !return_type
  name: (identifier) @name) @untyped_function
```

### 1.8 Anchors

The `.` anchor constrains child position:

```scheme
; Match only the FIRST child
(block . (expression_statement) @first_stmt)

; Match only the LAST child
(block (expression_statement) @last_stmt .)

; Match ADJACENT siblings
(block
  (expression_statement) @stmt1
  .
  (expression_statement) @stmt2)
```

### 1.9 Predicates

Predicates add text-level constraints beyond structural matching. They are enclosed in parentheses at the pattern level:

**`#eq?` / `#not-eq?`** -- Exact string comparison:
```scheme
; Match identifier whose text is exactly "main"
((identifier) @name
  (#eq? @name "main"))

; Match two captures with the same text
((identifier) @a
 (identifier) @b
 (#eq? @a @b))
```

**`#match?` / `#not-match?`** -- Regex matching (anchored to node text):
```scheme
; Match SCREAMING_CASE identifiers (constants)
((identifier) @constant
  (#match? @constant "^[A-Z_][A-Z0-9_]*$"))

; Match common iterator methods
((field_identifier) @iter_method
  (#match? @iter_method "^(map|filter|fold|collect|iter)$"))
```

**`#any-of?`** -- Efficient multi-string matching:
```scheme
; Match any of these keywords
((identifier) @keyword
  (#any-of? @keyword "async" "await" "yield"))
```

**`#is?` / `#is-not?`** -- Assertion predicates for metadata:
```scheme
; Assert properties on captures
((string_literal) @injection.content
  (#is? @injection.content "sql"))
```

**`#set!`** -- Set properties (used in highlighting/injection):
```scheme
; Set the injection language for matched content
((raw_string_literal) @injection.content
  (#set! injection.language "sql"))
```

### 1.10 Supertypes

Some grammars define supertypes that group multiple node types. In queries, matching a supertype matches any subtype:

```scheme
; _expression is a supertype matching all expression variants
(_expression) @any_expr
```

### 1.11 Grouping for Predicates

To attach a predicate to a pattern, wrap the pattern and predicate in outer parentheses:

```scheme
(
  (call_expression
    function: (identifier) @fn_name)
  (#eq? @fn_name "println")
) @println_call
```

---

## 2. Extraction Target Queries

This section provides real, grammar-validated query patterns for each extraction target that v2.0.0 needs. Each query is annotated with which languages it applies to and what grammar nodes are involved.

### 2.1 Function Declarations with Visibility, Modifiers, Return Types, and Parameters

**Rust -- Full function signature extraction:**

```scheme
; Full function extraction with all metadata
; Grammar nodes: function_item, visibility_modifier, function_modifiers,
;   identifier, type_parameters, parameters, parameter, type_identifier, block
(function_item
  (visibility_modifier)? @visibility
  (function_modifiers)? @modifiers
  name: (identifier) @name
  type_parameters: (type_parameters)? @generics
  parameters: (parameters) @params
  return_type: (_)? @return_type
  body: (block) @body) @definition.function
```

Extracting individual parameters within the function:

```scheme
; Extract each parameter's name and type
(function_item
  parameters: (parameters
    (parameter
      pattern: (identifier) @param_name
      type: (_) @param_type)))
```

Extracting function modifiers individually:

```scheme
; Async functions
(function_item
  (function_modifiers
    "async") @async_marker
  name: (identifier) @name) @definition.async_function

; Unsafe functions
(function_item
  (function_modifiers
    "unsafe") @unsafe_marker
  name: (identifier) @name) @definition.unsafe_function

; Const functions
(function_item
  (function_modifiers
    "const") @const_marker
  name: (identifier) @name) @definition.const_function
```

**Python -- Full function with decorators and type hints:**

```scheme
; Function with optional decorators, return type, async
; Grammar: decorated_definition wraps function_definition with decorator children
(decorated_definition
  (decorator)* @decorators
  definition: (function_definition
    name: (identifier) @name
    parameters: (parameters) @params
    return_type: (type)? @return_type
    body: (block) @body)) @definition.function

; Standalone function (no decorators)
(function_definition
  name: (identifier) @name
  parameters: (parameters) @params
  return_type: (type)? @return_type
  body: (block) @body) @definition.function
```

Python parameter extraction:

```scheme
; Parameters with type annotations and defaults
(function_definition
  parameters: (parameters
    (typed_parameter
      (identifier) @param_name
      type: (type) @param_type)?
    (default_parameter
      name: (identifier) @param_name
      value: (_) @default_value)?
    (typed_default_parameter
      name: (identifier) @param_name
      type: (type) @param_type
      value: (_) @default_value)?
    (list_splat_pattern
      (identifier) @varargs)?
    (dictionary_splat_pattern
      (identifier) @kwargs)?))
```

**Go -- Function and method declarations:**

```scheme
; Regular function
(function_declaration
  name: (identifier) @name
  parameters: (parameter_list) @params
  result: (_)? @return_type
  body: (block) @body) @definition.function

; Method (has receiver)
(method_declaration
  receiver: (parameter_list
    (parameter_declaration
      type: (_) @receiver_type)) @receiver
  name: (field_identifier) @name
  parameters: (parameter_list) @params
  result: (_)? @return_type
  body: (block) @body) @definition.method
```

**Java -- Method with modifiers and annotations:**

```scheme
; Method with access modifiers and annotations
(method_declaration
  (modifiers
    (marker_annotation)* @annotations
    ["public" "private" "protected"]? @access_modifier
    "static"? @static_modifier
    "abstract"? @abstract_modifier
    "synchronized"? @sync_modifier)? @modifiers
  type: (_) @return_type
  name: (identifier) @name
  parameters: (formal_parameters) @params
  (throws)? @throws
  body: (block)? @body) @definition.method
```

**TypeScript -- Function with generics and type annotations:**

```scheme
; Function declaration with full type info
(function_declaration
  name: (identifier) @name
  type_parameters: (type_parameters)? @generics
  parameters: (formal_parameters) @params
  return_type: (type_annotation)? @return_type
  body: (statement_block) @body) @definition.function

; Arrow function assigned to const/let
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    type: (type_annotation)? @type_annotation
    value: (arrow_function
      type_parameters: (type_parameters)? @generics
      parameters: (formal_parameters) @params
      return_type: (type_annotation)? @return_type
      body: (_) @body))) @definition.function
```

**C++ -- Function with templates and qualifiers:**

```scheme
; Function definition with template and qualifiers
(function_definition
  (storage_class_specifier)? @storage_class
  type: (_)? @return_type
  declarator: (function_declarator
    declarator: [
      (identifier) @name
      (qualified_identifier
        name: (identifier) @name
        scope: (namespace_identifier) @namespace)
      (field_identifier) @name
    ]
    parameters: (parameter_list) @params)
  body: (compound_statement) @body) @definition.function

; Template function
(template_declaration
  parameters: (template_parameter_list) @template_params
  (function_definition
    declarator: (function_declarator
      declarator: (identifier) @name))) @definition.template_function
```

**C# -- Method with modifiers and attributes:**

```scheme
; Method with attributes and modifiers
(method_declaration
  (attribute_list)* @attributes
  (modifier)* @modifiers
  returns: (_) @return_type
  name: (identifier) @name
  type_parameters: (type_parameter_list)? @generics
  parameters: (parameter_list) @params
  body: (block)? @body) @definition.method
```

**Swift -- Function with attributes:**

```scheme
; Function declaration
(function_declaration
  (attribute)* @attributes
  (modifiers)? @modifiers
  name: (simple_identifier) @name
  (type_parameters)? @generics
  (function_value_parameters) @params
  (throws_keyword)? @throws
  (function_type)? @return_type
  body: (function_body) @body) @definition.function
```

**Ruby -- Method definition:**

```scheme
; Instance method
(method
  name: (identifier) @name
  parameters: (method_parameters)? @params
  body: (_)? @body) @definition.method

; Class method (singleton method)
(singleton_method
  object: (_) @receiver
  name: (identifier) @name
  parameters: (method_parameters)? @params
  body: (_)? @body) @definition.method
```

**PHP -- Function and method:**

```scheme
; Function declaration
(function_definition
  name: (name) @name
  parameters: (formal_parameters) @params
  return_type: (union_type)? @return_type
  body: (compound_statement) @body) @definition.function

; Method declaration
(method_declaration
  (visibility_modifier)? @visibility
  (static_modifier)? @static
  name: (name) @name
  parameters: (formal_parameters) @params
  return_type: (union_type)? @return_type
  body: (compound_statement)? @body) @definition.method
```

### 2.2 Struct / Class / Trait / Interface Declarations with Generic Bounds

**Rust -- Struct with generics and where clauses:**

```scheme
; Struct with full generic information
(struct_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name
  type_parameters: (type_parameters
    (type_identifier)* @generic_param
    (constrained_type_parameter
      name: (type_identifier) @constrained_name
      bounds: (trait_bounds
        (type_identifier)* @bound))*)? @generics
  (where_clause
    (where_predicate
      left: (type_identifier) @where_type
      bounds: (trait_bounds
        (type_identifier)* @where_bound))*)? @where_clause
  body: (field_declaration_list)? @fields) @definition.struct
```

**Rust -- Trait with supertraits:**

```scheme
; Trait definition with supertraits
(trait_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics
  (trait_bounds
    (type_identifier)* @supertrait)? @bounds
  (where_clause)? @where_clause
  body: (declaration_list) @body) @definition.trait
```

**Go -- Struct and interface:**

```scheme
; Struct type declaration
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type_parameters: (type_parameter_list)? @generics
    type: (struct_type
      (field_declaration_list) @fields))) @definition.struct

; Interface type declaration
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type_parameters: (type_parameter_list)? @generics
    type: (interface_type
      (method_spec_list)? @methods))) @definition.interface
```

**Java -- Class with generics and extends/implements:**

```scheme
; Class with inheritance
(class_declaration
  (modifiers)? @modifiers
  name: (identifier) @name
  type_parameters: (type_parameters)? @generics
  (superclass
    (type_identifier) @extends)?
  (super_interfaces
    (type_list
      (type_identifier)* @implements))?
  body: (class_body) @body) @definition.class

; Interface with extends
(interface_declaration
  (modifiers)? @modifiers
  name: (identifier) @name
  type_parameters: (type_parameters)? @generics
  (extends_interfaces
    (type_list
      (type_identifier)* @extends))?
  body: (interface_body) @body) @definition.interface
```

**TypeScript -- Interface with extends:**

```scheme
; Interface declaration
(interface_declaration
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics
  (extends_type_clause
    (type_identifier)* @extends)?
  body: (object_type) @body) @definition.interface

; Type alias
(type_alias_declaration
  name: (type_identifier) @name
  type_parameters: (type_parameters)? @generics
  value: (_) @type_value) @definition.typedef
```

**C# -- Class with constraints:**

```scheme
; Class with base types and constraints
(class_declaration
  (attribute_list)* @attributes
  (modifier)* @modifiers
  name: (identifier) @name
  type_parameters: (type_parameter_list)? @generics
  (base_list
    (_)* @base_types)?
  (type_parameter_constraints_clause)* @constraints
  body: (declaration_list) @body) @definition.class
```

### 2.3 Impl Blocks with Trait Associations (Rust-Specific)

```scheme
; Trait implementation: impl Trait for Type
(impl_item
  (visibility_modifier)? @visibility
  type_parameters: (type_parameters)? @generics
  trait: [
    (type_identifier) @trait_name
    (scoped_type_identifier
      path: (_) @trait_path
      name: (type_identifier) @trait_name)
  ]
  type: (type_identifier) @type_name
  (where_clause)? @where_clause
  body: (declaration_list
    (function_item)* @methods)) @definition.impl_trait

; Inherent implementation: impl Type
(impl_item
  !trait
  type_parameters: (type_parameters)? @generics
  type: (type_identifier) @type_name
  (where_clause)? @where_clause
  body: (declaration_list
    (function_item)* @methods)) @definition.impl_inherent
```

### 2.4 Import / Use Statements (Full Paths)

**Rust:**

```scheme
; Simple use: use std::collections::HashMap;
(use_declaration
  argument: (scoped_identifier) @full_path) @definition.import

; Use with list: use std::{fs, io};
(use_declaration
  argument: (use_list
    (scoped_identifier)* @import_item
    (identifier)* @import_item)) @definition.import_list

; Use with wildcard: use std::prelude::*;
(use_declaration
  argument: (use_wildcard
    (scoped_identifier) @wildcard_path)) @definition.import_wildcard

; Use with alias: use std::collections::HashMap as Map;
(use_declaration
  argument: (use_as_clause
    path: (scoped_identifier) @original_path
    alias: (identifier) @alias_name)) @definition.import_alias
```

**Python:**

```scheme
; import os
(import_statement
  name: (dotted_name) @module_path) @definition.import

; from pathlib import Path
(import_from_statement
  module_name: (dotted_name) @module_path
  name: [
    (dotted_name (identifier) @imported_name)
    (aliased_import
      name: (dotted_name (identifier) @imported_name)
      alias: (identifier) @alias_name)
  ]) @definition.import_from

; from pathlib import *
(import_from_statement
  module_name: (dotted_name) @module_path
  (wildcard_import)) @definition.import_wildcard
```

**JavaScript / TypeScript:**

```scheme
; import { useState } from 'react';
(import_statement
  (import_clause
    (named_imports
      (import_specifier
        name: (identifier) @imported_name
        alias: (identifier)? @alias_name)*))?
  source: (string) @source_module) @definition.import

; import React from 'react';
(import_statement
  (import_clause
    (identifier) @default_import)?
  source: (string) @source_module) @definition.import_default

; const fs = require('fs');
(
  (call_expression
    function: (identifier) @fn_name
    arguments: (arguments
      (string) @module_path))
  (#eq? @fn_name "require")
) @definition.require
```

**Go:**

```scheme
; import "fmt"
(import_declaration
  (import_spec
    path: (interpreted_string_literal) @import_path)) @definition.import

; import block
(import_declaration
  (import_spec_list
    (import_spec
      name: (package_identifier)? @alias
      path: (interpreted_string_literal) @import_path)*)) @definition.import_block
```

**Java:**

```scheme
; import java.util.HashMap;
(import_declaration
  (scoped_identifier) @import_path) @definition.import

; import java.util.*;
(import_declaration
  (scoped_identifier) @import_path
  (asterisk)) @definition.import_wildcard

; import static java.lang.Math.PI;
(import_declaration
  "static"
  (scoped_identifier) @import_path) @definition.static_import
```

**C / C++:**

```scheme
; #include <stdio.h> or #include "mylib.h"
(preproc_include
  path: [
    (system_lib_string) @system_include
    (string_literal) @local_include
  ]) @definition.include
```

**C#:**

```scheme
; using System.Collections.Generic;
(using_directive
  (qualified_name) @namespace_path) @definition.using

; using Dict = System.Collections.Generic.Dictionary<string, int>;
(using_directive
  (name_equals
    (identifier) @alias_name)?
  (qualified_name) @namespace_path) @definition.using_alias
```

**Ruby:**

```scheme
; require 'json'
(
  (call
    method: (identifier) @method_name
    arguments: (argument_list
      (string) @module_path))
  (#any-of? @method_name "require" "require_relative")
) @definition.require

; include ActiveModel::Validations
(
  (call
    method: (identifier) @method_name
    arguments: (argument_list
      (scope_resolution) @mixin_path))
  (#eq? @method_name "include")
) @definition.include
```

### 2.5 Attributes / Decorators

**Rust -- #[...] attributes:**

```scheme
; Simple attribute: #[test]
(attribute_item
  (attribute
    (identifier) @attr_name)) @definition.attribute

; Attribute with arguments: #[derive(Debug, Clone)]
(attribute_item
  (attribute
    (identifier) @attr_name
    arguments: (token_tree) @attr_args)) @definition.attribute_with_args

; Inner attribute: #![allow(dead_code)]
(inner_attribute_item
  (attribute
    (identifier) @attr_name
    arguments: (token_tree)? @attr_args)) @definition.inner_attribute

; Specific derive extraction
(
  (attribute_item
    (attribute
      (identifier) @attr_name
      arguments: (token_tree
        (identifier)* @derived_trait)))
  (#eq? @attr_name "derive")
) @definition.derive
```

**Python -- @decorator:**

```scheme
; Simple decorator: @property
(decorator
  (identifier) @decorator_name) @definition.decorator

; Decorator with call: @app.route("/api")
(decorator
  (call
    function: [
      (identifier) @decorator_name
      (attribute
        object: (_) @decorator_object
        attribute: (identifier) @decorator_name)
    ]
    arguments: (argument_list) @decorator_args)) @definition.decorator_call

; Decorator with dotted name: @app.middleware
(decorator
  (attribute
    object: (_) @decorator_object
    attribute: (identifier) @decorator_name)) @definition.decorator_dotted
```

**Java -- @Annotation:**

```scheme
; Simple annotation: @Override
(marker_annotation
  name: (identifier) @annotation_name) @definition.annotation

; Annotation with arguments: @RequestMapping(value="/api")
(annotation
  name: (identifier) @annotation_name
  arguments: (annotation_argument_list) @annotation_args) @definition.annotation_with_args
```

**C# -- [Attribute]:**

```scheme
; Attribute: [Serializable]
(attribute_list
  (attribute
    name: (identifier) @attr_name
    (attribute_argument_list)? @attr_args)) @definition.attribute
```

### 2.6 Closure / Lambda Expressions

**Rust -- Closure with captures:**

```scheme
; Closure expression: |x, y| x + y
; Move closure: move |x| async { x.process() }
(closure_expression
  "move"? @is_move
  parameters: (closure_parameters
    (identifier)* @param_name
    (closure_pattern)* @pattern_param)
  return_type: (_)? @return_type
  body: (_) @body) @definition.closure
```

**Python -- Lambda:**

```scheme
; lambda x, y: x + y
(lambda
  parameters: (lambda_parameters
    (identifier)* @param_name)?
  body: (_) @body) @definition.lambda
```

**JavaScript / TypeScript -- Arrow function:**

```scheme
; Arrow function: (x, y) => x + y
(arrow_function
  parameters: [
    (identifier) @single_param
    (formal_parameters) @params
  ]
  body: (_) @body) @definition.arrow_function
```

**Java -- Lambda:**

```scheme
; Lambda: (x, y) -> x + y
(lambda_expression
  parameters: [
    (identifier) @single_param
    (inferred_parameters) @params
    (formal_parameters) @typed_params
  ]
  body: (_) @body) @definition.lambda
```

**Go -- Function literal (closure):**

```scheme
; func(x int) int { return x * 2 }
(func_literal
  parameters: (parameter_list) @params
  result: (_)? @return_type
  body: (block) @body) @definition.func_literal
```

**Ruby -- Block / Lambda:**

```scheme
; Block: do |x| ... end  or  { |x| ... }
(block
  parameters: (block_parameters
    (identifier)* @param_name)?) @definition.block

; Lambda: -> (x) { x * 2 }  or  lambda { |x| x * 2 }
(lambda
  parameters: (lambda_parameters
    (identifier)* @param_name)?
  body: (_) @body) @definition.lambda
```

### 2.7 String Literals (for Cross-Language Edge Detection)

String literal extraction is essential for v2.0.0's cross-language boundary detection (Section 5 of Prep-Doc-V200.md). We need to find URL paths, topic names, queue names, and FFI function names embedded in strings.

**Rust:**

```scheme
; Regular string: "api/v1/users"
(string_literal) @string.content

; Raw string: r#"SELECT * FROM users"#
(raw_string_literal) @string.raw_content

; String with suspected URL path
(
  (string_literal) @string.url_path
  (#match? @string.url_path "^\"(/[a-zA-Z0-9_/-]+)\"$")
)

; String with suspected SQL
(
  (raw_string_literal) @string.sql
  (#match? @string.sql "(SELECT|INSERT|UPDATE|DELETE|CREATE TABLE)")
)
```

**Python:**

```scheme
(string) @string.content

; f-string
(string
  (string_start) @fstring_start
  (interpolation) @fstring_interpolation
  (#match? @fstring_start "^f")) @string.fstring
```

**JavaScript / TypeScript:**

```scheme
(string) @string.content
(template_string) @string.template

; Template literal with expressions
(template_string
  (template_substitution
    (_) @template_expr)*) @string.template_with_expr
```

**Go:**

```scheme
(interpreted_string_literal) @string.content
(raw_string_literal) @string.raw_content
```

**Java:**

```scheme
(string_literal) @string.content

; Text block (multi-line string)
(text_block) @string.text_block
```

### 2.8 Doc Comments

**Rust -- /// and //! comments:**

```scheme
; Line doc comment: /// This is documentation
(line_comment) @comment.line
(
  (line_comment) @comment.doc
  (#match? @comment.doc "^///")
)

; Inner doc comment: //! This module does...
(
  (line_comment) @comment.inner_doc
  (#match? @comment.inner_doc "^//!")
)

; Block doc comment: /** ... */
(block_comment) @comment.block
(
  (block_comment) @comment.doc_block
  (#match? @comment.doc_block "^/\\*\\*")
)
```

**Python -- Docstrings:**

```scheme
; Docstring is the first statement in a function/class body
; It is an expression_statement containing a string
(function_definition
  body: (block
    . (expression_statement
        (string) @doc_string))) @function_with_doc

(class_definition
  body: (block
    . (expression_statement
        (string) @doc_string))) @class_with_doc

; Module-level docstring
(module
  . (expression_statement
      (string) @module_doc_string))
```

**Go -- Doc comments (comments immediately preceding declarations):**

```scheme
; Comment preceding function
(
  (comment)+ @doc_comment
  .
  (function_declaration
    name: (identifier) @name)
)
```

**Java -- Javadoc:**

```scheme
; Block comment starting with /**
(
  (block_comment) @javadoc
  (#match? @javadoc "^/\\*\\*")
)
```

**JavaScript / TypeScript -- JSDoc:**

```scheme
; JSDoc comment
(
  (comment) @jsdoc
  (#match? @jsdoc "^/\\*\\*")
)
```

---

## 3. Per-Language Grammar Differences

The same conceptual construct (e.g., "function declaration") maps to different grammar node types across languages. This section catalogs the critical differences.

### 3.1 Function Declaration Node Types

| Language | Node Type | Name Field | Modifiers | Return Type |
|----------|-----------|------------|-----------|-------------|
| Rust | `function_item` | `name: (identifier)` | `(function_modifiers)` child | `return_type:` field |
| Python | `function_definition` | `name: (identifier)` | `async` keyword child | `return_type: (type)` field |
| JavaScript | `function_declaration` | `name: (identifier)` | `async` keyword child | N/A |
| TypeScript | `function_declaration` | `name: (identifier)` | `async` keyword child | `return_type: (type_annotation)` |
| Go | `function_declaration` | `name: (identifier)` | N/A | `result:` field |
| Java | `method_declaration` | `name: (identifier)` | `(modifiers)` child | `type:` field |
| C | `function_definition` | via `(function_declarator (identifier))` | `(storage_class_specifier)` | type specifier before declarator |
| C++ | `function_definition` | via `(function_declarator (identifier))` | `(storage_class_specifier)`, `virtual` | type specifier before declarator |
| Ruby | `method` | `name: (identifier)` | N/A | N/A (dynamic) |
| PHP | `function_definition` | `name: (name)` | N/A | `return_type:` field |
| C# | `method_declaration` | `name: (identifier)` | `(modifier)*` children | `returns:` field |
| Swift | `function_declaration` | `name: (simple_identifier)` | `(modifiers)` child | `(function_type)` child |

### 3.2 Class / Struct / Trait Equivalents

| Concept | Rust | Python | JS/TS | Go | Java | C++ | C# | Swift | Ruby | PHP |
|---------|------|--------|-------|-----|------|-----|-----|-------|------|-----|
| Class | N/A | `class_definition` | `class_declaration` | N/A | `class_declaration` | `class_specifier` | `class_declaration` | `class_declaration` | `class` | `class_declaration` |
| Struct | `struct_item` | N/A (dataclass) | N/A | `struct_type` (inside `type_spec`) | N/A | `struct_specifier` | `struct_declaration` | `class_declaration` (keyword "struct") | N/A | N/A |
| Interface | N/A | N/A (Protocol) | `interface_declaration` (TS only) | `interface_type` (inside `type_spec`) | `interface_declaration` | N/A (pure virtual class) | `interface_declaration` | `protocol_declaration` | N/A | `interface_declaration` |
| Trait | `trait_item` | N/A | N/A | N/A | N/A | N/A | N/A | `protocol_declaration` | N/A | N/A |
| Enum | `enum_item` | N/A | `enum_declaration` (TS) | N/A | `enum_declaration` | `enum_specifier` | `enum_declaration` | `class_declaration` (keyword "enum") | N/A | `enum_declaration` |

### 3.3 Visibility / Access Modifiers

| Language | Node Type | Values |
|----------|-----------|--------|
| Rust | `(visibility_modifier)` | `pub`, `pub(crate)`, `pub(super)`, `pub(in path)` |
| Python | N/A (convention) | `_name` (private), `__name` (mangled) |
| Java | Inside `(modifiers)` | `public`, `private`, `protected` (default=package) |
| C++ | `(access_specifier)` | `public:`, `private:`, `protected:` |
| C# | `(modifier)` | `public`, `private`, `protected`, `internal` |
| TypeScript | `(accessibility_modifier)` | `public`, `private`, `protected` |
| Go | Convention | Uppercase = exported, lowercase = unexported |
| Swift | `(modifiers)` | `public`, `private`, `internal`, `fileprivate`, `open` |
| PHP | `(visibility_modifier)` | `public`, `private`, `protected` |
| Ruby | Method calls | `public`, `private`, `protected` (method calls, not modifiers) |

### 3.4 Generic Type Parameter Syntax

| Language | Node Type | Example |
|----------|-----------|---------|
| Rust | `(type_parameters)` with `(constrained_type_parameter)` | `<T: Display + Clone>` |
| Java | `(type_parameters)` with `(type_parameter)` | `<T extends Comparable<T>>` |
| TypeScript | `(type_parameters)` | `<T extends string>` |
| Go | `(type_parameter_list)` | `[T comparable]` |
| C++ | `(template_parameter_list)` inside `(template_declaration)` | `template<typename T>` |
| C# | `(type_parameter_list)` with `(type_parameter_constraints_clause)` | `<T> where T : IDisposable` |
| Swift | `(type_parameters)` | `<T: Equatable>` |
| Python | `(type_parameter)` field on `function_definition` (3.12+) | `def foo[T](x: T) -> T:` |

### 3.5 Import System Differences

| Language | Node Type | Example |
|----------|-----------|---------|
| Rust | `use_declaration` | `use std::collections::HashMap;` |
| Python | `import_statement`, `import_from_statement` | `from pathlib import Path` |
| JavaScript | `import_statement` | `import { useState } from 'react';` |
| Go | `import_declaration` | `import "fmt"` |
| Java | `import_declaration` | `import java.util.HashMap;` |
| C/C++ | `preproc_include` | `#include <stdio.h>` |
| C# | `using_directive` | `using System.Collections.Generic;` |
| Ruby | `call` with `require`/`require_relative` | `require 'json'` |
| PHP | `use_declaration` (namespace), `expression_statement` (require) | `use App\Models\User;` |
| Swift | `import_declaration` | `import Foundation` |

### 3.6 Critical Grammar Gotchas

1. **Swift `class_declaration` covers class, struct, AND enum.** The grammar uses one node type with different keywords. v2.0.0 must check the first keyword child to distinguish them.

2. **Python `decorated_definition` wraps `function_definition`.** A decorated function is NOT a `function_definition` at the top level -- it is a `decorated_definition` that CONTAINS a `function_definition`. Queries must handle both.

3. **Go methods have receivers, not `self`.** The `method_declaration` has an explicit `receiver:` field with its own parameter list. This is fundamentally different from Rust/Python/Java where the receiver is implicit or part of the parameter list.

4. **C/C++ separate type from declarator.** Unlike most languages where `fn name(params) -> Type` puts everything in one node, C/C++ split the return type as a sibling before the function declarator: `int` `foo(int x)`. This means queries must access the type node as a sibling, not a field.

5. **Ruby uses `constant` for class/module names, not `identifier`.** Class names in Ruby are `(constant)` nodes, not `(identifier)` nodes.

6. **PHP uses `name` instead of `identifier` for function/class names.** The tree-sitter-php grammar uses a `(name)` node type, not `(identifier)`.

7. **TypeScript uses `type_identifier` for class/interface names** while JavaScript uses `identifier`. Despite sharing most grammar, they diverge here.

8. **Rust `function_modifiers` is a single node** containing keywords like `async`, `unsafe`, `const`, `extern`. These are anonymous children that must be checked individually.

---

## 4. tree-sitter-tags System

### 4.1 What Is Tagging?

Tagging identifies named entities in source code. The system uses tree-sitter queries with a standardized capture naming convention to produce definition/reference pairs. This powers GitHub's code navigation (jump-to-definition, find-references).

### 4.2 Capture Name Convention: @role.kind

The standard convention uses `@role.kind` capture names:

**Roles:**
- `definition` -- entity is being defined here
- `reference` -- entity is being used/called here

**Standard kinds:**
- `function` -- function or subroutine
- `method` -- method on a class/struct
- `class` -- class type
- `module` -- module or namespace
- `type` -- type definition
- `interface` -- interface or protocol
- `call` -- function/method call
- `implementation` -- trait/interface implementation

**Special captures:**
- `@name` -- always captures the identifier text of the entity
- `@doc` -- captures associated documentation string

### 4.3 Built-in Functions for Docstrings

The tagging system provides two built-in functions for cleaning doc comments:

- **`#strip!`** -- Removes a prefix string from each line of a captured node's text. Used to strip comment syntax (e.g., `#`, `//`, `///`).
- **`#select-adjacent!`** -- Selects only the nodes that are adjacent to the tagged definition, filtering out stray comments.

Example from Ruby:

```scheme
(
  (comment)* @doc
  .
  (method
    name: (identifier) @name) @definition.method
  (#strip! @doc "# ")
)
```

### 4.4 Tags Query File Location

- Expected at `queries/tags.scm` in each language grammar repository
- Can be tested with `tree-sitter tags <file>` CLI command
- Test files go under `test/tags/`

### 4.5 Relationship to v2.0.0

v1.x entity queries (in `entity_queries/*.scm`) already use the `@definition.*` / `@name` convention borrowed from tree-sitter-tags. v2.0.0 should:

1. **Adopt the full tags vocabulary** including `@reference.call`, `@reference.type`, etc.
2. **Add the `@doc` capture** to extract documentation alongside entities.
3. **Use `#strip!`** to clean doc comment prefixes automatically.
4. **Align capture names** with the standard so queries are portable across tools.

### 4.6 How Aider Uses Tags for Repo Maps

Aider (the AI coding assistant) uses tree-sitter tags to build repository maps. Its approach:

1. Parse every file with tree-sitter
2. Extract `@definition.*` and `@reference.call` captures using `tags.scm` queries
3. Build a graph: nodes = files, edges = reference relationships
4. Run PageRank to rank files by centrality
5. Output a token-budget-constrained symbol map

This is architecturally identical to what Parseltongue does, validating our approach. The difference: Parseltongue goes deeper (generics, bounds, cross-language edges, Ascent reasoning).

---

## 5. Incremental Parsing API

### 5.1 How Incremental Parsing Works

Tree-sitter's incremental parsing is a two-step process:

**Step 1: Edit the old tree.**

```rust
// Rust API (tree-sitter 0.25+)
let edit = InputEdit {
    start_byte: 10,
    old_end_byte: 15,
    new_end_byte: 20,
    start_position: Point { row: 0, column: 10 },
    old_end_position: Point { row: 0, column: 15 },
    new_end_position: Point { row: 0, column: 20 },
};
old_tree.edit(&edit);
```

This adjusts all node positions in the old tree to account for the edit. It does NOT re-parse; it just shifts byte offsets.

**Step 2: Re-parse with the edited old tree.**

```rust
let new_tree = parser.parse(new_source, Some(&old_tree))?;
```

When the parser receives the old tree, it reuses all subtrees that were not affected by the edit. Only nodes in the edited region (and their ancestors) are re-parsed. The new tree internally shares structure with the old tree.

### 5.2 Detecting Changed Ranges

After re-parsing, you can get the minimal set of changed ranges:

```rust
let changed_ranges = old_tree.changed_ranges(&new_tree);
for range in changed_ranges {
    // Only re-process these byte ranges
    println!("Changed: {} to {}", range.start_byte, range.end_byte);
}
```

This is critical for v2.0.0's file watcher: instead of re-extracting all entities from a changed file, we can re-extract only the entities whose byte ranges intersect with `changed_ranges`.

### 5.3 Performance Characteristics

- **Initial parse**: approximately 2-3x slower than a hand-written parser (e.g., rustc's parser)
- **Incremental re-parse**: typically **sub-millisecond** for small edits
- **Memory**: New trees share structure with old trees (copy-on-write semantics)
- **Thread safety**: Trees are immutable after creation; copying is O(1) (atomic refcount increment)

### 5.4 Integration with v2.0.0 File Watcher

v1.x's file watcher (in `crates/pt01-folder-to-cozodb-streamer/src/file_watcher.rs`) re-parses entire files on change. v2.0.0 should:

1. **Store the previous `Tree` for each watched file** in a `HashMap<PathBuf, Tree>`
2. **On file change notification**, compute the edit delta from the notification event
3. **Call `old_tree.edit(&input_edit)`** then `parser.parse(new_source, Some(&old_tree))`
4. **Use `changed_ranges()`** to identify which entities need re-extraction
5. **Re-run queries only on changed ranges** using `QueryCursor::set_byte_range()`
6. **Delta-update the Ascent fact store** instead of full re-ingestion

Expected speedup: For a typical single-function edit in a 1000-line file, incremental parsing should be **100-1000x faster** than full re-parse + full re-extraction.

### 5.5 QueryCursor Range Limiting

```rust
let mut cursor = QueryCursor::new();
// Only search within the changed byte range
cursor.set_byte_range(changed_start..changed_end);
let matches = cursor.matches(&query, new_tree.root_node(), source.as_bytes());
```

This limits the query engine to only visit nodes intersecting the specified range, further reducing work for incremental updates.

---

## 6. Language Injection

### 6.1 What Is Language Injection?

Language injection allows tree-sitter to parse embedded languages within a host language. Examples:
- SQL inside Rust string literals
- JavaScript inside HTML `<script>` tags
- CSS inside HTML `<style>` tags
- Regex inside string arguments
- SQL inside Python f-strings

### 6.2 How Injection Queries Work

Injection queries use special captures and directives:

```scheme
; SQL inside Rust raw strings that contain SQL keywords
(
  (raw_string_literal) @injection.content
  (#match? @injection.content "(SELECT|INSERT|UPDATE|DELETE|CREATE)")
  (#set! injection.language "sql")
)

; JavaScript inside HTML script tags
(script_element
  (raw_text) @injection.content
  (#set! injection.language "javascript"))

; CSS inside HTML style tags
(style_element
  (raw_text) @injection.content
  (#set! injection.language "css"))
```

The key captures and directives:
- `@injection.content` -- the text to parse with the embedded language parser
- `#set! injection.language "name"` -- which language parser to use
- `#offset!` -- adjust byte offsets to exclude quote characters

### 6.3 Relevance to v2.0.0 Cross-Language Edge Detection

Language injection directly enables Pattern 4 (gRPC/HTTP) and Pattern 5 (Message Queues) from Prep-Doc-V200.md Section 5:

**Pattern 4: HTTP/gRPC routes**

```rust
// Rust side: #[post("/api/v1/analyze")]
// Python side: requests.post("/api/v1/analyze", ...)
```

With injection, we can:
1. Extract all string literals matching URL patterns
2. Parse the URL path as a structured value
3. Match string literals across languages to create cross-language edges

**Pattern 5: Message queue topics**

```rust
// Rust: client.send_messages("user-events", ...)
// Java: consumer.subscribe("user-events")
```

Same approach: extract string arguments to known message queue functions, match across languages.

### 6.4 Implementation Strategy for v2.0.0

Rather than full language injection (parsing SQL AST inside Rust strings), v2.0.0 should use a lightweight approach:

1. **Extract string literals** from all 12 languages using the queries in Section 2.7
2. **Apply regex predicates** to classify strings (URL paths, SQL, topic names, etc.)
3. **Store classified strings as facts** in the Ascent reasoning engine
4. **Use Ascent rules** to match strings across languages and create cross-language edges

This avoids the complexity of managing multiple nested parsers while achieving the same edge detection capability.

### 6.5 Injection Query Locations

By convention, injection queries are stored at `queries/injections.scm` in each grammar repository. The parseltongue project could store custom injection queries at:

```
injection_queries/
  rust.scm       ; SQL in raw strings, regex in strings
  python.scm     ; SQL in strings, HTML in templates
  javascript.scm ; SQL in template literals, HTML in JSX
  html.scm       ; JS in <script>, CSS in <style>
```

---

## 7. Performance: Queries vs Cursor Walking

### 7.1 Summary of Tradeoffs

| Characteristic | Queries (S-expression) | Cursor Walking (Imperative) |
|---------------|----------------------|----------------------------|
| **Code volume** | ~5-15 lines per pattern | ~50-200 lines per extraction target |
| **Maintenance** | Declarative, easy to modify | Imperative, brittle to grammar changes |
| **Cross-language** | Same API, different .scm files | Different code per language |
| **Simple patterns** | Very fast (optimized in C) | Fast (manual traversal) |
| **Complex patterns** | Can degrade on large files | Consistent O(n) |
| **Bulk traversal** | Overhead per pattern | Single pass, zero overhead |
| **Incremental** | Range-limited via `set_byte_range` | Must implement manually |
| **Nested extraction** | Natural (nested S-expressions) | Manual stack management |

### 7.2 When Queries Are Faster

Queries excel when you need to find specific structural patterns scattered throughout a file. The query engine uses a state machine that processes the tree in a single pass, matching multiple patterns simultaneously.

For Parseltongue's use case (extracting specific node types like functions, structs, imports), queries are the right choice because:
- We extract ~10-15 distinct patterns per language
- Each pattern is structurally local (does not require global context)
- The query engine matches all patterns in a single tree traversal

### 7.3 When Cursor Walking Is Faster

Cursor walking wins for:
- **Full tree traversal** where you need to visit every node (e.g., word counting in v1.6.5 coverage metrics)
- **Complex structural analysis** requiring sibling relationships or cross-subtree correlation
- **Very large files** (>100KB) where query pattern explosion can cause exponential blowup

### 7.4 Performance Pitfall: Mixing APIs

A known pitfall: mixing `ts_node_*` APIs with `TreeCursor` APIs causes performance degradation. The cursor maintains an internal path stack; node APIs bypass it and force re-computation. v2.0.0 must use either queries OR cursor, never mix them in the same extraction pass.

### 7.5 v2.0.0 Hybrid Strategy

Based on the evidence, v2.0.0 should use:

1. **Queries** for all entity/dependency extraction (functions, structs, imports, calls, etc.)
2. **Queries** for attribute/decorator/modifier extraction
3. **Queries** for string literal extraction (cross-language edges)
4. **Cursor walking** for coverage word counting (needs full traversal)
5. **Cursor walking** for fallback when a query pattern cannot express the needed logic

The v1.x codebase already demonstrates this hybrid approach: `query_extractor.rs` uses queries for entity extraction, while `count_top_level_comment_words()` and `compute_import_word_count_safely()` use cursor walking.

### 7.6 Benchmark Expectations

Based on v1.x benchmarks (from `streamer.rs` comments):

```
v1.x performance (302 files, 3845 entities):
  Sequential: 5.4s
  Parallel (Rayon): 1.8s

v2.0.0 expected (same codebase, enriched extraction):
  Initial parse: ~same (parsing is the bottleneck, not extraction)
  Query execution: +10-20% overhead for richer queries (~2.0s parallel)
  Incremental re-parse: <10ms per file change
```

The enriched extraction in v2.0.0 (visibility, modifiers, generics, attributes, return types, params) should add minimal overhead because:
- All new data is captured in the SAME query pass (additional captures, not additional passes)
- The query engine processes all patterns simultaneously
- The overhead is in result processing, not tree traversal

---

## 8. Current v1.x Query Inventory and Gaps

### 8.1 What v1.x Entity Queries Extract

Current entity queries (in `entity_queries/*.scm`) extract only the skeleton:

| Language | Entities Extracted | Captures Used |
|----------|--------------------|---------------|
| Rust | function, struct, enum, trait, impl, method, module | `@name`, `@definition.*` |
| Python | class, method, function | `@name`, `@definition.*` |
| JavaScript | function, generator, arrow, class, method | `@name`, `@definition.*` |
| TypeScript | function, arrow, class, interface, type_alias, enum, method | `@name`, `@definition.*` |
| Go | function, method, struct, interface | `@name`, `@definition.*` |
| Java | class, interface, enum, method, constructor | `@name`, `@definition.*` |
| C++ | function, class, struct, enum | `@name`, `@definition.*` |
| C# | class, interface, struct, enum, method, property | `@name`, `@definition.*` |
| Swift | function, class (all types), protocol | `@name`, `@definition.*` |
| Ruby | class, module, method, singleton_method | `@name`, `@definition.*` |
| PHP | class, interface, function, method, trait, namespace | `@name`, `@definition.*` |
| C | function, struct, enum, typedef | `@name`, `@definition.*` |

### 8.2 What v1.x Dependency Queries Extract

Current dependency queries (in `dependency_queries/*.scm`) extract call/use/implements edges. The Rust dependency query is the most complete (8 dependency types). Other languages have 3-7 dependency types.

### 8.3 What v1.x Throws Away (the Gap)

From Prep-Doc-V200.md Section 3, here is the complete inventory of data available in tree-sitter that v1.x ignores:

| Data | Available In Grammar | v1.x Status | v2.0.0 Priority |
|------|---------------------|-------------|-----------------|
| Visibility modifiers | All languages | **IGNORED** | P0 -- Critical for API surface analysis |
| Function modifiers (async/unsafe/const) | Rust, Python, JS/TS | **IGNORED** | P0 -- Critical for safety analysis |
| Return types | All typed languages | **IGNORED** | P0 -- Creates dependency edges |
| Parameter types | All typed languages | **IGNORED** | P0 -- Creates dependency edges |
| Generic type parameters | Rust, Java, TS, Go, C++, C# | **IGNORED** | P0 -- Creates trait bound edges |
| Where clauses | Rust, C# | **IGNORED** | P0 -- Creates trait bound edges |
| Attributes / decorators | Rust, Python, Java, C#, Swift | **IGNORED** | P0 -- FFI/WASM/PyO3 detection |
| Trait bounds / supertraits | Rust | **IGNORED** | P0 -- Supertrait hierarchy |
| Doc comments | All languages | **IGNORED** | P1 -- LLM context enrichment |
| Closure expressions | Rust, Python, JS, Java, Go, Ruby | **IGNORED** | P1 -- Call graph completeness |
| String literals | All languages | **IGNORED** | P1 -- Cross-language edge detection |
| Lifetime annotations | Rust | **IGNORED** | P2 -- Data flow analysis |
| Macro invocations | Rust | **IGNORED** | P2 -- Derive macro = trait impls |
| Error nodes | All languages | **IGNORED** | P2 -- Code quality signal |

### 8.4 Gap Quantification

v1.x extracts approximately **20% of the structural information** available in tree-sitter parse trees. The remaining 80% is walked past by the cursor (before v1.x switched to queries) or left uncaptured (current queries).

v2.0.0's enriched queries should capture approximately **70-80%** of structural information. The remaining 20-30% (macro expansion, lifetime inference, control flow) requires semantic analysis (rust-analyzer) and is the responsibility of `rust-llm-03-rust-analyzer`.

---

## 9. v2.0.0 Implementation Plan

### 9.1 Query File Organization

```
crates/rust-llm-01-fact-extractor/queries/
  entities/
    rust.scm          ; Functions, structs, enums, traits, impls, modules
    python.scm        ; Functions, classes, methods
    javascript.scm    ; Functions, classes, methods
    typescript.scm    ; Functions, classes, interfaces, type aliases, enums
    go.scm            ; Functions, methods, structs, interfaces
    java.scm          ; Classes, interfaces, enums, methods
    cpp.scm           ; Functions, classes, structs, enums, namespaces
    c.scm             ; Functions, structs, enums, typedefs
    csharp.scm        ; Classes, interfaces, structs, enums, methods
    swift.scm         ; Functions, classes, protocols, enums
    ruby.scm          ; Classes, modules, methods
    php.scm           ; Classes, interfaces, traits, functions, methods
  dependencies/
    rust.scm          ; Calls, uses, implements, type refs, generics
    python.scm        ; Imports, calls, inheritance, decorators, type hints
    javascript.scm    ; Imports, requires, calls, inheritance
    typescript.scm    ; Imports, calls, inheritance, type refs
    go.scm            ; Imports, calls, interface satisfaction
    java.scm          ; Imports, calls, extends, implements, annotations
    cpp.scm           ; Includes, calls, inheritance, templates
    c.scm             ; Includes, calls, type refs
    csharp.scm        ; Usings, calls, inheritance, attributes
    swift.scm         ; Imports, calls, protocol conformance
    ruby.scm          ; Requires, calls, include/extend
    php.scm           ; Uses, requires, calls, implements
  metadata/
    rust.scm          ; Attributes, doc comments, visibility, modifiers
    python.scm        ; Decorators, docstrings, type hints
    <one per language>
  cross_language/
    string_literals.scm  ; URL paths, SQL, topic names (shared patterns)
    ffi_markers.scm      ; extern "C", #[wasm_bindgen], #[pyfunction]
```

### 9.2 Fact Types Extracted Per Query Category

**Entity queries** produce:
```rust
struct EntityFact {
    name: String,
    entity_type: EntityType,  // Function, Struct, Class, etc.
    language: Language,
    file_path: PathBuf,
    byte_range: Range<usize>,
    line_range: Range<u32>,
}
```

**Metadata queries** produce (NEW in v2.0.0):
```rust
struct MetadataFact {
    entity_ref: EntityRef,     // Points to parent EntityFact
    visibility: Option<Visibility>,
    modifiers: Vec<Modifier>,  // async, unsafe, const, static, abstract
    return_type: Option<String>,
    parameters: Vec<Parameter>,
    generics: Vec<GenericParam>,
    where_clauses: Vec<WherePredicate>,
    attributes: Vec<Attribute>,
    doc_comment: Option<String>,
}
```

**Dependency queries** produce:
```rust
struct DependencyFact {
    from_entity: EntityRef,
    to_name: String,           // Unresolved name
    edge_type: EdgeType,       // Calls, Uses, Implements
    source_location: SourceLocation,
}
```

**Cross-language queries** produce (NEW in v2.0.0):
```rust
struct CrossLangFact {
    string_value: String,      // The literal string content
    classification: StringClass, // UrlPath, SqlQuery, TopicName, FfiSymbol
    context: CallContext,       // What function/method contains this string
    file_path: PathBuf,
    language: Language,
}
```

### 9.3 Migration Path from v1.x Queries

| Step | Action | Files Affected |
|------|--------|---------------|
| 1 | Copy existing `entity_queries/*.scm` as baseline | 12 files |
| 2 | Add visibility, modifier, return_type captures to entity queries | 12 files |
| 3 | Add generic_param, where_clause captures | Rust, Java, TS, Go, C++, C# (6 files) |
| 4 | Create metadata queries for attributes/decorators | 12 new files |
| 5 | Enhance dependency queries with type reference edges | 12 files |
| 6 | Add doc comment extraction queries | 12 files |
| 7 | Create cross-language string literal queries | 2 new files |
| 8 | Add closure/lambda extraction queries | 8 files (languages with closures) |
| 9 | Implement `QueryBasedExtractor` v2 with multi-capture processing | 1 Rust file |
| 10 | Implement incremental parsing integration | 1 Rust file |

### 9.4 Testing Strategy

Each `.scm` query file should have a corresponding test fixture:

```
crates/rust-llm-01-fact-extractor/test-fixtures/
  rust/
    function_signatures.rs       ; All function modifier combinations
    struct_generics.rs           ; Structs with generic bounds
    impl_blocks.rs               ; Trait impls and inherent impls
    attributes.rs                ; All attribute patterns
    imports.rs                   ; All use declaration patterns
    closures.rs                  ; Closure expressions
    doc_comments.rs              ; Doc comment patterns
    string_literals.rs           ; Cross-language string patterns
  python/
    function_decorators.py       ; Decorated functions with type hints
    class_inheritance.py         ; Classes with multiple bases
    <etc>
```

Each test fixture is a minimal source file containing all variations of a construct. Tests parse the fixture and verify that all expected captures are present with correct text content.

### 9.5 Performance Budget

| Operation | Budget | Rationale |
|-----------|--------|-----------|
| Parse 1K LOC file | <20ms (release) | Same as v1.x |
| Entity + metadata extraction | <5ms per file | All done in single query pass |
| Dependency extraction | <5ms per file | Single query pass |
| Cross-language string extraction | <2ms per file | Simple pattern matching |
| Incremental re-parse | <1ms per edit | Tree-sitter guarantee |
| Incremental re-extraction | <2ms per edit | Range-limited query |
| Total per file (initial) | <30ms | Comparable to v1.x |
| Total per file (incremental) | <5ms | 6x improvement over v1.x |

---

## Sources

- [Tree-sitter Query Syntax (Official)](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html)
- [Tree-sitter Code Navigation / Tags](https://tree-sitter.github.io/tree-sitter/4-code-navigation.html)
- [Tree-sitter CLI Tags Command](https://tree-sitter.github.io/tree-sitter/cli/tags.html)
- [Tree-sitter Advanced Parsing (Incremental)](https://tree-sitter.github.io/tree-sitter/using-parsers/3-advanced-parsing.html)
- [tree-sitter-rust Grammar](https://github.com/tree-sitter/tree-sitter-rust)
- [tree-sitter-python Grammar](https://github.com/tree-sitter/tree-sitter-python)
- [tree-sitter-python tags.scm](https://github.com/tree-sitter/tree-sitter-python/blob/master/queries/tags.scm)
- [Tips for Using Tree-Sitter Queries (Cycode)](https://cycode.com/blog/tips-for-using-tree-sitter-queries/)
- [Knee Deep in tree-sitter Queries (Parsiya)](https://parsiya.net/blog/knee-deep-tree-sitter-queries/)
- [Tree-sitter Query Performance Discussion #2018](https://github.com/tree-sitter/tree-sitter/discussions/2018)
- [Tree-sitter Traversal Optimization Discussion #878](https://github.com/tree-sitter/tree-sitter/discussions/878)
- [Incremental Queries Discussion #1976](https://github.com/tree-sitter/tree-sitter/discussions/1976)
- [Incremental Parsing Using Tree-sitter (Strumenta)](https://tomassetti.me/incremental-parsing-using-tree-sitter/)
- [Tree-sitter Language Injection Blog](https://blog.achims.world/tree-sitter-adventures-language-injections.html)
- [SQL Injection Query Discussion #1577](https://github.com/tree-sitter/tree-sitter/discussions/1577)
- [tree-sitter-highlight Crate](https://crates.io/crates/tree-sitter-highlight)
- [Query struct in tree-sitter Rust docs](https://docs.rs/tree-sitter/latest/tree_sitter/struct.Query.html)
- [DeepWiki: tree-sitter-java Query System](https://deepwiki.com/tree-sitter/tree-sitter-java/4.2-query-system)
- [RFC 001: Tree-sitter Based File-Incremental Indexing](https://github.com/orgs/sheeptechnologies/discussions/4)
- [Code Retrieval in Coding Agents (Preprint)](https://www.preprints.org/manuscript/202510.0924)
