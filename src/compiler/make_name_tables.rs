use std::collections::HashMap;

use super::{
    arena,
    error::{CompileError, Report, Span},
    ir::{circuit1, named_type},
    parser,
};

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOrIntrinsicId(usize);
impl<'file> arena::ArenaId for CircuitOrIntrinsicId { // TODO: move to different module that makes more sense
    fn make(i: usize) -> Self {
        todo!()
    }

    fn get(&self) -> usize {
        todo!()
    }
}
impl<'file, TypeInfo, TypeExpr> arena::IsArenaIdFor<circuit1::CircuitOrIntrinsic<'file, TypeInfo, TypeExpr>> for CircuitOrIntrinsicId {}
#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub(crate) struct TypeDeclId(usize); // TODO: also move to different module that makes more sense
impl<'file> arena::ArenaId for TypeDeclId {
    fn make(i: usize) -> Self {
        todo!()
    }

    fn get(&self) -> usize {
        todo!()
    }
}
impl<'file> arena::IsArenaIdFor<named_type::NamedTypeDecl<'file>> for TypeDeclId {}
pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::UntypedCircuitOrIntrinsic<'file>, CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, CircuitOrIntrinsicId>,

    pub(crate) type_decls: arena::Arena<named_type::NamedTypeDecl<'file>, TypeDeclId>,
    pub(crate) type_table: HashMap<String, TypeDeclId>,
}

struct Duplicate<'file>(&'static str, Span<'file>, &'file str); // TODO: show previous declaration
impl<'file> From<Duplicate<'file>> for CompileError<'file> {
    fn from(Duplicate(thing, name_sp, name): Duplicate<'file>) -> Self {
        CompileError::new(name_sp, format!("{} '{}' defined more than once", thing, name))
    }
}

pub(crate) fn make(ast: parser::AST) -> Option<IR> {
    let circuits = make_circuit_table(ast.circuits);
    let types = make_type_table(ast.type_decls);
    let circuits = circuits?;
    let types = types?;
    Some(IR { circuits: circuits.0, circuit_table: circuits.1, type_decls: types.0, type_table: types.1 })
}

fn make_circuit_table(circuits: Vec<circuit1::UntypedCircuit>) -> Option<(arena::Arena<circuit1::UntypedCircuitOrIntrinsic, CircuitOrIntrinsicId>, HashMap<String, CircuitOrIntrinsicId>)> {
    let mut arena = arena::Arena::new();
    let mut table = HashMap::new();
    table.insert("nand".into(), arena.add(circuit1::UntypedCircuitOrIntrinsic::Nand));

    let mut errored = false;
    for circuit in circuits {
        if table.contains_key(circuit.name.1) {
            Duplicate("circuit", circuit.name.0, circuit.name.1).report();
            errored = true;
        }
        table.insert(circuit.name.1.into(), arena.add(circuit1::UntypedCircuitOrIntrinsic::Circuit(circuit)));
    }

    if errored {
        None
    } else {
        Some((arena, table))
    }
}

fn make_type_table(type_decls: Vec<super::ir::named_type::NamedTypeDecl>) -> Option<(arena::Arena<named_type::NamedTypeDecl, TypeDeclId>, HashMap<String, TypeDeclId>)> {
    let mut type_table = HashMap::new();
    let mut type_decl_arena = arena::Arena::new();
    let mut errored = false;
    for decl in type_decls {
        if type_table.contains_key(decl.name.1) {
            Duplicate("named type", decl.name.0, decl.name.1).report();
            errored = true;
        }
        type_table.insert(decl.name.1.into(), type_decl_arena.add(decl));
    }

    if errored {
        None
    } else {
        Some((type_decl_arena, type_table))
    }
}
