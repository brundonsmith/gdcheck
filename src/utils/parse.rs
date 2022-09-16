use std::{
    fmt::{Debug, Display},
    ops::Range,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseResult<T: Clone + Debug + PartialEq> {
    Some(ParsedAndIndex<T>),
    Err(ParseError),
    None,
}

impl<T: Clone + Debug + PartialEq> From<Option<ParsedAndIndex<T>>> for ParseResult<T> {
    fn from(opt: Option<ParsedAndIndex<T>>) -> Self {
        match opt {
            Some(val) => ParseResult::Some(val),
            None => ParseResult::None,
        }
    }
}

impl<T: Clone + Debug + PartialEq> ParseResult<T> {
    pub fn map<R: Clone + Debug + PartialEq, F: FnOnce(T) -> R>(self, cb: F) -> ParseResult<R> {
        match self {
            ParseResult::Some(ParsedAndIndex { parsed, index }) => {
                ParseResult::Some(ParsedAndIndex {
                    parsed: cb(parsed),
                    index,
                })
            }
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::None => ParseResult::None,
        }
    }

    pub fn given<R: Clone + Debug + PartialEq, F: FnOnce(ParsedAndIndex<T>) -> ParseResult<R>>(
        self,
        cb: F,
    ) -> ParseResult<R> {
        match self {
            ParseResult::Some(parsed) => cb(parsed),
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::None => ParseResult::None,
        }
    }

    pub fn expec<R: Clone + Debug + PartialEq, F: FnOnce(ParsedAndIndex<T>) -> ParseResult<R>>(
        self,
        err: &str,
        index: usize,
        cb: F,
    ) -> ParseResult<R> {
        match self {
            ParseResult::Some(parsed) => cb(parsed),
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::None => ParseResult::Err(ParseError {
                msg: err.into(),
                index,
            }),
        }
    }

    pub fn or<F: FnOnce() -> Self>(self, cb: F) -> Self {
        match self {
            ParseResult::None => cb(),
            _ => self,
        }
    }

    pub fn optional<R: Clone + Debug + PartialEq, F: FnOnce(ParsedAndIndex<Option<T>>) -> ParseResult<R>>(
        self, 
        index: usize, 
        cb: F
    ) -> ParseResult<R> {
        match self {
            ParseResult::Some(parsed) => cb(parsed.map(Some)),
            ParseResult::Err(err) => ParseResult::Err(err),
            ParseResult::None => cb(ParsedAndIndex { parsed: None, index }),
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            ParseResult::Some(parsed) => parsed.parsed,
            _ => panic!("Unwrapped bad ParseResult"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedAndIndex<T: Clone + Debug + PartialEq> {
    pub parsed: T,
    pub index: usize,
}

impl<T: Clone + Debug + PartialEq> ParsedAndIndex<T> {
    pub fn map<R: Clone + Debug + PartialEq, F: FnOnce(T) -> R>(self, cb: F) -> ParsedAndIndex<R> {
        ParsedAndIndex {
            parsed: cb(self.parsed),
            index: self.index,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sourced<T: Debug + Clone + PartialEq> {
    pub ast: T,
    pub span: Option<Range<usize>>,
}

impl<T: Debug + Clone + PartialEq> Sourced<T> {
    pub fn map<R: Clone + Debug + PartialEq, F: FnOnce(T) -> R>(self, cb: F) -> Sourced<R> {
        Sourced {
            ast: cb(self.ast),
            span: self.span,
        }
    }
}

pub trait Sourcable: Debug + Clone + PartialEq {
    fn spanning(self, span: Range<usize>) -> Sourced<Self> {
        Sourced {
            ast: self,
            span: Some(span),
        }
    }
    fn unsourced(self) -> Sourced<Self> {
        Sourced {
            ast: self,
            span: None,
        }
    }
}

impl<T: Debug + Clone + PartialEq> Sourcable for T {
    fn spanning(self, span: Range<usize>) -> Sourced<Self> {
        Sourced {
            ast: self,
            span: Some(span),
        }
    }

    fn unsourced(self) -> Sourced<Self> {
        Sourced {
            ast: self,
            span: None,
        }
    }
}

/**
 * An error that occurred while parsing a string as lisp code
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub msg: String,
    pub index: usize,
}

impl Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        return write!(
            formatter,
            "Parse error at index {}: {}",
            self.index, self.msg
        );
    }
}

impl<T: Clone + Debug + PartialEq> ParsedAndIndex<T> {
    pub fn map_parsed<R: Clone + Debug + PartialEq, F: FnOnce(T) -> R>(
        self,
        cb: F,
    ) -> ParsedAndIndex<R> {
        ParsedAndIndex {
            parsed: cb(self.parsed),
            index: self.index,
        }
    }
}

pub fn consume<'a>(code: &'a str, index: usize, s: &'static str) -> ParseResult<&'a str> {
    let slice = code.get(index..).unwrap_or("");

    if slice.len() >= s.len()
        && slice
            .chars()
            .zip(s.chars())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
    {
        ParseResult::Some(ParsedAndIndex {
            parsed: s,
            index: index + s.len(),
        })
    } else {
        ParseResult::None
    }
}

#[test]
fn test_consume() {
    let res = consume("dfjkhgsdfg", 3, "k");
    assert_eq!(
        res,
        ParseResult::Some(ParsedAndIndex {
            parsed: "k",
            index: 4
        })
    );
}

pub fn parse_whitespace_and_comments(
    code: &str,
    start_index: usize,
    comment_start: char,
    required: bool,
) -> ParseResult<usize> {
    let index = consume_whitespace_and_comments(code, start_index, comment_start).unwrap();

    if !required || index > start_index {
        ParseResult::Some(ParsedAndIndex {
            parsed: index,
            index,
        })
    } else {
        ParseResult::Err(ParseError {
            msg: "Expected whitespace".into(),
            index,
        })
    }
}

pub fn consume_whitespace_and_comments(
    code: &str,
    index: usize,
    comment_start: char,
) -> ParseResult<usize> {
    let mut in_comment = false;

    let index = index
        + consume_while(code, index, move |(_, ch)| {
            if ch == comment_start {
                in_comment = true;
            } else if ch == '\n' {
                in_comment = false;
            }

            return ch.is_whitespace() || ch == comment_start || in_comment;
        })
        .unwrap()
        .len();

    ParseResult::Some(ParsedAndIndex {
        parsed: index,
        index,
    })
}

pub fn consume_while<F: FnMut((usize, char)) -> bool>(
    code: &str,
    start_index: usize,
    mut pred: F,
) -> ParseResult<&str> {
    let last = code
        .get(start_index..)
        .unwrap_or("")
        .char_indices()
        .take_while(|(i, c)| pred((*i, *c)))
        .last()
        .map(|(length, _)| start_index + length + 1)
        .unwrap_or(start_index);

    ParseResult::Some(ParsedAndIndex {
        parsed: &code[start_index..last],
        index: last,
    })
}

#[test]
fn test_consume_while() {
    let res = consume_while("abc123", 0, |(_, ch)| ch.is_alphabetic());

    assert_eq!(
        res,
        ParseResult::Some(ParsedAndIndex {
            parsed: "abc",
            index: 3
        })
    );
}

pub fn next_char_is_break(code: &str, index: usize) -> bool {
    code.get(index..)
        .map(|s| s.chars().next())
        .flatten()
        .map(|ch| ch.is_whitespace())
        .unwrap_or(true)
}

pub trait Log: Debug {
    fn deb(self) -> Self;
}

impl<T: Debug> Log for T {
    fn deb(self) -> Self {
        println!("{:?}", &self);
        self
    }
}

// use std::{
//     fmt::Display,
//     rc::{Rc, Weak},
// };

// #[derive(Clone, Debug)]
// pub struct SourceInfo {
//     pub parent: Option<Weak<ASTEnum>>,
//     pub module: Option<ModuleName>,
//     pub start_index: Option<usize>,
//     pub end_index: Option<usize>,
// }

// impl SourceInfo {
//     pub fn empty() -> Self {
//         Self {
//             parent: None,
//             module: None,
//             start_index: None,
//             end_index: None,
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct ModuleName(pub String);

// #[derive(Clone, Debug)]
// pub enum AST<T: Display> {
//     Ok(Rc<WithSourceInfo<T>>),
//     ParseError,
// }

// impl<T: Display> Display for AST<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             AST::Ok(x) => Display::fmt(&x.node, f),
//             AST::ParseError => f.write_str("<parse error>"),
//         }
//     }
// }

// #[derive(Clone, Debug)]
// pub struct WithSourceInfo<T: Display> {
//     pub source_info: SourceInfo,
//     pub node: T,
// }

// impl<T: Display> Display for WithSourceInfo<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         self.node.fmt(f)
//     }
// }

// impl<T: Display> WithSourceInfo<T> {
//     pub fn empty(node: T) -> Self {
//         Self {
//             source_info: SourceInfo::empty(),
//             node,
//         }
//     }
// }

// pub fn visit_ast<F: FnMut(&ASTEnum)>(ast: &ASTEnum, cb: &mut F) {
//     cb(ast);

//     match &ast {
//         ASTEnum::Expression(AST::Ok(x)) => match &x.node {
//             Expression::NilLiteral => {}
//             Expression::NumberLiteral { value: _ } => {}
//             Expression::BinaryOperator { left, op: _, right } => {
//                 visit_ast(&left.clone().into(), cb);
//                 visit_ast(&right.clone().into(), cb);
//             }
//             Expression::Parenthesis { inner } => {
//                 visit_ast(&inner.clone().into(), cb);
//             }
//         },
//         ASTEnum::TypeExpression(AST::Ok(x)) => match &x.node {
//             TypeExpression::UnknownType => {}
//             TypeExpression::NilType => {}
//             TypeExpression::BooleanType => {}
//             TypeExpression::NumberType => {}
//             TypeExpression::StringType => {}
//         },
//         ASTEnum::PlainIdentifier(_) => {}
//         ASTEnum::NameAndType { name, typ } => {
//             visit_ast(&name.clone().into(), cb);

//             if let Some(typ) = typ {
//                 visit_ast(&typ.clone().into(), cb);
//             }
//         }
//         _ => {}
//     };
// }

// #[derive(Clone, Debug)]
// pub enum ASTEnum {
//     // Module {
//     //     module_type: ModuleType,
//     //     declarations: Vec<AST>,
//     // },

//     // Declaration(Declaration),
//     Expression(AST<Expression>),
//     TypeExpression(AST<TypeExpression>),
//     // Statement(Statement),

//     // Attribute,
//     PlainIdentifier(AST<PlainIdentifier>),
//     // Block {
//     //     statements: Vec<AST>,
//     // },
//     // Operator,
//     // Case,
//     // SwitchCase,
//     // CaseBlock,
//     // Args,
//     // Arg,
//     // SpreadArgs,
//     // ImportItem,
//     // Spread,
//     // InlineDeclaration,
//     // ObjectEntry,
//     NameAndType {
//         name: AST<PlainIdentifier>,
//         typ: Option<AST<TypeExpression>>,
//     },
//     // Destructure {
//     //     properties: Vec<AST>,
//     //     spread: Option<AST>,
//     // },
//     // Decorator,
// }

// #[derive(Clone, Debug)]
// pub struct PlainIdentifier(pub String);

// impl Display for PlainIdentifier {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(self.0.as_str())
//     }
// }

// #[derive(Clone, Copy, Debug)]
// pub enum ModuleType {
//     Bgl,
//     Json,
//     Text,
// }

// impl Display for ASTEnum {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             ASTEnum::Expression(AST::Ok(x)) => Display::fmt(x, f),
//             ASTEnum::TypeExpression(AST::Ok(x)) => Display::fmt(x, f),
//             ASTEnum::PlainIdentifier(AST::Ok(x)) => Display::fmt(x, f),
//             ASTEnum::NameAndType {
//                 name: AST::Ok(name),
//                 typ: Some(AST::Ok(typ)),
//             } => {
//                 f.write_str(name.node.0.as_str())?;
//                 f.write_str(": ")?;
//                 Display::fmt(&typ.node, f)
//             }
//             ASTEnum::NameAndType {
//                 name: AST::Ok(name),
//                 typ: None,
//             } => f.write_str(name.node.0.as_str()),
//             _ => f.write_str("<parse error>"),
//         }
//     }
// }

// pub fn ast_from(expr: Expression) -> AST<Expression> {
//     AST::Ok(Rc::new(WithSourceInfo::empty(expr)))
// }

// impl From<AST<Expression>> for ASTEnum {
//     fn from(expr: AST<Expression>) -> Self {
//         Self::Expression(expr)
//     }
// }

// impl From<AST<TypeExpression>> for ASTEnum {
//     fn from(expr: AST<TypeExpression>) -> Self {
//         Self::TypeExpression(expr)
//     }
// }

// impl From<AST<PlainIdentifier>> for ASTEnum {
//     fn from(expr: AST<PlainIdentifier>) -> Self {
//         Self::PlainIdentifier(expr)
//     }
// }
