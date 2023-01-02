use super::{parser::ast, ty};
use crate::compiler::error::Span;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, Pattern, Expr> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern,
    pub(crate) lets: Vec<Let<Pattern, Expr>>,
    pub(crate) output: Expr,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<Pattern, Expr> {
    pub(crate) pat: Pattern,
    pub(crate) val: Expr,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file, TypeInfo> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, Box<Expr<'file, TypeInfo>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, TypeInfo>>, (Span<'file>, &'file str)),
    Multiple(Span<'file>, Vec<Expr<'file, TypeInfo>>),
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

impl<'file, TypeInfo> ExprKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            ExprKind::Ref(sp, _) => *sp,
            ExprKind::Call((circuit_name_sp, _), _, arg) => *circuit_name_sp + arg.kind.span(),
            ExprKind::Const(sp, _) => *sp,
            ExprKind::Get(expr, (field_sp, _)) => expr.kind.span() + *field_sp,
            ExprKind::Multiple(sp, _) => *sp,
        }
    }
}
