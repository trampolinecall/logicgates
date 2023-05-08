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
    pub(crate) circuits: arena::Arena<ast::CircuitOrIntrinsic<'file, ast::Typed>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}
pub(crate) fn type_(type_pats::IR { circuits, circuit_table, mut type_context }: type_pats::IR) -> Option<IR> {
    let circuits = circuits.transform(|circuit| match circuit {
        ast::CircuitOrIntrinsic::Circuit(circuit) => {
            let mut local_table = HashMap::new();

            // TODO: insert inputs and outputs
            put_pat_type(&mut local_table, &circuit.input);
            put_pat_type(&mut local_table, &circuit.output);
            for let_ in &circuit.lets {
                put_pat_type(&mut local_table, &let_.inputs);
                put_pat_type(&mut local_table, &let_.outputs);
            }

            Some(ast::CircuitOrIntrinsic::Circuit(ast::Circuit {
                name: circuit.name,
                input: convert_pattern(circuit.input),
                output: convert_pattern(circuit.output),
                lets: circuit.lets.into_iter().map(|ast::Let { inputs, outputs, gate }| ast::Let { inputs: convert_pattern(inputs), outputs: convert_pattern(outputs), gate }).collect(),
                connects: circuit
                    .connects
                    .into_iter()
                    .map(|ast::Connect { start, end }| Some(ast::Connect { start: type_expr(&mut type_context, &local_table, start)?, end: type_expr(&mut type_context, &local_table, end)? }))
                    .collect::<Option<Vec<_>>>()?,
                aliases: circuit
                    .aliases
                    .into_iter()
                    .map(|ast::Alias { pat, expr }| Some(ast::Alias { pat: convert_pattern(pat), expr: type_expr(&mut type_context, &local_table, expr)? }))
                    .collect::<Option<Vec<_>>>()?,
            }))
        }
        ast::CircuitOrIntrinsic::Nand => Some(ast::CircuitOrIntrinsic::Nand),
        ast::CircuitOrIntrinsic::Const(value) => Some(ast::CircuitOrIntrinsic::Const(value)),
        ast::CircuitOrIntrinsic::Unerror => Some(ast::CircuitOrIntrinsic::Unerror),
    })?;

    let circuit_table = circuit_table
        .into_iter()
        .map(|(name, circuit_id)| {
            let circuit = circuits.get(circuit_id);
            (name, (circuit.input_type(&mut type_context), circuit.output_type(&mut type_context), circuit_id))
        })
        .collect();

    Some(IR { circuits, circuit_table, type_context })
}

fn convert_pattern(ast::Pattern { kind, type_info, span }: ast::Pattern<ast::PatTyped>) -> ast::Pattern<ast::Typed> {
    ast::Pattern {
        kind: match kind {
            ast::PatternKind::Identifier(i, ty) => ast::PatternKind::Identifier(i, ty),
            ast::PatternKind::Product(cs) => ast::PatternKind::Product(cs.into_iter().map(|(name, c)| (name, convert_pattern(c))).collect()),
        },
        type_info,
        span,
    }
}

fn put_pat_type<'file>(local_table: &mut HashMap<&'file str, ty::TypeSym>, pat: &ast::Pattern<'file, ast::PatTyped>) {
    match &pat.kind {
        ast::PatternKind::Identifier(name, ty) => {
            local_table.insert(name.name, ty.1); // TODO: report error for duplicate locals
        }
        ast::PatternKind::Product(subpats) => {
            for subpat in subpats {
                put_pat_type(local_table, &subpat.1);
            }
        }
    }
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    local_types: &HashMap<&str, ty::TypeSym>,
    expr: ast::Expr<'file, ast::PatTyped>,
) -> Option<ast::Expr<'file, ast::Typed>> {
    let (kind, type_info) = match expr.kind {
        ast::ExprKind::Ref(name) => {
            let local_type = if let Some(ty) = local_types.get(name.name) {
                *ty
            } else {
                NoSuchLocal(name).report();
                return None;
            };
            // TODO: replace with a ref to the locals id
            (ast::ExprKind::Ref(name), local_type)
        }
        ast::ExprKind::Const(sp, value) => (ast::ExprKind::Const(sp, value), type_context.intern(ty::Type::Bit)),
        ast::ExprKind::Get(base, field) => {
            let base = type_expr(type_context, local_types, *base)?;
            let base_ty = base.type_info;
            let field_ty = ty::Type::get_field_type(&type_context.get(base_ty).fields(type_context), field.1);
            if let Some(field_ty) = field_ty {
                (ast::ExprKind::Get(Box::new(base), field), field_ty)
            } else {
                (&*type_context, NoField { ty: base_ty, field_name_sp: field.0, field_name: field.1 }).report();
                return None;
            }
        }
        ast::ExprKind::Product(exprs) => {
            let exprs: Vec<_> = exprs.into_iter().map(|(subexpr_name, subexpr)| Some((subexpr_name, type_expr(type_context, local_types, subexpr)?))).collect_all()?;
            let types = exprs.iter().map(|(field_name, subexpr)| (field_name.to_string(), subexpr.type_info)).collect();
            (ast::ExprKind::Product(exprs), type_context.intern(ty::Type::Product(types)))
        }
    };

    Some(ast::Expr { type_info, kind, span: expr.span })
}
