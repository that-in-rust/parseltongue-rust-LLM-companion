use crate::SyntaxFileInputData;
use parseltongue_core_row_types::{StoreBackendFailureError, UnitPreludeConfigurationInput};

pub fn read_unit_prelude_configuration(
    _input: &SyntaxFileInputData,
) -> Result<UnitPreludeConfigurationInput, StoreBackendFailureError> {
    todo!()
}

pub fn check_sysroot_context_needed(_source: &str) -> bool {
    todo!()
}

pub fn check_recognized_lang_item(_name: &str) -> bool {
    todo!()
}
