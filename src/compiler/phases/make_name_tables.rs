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

struct Duplicate<'file>(&'static str, Span<'file>, &'file str); // TODO: show previous declaration
impl<'file> From<Duplicate<'file>> for CompileError<'file> {
    fn from(Duplicate(thing, name_sp, name): Duplicate<'file>) -> Self {
        CompileError::new(name_sp, format!("{} '{}' defined more than once", thing, name))
    }
}

pub(crate) fn make(ast: parser::AST) -> Option<IR> {
    let circuits = make_circuit_table(ast.circuits);
    let types = make_type_table(ast.type_decls);
    let (circuits, circuit_table) = circuits?;
    let (type_context, type_table) = types?;
    Some(IR { circuits, circuit_table, type_context, type_table })
}

fn intrinsic<'a, T>(table: &mut HashMap<&'a str, T>, name: &'a str, thing: T) {
    let old_t = table.insert(name, thing);
    assert!(old_t.is_none(), "cannot have other item named '{}' in empty table", name);
}
fn circuit_intrinsics(arena: &mut arena::Arena<circuit1::UntypedCircuitOrIntrinsic, circuit1::CircuitOrIntrinsicId>, table: &mut HashMap<&str, circuit1::CircuitOrIntrinsicId>) {
    intrinsic(table, "nand", arena.add(circuit1::UntypedCircuitOrIntrinsic::Nand))
}
fn type_intrinsics(context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct>, table: &mut HashMap<&str, ty::TypeSym>) {
    intrinsic(table, "bit", context.intern(ty::Type::Bit))
}

fn make_circuit_table(
    circuits: Vec<circuit1::UntypedCircuit>,
) -> Option<(arena::Arena<circuit1::UntypedCircuitOrIntrinsic, circuit1::CircuitOrIntrinsicId>, HashMap<&str, circuit1::CircuitOrIntrinsicId>)> {
    let mut arena = arena::Arena::new();
    let mut table = HashMap::new();

    circuit_intrinsics(&mut arena, &mut table);

    let mut errored = false;
    for circuit in circuits {
        if table.contains_key(circuit.name.1) {
            Duplicate("circuit", circuit.name.0, circuit.name.1).report();
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
    let mut type_context = ty::TypeContext::new();
    let mut type_table = HashMap::new();

    type_intrinsics(&mut type_context, &mut type_table);

    let mut errored = false;
    for decl in type_decls {
        if type_table.contains_key(decl.name.1) {
            Duplicate("nominal type", decl.name.0, decl.name.1).report();
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
