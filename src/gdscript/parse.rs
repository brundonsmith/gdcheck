use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    combinator::{map, opt},
    multi::{many0, many1, separated_list0},
    sequence::{preceded, terminated, tuple},
};

use crate::utils::{
    errors::ParseError,
    slice::{Slicable, Slice},
    ParseResult, RawParseError, RawParseErrorDetails,
};

use super::ast::*;

macro_rules! seq {
    ($( $s:expr ),* $(,)?) => {
        tuple(( $(preceded(whitespace, $s)),* ))
    };
}

macro_rules! make_node {
    ($kind:ident, $src:expr, $( $prop:ident ),* $(,)?) => {
        {
            let src = $src;
            let this = $kind {
                $($prop: $prop.clone(),)*
            }
            .as_ast(src);

            $($prop.set_parent(&this);)*

            this
        }
    };
}

macro_rules! make_node_tuple {
    ($kind:ident, $src:expr, $( $prop:ident ),* $(,)?) => {
        {
            let src = $src;
            let this = $kind(
                $($prop.clone()),*
            )
            .as_ast(src);

            $($prop.set_parent(&this);)*

            this
        }
    };
}

pub fn parse_script(module_id: ModuleID, code: Slice) -> Result<GDScript, ParseError> {
    let res = terminated(
        many0(preceded(whitespace_and_comments, parse_declaration)),
        whitespace_and_comments,
    )(code.clone());

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
        Err(error) => Err(match error {
            nom::Err::Error(RawParseError { src, details }) => ParseError {
                module_id: None,
                src,
                message: match details {
                    RawParseErrorDetails::Kind(kind) => kind.description().to_owned(),
                    RawParseErrorDetails::Char(ch) => format!("Expected '{}'", ch),
                },
            },
            nom::Err::Failure(RawParseError { src, details }) => ParseError {
                module_id: None,
                src,
                message: match details {
                    RawParseErrorDetails::Kind(kind) => kind.description().to_owned(),
                    RawParseErrorDetails::Char(ch) => format!("Expected '{}'", ch),
                },
            },
            nom::Err::Incomplete(_) => ParseError {
                module_id: None,
                src: code,
                message: "Failed to parse".to_owned(),
            },
        }),
    }
}

fn parse_declaration(i: Slice) -> ParseResult<AST<Declaration>> {
    alt((
        map(parse_extends, AST::recast::<Declaration>),
        map(parse_class_name, AST::recast::<Declaration>),
        map(parse_val_declaration, AST::recast::<Declaration>),
        map(parse_func(0), AST::recast::<Declaration>),
        map(parse_annotation, AST::recast::<Declaration>),
    ))(i)
}

fn parse_extends(i: Slice) -> ParseResult<AST<ExtendsDeclaration>> {
    map(
        tuple((
            tag("extends"),
            preceded(whitespace_and_comments1, plain_identifier),
        )),
        |(start, mut extends_class)| {
            make_node!(
                ExtendsDeclaration,
                start.spanning(&extends_class),
                extends_class
            )
        },
    )(i)
}

fn parse_class_name(i: Slice) -> ParseResult<AST<ClassNameDeclaration>> {
    map(
        tuple((
            tag("class_name"),
            preceded(whitespace_and_comments1, plain_identifier),
        )),
        |(start, mut class_name)| {
            make_node!(
                ClassNameDeclaration,
                start.spanning(&class_name),
                class_name
            )
        },
    )(i)
}

fn parse_annotation(i: Slice) -> ParseResult<AST<Annotation>> {
    map(
        tuple((
            tag("@"),
            preceded(whitespace_and_comments, plain_identifier),
        )),
        |(start, mut name)| {
            let mut arguments = vec![]; // TODO

            make_node!(Annotation, start.spanning(&name), name, arguments)
        },
    )(i)
}

fn parse_func(indentation: usize) -> impl Fn(Slice) -> ParseResult<AST<FuncDeclaration>> {
    move |i: Slice| -> ParseResult<AST<FuncDeclaration>> {
        map(
            tuple((
                tag("func"),
                preceded(whitespace_and_comments, plain_identifier),
                preceded(whitespace_and_comments, tag("(")),
                // preceded(whitespace_and_comments, args),
                preceded(whitespace_and_comments, tag(")")),
                preceded(whitespace_and_comments, tag(":")),
                parse_block(indentation + 1),
            )),
            |(keyword, mut name, _, _, _, mut body)| {
                let mut is_static = false; // TODO
                let mut args = vec![]; // TODO
                let mut return_type = None; // TODO

                make_node!(
                    FuncDeclaration,
                    keyword.spanning(&body),
                    is_static,
                    name,
                    args,
                    return_type,
                    body
                )
            },
        )(i)
    }
}

fn parse_block(indentation: usize) -> impl Fn(Slice) -> ParseResult<AST<Block>> {
    move |i: Slice| -> ParseResult<AST<Block>> {
        map(
            many1(preceded(
                whitespace_and_comments,
                preceded(parse_indentation(indentation), parse_statement(indentation)),
            )),
            |mut statements| {
                make_node!(
                    Block,
                    statements[0]
                        .slice()
                        .clone()
                        .spanning(statements.iter().last().unwrap()),
                    statements
                )
            },
        )(i)
    }
}

