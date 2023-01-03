use std::collections::HashMap;

use super::ir::{self, ty};
use crate::circuit;

// TODO: clean up all imports everywhere

pub(crate) fn convert(types: &mut ty::Types, circuit: &ir::circuit2::Circuit) -> circuit::Circuit {
    if let ir::circuit2::Circuit::CustomCircuit(circuit) = circuit {
        let mut new_circuit = circuit::Circuit::new(circuit.name.clone()); // TODO: consume
        new_circuit.set_num_inputs(types.get(circuit.input_type).size(types));
        new_circuit.set_num_outputs(types.get(circuit.result_type).size(types)); // TODO: rename this to output_type instead of result_type
        let mut gate_index_map = HashMap::new();

        for (old_gate_i, gate) in circuit.iter_gates() {
            let new_gate_i = add_gate(types, &mut new_circuit, gate);
            gate_index_map.insert(old_gate_i, new_gate_i);
        }

        for (producer, receiver) in circuit.iter_connections() {
            connect(types, &circuit, &mut new_circuit, &mut gate_index_map, producer, receiver);
        }

        new_circuit.calculate_locations();

        new_circuit
    } else {
        unreachable!("builtin circuit2 circuit being converted as main circuit")
    }
}

fn add_gate(types: &mut ty::Types, new_circuit: &mut circuit::Circuit, gate: &ir::circuit2::Circuit) -> circuit::GateIndex {
    match gate {
        ir::circuit2::Circuit::CustomCircuit(subcircuit) => {
            // TODO: make convert accept a custom circuit so this does not have to be wrapped, and also that will probably will also remove this clone
            new_circuit.new_subcircuit_gate(convert(types, &ir::circuit2::Circuit::CustomCircuit(subcircuit.clone())))
        }
        ir::circuit2::Circuit::Nand => new_circuit.new_nand_gate(),
        ir::circuit2::Circuit::Const(value) => new_circuit.new_const_gate(*value),
    }
}

fn connect(
    types: &mut ty::Types,
    old_circuit: &ir::circuit2::CustomCircuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    producer: &ir::circuit2::bundle::ProducerBundle,
    receiver: &ir::circuit2::bundle::ReceiverBundle,
) {
    let producer_nodes: Vec<circuit::ProducerIdx> = convert_producer_bundle(types, old_circuit, new_circuit, gate_index_map, producer);
    let receiver_nodes: Vec<circuit::ReceiverIdx> = convert_receiver_bundle(types, old_circuit, new_circuit, gate_index_map, receiver);

    for (p, r) in producer_nodes.into_iter().zip(receiver_nodes) {
        new_circuit.connect(p, r);
    }
}

fn convert_producer_bundle(
    types: &mut ty::Types,
    old_circuit: &ir::circuit2::CustomCircuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    producer: &ir::circuit2::bundle::ProducerBundle,
) -> Vec<circuit::ProducerIdx> {
    match producer {
        // TODO: figure out a better solution than to collect
        ir::circuit2::bundle::ProducerBundle::CurCircuitInput => new_circuit.input_indexes().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::GateOutput(old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).outputs().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::Get(b, field) => {
            let b_type = b.type_(types, old_circuit);
            let field_indexes = types.get(b_type).field_indexes(types, field).expect("producer bundle should have field after type checking");
            let b_nodes = convert_producer_bundle(types, old_circuit, new_circuit, gate_index_map, b);

            b_nodes[field_indexes].to_vec()
        }
        ir::circuit2::bundle::ProducerBundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(types, old_circuit, new_circuit, gate_index_map, sb)).collect(),
    }
}
fn convert_receiver_bundle(
    _: &mut ty::Types, // keep arguments for symmetry with convert_producer_bundle
    _: &ir::circuit2::CustomCircuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    receiver: &ir::circuit2::bundle::ReceiverBundle,
) -> Vec<circuit::ReceiverIdx> {
    match receiver {
        // TODO: figure out a better solution than to collect
        ir::circuit2::bundle::ReceiverBundle::CurCircuitOutput => new_circuit.output_indexes().map(Into::into).collect(),
        ir::circuit2::bundle::ReceiverBundle::GateInput(old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).inputs().map(Into::into).collect(),
    }
}
