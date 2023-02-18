use std::collections::HashMap;

use crate::{
    gdproject_metadata::ast::GDProjectMetadata,
    gdscript::{
        ast::{GDScript, ModuleID},
        rules::{Rule, RuleSeverity},
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct GodotProject {
    pub metadata: GDProjectMetadata,
    pub rule_severity: HashMap<Rule, RuleSeverity>,
    pub scripts: HashMap<ModuleID, GDScript>,
}
