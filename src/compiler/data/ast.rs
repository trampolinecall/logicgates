use crate::{
    compiler::{
        data::{nominal_type, token, ty, type_expr},
        error::Span,
    },
    utils::arena,
};

use std::fmt::Debug;

pub(crate) trait Stage {
    // TODO: have ExprTypeInfo instaed of Expr?
    type Expr<'file>: Debug + PartialEq;
    type TypeExpr<'file>: Debug + PartialEq;
    type PatTypeInfo<'file>: Debug + PartialEq;
    type ExprTypeInfo<'file>: Debug + PartialEq;
}

#[derive(Debug, PartialEq)]
pub(crate) enum Untyped {}
impl Stage for Untyped {
    type Expr<'file> = Expr<'file, Untyped>;
    type TypeExpr<'file> = type_expr::TypeExpr<'file>;
    type PatTypeInfo<'file> = ();
    type ExprTypeInfo<'file> = ();
}

#[derive(Debug, PartialEq)]
pub(crate) enum TypeResolved {}
impl Stage for TypeResolved {
    type Expr<'file> = Expr<'file, TypeResolved>;
    type TypeExpr<'file> = (Span<'file>, ty::TypeSym);
    type PatTypeInfo<'file> = ();
    type ExprTypeInfo<'file> = ();
}

#[derive(Debug, PartialEq)]
pub(crate) enum PatTyped {}
impl Stage for PatTyped {
    type Expr<'file> = Expr<'file, PatTyped>;
    type TypeExpr<'file> = (Span<'file>, ty::TypeSym);
    type PatTypeInfo<'file> = ty::TypeSym;
    type ExprTypeInfo<'file> = ();
}

#[derive(Debug, PartialEq)]
pub(crate) enum Typed {}
impl Stage for Typed {
    type Expr<'file> = Expr<'file, Typed>;
    type TypeExpr<'file> = (Span<'file>, ty::TypeSym);
    type PatTypeInfo<'file> = ty::TypeSym;
    type ExprTypeInfo<'file> = ty::TypeSym;
}

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
pub(crate) struct Circuit<'file, CurStage: Stage> {
    pub(crate) name: token::CircuitIdentifier<'file>,

    pub(crate) input: Pattern<'file, CurStage>,
    pub(crate) output: Pattern<'file, CurStage>,

    pub(crate) lets: Vec<Let<'file, CurStage>>,
    pub(crate) aliases: Vec<Alias<'file, CurStage>>,
    pub(crate) connects: Vec<Connect<'file, CurStage>>,
}

#[derive(PartialEq, Debug)]
pub(crate) enum CircuitOrIntrinsic<'file, CurStage: Stage> {
    Circuit(Circuit<'file, CurStage>),
    Nand,
    Const(bool), // never in circuit table
}

#[derive(PartialEq, Debug)]
pub(crate) struct Let<'file, CurStage: Stage> {
    pub(crate) inputs: Pattern<'file, CurStage>,
    pub(crate) outputs: Pattern<'file, CurStage>,
    pub(crate) gate: token::CircuitIdentifier<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) struct Alias<'file, CurStage: Stage> {
    pub(crate) pat: Pattern<'file, CurStage>,
    pub(crate) expr: CurStage::Expr<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) struct Connect<'file, CurStage: Stage> {
    pub(crate) start: CurStage::Expr<'file>,
    pub(crate) end: CurStage::Expr<'file>,
}

#[derive(PartialEq, Debug)]
pub(crate) struct Expr<'file, CurStage: Stage> {
    pub(crate) kind: ExprKind<'file, CurStage>,
    pub(crate) type_info: CurStage::ExprTypeInfo<'file>,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum ExprKind<'file, CurStage: Stage> {
    Ref(token::PlainIdentifier<'file>),
    Const(Span<'file>, bool),
    Get(Box<Expr<'file, CurStage>>, (Span<'file>, &'file str)),
    Product(Vec<(String, Expr<'file, CurStage>)>),
}

#[derive(PartialEq, Debug)]
pub(crate) struct Pattern<'file, CurStage: Stage> {
    pub(crate) kind: PatternKind<'file, CurStage>,
    pub(crate) type_info: CurStage::PatTypeInfo<'file>,
    pub(crate) span: Span<'file>,
}
#[derive(PartialEq, Debug)]
pub(crate) enum PatternKind<'file, CurStage: Stage> {
    Identifier(token::PlainIdentifier<'file>, CurStage::TypeExpr<'file>),
    Product(Vec<(String, Pattern<'file, CurStage>)>),
}

// TODO: this will probably be duplicated with the type code from the ir but i dont know how to fix that (although i think the solution might be a separate type checking phase so that the ir doesnt need to have type information)
impl<'file, CurStage: Stage<PatTypeInfo<'file> = ty::TypeSym>> CircuitOrIntrinsic<'file, CurStage> {
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
