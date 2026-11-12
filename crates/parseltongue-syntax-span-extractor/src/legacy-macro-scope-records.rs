use parseltongue_core_row_types::{StageTableRowRecord, StoreBackendFailureError};
use serde_json::Value;

pub fn visit_legacy_macro_scope_records(
    _events: &[Value],
    _visitor: &mut dyn FnMut(StageTableRowRecord) -> Result<(), StoreBackendFailureError>,
) -> Result<usize, StoreBackendFailureError> {
    todo!("ordered textual macro scope records")
}
