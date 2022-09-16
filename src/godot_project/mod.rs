use std::{collections::HashMap, convert::TryFrom};

use crate::utils::parse::ParseError;

use self::parse::parse_godot_project;

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
    KeyAndValue((String, EntryValue))
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryValue {
    String(String),
    Int(i64),
    Float(f64),
    Raw(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalScriptClass {
    pub base: String,
    pub class: String,
    pub language: String,
    pub path: String,
}

impl TryFrom<&str> for GodotProject {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_godot_project(value)
    }
}

mod parse {
    use std::collections::HashMap;

    use crate::utils::parse::{
        consume, consume_while, consume_whitespace_and_comments, next_char_is_break, ParseError,
        ParsedAndIndex, ParseResult, Log,
    };

    use super::{EntryValue, GodotProject, Item};

    pub fn parse_godot_project(code: &str) -> Result<GodotProject, ParseError> {
        let mut project = GodotProject::new();
        let mut index = 0;
        let mut current_section_name = None;
        let mut current_section_entries = HashMap::new();

        index = consume_whitespace_and_comments(code, index, ';').unwrap();

        while index < code.len() {
            match parse_item(code, index) {
                ParseResult::Some(item) => {
                    index = item.index;

                    match item.parsed {
                        Item::SectionName(new_section_name) => {
                            if let Some(current_section_name) = current_section_name {
                                let mut new_section = HashMap::new();

                                std::mem::swap(&mut new_section, &mut current_section_entries);

                                project
                                    .other_sections
                                    .insert(current_section_name, new_section);
                            } else {
                                std::mem::swap(&mut project.front_section, &mut current_section_entries);
                            }

                            current_section_name = Some(new_section_name);
                        },
                        Item::KeyAndValue((key, value)) => {
                            current_section_entries.insert(key, value);
                        },
                    }
                },
                ParseResult::Err(err) => {
                    return Err(err);
                },
                ParseResult::None => {
                    return Err(ParseError { msg: "Failed to consume entire input".into(), index });
                },
            };

            index = consume_whitespace_and_comments(code, index, ';').unwrap();
        }

        Ok(project)
    }

    fn parse_item(code: &str, index: usize) -> ParseResult<Item> {
        parse_section_name(code, index).map(Item::SectionName)
        .or(|| parse_key_and_value(code, index).map(Item::KeyAndValue))
    }

    fn parse_section_name(code: &str, index: usize) -> ParseResult<String> {
        consume(code, index, "[").given(|ParsedAndIndex { parsed: _, index }|
        parse_key(code, index).expec("Expected section name", index, |ParsedAndIndex { parsed: name, index }|
        consume(code, index, "]").expec("Expected ']'", index, |ParsedAndIndex { parsed: _, index }|
            ParseResult::Some(ParsedAndIndex {
                parsed: name,
                index,
            })
        )))
    }

    fn parse_key_and_value(code: &str, index: usize) -> ParseResult<(String, EntryValue)> {
        parse_key(code, index).given(|ParsedAndIndex { parsed: key, index }|
        consume(code, index, "=").expec("Expected '='", index, |ParsedAndIndex { parsed: _, index }|
        parse_value(code, index).expec("Expected value", index, |ParsedAndIndex { parsed: value, index }|
            ParseResult::Some(ParsedAndIndex {
                parsed: (key, value),
                index
            })
        )))
    }

    fn parse_key(code: &str, index: usize) -> ParseResult<String> {
        code[index..]
            .char_indices()
            .take_while(|(_, ch)| ch.is_alphanumeric() || *ch == '_' || *ch == '/')
            .last()
            .filter(|(length, _)| *length > 0)
            .map(|(length, _)| ParsedAndIndex {
                parsed: code[index..index + length + 1].to_owned(),
                index: index + length + 1,
            })
            .into()
    }

    #[test]
    fn test_parse_key() {
        let res = parse_key("sdkfjg foo=12", 7);
        assert_eq!(res, ParseResult::Some(ParsedAndIndex { parsed: "foo".into(), index: 10 }))
    }

    fn parse_value(code: &str, index: usize) -> ParseResult<EntryValue> {
        parse_number(code, index)
        .or(|| parse_string(code, index))
        .or(|| {
            if code.as_bytes()[index] == b'{' || code.as_bytes()[index] == b'[' {
                let mut open_brackets = Vec::new();

                for (current_index, ch) in code[index..].char_indices() {
                    if ch == '{' || ch == '[' {
                        open_brackets.push(ch);
                    } else if ch == '}' || ch == ']' {
                        let top = open_brackets.last();

                        if (top == Some(&'{') && ch == '}') || (top == Some(&'[') && ch == ']')
                        {
                            open_brackets.pop();
                        } else {
                            return ParseResult::Err(ParseError { msg: "Failed to parse value".into(), index: current_index });
                        }

                        if open_brackets.len() == 0 {
                            return ParseResult::Some(ParsedAndIndex {
                                parsed: EntryValue::Raw(
                                    code[index..index + current_index + 1].trim().to_owned(),
                                ),
                                index: index + current_index + 1,
                            });
                        }
                    }
                }

                ParseResult::Err(ParseError { msg: "Failed to parse value".into(), index })
            } else {
                code[index..]
                    .char_indices()
                    .take_while(|(_, ch)| *ch != '\n')
                    .last()
                    .map(|(current_index, _)| ParsedAndIndex {
                        parsed: EntryValue::Raw(
                            code[index..index + current_index + 1].trim().to_owned(),
                        ),
                        index: index + current_index + 1,
                    })
                    .into()
            }
        })
    }

    fn parse_number(code: &str, index: usize) -> ParseResult<EntryValue> {
        let result = consume_while(code, index, |(index, ch)| {
            (index == 0 && ch == '-') || ch.is_numeric()
        });

        match result {
            ParseResult::Some(ParsedAndIndex { parsed: front, index: front_last_index }) => {
                if front.chars().last().map(|ch| ch.is_numeric()).unwrap_or(false) {
                    let front_last_index = front_last_index + 1;

                    let back_last_index = consume_while(code, front_last_index, |(index, ch)| {
                        (index == 0 && ch == '.') || ch.is_numeric()
                    });

                    if let ParseResult::Some(ParsedAndIndex { parsed: _, index: back_last_index }) = back_last_index {
                        if back_last_index >= front_last_index + 2 {
                            if next_char_is_break(code, back_last_index) {
                                if let Ok(float) = code
                                    .get(index..back_last_index)
                                    .unwrap_or("")
                                    .parse::<f64>()
                                {
                                    return ParseResult::Some(ParsedAndIndex {
                                        parsed: EntryValue::Float(float),
                                        index: back_last_index,
                                    });
                                }
                            }
                        } else if code.as_bytes().get(back_last_index - 1) == Some(&b'.') {
                            return ParseResult::Err(ParseError {
                                msg: "Expected decimal value after '.'".into(),
                                index: back_last_index - 1
                            })
                        }
                    }

                    if next_char_is_break(code, front_last_index) {
                        if let Ok(int) = code
                            .get(index..front_last_index)
                            .unwrap_or("")
                            .parse::<i64>()
                        {
                            return ParseResult::Some(ParsedAndIndex {
                                parsed: EntryValue::Int(int),
                                index: front_last_index,
                            });
                        }
                    }
                }

                ParseResult::None
            }
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::None => ParseResult::None
        }
    }

    fn parse_string(code: &str, index: usize) -> ParseResult<EntryValue> {
        let res = consume_while(code, index, |(index, ch)| {
            (index == 0 && ch == '"') || (index > 0 && ch != '"')
        });

        if let ParseResult::Some(ParsedAndIndex { parsed, index: end_of_contents_index }) = res {
            if parsed.len() > 0 {
                return (
                    if code.as_bytes().get(end_of_contents_index) == Some(&b'"') {
                        ParseResult::Some(ParsedAndIndex {
                            parsed: EntryValue::String(
                                code.get(index + 1..end_of_contents_index).unwrap_or("").to_owned(),
                            ),
                            index: end_of_contents_index + 1,
                        })
                    } else {
                        ParseResult::Err(ParseError { msg: "Unclosed string".into(), index })
                    }
                )
            }
        }

        ParseResult::None
    }

    #[test]
    fn test_parse_string() {
        let res = parse_string("\"foo\"", 0);

        assert_eq!(res, ParseResult::Some(ParsedAndIndex { parsed: EntryValue::String("foo".into()), index: 5 }))
    }
}
