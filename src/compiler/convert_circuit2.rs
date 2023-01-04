use std::collections::HashMap;

use super::ir::{self, ty};
use crate::circuit;

// TODO: clean up all imports everywhere

pub(crate) fn convert(type_context: &mut ty::TypeContext, circuit: &ir::circuit2::Circuit) -> circuit::Circuit {
    let mut new_circuit = circuit::Circuit::new(circuit.name.clone());
    new_circuit.set_num_inputs(type_context.get(circuit.input_type).size(type_context));
    new_circuit.set_num_outputs(type_context.get(circuit.output_type).size(type_context));
    let mut gate_index_map = HashMap::new();

    for (old_gate_i, gate) in circuit.iter_gates() {
        let new_gate_i = add_gate(type_context, &mut new_circuit, gate);
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (producer, receiver) in circuit.iter_connections() {
        connect(type_context, circuit, &mut new_circuit, &mut gate_index_map, producer, receiver);
    }

    new_circuit.calculate_locations();

    new_circuit
}

fn add_gate(type_context: &mut ty::TypeContext, new_circuit: &mut circuit::Circuit, gate: &ir::circuit2::Gate) -> circuit::GateIndex {
    match gate {
        ir::circuit2::Gate::Custom(subcircuit) => new_circuit.new_subcircuit_gate(convert(type_context, subcircuit)),
        ir::circuit2::Gate::Nand => new_circuit.new_nand_gate(),
        ir::circuit2::Gate::Const(value) => new_circuit.new_const_gate(*value),
    }
}

fn connect(
    type_context: &mut ty::TypeContext,
    old_circuit: &ir::circuit2::Circuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    producer: &ir::circuit2::bundle::ProducerBundle,
    receiver: &ir::circuit2::bundle::ReceiverBundle,
) {
    let producer_nodes: Vec<circuit::ProducerIdx> = convert_producer_bundle(type_context, old_circuit, new_circuit, gate_index_map, producer);
    let receiver_nodes: Vec<circuit::ReceiverIdx> = convert_receiver_bundle(type_context, old_circuit, new_circuit, gate_index_map, receiver);

    for (p, r) in producer_nodes.into_iter().zip(receiver_nodes) {
        new_circuit.connect(p, r);
    }
}

fn convert_producer_bundle(
    type_context: &mut ty::TypeContext,
    old_circuit: &ir::circuit2::Circuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    producer: &ir::circuit2::bundle::ProducerBundle,
) -> Vec<circuit::ProducerIdx> {
    match producer {
        // TODO: figure out a better solution than to collect
        ir::circuit2::bundle::ProducerBundle::CurCircuitInput => new_circuit.input_indexes().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::GateOutput(old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).outputs().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::Get(b, field) => {
            let b_type = b.type_(type_context, old_circuit);
            let field_indexes = type_context.get(b_type).field_indexes(type_context, field).expect("producer bundle should have field after type checking");
            let b_nodes = convert_producer_bundle(type_context, old_circuit, new_circuit, gate_index_map, b);

            b_nodes[field_indexes].to_vec()
        }
        ir::circuit2::bundle::ProducerBundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(type_context, old_circuit, new_circuit, gate_index_map, sb)).collect(),
    }
}
fn convert_receiver_bundle(
    _: &mut ty::TypeContext, // keep arguments for symmetry with convert_producer_bundle
    _: &ir::circuit2::Circuit,
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
