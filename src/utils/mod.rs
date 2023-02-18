use nom::{
    bytes::complete::{escaped, tag, take_while, take_while1},
    character::complete::one_of,
    combinator::{cut, map, opt},
    error::context,
    sequence::{pair, tuple},
    IResult,
};

use self::slice::{Slicable, Slice};

pub mod errors;
pub mod slice;

pub type ParseResult<T> = IResult<Slice, T, RawParseError>;

#[derive(Debug, Clone, PartialEq)]
pub struct RawParseError {
    pub src: Slice,
    pub details: RawParseErrorDetails,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RawParseErrorDetails {
    Kind(nom::error::ErrorKind),
    Char(char),
}

impl nom::error::ParseError<Slice> for RawParseError {
    fn from_error_kind(input: Slice, kind: nom::error::ErrorKind) -> Self {
        Self {
            src: input,
            details: RawParseErrorDetails::Kind(kind),
        }
    }

    fn append(_input: Slice, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: Slice, ch: char) -> Self {
        Self {
            src: input,
            details: RawParseErrorDetails::Char(ch),
        }
    }
}

impl nom::error::ContextError<Slice> for RawParseError {
    fn add_context(_input: Slice, _ctx: &'static str, other: Self) -> Self {
        other
    }
}

impl<E> nom::error::FromExternalError<Slice, E> for RawParseError {
    fn from_external_error(input: Slice, kind: nom::error::ErrorKind, _e: E) -> Self {
        Self {
            src: input,
            details: RawParseErrorDetails::Kind(kind),
        }
    }
}

pub fn string_literal(i: Slice) -> ParseResult<Slice> {
    context(
        "string",
        map(
            pair(tag("\""), cut(pair(string_contents, tag("\"")))),
            |(open_quote, (contents, close_quote))| open_quote.spanning(&close_quote),
        ),
    )(i)
}

pub fn string_contents(i: Slice) -> ParseResult<Slice> {
    escaped(take_while(|ch: char| ch != '"'), '\\', one_of("\"n\\"))(i)
}

pub fn number_literal<'a>(i: Slice) -> ParseResult<Slice> {
    map(
        tuple((opt(tag("-")), numeric, opt(tuple((tag("."), cut(numeric)))))),
        |(neg, int, tail)| {
            let front = neg.unwrap_or(int.clone());
            let back = tail.map(|(_, decimal)| decimal).unwrap_or(int);

            front.spanning(&back)
        },
    )(i)
}

pub fn numeric<'a>(i: Slice) -> ParseResult<Slice> {
    take_while1(|c: char| c.is_numeric())(i)
}
