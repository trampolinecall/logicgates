use std::collections::HashMap;

use super::{
    error::{CompileError, Span, Report},
    ir::circuit1,
};

struct Duplicate<'file>(Span<'file>, &'file str);
impl<'file> From<Duplicate<'file>> for CompileError<'file> {
    fn from(Duplicate(name_sp, name): Duplicate<'file>) -> Self {
        CompileError::new(name_sp, format!("circuit '{}' defined more than once", name))
    }
}

pub(crate) fn make(circuit1s: Vec<circuit1::UntypedCircuit>) -> Option<HashMap<String, circuit1::UntypedCircuitOrIntrinsic>> {
    let mut table = HashMap::new();
    table.insert("nand".into(), circuit1::UntypedCircuitOrIntrinsic::Nand);
    let mut errored = false;
    for circuit in circuit1s {
        if table.contains_key(circuit.name.1) {
            Duplicate(circuit.name.0, circuit.name.1).report();
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

