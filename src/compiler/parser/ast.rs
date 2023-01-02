use crate::compiler::{error::Span, ir};

pub(crate) type CircuitAST<'file> = ir::Circuit<'file, PatternAST<'file>>;
pub(crate) type LetAST<'file> = ir::Let<'file, PatternAST<'file>>;
pub(crate) type PatternAST<'file> = ir::Pattern<'file, ()>;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TypeAST<'file> {
    Bit(Span<'file>),
    Product { obrack: Span<'file>, types: Vec<TypeAST<'file>>, cbrack: Span<'file> }, // TODO: named product types
    RepProduct { obrack: Span<'file>, num: (Span<'file>, usize), cbrack: Span<'file>, type_: Box<TypeAST<'file>> },
    NamedProduct { obrack: Span<'file>, named: Span<'file>, types: Vec<((Span<'file>, &'file str), TypeAST<'file>)>, cbrack: Span<'file> },
}

impl<'file> TypeAST<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            TypeAST::Bit(sp) => *sp,
            TypeAST::Product { obrack, types: _, cbrack } => *obrack + *cbrack,
            TypeAST::RepProduct { obrack, num: _, cbrack: _, type_ } => *obrack + type_.span(),
            TypeAST::NamedProduct { obrack, named: _, types: _, cbrack } => *obrack + *cbrack,
        }
    }
}
