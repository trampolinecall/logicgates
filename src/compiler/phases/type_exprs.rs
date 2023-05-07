use crate::{
    compiler::{
        data::{ast, nominal_type, token, ty},
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
struct NoSuchLocal<'file>(token::PlainIdentifier<'file>);
struct NoSuchCircuit<'file>(token::CircuitIdentifier<'file>); // TODO: decide what to do with this

impl<'file> From<(&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, NoField<'file>)> for CompileError<'file> {
    fn from((types, NoField { ty, field_name_sp, field_name }): (&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, NoField<'file>)) -> Self {
        CompileError::new(field_name_sp, format!("no field called '{}' on type '{}'", field_name, types.get(ty).fmt(types)))
    }
}
impl<'file> From<NoSuchLocal<'file>> for CompileError<'file> {
    fn from(NoSuchLocal(name): NoSuchLocal<'file>) -> Self {
        CompileError::new(name.span, format!("no local called '{}'", name.name))
    }
}
impl<'file> From<NoSuchCircuit<'file>> for CompileError<'file> {
    fn from(NoSuchCircuit(name): NoSuchCircuit<'file>) -> Self {
        CompileError::new(name.span, format!("no circuit called '{}'", name.with_tag))
    }
}

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::TypedCircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}
pub(crate) fn type_(type_pats::IR { circuits, circuit_table, mut type_context }: type_pats::IR) -> Option<IR> {
    let circuit_table: HashMap<_, _> = circuit_table
        .into_iter()
        .map(|(name, circuit_id)| {
            let circuit = circuits.get(circuit_id);
            (name, (circuit.input_type(&mut type_context), circuit.output_type(&mut type_context), circuit_id))
        })
        .collect();

    let circuits = circuits.transform(|circuit| match circuit {
        ast::PatTypedCircuitOrIntrinsic::Circuit(circuit) => {
            let mut local_table = HashMap::new();

            // TODO: insert inputs and outputs
            put_pat_type(&mut local_table, &circuit.input);
            put_pat_type(&mut local_table, &circuit.output);
            for let_ in &circuit.lets {
                put_pat_type(&mut local_table, &let_.inputs);
                put_pat_type(&mut local_table, &let_.outputs);
            }

            Some(ast::TypedCircuitOrIntrinsic::Circuit(ast::TypedCircuit {
                name: circuit.name,
                input: circuit.input,
                output: circuit.output,
                lets: circuit.lets,
                connects: circuit
                    .connects
                    .into_iter()
                    .map(|ast::PatTypedConnect { start, end }| {
                        Some(ast::TypedConnect { start: type_expr(&mut type_context, &local_table, start)?, end: type_expr(&mut type_context, &local_table, end)? })
                    })
                    .collect::<Option<Vec<_>>>()?,
                aliases: circuit
                    .aliases
                    .into_iter()
                    .map(|ast::PatTypedAlias { pat, expr }| Some(ast::TypedAlias { pat, expr: type_expr(&mut type_context, &local_table, expr)? }))
                    .collect::<Option<Vec<_>>>()?,
            }))
        }
        ast::PatTypedCircuitOrIntrinsic::Nand => Some(ast::TypedCircuitOrIntrinsic::Nand),
        ast::PatTypedCircuitOrIntrinsic::Const(value) => Some(ast::TypedCircuitOrIntrinsic::Const(value)),
    })?;

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, old_id)).collect();

    Some(IR { circuits, circuit_table, type_context })
}

fn put_pat_type<'file>(local_table: &mut HashMap<&'file str, ty::TypeSym>, pat: &ast::PatTypedPattern<'file>) {
    match &pat.kind {
        ast::PatTypedPatternKind::Identifier(name, ty) => {
            local_table.insert(name.name, ty.1); // TODO: report error for duplicate locals
        }
        ast::PatTypedPatternKind::Product(subpats) => {
            for subpat in subpats {
                put_pat_type(local_table, &subpat.1);
            }
        }
    }
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    local_types: &HashMap<&str, ty::TypeSym>,
    expr: ast::UntypedExpr<'file>,
) -> Option<ast::TypedExpr<'file>> {
    let (kind, type_info) = match expr.kind {
        ast::UntypedExprKind::Ref(name) => {
            let local_type = if let Some(ty) = local_types.get(name.name) {
                *ty
            } else {
                NoSuchLocal(name).report();
                return None;
            };
            // TODO: replace with a ref to the locals id
            (ast::TypedExprKind::Ref(name), local_type)
        }
        ast::UntypedExprKind::Const(sp, value) => (ast::TypedExprKind::Const(sp, value), type_context.intern(ty::Type::Bit)),
        ast::UntypedExprKind::Get(base, field) => {
            let base = type_expr(type_context, local_types, *base)?;
            let base_ty = base.type_info;
            let field_ty = ty::Type::get_field_type(&type_context.get(base_ty).fields(type_context), field.1);
            if let Some(field_ty) = field_ty {
                (ast::TypedExprKind::Get(Box::new(base), field), field_ty)
            } else {
                (&*type_context, NoField { ty: base_ty, field_name_sp: field.0, field_name: field.1 }).report();
                return None;
            }
        }
        ast::UntypedExprKind::Product(exprs) => {
            let exprs: Vec<_> = exprs.into_iter().map(|(subexpr_name, subexpr)| Some((subexpr_name, type_expr(type_context, local_types, subexpr)?))).collect_all()?;
            let types = exprs.iter().map(|(field_name, subexpr)| (field_name.to_string(), subexpr.type_info)).collect();
            (ast::TypedExprKind::Product(exprs), type_context.intern(ty::Type::Product(types)))
        }
    };

    Some(ast::TypedExpr { type_info, kind, span: expr.span })
}
