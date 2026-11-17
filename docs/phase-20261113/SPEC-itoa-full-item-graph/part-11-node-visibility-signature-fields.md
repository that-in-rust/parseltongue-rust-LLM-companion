# Part 11 — visibility, reachability and signature on every node

Back to the [index](README.md).

Status on 2026-11-17: **specified, not built.** Picked by the user on 2026-11-17 ("is it
public API, and what does it take and return"). Built after part 12, whose edge from a
module to its file area this part walks.

**The defect.** Every item node stores visibility `Unknown`, no reachability and no
signature. A reader must open the source for each node to learn whether it is public API
and what it takes and returns.

**The change.** Three fields of the node row are filled from source, and compared with
the expected graph:

```text
field                               itoa, 170 nodes
node_effective_visibility_kind      46 Public, 26 PrivateEquivalent, 1 RestrictedNonPrivate,
                                    15 NotApplicable (macro calls), 82 without a value
node_reachable_outside_crate_flag   19 true, 69 false, 82 without a value
node_signature_text                 88 with a text (the items of itoa), 82 without
```

The 82 nodes without a value are the crate root, the 2 file areas, the 66 names outside
itoa and the 13 builtin types: none of them has a declaration the run reads.

**One correction to the expected graph (2026-11-17).** It listed the 2 file areas as
public and not reachable. Nothing in the source declares that: a file area is not an
item. They now have no value, which is also what the node rules in
`parseltongue-core-row-types` already require of a file-area node.

**One limit, stated so the flag is not over-read.** "Reachable" means: can be named by a
public path from the crate root. The methods of `private::Sealed` are stored as not
reachable, because `private` is a private module. Outside code can still call them
through a bound `I: Integer`, since `Sealed` is a supertrait of the public `Integer`.
The flag does not model that route.

## 1. Rust work mode

- **Component:** the syntax extractor (signature text), the DuckDB node projection
  (declared visibility and signature onto the node), and one new pass over the finished
  graph (effective visibility and reachability).
- **Modes:** Spec and Delivery. No row is added; three fields of each node row change, so
  both expected graphs are re-frozen.
- **Measured before writing** (2026-11-17, run of commit `b32b47200`):
  - Each stored span already carries `span_visibility_source_text` (`pub`, `pub(crate)`,
    or `private` when none is written): `syntax-span-extractor/src/lib.rs`, line 534.
  - The node row has the three fields (`HldNodeInputRecord`), and the store and the
    DuckDB loader carry them; the projection writes `Unknown`, none, none.
  - `impl-header-index-table` lists, per impl header, its members and the resolution of
    its self type and trait. `resolve_impl_header_edges` reads it.
  - The expected graph already has `effective_visibility` and `reachable_outside_crate`
    on its 170 nodes, written from source on 2026-11-16 and never compared. It has no
    signature.
- **codebase-memory:** `trace_path publish_full_graph_checkpoint` outbound (17 callees:
  `derive_original_graph_input`, the five `resolve_*_edges`, the two `collect_*_nodes`,
  `write_full_graph_input`); `trace_path derive_original_graph_input` inbound.

## 2. Executable requirements: REQ-RUST-069.0

Visibility (the value stored is the effective one):

```text
V1  WHEN an item is written with `pub` THEN its declared visibility SHALL be Public; with
    `pub(crate)`, `pub(super)` or `pub(in path)` RestrictedNonPrivate; with `pub(self)`,
    `pub(in self)` or no visibility PrivateEquivalent
V2  WHEN an item is a member of a trait THEN it SHALL store the trait's visibility
V3  WHEN an item is a member of a trait impl THEN it SHALL store the trait's visibility:
    Public when the trait is a name outside the read source, Unknown when the trait of
    the impl is not resolved
V4  WHEN an item is a member of an inherent impl, or any other item THEN it SHALL store
    its declared visibility
V5  WHEN a node is a macro call THEN it SHALL store NotApplicable
V6  WHEN a node is the crate root, a file area, a name outside the read source or a
    builtin type THEN it SHALL store no visibility, no reachability and no signature
V7  WHEN a `macro_rules!` definition carries `#[macro_export]` THEN it SHALL store Unknown
    and no reachability (the export rule is not built); otherwise PrivateEquivalent
```

Reachability (can be named by a public path from the crate root):

```text
R1  WHEN a node's visibility is not Public or Unknown THEN the flag SHALL be false, a
    macro call included; with Unknown it SHALL have no value
R2  WHEN a node is Public THEN the flag SHALL be true exactly when its parent is reachable.
    The parent is: the trait, for a trait member; the self type, for a member of an
    inherent impl; the trait and the self type both, for a member of a trait impl; for any
    other item the module it is written in (the module that contains its file area, or
    its inline module)
R3  WHEN the parent is the crate root, a name outside the read source or a builtin type
    THEN it SHALL count as reachable
R4  WHEN a parent of R2 is not resolved to a node THEN the flag SHALL have no value
R5  WHEN the graph has a `use` item whose visibility is not PrivateEquivalent THEN a Public
    node whose parent is not reachable SHALL have no value instead of false (a re-export
    may name it; re-exports are not followed in this part). itoa has no such `use`
```

Signature:

```text
S1  WHEN an item is a node THEN its signature SHALL be the item's own tokens, without
    attributes, comments and white space, and without: the body of a function, the item
    list of a module, trait or impl, the field or variant list of a struct, union or enum (record and tuple alike),
    the body of a `macro_rules!` definition, the `=` and value of a const or static, and a
    final `;`
S2  WHEN the tokens are printed THEN one space SHALL separate two tokens, except:
    no space after   ( [ { < & * :: # . ? .. ..=   and after a prefix operator
    no space before  ) ] } > , ; : :: . .. ..=
    no space before  !  that follows a name
    no space before  <  that follows a name, `Self`, `impl` or `for`
    no space before  (  that follows a name, `self`, `Self`, `super`, `crate`, `pub`,
                        `fn`, `)`, `]`, `>` or `!`
    no space before  [  that follows a name, `)`, `]`, `#` or `!`
    `<` `>` `&` `*` written in a binary expression keep a space on both sides
    A trailing comma is kept: no token is dropped or added
S3  WHEN the same item is written in source and in a macro expansion with the same tokens
    THEN both SHALL store the same signature
```

Acceptance:

```text
A1  WHEN a run of pinned itoa is compared with the re-frozen expected graph THEN the three
    fields of each of its nodes SHALL equal the expected node's, for scope `declarations`
    (170 nodes) and scope `excluded` (151 nodes)
A2  WHEN that run has ended THEN no node of itoa SHALL store Unknown
A3  WHEN the graph is read twice, or built with 1, 4 and 16 workers THEN the fields SHALL
    be equal
```

Out of scope: following `pub use` re-exports (R5 stores "no value" instead), the
supertrait route named above, `#[macro_export]` (V7), `#[doc(hidden)]`.

## 3. Design (L1 / L2 / L3)

- **L1 (row types, no dependency):**
  - `HldEffectiveVisibilityKind` gains the variant `NotApplicable`.
  - A new pure function `derive_node_reach_fields` takes, per node, its declared
    visibility and its parents as an enum (`Module`, `Trait`, `InherentSelf`,
    `TraitImpl { trait, self }`, each parent `Node`, `Outside` or `Unresolved`), and
    returns the effective visibility and the flag. No store, no JSON. A parent cycle
    cannot be built from source containment; the function still ends on one, with no
    value.
- **L2 (syntax extractor):**
  - `print_item_signature_text` (pure): from the item's syntax node to the text of S1 and
    S2. It runs where the span row is built, for source and for expanded text alike, and
    adds the key `span_signature_source_text` to the span row.
  - `resolve_node_visibility_fields`: reads `impl-header-index-table`, maps each member to
    its header's trait and self type with the helpers `resolve_impl_header_edges` uses,
    reads the `contains` rows of the graph, calls the L1 function, and returns the nodes
    with the two fields set.
- **L3 (DuckDB adapter, checkpoint):**
  - The node projection copies the declared visibility (V1) and the signature from the
    span row onto the node, for source-written and generated items.
  - `publish_full_graph_checkpoint` calls `resolve_node_visibility_fields` after all edges
    are resolved and before the graph is written.
  - The Turso codec learns the new variant.
- **Errors:** typed, one enum for the new pass (`NodeVisibilityFieldError`: cancelled,
  invalid scope, invalid stored row with table and field).

## 4. Verification matrix

```text
req        test                                    type          assertion
V1-V7      test_node_reach_field_rules             unit (L1)     one case per rule, incl. R4, R5
R1-R5      test_node_reach_field_rules             unit (L1)
S1 S2      test_item_signature_text_printed        unit (L2)     fn, struct, trait, const, static,
                                                                 type alias, module, use, macro
S3         test_expanded_signature_text_equal      unit (L2)     source text and token-spaced
                                                                 expansion text print the same
A1 A2      test_itoa_declarations_graph_equality   real source   three more fields per node row
A1         test_itoa_whole_graph_equality          real source
A3         test_generated_nodes_read_twice and the S4 runs       real source
```

## 5. Implementation plan

```text
11.1  expected: a signature per item node written from source in the builder, checked
      against the source text; both graphs re-frozen
11.2  the two equality tests compare the three fields; seen failing on them
11.3  signature printer and span key (L2), with its unit tests
11.4  NotApplicable, the L1 function and its unit tests
11.5  projection, the new pass, the checkpoint call; the equality tests pass
11.6  gates, log entry
```

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

1. The supertrait route (top of this file): should a member of a private supertrait of a
   reachable trait count as reachable? Stored as not reachable until decided.
