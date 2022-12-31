#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file> {
    pub(crate) name: &'file str,
    pub(crate) inputs: Vec<(Pattern<'file>, Type)>,
    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) outputs: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file> {
    pub(crate) pat: Pattern<'file>,
    pub(crate) type_: Type,
    pub(crate) val: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Expr<'file> {
    Ref(&'file str),
    Call(&'file str, bool, Box<Expr<'file>>),
    Const(bool),
    Get(Box<Expr<'file>>, usize),
    Multiple(Vec<Expr<'file>>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file>(pub(crate) &'file str);

#[derive(PartialEq, Debug)]
pub(crate) enum Type {
    Bit,
    Array(usize, Box<Type>),
}

impl Type {
    fn size(&self) -> usize {
        match self {
            Type::Bit => 1,
            Type::Array(len, item_type) => item_type.size() * len
        }
    }
}
