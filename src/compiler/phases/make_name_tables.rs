use crate::{
    compiler::{
        data::{circuit1, nominal_type, ty},
        error::{CompileError, Report, Span},
        phases::parser,
    },
    utils::arena,
};

use std::collections::HashMap;

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit1::UntypedCircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, circuit1::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    pub(crate) type_table: HashMap<&'file str, ty::TypeSym>,
}

struct Duplicate<'file>(&'static str, Span<'file>, &'file str, Span<'file>);
impl<'file> From<Duplicate<'file>> for CompileError<'file> {
    fn from(Duplicate(thing, name_sp, name, prev_sp): Duplicate<'file>) -> Self {
        CompileError::new(name_sp, format!("{} '{}' defined more than once", thing, name))
            .note(prev_sp, "previous definition here".into())
    }
}

pub(crate) fn make(ast: parser::AST) -> Option<IR> {
    let circuits = make_circuit_table(ast.circuits);
    let types = make_type_table(ast.type_decls);
    let (circuits, circuit_table) = circuits?;
    let (type_context, type_table) = types?;
    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn make_circuit_table(
    circuits: Vec<circuit1::UntypedCircuit>,
) -> Option<(arena::Arena<circuit1::UntypedCircuitOrIntrinsic, circuit1::CircuitOrIntrinsicId>, HashMap<&str, circuit1::CircuitOrIntrinsicId>)> {
    let mut arena = arena::Arena::new();
    let mut table = HashMap::new();
    table.insert("nand", arena.add(circuit1::UntypedCircuitOrIntrinsic::Nand));

    let mut errored = false;
    for circuit in circuits {
        if let Some(old_circuit) = table.get(circuit.name.1) {
            Duplicate("circuit", circuit.name.0, circuit.name.1, arena.get(old_circuit).name.0).report();
            errored = true;
        }
        table.insert(circuit.name.1, arena.add(circuit1::UntypedCircuitOrIntrinsic::Circuit(circuit)));
    }

    if errored {
        None
    } else {
        Some((arena, table))
    }
}

fn make_type_table(type_decls: Vec<crate::compiler::data::nominal_type::PartiallyDefinedStruct>) -> Option<(ty::TypeContext<nominal_type::PartiallyDefinedStruct>, HashMap<&str, ty::TypeSym>)> {
    let mut type_table = HashMap::new();
    let mut type_context = ty::TypeContext::new();
    let mut errored = false;
    for decl in type_decls {
        if let Some(old_type) = type_table.get(decl.name.1) {
            Duplicate("nominal type", decl.name.0, decl.name.1, decl.name.0 /* TODO: this should be the previous span */).report();
            errored = true;
        }
        let name = decl.name.1;
        let nominal_index = type_context.structs.add(decl);
        type_table.insert(name, type_context.intern(ty::Type::Nominal(nominal_index)));
    }

    if errored {
        None
    } else {
        Some((type_context, type_table))
    }
}
