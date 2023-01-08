use std::collections::HashMap;

use crate::{
    compiler::{
        data::{circuit1, circuit2, nominal_type, ty},
        error::{CompileError, File, Report},
        phases::convert_circuit1,
    },
    simulation::{circuit, connections},
    utils::arena,
};

struct NoMain<'file>(&'file File);
impl<'file> From<NoMain<'file>> for CompileError<'file> {
    fn from(NoMain(file): NoMain<'file>) -> Self {
        CompileError::new(file.eof_span(), "no 'main' circuit".into())
    }
}

type ExpandedStack<'file, 'circuit> = Vec<&'circuit circuit2::Circuit<'file>>;
struct InfiniteRecursion<'file, 'circuit>(ExpandedStack<'file, 'circuit>, &'circuit circuit2::Circuit<'file>);
impl<'file, 'circuit> From<InfiniteRecursion<'file, 'circuit>> for CompileError<'file> {
    fn from(InfiniteRecursion(circuits, repeat): InfiniteRecursion<'file, 'circuit>) -> Self {
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
fn convert_circuit<'file, 'circuit>(
    circuits: &'circuit arena::Arena<circuit2::CircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    mut expansion_stack: ExpandedStack<'file, 'circuit>,
    circuit: &'circuit circuit2::Circuit<'file>,
) -> Result<(ExpandedStack<'file, 'circuit>, circuit::Circuit), InfiniteRecursion<'file, 'circuit>> {
    if expansion_stack.iter().any(|c| std::ptr::eq(*c, circuit)) {
        return Err(InfiniteRecursion(expansion_stack, circuit));
    }

    expansion_stack.push(circuit);

    let mut new_circuit = circuit::Circuit::new(circuit.name.into(), type_context.get(circuit.input_type).size(type_context), type_context.get(circuit.output_type).size(type_context));
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

fn add_gate<'file, 'circuit>(
    circuits: &'circuit arena::Arena<circuit2::CircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    expansion_stack: ExpandedStack<'file, 'circuit>,
    new_circuit: &mut circuit::Circuit,
    circuit_id: circuit1::CircuitOrIntrinsicId,
) -> Result<(ExpandedStack<'file, 'circuit>, circuit::GateIndex), InfiniteRecursion<'file, 'circuit>> {
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
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    producer: &circuit2::bundle::ProducerBundle,
    receiver: &circuit2::bundle::ReceiverBundle,
) {
    let producer_nodes: Vec<connections::ProducerIdx> = convert_producer_bundle(type_context, new_circuit, gate_index_map, producer);
    let receiver_nodes: Vec<connections::ReceiverIdx> = convert_receiver_bundle(type_context, new_circuit, gate_index_map, receiver);

    assert_eq!(producer_nodes.len(), receiver_nodes.len(), "connecting producer and receiver that have different size");

    for (p, r) in producer_nodes.into_iter().zip(receiver_nodes) {
        connections::connect(new_circuit, p, r);
    }
}

fn convert_producer_bundle(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    producer: &circuit2::bundle::ProducerBundle,
) -> Vec<connections::ProducerIdx> {
    match producer {
        // TODO: figure out a better solution than to collect
        circuit2::bundle::ProducerBundle::CurCircuitInput(_) => connections::circuit_input_indexes(new_circuit).map(Into::into).collect(),
        circuit2::bundle::ProducerBundle::GateOutput(_, old_gate_index) => connections::gate_outputs(new_circuit.get_gate(gate_index_map[old_gate_index])).map(Into::into).collect(),
        circuit2::bundle::ProducerBundle::Get(b, field) => {
            fn field_indexes(ty: &ty::Type, type_context: &ty::TypeContext<nominal_type::FullyDefinedStruct>, field: &str) -> Option<std::ops::Range<usize>> {
                match ty {
                    ty::Type::Bit => None,
                    ty::Type::Product(fields) => {
                        let mut cur_index = 0;
                        for (field_name, field_type) in fields {
                            let cur_type_size = type_context.get(*field_type).size(type_context);
                            if field_name == field {
                                return Some(cur_index..cur_index + cur_type_size);
                            }
                            cur_index += cur_type_size;
                        }

                        None
                    }
                    ty::Type::Nominal(struct_id) => {
                        let fields = &type_context.structs.get(*struct_id).fields;
                        let mut cur_index = 0;
                        for ((_, field_name), field_type) in fields {
                            let cur_type_size = type_context.get(*field_type).size(type_context);
                            if *field_name == field {
                                return Some(cur_index..cur_index + cur_type_size);
                            }
                            cur_index += cur_type_size;
                        }

                        None
                    }
                }
            }
            let b_nodes = convert_producer_bundle(type_context, new_circuit, gate_index_map, b);

            let b_type = b.type_(type_context);
            let field_indexes = field_indexes(type_context.get(b_type), type_context, field).expect("producer bundle should have field after type checking");

            b_nodes[field_indexes].to_vec()
        }
        circuit2::bundle::ProducerBundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(type_context, new_circuit, gate_index_map, sb)).collect(),
    }
}
fn convert_receiver_bundle(
    _: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>, // keep arguments for symmetry with convert_producer_bundle
    new_circuit: &mut circuit::Circuit,
    gate_index_map: &mut HashMap<circuit2::GateIdx, circuit::GateIndex>,
    receiver: &circuit2::bundle::ReceiverBundle,
) -> Vec<connections::ReceiverIdx> {
    match receiver {
        // TODO: figure out a better solution than to collect
        circuit2::bundle::ReceiverBundle::CurCircuitOutput(_) => connections::circuit_output_indexes(new_circuit).map(Into::into).collect(),
        circuit2::bundle::ReceiverBundle::GateInput(_, old_gate_index) => connections::gate_inputs(new_circuit.get_gate(gate_index_map[old_gate_index])).map(Into::into).collect(),
    }
}
