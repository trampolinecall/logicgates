use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::{circuit1, named_type, ty, type_expr};
use super::{arena, ir, make_name_tables};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::circuit1::TypedCircuitOrIntrinsic<'file>, make_name_tables::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, make_name_tables::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}
pub(crate) fn fill<'file>(make_name_tables::IR { circuits, circuit_table, type_decls, type_table }: make_name_tables::IR) -> Option<IR> {
    // this whole function is really messy but i dont know how to fix it
    let mut type_context = ty::TypeContext::new();

    let type_decls = match type_decls.transform_dependant(|type_decl: &named_type::NamedTypeDecl, get_dep| {
        let ty = convert_type_ast_dependant(&mut type_context, &type_table, get_dep, &type_decl.ty);
        let named_type = type_context.named.add((type_decl.name.1.to_string(), try_transform_result!(ty)));
        todo!()
        // arena::SingleTransformResult::Ok(named_type) TODO
    }) {
        Ok(res) => res,
        Err((loops, errors)) => todo!("report error from type name resolution in type filling"),
    };

    /*
    let type_table = type_table.into_iter().map(|(name, type_decl_id)| (name, *type_decls.get(type_decl_id))).collect();

    // TODO: disallow recursive types / infinitely sized types

    let circuits = match circuits.transform_dependant(|circuit, get_other_circuit| {
        use super::arena::SingleTransformResult;
        match circuit {
            ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
                let output_type = convert_type_ast(&mut type_context, &type_table, &circuit.output_type);

                let mut local_table = HashMap::new();

                let input = type_pat(&mut type_context, &type_table, &mut local_table, &circuit.input);
                let let_pats: Option<Vec<_>> = circuit.lets.iter().map(|let_| type_let_pat(&mut type_context, &type_table, &mut local_table, let_)).collect_all();

                // let (expressions, transform_expr_id) = circuit.expressions.transform(|expr, transform_expr_id| type_expr(&mut type_context, &type_table, &local_table, expr, transform_expr_id)); TODO
                let (expressions, transform_expr_id): (_, fn(_) -> _) = todo!();

                SingleTransformResult::Ok(ir::circuit1::CircuitOrIntrinsic::Circuit(ir::circuit1::Circuit {
                    name: circuit.name,
                    input: if let Some(r) = input { r } else { return SingleTransformResult::Err(()) },
                    expressions,
                    output_type_annotation: circuit.output_type_annotation,
                    output_type: if let Some(r) = output_type { r } else { return SingleTransformResult::Err(()) },
                    lets: if let Some(r) = let_pats { r } else { return SingleTransformResult::Err(()) }
                        .into_iter()
                        .map(|let_| ir::circuit1::Let { pat: let_.0, val: transform_expr_id(let_.1) })
                        .collect(),
                    output: transform_expr_id(circuit.output),
                }))
            }
            ir::circuit1::CircuitOrIntrinsic::Nand => SingleTransformResult::Ok(ir::circuit1::CircuitOrIntrinsic::Nand),
        }
    }) {
        Ok(r) => r,
        Err(_) => todo!("report error from circuit typing"),
    };

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, (old_id))).collect();

    Some(IR { circuits, circuit_table, type_context, type_table })
    */
    todo!()
}

