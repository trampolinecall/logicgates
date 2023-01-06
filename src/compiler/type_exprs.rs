use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::{circuit1, named_type, ty, type_expr};
use super::{arena, ir, make_name_tables, resolve_type_expr, type_pats};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::circuit1::TypedCircuitOrIntrinsic<'file>, make_name_tables::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, make_name_tables::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}
pub(crate) fn type_<'file>(type_pats::IR { circuits, circuit_table, mut type_context, type_table }: type_pats::IR) -> Option<IR> {
    let circuit_output_types = circuit_table
        .iter()
        .map(|(name, circuit)| {
            let circuit = circuits.get(*circuit);
            (name as &str, (circuit.output_type(&mut type_context)))
        })
        .collect();

    let circuits = circuits.transform(|circuit| match circuit {
        ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
            let mut local_table = todo!();

            // not ideal because expressions still represent the ast and are therefore in a tree so there will never be loops
            // but moving them out of the arena would make circuit1 have to be split into two datatypes:
            // one with expressions in a tree and one with expressions in an arena, because converting to circuit2 needs exprs in an arena
            let expressions = circuit.expressions.transform_dependant(|expr, get_other_expr_type| type_expr(&mut type_context, &circuit_output_types, &local_table, get_other_expr_type, expr));

            Some(ir::circuit1::CircuitOrIntrinsic::Circuit(ir::circuit1::Circuit {
                name: circuit.name,
                input: circuit.input,
                expressions: match expressions {
                    Ok(r) => r,
                    Err((loop_errors, typing_errors)) => {
                        assert!(loop_errors.is_empty(), "expressions are in a tree, which cannot have loops");

                        todo!("report typing errors in typing expressions")
                    }
                },
                output_type: circuit.output_type,
                lets: circuit.lets,
                output: (circuit.output),
            }))
        }
        ir::circuit1::CircuitOrIntrinsic::Nand => Some(ir::circuit1::CircuitOrIntrinsic::Nand),
    })?;

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, (old_id))).collect();

    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_output_types: &HashMap<&str, ty::TypeSym>,
    local_types: &HashMap<&str, ty::TypeSym>,
    get_other_expr: arena::DependancyGetter<circuit1::expr::TypedExpr, circuit1::expr::UntypedExpr, Vec<()>, circuit1::expr::ExprId>,
    expr: &circuit1::expr::UntypedExpr<'file>,
) -> arena::SingleTransformResult<circuit1::expr::TypedExpr<'file>, circuit1::expr::ExprId, Vec<()>> {
    let (kind, type_info) = match &expr.kind {
        circuit1::expr::ExprKind::Ref(sp, name) => {
            let local_type = if let Some(ty) = local_types.get(name) { *ty } else { todo!("report error for undefined local usage") };
            (circuit1::expr::ExprKind::Ref(*sp, name), local_type)
        }
        circuit1::expr::ExprKind::Call(name, inline, arg) => {
            let ty = if let Some(ty) = circuit_output_types.get(&name.1) { *ty } else { todo!("report error for undefined circuit usage") }; // this also does circuit name resolution
            (circuit1::expr::ExprKind::Call(*name, *inline, *arg), ty)
        }
        circuit1::expr::ExprKind::Const(sp, value) => (circuit1::expr::ExprKind::Const(*sp, *value), type_context.intern(ty::Type::Bit)),
        circuit1::expr::ExprKind::Get(base, field) => {
            let ty = try_transform_result!(get_other_expr.get(*base)).type_info;
            let field_ty = type_context.get(ty).field_type(type_context, field.1);
            if let Some(field_ty) = field_ty {
                (circuit1::expr::ExprKind::Get(*base, *field), field_ty)
            } else {
                return arena::SingleTransformResult::Err(todo!("report error for field doesnt exist on type"));
            }
        }
        circuit1::expr::ExprKind::Multiple { obrack, exprs, cbrack } => {
            let ty = type_context.intern(ty::Type::Product(try_transform_result!(exprs
                .iter()
                .enumerate()
                .map(|(field_index, subexpr)| arena::SingleTransformResult::Ok((field_index.to_string(), try_transform_result!(get_other_expr.get(*subexpr)).type_info)))
                .collect_all::<Vec<_>>())));

            (circuit1::expr::ExprKind::Multiple { obrack: *obrack, exprs: exprs.clone(), cbrack: *cbrack }, ty) // TODO: no clone
        }
    };

    arena::SingleTransformResult::Ok(circuit1::expr::Expr { kind, type_info })
}
