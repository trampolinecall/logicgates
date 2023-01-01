use std::collections::HashMap;

use super::ty;
use super::CircuitGenState;
use crate::circuit;
use crate::compiler::circuitgen::bundle;

pub(super) enum CircuitDef {
    Circuit { circuit: circuit::Circuit, input_type: ty::Type, result_type: ty::Type },
    And,
    Not,
    Const(bool),
}
impl CircuitDef {
    fn input_type(&self) -> ty::Type {
        match self {
            CircuitDef::Circuit { circuit: _, input_type, result_type: _ } => input_type.clone(), // clone will be gone when type interner is implemented (TODO)
            CircuitDef::And => ty::Type::Product(vec![ty::Type::Bit, ty::Type::Bit]),
            CircuitDef::Not => ty::Type::Bit,
            CircuitDef::Const(_) => ty::Type::Product(vec![]),
        }
    }
    fn output_type(&self) -> ty::Type {
        match self {
            CircuitDef::Circuit { circuit: _, input_type: _, result_type } => result_type.clone(),
            CircuitDef::And => ty::Type::Bit,
            CircuitDef::Not => ty::Type::Bit,
            CircuitDef::Const(_) => ty::Type::Bit,
        }
    }

    fn make_receiver_bundle(&self, inputs: &mut impl ExactSizeIterator<Item = circuit::ReceiverIdx>) -> bundle::ReceiverBundle {
        assert_eq!(self.input_type().size(), inputs.len(), "receiver bundles have a different total size than the number of input nodes on the gate"); // sanity check
        bundle::make_receiver_bundle(&self.input_type(), inputs)
    }

    fn make_producer_bundle(&self, outputs: &mut impl ExactSizeIterator<Item = circuit::ProducerIdx>) -> bundle::ProducerBundle {
        assert_eq!(self.output_type().size(), outputs.len(), "producer bundle has a different size than the number of output nodes on the gate"); // sanity check
        bundle::make_producer_bundle(&self.output_type(), outputs)
    }

    pub(super) fn add_gate(&self, circuit_state: &mut CircuitGenState) -> Option<(bundle::ReceiverBundle, bundle::ProducerBundle)> {
        let gate_i = match self {
            CircuitDef::Circuit { circuit, input_type: _, result_type: _ } => circuit_state.circuit.new_subcircuit_gate(circuit.clone()),
            CircuitDef::And => circuit_state.circuit.new_and_gate(),
            CircuitDef::Not => circuit_state.circuit.new_not_gate(),
            CircuitDef::Const(value) => circuit_state.circuit.new_const_gate(*value),
        };

        let input_bundle = self.make_receiver_bundle(&mut circuit_state.circuit.get_gate(gate_i).inputs().map(|input| input.into()));
        let output_bundle = self.make_producer_bundle(&mut circuit_state.circuit.get_gate(gate_i).outputs().map(|output| output.into()));

        Some((input_bundle, output_bundle))
    }
    pub(crate) fn inline_gate(&self, circuit_state: &mut CircuitGenState) -> Option<(bundle::ReceiverBundle, bundle::ProducerBundle)> {
        if let CircuitDef::Circuit { circuit: subcircuit, input_type: _, result_type: _ } = self {
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
                    circuit::GateKind::And(inputs, _) => (&inputs[..], circuit_state.circuit.new_and_gate()),
                    circuit::GateKind::Not(inputs, _) => (&inputs[..], circuit_state.circuit.new_not_gate()),
                    circuit::GateKind::Const(inputs, [circuit::Producer { value, .. }]) => (&inputs[..], circuit_state.circuit.new_const_gate(*value)),
                    circuit::GateKind::Subcircuit(inputs, _, subcircuit) => (&inputs[..], circuit_state.circuit.new_subcircuit_gate(subcircuit.borrow().clone())),
                };

                for (input, new_gate_input) in inner_inputs.iter().zip(circuit_state.circuit.get_gate(gate_added_to_main_circuit).inputs().collect::<Vec<_>>().into_iter()) {
                    // TODO: dont clone this
                    if let Some(inner_producer_idx) = input.producer {
                        if let Some(producer) = convert_producer_idx(inner_producer_idx, &circuit_state.circuit, &gate_number_mapping) {
                            circuit_state.circuit.connect(producer, new_gate_input.into())
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
                        .flat_map(|co| subcircuit.get_receiver(co.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping))),
                ),
            ))
            */
            // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state)
        }
    }
}