fn convert_type_ast_dependant<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    type_table: &HashMap<String, named_type::NamedTypeId>,
    get_other_type: arena::DependancyGetter<(String, ty::TypeSym), named_type::NamedTypeDecl<'file>, Vec<()>, named_type::NamedTypeId>,
    ty: &type_expr::TypeExpr,
) -> arena::SingleTransformResult<ty::TypeSym, named_type::NamedTypeId, Vec<()>> {
    use arena::SingleTransformResult;
    match ty {
        type_expr::TypeExpr::Bit(_) => SingleTransformResult::Ok(type_context.intern(ty::Type::Bit)),
        type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(try_transform_result!(subtypes
                .iter()
                .enumerate()
                .map(|(ind, subty_ast)| SingleTransformResult::Ok((ind.to_string(), try_transform_result!(convert_type_ast_dependant(type_context, type_table, get_other_type, subty_ast)))))
                .collect_all()));
            SingleTransformResult::Ok(type_context.intern(ty))
        }
        type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = try_transform_result!(convert_type_ast_dependant(type_context, type_table, get_other_type, type_));
            SingleTransformResult::Ok(type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        ir::type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(try_transform_result!(subtypes
                .iter()
                .map(|(name, ty)| { SingleTransformResult::Ok((name.1.to_string(), try_transform_result!(convert_type_ast_dependant(type_context, type_table, get_other_type, ty)))) })
                .collect_all())); // TODO: report error if there are any duplicate fields
            SingleTransformResult::Ok(type_context.intern(ty))
        }
        ir::type_expr::TypeExpr::Named(_, name) => {
            let res = type_table.get(*name).copied();
            if let Some(other_type_decl) = res {
                SingleTransformResult::Ok((try_transform_result!(get_other_type.get_dep(other_type_decl))).1)
            } else {
                todo!("report error for undefined named type")
            }
        }
    }
}
// TODO: there is probably a better way of doing this that doesn't need this code to be copied and pasted
fn convert_type_ast<'file>(type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>, type_table: &HashMap<String, ty::TypeSym>, ty: &type_expr::TypeExpr) -> Option<ty::TypeSym> {
    match ty {
        type_expr::TypeExpr::Bit(_) => Some(type_context.intern(ty::Type::Bit)),
        type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), convert_type_ast(type_context, type_table, subty_ast)?))).collect_all())?);
            Some(type_context.intern(ty))
        }
        type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type_ast(type_context, type_table, type_)?;
            Some(type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        ir::type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), (convert_type_ast(type_context, type_table, ty)?)))).collect_all())?); // TODO: report error if there are any duplicate fields
            Some(type_context.intern(ty))
        }
        ir::type_expr::TypeExpr::Named(_, name) => {
            let res = type_table.get(*name).copied();
            if let Some(other_type_decl) = res {
                Some(other_type_decl)
            } else {
                todo!("report error for undefined named type")
            }
        }
    }
}

fn type_let_pat<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    type_table: &HashMap<String, ty::TypeSym>,
    local_types: &mut HashMap<String, symtern::Sym<usize>>,
    let_: &ir::circuit1::UntypedLet<'file>,
) -> Option<(circuit1::TypedPattern<'file>, circuit1::expr::ExprId)> {
    Some((type_pat(type_context, type_table, local_types, &let_.pat)?, let_.val))
}
fn type_pat<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    type_table: &HashMap<String, ty::TypeSym>,
    local_types: &mut HashMap<String, symtern::Sym<usize>>,
    pat: &ir::circuit1::UntypedPattern<'file>,
) -> Option<ir::circuit1::TypedPattern<'file>> {
    let (kind, type_info) = match &pat.kind {
        ir::circuit1::PatternKind::Identifier(name_sp, name, ty) => {
            let type_info = convert_type_ast(type_context, type_table, &ty)?;
            local_types.insert(name.to_string(), type_info);
            (ir::circuit1::PatternKind::Identifier(*name_sp, name, /* *ty */ todo!()), type_info)
            // TODO
        }
        ir::circuit1::PatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(type_context, type_table, local_types, &subpat)).collect_all()?;

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| Some((ind.to_string(), subpat.type_info))).collect_all()?);
            (ir::circuit1::PatternKind::Product(*sp, typed_pats), type_context.intern(ty))
        }
    };

    Some(ir::circuit1::Pattern { kind, type_info })
}

fn type_expr<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    type_table: &HashMap<String, symtern::Sym<usize>>,
    local_types: &HashMap<String, symtern::Sym<usize>>,
    expr: circuit1::expr::UntypedExpr<'file>,
    transform_expr_id: fn(ir::circuit1::expr::ExprId) -> ir::circuit1::expr::ExprId,
) -> Option<circuit1::expr::Expr<'file, symtern::Sym<usize>>> {
    let (kind, type_info) = /* match expr.kind {
        circuit1::expr::ExprKind::Ref(sp, name) => {
            let local_type = if let Some(ty) = local_types.get(name) { ty } else { todo!("report error for undefined local usage") };
            (circuit1::expr::ExprKind::Ref(sp, name), *local_type)
        }
        circuit1::expr::ExprKind::Call(name, inline, arg) => (circuit1::expr::ExprKind::Call(name, inline, transform_expr_id(arg)), {let x = todo!(); x}),
        circuit1::expr::ExprKind::Const(sp, value) => (circuit1::expr::ExprKind::Const(sp, value), type_context.intern(ty::Type::Bit)),
        circuit1::expr::ExprKind::Get(base, field) => (circuit1::expr::ExprKind::Get(transform_expr_id(base), field), todo!()),
        circuit1::expr::ExprKind::Multiple { obrack, exprs, cbrack } => (circuit1::expr::ExprKind::Multiple { obrack, exprs: exprs.into_iter().map(transform_expr_id).collect(), cbrack }, todo!()),
    }*/ todo!() ;

    Some(circuit1::expr::Expr { kind, type_info })
}
