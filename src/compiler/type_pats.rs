use std::collections::HashMap;

use super::{
    arena,
    ir::{circuit1, named_type, ty},
    make_name_tables, resolve_type_expr,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::PatTypedCircuitOrIntrinsic<'file>, make_name_tables::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, make_name_tables::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
}

pub(crate) fn type_(resolve_type_expr::IR { circuits, circuit_table, mut type_context }: resolve_type_expr::IR) -> IR {
    IR {
        circuits: circuits.transform_infallible(|circuit| match circuit {
            circuit1::CircuitOrIntrinsic::Circuit(circuit) => circuit1::CircuitOrIntrinsic::Circuit(circuit1::Circuit {
                name: circuit.name,
                input: type_pat(&mut type_context, circuit.input),
                expressions: circuit.expressions,
                output_type: circuit.output_type,
                lets: circuit.lets.into_iter().map(|pat| type_pat_in_let(&mut type_context, pat)).collect(),
                output: circuit.output,
            }),
            circuit1::CircuitOrIntrinsic::Nand => circuit1::CircuitOrIntrinsic::Nand,
            circuit1::CircuitOrIntrinsic::Const(value) => circuit1::CircuitOrIntrinsic::Const(value),
        }),
        circuit_table,
        type_context,
    }
}

fn type_pat_in_let<'file>(type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>, let_: circuit1::TypeResolvedLet<'file>) -> circuit1::PatTypedLet<'file> {
    circuit1::Let { pat: type_pat(type_context, let_.pat), val: let_.val }
}

fn type_pat<'file>(type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>, pat: circuit1::TypeResolvedPattern<'file>) -> circuit1::PatTypedPattern<'file> {
    let (kind, type_info) = match pat.kind {
        circuit1::PatternKind::Identifier(name_sp, name, ty) => (circuit1::PatternKind::Identifier(name_sp, name, ty), ty.1),
        circuit1::PatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(type_context, subpat)).collect();

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| (ind.to_string(), subpat.type_info)).collect());
            (circuit1::PatternKind::Product(sp, typed_pats), type_context.intern(ty))
        }
    };

    circuit1::Pattern { kind, type_info }
}
