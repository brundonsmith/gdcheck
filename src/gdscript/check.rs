use crate::{
    godot_project::GodotProject,
    utils::{
        errors::GDError,
        slice::{Slicable, Slice},
    },
};

use super::{ast::*, gd_type::Type};

#[derive(Clone, Copy, Debug)]
pub struct CheckContext<'a> {
    pub module_id: &'a ModuleID,
    pub godot_project: &'a GodotProject,
}

pub trait Checkable {
    fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F);
}

impl<TKind> Checkable for AST<TKind>
where
    TKind: Clone + TryFrom<Any> + TryFrom<TKind>,
    Any: From<TKind>,
{
    fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
        let module_id = &ctx.module_id.clone();
        // let subsumation_context = SubsumationContext::from(ctx);
        let check_subsumation =
            |destination: &Type, value: Type, slice: &Slice, report_error: &mut F| {
                let subsumes = destination.subsumes(&value);

                if !subsumes {
                    report_error(GDError::CheckError {
                        module_id: module_id.clone(),
                        src: Some(slice.clone()),
                        message: format!("Can't assign!"), // TODO
                    });
                }
            };

        match self.details() {
            Any::GDScript(GDScript { declarations }) => {
                declarations.check(ctx, report_error);
            }
            Any::ExtendsDeclaration(ExtendsDeclaration { extends_class }) => {
                extends_class.check(ctx, report_error);
            }
            Any::ClassNameDeclaration(ClassNameDeclaration { class_name }) => {
                class_name.check(ctx, report_error);
            }
            Any::ValueDeclaration(ValueDeclaration {
                is_const,
                name,
                declared_type,
                value,
            }) => {
                name.check(ctx, report_error);
                declared_type.check(ctx, report_error);
                value.check(ctx, report_error);

                if let (Some(declared_type), Some(value)) = (declared_type, value) {
                    let declared_type = declared_type.resolve_type(ctx.into());
                    let value_type = value.infer_type(ctx.into());

                    check_subsumation(&declared_type, value_type, value.slice(), report_error);
                }
            }
            Any::Annotation(Annotation { name, arguments }) => {
                name.check(ctx, report_error);
                arguments.check(ctx, report_error);
            }
            Any::EnumDeclaration(EnumDeclaration { name, variants }) => {
                name.check(ctx, report_error);
                variants.check(ctx, report_error);
            }
            Any::FuncDeclaration(FuncDeclaration {
                is_static,
                name,
                args,
                return_type,
                body,
            }) => {
                name.check(ctx, report_error);
                args.check(ctx, report_error);
                return_type.check(ctx, report_error);
                body.check(ctx, report_error);
            }
            Any::ClassDeclaration(ClassDeclaration { name, declarations }) => {
                name.check(ctx, report_error);
                declarations.check(ctx, report_error);
            }
            Any::ArrayLiteral(ArrayLiteral { members }) => members.check(ctx, report_error),
            Any::DictionaryLiteral(DictionaryLiteral { entries }) => {
                entries.check(ctx, report_error)
            }
            Any::UnaryOperation(UnaryOperation { op, subject }) => {
                op.check(ctx, report_error);
                subject.check(ctx, report_error);
            }
            Any::BinaryOperation(BinaryOperation { op, left, right }) => {
                op.check(ctx, report_error);
                left.check(ctx, report_error);
                right.check(ctx, report_error);
            }
            Any::LocalIdentifier(LocalIdentifier { name }) => todo!(),
            Any::NamedType(NamedType { name }) => todo!(),
            Any::NonNullType(NonNullType { inner }) => {
                inner.check(ctx, report_error);
            }
            Any::ArrayType(ArrayType { element }) => element.check(ctx, report_error),
            Any::DictionaryType(DictionaryType { key, value }) => {
                key.check(ctx, report_error);
                value.check(ctx, report_error);
            }
            Any::ExactDictionaryType(ExactDictionaryType { entries }) => {
                entries.check(ctx, report_error)
            }
            Any::AssignmentStatement(AssignmentStatement {
                target,
                value,
                operator,
            }) => {
                target.check(ctx, report_error);
                value.check(ctx, report_error);
                operator.check(ctx, report_error);
            }
            Any::WhileLoop(WhileLoop { condition, body }) => {
                condition.check(ctx, report_error);
                body.check(ctx, report_error);
            }
            Any::IfElseStatement(IfElseStatement {
                conditions,
                default_outcome,
            }) => {
                conditions.check(ctx, report_error);
                default_outcome.check(ctx, report_error);
            }
            Any::ForLoop(ForLoop {
                item_name,
                iteree,
                body,
            }) => {
                item_name.check(ctx, report_error);
                iteree.check(ctx, report_error);
                body.check(ctx, report_error);
            }
            Any::Return(Return { expr }) => expr.check(ctx, report_error),
            Any::Block(Block { statements }) => statements.check(ctx, report_error),

            Any::NullLiteral(NullLiteral) => {}
            Any::BooleanLiteral(BooleanLiteral { value }) => {}
            Any::IntLiteral(IntLiteral { value_raw }) => {}
            Any::FloatLiteral(FloatLiteral { value_raw }) => {}
            Any::StringLiteral(StringLiteral { value, multiline }) => {}
            Any::NullType(NullType) => {}
            Any::BooleanType(BooleanType) => {}
            Any::IntType(IntType) => {}
            Any::FloatType(FloatType) => {}
            Any::StringType(StringType) => {}
            Any::StringNameType(StringNameType) => {}
            Any::Vector2Type(Vector2Type) => {}
            Any::Vector2iType(Vector2iType) => {}
            Any::Vector3Type(Vector3Type) => {}
            Any::Vector3iType(Vector3iType) => {}
            Any::Transform2DType(Transform2DType) => {}
            Any::PlaneType(PlaneType) => {}
            Any::AABBType(AABBType) => {}
            Any::BasisType(BasisType) => {}
            Any::Transform3DType(Transform3DType) => {}
            Any::ColorType(ColorType) => {}
            Any::NodePathType(NodePathType) => {}
            Any::RIDType(RIDType) => {}
            Any::ObjectType(ObjectType) => {}
            Any::MatchStatement(MatchStatement) => {}
            Any::Pass(Pass) => {}
            Any::Break(Break) => {}
            Any::PlainIdentifier(PlainIdentifier { name }) => {}
            Any::UnaryOperator(op) => {}
            Any::BinaryOperator(op) => {}
        }
    }
}

