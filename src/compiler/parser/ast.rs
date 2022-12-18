#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file> {
    pub(crate) name: &'file str,
    pub(crate) inputs: Vec<Pattern<'file>>,
    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) outputs: Vec<Expr<'file>>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file> {
    pub(crate) pat: Vec<Pattern<'file>>,
    pub(crate) val: Vec<Expr<'file>>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Expr<'file> {
    Ref(&'file str, Vec<usize>),
    Call(&'file str, Vec<Expr<'file>>),
    Const(bool),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file>(pub(crate) &'file str, pub(crate) usize);
