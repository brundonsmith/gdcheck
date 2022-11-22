use enum_variant_type::EnumVariantType;

use crate::utils::Src;

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Declaration {
    #[evt(derive(Debug, Clone, PartialEq))]
    Extends(String),

    #[evt(derive(Debug, Clone, PartialEq))]
    ClassName(String),

    #[evt(skip)]
    ValDeclaration(ValDeclaration),

    #[evt(derive(Debug, Clone, PartialEq))]
    Annotation {
        name: Src<String>,
        arguments: Vec<Src<Expression>>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Enum {
        name: Option<Src<String>>,
        variants: Vec<(Src<String>, Option<Src<Expression>>)>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Func {
        is_static: bool,
        name: String,
        args: Vec<(String, Option<Type>)>,
        return_type: Option<Type>,
        body: Block,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Class {
        name: Src<String>,
        declarations: Vec<Src<Declaration>>,
    },
}

impl From<ValDeclaration> for Declaration {
    fn from(vd: ValDeclaration) -> Self {
        Self::ValDeclaration(vd)
    }
}

type Block = Vec<Src<Statement>>;

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Expression {
    #[evt(derive(Debug, Clone, PartialEq))]
    Int { value: i64, base: NumberBase },

    #[evt(derive(Debug, Clone, PartialEq))]
    Float(f64),

    #[evt(derive(Debug, Clone, PartialEq))]
    GDString { value: String, multiline: bool },

    #[evt(derive(Debug, Clone, PartialEq))]
    Boolean(bool),

    #[evt(derive(Debug, Clone, PartialEq))]
    Null,

    #[evt(derive(Debug, Clone, PartialEq))]
    Array(Vec<Src<Expression>>),

    #[evt(derive(Debug, Clone, PartialEq))]
    Dictionary(Vec<(Src<Expression>, Src<Expression>)>),

    #[evt(derive(Debug, Clone, PartialEq))]
    UnaryOperation {
        op: Src<UnaryOperator>,
        subject: Box<Src<Expression>>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    BinaryOperation {
        op: Src<BinaryOperator>,
        left: Box<Src<Expression>>,
        right: Box<Src<Expression>>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, EnumVariantType)]
pub enum Statement {
    #[evt(skip)]
    ValDeclaration(ValDeclaration),

    #[evt(derive(Debug, Clone, PartialEq))]
    Assignment {
        target: Src<Expression>,
        value: Src<Expression>,
        operator: Option<Src<BinaryOperator>>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Match,

    #[evt(derive(Debug, Clone, PartialEq))]
    While {
        condition: Src<Expression>,
        body: Block,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    If {
        conditions: Vec<(Src<Expression>, Block)>,
        default_outcome: Option<Block>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    For {
        name: Src<String>,
        iteree: Src<Expression>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    Pass,

    #[evt(derive(Debug, Clone, PartialEq))]
    Break,

    #[evt(derive(Debug, Clone, PartialEq))]
    Return(Src<Expression>),
}

impl From<ValDeclaration> for Statement {
    fn from(vd: ValDeclaration) -> Self {
        Self::ValDeclaration(vd)
    }
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
    #[evt(derive(Debug, Clone, PartialEq))]
    NullType,

    #[evt(derive(Debug, Clone, PartialEq))]
    BoolType,

    #[evt(derive(Debug, Clone, PartialEq))]
    IntType,

    #[evt(derive(Debug, Clone, PartialEq))]
    FloatType,

    #[evt(derive(Debug, Clone, PartialEq))]
    StringType,

    #[evt(derive(Debug, Clone, PartialEq))]
    StringNameType,

    #[evt(derive(Debug, Clone, PartialEq))]
    Vector2Type,

    #[evt(derive(Debug, Clone, PartialEq))]
    Vector2iType,

    #[evt(derive(Debug, Clone, PartialEq))]
    Vector3Type,

    #[evt(derive(Debug, Clone, PartialEq))]
    Vector3iType,

    #[evt(derive(Debug, Clone, PartialEq))]
    Transform2DType,

    #[evt(derive(Debug, Clone, PartialEq))]
    PlaneType,

    #[evt(derive(Debug, Clone, PartialEq))]
    AABBType,

    #[evt(derive(Debug, Clone, PartialEq))]
    BasisType,

    #[evt(derive(Debug, Clone, PartialEq))]
    Transform3DType,

    #[evt(derive(Debug, Clone, PartialEq))]
    ColorType,

    #[evt(derive(Debug, Clone, PartialEq))]
    NodePathType,

    #[evt(derive(Debug, Clone, PartialEq))]
    RIDType,

    #[evt(derive(Debug, Clone, PartialEq))]
    ObjectType,
    // ArrayType,
    // DictionaryType,
    #[evt(derive(Debug, Clone, PartialEq))]
    NamedType(String),

    // gdcheck types
    #[evt(derive(Debug, Clone, PartialEq))]
    NonNullType(Box<Src<Type>>),

    #[evt(derive(Debug, Clone, PartialEq))]
    DictionaryType {
        key: Box<Src<Type>>,
        value: Box<Src<Type>>,
    },

    #[evt(derive(Debug, Clone, PartialEq))]
    ExactDictionaryType(Vec<(Src<Expression>, Src<Expression>)>),

    #[evt(derive(Debug, Clone, PartialEq))]
    ArrayType(Box<Src<Type>>),

    #[evt(derive(Debug, Clone, PartialEq))]
    AnyType,
}
