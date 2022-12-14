use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map, opt},
    error::{VerboseError, VerboseErrorKind},
    multi::{many0, many1, separated_list0},
    sequence::{preceded, terminated, tuple},
};

use crate::utils::{
    errors::ParseError, string_and_slice::StringAndSlice, ParseResult, Src, Srcable,
};

use super::ast::*;

pub fn parse_script(code: &String) -> Result<GDScript, ParseError> {
    let res = terminated(
        many0(preceded(whitespace_and_comments, parse_declaration)),
        whitespace_and_comments,
    )(code.into());

    match res {
        Ok((i, declarations)) => {
            // if i.len() > 0 {
            //     Err(ParseError {
            //         index: Some(i.slice.start),
            //         src: code.clone(),
            //         message: "Failed to parse entire input".to_owned(),
            //     })
            // } else {
            Ok(GDScript { declarations })
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

fn parse_declaration<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Declaration>> {
    alt((
        map(parse_extends, |ast| ast.map(Declaration::from)),
        map(parse_class_name, |ast| ast.map(Declaration::from)),
        map(parse_val_declaration, |ast| ast.map(Declaration::from)),
        map(parse_annotation, |ast| ast.map(Declaration::from)),
    ))(i)
}

fn parse_extends<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Extends>> {
    map(
        tuple((
            tag("extends"),
            preceded(whitespace_and_comments1, identifier),
        )),
        |(_, name)| Src {
            src: Some(name.slice),
            node: Extends(name.as_str().to_owned()),
        },
    )(i)
}

fn parse_class_name<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<ClassName>> {
    map(
        tuple((
            tag("class_name"),
            preceded(whitespace_and_comments1, identifier),
        )),
        |(_, name)| Src {
            src: Some(name.slice),
            node: ClassName(name.as_str().to_owned()),
        },
    )(i)
}

fn parse_annotation<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Annotation>> {
    map(
        tuple((tag("@"), preceded(whitespace_and_comments, identifier))),
        |(at, name)| Src {
            src: Some(at.slice.spanning(name.slice)),
            node: Annotation {
                name: Src {
                    src: Some(name.slice),
                    node: name.as_str().to_owned(),
                },
                arguments: vec![],
            },
        },
    )(i)
}

fn parse_val_declaration<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<ValDeclaration>> {
    map(
        tuple((
            alt((tag("var"), tag("const"))),
            preceded(whitespace_and_comments, identifier),
            opt(preceded(whitespace_and_comments, parse_type_declaration)),
            opt(preceded(whitespace_and_comments, parse_initial_value)),
        )),
        |(keyword, name, declared_type, value)| Src {
            src: value
                .as_ref()
                .map(|value| value.src.map(|src| keyword.slice.spanning(src)))
                .flatten(),
            node: ValDeclaration {
                is_const: keyword.as_str() == "const",
                name: Src {
                    src: Some(name.slice),
                    node: name.as_str().to_owned(),
                },
                declared_type,
                value,
            },
        },
    )(i)
}

fn parse_type_declaration<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Type>> {
    map(
        tuple((tag(":"), preceded(whitespace_and_comments, parse_type))),
        |(eq, typ)| Src {
            src: typ.src.map(|src| eq.slice.spanning(src)),
            node: typ.node,
        },
    )(i)
}

fn parse_initial_value<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Expression>> {
    map(
        tuple((
            tag("="),
            preceded(whitespace_and_comments, parse_expression),
        )),
        |(eq, expression)| Src {
            src: expression.src.map(|src| eq.slice.spanning(src)),
            node: expression.node,
        },
    )(i)
}

fn parse_expression<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Expression>> {
    alt((
        map(parse_array_expression, |ast| ast.map(Expression::from)),
        map(tag("true"), |s: StringAndSlice| {
            Expression::Boolean(true).with_src(s.slice)
        }),
        map(tag("false"), |s: StringAndSlice| {
            Expression::Boolean(false).with_src(s.slice)
        }),
        map(tag("null"), |s: StringAndSlice| {
            Expression::Null.with_src(s.slice)
        }),
    ))(i)
}

fn parse_array_expression<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Array>> {
    map(
        tuple((
            tag("["),
            preceded(
                whitespace_and_comments,
                separated_list0(
                    preceded(whitespace_and_comments, tag(",")),
                    preceded(whitespace_and_comments, parse_expression),
                ),
            ),
            preceded(whitespace_and_comments, tag("]")),
        )),
        |(open, values, close)| Array(values).with_src(open.spanning(&close).slice),
    )(i)
}

fn parse_type<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<Type>> {
    map(
        alt((tag("int"), tag("float"), tag("String"))),
        |s: StringAndSlice<'a>| {
            match s.as_str() {
                "int" => Type::IntType,
                "float" => Type::FloatType,
                "String" => Type::StringType,
                _ => unreachable!(),
            }
            .with_src(s.slice)
        },
    )(i)
}

fn identifier<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, StringAndSlice<'a>> {
    take_while1(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '$')(i)
}

fn whitespace_and_comments<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ()> {
    map(
        many0(alt((
            map(
                take_while1(|c: char| c == ' ' || c == '\n' || c == '\t' || c == '\r'),
                |_| (),
            ),
            map(tuple((tag("#"), take_while(|c| c != '\n'))), |_| ()),
        ))),
        |_| (),
    )(i)
}

fn whitespace_and_comments1<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, ()> {
    map(
        many1(alt((
            map(
                take_while1(|c: char| c == ' ' || c == '\n' || c == '\t' || c == '\r'),
                |_| (),
            ),
            map(tuple((tag("#"), take_while(|c| c != '\n'))), |_| ()),
        ))),
        |_| (),
    )(i)
}
