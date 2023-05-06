use crate::{
    compiler::{
        data::{nominal_type, token, ty, type_expr},
        error::Span,
    },
    utils::arena,
};

pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, type_expr::TypeExpr<'file>>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, type_expr::TypeExpr<'file>>;
pub(crate) type UntypedLet<'file> = Let<'file>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPatternKind<'file> = PatternKind<'file, type_expr::TypeExpr<'file>>;
pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type UntypedExprKind<'file> = ExprKind<'file, ()>;

pub(crate) type TypeResolvedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedLet<'file> = Let<'file>;
pub(crate) type TypeResolvedPattern<'file> = Pattern<'file, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedPatternKind<'file> = PatternKind<'file, (Span<'file>, ty::TypeSym)>;

pub(crate) type PatTypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedLet<'file> = Let<'file>;
pub(crate) type PatTypedPattern<'file> = Pattern<'file, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedPatternKind<'file> = PatternKind<'file, (Span<'file>, ty::TypeSym)>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, TypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, TypedExpr<'file>, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedLet<'file> = Let<'file>;
pub(crate) type TypedPattern<'file> = Pattern<'file, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPatternKind<'file> = PatternKind<'file, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedExpr<'file> = Expr<'file, ty::TypeSym>;
pub(crate) type TypedExprKind<'file> = ExprKind<'file, ty::TypeSym>;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOrIntrinsicId(usize); // not ideal because this is also the id for ir::CircuitOrIntrinsic but i dont know where else to put it
impl arena::ArenaId for CircuitOrIntrinsicId {
    fn make(i: usize) -> Self {
        CircuitOrIntrinsicId(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, Expr, TypeExpr> {
    pub(crate) name: token::CircuitIdentifier<'file>,

    pub(crate) input_type: TypeExpr,
    pub(crate) output_type: TypeExpr,

    pub(crate) lets: Vec<Let<'file>>,
    pub(crate) connects: Vec<Connect<Expr>>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, Expr, TypeExpr> {
    Circuit(Circuit<'file, Expr, TypeExpr>),
    Nand,
    Const(bool), // never in circuit table
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file> {
    pub(crate) name: token::PlainIdentifier<'file>,
    pub(crate) gate: token::CircuitIdentifier<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) struct Connect<Expr> {
    pub(crate) start: Expr,
    pub(crate) end: Expr,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file, TypeInfo> {
    Ref(token::PlainIdentifier<'file>),
    Call(token::CircuitIdentifier<'file>, bool, Box<Expr<'file, TypeInfo>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, TypeInfo>>, (Span<'file>, &'file str)),
    Product(Vec<(String, Expr<'file, TypeInfo>)>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, TypeExpr> {
    pub(crate) kind: PatternKind<'file, TypeExpr>,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, TypeExpr> {
    Identifier(token::PlainIdentifier<'file>, TypeExpr),
    Product(Vec<(String, Pattern<'file, TypeExpr>)>),
}
