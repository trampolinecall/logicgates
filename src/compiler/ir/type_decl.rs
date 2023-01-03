use crate::compiler::error::Span;

use super::type_expr::TypeExpr;

#[derive(PartialEq, Debug)]
pub(crate) struct NamedTypeDecl<'file> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) ty: TypeExpr<'file>,
}

