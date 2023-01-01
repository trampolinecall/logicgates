use crate::{circuit, compiler::{parser::ast, error::Report}};

use super::{Gates, CircuitGenState, bundle::{ProducerBundle, ReceiverBundle}, Error, connect_bundle};

pub(super) enum CircuitDef {
    Circuit(circuit::Circuit),
    AndBuiltin,
    NotBuiltin,
}
impl CircuitDef {
    fn to_gate(&self, gates: Gates, circuit_state: &mut CircuitGenState) -> (circuit::GateIndex, Vec<ReceiverBundle>, ProducerBundle) {
        // TODO: refactor this and probably refactor the rest of the module too
        match self {
            CircuitDef::Circuit(c) => {
                let gate_i = circuit_state.circuit.new_subcircuit_gate(gates, c.clone());
                (gate_i, gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(), todo!())
                // TODO: make this actually respond to types
            }
            CircuitDef::AndBuiltin => {
                let gate_i = circuit_state.circuit.new_and_gate(gates);
                (
                    gate_i,
                    gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(),
                    ProducerBundle::Single(gates[gate_i].outputs().nth(0).expect("and gate should have exactly one output").into()),
                )
            }
            CircuitDef::NotBuiltin => {
                let gate_i = circuit_state.circuit.new_not_gate(gates);
                (
                    gate_i,
                    gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(),
                    ProducerBundle::Single(gates[gate_i].outputs().nth(0).expect("and gate should have exactly one output").into()),
                )
            }
        }
    }

    pub(super) fn add_gate(&self, gates: Gates, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
        let (gate_i, input_bundles, output_bundles) = self.to_gate(gates, circuit_state);
        let gate = &gates[gate_i];

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
            connect_bundle(gates, &mut circuit_state.circuit, producer_bundle, receiver_bundle)?;
            // circuit_state.circuit.connect(input_value, gate_input_node.into());
        }

        Some(output_bundles)
    }

    pub(super) fn inline_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
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

            for (subcircuit_gate_i, gate) in subcircuit.gates.iter() {
                let (inner_inputs, gate_added_to_main_circuit) = match &gate.kind {
                    circuit::GateKind::And(inputs, _) => (&inputs[..], circuit_state.circuit.new_and_gate()),
                    circuit::GateKind::Not(inputs, _) => (&inputs[..], circuit_state.circuit.new_not_gate()),
                    circuit::GateKind::Const(inputs, [circuit::Producer { value, .. }]) => (&inputs[..], circuit_state.circuit.new_const_gate(*value)),
                    circuit::GateKind::Subcircuit(inputs, _, subcircuit) => (&inputs[..], circuit_state.circuit.new_subcircuit_gate(subcircuit.borrow().clone())),
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
                    .flat_map(|o| subcircuit.get_receiver(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping)))
                    .collect(),
            ) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, inputs)
        }
        */
    }

    fn expected_input_types(&self) -> Vec<ast::Type> {
        match self {
            CircuitDef::AndBuiltin => vec![ast::Type::Bit, ast::Type::Bit],
            CircuitDef::NotBuiltin => vec![ast::Type::Bit],
            CircuitDef::Circuit(_) => todo!(),
        }
    }
}
