use crate::compiler::{error::Span, ir};

pub(crate) type CircuitAST<'file> = ir::Circuit<'file, TypeAST<'file>>;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TypeAST<'file> {
    Bit(Span<'file>),
    Product { obrack: Span<'file>, types: Vec<TypeAST<'file>>, cbrack: Span<'file> }, // TODO: named product types
    RepProduct { obrack: Span<'file>, num: (Span<'file>, usize), cbrack: Span<'file>, type_: Box<TypeAST<'file>> },
    NamedProduct { obrack: Span<'file>, named: Span<'file>, types: Vec<((Span<'file>, &'file str), TypeAST<'file>)>, cbrack: Span<'file> },
}
