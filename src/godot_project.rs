use std::{collections::HashMap, path::PathBuf};

use crate::{
    gdproject_metadata::ast::GDProjectMetadata,
    gdscript::{
        ast::GDScript,
        rules::{Rule, RuleSeverity},
    },
};

#[derive(Clone, Debug, PartialEq)]
pub struct GodotProject {
    pub metadata: GDProjectMetadata,
    pub rule_severity: HashMap<Rule, RuleSeverity>,
    pub scripts: HashMap<String, GDScript>,
}
