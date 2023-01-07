use std::collections::HashMap;

use crate::compiler::{
    data::{circuit1, circuit2, named_type, ty},
    error::{CompileError, File, Report},
    phases::convert_circuit1,
};
use crate::{circuit, utils::arena};

// TODO: clean up all imports everywhere

struct NoMain<'file>(&'file File);
impl<'file> From<NoMain<'file>> for CompileError<'file> {
    fn from(NoMain(file): NoMain<'file>) -> Self {
        CompileError::new(file.eof_span(), "no 'main' circuit".into())
    }
}

type ExpandedStack<'circuit> = Vec<&'circuit circuit2::Circuit>;
struct InfiniteRecursion<'circuit>(ExpandedStack<'circuit>, &'circuit circuit2::Circuit);
impl<'circuit> From<InfiniteRecursion<'circuit>> for CompileError<'circuit> {
    fn from(InfiniteRecursion(circuits, repeat): InfiniteRecursion<'circuit>) -> Self {
        todo!("{circuits:?} -> {repeat:?}")
        // CompileError::new(todo!("span"), "infinite recursion in gates".into())
    }
}

pub(crate) fn convert(file: &File, convert_circuit1::IR { circuits, circuit_table, mut type_context }: convert_circuit1::IR) -> Option<circuit::Circuit> {
    if let Some((_, _, main_id)) = circuit_table.get("main") {
        if let circuit2::CircuitOrIntrinsic::Custom(circuit) = circuits.get(*main_id) {
            match convert_circuit(&circuits, &mut type_context, Vec::new(), circuit) {
                Ok((_, r)) => Some(r),
                Err(e) => {
                    e.report();
                    None
                }
            }
        } else {
            unreachable!("builtin circuit called main")
        }
    } else {
        NoMain(file).report();
        None
    }
}
fn convert_circuit<'circuit>(
    circuits: &'circuit arena::Arena<circuit2::CircuitOrIntrinsic, circuit1::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedStruct>,
    mut expansion_stack: ExpandedStack<'circuit>,
    circuit: &'circuit circuit2::Circuit,
) -> Result<(ExpandedStack<'circuit>, circuit::Circuit), InfiniteRecursion<'circuit>> {
    if expansion_stack.iter().find(|c| std::ptr::eq(**c, circuit)).is_some() {
        return Err(InfiniteRecursion(expansion_stack, circuit));
    }

    expansion_stack.push(circuit);

    let mut new_circuit = circuit::Circuit::new(circuit.name.clone());
    new_circuit.set_num_inputs(type_context.get(circuit.input_type).size(type_context));
    new_circuit.set_num_outputs(type_context.get(circuit.output_type).size(type_context));
    let mut gate_index_map = HashMap::new();

    for (old_gate_i, gate) in circuit.iter_gates() {
        let (expansion_stack_2, new_gate_i) = add_gate(circuits, type_context, expansion_stack, &mut new_circuit, *gate)?;
        expansion_stack = expansion_stack_2;
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (producer, receiver) in circuit.iter_connections() {
        connect(type_context, &mut new_circuit, &mut gate_index_map, producer, receiver);
    }

    new_circuit.calculate_locations();

    assert!(std::ptr::eq(*expansion_stack.last().unwrap(), circuit), "expansion stack should be in the same state as at the start of the function");
    expansion_stack.pop();

    Ok((expansion_stack, new_circuit))
}

fn add_gate<'circuit>(
    circuits: &'circuit arena::Arena<circuit2::CircuitOrIntrinsic, circuit1::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedStruct>,
    expansion_stack: ExpandedStack<'circuit>,
    new_circuit: &mut circuit::Circuit,
    circuit_id: circuit1::CircuitOrIntrinsicId,
) -> Result<(ExpandedStack<'circuit>, circuit::GateIndex), InfiniteRecursion<'circuit>> {
    match circuits.get(circuit_id) {
        circuit2::CircuitOrIntrinsic::Custom(subcircuit) => {
            let (expansion_stack, subcircuit) = convert_circuit(circuits, type_context, expansion_stack, subcircuit)?;
            Ok((expansion_stack, new_circuit.new_subcircuit_gate(subcircuit)))
        }
        circuit2::CircuitOrIntrinsic::Nand => Ok((expansion_stack, new_circuit.new_nand_gate())),
        circuit2::CircuitOrIntrinsic::Const(value) => Ok((expansion_stack, new_circuit.new_const_gate(*value))),
    }
}

fn connect(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedStruct>,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    producer: &circuit2::bundle::ProducerBundle,
    receiver: &circuit2::bundle::ReceiverBundle,
) {
    let producer_nodes: Vec<circuit::ProducerIdx> = convert_producer_bundle(type_context, new_circuit, gate_index_map, producer);
    let receiver_nodes: Vec<circuit::ReceiverIdx> = convert_receiver_bundle(type_context, new_circuit, gate_index_map, receiver);

    assert_eq!(producer_nodes.len(), receiver_nodes.len(), "connecting producer and receiver that have different size");

    for (p, r) in producer_nodes.into_iter().zip(receiver_nodes) {
        new_circuit.connect(p, r);
    }
}

fn convert_producer_bundle(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedStruct>,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    producer: &circuit2::bundle::ProducerBundle,
) -> Vec<circuit::ProducerIdx> {
    match producer {
        // TODO: figure out a better solution than to collect
        circuit2::bundle::ProducerBundle::CurCircuitInput(_) => new_circuit.input_indexes().map(Into::into).collect(),
        circuit2::bundle::ProducerBundle::GateOutput(_, old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).outputs().map(Into::into).collect(),
        circuit2::bundle::ProducerBundle::Get(b, field) => {
            let b_type = b.type_(type_context);
            let field_indexes = type_context.get(b_type).field_indexes(type_context, field).expect("producer bundle should have field after type checking");
            let b_nodes = convert_producer_bundle(type_context, new_circuit, gate_index_map, b);

            b_nodes[field_indexes].to_vec()
        }
        circuit2::bundle::ProducerBundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(type_context, new_circuit, gate_index_map, sb)).collect(),
    }
}
fn convert_receiver_bundle(
    _: &mut ty::TypeContext<named_type::FullyDefinedStruct>, // keep arguments for symmetry with convert_producer_bundle
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    receiver: &circuit2::bundle::ReceiverBundle,
) -> Vec<circuit::ReceiverIdx> {
    match receiver {
        // TODO: figure out a better solution than to collect
        circuit2::bundle::ReceiverBundle::CurCircuitOutput(_) => new_circuit.output_indexes().map(Into::into).collect(),
        circuit2::bundle::ReceiverBundle::GateInput(_, old_gate_index) => new_circuit.get_gate(gate_index_map[old_gate_index]).inputs().map(Into::into).collect(),
    }
}
