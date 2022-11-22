use nom::{
    bytes::complete::{escaped, tag, take_while, take_while1},
    character::complete::one_of,
    combinator::{cut, map, opt},
    error::{context, VerboseError},
    sequence::{pair, tuple},
    IResult,
};

use self::{slice::Slice, string_and_slice::StringAndSlice};

pub mod errors;
pub mod slice;
pub mod string_and_slice;

#[derive(Clone, Debug, PartialEq)]
pub struct Src<T> {
    pub src: Option<Slice>,
    pub node: T,
}

impl<T: Clone + std::fmt::Debug + PartialEq> Src<T> {
    pub fn contains(&self, other: Slice) -> bool {
        self.src.map(|s| s.contains(other)).unwrap_or(false)
    }

    pub fn spanning<O>(&self, other: &Src<O>) -> Option<Slice> {
        self.src
            .map(|left_src| other.src.map(move |right_src| left_src.spanning(right_src)))
            .flatten()
    }

    pub fn map<O, F: Fn(T) -> O>(self, f: F) -> Src<O> {
        Src {
            src: self.src,
            node: f(self.node),
        }
    }
}

pub type ParseResult<'a, T> = IResult<StringAndSlice<'a>, T, VerboseError<StringAndSlice<'a>>>;

pub fn string_literal<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<String>> {
    context(
        "string",
        map(
            pair(tag("\""), cut(pair(string_contents, tag("\"")))),
            |(open_quote, (contents, close_quote))| Src {
                src: Some(open_quote.slice.spanning(close_quote.slice)),
                node: contents.as_str().to_owned(),
            },
        ),
    )(i)
}

pub fn string_contents<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, StringAndSlice<'a>> {
    escaped(take_while(|ch: char| ch != '"'), '\\', one_of("\"n\\"))(i)
}

pub fn number_literal<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, Src<String>> {
    map(
        tuple((opt(tag("-")), numeric, opt(tuple((tag("."), cut(numeric)))))),
        |(neg, int, tail)| {
            let front = neg.unwrap_or(int);
            let back = tail.map(|(_, decimal)| decimal).unwrap_or(int);
            let full = front.spanning(&back);

            Src {
                src: Some(full.slice),
                node: full.as_str().to_owned(),
            }
        },
    )(i)
}

pub fn numeric<'a>(i: StringAndSlice<'a>) -> ParseResult<'a, StringAndSlice<'a>> {
    take_while1(|c: char| c.is_numeric())(i)
}
