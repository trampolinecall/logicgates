use crate::{
    circuit,
    compiler::{error::Report, parser::ast},
};

use super::{
    bundle::{ProducerBundle, ReceiverBundle},
    connect_bundle, CircuitGenState, Error,
};

pub(super) enum CircuitDef {
    Circuit { circuit: circuit::Circuit, input_types: Vec<ast::Type>, result_type: ast::Type },
    And,
    Not,
}
impl CircuitDef {
    fn to_gate(&self, circuit_state: &mut CircuitGenState) -> (circuit::GateIndex, Vec<ReceiverBundle>, ProducerBundle) {
        // TODO: refactor this and probably refactor the rest of the module too
        match self {
            CircuitDef::Circuit { circuit, input_types, result_type } => {
                let gate_i = circuit_state.circuit.new_subcircuit_gate(circuit.clone());
                (gate_i, make_receiver_bundles(input_types, circuit_state.circuit.get_gate(gate_i).inputs()), make_producer_bundle(result_type, circuit_state.circuit.get_gate(gate_i).outputs()))
            }
            CircuitDef::And => {
                let gate_i = circuit_state.circuit.new_and_gate();
                (
                    gate_i,
                    circuit_state.circuit.get_gate(gate_i).inputs().map(|input| ReceiverBundle::Single(input.into())).collect(),
                    ProducerBundle::Single(circuit_state.circuit.get_gate(gate_i).outputs().nth(0).expect("and gate should have exactly one output").into()),
                )
            }
            CircuitDef::Not => {
                let gate_i = circuit_state.circuit.new_not_gate();
                (
                    gate_i,
                    circuit_state.circuit.get_gate(gate_i).inputs().map(|input| ReceiverBundle::Single(input.into())).collect(),
                    ProducerBundle::Single(circuit_state.circuit.get_gate(gate_i).outputs().nth(0).expect("and gate should have exactly one output").into()),
                )
            }
        }
    }

    pub(super) fn add_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
        let (gate_i, input_bundles, output_bundles) = self.to_gate(circuit_state);
        let gate = &circuit_state.circuit.get_gate(gate_i);

        // connect the inputs
        let input_types: Vec<_> = inputs.iter().map(|bundle| bundle.type_()).collect();
        let expected_input_types: Vec<_> = input_bundles.iter().map(|bundle| bundle.type_()).collect();
        if input_types.len() != expected_input_types.len() {
            Error::ArgNumMismatchInCall { actual_arity: input_types.len(), expected_arity: expected_input_types.len() }.report();
            None?
        }
        for (input_type, expected_type) in input_types.iter().zip(expected_input_types) {
            if *input_type != expected_type {}
        }

        for (producer_bundle, receiver_bundle) in inputs.into_iter().zip(input_bundles) {
            connect_bundle(&mut circuit_state.circuit, producer_bundle, receiver_bundle)?;
        }

        Some(output_bundles)
    }
    pub(crate) fn inline_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
        todo!("inlining gates")
        /*
        if let CircuitEntity::Circuit(subcircuit) = self {
            use crate::circuit::GateIndex;

            let mut gate_number_mapping: HashMap<GateIndex, GateIndex> = HashMap::new();
            let convert_producer_idx = |p, circuit: &circuit::Circuit, gate_number_mapping: &HashMap<GateIndex, GateIndex>| match p {
                circuit::ProducerIdx::CI(ci) => inputs[ci.0],
                circuit::ProducerIdx::GO(go) => circuit::ProducerIdx::GO(
                    circuit
                        .get_gate(gate_number_mapping[&go.0])
                        .outputs()
                        .nth(go.1)
                        .expect("gate index should be in range for the same gate type when converting producer index for inlining subcircuit"),
                ),
            };

            for (subcircuit_gate_i, gate) in subcircuit_state.circuit.gates.iter() {
                let (inner_inputs, gate_added_to_main_circuit) = match &gate.kind {
                    circuit::GateKind::And(inputs, _) => (&inputs[..], circuit_state.circuit.new_and_gate()),
                    circuit::GateKind::Not(inputs, _) => (&inputs[..], circuit_state.circuit.new_not_gate()),
                    circuit::GateKind::Const(inputs, [circuit::Producer { value, .. }]) => (&inputs[..], circuit_state.circuit.new_const_gate(*value)),
                    circuit::GateKind::Subcircuit(inputs, _, subcircuit) => (&inputs[..], circuit_state.circuit.new_subcircuit_gate(subcircuit_state.circuit.borrow().clone())),
                };

                for (input, new_gate_input) in inner_inputs.iter().zip(circuit_state.circuit.get_gate(gate_added_to_main_circuit).inputs().collect::<Vec<_>>().into_iter()) {
                    // TODO: dont clone this
                    if let Some(inner_producer_idx) = input.producer {
                        circuit_state.circuit.connect(convert_producer_idx(inner_producer_idx, &circuit_state.circuit, &gate_number_mapping), new_gate_input.into())
                    }
                }

                gate_number_mapping.insert(subcircuit_gate_i, gate_added_to_main_circuit);
            }

            Some(
                subcircuit
                    .output_indexes()
                    .flat_map(|o| subcircuit_state.circuit.get_receiver(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping)))
                    .collect(),
            ) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, inputs)
        }
        */
    }
}

// TODO: refactor
fn make_receiver_bundles(types: &[ast::Type], mut inputs: impl Iterator<Item = circuit::GateInputNodeIdx>) -> Vec<ReceiverBundle> {
    let mut bundles = Vec::new();
    for input_type in types {
        bundles.push(make_receiver_bundle(input_type, &mut inputs))
    }

    bundles
}

fn make_receiver_bundle(type_: &ast::Type, inputs: &mut impl Iterator<Item = circuit::GateInputNodeIdx>) -> ReceiverBundle {
    match type_ {
        ast::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle").into()),
    }
}

fn make_producer_bundle(type_: &ast::Type, mut outputs: impl Iterator<Item = circuit::GateOutputNodeIdx>) -> ProducerBundle {
    match type_ {
        ast::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle").into()),
    }
}
