use crate::utils::Src;

use super::ast::{Declaration, Expression, GDScript, Statement, Type, ValDeclaration};

pub trait Resolve {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding>;
}

impl Resolve for GDScript {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding> {
        self.declarations
            .iter()
            .filter(|decl| decl.before(identifier))
            .map(|decl| decl.resolve(identifier))
            .next()
            .flatten()
    }
}

impl Resolve for Src<Declaration> {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding> {
        match &self.node {
            Declaration::Extends(class) => None,
            Declaration::ClassName(class) => None,
            Declaration::Annotation {
                name: _,
                arguments: _,
            } => None,
            Declaration::Enum { name, variants } => todo!(),
            Declaration::ValDeclaration(ValDeclaration {
                is_const: _,
                name,
                declared_type,
                value,
            }) => {
                if name.node == identifier.node {
                    Some(Binding::Value {
                        declared_type: declared_type.clone(),
                        value: value.clone(),
                    })
                } else {
                    None
                }
            }
            Declaration::Func {
                is_static,
                name,
                args,
                return_type,
                body,
            } => {
                if name.node == identifier.node {
                    Some(Binding::Func {
                        args: args.clone(),
                        return_type: return_type.clone(),
                    })
                } else {
                    None
                }
            }
            Declaration::Class { name, declarations } => todo!(),
        }
    }
}

impl Resolve for Src<Expression> {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding> {
        if !self.contains(identifier) {
            None
        } else {
            todo!()
        }
    }
}

impl Resolve for Src<Statement> {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding> {
        if !self.contains(identifier) {
            None
        } else {
            todo!()
        }
    }
}

impl Resolve for Src<Type> {
    fn resolve(&self, identifier: &Src<String>) -> Option<Binding> {
        if !self.contains(identifier) {
            None
        } else {
            todo!()
        }
    }
}

pub enum Binding {
    Type(Src<Type>),
    Value {
        declared_type: Option<Src<Type>>,
        value: Option<Src<Expression>>,
    },
    Func {
        args: Vec<(Src<String>, Option<Type>)>,
        return_type: Option<Type>,
    },
}
