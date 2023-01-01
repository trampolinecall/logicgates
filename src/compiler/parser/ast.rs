#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file> {
    pub(crate) name: &'file str,
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
    Ref(&'file str),
    Call(&'file str, bool, Box<Expr<'file>>),
    Const(bool),
    Get(Box<Expr<'file>>, &'file str),
    Multiple(Vec<Expr<'file>>),
}

#[derive(PartialEq, Debug)]
pub(crate) enum Pattern<'file> {
    Identifier( &'file str, Type),
    Product(Vec<Pattern<'file>>),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<Type>), // TODO: named product types
}

impl Pattern<'_> {
    pub(crate) fn type_(&self) -> Type {
        match self {
            Pattern::Identifier(_, ty) => ty.clone(),
            Pattern::Product(pats) => Type::Product(pats.iter().map(Pattern::type_).collect()),
        }
    }
}
impl Type {
    pub(crate) fn size(&self) -> usize {
        match self {
            Type::Bit => 1,
            Type::Product(items) => items.iter().map(Type::size).sum(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Bit => write!(f, "`"),
            Type::Product(items) => {
                write!(f, "[")?;
                if let Some((first, more)) = items.split_first() {
                    write!(f, "{first}")?;
                    for more in more {
                        write!(f, ", {more}")?;
                    }
                }
                write!(f, "]")?;

                Ok(())
            }
        }
    }
}
