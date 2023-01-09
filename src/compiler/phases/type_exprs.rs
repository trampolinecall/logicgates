use crate::{
    compiler::{
        data::{circuit1, nominal_type, ty},
        error::{CompileError, Report, Span},
        phases::type_pats,
    },
    utils::{arena, collect_all::CollectAll},
};

use std::collections::HashMap;

struct NoField<'file> {
    // TODO: list names of fields that do exist
    ty: ty::TypeSym,
    field_name_sp: Span<'file>,
    field_name: &'file str,
}
struct NoSuchLocal<'file>(Span<'file>, &'file str);
struct NoSuchCircuit<'file>(Span<'file>, &'file str);

impl<'file> From<(&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, NoField<'file>)> for CompileError<'file> {
    fn from((types, NoField { ty, field_name_sp, field_name }): (&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, NoField<'file>)) -> Self {
        CompileError::new(field_name_sp, format!("no field called '{}' on type '{}'", field_name, types.get(ty).fmt(types)))
    }
}
impl<'file> From<NoSuchLocal<'file>> for CompileError<'file> {
    fn from(NoSuchLocal(name_sp, name): NoSuchLocal<'file>) -> Self {
        CompileError::new(name_sp, format!("no local called '{}'", name))
    }
}
impl<'file> From<NoSuchCircuit<'file>> for CompileError<'file> {
    fn from(NoSuchCircuit(name_sp, name): NoSuchCircuit<'file>) -> Self {
        CompileError::new(name_sp, format!("no circuit called '{}'", name))
    }
}

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::TypedCircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, circuit1::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}
pub(crate) fn type_(type_pats::IR { circuits, circuit_table, mut type_context }: type_pats::IR) -> Option<IR> {
    let circuit_table = circuit_table
        .into_iter()
        .map(|(name, circuit_id)| {
            let circuit = circuits.get(circuit_id);
            (name, (circuit.input_type(&mut type_context), circuit.output_type(&mut type_context), circuit_id))
        })
        .collect();

    let circuits = circuits.transform(|circuit| match circuit {
        circuit1::PatTypedCircuitOrIntrinsic::Circuit(circuit) => {
            let mut local_table = HashMap::new();

            put_pat_type(&mut local_table, &circuit.input);
            for let_ in &circuit.lets {
                put_pat_type(&mut local_table, &let_.pat);
            }

            Some(circuit1::TypedCircuitOrIntrinsic::Circuit(circuit1::TypedCircuit {
                name: circuit.name,
                input: circuit.input,
                output_type: circuit.output_type,
                lets: circuit
                    .lets
                    .into_iter()
                    .map(|circuit1::PatTypedLet { pat, val }| Some(circuit1::TypedLet { pat, val: type_expr(&mut type_context, &circuit_table, &local_table, val)? }))
                    .collect_all()?,
                output: type_expr(&mut type_context, &circuit_table, &local_table, circuit.output)?,
            }))
        }
        circuit1::PatTypedCircuitOrIntrinsic::Nand => Some(circuit1::TypedCircuitOrIntrinsic::Nand),
        circuit1::PatTypedCircuitOrIntrinsic::Const(value) => Some(circuit1::TypedCircuitOrIntrinsic::Const(value)),
    })?;

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, old_id)).collect();

    Some(IR { circuits, circuit_table, type_context })
}

fn put_pat_type<'file>(local_table: &mut HashMap<&'file str, ty::TypeSym>, pat: &circuit1::PatTypedPattern<'file>) {
    match &pat.kind {
        circuit1::PatTypedPatternKind::Identifier(_, name, ty) => {
            local_table.insert(name, ty.1); // TODO: report error for duplicate locals
        }
        circuit1::PatTypedPatternKind::Product(subpats) => {
            for subpat in subpats {
                put_pat_type(local_table, &subpat.1);
            }
        }
    }
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    circuit_table: &HashMap<&str, (ty::TypeSym, ty::TypeSym, circuit1::CircuitOrIntrinsicId)>,
    local_types: &HashMap<&str, ty::TypeSym>,
    expr: circuit1::UntypedExpr<'file>,
) -> Option<circuit1::TypedExpr<'file>> {
    let (kind, type_info) = match expr.kind {
        circuit1::UntypedExprKind::Ref(name_sp, name) => {
            let local_type = if let Some(ty) = local_types.get(name) {
                *ty
            } else {
                NoSuchLocal(name_sp, name).report();
                return None;
            };
            // TODO: replace with a ref to the locals id
            (circuit1::TypedExprKind::Ref(name_sp, name), local_type)
        }
        circuit1::UntypedExprKind::Call((name_sp, name), inline, arg) => {
            // this also does circuit name resolution
            if let Some((_, ty, _)) = circuit_table.get(name) {
                // TODO: replace with a call to the circuitid
                (circuit1::TypedExprKind::Call((name_sp, name), inline, Box::new(type_expr(type_context, circuit_table, local_types, *arg)?)), *ty)
            } else {
                NoSuchCircuit(name_sp, name).report();
                return None;
            }
        }
        circuit1::UntypedExprKind::Const(sp, value) => (circuit1::TypedExprKind::Const(sp, value), type_context.intern(ty::Type::Bit)),
        circuit1::UntypedExprKind::Get(base, field) => {
            let base = type_expr(type_context, circuit_table, local_types, *base)?;
            let base_ty = base.type_info;
            let field_ty = type_context.get(base_ty).field_type(type_context, field.1);
            if let Some(field_ty) = field_ty {
                (circuit1::TypedExprKind::Get(Box::new(base), field), field_ty)
            } else {
                (&*type_context, NoField { ty: base_ty, field_name_sp: field.0, field_name: field.1 }).report();
                return None;
            }
        }
        circuit1::UntypedExprKind::Product(exprs) => {
            let exprs: Vec<_> = exprs.into_iter().map(|(subexpr_name, subexpr)| Some((subexpr_name, type_expr(type_context, circuit_table, local_types, subexpr)?))).collect_all()?;
            let types = exprs.iter().map(|(field_name, subexpr)| (field_name.to_string(), subexpr.type_info)).collect();
            (circuit1::TypedExprKind::Product(exprs), type_context.intern(ty::Type::Product(types)))
        }
    };

    Some(circuit1::TypedExpr { type_info, kind, span: expr.span })
}
