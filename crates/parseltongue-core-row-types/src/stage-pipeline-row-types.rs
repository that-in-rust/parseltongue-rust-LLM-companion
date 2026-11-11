use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Default)]
pub struct RunCancelFlagValue(Arc<AtomicBool>);

impl RunCancelFlagValue {
    pub fn request_stage_work_cancellation(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn check_stage_work_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageRunScopeValue {
    pub run_generation_digest_value: String,
    pub stage_attempt_sequence_value: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageTableRowRecord {
    pub stage_owned_table_name: String,
    pub stage_row_record_key: String,
    pub stage_row_payload_text: String,
}

pub struct StageUnitWriteRequest<'a> {
    pub stage_run_scope_value: &'a StageRunScopeValue,
    pub stage_owner_name_value: &'a str,
    pub stage_unit_work_key: &'a str,
    pub stage_input_stage_names: &'a [&'a str],
    pub stage_unit_output_rows: &'a [StageTableRowRecord],
}
