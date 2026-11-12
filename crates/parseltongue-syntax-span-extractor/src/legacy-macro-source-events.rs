use parseltongue_core_row_types::StoreBackendFailureError;
use ra_ap_syntax::SyntaxNode;
use serde_json::Value;
use std::collections::BTreeSet;

pub struct LegacyMacroNodeEvents {
    pub before_node_rows: Vec<Value>,
    pub after_node_rows: Vec<Value>,
}

pub fn read_legacy_node_events(
    _node: &SyntaxNode,
    _prefix: &[u64],
    _unit: &Value,
    _cfg: &BTreeSet<String>,
    _read_key: &dyn Fn(&SyntaxNode, bool) -> Result<Value, StoreBackendFailureError>,
) -> Result<LegacyMacroNodeEvents, StoreBackendFailureError> {
    todo!("ordered legacy macro source events")
}
