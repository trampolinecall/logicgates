use crate::{
    compiler::{
        data::{ast, nominal_type, ty, type_expr, token},
        error::{CompileError, Report, Span},
        phases::make_name_tables,
    },
    utils::{arena, collect_all::CollectAll},
};

use std::collections::HashMap;

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::TypeResolvedCircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, ast::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

struct UndefinedType<'file, 'tok>(&'tok token::TypeIdentifier<'file>);
impl<'file> From<UndefinedType<'file, '_>> for CompileError<'file> {
    fn from(UndefinedType(i): UndefinedType<'file, '_>) -> Self {
        CompileError::new(i.span, format!("undefined type '{}'", i.with_tag))
    }
}

pub(crate) fn resolve(make_name_tables::IR { circuits, circuit_table, mut type_context, type_table }: make_name_tables::IR) -> Option<IR> {
    let circuits = circuits.transform(|circuit| match circuit {
        ast::UntypedCircuitOrIntrinsic::Circuit(circuit) => Some(ast::TypeResolvedCircuitOrIntrinsic::Circuit(ast::TypeResolvedCircuit {
            name: circuit.name,
            input: resolve_in_pat(&mut type_context, &type_table, circuit.input)?,
            output_type: resolve_type_expr(&mut type_context, &type_table, circuit.output_type)?,
            lets: resolve_in_let(&mut type_context, &type_table, circuit.lets)?,
            output: circuit.output,
        })),
        ast::UntypedCircuitOrIntrinsic::Nand => Some(ast::TypeResolvedCircuitOrIntrinsic::Nand),
        ast::UntypedCircuitOrIntrinsic::Const(value) => Some(ast::TypeResolvedCircuitOrIntrinsic::Const(value)),
    })?;

    let type_context = type_context.transform_nominals(|type_context, struct_decl| {
        Some(nominal_type::FullyDefinedStruct {
            name: struct_decl.name,
            fields: struct_decl.fields.into_iter().map(|(field_name, field_ty)| Some((field_name, resolve_type_expr_no_span(type_context, &type_table, field_ty)?))).collect_all()?,
        })
    })?;
    // TODO: disallow recursive types / infinitely sized types

    Some(IR { circuits, circuit_table, type_context })
}

fn resolve_in_pat<'file>(
    type_context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    pat: ast::UntypedPattern<'file>,
) -> Option<ast::TypeResolvedPattern<'file>> {
    Some(ast::TypeResolvedPattern {
        kind: match pat.kind {
            ast::UntypedPatternKind::Identifier(name, type_expr) => ast::TypeResolvedPatternKind::Identifier(name, resolve_type_expr(type_context, type_table, type_expr)?),
            ast::UntypedPatternKind::Product(subpats) => {
                ast::TypeResolvedPatternKind::Product(subpats.into_iter().map(|(subpat_name, subpat)| Some((subpat_name, resolve_in_pat(type_context, type_table, subpat)?))).collect_all()?)
            }
        },
        type_info: (),
        span: pat.span,
    })
}

fn resolve_in_let<'file>(
    type_context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    lets: Vec<ast::UntypedLet<'file>>,
) -> Option<Vec<ast::TypeResolvedLet<'file>>> {
    lets.into_iter().map(|let_| Some(ast::TypeResolvedLet { pat: resolve_in_pat(type_context, type_table, let_.pat)?, val: let_.val })).collect_all()
}

fn resolve_type_expr<'file, Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: type_expr::TypeExpr<'file>) -> Option<(Span<'file>, ty::TypeSym)>
where
    nominal_type::StructId: arena::IsArenaIdFor<Struct>,
{
    let sp = ty.span;
    Some((sp, resolve_type_expr_no_span(type_context, type_table, ty)?))
}
fn resolve_type_expr_no_span<Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: type_expr::TypeExpr) -> Option<ty::TypeSym>
where
    nominal_type::StructId: arena::IsArenaIdFor<Struct>,
{
    match ty.kind {
        type_expr::TypeExprKind::Product(subtypes) => {
            let ty = ty::Type::Product((subtypes.into_iter().map(|(field_name, subty_ast)| Some((field_name, resolve_type_expr_no_span(type_context, type_table, subty_ast)?))).collect_all())?); // TODO: report error if there are any duplicate fields, and also same in patterns and expressions
            Some(type_context.intern(ty))
        }
        type_expr::TypeExprKind::RepProduct(num, type_) => {
            let ty = resolve_type_expr_no_span(type_context, type_table, *type_)?;
            Some(type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        type_expr::TypeExprKind::Nominal(iden) => {
            let res = type_table.get(iden.name).copied();
            if let Some(other_type_decl) = res {
                Some(other_type_decl)
            } else {
                UndefinedType(&iden).report();
                None
            }
        }
    }
}
