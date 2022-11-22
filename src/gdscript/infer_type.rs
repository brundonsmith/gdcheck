use crate::{
    godot_project::GodotProject,
    utils::{Src, Srcable},
};

use super::resolve::Resolve;
use super::{
    ast::{Expression, GDScript, Type},
    resolve::Binding,
};

impl Src<Expression> {
    pub fn infer_type(&self, project: &GodotProject, current_module: &GDScript) -> Src<Type> {
        match &self.node {
            Expression::Int { value: _, base: _ } => Type::IntType.no_src(),
            Expression::Float(_) => Type::FloatType.no_src(),
            Expression::GDString {
                value: _,
                multiline: _,
            } => Type::StringType.no_src(),
            Expression::Boolean(_) => Type::BoolType.no_src(),
            Expression::Null => Type::NullType.no_src(),
            Expression::Array(entries) => Type::ArrayType(Box::new(
                entries
                    .get(0)
                    .map(|first| first.infer_type(project, current_module))
                    .unwrap_or(Type::AnyType.no_src()),
            ))
            .no_src(),
            Expression::Dictionary(_) => todo!(),
            Expression::UnaryOperation { op, subject } => todo!(),
            Expression::BinaryOperation { op, left, right } => todo!(),
            Expression::Identifier(identifier) => {
                let resolved = current_module.resolve(&Src {
                    src: self.src,
                    node: identifier.clone(),
                });

                match resolved {
                    Some(Binding::Value {
                        declared_type,
                        value,
                    }) => declared_type.unwrap_or_else(|| {
                        value
                            .map(|expr| expr.infer_type(project, current_module))
                            .unwrap_or(Type::AnyType.no_src())
                    }),
                    _ => Type::AnyType.no_src(),
                }
            }
        }
    }
}
