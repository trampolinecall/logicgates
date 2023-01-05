use crate::compiler::{arena, error::Span};

use super::{ty, type_expr::TypeExpr};

// TODO: separate ast from this?

pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ()>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, ()>;
pub(crate) type UntypedLet<'file> = Let<'file, ()>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, ()>;
pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type UntypedExprArena<'file> = arena::Arena<UntypedExpr<'file>, ExprId>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ty::TypeSym>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym>;
pub(crate) type TypedExpr<'file> = Expr<'file, ty::TypeSym>;
pub(crate) type TypedExprArena<'file> = arena::Arena<TypedExpr<'file>, ExprId>;

pub(crate) type ExprArena<'file, TypeInfo> = arena::Arena<Expr<'file, TypeInfo>, ExprId>;
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub(crate) struct ExprId(usize);
impl<'file, TypeInfo> arena::IsArenaIdFor<Expr<'file, TypeInfo>> for ExprId {}
impl arena::ArenaId for ExprId {
    fn make(i: usize) -> Self {
        todo!()
    }

    fn get(&self) -> usize {
        todo!()
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, TypeInfo>,
    pub(crate) expressions: ExprArena<'file, TypeInfo>,
    pub(crate) output_type_annotation: TypeExpr<'file>,
    pub(crate) output_type: TypeInfo,
    pub(crate) lets: Vec<Let<'file, TypeInfo>>,
    pub(crate) output: ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, TypeInfo> {
    Circuit(Circuit<'file, TypeInfo>),
    Nand,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, TypeInfo> {
    pub(crate) pat: Pattern<'file, TypeInfo>,
    pub(crate) val: ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file>,
    pub(crate) type_info: TypeInfo,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, ExprId),
    Const(Span<'file>, bool),
    Get(ExprId, (Span<'file>, &'file str)),
    Multiple { obrack: Span<'file>, exprs: Vec<ExprId>, cbrack: Span<'file> },
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

impl<'file> ExprKind<'file> {
    pub(crate) fn span<TypeInfo>(&self, expressions: &ExprArena<'file, TypeInfo>) -> Span<'file> {
        match self {
            ExprKind::Ref(sp, _) | ExprKind::Const(sp, _) => *sp,
            ExprKind::Call((circuit_name_sp, _), _, arg) => *circuit_name_sp + expressions.get(*arg).kind.span(expressions),
            ExprKind::Get(expr, (field_sp, _)) => expressions.get(*expr).kind.span(expressions) + *field_sp,
            ExprKind::Multiple { obrack, cbrack, exprs: _ } => *obrack + *cbrack,
        }
    }
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, ty) => *sp + ty.span(),
            PatternKind::Product(sp, _) => *sp,
        }
    }
}
