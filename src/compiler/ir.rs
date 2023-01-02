use super::{parser::ast, ty};
use crate::compiler::error::Span;

pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, TypeInfo>,
    pub(crate) lets: Vec<Let<'file, TypeInfo>>,
    pub(crate) output: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, TypeInfo> {
    pub(crate) pat: Pattern<'file, TypeInfo>,
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
    Identifier(Span<'file>, &'file str, ast::TypeAST<'file>),
    Product(Span<'file>, Vec<Pattern<'file, TypeInfo>>),
}

impl<'file, TypeInfo> PatternKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, ty) => *sp + ty.span(),
            PatternKind::Product(sp, _) => *sp,
        }
    }
}

impl<'file> Expr<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Expr::Ref(sp, _) | Expr::Const(sp, _) => *sp,
            Expr::Call((circuit_name_sp, _), _, arg) => *circuit_name_sp + arg.span(),
            Expr::Get(expr, (field_sp, _)) => expr.span() + *field_sp,
            Expr::Multiple { obrack, cbrack, exprs: _ } => *obrack + *cbrack,
        }
    }
}
