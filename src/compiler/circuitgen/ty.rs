use crate::compiler::parser::ast;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type {
    // TODO: type interner
    Bit,
    Product(Vec<Type>), // TODO: named product types
}
impl<'file> Type {
    pub(crate) fn size(&self) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(types) => types.iter().map(Type::size).sum(),
        }
    }
}
impl Type {
    pub(crate) fn from_ast<'file>(ty: &ast::Type<'file>) -> Type {
        // TODO: this should have to happen through name resolution
        match ty {
            ast::Type::Bit(_) => Type::Bit,
            ast::Type::Product { obrack, types, cbrack } => Type::Product(types.iter().map(Type::from_ast).collect()),
            ast::Type::RepProduct { obrack, num, cbrack, type_ } => {
                let ty = Type::from_ast(type_);
                Type::Product((0..num.1).map(|_| ty.clone()).collect())
            }
        }
    }

    // TODO: make an ir pattern type which will be needed when name resolution has to happen
    pub(crate) fn pat_type(pat: &ast::Pattern) -> Type {
        match pat {
            ast::Pattern::Identifier(_, ty) => Type::from_ast(ty),
            ast::Pattern::Product(_, pats) => Type::Product(pats.iter().map(Type::pat_type).collect()),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bit => write!(f, "`"),
            Type::Product(items) => {
                write!(f, "[")?;
                if let Some((first, more)) = items.split_first() {
                    write!(f, "{first}")?;
                    for more in more {
                        write!(f, ", {more}")?;
                    }
                }
                write!(f, "]")?;

                Ok(())
            }
        }
    }
}
