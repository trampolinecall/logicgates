use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::{circuit1, named_type, ty};
use super::{arena, ir, make_name_tables, type_pats};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::circuit1::TypedCircuitOrIntrinsic<'file>, make_name_tables::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}
pub(crate) fn type_<'file>(type_pats::IR { circuits, circuit_table, mut type_context, type_table }: type_pats::IR) -> Option<IR> {
    let circuit_table = circuit_table
        .into_iter()
        .map(|(name, circuit_id)| {
            let circuit = circuits.get(circuit_id);
            (name, (circuit.input_type(&mut type_context), circuit.output_type(&mut type_context), circuit_id))
        })
        .collect();

    let circuits = circuits.transform(|circuit| match circuit {
        ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
            let mut local_table = HashMap::new();

            put_pat_type(&mut local_table, &circuit.input);
            for let_ in &circuit.lets {
                put_pat_type(&mut local_table, &let_.pat);
            }

            // not ideal because expressions still represent the ast and are therefore in a tree so there will never be loops
            // but moving them out of the arena would make circuit1 have to be split into two datatypes:
            // one with expressions in a tree and one with expressions in an arena, because converting to circuit2 needs exprs in an arena
            let expressions = circuit.expressions.annotate_dependant(
                |expr, get_other_expr_type| type_expr(&mut type_context, &circuit_table, &local_table, get_other_expr_type, expr),
                |circuit1::expr::Expr { kind, type_info: () }, expr_ty| circuit1::expr::Expr { kind, type_info: expr_ty },
            );

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
        circuit1::CircuitOrIntrinsic::Const(value) => Some(ir::circuit1::CircuitOrIntrinsic::Const(value)),
    })?;

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, old_id)).collect();

    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn put_pat_type<'file>(local_table: &mut HashMap<&'file str, ty::TypeSym>, pat: &circuit1::PatTypedPattern<'file>) {
    match &pat.kind {
        circuit1::PatternKind::Identifier(_, name, ty) => {
            local_table.insert(name, ty.1); // TODO: report error for duplicate locals
        }
        circuit1::PatternKind::Product(_, subpats) => {
            for subpat in subpats {
                put_pat_type(local_table, &subpat);
            }
        }
    }
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_table: &HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,
    local_types: &HashMap<&str, ty::TypeSym>,
    get_other_expr_type: arena::DependancyGetter<ty::TypeSym, circuit1::expr::UntypedExpr, Vec<()>, circuit1::expr::ExprId>,
    expr: &circuit1::expr::UntypedExpr<'file>,
) -> arena::SingleTransformResult<ty::TypeSym, circuit1::expr::ExprId, Vec<()>> {
    let type_info = match &expr.kind {
        circuit1::expr::ExprKind::Ref(_, name) => {
            let local_type = if let Some(ty) = local_types.get(name) { *ty } else { todo!("report error for undefined local usage") };
            local_type
        }
        circuit1::expr::ExprKind::Call(name, _, _) => {
            // this also does circuit name resolution
            if let Some((_, ty, _)) = circuit_table.get(name.1) {
                *ty
            } else {
                todo!("report error for undefined circuit usage")
            }
        }
        circuit1::expr::ExprKind::Const(_, _) => type_context.intern(ty::Type::Bit),
        circuit1::expr::ExprKind::Get(base, field) => {
            let base_ty = try_annotation_result!(get_other_expr_type.get(*base));
            let field_ty = type_context.get(*base_ty.1).field_type(type_context, field.1);
            if let Some(field_ty) = field_ty {
                field_ty
            } else {
                return arena::SingleTransformResult::Err(todo!("report error for field doesnt exist on type"));
            }
        }
        circuit1::expr::ExprKind::Multiple { obrack: _, exprs, cbrack: _ } => {
            let ty = type_context.intern(ty::Type::Product(try_annotation_result!(exprs
                .iter()
                .enumerate()
                .map(|(field_index, subexpr)| arena::SingleTransformResult::Ok((field_index.to_string(), *try_annotation_result!(get_other_expr_type.get(*subexpr)).1)))
                .collect_all::<Vec<_>>())));

            ty
        }
    };

    arena::SingleTransformResult::Ok(type_info)
}
