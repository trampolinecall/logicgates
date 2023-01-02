use super::{ir, parser::ast, ty};

pub(crate) fn type_<'file>(types: &mut ty::Types, circuits: Vec<ast::CircuitAST<'file>>) -> Vec<ir::TypedCircuit<'file>> {
    circuits
        .into_iter()
        .map(|circuit| ir::Circuit { name: circuit.name, input: type_pat(types, circuit.input), lets: circuit.lets.into_iter().map(|let_| type_let(types, let_)).collect(), output: circuit.output })
        .collect()
}

fn type_let<'file>(types: &mut ty::Types, let_: ast::LetAST<'file>) -> ir::TypedLet<'file> {
    ir::Let { pat: type_pat(types, let_.pat), val: let_.val }
}

fn type_pat<'file>(types: &mut ty::Types, pat: ast::PatternAST<'file>) -> ir::TypedPattern<'file> {
    let (kind, type_info) = match pat.kind {
        ir::PatternKind::Identifier(name_sp, name, ty) => {
            let type_info = convert_type_ast(types, &ty);
            (ir::PatternKind::Identifier(name_sp, name, ty), type_info)
        }
        ir::PatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(types, subpat)).collect();

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| (ind.to_string(), subpat.type_info)).collect());
            (ir::PatternKind::Product(sp, typed_pats), types.intern(ty))
        }
    };

    ir::Pattern { kind, type_info }
}

fn convert_type_ast(types: &mut ty::Types, ty: &ast::TypeAST) -> ty::TypeSym {
    match ty {
        ast::TypeAST::Bit(_) => types.intern(ty::Type::Bit),
        ast::TypeAST::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| (ind.to_string(), convert_type_ast(types, subty_ast))).collect());
            types.intern(ty)
        }
        ast::TypeAST::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type_ast(types, type_);
            types.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect()))
        }
        ast::TypeAST::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| (name.1.to_string(), convert_type_ast(types, ty))).collect());
            // TODO: report error if there are any duplicate fields
            types.intern(ty)
        }
    }
}
