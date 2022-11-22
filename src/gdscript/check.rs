use crate::{
    godot_project::GodotProject,
    utils::{errors::GDError, Src, Srcable},
};

use super::ast::{Declaration, Expression, GDScript, Statement, Type, ValDeclaration};

pub trait Check {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    );
}

impl Check for GDScript {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        for decl in &self.declarations {
            decl.check(project, current_module, report_error);
        }
    }
}

impl Check for Src<Declaration> {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        match &self.node {
            Declaration::Extends(class) => {}
            Declaration::ClassName(class) => {}
            Declaration::ValDeclaration(val_declaration) => {
                val_declaration.check(project, current_module, report_error)
            }
            Declaration::Annotation { name, arguments } => {}
            Declaration::Enum { name, variants } => {}
            Declaration::Func {
                is_static,
                name,
                args,
                return_type,
                body,
            } => {}
            Declaration::Class { name, declarations } => {}
        };
    }
}

impl Check for Src<Expression> {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        match &self.node {
            Expression::Int { value, base } => {}
            Expression::Float(_) => {}
            Expression::GDString { value, multiline } => {}
            Expression::Boolean(_) => {}
            Expression::Null => {}
            Expression::Array(_) => {}
            Expression::Dictionary(_) => {}
            Expression::UnaryOperation { op, subject } => {}
            Expression::BinaryOperation { op, left, right } => {}
            Expression::Identifier(_) => {}
        };
    }
}

impl Check for Src<Statement> {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        match &self.node {
            Statement::ValDeclaration(val_declaration) => {
                val_declaration.check(project, current_module, report_error)
            }
            Statement::Assignment {
                target,
                value,
                operator,
            } => {}
            Statement::Match => {}
            Statement::While { condition, body } => {}
            Statement::If {
                conditions,
                default_outcome,
            } => {}
            Statement::For { name, iteree } => {}
            Statement::Pass => {}
            Statement::Break => {}
            Statement::Return(_) => {}
        };
    }
}

impl Check for Src<Type> {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        match &self.node {
            Type::NullType => {}
            Type::BoolType => {}
            Type::IntType => {}
            Type::FloatType => {}
            Type::StringType => {}
            Type::StringNameType => {}
            Type::Vector2Type => {}
            Type::Vector2iType => {}
            Type::Vector3Type => {}
            Type::Vector3iType => {}
            Type::Transform2DType => {}
            Type::PlaneType => {}
            Type::AABBType => {}
            Type::BasisType => {}
            Type::Transform3DType => {}
            Type::ColorType => {}
            Type::NodePathType => {}
            Type::RIDType => {}
            Type::ObjectType => {}
            Type::NamedType(name) => {}
            Type::ExactDictionaryType(entries) => {}
            Type::AnyType => {}
            Type::NonNullType(inner) => {
                inner.check(project, current_module, report_error);
            }
            Type::DictionaryType { key, value } => {
                key.check(project, current_module, report_error);
                value.check(project, current_module, report_error);
            }
            Type::ArrayType(element) => {
                element.check(project, current_module, report_error);
            }
        };
    }
}

impl Check for ValDeclaration {
    fn check<F: Fn(GDError)>(
        &self,
        project: &GodotProject,
        current_module: &GDScript,
        report_error: &F,
    ) {
        if let Some(declared_type) = &self.declared_type {
            if let Some(value) = &self.value {
                if !value
                    .infer_type(project, current_module)
                    .assignable_to(declared_type)
                {
                    println!("{:?} isn't assignable to {:?}!", value, declared_type);
                    // report_error(todo!());
                }
            }
        }
    }
}
