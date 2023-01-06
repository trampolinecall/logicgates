use crate::compiler::{arena, error::Span, ir::ty};

pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type UntypedExprArena<'file> = arena::Arena<UntypedExpr<'file>, ExprId>;

pub(crate) type TypedExpr<'file> = Expr<'file, ty::TypeSym>;
pub(crate) type TypedExprArena<'file> = arena::Arena<TypedExpr<'file>, ExprId>;

pub(crate) type ExprArena<'file, TypeInfo> = arena::Arena<Expr<'file, TypeInfo>, ExprId>;
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Debug, Hash)]
pub(crate) struct ExprId(usize);
impl<'file, TypeInfo> arena::IsArenaIdFor<Expr<'file, TypeInfo>> for ExprId {}
impl arena::ArenaId for ExprId {
    fn make(i: usize) -> Self {
        ExprId(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file>,
    pub(crate) type_info: TypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, ExprId),
    Const(Span<'file>, bool),
    Get(ExprId, (Span<'file>, &'file str)),
    Multiple(Vec<ExprId>),
}

// TODO: remove all span methods
