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
    Call(&'file str, bool, Vec<Expr<'file>>),
    Const(bool),
    Get(Box<Expr<'file>>, &'file str),
    Multiple(Vec<Expr<'file>>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file>(pub(crate) &'file str);

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type {
    Bit,
    Product(Vec<Type>), // TODO: named product types
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
