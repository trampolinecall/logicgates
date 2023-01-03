use super::ty;

pub(crate) mod bundle;

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq)]
pub(crate) struct GateIdx(usize);

#[derive(Clone, Debug)]
pub(crate) struct Circuit {
    pub(crate) name: String,
    gates: Vec<Gate>,
    connections: Vec<(bundle::ProducerBundle, bundle::ReceiverBundle)>,
    pub(crate) input_type: ty::TypeSym,
    pub(crate) output_type: ty::TypeSym,
}

#[derive(Clone, Debug)]
pub(crate) enum Gate {
    // TODO: rename to gate
    Custom(Circuit),
    Nand,
    Const(bool),
}
impl Gate {
    fn input_type(&self, types: &mut ty::Types) -> ty::TypeSym {
        match self {
            Gate::Custom(Circuit { input_type, output_type: _, gates: _, connections: _, name: _ }) => *input_type,
            Gate::Nand {} => {
                let b = types.intern(ty::Type::Bit);
                types.intern(ty::Type::Product(vec![("0".into(), b), ("1".into(), b)]))
            }
            Gate::Const(_) => types.intern(ty::Type::Product(vec![])),
        }
    }
    fn output_type(&self, types: &mut ty::Types) -> ty::TypeSym {
        match self {
            Gate::Custom(Circuit { input_type: _, output_type, gates: _, connections: _, name: _ }) => *output_type,
            Gate::Nand {} | Gate::Const(_) => types.intern(ty::Type::Bit),
        }
    }

    /*
    pub(crate) fn add_gate(&self, types: &ty::Types, circuit: &mut circuit::Circuit) -> (bundle::ReceiverBundle, bundle::ProducerBundle) {
        let gate_i = match self {
            Circuit::Circuit { circuit: circuit_def, input_type: _, result_type: _ } => circuit.new_subcircuit_gate(circuit_def.clone()),
            Circuit::Nand { input_type: _, result_type: _ } => circuit.new_nand_gate(),
            Circuit::Const { value, input_type: _, result_type: _ } => circuit.new_const_gate(*value),
        };

        let input_bundle = self.make_receiver_bundle(types, &mut circuit.get_gate(gate_i).inputs().map(Into::into));
        let output_bundle = self.make_producer_bundle(types, &mut circuit.get_gate(gate_i).outputs().map(Into::into));

        (input_bundle, output_bundle)
    }
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
    */
}
impl Circuit {
    pub(crate) fn new(name: String, input_type: symtern::Sym<usize>, output_type: symtern::Sym<usize>) -> Circuit {
        Circuit { name, input_type, output_type, gates: Vec::new(), connections: Vec::new() }
    }

    pub(crate) fn add_gate(&mut self, gate: Gate) -> GateIdx {
        self.gates.push(gate);
        GateIdx(self.gates.len() - 1)
    }

    fn get_gate(&self, gate_idx: GateIdx) -> &Gate {
        &self.gates[gate_idx.0]
    }

    pub(crate) fn add_connection(&mut self, producer: bundle::ProducerBundle, receiver: bundle::ReceiverBundle) {
        // TODO: probably should put type error here or assertion
        self.connections.push((producer, receiver));
    }

    pub(crate) fn iter_gates(&self) -> impl Iterator<Item = (GateIdx, &Gate)> {
        self.gates.iter().enumerate().map(|(i, g)| (GateIdx(i), g))
    }

    pub(crate) fn iter_connections(&self) -> std::slice::Iter<(bundle::ProducerBundle, bundle::ReceiverBundle)> {
        self.connections.iter()
    }
}
