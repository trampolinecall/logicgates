use crate::compiler::{error::Span, ir};

// TODO: make enums into Thing and ThingKind
pub(crate) type Circuit<'file> = ir::Circuit<'file, Pattern<'file>>;
pub(crate) type Let<'file> = ir::Let<'file, Pattern<'file>>;
pub(crate) type Pattern<'file> = ir::Pattern<'file, ()>;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type<'file> {
    Bit(Span<'file>),
    Product { obrack: Span<'file>, types: Vec<Type<'file>>, cbrack: Span<'file> }, // TODO: named product types
    RepProduct { obrack: Span<'file>, num: (Span<'file>, usize), cbrack: Span<'file>, type_: Box<Type<'file>> },
    NamedProduct { obrack: Span<'file>, named: Span<'file>, types: Vec<((Span<'file>, &'file str), Type<'file>)>, cbrack: Span<'file> },
}

impl<'file> Type<'file> {
    fn span(&self) -> Span<'file> {
        match self {
            Type::Bit(sp) => *sp,
            Type::Product { obrack, types: _, cbrack } => *obrack + *cbrack,
            Type::RepProduct { obrack, num: _, cbrack: _, type_ } => *obrack + type_.span(),
            Type::NamedProduct { obrack, named: _, types: _, cbrack } => *obrack + *cbrack,
        }
    }
}
