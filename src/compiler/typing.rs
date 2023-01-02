use super::{ir, parser::ast, ty};

pub(crate) fn type_(types: &mut ty::Types, circuits: Vec<ast::Circuit>) -> Vec<ir::Circuit<ir::Pattern<ty::TypeSym>, ir::Expr<ty::TypeSym>>> {
    circuits.into_iter().map(|circuit| {
        ir::Circuit {
            name: circuit.name,
            input: type_pat(circuit.input),
            lets: circuit.lets,
            output: circuit.output,
        }
    }) .collect()
}

fn from_ast(types: &mut ty::Types, ty: &ast::Type) -> ty::TypeSym {
    // TODO: this should have to happen through name resolution
    match ty {
        ast::Type::Bit(_) => types.intern(ty::Type::Bit),
        ast::Type::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| (ind.to_string(), ty::Type::from_ast(types, subty_ast))).collect());
            types.intern(ty)
        }
        ast::Type::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = ty::Type::from_ast(types, type_);
            types.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect()))
        }
        ast::Type::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| (name.1.to_string(), ty::Type::from_ast(types, ty))).collect());
            // TODO: report error if there are any duplicate fields
            types.intern(ty)
        }
    }
}

// TODO: make an ir pattern type which will be needed when name resolution has to happen
fn type_pat(types: &mut ty::Types, pat: &ast::Pattern) -> ty::TypeSym {
    match &pat.kind {
        ir::PatternKind::Identifier(_, _, ty) => ty::Type::from_ast(types, ty),
        ir::PatternKind::Product(_, pats) => {
            let ty = ty::Type::Product(pats.iter().enumerate().map(|(ind, subpat)| (ind.to_string(), ty::Type::pat_type(types, subpat))).collect());
            types.intern(ty)
        }
    }
}
