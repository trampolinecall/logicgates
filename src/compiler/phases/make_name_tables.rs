use crate::{
    compiler::{
        data::{ast, nominal_type, ty, token},
        error::{CompileError, Report},
        phases::parser,
    },
    utils::arena,
};

use std::collections::HashMap;

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ast::UntypedCircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, ast::CircuitOrIntrinsicId>,

    pub(crate) type_context: ty::TypeContext<nominal_type::PartiallyDefinedStruct<'file>>,
    pub(crate) type_table: HashMap<&'file str, ty::TypeSym>,
}

struct DuplicateCircuit<'file, 'tok>(&'tok token::CircuitIdentifier<'file>); // TODO: show previous declaration
struct DuplicateType<'file, 'tok>(&'tok token::TypeIdentifier<'file>); // TODO: show previous declaration
impl<'file> From<DuplicateCircuit<'file, '_>> for CompileError<'file> {
    fn from(DuplicateCircuit(i): DuplicateCircuit<'file, '_>) -> Self {
        CompileError::new(i.span, format!("circuit '{}' defined more than once", i.with_tag))
    }
}
impl<'file> From<DuplicateType<'file, '_>> for CompileError<'file> {
    fn from(DuplicateType(i): DuplicateType<'file, '_>) -> Self {
        CompileError::new(i.span, format!("type '{}' defined more than once", i.with_tag))
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
fn circuit_intrinsics(arena: &mut arena::Arena<ast::UntypedCircuitOrIntrinsic, ast::CircuitOrIntrinsicId>, table: &mut HashMap<&str, ast::CircuitOrIntrinsicId>) {
    intrinsic(table, "nand", arena.add(ast::UntypedCircuitOrIntrinsic::Nand))
}
fn type_intrinsics(context: &mut ty::TypeContext<nominal_type::PartiallyDefinedStruct>, table: &mut HashMap<&str, ty::TypeSym>) {
    intrinsic(table, "bit", context.intern(ty::Type::Bit))
}

fn make_circuit_table(
    circuits: Vec<ast::UntypedCircuit>,
) -> Option<(arena::Arena<ast::UntypedCircuitOrIntrinsic, ast::CircuitOrIntrinsicId>, HashMap<&str, ast::CircuitOrIntrinsicId>)> {
    let mut arena = arena::Arena::new();
    let mut table = HashMap::new();

    circuit_intrinsics(&mut arena, &mut table);

    let mut errored = false;
    for circuit in circuits {
        if table.contains_key(circuit.name.name) {
            DuplicateCircuit(&circuit.name).report();
            errored = true;
        }
        table.insert(circuit.name.name, arena.add(ast::UntypedCircuitOrIntrinsic::Circuit(circuit)));
    }

    if errored {
        None
    } else {
        Some((arena, table))
    }
}

fn make_type_table(type_decls: Vec<nominal_type::PartiallyDefinedStruct>) -> Option<(ty::TypeContext<nominal_type::PartiallyDefinedStruct>, HashMap<&str, ty::TypeSym>)> {
    let mut type_context = ty::TypeContext::new();
    let mut type_table = HashMap::new();

    type_intrinsics(&mut type_context, &mut type_table);

    let mut errored = false;
    for decl in type_decls {
        if type_table.contains_key(decl.name.name) {
            DuplicateType(&decl.name).report();
            errored = true;
        }
        let name = decl.name.name;
        let nominal_index = type_context.structs.add(decl);
        type_table.insert(name, type_context.intern(ty::Type::Nominal(nominal_index)));
    }

    if errored {
        None
    } else {
        Some((type_context, type_table))
    }
}
