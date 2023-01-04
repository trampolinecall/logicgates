use std::collections::HashMap;

use super::{
    error::{CompileError, Report, Span},
    ir::{circuit1, type_decl},
    parser,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: HashMap<String, circuit1::UntypedCircuitOrIntrinsic<'file>>,
    pub(crate) types: HashMap<String, type_decl::TypeDecl<'file>>,
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
    Some(IR { circuits: circuits?, types: types? })
}

fn make_circuit_table(circuits: Vec<circuit1::UntypedCircuit>) -> Option<HashMap<String, circuit1::UntypedCircuitOrIntrinsic>> {
    let mut table = HashMap::new();
    table.insert("nand".into(), circuit1::UntypedCircuitOrIntrinsic::Nand);

    let mut errored = false;
    for circuit in circuits {
        if table.contains_key(circuit.name.1) {
            Duplicate("circuit", circuit.name.0, circuit.name.1).report();
            errored = true;
        }
        table.insert(circuit.name.1.into(), circuit1::UntypedCircuitOrIntrinsic::Circuit(circuit));
    }

    if errored {
        None
    } else {
        Some(table)
    }
}

fn make_type_table(type_decls: Vec<super::ir::type_decl::TypeDecl>) -> Option<HashMap<String, type_decl::TypeDecl>> {
    let mut type_table = HashMap::new();
    let mut errored = false;
    for decl in type_decls {
        if type_table.contains_key(decl.name.1) {
            Duplicate("named type", decl.name.0, decl.name.1).report();
            errored = true;
        }
        type_table.insert(decl.name.1.into(), decl);
    }

    if errored {
        None
    } else {
        Some(type_table)
    }
}
