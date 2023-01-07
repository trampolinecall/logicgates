use crate::compiler::data::ty;
use crate::compiler::error::Span;
use crate::utils::arena;

use super::{nominal_type, type_expr};

// TODO: separate ast from this?

pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedLet<'file> = Let<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;

pub(crate) type TypeResolvedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedLet<'file> = Let<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedPattern<'file> = Pattern<'file, (), (Span<'file>, ty::TypeSym)>;

pub(crate) type PatTypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedLet<'file> = Let<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedLet<'file> = Let<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedExpr<'file> = Expr<'file, ty::TypeSym>;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOrIntrinsicId(usize); // not ideal because this is also the id for circuit2::CircuitOrIntrinsic but i dont know where else to put it
impl arena::ArenaId for CircuitOrIntrinsicId {
    fn make(i: usize) -> Self {
        CircuitOrIntrinsicId(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}
impl<'file, PatTypeInfo, ExprTypeInfo, TypeExpr> arena::IsArenaIdFor<CircuitOrIntrinsic<'file, PatTypeInfo, ExprTypeInfo, TypeExpr>> for CircuitOrIntrinsicId {}

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, Expr, PatTypeInfo, TypeExpr> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) output_type: TypeExpr,
    pub(crate) lets: Vec<Let<'file, Expr, PatTypeInfo, TypeExpr>>,
    pub(crate) output: Expr,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, Expr, PatTypeInfo, TypeExpr> {
    Circuit(Circuit<'file, Expr, PatTypeInfo, TypeExpr>),
    Nand,
    Const(bool), // never in circuit table
}

// TODO: remove all span methods
#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, Expr, PatTypeInfo, TypeExpr> {
    pub(crate) pat: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) val: Expr,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, TypeInfo> {
    pub(crate) kind: ExprKind<'file, TypeInfo>,
    pub(crate) type_info: TypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file, TypeInfo> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, Box<Expr<'file, TypeInfo>>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, TypeInfo>>, (Span<'file>, &'file str)),
    Multiple(Vec<Expr<'file, TypeInfo>>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, PatTypeInfo, TypeExpr> {
    pub(crate) kind: PatternKind<'file, PatTypeInfo, TypeExpr>,
    pub(crate) type_info: PatTypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, PatTypeInfo, TypeExpr> {
    Identifier(Span<'file>, &'file str, TypeExpr),
    Product(Span<'file>, Vec<Pattern<'file, PatTypeInfo, TypeExpr>>),
}

// TODO: this will probably be duplicated with the type code from circuit2 but i dont know how to fix that (although i think the solution might be a separate type checking phase so that circuit2 doesnt need to have type information)
impl<'file, ExprTypeInfo, TypeExpr> CircuitOrIntrinsic<'file, ExprTypeInfo, ty::TypeSym, TypeExpr> {
    pub(crate) fn input_type(&self, type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>) -> ty::TypeSym {
        match self {
            CircuitOrIntrinsic::Circuit(circuit) => circuit.input.type_info,
            CircuitOrIntrinsic::Nand => {
                let b = type_context.intern(ty::Type::Bit);
                type_context.intern(ty::Type::Product(vec![("0".into(), b), ("1".into(), b)]))
            }
            CircuitOrIntrinsic::Const(_) => type_context.intern(ty::Type::Product(vec![])),
        }
    }
}

impl<'file, ExprTypeInfo, PatTypeInfo> CircuitOrIntrinsic<'file, ExprTypeInfo, PatTypeInfo, (Span<'file>, ty::TypeSym)> {
    pub(crate) fn output_type(&self, type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>) -> ty::TypeSym {
        match self {
            CircuitOrIntrinsic::Circuit(circuit) => circuit.output_type.1,
            CircuitOrIntrinsic::Nand | CircuitOrIntrinsic::Const(_) => type_context.intern(ty::Type::Bit),
        }
    }
}
