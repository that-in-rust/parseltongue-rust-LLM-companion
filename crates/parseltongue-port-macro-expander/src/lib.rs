use parseltongue_core_row_types::{
    DeclarativeMacroExpansionInput, DeclarativeMacroExpansionOutput, MacroExpansionFailureKind,
    RunCancelFlagValue,
};

pub fn expand_declarative_macro_items(
    _macro_expansion_input_rows: &DeclarativeMacroExpansionInput,
    _stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<DeclarativeMacroExpansionOutput, MacroExpansionFailureKind> {
    todo!()
}
