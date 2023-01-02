use super::{ir, parser::ast, ty};

pub(crate) fn type_<'file>(types: &mut ty::Types, circuits: Vec<ast::CircuitAST<'file>>) -> Vec<ir::TypedCircuit<'file>> {
    circuits
        .into_iter()
        .map(|circuit| ir::Circuit {
            name: circuit.name,
            input_type: convert_type(types, &circuit.input_type),
            output_type: convert_type(types, &circuit.output_type),
            gates: circuit.gates.into_iter().map(|gate| type_gate(types, gate)).collect(),
            connections: circuit.connections,
        })
        .collect()
}

fn type_gate<'file>(types: &mut ty::Types, gate: ir::GateInstance<'file>) -> ir::GateInstance<'file> {
    gate
}

fn convert_type(types: &mut ty::Types, ty: &ast::TypeAST) -> ty::TypeSym {
    match ty {
        ast::TypeAST::Bit(_) => types.intern(ty::Type::Bit),
        ast::TypeAST::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| (ind.to_string(), convert_type(types, subty_ast))).collect());
            types.intern(ty)
        }
        ast::TypeAST::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type(types, type_);
            types.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect()))
        }
        ast::TypeAST::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| (name.1.to_string(), convert_type(types, ty))).collect());
            // TODO: report error if there are any duplicate fields
            types.intern(ty)
        }
    }
}
