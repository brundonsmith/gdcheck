use std::rc::Rc;

use crate::utils::slice::Slice;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Null,
    Boolean(Option<bool>),
    Int(Option<i64>),
    Float(Option<f64>),
    String(Option<Slice>),
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
    NonNull { inner: Rc<Type> },
    Array { element: Rc<Type> },
    Dictionary { key: Rc<Type>, value: Rc<Type> },
    ExactDictionary { entries: Vec<(Type, Type)> },
    ExactArray { members: Vec<Type> },

    Unknown,
    Poisoned,
    Any,
}

impl Type {
    pub fn subsumes(&self, other: &Type) -> bool {
        self == other
    }
}