fn parse_statement(indentation: usize) -> impl Fn(Slice) -> ParseResult<AST<Statement>> {
    |i: Slice| -> ParseResult<AST<Statement>> {
        alt((
            map(tag("pass"), |src| Pass.as_ast(src).recast::<Statement>()),
            map(tag("break"), |src| Break.as_ast(src).recast::<Statement>()),
        ))(i)
    }
}

fn parse_val_declaration(i: Slice) -> ParseResult<AST<ValueDeclaration>> {
    map(
        tuple((
            alt((tag("var"), tag("const"))),
            preceded(whitespace_and_comments, plain_identifier),
            opt(preceded(whitespace_and_comments, parse_type_declaration)),
            opt(preceded(whitespace_and_comments, parse_initial_value)),
        )),
        |(keyword, mut name, mut declared_type, mut value)| {
            let mut is_const = keyword.as_str() == "const";

            make_node!(
                ValueDeclaration,
                keyword.spanning(
                    value.as_ref().map(|v| v.slice()).unwrap_or(
                        declared_type
                            .as_ref()
                            .map(|d| d.slice())
                            .unwrap_or(name.slice())
                    )
                ),
                is_const,
                name,
                declared_type,
                value
            )
        },
    )(i)
}

fn parse_type_declaration(i: Slice) -> ParseResult<AST<TypeExpression>> {
    map(
        tuple((tag(":"), preceded(whitespace_and_comments, parse_type))),
        |(_, typ)| typ,
    )(i)
}

fn parse_initial_value(i: Slice) -> ParseResult<AST<Expression>> {
    map(
        tuple((
            tag("="),
            preceded(whitespace_and_comments, parse_expression),
        )),
        |(eq, expression)| expression,
    )(i)
}

fn parse_expression(i: Slice) -> ParseResult<AST<Expression>> {
    alt((
        map(parse_array_literal, AST::recast::<Expression>),
        map(tag("true"), |src| {
            BooleanLiteral { value: true }
                .as_ast(src)
                .recast::<Expression>()
        }),
        map(tag("true"), |src| {
            BooleanLiteral { value: false }
                .as_ast(src)
                .recast::<Expression>()
        }),
        map(tag("null"), |src| {
            NullLiteral.as_ast(src).recast::<Expression>()
        }),
    ))(i)
}

fn parse_array_literal(i: Slice) -> ParseResult<AST<ArrayLiteral>> {
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
        |(open, mut members, close)| make_node!(ArrayLiteral, open.spanning(&close), members),
    )(i)
}

fn parse_type(i: Slice) -> ParseResult<AST<TypeExpression>> {
    alt((
        map(tag("int"), |src| {
            IntType.as_ast(src).recast::<TypeExpression>()
        }),
        map(tag("float"), |src| {
            FloatType.as_ast(src).recast::<TypeExpression>()
        }),
        map(tag("String"), |src| {
            StringType.as_ast(src).recast::<TypeExpression>()
        }),
    ))(i)
}

fn plain_identifier(i: Slice) -> ParseResult<AST<PlainIdentifier>> {
    map(
        take_while1(|ch: char| ch.is_alphanumeric() || ch == '_' || ch == '$'),
        |name: Slice| PlainIdentifier { name: name.clone() }.as_ast(name),
    )(i)
}

fn whitespace_and_comments(i: Slice) -> ParseResult<()> {
    map(
        many0(alt((
            map(
                take_while1(|c: char| c == ' ' || c == '\n' || c == '\r'),
                |_| (),
            ),
            map(tuple((tag("#"), take_while(|c| c != '\n'))), |_| ()),
        ))),
        |_| (),
    )(i)
}

fn whitespace_and_comments1(i: Slice) -> ParseResult<()> {
    map(
        many1(alt((
            map(
                take_while1(|c: char| c == ' ' || c == '\n' || c == '\r'),
                |_| (),
            ),
            map(tuple((tag("#"), take_while(|c| c != '\n'))), |_| ()),
        ))),
        |_| (),
    )(i)
}

fn parse_indentation(indentation: usize) -> impl Fn(Slice) -> ParseResult<()> {
    move |i: Slice| -> ParseResult<()> {
        map(
            alt((
                tag(get_indentation_str_tabs(indentation)),
                tag(get_indentation_str_spaces(indentation)),
            )),
            |_: Slice| (),
        )(i)
    }
}

fn get_indentation_str_tabs(indentation: usize) -> &'static str {
    match indentation {
        0 => "",
        1 => "\t",
        2 => "\t\t",
        3 => "\t\t\t",
        4 => "\t\t\t\t",
        5 => "\t\t\t\t\t",
        _ => unimplemented!(),
    }
}

fn get_indentation_str_spaces(indentation: usize) -> &'static str {
    match indentation {
        0 => "",
        1 => "    ",
        2 => "        ",
        3 => "            ",
        4 => "                ",
        5 => "                    ",
        _ => unimplemented!(),
    }
}
