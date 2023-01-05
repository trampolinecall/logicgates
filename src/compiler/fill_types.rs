use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::{ty, type_decl, type_expr};
use super::{arena, ir, make_name_tables};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::circuit1::TypedCircuitOrIntrinsic<'file>>,
    pub(crate) circuit_table: HashMap<String, arena::Id<ir::circuit1::TypedCircuitOrIntrinsic<'file>>>,

    pub(crate) type_context: ty::TypeContext,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}
pub(crate) fn fill<'file>(make_name_tables::IR { circuits, circuit_table, type_decls, type_table }: make_name_tables::IR) -> Option<IR> {
    let mut type_context = ty::TypeContext::new();

    let (type_decls, convert_type_decl_id) = match type_decls.transform_dependant(|type_decl, get_dep, _| {
        let ty = convert_type_ast_dependant(&mut type_context, &type_table, get_dep, &type_decl.ty);
        let named_type = type_context.new_named(type_decl.name.1.to_string(), try_transform_result!(ty));
        arena::SingleTransformResult::Ok(named_type)
    }) {
        Ok(type_decl) => type_decl,
        Err((loops, errors)) => todo!("report error from type name resolution in type filling"),
    };

    let type_table = type_table.into_iter().map(|(name, type_decl_id)| (name, *type_decls.get(convert_type_decl_id(type_decl_id)))).collect();

    // TODO: disallow recursive types / infinitely sized types

    let (circuits, transform_circuit_id) = circuits.transform(|circuit, transform_id| {
        match circuit {
            ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
                let output_type = convert_type_ast(&mut type_context, &type_table, &circuit.output_type_annotation)?;
                Some(ir::circuit1::CircuitOrIntrinsic::Circuit(ir::circuit1::Circuit {
                    name: circuit.name,
                    input: type_pat(&mut type_context, &type_table, circuit.input)?,
                    expressions: todo!("typing expressions arena"), // circuit.expressions,
                    output_type_annotation: circuit.output_type_annotation,
                    output_type,
                    lets: circuit.lets.into_iter().map(|let_| type_let(&mut type_context, &type_table, let_)).collect_all()?,
                    output: todo!("transform expression arena id"), // circuit.output,
                }))
            }
            ir::circuit1::CircuitOrIntrinsic::Nand => Some(ir::circuit1::CircuitOrIntrinsic::Nand),
        }
    })?;

    let circuit_table = circuit_table.into_iter().map(|(name, old_id)| (name, transform_circuit_id(old_id))).collect();

    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn type_let<'file>(type_context: &mut ty::TypeContext, type_table: &HashMap<String, ty::TypeSym>, let_: ir::circuit1::UntypedLet<'file>) -> Option<ir::circuit1::TypedLet<'file>> {
    Some(ir::circuit1::Let { pat: type_pat(type_context, type_table, let_.pat)?, val: todo!("transform expression arena id") /* let_.val */ })
}

fn type_pat<'file>(type_context: &mut ty::TypeContext, type_table: &HashMap<String, ty::TypeSym>, pat: ir::circuit1::UntypedPattern<'file>) -> Option<ir::circuit1::TypedPattern<'file>> {
    let (kind, type_info) = match pat.kind {
        ir::circuit1::PatternKind::Identifier(name_sp, name, ty) => {
            let type_info = convert_type_ast(type_context, type_table, &ty)?;
            (ir::circuit1::PatternKind::Identifier(name_sp, name, ty), type_info)
        }
        ir::circuit1::PatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(type_context, type_table, subpat)).collect_all()?;

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| Some((ind.to_string(), subpat.type_info))).collect_all()?);
            (ir::circuit1::PatternKind::Product(sp, typed_pats), type_context.intern(ty))
        }
    };

    Some(ir::circuit1::Pattern { kind, type_info })
}

fn convert_type_ast_dependant<'file>(
    type_context: &mut ty::TypeContext,
    type_table: &HashMap<String, arena::Id<type_decl::TypeDecl<'file>>>,
    get_other_type: arena::DependancyGetter<ty::TypeSym, type_decl::TypeDecl<'file>, Vec<()>>,
    ty: &type_expr::TypeExpr,
) -> arena::SingleTransformResult<ty::TypeSym, type_decl::TypeDecl<'file>, Vec<()>> {
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
                SingleTransformResult::Ok(*(try_transform_result!(get_other_type.get_dep(other_type_decl))))
            } else {
                todo!("report error for undefined named type")
            }
        }
    }
}
// TODO: there is probably a better way of doing this that doesn't need this code to be copied and pasted
fn convert_type_ast<'file>(type_context: &mut ty::TypeContext, type_table: &HashMap<String, ty::TypeSym>, ty: &type_expr::TypeExpr) -> Option<ty::TypeSym> {
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
