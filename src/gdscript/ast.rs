use std::fmt::Debug;
use std::marker::PhantomData;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use strum_macros::{EnumString, IntoStaticStr};

use crate::utils::slice::{Slicable, Slice};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleID(pub Rc<String>);

macro_rules! union_type {
    ($name:ident = $( $s:ident )|*) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum $name {
            $($s($s)),*
        }

        $(
            impl From<$s> for $name {
                fn from(variant: $s) -> Self {
                    $name::$s(variant)
                }
            }

            impl TryFrom<$name> for $s {
                type Error = ();

                fn try_from(un: $name) -> Result<Self, Self::Error> {
                    match un {
                        $name::$s(variant) => Ok(variant),
                        _ => Err(())
                    }
                }
            }
        )*
    };
}

macro_rules! union_subtype {
    ($name:ident = $( $s:ident )|*) => {
        union_type!($name = $($s)|*);

        impl From<$name> for Any {
            fn from(sub: $name) -> Self {
                match sub {
                    $(
                        $name::$s(s) => Any::$s(s),
                    )*
                }
            }
        }

        impl TryFrom<Any> for $name {
            type Error = ();

            fn try_from(det: Any) -> Result<Self, Self::Error> {
                match det {
                    $(
                        Any::$s(s) => Ok($name::$s(s)),
                    )*
                    _ => Err(())
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub struct AST<TKind>(Rc<ASTInner>, PhantomData<TKind>)
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>;

pub type ASTAny = AST<Any>;

impl<TKind> AST<TKind>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    pub fn details(&self) -> &Any {
        &self.0.details
    }

    pub fn ptr_eq<TOtherKind>(&self, other: &AST<TOtherKind>) -> bool
    where
        TOtherKind: Clone + TryFrom<Any>,
        Any: From<TOtherKind>,
    {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn parent(&self) -> Option<ASTAny> {
        self.0
            .parent
            .borrow()
            .as_ref()
            .map(|weak| weak.upgrade())
            .flatten()
            .map(|node| AST::<Any>(node, PhantomData))
    }

    pub fn downcast(&self) -> TKind {
        match TKind::try_from(self.0.details.clone()) {
            Ok(res) => res,
            Err(_) => unreachable!(),
        }
    }

    pub fn contains(&self, other: &AST<Any>) -> bool {
        let mut current = Some(other.clone());

        while let Some(some_current) = current {
            if self.ptr_eq::<Any>(&some_current) {
                return true;
            }

            current = some_current.parent();
        }

        return false;
    }

    pub fn try_downcast<TExpected>(&self) -> Option<TExpected>
    where
        TExpected: TryFrom<Any>,
        Any: From<TExpected>,
    {
        TExpected::try_from(self.0.details.clone()).ok()
    }

    pub fn upcast(self) -> ASTAny {
        AST::<Any>(self.0, PhantomData)
    }

    pub fn recast<TExpected>(self) -> AST<TExpected>
    where
        TExpected: Clone + TryFrom<Any> + From<TKind>,
        Any: From<TExpected>,
    {
        AST::<TExpected>(self.0, PhantomData)
    }

    pub fn try_recast<TExpected>(self) -> Option<AST<TExpected>>
    where
        TExpected: Clone + TryFrom<Any> + TryFrom<TKind>,
        Any: From<TExpected>,
    {
        TExpected::try_from(self.0.details.clone())
            .ok()
            .map(|_| AST::<TExpected>(self.0, PhantomData))
    }
}

impl<TKind> Slicable for AST<TKind>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    fn slice(&self) -> &Slice {
        &self.0.slice
    }
}

impl<TKind> Slicable for &AST<TKind>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    fn slice(&self) -> &Slice {
        &self.0.slice
    }
}

pub trait Parentable {
    fn set_parent<TParentKind>(&mut self, parent: &AST<TParentKind>)
    where
        TParentKind: Clone + TryFrom<Any>,
        Any: From<TParentKind>,
    {
        // do nothing by default
    }
}

impl<TKind> Parentable for AST<TKind>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    fn set_parent<TParentKind>(&mut self, parent: &AST<TParentKind>)
    where
        TParentKind: Clone + TryFrom<Any>,
        Any: From<TParentKind>,
    {
        self.0
            .as_ref()
            .parent
            .replace(Some(Rc::downgrade(&parent.0)));
    }
}

impl<T> Parentable for Option<T>
where
    T: Parentable,
{
    fn set_parent<TParentKind>(&mut self, parent: &AST<TParentKind>)
    where
        TParentKind: Clone + TryFrom<Any>,
        Any: From<TParentKind>,
    {
        if let Some(s) = self {
            s.set_parent(parent);
        }
    }
}

impl<T> Parentable for Vec<T>
where
    T: Parentable,
{
    fn set_parent<TParentKind>(&mut self, parent: &AST<TParentKind>)
    where
        TParentKind: Clone + TryFrom<Any>,
        Any: From<TParentKind>,
    {
        for ast in self.iter_mut() {
            ast.set_parent(parent);
        }
    }
}

impl<T, U> Parentable for (T, U)
where
    T: Parentable,
    U: Parentable,
{
    fn set_parent<TParentKind>(&mut self, parent: &AST<TParentKind>)
    where
        TParentKind: Clone + TryFrom<Any>,
        Any: From<TParentKind>,
    {
        self.0.set_parent(parent);
        self.1.set_parent(parent);
    }
}

// just for the sake of make_node!() macros
impl Parentable for bool {}
impl Parentable for Slice {}

#[derive(Debug, Clone)]
pub struct ASTInner {
    pub parent: RefCell<Option<Weak<ASTInner>>>,
    pub slice: Slice,
    pub details: Any,
}

impl PartialEq for ASTInner {
    fn eq(&self, other: &Self) -> bool {
        self.parent.borrow().as_ref().map(|w| w.as_ptr())
            == other.parent.borrow().as_ref().map(|w| w.as_ptr())
            && self.slice == other.slice
            && self.details == other.details
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GDScript {
    pub declarations: Vec<AST<Declaration>>,
}

// --- Declarations ---
#[derive(Debug, Clone, PartialEq)]
pub struct ExtendsDeclaration {
    pub extends_class: AST<PlainIdentifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassNameDeclaration {
    pub class_name: AST<PlainIdentifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueDeclaration {
    pub is_const: bool,
    pub name: AST<PlainIdentifier>,
    pub declared_type: Option<AST<TypeExpression>>,
    pub value: Option<AST<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: AST<PlainIdentifier>,
    pub arguments: Vec<AST<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration {
    pub name: Option<AST<PlainIdentifier>>,
    pub variants: Vec<(AST<PlainIdentifier>, Option<AST<Expression>>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDeclaration {
    pub is_static: bool,
    pub name: AST<PlainIdentifier>,
    pub args: Vec<(AST<PlainIdentifier>, Option<AST<TypeExpression>>)>,
    pub return_type: Option<AST<TypeExpression>>,
    pub body: AST<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclaration {
    pub name: AST<PlainIdentifier>,
    pub declarations: Vec<AST<Declaration>>,
}

// --- Expressions ---
#[derive(Debug, Clone, PartialEq)]
pub struct NullLiteral;

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanLiteral {
    pub value: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntLiteral {
    pub value_raw: Slice,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatLiteral {
    pub value_raw: Slice,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringLiteral {
    pub value: Slice,
    pub multiline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    pub members: Vec<AST<Expression>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryLiteral {
    pub entries: Vec<(AST<Expression>, AST<Expression>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryOperation {
    pub op: AST<UnaryOperator>,
    pub subject: AST<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryOperation {
    pub op: AST<BinaryOperator>,
    pub left: AST<Expression>,
    pub right: AST<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalIdentifier {
    pub name: Slice,
}

// --- Type expressions ---

#[derive(Debug, Clone, PartialEq)]
pub struct NullType;

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanType;

#[derive(Debug, Clone, PartialEq)]
pub struct IntType;

#[derive(Debug, Clone, PartialEq)]
pub struct FloatType;

#[derive(Debug, Clone, PartialEq)]
pub struct StringType;

#[derive(Debug, Clone, PartialEq)]
pub struct StringNameType;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector2Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector2iType;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector3Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Vector3iType;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform2DType;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaneType;

#[derive(Debug, Clone, PartialEq)]
pub struct AABBType;

#[derive(Debug, Clone, PartialEq)]
pub struct BasisType;

#[derive(Debug, Clone, PartialEq)]
pub struct Transform3DType;

#[derive(Debug, Clone, PartialEq)]
pub struct ColorType;

#[derive(Debug, Clone, PartialEq)]
pub struct NodePathType;

#[derive(Debug, Clone, PartialEq)]
pub struct RIDType;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType;

#[derive(Debug, Clone, PartialEq)]
pub struct NamedType {
    pub name: AST<PlainIdentifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonNullType {
    pub inner: AST<TypeExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayType {
    pub element: AST<TypeExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DictionaryType {
    pub key: AST<TypeExpression>,
    pub value: AST<TypeExpression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExactDictionaryType {
    pub entries: Vec<(AST<TypeExpression>, AST<TypeExpression>)>,
}

// --- Statements ---

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStatement {
    pub target: AST<Expression>,
    pub value: AST<Expression>,
    pub operator: Option<AST<BinaryOperator>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchStatement;

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: AST<Expression>,
    pub body: AST<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfElseStatement {
    pub conditions: Vec<(AST<Expression>, AST<Block>)>,
    pub default_outcome: Option<AST<Block>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    pub item_name: AST<PlainIdentifier>,
    pub iteree: AST<Expression>,
    pub body: AST<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pass;

#[derive(Debug, Clone, PartialEq)]
pub struct Break;

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub expr: Option<AST<Expression>>,
}

// --- Misc ---

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<AST<Statement>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlainIdentifier {
    pub name: Slice,
}

// --- Utils ---

pub fn covering<TKind>(vec: &Vec<AST<TKind>>) -> Option<Slice>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    vec.get(0)
        .map(|first| first.slice().clone().join(vec[vec.len() - 1].slice()))
}

pub trait WithSlice: Sized
where
    Self: Clone + TryFrom<Any>,
    Any: From<Self>,
{
    fn as_ast(self, src: Slice) -> AST<Self>;
}

impl<TKind> WithSlice for TKind
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    fn as_ast(self, src: Slice) -> AST<TKind> {
        AST(
            Rc::new(ASTInner {
                parent: RefCell::new(None),
                slice: src,
                details: self.into(),
            }),
            PhantomData,
        )
    }
}

// --- Misc data ---

pub fn identifier_to_string(ast: AST<PlainIdentifier>) -> AST<StringLiteral> {
    StringLiteral {
        value: ast.downcast().name.clone(),
        multiline: false,
    }
    .as_ast(ast.slice().clone())
}

// pub fn identifier_to_string_type(ast: AST<PlainIdentifier>) -> AST<StringType> {
//     StringLiteralType(ast.downcast().0.clone()).as_ast(ast.slice().clone())
// }

// --- AST groups ---

union_type! {
    Any = GDScript
        | ExtendsDeclaration
        | ClassNameDeclaration
        | ValueDeclaration
        | Annotation
        | EnumDeclaration
        | FuncDeclaration
        | ClassDeclaration
        | NullLiteral
        | BooleanLiteral
        | IntLiteral
        | FloatLiteral
        | StringLiteral
        | ArrayLiteral
        | DictionaryLiteral
        | UnaryOperation
        | BinaryOperation
        | LocalIdentifier
        | NullType
        | BooleanType
        | IntType
        | FloatType
        | StringType
        | StringNameType
        | Vector2Type
        | Vector2iType
        | Vector3Type
        | Vector3iType
        | Transform2DType
        | PlaneType
        | AABBType
        | BasisType
        | Transform3DType
        | ColorType
        | NodePathType
        | RIDType
        | ObjectType
        | NamedType
        | NonNullType
        | ArrayType
        | DictionaryType
        | ExactDictionaryType
        | AssignmentStatement
        | MatchStatement
        | WhileLoop
        | IfElseStatement
        | ForLoop
        | Pass
        | Break
        | Return
        | Block
        | PlainIdentifier
        | UnaryOperator
        | BinaryOperator
}

union_subtype!(
    Declaration = ExtendsDeclaration
        | ClassNameDeclaration
        | ValueDeclaration
        | Annotation
        | EnumDeclaration
        | FuncDeclaration
        | ClassDeclaration
);

union_subtype!(
    Expression = NullLiteral
        | BooleanLiteral
        | IntLiteral
        | FloatLiteral
        | StringLiteral
        | ArrayLiteral
        | DictionaryLiteral
        | UnaryOperation
        | BinaryOperation
        | LocalIdentifier
);

union_subtype!(
    TypeExpression = NullType
        | BooleanType
        | IntType
        | FloatType
        | StringType
        | StringNameType
        | Vector2Type
        | Vector2iType
        | Vector3Type
        | Vector3iType
        | Transform2DType
        | PlaneType
        | AABBType
        | BasisType
        | Transform3DType
        | ColorType
        | NodePathType
        | RIDType
        | ObjectType
        | NamedType
        | NonNullType
        | ArrayType
        | DictionaryType
        | ExactDictionaryType
);

union_subtype!(
    Statement = ValueDeclaration
        | AssignmentStatement
        | MatchStatement
        | WhileLoop
        | IfElseStatement
        | ForLoop
        | Pass
        | Break
        | Return
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
pub enum UnaryOperator {
    #[strum(serialize = "~")]
    BitwiseNot,

    #[strum(serialize = "-")]
    Negative,

    #[strum(serialize = "!")]
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, IntoStaticStr)]
pub enum BinaryOperator {
    #[strum(serialize = "is")]
    Is,

    #[strum(serialize = "**")]
    DoubleStar,

    #[strum(serialize = "*")]
    Star,

    #[strum(serialize = "/")]
    Slash,

    #[strum(serialize = "%")]
    Percent,

    #[strum(serialize = "+")]
    Plus,

    #[strum(serialize = "-")]
    Minus,

    #[strum(serialize = "<<")]
    ShiftLeft,

    #[strum(serialize = ">>")]
    ShiftRight,

    #[strum(serialize = "&")]
    BitwiseAnd,

    #[strum(serialize = "^")]
    BitwiseXor,

    #[strum(serialize = "|")]
    BitwiseOr,

    #[strum(serialize = "<")]
    Less,

    #[strum(serialize = ">")]
    Greater,

    #[strum(serialize = "==")]
    Equals,

    #[strum(serialize = "!=")]
    NotEquals,

    #[strum(serialize = ">=")]
    GreaterEqual,

    #[strum(serialize = "<=")]
    LessEqual,

    #[strum(serialize = "in")]
    In,

    #[strum(serialize = "and")]
    And,

    #[strum(serialize = "or")]
    Or,

    #[strum(serialize = "as")]
    As,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberBase {
    Two,
    Ten,
    Sixteen,
}

impl AST<Any> {
    pub fn find_parent<F: Fn(&AST<Any>) -> bool>(&self, f: F) -> Option<AST<Any>> {
        let mut current: Option<AST<Any>> = Some(self.clone());

        while let Some(parent) = current {
            if f(&parent) {
                return Some(parent.clone());
            }

            current = parent.parent();
        }

        return None;
    }

    pub fn find_parent_of_type<TExpected>(&self) -> Option<AST<TExpected>>
    where
        TExpected: TryFrom<Any> + Clone,
        Any: From<TExpected>,
    {
        self.find_parent(|p| p.try_downcast::<TExpected>().is_some())
            .map(|module| module.try_recast::<TExpected>().unwrap())
    }
}
