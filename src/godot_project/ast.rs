use enum_variant_type::EnumVariantType;

use super::parse::parse_godot_project;
use crate::utils::errors::ParseError;
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Clone, PartialEq)]
pub struct GodotProject {
    pub front_section: Section,
    pub other_sections: HashMap<String, Section>,
}

type Section = HashMap<String, EntryValue>;

impl GodotProject {
    pub fn new() -> Self {
        GodotProject {
            front_section: HashMap::new(),
            other_sections: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    SectionName(String),
    KeyAndValue((String, EntryValue)),
}

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum EntryValue {
    Null,
    BooleanValue(bool),
    StringValue {
        s: String,
        ampersand: bool,
    },
    NumberValue(String),
    ListValue(Vec<EntryValue>),
    DictValue(HashMap<String, EntryValue>),
    ObjectValue {
        class: String,
        properties: HashMap<String, EntryValue>,
    },
    ConstructedValue {
        class: String,
        entries: Vec<EntryValue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScriptClass {
    pub base: String,
    pub class: String,
    pub language: String,
    pub path: String,
}

impl TryFrom<&String> for GodotProject {
    type Error = ParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        parse_godot_project(value)
    }
}
