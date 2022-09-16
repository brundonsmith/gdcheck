use std::{collections::HashMap};

use crate::utils::parse::Sourced;

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Extends(String),
    ClassName(String),
    ValDeclaration(ValDeclaration),
    Annotation(Annotation),
    Enum {
        name: Option<Sourced<String>>,
        variants: Vec<(Sourced<String>, Option<Sourced<Expression>>)>,
    },
    Func {
        is_static: bool,
        name: String,
        args: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
        body: Block,
    },
    Class(Class),
}

type Block = Vec<Sourced<Statement>>;

#[derive(Debug, Clone, PartialEq)]
pub struct ValDeclaration {
    pub is_const: bool,
    pub name: Sourced<String>,
    pub declared_type: Option<Sourced<Type>>,
    pub value: Option<Sourced<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: Sourced<String>,
    pub arguments: Vec<Sourced<Expression>>,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: Sourced<String>,
    pub declarations: Vec<Sourced<Declaration>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Int {
        value: i64,
        base: NumberBase,
    },
    Float(f64),
    String {
        value: String,
        multiline: bool,
    },
    Boolean(bool),
    Null,
    Array(Vec<Sourced<Expression>>),
    Dictionary(Vec<(Sourced<Expression>, Sourced<Expression>)>),
    UnaryOperation {
        op: Sourced<UnaryOperator>,
        subject: Box<Sourced<Expression>>,
    },
    BinaryOperation {
        op: Sourced<BinaryOperator>,
        left: Box<Sourced<Expression>>,
        right: Box<Sourced<Expression>>,
    },
    Identifier(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    BitwiseNot,
    Negative,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Is,
    DoubleStar,
    Star,
    Slash,
    Percent,
    Plus,
    Minus,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseXor,
    BitwiseOr,
    Less,
    Greater,
    Equals,
    NotEquals,
    GreaterEqual,
    LessEqual,
    In,
    And,
    Or,
    As,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberBase {
    Two,
    Ten,
    Sixteen,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Godot types
    Null,
    Bool,
    Int,
    Float,
    String,
    StringName,
    Vector2,
    Vector2i,
    Vector3,
    Vector3i,
    Transform2D,
    Plane,
    AABB,
    Basis,
    Transform3D,
    Color,
    NodePath,
    RID,
    Object,
    // Array,
    // Dictionary,
    Named(String),

    // gdcheck types
    NonNull(Box<Sourced<Type>>),
    Dictionary {
        key: Box<Sourced<Type>>,
        value: Box<Sourced<Type>>,
    },
    ExactDictionary(Vec<(Sourced<Expression>, Sourced<Expression>)>),
    Array(Box<Sourced<Type>>),
    Any,
}

impl Type {
    pub fn from_str(s: &str) -> Self {
        match s {
            "null" => Type::Null,
            "bool" => Type::Bool,
            "int" => Type::Int,
            "float" => Type::Float,
            "String" => Type::String,
            _ => Type::Named(s.into())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    ValDeclaration(ValDeclaration),
    Assignment {
        target: Sourced<Expression>,
        value: Sourced<Expression>,
        operator: Option<Sourced<BinaryOperator>>,
    },
    Match,
    While {
        condition: Sourced<Expression>,
        body: Block,
    },
    If {
        conditions: Vec<(Sourced<Expression>, Block)>,
        default_outcome: Option<Block>,
    },
    For {
        name: Sourced<String>,
        iteree: Sourced<Expression>,
    },
    Pass,
    Break,
    Return(Sourced<Expression>),
}