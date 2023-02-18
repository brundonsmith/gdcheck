use crate::godot_project::GodotProject;

use super::ast::*;
use super::check::CheckContext;
use super::gd_type::Type;

#[derive(Clone, Copy, Debug)]
pub struct InferTypeContext<'a> {
    pub godot_project: &'a GodotProject,
}

impl<'a> From<CheckContext<'a>> for InferTypeContext<'a> {
    fn from(ctx: CheckContext<'a>) -> Self {
        Self {
            godot_project: ctx.godot_project,
        }
    }
}

impl AST<Expression> {
    pub fn infer_type<'a>(&self, ctx: InferTypeContext<'a>) -> Type {
        match self.downcast() {
            Expression::NullLiteral(_) => Type::Null,
            Expression::BooleanLiteral(BooleanLiteral { value }) => Type::Boolean(Some(value)),
            Expression::IntLiteral(IntLiteral { value_raw }) => {
                Type::Int(Some(value_raw.as_str().parse().unwrap()))
            }
            Expression::FloatLiteral(FloatLiteral { value_raw }) => {
                Type::Float(Some(value_raw.as_str().parse().unwrap()))
            }
            Expression::StringLiteral(StringLiteral { value, multiline }) => {
                Type::String(Some(value))
            }
            Expression::ArrayLiteral(ArrayLiteral { members }) => Type::ExactArray {
                members: members.iter().map(|m| m.infer_type(ctx)).collect(),
            },
            Expression::DictionaryLiteral(DictionaryLiteral { entries }) => Type::ExactDictionary {
                entries: entries
                    .iter()
                    .map(|(key, value)| (key.infer_type(ctx), value.infer_type(ctx)))
                    .collect(),
            },
            Expression::UnaryOperation(UnaryOperation { op, subject }) => match op.downcast() {
                UnaryOperator::BitwiseNot => todo!(),
                UnaryOperator::Negative => todo!(),
                UnaryOperator::Not => todo!(),
            },
            Expression::BinaryOperation(BinaryOperation { op, left, right }) => {
                let left_type = left.infer_type(ctx);
                let right_type = right.infer_type(ctx);

                match op.downcast() {
                    BinaryOperator::Is => todo!(),
                    BinaryOperator::DoubleStar => todo!(),
                    BinaryOperator::Star => todo!(),
                    BinaryOperator::Slash => todo!(),
                    BinaryOperator::Percent => todo!(),
                    BinaryOperator::Plus => todo!(),
                    BinaryOperator::Minus => todo!(),
                    BinaryOperator::ShiftLeft => todo!(),
                    BinaryOperator::ShiftRight => todo!(),
                    BinaryOperator::BitwiseAnd => todo!(),
                    BinaryOperator::BitwiseXor => todo!(),
                    BinaryOperator::BitwiseOr => todo!(),
                    BinaryOperator::Less => Type::Boolean(None),
                    BinaryOperator::Greater => Type::Boolean(None),
                    BinaryOperator::Equals => Type::Boolean(None),
                    BinaryOperator::NotEquals => Type::Boolean(None),
                    BinaryOperator::GreaterEqual => Type::Boolean(None),
                    BinaryOperator::LessEqual => Type::Boolean(None),
                    BinaryOperator::In => Type::Boolean(None),
                    BinaryOperator::And => todo!(),
                    BinaryOperator::Or => todo!(),
                    BinaryOperator::As => todo!(),
                }
            }
            Expression::LocalIdentifier(LocalIdentifier { name }) => {
                let resolved = self.resolve_symbol(name.as_str());

                match resolved.as_ref().map(|ast| ast.details()) {
                    Some(Any::ValueDeclaration(ValueDeclaration {
                        is_const,
                        name,
                        declared_type,
                        value,
                    })) => declared_type
                        .as_ref()
                        .map(|t| t.resolve_type(ctx.into()))
                        .unwrap_or_else(|| {
                            value
                                .as_ref()
                                .map(|v| v.infer_type(ctx))
                                .unwrap_or(Type::Poisoned)
                        }),
                    _ => Type::Poisoned,
                }
            }
        }
    }
}

// impl Src<Expression> {
//     pub fn infer_type(&self, project: &GodotProject, current_module: &GDScript) -> Src<Type> {
//         match &self.node {
//             Expression::Int { value: _, base: _ } => Type::IntType.no_src(),
//             Expression::Float(_) => Type::FloatType.no_src(),
//             Expression::GDString {
//                 value: _,
//                 multiline: _,
//             } => Type::StringType.no_src(),
//             Expression::Boolean(_) => Type::BoolType.no_src(),
//             Expression::Null => Type::NullType.no_src(),
//             Expression::Array(entries) => Type::ArrayType(Box::new(
//                 entries
//                     .get(0)
//                     .map(|first| first.infer_type(project, current_module))
//                     .unwrap_or(Type::AnyType.no_src()),
//             ))
//             .no_src(),
//             Expression::Dictionary(_) => todo!(),
//             Expression::UnaryOperation { op, subject } => todo!(),
//             Expression::BinaryOperation { op, left, right } => todo!(),
//             Expression::Identifier(identifier) => {
//                 let resolved = current_module.resolve(&Src {
//                     src: self.src,
//                     node: identifier.clone(),
//                 });

//                 match resolved {
//                     Some(Binding::Value {
//                         declared_type,
//                         value,
//                     }) => declared_type.unwrap_or_else(|| {
//                         value
//                             .map(|expr| expr.infer_type(project, current_module))
//                             .unwrap_or(Type::AnyType.no_src())
//                     }),
//                     _ => Type::AnyType.no_src(),
//                 }
//             }
//         }
//     }
// }
