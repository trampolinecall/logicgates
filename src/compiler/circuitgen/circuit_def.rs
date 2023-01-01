use std::collections::HashMap;

use super::bundle::ProducerBundle;
use super::bundle::ReceiverBundle;
use super::CircuitGenState;
use super::Error;
use crate::circuit;
use crate::compiler::error::Report;
use crate::compiler::parser::ast;

pub(super) enum CircuitDef {
    Circuit { circuit: circuit::Circuit, input_types: Vec<ast::Type>, result_type: ast::Type },
    And,
    Not,
    Const(bool),
}
impl CircuitDef {
    fn to_gate(&self, circuit_state: &mut CircuitGenState) -> (circuit::GateIndex, Vec<ReceiverBundle>, ProducerBundle) {
        // TODO: refactor this and probably refactor the rest of the module too
        let make_receiver_bundles = |circuit_state: &CircuitGenState, types: &[ast::Type], gate_i| {
            let inputs = circuit_state.circuit.get_gate(gate_i).inputs();
            assert_eq!(types.iter().map(|type_| type_.size()).sum::<usize>(), inputs.len(), "receiver bundles have a different total size than the number of input nodes on the gate"); // sanity check
            make_receiver_bundles(types, inputs.map(|input| input.into()))
        };
        let make_producer_bundle = |circuit_state: &CircuitGenState, type_: &ast::Type, gate_i| {
            let outputs = circuit_state.circuit.get_gate(gate_i).outputs();
            assert_eq!(type_.size(), outputs.len(), "producer bundle has a different size than the number of output nodes on the gate"); // sanity check
            make_producer_bundle(&type_, outputs.map(|output| output.into()))
        };

        match self {
            CircuitDef::Circuit { circuit, input_types, result_type } => {
                let gate_i = circuit_state.circuit.new_subcircuit_gate(circuit.clone());
                (gate_i, make_receiver_bundles(circuit_state, input_types, gate_i), make_producer_bundle(circuit_state, result_type, gate_i))
            }
            CircuitDef::And => {
                let gate_i = circuit_state.circuit.new_and_gate();
                (gate_i, make_receiver_bundles(circuit_state, &[ast::Type::Bit, ast::Type::Bit], gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
            CircuitDef::Not => {
                let gate_i = circuit_state.circuit.new_not_gate();
                (gate_i, make_receiver_bundles(circuit_state, &[ast::Type::Bit], gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
            CircuitDef::Const(value) => {
                let gate_i = circuit_state.circuit.new_const_gate(*value);
                (gate_i, make_receiver_bundles(circuit_state, &[], gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
        }
    }

    pub(super) fn add_gate(&self, circuit_state: &mut CircuitGenState, inputs: &[ProducerBundle]) -> Option<ProducerBundle> {
        let (gate_i, input_bundles, output_bundles) = self.to_gate(circuit_state);

        // connect the inputs
        let input_types: Vec<_> = inputs.iter().map(|bundle| bundle.type_()).collect();
        let expected_input_types: Vec<_> = input_bundles.iter().map(|bundle| bundle.type_()).collect();
        if input_types.len() != expected_input_types.len() {
            Error::ArgNumMismatchInCall { actual_arity: input_types.len(), expected_arity: expected_input_types.len() }.report();
            None?
        }

        for (producer_bundle, receiver_bundle) in inputs.iter().zip(input_bundles) {
            connect_bundle(&mut circuit_state.circuit, producer_bundle, &receiver_bundle)?;
        }

        Some(output_bundles)
    }
    pub(crate) fn inline_gate(&self, circuit_state: &mut CircuitGenState, inputs: &[ProducerBundle]) -> Option<ProducerBundle> {
        if let CircuitDef::Circuit { circuit: subcircuit, input_types: expected_input_types, result_type } = self {
            use crate::circuit::GateIndex;

            let actual_input_types: Vec<_> = inputs.iter().map(|bundle| bundle.type_()).collect();
            if actual_input_types.len() != expected_input_types.len() {
                Error::ArgNumMismatchInCall { actual_arity: actual_input_types.len(), expected_arity: expected_input_types.len() }.report();
                None?
            }
            for (input_type, expected_type) in actual_input_types.iter().zip(expected_input_types) {
                if *input_type != *expected_type {
                    Error::TypeMismatchInCall { actual_type: input_type.clone(), expected_type: expected_type.clone() }.report()
                }
            }

            let flat_inputs: Vec<_> = inputs.iter().flat_map(ProducerBundle::flatten).collect();
            let mut gate_number_mapping: HashMap<GateIndex, GateIndex> = HashMap::new();
            let convert_producer_idx = |p, circuit: &circuit::Circuit, gate_number_mapping: &HashMap<GateIndex, GateIndex>| match p {
                circuit::ProducerIdx::CI(ci) => flat_inputs[ci.0],
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

            Some(make_producer_bundle(
                result_type,
                subcircuit.output_indexes().flat_map(|o| subcircuit.get_receiver(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping))),
            )) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, inputs)
        }
    }
}

// TODO: refactor
fn make_receiver_bundles(types: &[ast::Type], mut inputs: impl Iterator<Item = circuit::ReceiverIdx>) -> Vec<ReceiverBundle> {
    let mut bundles = Vec::new();
    for input_type in types {
        bundles.push(make_receiver_bundle(input_type, &mut inputs))
    }

    bundles
}

fn make_receiver_bundle(type_: &ast::Type, inputs: &mut impl Iterator<Item = circuit::ReceiverIdx>) -> ReceiverBundle {
    match type_ {
        ast::Type::Bit => ReceiverBundle::Single(inputs.next().expect("inputs should not run out when converting to bundle")),
    }
}

fn make_producer_bundle(type_: &ast::Type, mut outputs: impl Iterator<Item = circuit::ProducerIdx>) -> ProducerBundle {
    match type_ {
        ast::Type::Bit => ProducerBundle::Single(outputs.next().expect("outputs should not run out when converting to bundle")),
    }
}

fn connect_bundle(circuit: &mut circuit::Circuit, producer_bundle: &ProducerBundle, receiver_bundle: &ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
    }

    Some(())
}
