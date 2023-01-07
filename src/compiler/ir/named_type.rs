use crate::compiler::{arena, error::Span};

use super::ty;

use super::type_expr::TypeExpr;

#[derive(PartialEq, Debug)]
pub(crate) struct NamedTypeDecl<'file> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) ty: TypeExpr<'file>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub(crate) struct NamedTypeId(usize);
impl arena::ArenaId for NamedTypeId {
    // TODO: derive macro for this trait
    fn make(i: usize) -> Self {
        NamedTypeId(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}

impl arena::IsArenaIdFor<FullyDefinedNamedType> for NamedTypeId {}
impl<'file> arena::IsArenaIdFor<PartiallyDefinedNamedType<'file>> for NamedTypeId {}

pub(crate) type PartiallyDefinedNamedType<'file> = NamedTypeDecl<'file>;
pub(crate) type FullyDefinedNamedType = (String, ty::TypeSym);
