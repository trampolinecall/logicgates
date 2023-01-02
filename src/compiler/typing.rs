use std::collections::HashMap;

use super::{ir, parser::ast, ty};

pub(crate) fn type_<'file>(types: &mut ty::Types, circuits: Vec<ast::CircuitAST<'file>>, type_decls: Vec<ast::NamedTypeDecl>) -> Option<Vec<ir::TypedCircuit<'file>>> {
    let mut type_table = HashMap::new();
    for decl in type_decls {
        let ty = convert_type(types, &type_table, &decl.ty)?;

        let named_type = types.new_named(decl.name.1.into(), ty);

        if type_table.contains_key(decl.name.1) {
            todo!("throw duplicate named type error")
        }
        type_table.insert(decl.name.1.into(), named_type);
    }

    // TODO: report more than just the first error

    Some(
        circuits
            .into_iter()
            .map(|circuit| {
                Some(ir::Circuit {
                    name: circuit.name,
                    input_type: convert_type(types, &type_table, &circuit.input_type)?,
                    output_type: convert_type(types, &type_table, &circuit.output_type)?,
                    gates: circuit.gates,
                    connections: circuit.connections,
                })
            })
            .collect::<Option<Vec<_>>>()?,
    )
}

fn convert_type(types: &mut ty::Types, type_table: &HashMap<String, ty::TypeSym>, ty: &ast::TypeAST) -> Option<ty::TypeSym> {
    match ty {
        ast::TypeAST::Bit(_) => Some(types.intern(ty::Type::Bit)),
        ast::TypeAST::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), convert_type(types, type_table, subty_ast)?))).collect::<Option<Vec<_>>>()?); // TODO: report more than just the first error
            Some(types.intern(ty))
        }
        ast::TypeAST::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type(types, type_table, type_)?;
            Some(types.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        ast::TypeAST::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), convert_type(types, type_table, ty)?))).collect::<Option<Vec<_>>>()?); // TODO: report more than just the first error
                                                                                                                                                                         // TODO: report error if there are any duplicate fields
            Some(types.intern(ty))
        }
        ast::TypeAST::Named(_, name) => {
            let res = type_table.get(*name).copied();
            if res.is_none() {
                todo!("report error for undefined named type")
            }
            res
        }
    }
}
