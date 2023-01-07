use crate::compiler::error::Span;
use crate::utils::arena;

use super::ty;

use super::type_expr::TypeExpr;

#[derive(PartialEq, Debug)]
pub(crate) struct StructDecl<'file> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) fields: Vec<((Span<'file>, &'file str), TypeExpr<'file>)>,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub(crate) struct StructId(usize);
impl arena::ArenaId for StructId {
    // TODO: derive macro for this trait
    fn make(i: usize) -> Self {
        StructId(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}

impl arena::IsArenaIdFor<FullyDefinedStruct> for StructId {}
impl<'file> arena::IsArenaIdFor<PartiallyDefinedStruct<'file>> for StructId {}

pub(crate) type PartiallyDefinedStruct<'file> = StructDecl<'file>;
pub(crate) type FullyDefinedStruct = (String, ty::TypeSym);
