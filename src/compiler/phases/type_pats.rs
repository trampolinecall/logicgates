use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, nominal_type, ty},
        phases::resolve_type_expr,
    },
    utils::arena,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::PatTypedCircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, ast::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn type_(resolve_type_expr::IR { circuits, circuit_table, mut type_context }: resolve_type_expr::IR) -> IR {
    // TODO: remove this
    IR {
        circuits: circuits.transform_infallible(|circuit| match circuit {
            ast::TypeResolvedCircuitOrIntrinsic::Circuit(circuit) => ast::PatTypedCircuitOrIntrinsic::Circuit(ast::PatTypedCircuit {
                name: circuit.name,
                input_type: circuit.input_type,
                output_type: circuit.output_type,
                lets: circuit.lets,
                connects: circuit.connects,
            }),
            ast::TypeResolvedCircuitOrIntrinsic::Nand => ast::PatTypedCircuitOrIntrinsic::Nand,
            ast::TypeResolvedCircuitOrIntrinsic::Const(value) => ast::PatTypedCircuitOrIntrinsic::Const(value),
        }),
        circuit_table,
        type_context,
    }
}

