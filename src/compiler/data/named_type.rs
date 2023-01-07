use crate::compiler::error::Span;
use crate::utils::arena;

use super::ty;

use super::type_expr::TypeExpr;

#[derive(PartialEq, Debug)]
pub(crate) struct Struct<'file, TypeExpr> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) fields: Vec<((Span<'file>, &'file str), TypeExpr)>,
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

impl arena::IsArenaIdFor<FullyDefinedStruct<'_>> for StructId {}
impl arena::IsArenaIdFor<PartiallyDefinedStruct<'_>> for StructId {}

pub(crate) type PartiallyDefinedStruct<'file> = Struct<'file, TypeExpr<'file>>;
pub(crate) type FullyDefinedStruct<'file> = Struct<'file, ty::TypeSym>;