// impl Checkable for Src<Declaration> {
//     fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
//         match &self.node {
//             Declaration::Extends(class) => {}
//             Declaration::ClassName(class) => {}
//             Declaration::ValDeclaration(val_declaration) => {
//                 val_declaration.check(project, current_module, report_error)
//             }
//             Declaration::Annotation { name, arguments } => {}
//             Declaration::Enum { name, variants } => {}
//             Declaration::Func {
//                 is_static,
//                 name,
//                 args,
//                 return_type,
//                 body,
//             } => {}
//             Declaration::Class { name, declarations } => {}
//         };
//     }
// }

// impl Checkable for Src<Expression> {
//     fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
//         match &self.node {
//             Expression::Int { value, base } => {}
//             Expression::Float(_) => {}
//             Expression::GDString { value, multiline } => {}
//             Expression::Boolean(_) => {}
//             Expression::Null => {}
//             Expression::Array(_) => {}
//             Expression::Dictionary(_) => {}
//             Expression::UnaryOperation { op, subject } => {}
//             Expression::BinaryOperation { op, left, right } => {}
//             Expression::Identifier(_) => {}
//         };
//     }
// }

// impl Checkable for Src<Statement> {
//     fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
//         match &self.node {
//             Statement::ValDeclaration(val_declaration) => {
//                 val_declaration.check(project, current_module, report_error)
//             }
//             Statement::Assignment {
//                 target,
//                 value,
//                 operator,
//             } => {}
//             Statement::Match => {}
//             Statement::While { condition, body } => {}
//             Statement::If {
//                 conditions,
//                 default_outcome,
//             } => {}
//             Statement::For { name, iteree } => {}
//             Statement::Pass => {}
//             Statement::Break => {}
//             Statement::Return(_) => {}
//         };
//     }
// }

// impl Checkable for Src<Type> {
//     fn check<'a, F: FnMut(GDError)>(
//         &self,
//         project: &GodotProject,
//         current_module: &GDScript,
//         report_error: &F,
//     ) {
//         match &self.node {
//             Type::NullType => {}
//             Type::BoolType => {}
//             Type::IntType => {}
//             Type::FloatType => {}
//             Type::StringType => {}
//             Type::StringNameType => {}
//             Type::Vector2Type => {}
//             Type::Vector2iType => {}
//             Type::Vector3Type => {}
//             Type::Vector3iType => {}
//             Type::Transform2DType => {}
//             Type::PlaneType => {}
//             Type::AABBType => {}
//             Type::BasisType => {}
//             Type::Transform3DType => {}
//             Type::ColorType => {}
//             Type::NodePathType => {}
//             Type::RIDType => {}
//             Type::ObjectType => {}
//             Type::NamedType(name) => {}
//             Type::ExactDictionaryType(entries) => {}
//             Type::AnyType => {}
//             Type::NonNullType(inner) => {
//                 inner.check(project, current_module, report_error);
//             }
//             Type::DictionaryType { key, value } => {
//                 key.check(project, current_module, report_error);
//                 value.check(project, current_module, report_error);
//             }
//             Type::ArrayType(element) => {
//                 element.check(project, current_module, report_error);
//             }
//         };
//     }
// }

// impl Checkable for ValDeclaration {
//     fn check<'a, F: FnMut(GDError)>(
//         &self,
//         project: &GodotProject,
//         current_module: &GDScript,
//         report_error: &F,
//     ) {
//         if let Some(declared_type) = &self.declared_type {
//             if let Some(value) = &self.value {
//                 if !value
//                     .infer_type(project, current_module)
//                     .assignable_to(declared_type)
//                 {
//                     println!("{:?} isn't assignable to {:?}!", value, declared_type);
//                     // report_error(todo!());
//                 }
//             }
//         }
//     }
// }
impl<T> Checkable for Option<T>
where
    T: Checkable,
{
    fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
        if let Some(sel) = self {
            sel.check(ctx, report_error);
        }
    }
}

impl<T> Checkable for Vec<T>
where
    T: Checkable,
{
    fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
        for el in self.iter() {
            el.check(ctx, report_error);
        }
    }
}

impl<T, U> Checkable for (T, U)
where
    T: Checkable,
    U: Checkable,
{
    fn check<'a, F: FnMut(GDError)>(&self, ctx: CheckContext<'a>, report_error: &mut F) {
        self.0.check(ctx, report_error);
        self.1.check(ctx, report_error);
    }
}
