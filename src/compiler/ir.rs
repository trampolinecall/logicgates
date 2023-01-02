use super::parser::ast;
use crate::compiler::error::Span;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, Pattern> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern,
    pub(crate) lets: Vec<Let<'file, Pattern>>,
    pub(crate) output: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, Pattern> {
    pub(crate) pat: Pattern,
    pub(crate) val: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Expr<'file> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, Box<Expr<'file>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file>>, (Span<'file>, &'file str)),
    Multiple { obrack: Span<'file>, exprs: Vec<Expr<'file>>, cbrack: Span<'file> },
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, TypeInfo> {
    pub(crate) kind: PatternKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, TypeInfo> {
    Identifier(Span<'file>, &'file str, ast::Type<'file>),
    Product(Span<'file>, Vec<Pattern<'file, TypeInfo>>),
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, _) => *sp,
            PatternKind::Product(sp, _) => *sp,
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
            Expr::Multiple { obrack, cbrack, exprs: _ } => *obrack + *cbrack,
        }
    }
}
