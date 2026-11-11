use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};

pub fn map_bounded_stage_workers<T: Send>(
    _stage_unit_work_keys: &[String],
    _stage_worker_count_value: usize,
    _stage_cancel_flag_value: &RunCancelFlagValue,
    _stage_unit_work_function: impl Fn(&str) -> Result<T, StoreBackendFailureError> + Sync,
) -> Result<Vec<T>, StoreBackendFailureError> {
    todo!()
}

pub fn run_bounded_stage_workers(
    stage_unit_work_keys: &[String],
    stage_worker_count_value: usize,
    stage_cancel_flag_value: &RunCancelFlagValue,
    stage_unit_work_function: impl Fn(&str) -> Result<(), StoreBackendFailureError> + Sync,
) -> Result<(), StoreBackendFailureError> {
    let failure = |message: &str| StoreBackendFailureError {
        error_context_detail_text: message.into(),
    };
    if stage_worker_count_value == 0 {
        return Err(failure("worker count must be positive"));
    }
    for batch in stage_unit_work_keys.chunks(stage_worker_count_value) {
        if stage_cancel_flag_value.check_stage_work_cancelled() {
            return Err(failure("stage cancelled"));
        }
        std::thread::scope(|scope| {
            let workers: Vec<_> = batch
                .iter()
                .map(|key| {
                    let work = &stage_unit_work_function;
                    scope.spawn(move || work(key))
                })
                .collect();
            workers.into_iter().try_for_each(|worker| {
                worker
                    .join()
                    .map_err(|_| failure("stage worker panicked"))?
            })
        })?;
    }
    Ok(())
}
