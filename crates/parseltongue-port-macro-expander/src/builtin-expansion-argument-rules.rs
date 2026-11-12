use parseltongue_core_row_types::*;

pub trait BuiltinMacroLookupPort {
    fn read_builtin_macro_name(&self, path: &[NamePathSegmentKind]) -> Option<String>;
}

pub fn expand_builtin_macro_items(
    _input: &BuiltinMacroExpansionInput,
    _lookup: &dyn BuiltinMacroLookupPort,
    _cancel: &RunCancelFlagValue,
) -> Result<BuiltinMacroExpansionOutput, MacroExpansionFailureKind> {
    todo!()
}
