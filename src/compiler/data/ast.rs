use crate::{
    compiler::{
        data::{nominal_type, token, ty, type_expr},
        error::Span,
    },
    utils::arena,
};

pub(crate) type UntypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedAlias<'file> = Alias<'file, UntypedExpr<'file>, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedConnect<'file> = Connect<UntypedExpr<'file>>;
pub(crate) type UntypedLet<'file> = Let<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPattern<'file> = Pattern<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedPatternKind<'file> = PatternKind<'file, (), type_expr::TypeExpr<'file>>;
pub(crate) type UntypedExpr<'file> = Expr<'file, ()>;
pub(crate) type UntypedExprKind<'file> = ExprKind<'file, ()>;

pub(crate) type TypeResolvedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedAlias<'file> = Alias<'file, UntypedExpr<'file>, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedConnect<'file> = Connect<UntypedExpr<'file>>;
pub(crate) type TypeResolvedLet<'file> = Let<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedPattern<'file> = Pattern<'file, (), (Span<'file>, ty::TypeSym)>;
pub(crate) type TypeResolvedPatternKind<'file> = PatternKind<'file, (), (Span<'file>, ty::TypeSym)>;

pub(crate) type PatTypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedCircuit<'file> = Circuit<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedAlias<'file> = Alias<'file, UntypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedConnect<'file> = Connect<UntypedExpr<'file>>;
pub(crate) type PatTypedLet<'file> = Let<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type PatTypedPatternKind<'file> = PatternKind<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;

pub(crate) type TypedCircuitOrIntrinsic<'file> = CircuitOrIntrinsic<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedCircuit<'file> = Circuit<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedAlias<'file> = Alias<'file, TypedExpr<'file>, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedConnect<'file> = Connect<TypedExpr<'file>>;
pub(crate) type TypedLet<'file> = Let<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPattern<'file> = Pattern<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
pub(crate) type TypedPatternKind<'file> = PatternKind<'file, ty::TypeSym, (Span<'file>, ty::TypeSym)>;
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
pub(crate) struct Circuit<'file, Expr, PatTypeInfo, TypeExpr> {
    pub(crate) name: token::CircuitIdentifier<'file>,

    pub(crate) input: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) output: Pattern<'file, PatTypeInfo, TypeExpr>,

    pub(crate) lets: Vec<Let<'file, PatTypeInfo, TypeExpr>>,
    pub(crate) aliases: Vec<Alias<'file, Expr, PatTypeInfo, TypeExpr>>,
    pub(crate) connects: Vec<Connect<Expr>>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, Expr, PatTypeInfo, TypeExpr> {
    Circuit(Circuit<'file, Expr, PatTypeInfo, TypeExpr>),
    Nand,
    Const(bool), // never in circuit table
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, PatTypeInfo, TypeExpr> {
    pub(crate) inputs: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) outputs: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) gate: token::CircuitIdentifier<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) struct Alias<'file, Expr, PatTypeInfo, TypeExpr> {
    pub(crate) pat: Pattern<'file, PatTypeInfo, TypeExpr>,
    pub(crate) expr: Expr,
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
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, TypeInfo>>, (Span<'file>, &'file str)),
    Product(Vec<(String, Expr<'file, TypeInfo>)>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, PatTypeInfo, TypeExpr> {
    pub(crate) kind: PatternKind<'file, PatTypeInfo, TypeExpr>,
    pub(crate) type_info: PatTypeInfo,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, PatTypeInfo, TypeExpr> {
    Identifier(token::PlainIdentifier<'file>, TypeExpr),
    Product(Vec<(String, Pattern<'file, PatTypeInfo, TypeExpr>)>),
}

// TODO: this will probably be duplicated with the type code from the ir but i dont know how to fix that (although i think the solution might be a separate type checking phase so that the ir doesnt need to have type information)
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
    pub(crate) fn output_type(&self, type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>) -> ty::TypeSym {
        match self {
            CircuitOrIntrinsic::Circuit(circuit) => circuit.output.type_info,
            CircuitOrIntrinsic::Nand | CircuitOrIntrinsic::Const(_) => type_context.intern(ty::Type::Bit),
        }
    }
}
