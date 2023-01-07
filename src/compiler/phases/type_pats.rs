use std::collections::HashMap;

use crate::{
    compiler::{
        data::{circuit1, nominal_type, ty},
        phases::resolve_type_expr,
    },
    utils::arena,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::PatTypedCircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, circuit1::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn type_(resolve_type_expr::IR { circuits, circuit_table, mut type_context }: resolve_type_expr::IR) -> IR {
    IR {
        circuits: circuits.transform_infallible(|circuit| match circuit {
            circuit1::TypeResolvedCircuitOrIntrinsic::Circuit(circuit) => circuit1::PatTypedCircuitOrIntrinsic::Circuit(circuit1::PatTypedCircuit {
                name: circuit.name,
                input: type_pat(&mut type_context, circuit.input),
                output_type: circuit.output_type,
                lets: circuit.lets.into_iter().map(|pat| type_pat_in_let(&mut type_context, pat)).collect(),
                output: circuit.output,
            }),
            circuit1::TypeResolvedCircuitOrIntrinsic::Nand => circuit1::PatTypedCircuitOrIntrinsic::Nand,
            circuit1::TypeResolvedCircuitOrIntrinsic::Const(value) => circuit1::PatTypedCircuitOrIntrinsic::Const(value),
        }),
        circuit_table,
        type_context,
    }
}

fn type_pat_in_let<'file>(type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>, let_: circuit1::TypeResolvedLet<'file>) -> circuit1::PatTypedLet<'file> {
    circuit1::PatTypedLet { pat: type_pat(type_context, let_.pat), val: let_.val }
}

fn type_pat<'file>(type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>, pat: circuit1::TypeResolvedPattern<'file>) -> circuit1::PatTypedPattern<'file> {
    let (kind, type_info) = match pat.kind {
        circuit1::TypeResolvedPatternKind::Identifier(name_sp, name, ty) => (circuit1::PatTypedPatternKind::Identifier(name_sp, name, ty), ty.1),
        circuit1::TypeResolvedPatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(type_context, subpat)).collect();

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| (ind.to_string(), subpat.type_info)).collect());
            (circuit1::PatTypedPatternKind::Product(sp, typed_pats), type_context.intern(ty))
        }
    };

    circuit1::PatTypedPattern { kind, type_info, span: pat.span }
}
