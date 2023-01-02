use super::ty;
use crate::compiler::error::Span;

pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, TypeInfo> {
    pub(crate) name: (Span<'file>, &'file str),

    pub(crate) input_type: TypeInfo,
    pub(crate) output_type: TypeInfo,

    pub(crate) gates: Vec<GateInstance<'file>>,
    pub(crate) connections: Vec<Connection<'file>>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct GateInstance<'file> {
    pub(crate) local_name: (Span<'file>, &'file str),
    pub(crate) gate_name: (Span<'file>, &'file str),
}
#[derive(PartialEq, Debug)]
pub(crate) struct Connection<'file> {
    pub(crate) arrow_span: Span<'file>,
    pub(crate) producer: Expr<'file>,
    pub(crate) receiver: Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum Expr<'file> {
    Ref(Span<'file>, &'file str),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file>>, (Span<'file>, &'file str)),
    Multiple { obrack: Span<'file>, exprs: Vec<Expr<'file>>, cbrack: Span<'file> }, // TODO: named product expressions
}

impl<'file> Expr<'file> {
    pub(crate) fn span(&self) -> Span<'file> {
        match self {
            Expr::Ref(sp, _) => *sp,
            Expr::Const(sp, _) => *sp,
            Expr::Get(expr, (field_sp, _)) => expr.span() + *field_sp,
            Expr::Multiple { obrack, cbrack, exprs: _ } => *obrack + *cbrack,
        }
    }
}
