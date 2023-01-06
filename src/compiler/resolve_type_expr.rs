use std::collections::HashMap;

use crate::utils::CollectAll;

use super::{
    arena,
    error::Span,
    ir::{circuit1, named_type, ty, type_expr},
    make_name_tables::{self, CircuitOrIntrinsicId},
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::TypeResolvedCircuitOrIntrinsic<'file>, CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
    pub(crate) type_table: HashMap<String, ty::TypeSym>,
}

pub(crate) fn resolve(make_name_tables::IR { circuits, circuit_table, mut type_context, type_table }: make_name_tables::IR) -> Option<IR> {
    let circuits = circuits.transform(|circuit| match circuit {
        circuit1::CircuitOrIntrinsic::Circuit(circuit) => Some(circuit1::CircuitOrIntrinsic::Circuit(circuit1::TypeResolvedCircuit {
            name: circuit.name,
            input: resolve_in_pat(&mut type_context, &type_table, circuit.input)?,
            expressions: circuit.expressions,
            output_type: resolve_type(&mut type_context, &type_table, &circuit.output_type)?,
            lets: resolve_in_let(&mut type_context, &type_table, circuit.lets)?,
            output: circuit.output,
        })),
        circuit1::CircuitOrIntrinsic::Nand => Some(circuit1::CircuitOrIntrinsic::Nand),
        circuit1::CircuitOrIntrinsic::Const(value) => Some(circuit1::CircuitOrIntrinsic::Const(value)),
    })?;

    // TODO: figure out how to make this work even though the type context is being moved because this will not compile
    let type_context = type_context.transform_named(|type_context, named_type| Some((named_type.name.1.to_string(), resolve_type_no_span(type_context, &type_table, &named_type.ty)?)))?;
    // TODO: disallow recursive types / infinitely sized types

    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn resolve_in_pat<'file>(
    type_context: &mut ty::TypeContext<named_type::NamedTypeDecl<'file>>,
    type_table: &HashMap<String, symtern::Sym<usize>>,
    pat: circuit1::UntypedPattern<'file>,
) -> Option<circuit1::TypeResolvedPattern<'file>> {
    Some(circuit1::Pattern {
        kind: match pat.kind {
            circuit1::PatternKind::Identifier(name_sp, name, type_expr) => circuit1::PatternKind::Identifier(name_sp, name, resolve_type(type_context, type_table, &type_expr)?),
            circuit1::PatternKind::Product(sp, subpats) => circuit1::PatternKind::Product(sp, subpats.into_iter().map(|subpat| resolve_in_pat(type_context, type_table, subpat)).collect_all()?),
        },
        type_info: (),
    })
}

fn resolve_in_let<'file>(
    type_context: &mut ty::TypeContext<named_type::NamedTypeDecl<'file>>,
    type_table: &HashMap<String, symtern::Sym<usize>>,
    lets: Vec<circuit1::UntypedLet<'file>>,
) -> Option<Vec<circuit1::TypeResolvedLet<'file>>> {
    lets.into_iter().map(|let_| Some(circuit1::Let { pat: resolve_in_pat(type_context, type_table, let_.pat)?, val: let_.val })).collect_all()
}

fn resolve_type<'file>(
    type_context: &mut ty::TypeContext<named_type::PartiallyDefinedNamedType>,
    type_table: &HashMap<String, ty::TypeSym>,
    ty: &type_expr::TypeExpr<'file>,
) -> Option<(Span<'file>, ty::TypeSym)> {
    let sp = ty.span();
    Some((sp, resolve_type_no_span(type_context, type_table, ty)?))
}
fn resolve_type_no_span<'file, NamedType>(type_context: &mut ty::TypeContext<NamedType>, type_table: &HashMap<String, ty::TypeSym>, ty: &type_expr::TypeExpr) -> Option<ty::TypeSym>
where
    named_type::NamedTypeId: arena::IsArenaIdFor<NamedType>,
{
    let ty = match ty {
        type_expr::TypeExpr::Bit(_) => type_context.intern(ty::Type::Bit),
        type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), resolve_type_no_span(type_context, type_table, subty_ast)?))).collect_all())?);
            type_context.intern(ty)
        }
        type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = resolve_type_no_span(type_context, type_table, type_)?;
            type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect()))
        }
        type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), (resolve_type_no_span(type_context, type_table, ty)?)))).collect_all())?); // TODO: report error if there are any duplicate fields
            type_context.intern(ty)
        }
        type_expr::TypeExpr::Named(_, name) => {
            let res = type_table.get(*name).copied();
            if let Some(other_type_decl) = res {
                other_type_decl
            } else {
                todo!("report error for undefined named type")
            }
        }
    };

    Some(ty)
}
