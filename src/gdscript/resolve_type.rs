use std::rc::Rc;

use crate::godot_project::GodotProject;

use super::{ast::*, check::CheckContext, gd_type::Type, infer_type::InferTypeContext};

#[derive(Clone, Copy, Debug)]
pub struct ResolveContext<'a> {
    pub godot_project: &'a GodotProject,
}

impl<'a> From<CheckContext<'a>> for ResolveContext<'a> {
    fn from(ctx: CheckContext<'a>) -> Self {
        Self {
            godot_project: ctx.godot_project,
        }
    }
}

impl<'a> From<InferTypeContext<'a>> for ResolveContext<'a> {
    fn from(ctx: InferTypeContext<'a>) -> Self {
        Self {
            godot_project: ctx.godot_project,
        }
    }
}

impl AST<TypeExpression> {
    pub fn resolve_type<'a>(&self, ctx: ResolveContext<'a>) -> Type {
        match self.downcast() {
            TypeExpression::NullType(_) => Type::Null,
            TypeExpression::BooleanType(_) => Type::Boolean(None),
            TypeExpression::IntType(_) => Type::Int(None),
            TypeExpression::FloatType(_) => Type::Float(None),
            TypeExpression::StringType(_) => Type::String(None),
            TypeExpression::StringNameType(_) => todo!(),
            TypeExpression::Vector2Type(_) => Type::Vector2,
            TypeExpression::Vector2iType(_) => Type::Vector2i,
            TypeExpression::Vector3Type(_) => Type::Vector3,
            TypeExpression::Vector3iType(_) => Type::Vector3i,
            TypeExpression::Transform2DType(_) => Type::Transform2D,
            TypeExpression::PlaneType(_) => Type::Plane,
            TypeExpression::AABBType(_) => Type::AABB,
            TypeExpression::BasisType(_) => Type::Basis,
            TypeExpression::Transform3DType(_) => Type::Transform3D,
            TypeExpression::ColorType(_) => Type::Color,
            TypeExpression::NodePathType(_) => Type::NodePath,
            TypeExpression::RIDType(_) => Type::RID,
            TypeExpression::ObjectType(_) => Type::Object,
            TypeExpression::NamedType(NamedType { name }) => todo!(),
            TypeExpression::NonNullType(NonNullType { inner }) => Type::NonNull {
                inner: Rc::new(inner.resolve_type(ctx)),
            },
            TypeExpression::ArrayType(ArrayType { element }) => Type::Array {
                element: Rc::new(element.resolve_type(ctx)),
            },
            TypeExpression::DictionaryType(DictionaryType { key, value }) => Type::Dictionary {
                key: Rc::new(key.resolve_type(ctx)),
                value: Rc::new(value.resolve_type(ctx)),
            },
            TypeExpression::ExactDictionaryType(ExactDictionaryType { entries }) => {
                Type::ExactDictionary {
                    entries: entries
                        .iter()
                        .map(|(key, value)| (key.resolve_type(ctx), value.resolve_type(ctx)))
                        .collect(),
                }
            }
        }
    }
}
