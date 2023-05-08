use crate::{
    compiler::{
        data::{ast, nominal_type, token, ty, type_expr},
        error::{CompileError, Report, Span},
        phases::make_name_tables,
    },
    utils::{arena, collect_all::CollectAll},
};

use std::collections::HashMap;

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::CircuitOrIntrinsic<'file, ast::TypeResolved>, ast::CircuitOrIntrinsicId>,
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
        ast::CircuitOrIntrinsic::Circuit(circuit) => Some(ast::CircuitOrIntrinsic::Circuit(ast::Circuit {
            name: circuit.name,
            input: resolve_in_pat(&mut type_context, &type_table, circuit.input)?,
            output: resolve_in_pat(&mut type_context, &type_table, circuit.output)?,
            lets: circuit.lets.into_iter().map(|let_| resolve_in_let(&mut type_context, &type_table, let_)).collect::<Option<Vec<_>>>()?,
            connects: circuit.connects.into_iter().map(|ast::Connect { start, end }| ast::Connect { start: resolve_in_expr(start), end: resolve_in_expr(end) }).collect(),
            aliases: circuit.aliases.into_iter().map(|alias| resolve_in_alias(&mut type_context, &type_table, alias)).collect::<Option<Vec<_>>>()?,
        })),
        ast::CircuitOrIntrinsic::Nand => Some(ast::CircuitOrIntrinsic::Nand),
        ast::CircuitOrIntrinsic::Const(value) => Some(ast::CircuitOrIntrinsic::Const(value)),
        ast::CircuitOrIntrinsic::Unerror => Some(ast::CircuitOrIntrinsic::Unerror),
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

fn resolve_in_let<'file>(
    type_context: &mut ty::TypeContext<nominal_type::Struct<'file, type_expr::TypeExpr<'file>>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    ast::Let { inputs, outputs, gate }: ast::Let<'file, ast::Untyped>,
) -> Option<ast::Let<'file, ast::TypeResolved>> {
    Some(ast::Let { inputs: resolve_in_pat(type_context, type_table, inputs)?, outputs: resolve_in_pat(type_context, type_table, outputs)?, gate })
}

fn resolve_in_alias<'file>(
    type_context: &mut ty::TypeContext<nominal_type::Struct<'file, type_expr::TypeExpr<'file>>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    alias: ast::Alias<'file, ast::Untyped>,
) -> Option<ast::Alias<'file, ast::TypeResolved>> {
    Some(ast::Alias { pat: resolve_in_pat(type_context, type_table, alias.pat)?, expr: resolve_in_expr(alias.expr) })
}

fn resolve_in_expr(ast::Expr { kind, type_info, span }: ast::Expr<ast::Untyped>) -> ast::Expr<ast::TypeResolved> {
    ast::Expr {
        kind: match kind {
            ast::ExprKind::Ref(r) => ast::ExprKind::Ref(r),
            ast::ExprKind::Const(s, v) => ast::ExprKind::Const(s, v),
            ast::ExprKind::Get(s, f) => ast::ExprKind::Get(Box::new(resolve_in_expr(*s)), f),
            ast::ExprKind::Product(cs) => ast::ExprKind::Product(cs.into_iter().map(|(name, c)| (name, resolve_in_expr(c))).collect()),
        },
        type_info,
        span,
    }
}

fn resolve_in_pat<'file>(
    type_context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    pat: ast::Pattern<'file, ast::Untyped>,
) -> Option<ast::Pattern<'file, ast::TypeResolved>> {
    Some(ast::Pattern {
        kind: match pat.kind {
            ast::PatternKind::Identifier(name, type_expr) => ast::PatternKind::Identifier(name, resolve_type_expr(type_context, type_table, type_expr)?),
            ast::PatternKind::Product(subpats) => {
                ast::PatternKind::Product(subpats.into_iter().map(|(subpat_name, subpat)| Some((subpat_name, resolve_in_pat(type_context, type_table, subpat)?))).collect_all()?)
            }
        },
        type_info: (),
        span: pat.span,
    })
}

fn resolve_type_expr<'file, Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: type_expr::TypeExpr<'file>) -> Option<(Span<'file>, ty::TypeSym)> {
    let sp = ty.span;
    Some((sp, resolve_type_expr_no_span(type_context, type_table, ty)?))
}
fn resolve_type_expr_no_span<Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: type_expr::TypeExpr) -> Option<ty::TypeSym> {
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
