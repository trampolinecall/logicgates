pub(crate) mod expr;

use crate::compiler::error::Span;

use super::{named_type, ty, type_expr};

// TODO: separate ast from this?

// TODO: move Circuit and CircuitOrIntrinsic into separate module
pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, (), (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, (), (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedLet<'file> = Let<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, (), type_expr::TypeExpr<'file>>;

pub(crate) type TypeResolvedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, (), (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedCircuit<'file> = Circuit<'file, (), (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedLet<'file> = Let<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedPattern<'file> = Pattern<'file, (), (Span<'file>, ty::TypeSym)>;

pub(crate) type PatTypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, (), ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedCircuit<'file> = Circuit<'file, (), ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedLet<'file> = Let<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, ty::TypeSym, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, ty::TypeSym, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;

#[derive(PartialEq, Debug)]
pub(crate) struct Circuit<'file, ExprTypeInfo, PatTypeInfo, TypeExpr> {
    pub(crate) name: (Span<'file>, &'file str),
    pub(crate) input: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) expressions: expr::ExprArena<'file, ExprTypeInfo>,
    pub(crate) output_type: TypeExpr,
    pub(crate) lets: Vec<Let<'file, PatTypeInfo, TypeExpr>>,
    pub(crate) output: expr::ExprId,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, ExprTypeInfo, PatTypeInfo, TypeExpr> {
    Circuit(Circuit<'file, ExprTypeInfo, PatTypeInfo, TypeExpr>),
    Nand,
    Const(bool), // never in circuit table
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, PatTypeInfo, TypeExpr> {
    pub(crate) pat: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) val: expr::ExprId,
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
    pub(crate) fn input_type(&self, type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>) -> ty::TypeSym {
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
    pub(crate) fn output_type(&self, type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>) -> ty::TypeSym {
        match self {
            CircuitOrIntrinsic::Circuit(circuit) => circuit.output_type.1,
            CircuitOrIntrinsic::Nand => type_context.intern(ty::Type::Bit),
            CircuitOrIntrinsic::Const(_) => type_context.intern(ty::Type::Bit),
        }
    }
}
