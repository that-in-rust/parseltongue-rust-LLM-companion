#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use parseltongue_core_row_types::{RunCancelFlagValue, StoreBackendFailureError};

pub fn map_bounded_stage_workers<T: Send>(
    stage_unit_work_keys: &[String],
    stage_worker_count_value: usize,
    stage_cancel_flag_value: &RunCancelFlagValue,
    stage_unit_work_function: impl Fn(&str) -> Result<T, StoreBackendFailureError> + Sync,
) -> Result<Vec<T>, StoreBackendFailureError> {
    let failure = |message: &str| StoreBackendFailureError {
        error_context_detail_text: message.into(),
    };
    if stage_worker_count_value == 0 {
        return Err(failure("worker count must be positive"));
    }
    stage_unit_work_keys
        .chunks(stage_worker_count_value)
        .try_fold(Vec::new(), |mut completed, batch| {
            if stage_cancel_flag_value.check_stage_work_cancelled() {
                return Err(failure("stage cancelled"));
            }
            let output = std::thread::scope(|scope| {
                let workers: Vec<_> = batch
                    .iter()
                    .map(|key| {
                        let work = &stage_unit_work_function;
                        scope.spawn(move || work(key))
                    })
                    .collect();
                let results: Vec<_> = workers
                    .into_iter()
                    .map(|worker| {
                        worker
                            .join()
                            .map_err(|_| failure("stage worker panicked"))
                            .and_then(std::convert::identity)
                    })
                    .collect();
                results.into_iter().collect::<Result<Vec<T>, _>>()
            })?;
            if stage_cancel_flag_value.check_stage_work_cancelled() {
                return Err(failure("stage cancelled"));
            }
            completed.extend(output);
            Ok(completed)
        })
}

pub fn run_bounded_stage_workers(
    stage_unit_work_keys: &[String],
    stage_worker_count_value: usize,
    stage_cancel_flag_value: &RunCancelFlagValue,
    stage_unit_work_function: impl Fn(&str) -> Result<(), StoreBackendFailureError> + Sync,
) -> Result<(), StoreBackendFailureError> {
    map_bounded_stage_workers(
        stage_unit_work_keys,
        stage_worker_count_value,
        stage_cancel_flag_value,
        stage_unit_work_function,
    )
    .map(|_| ())
}
