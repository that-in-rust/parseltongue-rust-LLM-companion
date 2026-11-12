pub(super) const NAMESPACE_APPENDER_ROW_LIMIT: usize = 1024;
pub(super) const NAMESPACE_APPENDER_BYTE_LIMIT: usize = 1024 * 1024;

#[derive(Default)]
pub(super) struct NamespaceAppenderBatchBudget {
    pending_appender_row_count: usize,
    pending_appender_byte_count: usize,
}

impl NamespaceAppenderBatchBudget {
    pub(super) fn record_appender_row_budget(&mut self, bytes: usize) -> bool {
        self.pending_appender_row_count += 1;
        self.pending_appender_byte_count = self.pending_appender_byte_count.saturating_add(bytes);
        let flush = self.pending_appender_row_count >= NAMESPACE_APPENDER_ROW_LIMIT
            || self.pending_appender_byte_count >= NAMESPACE_APPENDER_BYTE_LIMIT;
        if flush {
            self.pending_appender_row_count = 0;
            self.pending_appender_byte_count = 0;
        }
        flush
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-RUST-024.8
    #[test]
    fn test_namespace_appender_batch_limits() {
        let mut budget = NamespaceAppenderBatchBudget::default();
        for _ in 0..NAMESPACE_APPENDER_ROW_LIMIT - 1 {
            assert!(!budget.record_appender_row_budget(0));
        }
        assert!(budget.record_appender_row_budget(0));
        assert!(!budget.record_appender_row_budget(NAMESPACE_APPENDER_BYTE_LIMIT - 1));
        assert!(budget.record_appender_row_budget(1));
        assert!(budget.record_appender_row_budget(usize::MAX));
        assert!(!budget.record_appender_row_budget(1));
    }
}
