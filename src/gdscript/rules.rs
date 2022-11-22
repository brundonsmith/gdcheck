use strum_macros::EnumString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumString)]
pub enum Rule {
    RequireTypeAnnotations,
    AssignmentType,
    StrictAny,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumString)]
pub enum RuleSeverity {
    #[strum(serialize = "error")]
    Error,
    #[strum(serialize = "warning")]
    Warning,
    #[strum(serialize = "info")]
    Info,
    #[strum(serialize = "off")]
    Off,
}
