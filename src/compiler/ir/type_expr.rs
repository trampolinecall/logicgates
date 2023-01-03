use crate::compiler::error::Span;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TypeExpr<'file> {
    Bit(Span<'file>),
    Product { obrack: Span<'file>, types: Vec<TypeExpr<'file>>, cbrack: Span<'file> },
    RepProduct { obrack: Span<'file>, num: (Span<'file>, usize), cbrack: Span<'file>, type_: Box<TypeExpr<'file>> },
    NamedProduct { obrack: Span<'file>, named: Span<'file>, types: Vec<((Span<'file>, &'file str), TypeExpr<'file>)>, cbrack: Span<'file> },
    Named(Span<'file>, &'file str),
}

impl<'file> TypeExpr<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            TypeExpr::Bit(sp) | TypeExpr::Named(sp, _) => *sp,
            TypeExpr::RepProduct { obrack, num: _, cbrack: _, type_ } => *obrack + type_.span(),
            TypeExpr::Product { obrack, types: _, cbrack } | TypeExpr::NamedProduct { obrack, named: _, types: _, cbrack } => *obrack + *cbrack,
        }
    }
}
