use parseltongue_core_row_types::*;
use std::collections::BTreeMap;

pub fn extend_sysroot_crate_graph(
    _graph: &[StageTableRowRecord],
    _contexts: &BTreeMap<String, UnitPreludeConfigurationInput>,
) -> Result<Vec<StageTableRowRecord>, StoreBackendFailureError> {
    todo!()
}
