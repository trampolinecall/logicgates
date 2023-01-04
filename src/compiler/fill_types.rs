use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::ty;
use super::{arena, ir, make_name_tables};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::circuit1::TypedCircuitOrIntrinsic<'file>>,
    pub(crate) circuit_table: HashMap<String, arena::Id<ir::circuit1::TypedCircuitOrIntrinsic<'file>>>,

    pub(crate) type_context: ty::TypeContext,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}
pub(crate) fn fill<'file>(ir: make_name_tables::IR) -> Option<IR> {
    /*
    let mut type_context = ty::TypeContext::new();

    let type_decls = ir.type_decls.transform(|type_decl| {
        let ty = convert_type_ast(&mut type_context, &ir.type_table, &type_decl.ty)?;
        // let named_type = types.new_named(name.clone(), ty); TODO
        Some((todo!(), todo!() /* name, named_type */))
    })?;

    // TODO: disallow recursive types / infinitely sized types

    let circuits = ir
        .circuits
        .transform(|(/* name, */ circuit)| {
            match circuit {
                ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
                    let output_type = convert_type_ast(&mut type_context, &type_table, &circuit.output_type_annotation)?;
                    Some((
                        // name,
                        ir::circuit1::CircuitOrIntrinsic::Circuit(ir::circuit1::Circuit {
                            name: circuit.name,
                            input: type_pat(&mut type_context, &type_table, circuit.input)?,
                            expressions: todo!("typing expressions arena"), // circuit.expressions,
                            output_type_annotation: circuit.output_type_annotation,
                            output_type,
                            lets: circuit.lets.into_iter().map(|let_| type_let(&mut type_context, &type_table, let_)).collect_all()?,
                            output: todo!("transform expression arena id"), // circuit.output,
                        })
                    ))
                }
                ir::circuit1::CircuitOrIntrinsic::Nand => Some((/* name, */ ir::circuit1::CircuitOrIntrinsic::Nand)),
            }
        })?;

    Some(IR { circuits, circuit_table, type_context, type_table })
    */
    todo!()
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

fn convert_type_ast(type_context: &mut ty::TypeContext, type_table: &HashMap<String, ty::TypeSym>, ty: &ir::type_expr::TypeExpr) -> Option<ty::TypeSym> {
    match ty {
        ir::type_expr::TypeExpr::Bit(_) => Some(type_context.intern(ty::Type::Bit)),
        ir::type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), convert_type_ast(type_context, type_table, subty_ast)?))).collect_all()?);
            Some(type_context.intern(ty))
        }
        ir::type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type_ast(type_context, type_table, type_)?;
            Some(type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        ir::type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), convert_type_ast(type_context, type_table, ty)?))).collect_all()?); // TODO: report error if there are any duplicate fields
            Some(type_context.intern(ty))
        }
        ir::type_expr::TypeExpr::Named(_, name) => {
            let res = type_table.get(*name).copied();
            if res.is_none() {
                todo!("report error for undefined named type")
            }
            res
        }
    }
}
