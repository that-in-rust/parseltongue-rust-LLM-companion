#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitPreludeConfigurationInput {
    pub unit_no_std_flag: bool,
    pub unit_no_core_flag: bool,
    pub unit_implicit_prelude_flag: bool,
}
