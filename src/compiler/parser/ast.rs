#[derive(PartialEq, Debug)]
pub(crate) struct Gate<'file> {
    pub(crate) name: &'file str,
    pub(crate) arguments: Vec<Pattern<'file>>,
    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) ret: Vec<Expr<'file>>,
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
