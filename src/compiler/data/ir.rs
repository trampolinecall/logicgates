use crate::{
    compiler::data::{ast, ty},
    utils::arena,
};

pub(crate) mod bundle;

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub(crate) struct GateIdx(usize);

impl arena::ArenaId for GateIdx {
    fn make(i: usize) -> Self {
        GateIdx(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Inline {
    Inline,
    NoInline,
}
#[derive(Debug)]
pub(crate) struct Circuit<'file> {
    pub(crate) name: &'file str,
    pub(crate) gates: arena::Arena<(ast::CircuitOrIntrinsicId, Inline), GateIdx>,
    pub(crate) connections: Vec<(bundle::ProducerBundle, bundle::ReceiverBundle)>,
    pub(crate) input_type: ty::TypeSym,
    pub(crate) output_type: ty::TypeSym,
}

#[derive(Debug)]
pub(crate) enum CircuitOrIntrinsic<'file> {
    Custom(Circuit<'file>),
    Nand,
    Const(bool),
}

/*
impl Gate {
    // this is kept for when i implement inlining gates
    pub(crate) fn inline_gate(&self, types: &ty::Types, circuit: &mut circuit::Circuit) -> (bundle::ReceiverBundle, bundle::ProducerBundle) {
        if let Circuit::Circuit { circuit: subcircuit, input_type: _, result_type: _ } = self {
            use crate::circuit::GateIndex;

            let mut gate_number_mapping: HashMap<GateIndex, GateIndex> = HashMap::new();
            let convert_producer_idx = |p, circuit: &circuit::Circuit, gate_number_mapping: &HashMap<GateIndex, GateIndex>| match p {
                circuit::ProducerIdx::CI(_) => None, // circuit inputs are left unconnected, will be connected by caller
                circuit::ProducerIdx::GO(go) => Some(circuit::ProducerIdx::GO(
                    circuit
                        .get_gate(gate_number_mapping[&go.0])
                        .outputs()
                        .nth(go.1)
                        .expect("gate index should be in range for the same gate type when converting producer index for inlining subcircuit"),
                )),
            };

            for (subcircuit_gate_i, gate) in subcircuit.gates.iter() {
                let (inner_inputs, gate_added_to_main_circuit) = match &gate.kind {
                    circuit::GateKind::Nand(inputs, _) => (&inputs[..], circuit.new_nand_gate()),
                    circuit::GateKind::Const(inputs, [circuit::Producer { value, .. }]) => (&inputs[..], circuit.new_const_gate(*value)),
                    circuit::GateKind::Subcircuit(inputs, _, subcircuit) => (&inputs[..], circuit.new_subcircuit_gate(subcircuit.borrow().clone())),
                };

                for (input, new_gate_input) in inner_inputs.iter().zip(circuit.get_gate(gate_added_to_main_circuit).inputs().collect::<Vec<_>>().into_iter()) {
                    // TODO: dont clone this
                    if let Some(inner_producer_idx) = input.producer {
                        if let Some(producer) = convert_producer_idx(inner_producer_idx, &circuit, &gate_number_mapping) {
                            circuit.connect(producer, new_gate_input.into());
                        }
                    }
                }

                gate_number_mapping.insert(subcircuit_gate_i, gate_added_to_main_circuit);
            }

            // prerequisites
            // - null producers and null/void type (unconnected inputs)
            // - multiple receivers (producer that connects to multiple outputs)
            todo!("inlining gates")
            /*
            Some((
                todo!(),
                self.make_producer_bundle(
                    &mut subcircuit
                        .output_indexes()
                        .flat_map(|co| subcircuit.get_receiver(co.into()).producer.map(|producer| convert_producer_idx(producer, &circuit, &gate_number_mapping))),
                ),
            ))
            */
            // TODO: allow unconnected nodes
        } else {
            self.add_gate(types, circuit)
        }
    }
}
*/
impl<'file> Circuit<'file> {
    pub(crate) fn new(name: &'file str, input_type: symtern::Sym<usize>, output_type: symtern::Sym<usize>) -> Circuit {
        Circuit { name, input_type, output_type, gates: arena::Arena::new(), connections: Vec::new() }
    }
}
