; C# Dependency Queries (v0.9.0)

; Method calls
(invocation_expression
  function: (identifier) @reference.call) @dependency.call

(invocation_expression
  function: (member_access_expression
    name: (identifier) @reference.method_call)) @dependency.method_call

; Using directives
(using_directive
  (qualified_name) @reference.using) @dependency.using

; Class inheritance
(class_declaration
  name: (identifier) @definition.class
  (base_list
    (identifier) @reference.extends)) @dependency.extends

; Interface implementation
(class_declaration
  name: (identifier) @definition.class
  (base_list
    (identifier) @reference.implements)) @dependency.implements

; Constructor calls (Bug 2 Step 2)
; Detects: new TypeName(), new List<string>(), new Type { Prop = value }
(object_creation_expression
  type: [
    (identifier) @reference.constructor
    (qualified_name) @reference.constructor_qualified
    (generic_name
      (identifier) @reference.constructor_generic)
  ]) @dependency.constructor

; Property access (non-call) - REQ-CSHARP-002.0
; Detects: user.Name, config.Settings.Timeout, user.Age = 30
; Note: This also catches chained property access like config.Settings.Timeout
(member_access_expression
  name: (identifier) @reference.property_access) @dependency.property_access

; LINQ operations - REQ-CSHARP-003.0
; Detects: Where, Select, OrderBy, First, Count, etc.
(invocation_expression
  function: (member_access_expression
    name: (identifier) @reference.linq_method
    (#match? @reference.linq_method "^(Where|Select|First|FirstOrDefault|Any|All|Count|Sum|Average|OrderBy|OrderByDescending|ThenBy|ThenByDescending|GroupBy|ToList|ToArray|ToDictionary|Single|SingleOrDefault|Last|LastOrDefault|Take|Skip|TakeWhile|SkipWhile|Distinct|Union|Intersect|Except|Concat|Join|GroupJoin|Aggregate|Min|Max|Contains|SequenceEqual|Zip|DefaultIfEmpty|ElementAt|ElementAtOrDefault|Reverse|Cast|OfType)$"))) @dependency.linq

; Async/await - REQ-CSHARP-004.0
; Detects: await FetchAsync(), await this.SaveAsync()
(await_expression
  (invocation_expression) @reference.await_call) @dependency.await
