use crate::compiler::error::Span;

// TODO: make enums into Thing and ThingKind
#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file>,
    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) output: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file> {
    pub(crate) pat: Pattern<'file>,
    pub(crate) val: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Expr<'file> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, Box<Expr<'file>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file>>, (Span<'file>, &'file str)),
    Multiple(Span<'file>, Vec<Expr<'file>>),
}

#[derive(PartialEq, Debug)]
pub(crate) enum Pattern<'file> {
    Identifier((Span<'file>, &'file str), Type<'file>),
    Product(Span<'file>, Vec<Pattern<'file>>),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type<'file> {
    Bit(Span<'file>),
    Product { obrack: Span<'file>, types: Vec<Type<'file>>, cbrack: Span<'file> }, // TODO: named product types
    RepProduct { obrack: Span<'file>, num: (Span<'file>, usize), cbrack: Span<'file>, type_: Box<Type<'file>> }
}

impl<'file> Type<'file> {
    fn span(&self) -> Span<'file> {
        match self {
            Type::Bit(sp) => *sp,
            Type::Product { obrack, types: _, cbrack } => *obrack + *cbrack,
            Type::RepProduct { obrack, num: _, cbrack: _, type_ } => *obrack + type_.span(),
        }
    }
}

impl<'file> Pattern<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Pattern::Identifier((sp, _), ty) => *sp + ty.span(),
            Pattern::Product(sp, _) => *sp,
        }
    }
}
impl<'file> Expr<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Expr::Ref(sp, _) => *sp,
            Expr::Call((circuit_name_sp, _), _, arg) => *circuit_name_sp + arg.span(),
            Expr::Const(sp, _) => *sp,
            Expr::Get(expr, (field_sp, _)) => expr.span() + *field_sp,
            Expr::Multiple(sp, _) => *sp,
        }
    }
}
