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

impl<'file> arena::IsArenaIdFor<FullyDefinedNamedType> for NamedTypeId {}
impl<'file> arena::IsArenaIdFor<PartiallyDefinedNamedType<'file>> for NamedTypeId {}

// this stores all the named types, one for each named type definition ast
// this needs to be an arena and not an interner because every named type definition ast makes a unique type
// these are used through the Type::Named constructor which is compared based off of its index into this array, meaning that named types will not be equal unless they point to the same item in this array
pub(crate) type PartiallyDefinedNamedType<'file> = NamedTypeDecl<'file>;
pub(crate) type FullyDefinedNamedType = (String, ty::TypeSym);
// TODO: put these arenas in type context?
