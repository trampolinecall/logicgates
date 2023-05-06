use crate::{
    compiler::data::{token, ty, type_expr::TypeExpr},
    utils::arena,
};

#[derive(PartialEq, Debug)]
pub(crate) struct Struct<'file, TypeExpr> {
    pub(crate) name: token::TypeIdentifier<'file>,
    pub(crate) fields: Vec<(token::PlainIdentifier<'file>, TypeExpr)>,
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

pub(crate) type PartiallyDefinedStruct<'file> = Struct<'file, TypeExpr<'file>>;
pub(crate) type FullyDefinedStruct<'file> = Struct<'file, ty::TypeSym>;
