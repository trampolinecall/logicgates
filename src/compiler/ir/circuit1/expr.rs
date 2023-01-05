use crate::compiler::{arena, error::Span, ir::ty};

pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type UntypedExprArena<'file> = arena::Arena<UntypedExpr<'file>, ExprId>;

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

