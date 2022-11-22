use enum_variant_type::EnumVariantType;

use crate::utils::Src;

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Declaration {
    Extends(String),
    ClassName(String),
    #[evt(skip)]
    ValDeclaration(ValDeclaration),
    Annotation {
        name: Src<String>,
        arguments: Vec<Src<Expression>>,
    },
    Enum {
        name: Option<Src<String>>,
        variants: Vec<(Src<String>, Option<Src<Expression>>)>,
    },
    Func {
        is_static: bool,
        name: String,
        args: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
        body: Block,
    },
    Class {
        name: Src<String>,
        declarations: Vec<Src<Declaration>>,
    },
}

type Block = Vec<Src<Statement>>;

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Expression {
    Int {
        value: i64,
        base: NumberBase,
    },
    Float(f64),
    GDString {
        value: String,
        multiline: bool,
    },
    Boolean(bool),
    Null,
    Array(Vec<Src<Expression>>),
    Dictionary(Vec<(Src<Expression>, Src<Expression>)>),
    UnaryOperation {
        op: Src<UnaryOperator>,
        subject: Box<Src<Expression>>,
    },
    BinaryOperation {
        op: Src<BinaryOperator>,
        left: Box<Src<Expression>>,
        right: Box<Src<Expression>>,
    },
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Statement {
    #[evt(skip)]
    ValDeclaration(ValDeclaration),
    Assignment {
        target: Src<Expression>,
        value: Src<Expression>,
        operator: Option<Src<BinaryOperator>>,
    },
    Match,
    While {
        condition: Src<Expression>,
        body: Block,
    },
    If {
        conditions: Vec<(Src<Expression>, Block)>,
        default_outcome: Option<Block>,
    },
    For {
        name: Src<String>,
        iteree: Src<Expression>,
    },
    Pass,
    Break,
    Return(Src<Expression>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValDeclaration {
    pub is_const: bool,
    pub name: Src<String>,
    pub declared_type: Option<Src<Type>>,
    pub value: Option<Src<Expression>>,
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

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Type {
    // Godot types
    NullType,
    BoolType,
    IntType,
    FloatType,
    StringType,
    StringNameType,
    Vector2Type,
    Vector2iType,
    Vector3Type,
    Vector3iType,
    Transform2DType,
    PlaneType,
    AABBType,
    BasisType,
    Transform3DType,
    ColorType,
    NodePathType,
    RIDType,
    ObjectType,
    // ArrayType,
    // DictionaryType,
    NamedType(String),

    // gdcheck types
    NonNullType(Box<Src<Type>>),
    DictionaryType {
        key: Box<Src<Type>>,
        value: Box<Src<Type>>,
    },
    ExactDictionaryType(Vec<(Src<Expression>, Src<Expression>)>),
    ArrayType(Box<Src<Type>>),
    AnyType,
}
