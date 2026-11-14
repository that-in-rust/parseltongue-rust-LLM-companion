use crate::{SyntaxFileInputData, check_syntax_item_node, restore_source_byte_offset};
use ra_ap_syntax::{AstNode, SyntaxKind, SyntaxNode, ast};
use serde_json::{Value, json};

#[cfg(test)]
#[path = "signature-type-edge-tests.rs"]
mod signature_type_edge_tests;

fn check_signature_row_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::RECORD_FIELD
            | SyntaxKind::TUPLE_FIELD
            | SyntaxKind::PARAM
            | SyntaxKind::SELF_PARAM
            | SyntaxKind::RET_TYPE
            | SyntaxKind::TYPE_PARAM
            | SyntaxKind::CONST_PARAM
            | SyntaxKind::WHERE_PRED
            | SyntaxKind::TYPE_BOUND
    )
}

/// The type paths one signature row names, each with its own source range.
/// A path is listed on the nearest signature row that holds it, so a bound's
/// path is on the bound row and not also on its type parameter. Paths under an
/// expression (an array length, a const argument) are not listed.
/// `Value::Null` when the row is not in signature position: it sits under an
/// expression of its owning item, such as a closure parameter.
pub(super) fn extract_signature_type_paths(
    input: &SyntaxFileInputData,
    node: &SyntaxNode,
    removed: &[usize],
) -> Value {
    if !check_signature_row_kind(node.kind())
        || node
            .ancestors()
            .skip(1)
            .take_while(|owner| !check_syntax_item_node(owner))
            .any(|owner| ast::Expr::can_cast(owner.kind()))
    {
        return Value::Null;
    }
    node.descendants()
        .filter(|path| path.kind() == SyntaxKind::PATH_TYPE)
        .filter(|path| {
            let mut holders = path.ancestors().skip(1);
            let nearest = holders.by_ref().find(|holder| {
                check_signature_row_kind(holder.kind()) || ast::Expr::can_cast(holder.kind())
            });
            nearest.as_ref() == Some(node)
        })
        .map(|path| {
            let range = path.text_range();
            let start = restore_source_byte_offset(range.start(), removed);
            let end = restore_source_byte_offset(range.end(), removed);
            let text = std::str::from_utf8(&input.file_source_content_bytes[start..end]).ok();
            json!({"span_byte_start_offset":start,"span_byte_end_offset":end,
                "type_source_text":text,"type_path_self_flag":text == Some("Self")})
        })
        .collect()
}
