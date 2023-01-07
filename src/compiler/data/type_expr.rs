use crate::compiler::error::Span;

#[derive(PartialEq, Debug)]
pub(crate) struct TypeExpr<'file> {
    pub(crate) kind: TypeExprKind<'file>,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum TypeExprKind<'file> {
    Product(Vec<TypeExpr<'file>>),
    RepProduct((Span<'file>, usize), Box<TypeExpr<'file>>),
    NamedProduct(Vec<((Span<'file>, &'file str), TypeExpr<'file>)>),
    Nominal(Span<'file>, &'file str),
}
