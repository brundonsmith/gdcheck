use enum_variant_type::EnumVariantType;

use super::parse::parse_gdproject_metadata;
use crate::utils::{errors::ParseError, slice::Slice};
use std::{collections::HashMap, convert::TryFrom};

#[derive(Debug, Clone, PartialEq)]
pub struct GDProjectMetadata {
    pub front_section: Section,
    pub other_sections: HashMap<Slice, Section>,
}

type Section = HashMap<Slice, EntryValue>;

impl GDProjectMetadata {
    pub fn new() -> Self {
        GDProjectMetadata {
            front_section: HashMap::new(),
            other_sections: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    SectionName(Slice),
    KeyAndValue((Slice, EntryValue)),
}

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum EntryValue {
    Null,
    BooleanValue(bool),
    StringValue {
        s: Slice,
        ampersand: bool,
    },
    NumberValue(Slice),
    ListValue(Vec<EntryValue>),
    DictValue(HashMap<Slice, EntryValue>),
    ObjectValue {
        class: Slice,
        properties: HashMap<Slice, EntryValue>,
    },
    ConstructedValue {
        class: Slice,
        entries: Vec<EntryValue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScriptClass {
    pub base: Slice,
    pub class: Slice,
    pub language: Slice,
    pub path: Slice,
}

impl TryFrom<Slice> for GDProjectMetadata {
    type Error = ParseError;

    fn try_from(value: Slice) -> Result<Self, Self::Error> {
        parse_gdproject_metadata(value)
    }
}
