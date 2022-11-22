use nom::{
    branch::alt,
    bytes::complete::tag,
    sequence::{preceded, tuple},
};

use crate::utils::{
    string_and_slice::StringAndSlice, ParseResult, ParsedAndIndex, Sourcable, Sourced, Src,
};

use super::model::ast::{Annotation, Declaration, Expression, Type, ValDeclaration};

pub fn parse_script(code: &str) -> Result<Vec<Sourced<Declaration>>, ParseError> {
    let mut declarations = Vec::new();
    let mut index = 0;

    index = consume_whitespace_and_comments(code, index, '#').unwrap();

    while index < code.len() {
        match parse_declaration(code, index) {
            ParseResult::Some(ParsedAndIndex {
                parsed,
                index: new_index,
            }) => {
                index = new_index;
                declarations.push(parsed);
            }
            ParseResult::Err(err) => {
                return Err(err);
            }
            ParseResult::None => {
                return Err(ParseError {
                    msg: "Failed to consume entire input".into(),
                    index,
                });
            }
        };

        index = consume_whitespace_and_comments(code, index, '#').unwrap();
    }

    Ok(declarations)
}

fn parse_declaration(code: &str, start_index: usize) -> ParseResult<Sourced<Declaration>> {
    parse_extends(code, start_index)
        .map(|ast| ast.map(Declaration::Extends))
        .or(|| parse_class_name(code, start_index).map(|ast| ast.map(Declaration::ClassName)))
        .or(|| {
            parse_val_declaration(code, start_index).map(|ast| ast.map(Declaration::ValDeclaration))
        })
        .or(|| parse_annotation(code, start_index).map(|ast| ast.map(Declaration::Annotation)))
}

fn parse_extends(code: &str, start_index: usize) -> ParseResult<Sourced<String>> {
    consume(code, start_index, "extends").given(|ParsedAndIndex { parsed: _, index }| {
        parse_whitespace_and_comments(code, index, '#', true).given(
            |ParsedAndIndex { parsed: _, index }| {
                parse_identifier(code, index).given(
                    |ParsedAndIndex {
                         parsed: name,
                         index,
                     }| {
                        ParseResult::Some(ParsedAndIndex {
                            parsed: name,
                            index,
                        })
                    },
                )
            },
        )
    })
}

fn parse_class_name(code: &str, start_index: usize) -> ParseResult<Sourced<String>> {
    consume(code, start_index, "class_name").given(|ParsedAndIndex { parsed: _, index }| {
        parse_whitespace_and_comments(code, index, '#', true).given(
            |ParsedAndIndex { parsed: _, index }| {
                parse_identifier(code, index).given(
                    |ParsedAndIndex {
                         parsed: name,
                         index,
                     }| {
                        ParseResult::Some(ParsedAndIndex {
                            parsed: name,
                            index,
                        })
                    },
                )
            },
        )
    })
}

fn parse_annotation(code: &str, start_index: usize) -> ParseResult<Sourced<Annotation>> {
    consume(code, start_index, "@").given(|ParsedAndIndex { parsed: _, index }| {
        parse_identifier(code, index).expec(
            "Expected annotation name",
            index,
            |ParsedAndIndex {
                 parsed: name,
                 index,
             }| {
                ParseResult::Some(ParsedAndIndex {
                    parsed: Annotation {
                        name,
                        arguments: Vec::new(),
                    }
                    .spanning(start_index..index),
                    index,
                })
            },
        )
    })
}

fn parse_val_declaration(code: &str, start_index: usize) -> ParseResult<Sourced<ValDeclaration>> {
    consume(code, start_index, "var").or(|| consume(code, start_index, "const")).given(|ParsedAndIndex { parsed: keyword, index }|
    parse_whitespace_and_comments(code, index, '#', true).given(|ParsedAndIndex { parsed: _, index }|
    parse_identifier(code, index).given(|ParsedAndIndex { parsed: name, index }|
    parse_whitespace_and_comments(code, index, '#', false).given(|ParsedAndIndex { parsed: _, index }|
    parse_type_declaration(code, index).optional(index, |ParsedAndIndex { parsed: declared_type, index }|
    parse_whitespace_and_comments(code, index, '#', false).given(|ParsedAndIndex { parsed: _, index }|
    parse_initial_value(code, index).optional(index, |ParsedAndIndex { parsed: value, index }|
        ParseResult::Some(ParsedAndIndex {
            parsed: ValDeclaration {
                is_const: keyword == "const",
                name,
                declared_type,
                value,
            }.spanning(start_index..index),
            index
        }))))))))
}

fn parse_type_declaration(code: &str, start_index: usize) -> ParseResult<Sourced<Type>> {
    consume(code, start_index, ":").given(|ParsedAndIndex { parsed: _, index }| {
        parse_whitespace_and_comments(code, index, '#', false).given(
            |ParsedAndIndex { parsed: _, index }| {
                parse_type(code, index).expec(
                    "Expected type",
                    index,
                    |ParsedAndIndex {
                         parsed: type_expression,
                         index,
                     }| {
                        ParseResult::Some(ParsedAndIndex {
                            parsed: type_expression,
                            index,
                        })
                    },
                )
            },
        )
    })
}

fn parse_initial_value(code: &str, start_index: usize) -> ParseResult<Src<Expression>> {
    tuple((tag("="), preceded(whitespace, parse_expression)))(i)
}

fn parse_expression<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Expression>> {
    todo!()
}

fn parse_type<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Type>> {
    map(
        alt((tag("int"), tag("String"))),
        |src: StringAndSlice<'a>| Src {
            src: src.slice,
            node: match src.as_str() {
                "int" => Type::IntType,
                "String" => Type::StringType,
            },
        },
    )(i)
}
