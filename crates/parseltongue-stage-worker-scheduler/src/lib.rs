use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};

pub fn run_bounded_stage_workers(
    _stage_unit_work_keys: &[String],
    _stage_worker_count_value: usize,
    _stage_cancel_flag_value: &RunCancelFlagValue,
    _stage_unit_work_function: impl Fn(&str) -> Result<(), StoreBackendFailureError> + Sync,
) -> Result<(), StoreBackendFailureError> {
    todo!()
}
