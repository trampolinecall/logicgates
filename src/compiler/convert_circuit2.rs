use std::collections::HashMap;

use super::{
    arena, convert_circuit1,
    error::File,
    ir::{self, circuit2, named_type, ty},
    make_name_tables,
};
use crate::circuit;

// TODO: clean up all imports everywhere

pub(crate) fn convert(file: &File, convert_circuit1::IR { circuits, circuit_table, mut type_context, type_table }: convert_circuit1::IR) -> Option<circuit::Circuit> {
    let circuit = match circuit_table.get("main") {
        Some((_, _, main_id)) => match circuits.get(*main_id) {
            circuit2::CircuitOrIntrinsic::Custom(c) => c,
            _ => unreachable!("builtin circuit called main"),
        },
        None => {
            // (&type_context, error::Error::NoMain(file)).report();
            todo!("report error for no main");
            None?;
        }
    };

    Some(convert_circuit(&circuits, &mut type_context, circuit))
}
pub(crate) fn convert_circuit(
    circuits: &arena::Arena<circuit2::CircuitOrIntrinsic, make_name_tables::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit: &circuit2::Circuit,
) -> circuit::Circuit {
    let mut new_circuit = circuit::Circuit::new(circuit.name.clone());
    new_circuit.set_num_inputs(type_context.get(circuit.input_type).size(&type_context));
    new_circuit.set_num_outputs(type_context.get(circuit.output_type).size(&type_context));
    let mut gate_index_map = HashMap::new();

    for (old_gate_i, gate) in circuit.iter_gates() {
        let new_gate_i = add_gate(circuits, type_context, &mut new_circuit, *gate);
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (producer, receiver) in circuit.iter_connections() {
        connect(type_context, circuit, &mut new_circuit, &mut gate_index_map, producer, receiver);
    }

    new_circuit.calculate_locations();

    new_circuit
}

fn add_gate(
    circuits: &arena::Arena<circuit2::CircuitOrIntrinsic, make_name_tables::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    new_circuit: &mut circuit::Circuit,
    circuit_id: make_name_tables::CircuitOrIntrinsicId,
) -> circuit::GateIndex {
    match circuits.get(circuit_id) {
        ir::circuit2::CircuitOrIntrinsic::Custom(subcircuit) => new_circuit.new_subcircuit_gate(convert_circuit(circuits, type_context, subcircuit)),
        ir::circuit2::CircuitOrIntrinsic::Nand => new_circuit.new_nand_gate(),
        ir::circuit2::CircuitOrIntrinsic::Const(value) => new_circuit.new_const_gate(*value),
    }
}

fn connect(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
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
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    old_circuit: &ir::circuit2::Circuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    producer: &ir::circuit2::bundle::ProducerBundle,
) -> Vec<circuit::ProducerIdx> {
    match producer {
        // TODO: figure out a better solution than to collect
        ir::circuit2::bundle::ProducerBundle::CurCircuitInput(_) => new_circuit.input_indexes().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::GateOutput(_, old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).outputs().map(Into::into).collect(),
        ir::circuit2::bundle::ProducerBundle::Get(b, field) => {
            let b_type = b.type_(type_context, old_circuit);
            let field_indexes = type_context.get(b_type).field_indexes(type_context, field).expect("producer bundle should have field after type checking");
            let b_nodes = convert_producer_bundle(type_context, old_circuit, new_circuit, gate_index_map, b);

            b_nodes[field_indexes].to_vec()
        }
        ir::circuit2::bundle::ProducerBundle::Product(subbundles) => {
            subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(type_context, old_circuit, new_circuit, gate_index_map, sb)).collect()
        }
    }
}
fn convert_receiver_bundle(
    _: &mut ty::TypeContext<named_type::FullyDefinedNamedType>, // keep arguments for symmetry with convert_producer_bundle
    _: &ir::circuit2::Circuit,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<ir::circuit2::GateIdx, generational_arena::Index>,
    receiver: &ir::circuit2::bundle::ReceiverBundle,
) -> Vec<circuit::ReceiverIdx> {
    match receiver {
        // TODO: figure out a better solution than to collect
        ir::circuit2::bundle::ReceiverBundle::CurCircuitOutput(_) => new_circuit.output_indexes().map(Into::into).collect(),
        ir::circuit2::bundle::ReceiverBundle::GateInput(_, old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).inputs().map(Into::into).collect(),
    }
}
