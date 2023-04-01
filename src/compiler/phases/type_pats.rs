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
    IR {
        circuits: circuits.transform_infallible(|circuit| match circuit {
            ast::TypeResolvedCircuitOrIntrinsic::Circuit(circuit) => ast::PatTypedCircuitOrIntrinsic::Circuit(ast::PatTypedCircuit {
                name: circuit.name,
                input: type_pat(&mut type_context, circuit.input),
                output_type: circuit.output_type,
                lets: circuit.lets.into_iter().map(|let_| ast::PatTypedLet { pat: type_pat(&mut type_context, let_.pat), val: let_.val }).collect(),
                output: circuit.output,
            }),
            ast::TypeResolvedCircuitOrIntrinsic::Nand => ast::PatTypedCircuitOrIntrinsic::Nand,
            ast::TypeResolvedCircuitOrIntrinsic::Const(value) => ast::PatTypedCircuitOrIntrinsic::Const(value),
        }),
        circuit_table,
        type_context,
    }
}

fn type_pat<'file>(type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>, pat: ast::TypeResolvedPattern<'file>) -> ast::PatTypedPattern<'file> {
    let (kind, type_info) = match pat.kind {
        ast::TypeResolvedPatternKind::Identifier(name, ty) => (ast::PatTypedPatternKind::Identifier(name, ty), ty.1),
        ast::TypeResolvedPatternKind::Product(pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|(field_name, subpat)| (field_name, type_pat(type_context, subpat))).collect();

            let ty = ty::Type::Product(typed_pats.iter().map(|(field_name, subpat)| (field_name.clone(), subpat.type_info)).collect());
            (ast::PatTypedPatternKind::Product(typed_pats), type_context.intern(ty))
        }
    };

    ast::PatTypedPattern { kind, type_info, span: pat.span }
}
