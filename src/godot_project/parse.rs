use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::combinator::map;
use nom::combinator::opt;
use nom::error::VerboseError;
use nom::error::VerboseErrorKind;
use nom::multi::many0;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::sequence::terminated;
use nom::sequence::{preceded, tuple};

use crate::utils::errors::ParseError;
use crate::utils::identifier;
use crate::utils::{number_literal, string_literal};
use crate::utils::{string_and_slice::StringAndSlice, ParseResult};

use super::*;

pub fn parse_godot_project(code: &String) -> Result<GodotProject, ParseError> {
    let res = many0(terminated(
        preceded(whitespace_and_comments, parse_item),
        whitespace_and_comments,
    ))(code.into());

    match res {
        Ok((i, items)) => {
            println!("{:?}", items);
            // if i.len() > 0 {
            //     Err(ParseError {
            //         index: Some(i.slice.start),
            //         src: code.clone(),
            //         message: "Failed to parse entire input".to_owned(),
            //     })
            // } else {
            let mut project = GodotProject::new();
            let mut current_section_name = None;
            let mut current_section_entries = HashMap::new();

            for item in items {
                match item {
                    Item::SectionName(new_section_name) => {
                        if let Some(current_section_name) = current_section_name {
                            let mut new_section = HashMap::new();

                            std::mem::swap(&mut new_section, &mut current_section_entries);

                            project
                                .other_sections
                                .insert(current_section_name, new_section);
                        } else {
                            std::mem::swap(
                                &mut project.front_section,
                                &mut current_section_entries,
                            );
                        }

                        current_section_name = Some(new_section_name);
                    }
                    Item::KeyAndValue((key, value)) => {
                        current_section_entries.insert(key, value);
                    }
                }
            }

            Ok(project)
            // }
        }
        Err(error) => {
            let errors = match error {
                nom::Err::Error(VerboseError { errors }) => Some(errors),
                nom::Err::Failure(VerboseError { errors }) => Some(errors),
                nom::Err::Incomplete(_) => None,
            };

            println!("{:?}", errors);

            let info = errors
                .map(|errors| {
                    errors
                        .into_iter()
                        .filter_map(|(input, kind)| {
                            if let VerboseErrorKind::Context(context) = kind {
                                Some((input, context))
                            } else {
                                None
                            }
                        })
                        .next()
                })
                .flatten();

            Err(info
                .map(|(input, context)| ParseError {
                    index: Some(input.slice.start + input.len()),
                    src: code.clone(),
                    message: format!("Failed while parsing {}", context),
                })
                .unwrap_or_else(|| ParseError {
                    index: None,
                    src: code.clone(),
                    message: "Failed to parse".to_owned(),
                }))
        }
    }
}

fn parse_item<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Item> {
    alt((
        map(parse_section_name, |name| Item::SectionName(name)),
        map(parse_key_and_value, |key_value| {
            Item::KeyAndValue(key_value)
        }),
    ))(i)
}

fn parse_section_name<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, String> {
    map(
        tuple((
            tag("["),
            preceded(whitespace_and_comments, parse_key),
            preceded(whitespace_and_comments, tag("]")),
        )),
        |(_, name, _)| name,
    )(i)
}

fn parse_key_and_value<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, (String, EntryValue)> {
    map(
        tuple((
            parse_key,
            preceded(whitespace_and_comments, tag("=")),
            preceded(whitespace_and_comments, parse_value),
        )),
        |(key, _, value)| (key, value),
    )(i)
}

fn parse_key<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, String> {
    map(
        take_while1(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '/'),
        |s: StringAndSlice<'a>| s.as_str().to_owned(),
    )(i)
}

fn parse_value<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, EntryValue> {
    alt((
        map(
            pair(opt(tag("&")), string_literal),
            |(ampersand, string)| EntryValue::StringValue {
                s: string.node,
                ampersand: ampersand.is_some(),
            },
        ),
        map(number_literal, |s| EntryValue::NumberValue(s.node)),
        map(parse_list, EntryValue::from),
        map(parse_dict, EntryValue::from),
        map(parse_object, EntryValue::from),
        map(parse_constructed, EntryValue::from),
        map(tag("true"), |_| EntryValue::BooleanValue(true)),
        map(tag("false"), |_| EntryValue::BooleanValue(false)),
        map(tag("null"), |_| EntryValue::Null),
    ))(i)
}

fn parse_list<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ListValue> {
    map(
        tuple((
            tag("["),
            preceded(
                whitespace_and_comments,
                separated_list0(
                    preceded(whitespace_and_comments, tag(",")),
                    preceded(whitespace_and_comments, parse_value),
                ),
            ),
            preceded(whitespace_and_comments, tag("]")),
        )),
        |(_, values, _)| ListValue(values),
    )(i)
}

fn parse_dict<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, DictValue> {
    map(
        tuple((
            tag("{"),
            preceded(
                whitespace_and_comments,
                separated_list0(
                    preceded(whitespace_and_comments, tag(",")),
                    preceded(whitespace_and_comments, parse_dict_entry),
                ),
            ),
            preceded(whitespace_and_comments, tag("}")),
        )),
        |(_, entries, _)| DictValue(entries.into_iter().collect()),
    )(i)
}

fn parse_dict_entry<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, (String, EntryValue)> {
    map(
        tuple((
            string_literal,
            preceded(whitespace_and_comments, tag(":")),
            preceded(whitespace_and_comments, parse_value),
        )),
        |(key, _, value)| (key.node, value),
    )(i)
}

fn parse_object<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ObjectValue> {
    map(
        tuple((
            tag("Object"),
            preceded(whitespace_and_comments, tag("(")),
            preceded(whitespace_and_comments, identifier),
            preceded(whitespace_and_comments, tag(",")),
            separated_list1(
                preceded(whitespace_and_comments, tag(",")),
                preceded(whitespace_and_comments, parse_dict_entry),
            ),
            preceded(whitespace_and_comments, tag(")")),
        )),
        |(_, _, class, _, entries, _)| ObjectValue {
            class: class.as_str().to_owned(),
            properties: entries.into_iter().collect(),
        },
    )(i)
}

fn parse_constructed<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ConstructedValue> {
    map(
        tuple((
            identifier,
            preceded(whitespace_and_comments, tag("(")),
            separated_list1(preceded(whitespace_and_comments, tag(",")), parse_value),
            preceded(whitespace_and_comments, tag(")")),
        )),
        |(class, _, entries, _)| ConstructedValue {
            class: class.as_str().to_owned(),
            entries,
        },
    )(i)
}

pub fn whitespace_and_comments<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ()> {
    map(
        many0(alt((
            map(
                take_while1(|c: char| c == ' ' || c == '\n' || c == '\t' || c == '\r'),
                |_| (),
            ),
            map(tuple((tag(";"), take_while(|c| c != '\n'))), |_| ()),
        ))),
        |_| (),
    )(i)
}

// #[test]
// fn test_parse_key() {
//     let res = parse_key("sdkfjg foo=12", 7);
//     assert_eq!(
//         res,
//         ParseResult::Some(ParsedAndIndex {
//             parsed: "foo".into(),
//             index: 10
//         })
//     )
// }

// #[test]
// fn test_parse_string() {
//     let res = parse_string("\"foo\"", 0);

//     assert_eq!(
//         res,
//         ParseResult::Some(ParsedAndIndex {
//             parsed: EntryValue::String("foo".into()),
//             index: 5
//         })
//     )
// }
