pub(crate) mod expr;

use crate::compiler::error::Span;

use super::{ty, type_expr::TypeExpr};

// TODO: separate ast from this?

pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ()>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, ()>;
pub(crate) type UntypedLet<'file> = Let<'file, ()>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, ()>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ty::TypeSym>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, TypeInfo>,
    pub(crate) expressions: expr::ExprArena<'file, TypeInfo>,
    pub(crate) output_type_annotation: TypeExpr<'file>,
    pub(crate) output_type: TypeInfo,
    pub(crate) lets: Vec<Let<'file, TypeInfo>>,
    pub(crate) output: expr::ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, TypeInfo> {
    Circuit(Circuit<'file, TypeInfo>),
    Nand,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, TypeInfo> {
    pub(crate) pat: Pattern<'file, TypeInfo>,
    pub(crate) val: expr::ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, TypeInfo> {
    pub(crate) kind: PatternKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, TypeInfo> {
    Identifier(Span<'file>, &'file str, TypeExpr<'file>),
    Product(Span<'file>, Vec<Pattern<'file, TypeInfo>>),
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, ty) => *sp + ty.span(),
            PatternKind::Product(sp, _) => *sp,
        }
    }
}
