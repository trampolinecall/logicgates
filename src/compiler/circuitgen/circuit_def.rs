use std::collections::HashMap;

use super::bundle::ProducerBundle;
use super::bundle::ReceiverBundle;
use super::CircuitGenState;
use super::Error;
use crate::circuit;
use crate::compiler::circuitgen::bundle::connect_bundle;
use crate::compiler::circuitgen::bundle::make_producer_bundle;
use crate::compiler::circuitgen::bundle::make_receiver_bundle;
use crate::compiler::error::Report;
use crate::compiler::error::Span;
use crate::compiler::parser::ast;

pub(super) enum CircuitDef {
    Circuit { circuit: circuit::Circuit, input_type: ast::Type, result_type: ast::Type },
    And,
    Not,
    Const(bool),
}
impl CircuitDef {
    fn to_gate(&self, circuit_state: &mut CircuitGenState) -> (circuit::GateIndex, ReceiverBundle, ProducerBundle) {
        // TODO: refactor this and probably refactor the rest of the module too
        let make_receiver_bundle = |circuit_state: &CircuitGenState, type_: &ast::Type, gate_i| {
            let inputs = circuit_state.circuit.get_gate(gate_i).inputs();
            assert_eq!(type_.size(), inputs.len(), "receiver bundles have a different total size than the number of input nodes on the gate"); // sanity check
            make_receiver_bundle(type_, &mut inputs.map(|input| input.into()))
        };
        let make_producer_bundle = |circuit_state: &CircuitGenState, type_: &ast::Type, gate_i| {
            let outputs = circuit_state.circuit.get_gate(gate_i).outputs();
            assert_eq!(type_.size(), outputs.len(), "producer bundle has a different size than the number of output nodes on the gate"); // sanity check
            make_producer_bundle(type_, &mut outputs.map(|output| output.into()))
        };

        match self {
            CircuitDef::Circuit { circuit, input_type, result_type } => {
                let gate_i = circuit_state.circuit.new_subcircuit_gate(circuit.clone());
                (gate_i, make_receiver_bundle(circuit_state, input_type, gate_i), make_producer_bundle(circuit_state, result_type, gate_i))
            }
            CircuitDef::And => {
                let gate_i = circuit_state.circuit.new_and_gate();
                (gate_i, make_receiver_bundle(circuit_state, &ast::Type::Product(vec![ast::Type::Bit, ast::Type::Bit]), gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
            CircuitDef::Not => {
                let gate_i = circuit_state.circuit.new_not_gate();
                (gate_i, make_receiver_bundle(circuit_state, &ast::Type::Product(vec![ast::Type::Bit]), gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
            CircuitDef::Const(value) => {
                let gate_i = circuit_state.circuit.new_const_gate(*value);
                (gate_i, make_receiver_bundle(circuit_state, &ast::Type::Product(vec![]), gate_i), make_producer_bundle(circuit_state, &ast::Type::Bit, gate_i))
            }
        }
    }

    pub(super) fn add_gate(&self, circuit_state: &mut CircuitGenState, expr_span: Span, input_value: ProducerBundle) -> Option<ProducerBundle> {
        let (_, input_bundle, output_bundle) = self.to_gate(circuit_state);

        connect_bundle(&mut circuit_state.circuit, expr_span, &input_value, &input_bundle)?;

        Some(output_bundle)
    }
    pub(crate) fn inline_gate(&self, circuit_state: &mut CircuitGenState, expr_span: Span, inputs: ProducerBundle) -> Option<ProducerBundle> {
        if let CircuitDef::Circuit { circuit: subcircuit, input_type: expected_input_types, result_type } = self {
            use crate::circuit::GateIndex;

            let actual_input_type = inputs.type_();
            if actual_input_type != *expected_input_types {
                Error::TypeMismatchInCall { expr_span, actual_type: actual_input_type, expected_type: expected_input_types.clone() }.report();
                None?
            }

            let flat_inputs = inputs.flatten();
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
                &mut subcircuit.output_indexes().flat_map(|o| subcircuit.get_receiver(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping))),
            )) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, expr_span, inputs)
        }
    }
}
