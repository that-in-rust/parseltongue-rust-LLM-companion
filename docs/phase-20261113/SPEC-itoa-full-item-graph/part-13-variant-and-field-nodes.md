# Part 13 — enum variants and struct fields are nodes

Back to the [index](README.md).

Status on 2026-11-18: **specified, not built.** Picked by the user on 2026-11-17 ("who
uses this field is a common question"). The form is decided by Rule 6: a node, because it
tells a reader more than a hidden record. This closes OD-4 of the backlog.

**The defect.** A field and a variant are not in the graph. The type a field writes is
stored as an edge from the whole struct, so a reader cannot tell which field uses it.

**What this part adds on pinned itoa** (itoa has two fields and no enum):

```text
nodes     Buffer::bytes      src/lib.rs line 73    bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN]
          DecimalPairs::0    src/lib.rs line 219   [u8; 200]
contains  Buffer --> Buffer::bytes          DecimalPairs --> DecimalPairs::0
4 rows change their source from the struct to the field:
          Buffer::bytes    uses_type  core::mem::MaybeUninit
          Buffer::bytes    uses_type  u8
          Buffer::bytes    reads      <i128 as Integer>::MAX_STR_LEN
          DecimalPairs::0  uses_type  u8
```

Counts after it: 172 nodes and 686 edge rows for scope `declarations`, 153 nodes for
scope `excluded` (predicted; measured in section 6 when built). Both fields are private
and not reachable from outside the crate.

**Not in this part** (backlog): an edge for each *use* of a field (`self.bytes`,
`DECIMAL_PAIRS.0`: 25 source lines in itoa) and for a field named in a struct literal.
The run stores no site for a field access that is not the receiver of a method call.

## 1. Rust work mode

Spec Mode and Delivery Mode. Component: the syntax stage (`parseltongue-syntax-span-extractor`),
the node projection (`parseltongue-duckdb-analysis-adapter`), the node rules
(`parseltongue-core-row-types`) and the two codecs of the owner kind.

Found with codebase-memory on 2026-11-18 (`trace_path`, inbound, depth 1):

- `check_syntax_item_node` decides which syntax nodes get a span row and which node owns
  a site. Callers: `extract_file_syntax_rows` (span rows, site owners),
  `extract_impl_header_syntax`, `extract_macro_context_syntax`,
  `read_macro_argument_paths`, `check_call_lexical_shadowing`,
  `check_receiver_name_rebound`. It accepts a variant already, not a field.
- `decode_original_item_kinds` maps a span and its owner to an item kind and an owner
  kind. Callers: `derive_original_graph_input` and `build_generated_item_pair`. It has
  no arm for a variant, a field, or a struct, enum or union as owner.
- Both projection SQL files set variants and fields aside as `deferred-member`.
- `validate_graph_input_records` rejects a local item of kind `Field` or `EnumVariant`.

## 2. Executable requirements: REQ-RUST-071.0

```text
N1  WHEN an active record field, tuple field or enum variant is written outside a function
    body THEN it SHALL be a node of kind Field or EnumVariant, with owner kind Struct,
    Union, Enum or EnumVariant
N2  WHEN a field or variant is a node THEN its label SHALL be the label of its owner, `::`
    and its name; the name of a tuple field SHALL be its position, from 0, among the tuple
    fields of its list that the configuration keeps
N3  WHEN a field or variant is inside a function body, or the configuration removes it
    THEN it SHALL not be a node (the rules for every other item)
N4  WHEN a node has owner kind Struct, Union or EnumVariant THEN its kind SHALL be Field;
    with owner kind Enum it SHALL be EnumVariant; a Field or EnumVariant SHALL have no
    other owner kind. A graph that breaks this SHALL be rejected with InvalidNodeCategory
C1  WHEN a field or variant is a node THEN one `contains` row SHALL run to it from its
    struct, union, enum or variant, and none from a module or file area
E1  WHEN a type or a read is written inside a field THEN the row's source SHALL be the
    field; inside a variant and outside its fields, the variant
F1  WHEN a node is a field of a struct or union THEN its visibility SHALL be the one it
    declares (part 11 V1). WHEN it is a variant or a field of a variant THEN it SHALL take
    the visibility of its enum
F2  WHEN a field or variant is Public THEN it SHALL be reachable exactly when its owner is
    (part 11 R2)
F3  WHEN a node is a field THEN its signature SHALL be its tokens without attributes:
    `bytes: [MaybeUninit<u8>; i128::MAX_STR_LEN]`, `pub x: T`, `[u8; 200]`. A variant's
    SHALL be its name, without its fields and without `=` and a discriminant
G1  WHEN a macro expansion writes a field or variant THEN N1 to F3 SHALL hold for it, and
    its macro call SHALL `generates` it like any other item
A1  WHEN the mapper runs on pinned itoa THEN the stored graph SHALL equal the re-frozen
    expected graph: the 2 nodes, the 2 `contains` rows and the 4 moved rows above
```

## 3. Design (L1 / L2 / L3)

- **L1, `parseltongue-core-row-types`.** `HldOwnerKindValue` gains `Struct`, `Enum`,
  `Union`, `EnumVariant`. `validate_graph_input_records` enforces N4 in place of the
  blanket rejection. `NodeReachOwnerValue` gains `VariantMember(parent)`: a variant, or a
  field of one, takes the visibility and reachability of its parent (F1, F2). A field of
  a struct is the existing `Container(parent)` with the struct as parent.
- **L2, the syntax stage.** `check_syntax_item_node` accepts `RECORD_FIELD` and
  `TUPLE_FIELD`. The span row of a tuple field stores its position as its declared name
  (N2). A site that is itself a field is owned by that field, not by its struct (E1).
  `print_item_signature_text` cuts a variant at `=` (F3).
- **L2, `resolve_node_visibility_fields`.** Owner kind `Enum` or `EnumVariant` gives
  `VariantMember`, `Struct` or `Union` gives `Container`, each with the source of the
  node's `contains` row.
- **L3, the DuckDB projection.** Both SQL files include variants and fields, and take a
  struct, union, enum or variant as container and as a label segment.
  `decode_original_item_kinds` gets the arms of N4; any other pair stays
  `UnsupportedActiveSyntax`. The Turso and DuckDB codecs of the owner kind gain the four
  values.

No new trait, table or stage. No new error variant.

## 4. Verification matrix

| req_id | test_id | test_type | assertion | metric |
| --- | --- | --- | --- | --- |
| N4 | `test_member_owner_kind_pairs` | unit, core | each allowed pair passes, each other pair is `InvalidNodeCategory` | 0 wrong |
| F1 F2 | `test_variant_member_reach_rules` | unit, core | variant and variant field take the enum's fields; struct field keeps its own | 0 wrong |
| N1 N2 E1 | `test_field_span_rows_written` | unit, extractor | span rows, tuple position names, and site owners of a fixture | 0 wrong |
| F3 | `test_field_signature_text_printed` | unit, extractor | field and variant signatures | 0 wrong |
| N1 C1 G1 | `test_member_nodes_are_projected` | fixture, adapter | nodes, owner kinds, labels and `contains` rows of a struct, union and enum | 0 wrong |
| A1 | `test_itoa_declarations_graph_equality`, `test_itoa_whole_graph_equality` | real source | stored graph equals the re-frozen expected graph | 0 differing rows |

## 5. Implementation plan

1. Expected graph: the 2 nodes, 2 `contains` rows and 4 moved rows, from source; re-freeze.
2. See the two equality tests fail on those rows.
3. Core: owner kinds, N4, `VariantMember`, codecs; unit tests first.
4. Syntax stage: field spans, tuple names, site owner, variant signature; unit tests first.
5. Projection SQL and kinds, the visibility pass; fixture test first; equality tests pass.
6. Gates, a timed run, the documents and the log.

## 6. Quality gate results

Not run: nothing is built.

## 7. Open questions

- **Field uses.** An edge from a function to each field it reads or writes needs a stored
  site for every field access and the type of its receiver. Backlog.
- **Label clash.** A field and a method of one type may share a name (`len`), and then
  share a label. The node's identity and kind differ; the label alone does not tell them
  apart. itoa has no such pair.
