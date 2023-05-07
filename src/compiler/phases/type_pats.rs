use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, nominal_type, ty},
        phases::resolve_type_expr,
    },
    utils::arena,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::CircuitOrIntrinsic<'file, ast::PatTyped>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, ast::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn type_(resolve_type_expr::IR { circuits, circuit_table, mut type_context }: resolve_type_expr::IR) -> IR {
    IR {
        circuits: circuits.transform_infallible(|circuit| match circuit {
            ast::CircuitOrIntrinsic::Circuit(circuit) => ast::CircuitOrIntrinsic::Circuit(ast::Circuit {
                name: circuit.name,
                input: type_pat(&mut type_context, circuit.input),
                output: type_pat(&mut type_context, circuit.output),
                lets: circuit
                    .lets
                    .into_iter()
                    .map(|ast::Let { inputs, outputs, gate }| ast::Let { inputs: type_pat(&mut type_context, inputs), outputs: type_pat(&mut type_context, outputs), gate })
                    .collect(),
                connects: circuit.connects.into_iter().map(|ast::Connect { start, end }| ast::Connect { start: type_in_expr(start), end: type_in_expr(end) }).collect(),
                aliases: circuit.aliases.into_iter().map(|ast::Alias { pat, expr }| ast::Alias { pat: type_pat(&mut type_context, pat), expr: type_in_expr(expr) }).collect(),
            }),
            ast::CircuitOrIntrinsic::Nand => ast::CircuitOrIntrinsic::Nand,
            ast::CircuitOrIntrinsic::Const(value) => ast::CircuitOrIntrinsic::Const(value),
        }),
        circuit_table,
        type_context,
    }
}

fn type_in_expr(ast::Expr { kind, type_info, span }: ast::Expr<ast::TypeResolved>) -> ast::Expr<ast::PatTyped> {
    ast::Expr {
        kind: match kind {
            ast::ExprKind::Ref(r) => ast::ExprKind::Ref(r),
            ast::ExprKind::Const(s, v) => ast::ExprKind::Const(s, v),
            ast::ExprKind::Get(s, f) => ast::ExprKind::Get(Box::new(type_in_expr(*s)), f),
            ast::ExprKind::Product(cs) => ast::ExprKind::Product(cs.into_iter().map(|(name, c)| (name, type_in_expr(c))).collect()),
        },
        type_info,
        span,
    }
}

fn type_pat<'file>(type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>, pat: ast::Pattern<'file, ast::TypeResolved>) -> ast::Pattern<'file, ast::PatTyped> {
    let (kind, type_info) = match pat.kind {
        ast::PatternKind::Identifier(name, ty) => (ast::PatternKind::Identifier(name, ty), ty.1),
        ast::PatternKind::Product(pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|(field_name, subpat)| (field_name, type_pat(type_context, subpat))).collect();

            let ty = ty::Type::Product(typed_pats.iter().map(|(field_name, subpat)| (field_name.clone(), subpat.type_info)).collect());
            (ast::PatternKind::Product(typed_pats), type_context.intern(ty))
        }
    };

    ast::Pattern { kind, type_info, span: pat.span }
}
