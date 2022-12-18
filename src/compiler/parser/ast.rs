pub(crate) struct Gate<'file> {
    pub(crate) name: &'file str,
    pub(crate) arguments: Pattern<'file>,
    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) ret: Expr<'file>,
}

pub(crate) struct Let<'file> {
    pub(crate) pat: Pattern<'file>,
    pub(crate) val: Expr<'file>,
}

pub(crate) enum Expr<'file> {
    Ref(&'file str, Vec<usize>),
    Call(&'file str, Vec<Expr<'file>>),
    Const(bool),
}

pub(crate) enum Pattern<'file> {
    Iden(&'file str, usize),
    Multiple(Vec<Pattern<'file>>),
}
