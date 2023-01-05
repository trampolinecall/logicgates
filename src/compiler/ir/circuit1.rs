pub(crate) mod expr;

use crate::compiler::error::Span;

use super::{ty, type_expr};

// TODO: separate ast from this?

// TODO; move Circuit and CircuitOrIntrinsic into separate module
pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedLet<'file> = Let<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, (), type_expr::TypeExpr<'file>>;

pub(crate) type PartiallyTypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type PartiallyTypedCircuit<'file> = Circuit<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type PartiallyTypedLet<'file> = Let<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type PartiallyTypedPattern<'file> = Pattern<'file, (), (Span<'file>, ty::TypeSym)>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo, TypeExpr> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, TypeInfo, TypeExpr>,
    pub(crate) expressions: expr::ExprArena<'file, TypeInfo>,
    pub(crate) output_type: TypeExpr,
    pub(crate) lets: Vec<Let<'file, TypeInfo, TypeExpr>>,
    pub(crate) output: expr::ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, TypeInfo, TypeExpr> {
    Circuit(Circuit<'file, TypeInfo, TypeExpr>),
    Nand,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, TypeInfo, TypeExpr> {
    pub(crate) pat: Pattern<'file, TypeInfo, TypeExpr>,
    pub(crate) val: expr::ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, TypeInfo, TypeExpr> {
    pub(crate) kind: PatternKind<'file, TypeInfo, TypeExpr>,
    pub(crate) type_info: TypeInfo,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, TypeInfo, TypeExpr> {
    Identifier(Span<'file>, &'file str, TypeExpr),
    Product(Span<'file>, Vec<Pattern<'file, TypeInfo, TypeExpr>>),
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo, type_expr::TypeExpr<'file>> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, ty) => *sp + ty.span(),
            PatternKind::Product(sp, _) => *sp,
        }
    }
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo, (Span<'file>, ty::TypeSym)> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, (ty_sp, _)) => *sp + *ty_sp,
            PatternKind::Product(sp, _) => *sp,
        }
    }
}
