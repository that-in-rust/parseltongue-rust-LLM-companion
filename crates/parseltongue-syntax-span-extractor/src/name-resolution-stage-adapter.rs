use parseltongue_core_row_types::{
    RunCancelFlagValue, StageRunScopeValue, StoreBackendFailureError,
};
use parseltongue_table_store_traits::StageTableStorePort;

pub fn run_persisted_name_resolution(
    _stage_run_scope_value: &StageRunScopeValue,
    _stage_table_store_port: &dyn StageTableStorePort,
    _stage_cancel_flag_value: &RunCancelFlagValue,
) -> Result<usize, StoreBackendFailureError> {
    todo!()
}
