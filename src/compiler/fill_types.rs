use std::collections::HashMap;

use crate::utils::CollectAll;

use super::ir::ty;
use super::{ir, make_name_tables};

pub(crate) struct IR<'file> {
    pub(crate) types: ty::Types, // TODO: rename to type_context
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
    pub(crate) circuit_table: HashMap<String, ir::circuit1::TypedCircuitOrIntrinsic<'file>>,
}
pub(crate) fn fill<'file>(ir: make_name_tables::IR) -> Option<IR> {
    let mut types = ty::Types::new();
    let type_table = HashMap::new(); // TODO: remove this

    let type_table = ir
        .types
        .into_iter()
        .map(|(name, type_decl)| {
            let ty = convert_type_ast(&mut types, &type_table, &type_decl.ty)?;

            let named_type = types.new_named(name.clone(), ty);
            Some((name, named_type))
        })
        .collect_all()?;

    let circuit_table = ir
        .circuits
        .into_iter()
        .map(|(name, circuit)| {
            match circuit {
                ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => {
                    let output_type = convert_type_ast(&mut types, &type_table, &circuit.output_type_annotation)?;
                    Some((
                        name,
                        ir::circuit1::CircuitOrIntrinsic::Circuit(ir::circuit1::Circuit {
                            name: circuit.name,
                            input: type_pat(&mut types, &type_table, circuit.input)?,
                            expressions: todo!("typing expressions arena"), // circuit.expressions,
                            output_type_annotation: circuit.output_type_annotation,
                            output_type,
                            lets: circuit.lets.into_iter().map(|let_| type_let(&mut types, &type_table, let_)).collect_all()?,
                            output: todo!("transform expression arena id"), // circuit.output,
                        }),
                    ))
                }
                ir::circuit1::CircuitOrIntrinsic::Nand => Some((name, ir::circuit1::CircuitOrIntrinsic::Nand)),
            }
        })
        .collect_all()?;

    Some(IR { types, type_table, circuit_table })
}

fn type_let<'file>(types: &mut ty::Types, type_table: &HashMap<String, ty::TypeSym>, let_: ir::circuit1::UntypedLet<'file>) -> Option<ir::circuit1::TypedLet<'file>> {
    Some(ir::circuit1::Let { pat: type_pat(types, type_table, let_.pat)?, val: todo!("transform expression arena id") /* let_.val */ })
}

fn type_pat<'file>(types: &mut ty::Types, type_table: &HashMap<String, ty::TypeSym>, pat: ir::circuit1::UntypedPattern<'file>) -> Option<ir::circuit1::TypedPattern<'file>> {
    let (kind, type_info) = match pat.kind {
        ir::circuit1::PatternKind::Identifier(name_sp, name, ty) => {
            let type_info = convert_type_ast(types, type_table, &ty)?;
            (ir::circuit1::PatternKind::Identifier(name_sp, name, ty), type_info)
        }
        ir::circuit1::PatternKind::Product(sp, pats) => {
            let typed_pats: Vec<_> = pats.into_iter().map(|subpat| type_pat(types, type_table, subpat)).collect_all()?;

            let ty = ty::Type::Product(typed_pats.iter().enumerate().map(|(ind, subpat)| Some((ind.to_string(), subpat.type_info))).collect_all()?);
            (ir::circuit1::PatternKind::Product(sp, typed_pats), types.intern(ty))
        }
    };

    Some(ir::circuit1::Pattern { kind, type_info })
}

fn convert_type_ast(types: &mut ty::Types, type_table: &HashMap<String, ty::TypeSym>, ty: &ir::type_expr::TypeExpr) -> Option<ty::TypeSym> {
    match ty {
        ir::type_expr::TypeExpr::Bit(_) => Some(types.intern(ty::Type::Bit)),
        ir::type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), convert_type_ast(types, type_table, subty_ast)?))).collect_all()?);
            Some(types.intern(ty))
        }
        ir::type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = convert_type_ast(types, type_table, type_)?;
            Some(types.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        ir::type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product(subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), convert_type_ast(types, type_table, ty)?))).collect_all()?); // TODO: report error if there are any duplicate fields
            Some(types.intern(ty))
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
