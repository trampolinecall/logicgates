use crate::compiler::{error::Span, ir::ty};

pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type TypedExpr<'file> = Expr<'file, ty::TypeSym>;

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file, TypeInfo> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, Box<Expr<'file, TypeInfo>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, TypeInfo>>, (Span<'file>, &'file str)),
    Multiple(Vec<Expr<'file, TypeInfo>>),
}

// TODO: remove all span methods
