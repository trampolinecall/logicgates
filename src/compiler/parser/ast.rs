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
    Identifier((Span<'file>, &'file str), LType<'file>),
    Product(Span<'file>, Vec<Pattern<'file>>),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) struct LType<'file>(pub(crate) Span<'file>, pub(crate) Type); // TODO: turn this into ast::Type
#[derive(PartialEq, Debug, Clone)]
pub(crate) enum Type {
    // TODO: turn this into ir::Type
    Bit,
    Product(Vec<Type>), // TODO: named product types
}

impl<'file> Pattern<'file> {
    pub(crate) fn type_(&self) -> Type {
        match self {
            Pattern::Identifier(_, ty) => ty.1.clone(),
            Pattern::Product(_, pats) => Type::Product(pats.iter().map(Pattern::type_).collect()),
        }
    }

    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Pattern::Identifier((sp, _), ty) => *sp + ty.span(),
            Pattern::Product(sp, _) => *sp,
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
impl<'file> LType<'file> {
    fn span(&self) -> Span<'file> {
        self.0
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
