use crate::compiler::parser::ast;
use symtern::prelude::*;

pub(crate) struct Types {
    types: symtern::Pool<Type>, // ideally, i would use a interner crate that doesnt use ids to access types but they dont handle cyclic references nicely
}
pub(crate) type TypeSym = symtern::Sym<usize>;
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<(String, TypeSym)>),
}
impl Types {
    pub(crate) fn new() -> Self {
        Self { types: symtern::Pool::new() }
    }

    pub(crate) fn get(&self, sym: TypeSym) -> &Type {
        self.types.resolve(sym).expect("symtern resolution error")
    }

    pub(crate) fn intern(&mut self, ty: Type) -> TypeSym {
        self.types.intern(&ty).expect("symtern interning error")
    }
}
impl Type {
    pub(crate) fn size(&self, types: &Types) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(fields) => fields.iter().map(|(_, tyi)| types.get(*tyi).size(types)).sum(),
        }
    }

    pub(crate) fn from_ast(types: &mut Types, ty: &ast::Type) -> TypeSym {
        // TODO: this should have to happen through name resolution
        match ty {
            ast::Type::Bit(_) => types.intern(Type::Bit),
            ast::Type::Product { obrack: _, types: subtypes, cbrack: _ } => {
                let ty = Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| (ind.to_string(), Type::from_ast(types, subty_ast))).collect());
                types.intern(ty)
            }
            ast::Type::RepProduct { obrack: _, num, cbrack: _, type_ } => {
                let ty = Type::from_ast(types, type_);
                types.intern(Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect()))
            }
            ast::Type::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
                let ty = Type::Product(subtypes.iter().map(|(name, ty)| (name.1.to_string(), Type::from_ast(types, ty))).collect());
                // TODO: report error if there are any duplicate fields
                types.intern(ty)
            }
        }
    }

    // TODO: make an ir pattern type which will be needed when name resolution has to happen
    pub(crate) fn pat_type(types: &mut Types, pat: &ast::Pattern) -> TypeSym {
        match pat {
            ast::Pattern::Identifier(_, ty) => Type::from_ast(types, ty),
            ast::Pattern::Product(_, pats) => {
                let ty = Type::Product(pats.iter().enumerate().map(|(ind, subpat)| (ind.to_string(), Type::pat_type(types, subpat))).collect());
                types.intern(ty)
            }
        }
    }

    // TODO: there is probably a better solution to this
    pub(crate) fn fmt(&self, types: &Types) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        match self {
            Type::Bit => write!(s, "`").unwrap(),
            Type::Product(items) => {
                write!(s, "[named ").unwrap();
                if let Some(((first_name, first), more)) = items.split_first() {
                    write!(s, "{}; {}", first_name, types.get(*first).fmt(types)).unwrap();
                    for (more_name, more) in more {
                        write!(s, ", {}; {}", more_name, types.get(*more).fmt(types)).unwrap();
                    }
                }
                write!(s, "]").unwrap();
            }
        };

        s
    }
}
