use super::ast::*;
impl<TKind> AST<TKind>
where
    TKind: Clone + TryFrom<Any>,
    Any: From<TKind>,
{
    pub fn resolve_symbol(&self, symbol: &str) -> Option<ASTAny> {
        match self.parent().as_ref().map(|p| p.details()) {
            Some(Any::GDScript(GDScript { declarations })) => {
                for decl in declarations {
                    match &decl.downcast() {
                        Declaration::ValueDeclaration(ValueDeclaration {
                            is_const,
                            name,
                            declared_type,
                            value,
                        }) => {
                            if name.downcast().name.as_str() == symbol {
                                return Some(decl.clone().upcast());
                            }
                        }
                        Declaration::FuncDeclaration(FuncDeclaration {
                            is_static,
                            name,
                            args,
                            return_type,
                            body,
                        }) => {
                            if name.downcast().name.as_str() == symbol {
                                return Some(decl.clone().upcast());
                            }
                        }
                        Declaration::EnumDeclaration(_) => todo!(),
                        Declaration::ClassDeclaration(_) => todo!(),

                        Declaration::ExtendsDeclaration(_) => {}
                        Declaration::ClassNameDeclaration(_) => {}
                        Declaration::Annotation(_) => {}
                    }
                }
            }
            Some(Any::Block(Block { statements })) => {
                for stmt in statements {
                    match &stmt.downcast() {
                        Statement::ValueDeclaration(ValueDeclaration {
                            is_const,
                            name,
                            declared_type,
                            value,
                        }) => {
                            if name.downcast().name.as_str() == symbol {
                                return Some(stmt.clone().upcast());
                            }
                        }
                        Statement::ForLoop(_) => todo!(),

                        Statement::AssignmentStatement(_) => {}
                        Statement::MatchStatement(_) => {}
                        Statement::WhileLoop(_) => {}
                        Statement::IfElseStatement(_) => {}
                        Statement::Pass(_) => {}
                        Statement::Break(_) => {}
                        Statement::Return(_) => {}
                    }
                }
            }
            _ => {}
        }

        self.parent()
            .map(|parent| parent.resolve_symbol(symbol))
            .flatten()
    }
}
