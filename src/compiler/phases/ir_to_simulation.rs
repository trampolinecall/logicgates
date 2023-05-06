use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, ir, nominal_type, ty},
        error::{CompileError, File, Report},
        phases::ast_to_ir,
    },
    simulation::{self, logic},
    utils::arena,
};

struct NoMain<'file>(&'file File);
impl<'file> From<NoMain<'file>> for CompileError<'file> {
    fn from(NoMain(file): NoMain<'file>) -> Self {
        CompileError::new(file.eof_span(), "no '\\main' circuit".into())
    }
}

type ExpansionStack<'file, 'circuit> = Vec<&'circuit ir::Circuit<'file>>;
struct InfiniteRecursion<'file, 'circuit>(ExpansionStack<'file, 'circuit>, &'circuit ir::Circuit<'file>);
impl<'file, 'circuit> From<InfiniteRecursion<'file, 'circuit>> for CompileError<'file> {
    fn from(InfiniteRecursion(circuits, repeat): InfiniteRecursion<'file, 'circuit>) -> Self {
        todo!("{circuits:?} -> {repeat:?}")
        // CompileError::new(todo!("span"), "infinite recursion in gates".into())
    }
}

pub(crate) fn convert(file: &File, ast_to_ir::IR { circuits, circuit_table, mut type_context }: ast_to_ir::IR) -> Option<simulation::Simulation> {
    if let Some((_, _, main_id)) = circuit_table.get("main") {
        if let ir::CircuitOrIntrinsic::Custom(circuit) = circuits.get(*main_id) {
            let mut gate_map = simulation::GateMap::with_key();
            let mut circuit_map = simulation::CircuitMap::with_key();
            let mut node_map = simulation::NodeMap::with_key();
            let main_circuit = match convert_circuit(&mut circuit_map, &mut gate_map, &mut node_map, &circuits, &mut type_context, Vec::new(), circuit) {
                Ok((_, r)) => r,
                Err(e) => {
                    e.report();
                    None?
                }
            };

            for input_node_i in 0..circuit_map[main_circuit].inputs().len() {
                logic::set_input(&mut circuit_map, &mut node_map, main_circuit, input_node_i, logic::Value::L);
            }

            let mut simulation = simulation::Simulation { circuits: circuit_map, gates: gate_map, nodes: node_map, main_circuit };
            simulation::location::calculate_locations(&mut simulation);

            Some(simulation)
        } else {
            unreachable!("builtin circuit called main")
        }
    } else {
        NoMain(file).report();
        None
    }
}
fn convert_circuit<'file, 'circuit>(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    circuits: &'circuit arena::Arena<ir::CircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    mut expansion_stack: ExpansionStack<'file, 'circuit>,
    circuit: &'circuit ir::Circuit<'file>,
) -> Result<(ExpansionStack<'file, 'circuit>, simulation::CircuitKey), InfiniteRecursion<'file, 'circuit>> {
    if expansion_stack.iter().any(|c| std::ptr::eq(*c, circuit)) {
        return Err(InfiniteRecursion(expansion_stack, circuit));
    }

    expansion_stack.push(circuit);

    let new_circuit_idx = circuit_map.insert_with_key(|ck| {
        simulation::Circuit::new(ck, node_map, circuit.name.into(), type_context.get(circuit.input_type).size(type_context), type_context.get(circuit.output_type).size(type_context))
    });
    let mut gate_index_map = HashMap::new();

    for (old_gate_i, gate) in circuit.gates.iter_with_ids() {
        let (expansion_stack_2, new_gate_i) = add_gate(circuit_map, gate_map, node_map, circuits, type_context, expansion_stack, *gate, new_circuit_idx)?;
        expansion_stack = expansion_stack_2;
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (producer, receiver) in circuit.connections.iter() {
        connect(circuit_map, gate_map, node_map, type_context, new_circuit_idx, &mut gate_index_map, producer, receiver);
    }

    assert!(std::ptr::eq(*expansion_stack.last().unwrap(), circuit), "expansion stack should be in the same state as at the start of the function");
    expansion_stack.pop();

    Ok((expansion_stack, new_circuit_idx))
}

fn add_gate<'file, 'circuit>(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    circuits: &'circuit arena::Arena<ir::CircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    expansion_stack: ExpansionStack<'file, 'circuit>,
    (circuit_id, _): (ast::CircuitOrIntrinsicId, ir::Inline),
    new_circuit_idx: simulation::CircuitKey,
) -> Result<(ExpansionStack<'file, 'circuit>, simulation::GateKey), InfiniteRecursion<'file, 'circuit>> {
    let (expansion_stack, gate_idx) = match circuits.get(circuit_id) {
        ir::CircuitOrIntrinsic::Custom(subcircuit) => {
            let (expansion_stack, subcircuit_idx) = convert_circuit(circuit_map, gate_map, node_map, circuits, type_context, expansion_stack, subcircuit)?;
            // TODO: implement inlining
            (expansion_stack, gate_map.insert(simulation::Gate::Custom(subcircuit_idx)))
        }
        ir::CircuitOrIntrinsic::Nand => {
            (expansion_stack, gate_map.insert_with_key(|gk| simulation::Gate::Nand { logic: logic::NandLogic::new(node_map, gk), location: simulation::location::GateLocation::new() }))
        }
        ir::CircuitOrIntrinsic::Const(value) => {
            (expansion_stack, gate_map.insert_with_key(|gk| simulation::Gate::Const { logic: logic::ConstLogic::new(node_map, gk, *value), location: simulation::location::GateLocation::new() }))
        }
    };

    circuit_map[new_circuit_idx].gates.push(gate_idx);
    Ok((expansion_stack, gate_idx))
}

fn connect(
    circuits: &mut simulation::CircuitMap,
    gates: &mut simulation::GateMap,
    nodes: &mut simulation::NodeMap,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: simulation::CircuitKey,
    gate_index_map: &mut HashMap<ir::GateIdx, simulation::GateKey>,
    producer: &ir::bundle::ProducerBundle,
    receiver: &ir::bundle::ReceiverBundle,
) {
    let producer_nodes: Vec<simulation::NodeKey> = convert_producer_bundle(circuits, gates, type_context, new_circuit, gate_index_map, producer);
    let receiver_nodes: Vec<simulation::NodeKey> = convert_receiver_bundle(circuits, gates, type_context, new_circuit, gate_index_map, receiver).to_vec(); // TODO: figure out better solution than to clone

    assert_eq!(producer_nodes.len(), receiver_nodes.len(), "connecting producer and receiver that have different size");

    for (p, r) in producer_nodes.into_iter().zip(receiver_nodes) {
        logic::connect(nodes, p, r);
    }
}

fn convert_producer_bundle(
    circuits: &simulation::CircuitMap,
    gates: &simulation::GateMap,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: simulation::CircuitKey,
    gate_index_map: &HashMap<ir::GateIdx, simulation::GateKey>,
    producer: &ir::bundle::ProducerBundle,
) -> Vec<simulation::NodeKey> {
    match producer {
        ir::bundle::ProducerBundle::CurCircuitInput(_) => circuits[new_circuit].inputs().to_vec(),
        ir::bundle::ProducerBundle::GateOutput(_, old_gate_index) => simulation::gate_outputs(circuits, gates, gate_index_map[old_gate_index]).to_owned(),
        ir::bundle::ProducerBundle::Get(b, field) => {
            fn field_indexes(type_context: &ty::TypeContext<nominal_type::FullyDefinedStruct>, fields: &[(&str, ty::TypeSym)], field: &str) -> Option<std::ops::Range<usize>> {
                let mut cur_index = 0;
                for (field_name, field_type) in fields {
                    let cur_type_size = type_context.get(*field_type).size(type_context);
                    if *field_name == field {
                        return Some(cur_index..cur_index + cur_type_size);
                    }
                    cur_index += cur_type_size;
                }

                None
            }
            let b_nodes = convert_producer_bundle(circuits, gates, type_context, new_circuit, gate_index_map, b);

            let b_type = b.type_(type_context);
            let field_indexes = field_indexes(type_context, &type_context.get(b_type).fields(type_context), field).expect("producer bundle should have field after type checking");

            b_nodes[field_indexes].to_vec()
        }
        ir::bundle::ProducerBundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_producer_bundle(circuits, gates, type_context, new_circuit, gate_index_map, sb)).collect(),
    }
}
fn convert_receiver_bundle<'a>(
    circuits: &'a simulation::CircuitMap, // keep arguments for symmetry with convert_producer_bundle
    gates: &'a simulation::GateMap,
    _: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: simulation::CircuitKey,
    gate_index_map: &HashMap<ir::GateIdx, simulation::GateKey>,
    receiver: &ir::bundle::ReceiverBundle,
) -> Vec<simulation::NodeKey> {
    match receiver {
        ir::bundle::ReceiverBundle::CurCircuitOutput(_) => circuits[new_circuit].outputs().to_vec(),
        ir::bundle::ReceiverBundle::GateInput(_, old_gate_index) => simulation::gate_inputs(circuits, gates, gate_index_map[old_gate_index]).to_vec(),
    }
}
