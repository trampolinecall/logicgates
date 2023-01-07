use crate::{
    compiler::{
        data::{circuit1, nominal_type, ty, type_expr},
        error::{CompileError, Report, Span},
        phases::make_name_tables,
    },
    utils::{arena, collect_all::CollectAll},
};

use std::collections::HashMap;

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::TypeResolvedCircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, circuit1::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

struct UndefinedType<'file>(Span<'file>, &'file str);
impl<'file> From<UndefinedType<'file>> for CompileError<'file> {
    fn from(UndefinedType(sp, name): UndefinedType<'file>) -> Self {
        CompileError::new(sp, format!("undefined type '{}'", name))
    }
}

pub(crate) fn resolve(make_name_tables::IR { circuits, circuit_table, mut type_context, mut type_table }: make_name_tables::IR) -> Option<IR> {
    let bit_type = type_context.intern(ty::Type::Bit);
    let old_bit_type = type_table.insert("bit", bit_type);
    assert!(old_bit_type.is_none(), "cannot have other bit type in an empty type table");

    let circuits = circuits.transform(|circuit| match circuit {
        circuit1::UntypedCircuitOrIntrinsic::Circuit(circuit) => Some(circuit1::TypeResolvedCircuitOrIntrinsic::Circuit(circuit1::TypeResolvedCircuit {
            name: circuit.name,
            input: resolve_in_pat(&mut type_context, &type_table, circuit.input)?,
            output_type: resolve_type_expr(&mut type_context, &type_table, &circuit.output_type)?,
            lets: resolve_in_let(&mut type_context, &type_table, circuit.lets)?,
            output: circuit.output,
        })),
        circuit1::UntypedCircuitOrIntrinsic::Nand => Some(circuit1::TypeResolvedCircuitOrIntrinsic::Nand),
        circuit1::UntypedCircuitOrIntrinsic::Const(value) => Some(circuit1::TypeResolvedCircuitOrIntrinsic::Const(value)),
    })?;

    let type_context = type_context.transform_nominals(|type_context, struct_decl| {
        Some(nominal_type::FullyDefinedStruct {
            name: struct_decl.name,
            fields: struct_decl.fields.into_iter().map(|(field_name, field_ty)| Some((field_name, resolve_type_expr_no_span(type_context, &type_table, &field_ty)?))).collect_all()?,
        })
    })?;
    // TODO: disallow recursive types / infinitely sized types

    Some(IR { circuits, circuit_table, type_context })
}

fn resolve_in_pat<'file>(
    type_context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    pat: circuit1::UntypedPattern<'file>,
) -> Option<circuit1::TypeResolvedPattern<'file>> {
    Some(circuit1::TypeResolvedPattern {
        kind: match pat.kind {
            circuit1::UntypedPatternKind::Identifier(name_sp, name, type_expr) => circuit1::TypeResolvedPatternKind::Identifier(name_sp, name, resolve_type_expr(type_context, type_table, &type_expr)?),
            circuit1::UntypedPatternKind::Product(sp, subpats) => circuit1::TypeResolvedPatternKind::Product(sp, subpats.into_iter().map(|subpat| resolve_in_pat(type_context, type_table, subpat)).collect_all()?),
        },
        type_info: (),
        span: pat.span,
    })
}

fn resolve_in_let<'file>(
    type_context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    type_table: &HashMap<&str, symtern::Sym<usize>>,
    lets: Vec<circuit1::UntypedLet<'file>>,
) -> Option<Vec<circuit1::TypeResolvedLet<'file>>> {
    lets.into_iter().map(|let_| Some(circuit1::TypeResolvedLet { pat: resolve_in_pat(type_context, type_table, let_.pat)?, val: let_.val })).collect_all()
}

fn resolve_type_expr<'file, Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: &type_expr::TypeExpr<'file>) -> Option<(Span<'file>, ty::TypeSym)>
where
    nominal_type::StructId: arena::IsArenaIdFor<Struct>,
{
    let sp = ty.span();
    Some((sp, resolve_type_expr_no_span(type_context, type_table, ty)?))
}
fn resolve_type_expr_no_span<Struct>(type_context: &mut ty::TypeContext<Struct>, type_table: &HashMap<&str, ty::TypeSym>, ty: &type_expr::TypeExpr) -> Option<ty::TypeSym>
where
    nominal_type::StructId: arena::IsArenaIdFor<Struct>,
{
    match ty {
        type_expr::TypeExpr::Product { obrack: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().enumerate().map(|(ind, subty_ast)| Some((ind.to_string(), resolve_type_expr_no_span(type_context, type_table, subty_ast)?))).collect_all())?);
            Some(type_context.intern(ty))
        }
        type_expr::TypeExpr::RepProduct { obrack: _, num, cbrack: _, type_ } => {
            let ty = resolve_type_expr_no_span(type_context, type_table, type_)?;
            Some(type_context.intern(ty::Type::Product((0..num.1).map(|ind| (ind.to_string(), ty)).collect())))
        }
        type_expr::TypeExpr::NamedProduct { obrack: _, named: _, types: subtypes, cbrack: _ } => {
            let ty = ty::Type::Product((subtypes.iter().map(|(name, ty)| Some((name.1.to_string(), (resolve_type_expr_no_span(type_context, type_table, ty)?)))).collect_all())?); // TODO: report error if there are any duplicate fields
            Some(type_context.intern(ty))
        }
        type_expr::TypeExpr::Nominal(name_sp, name) => {
            let res = type_table.get(*name).copied();
            if let Some(other_type_decl) = res {
                Some(other_type_decl)
            } else {
                UndefinedType(*name_sp, name).report();
                None
            }
        }
    }
}
