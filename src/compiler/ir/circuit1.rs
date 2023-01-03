use crate::compiler::error::Span;

use super::{ty, type_expr::TypeExpr};

pub(crate) type UntypedCircuit<'file> = Circuit<'file, ()>;
pub(crate) type UntypedLet<'file> = Let<'file, ()>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, ()>;

pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, TypeInfo>,
    pub(crate) output_type_annotation: TypeExpr<'file>, // TODO: probably make TypeInfo TypeExpr instead of ()
    pub(crate) output_type: TypeInfo,
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
    Identifier(Span<'file>, &'file str, TypeExpr<'file>),
    Product(Span<'file>, Vec<Pattern<'file, TypeInfo>>),
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

impl<'file, TypeInfo> PatternKind<'file, TypeInfo> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            PatternKind::Identifier(sp, _, ty) => *sp + ty.span(),
            PatternKind::Product(sp, _) => *sp,
        }
    }
}
