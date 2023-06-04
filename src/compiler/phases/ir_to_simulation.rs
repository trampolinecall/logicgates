use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, ir, nominal_type, ty},
        error::{CompileError, File, Report, Span},
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

struct UseInputsInMain<'file>(Span<'file>);
struct UseOutputsInMain<'file>(Span<'file>);
impl<'file> From<UseInputsInMain<'file>> for CompileError<'file> {
    fn from(UseInputsInMain(sp): UseInputsInMain<'file>) -> Self {
        todo!("use inputs in main")
        // CompileError::new(todo!("span"), "infinite recursion in gates".into())
    }
}
impl<'file> From<UseOutputsInMain<'file>> for CompileError<'file> {
    fn from(UseOutputsInMain(sp): UseOutputsInMain<'file>) -> Self {
        todo!("use outputs in main")
        // CompileError::new(todo!("span"), "infinite recursion in gates".into())
    }
}

pub(crate) fn convert(file: &File, ast_to_ir::IR { circuits, circuit_table, mut type_context }: ast_to_ir::IR) -> Option<simulation::Simulation> {
    if let Some((_, _, main_id)) = circuit_table.get("main") {
        if let ir::CircuitOrIntrinsic::Custom(circuit) = circuits.get(*main_id) {
            let mut gate_map = simulation::GateMap::with_key();
            let mut circuit_map = simulation::CircuitMap::with_key();
            let mut node_map = simulation::NodeMap::with_key();
            let mut connections = simulation::connections::Connections::new();
            let main_children = match convert_circuit_as_toplevel(&mut circuit_map, &mut gate_map, &mut node_map, &mut connections, &circuits, &mut type_context, Vec::new(), circuit) {
                Ok((_, r)) => r,
                Err(e) => {
                    e.report();
                    None?
                }
            };

            let mut simulation = simulation::Simulation { circuits: circuit_map, gates: gate_map, nodes: node_map, toplevel_gates: main_children, widget: simulation::ui::simulation::SimulationWidget::new(), connections: simulation::connections::Connections::new() };
            // simulation::location::calculate_locations(&mut simulation); TODO: figure out what to do with this

            Some(simulation)
        } else {
            unreachable!("builtin circuit called main")
        }
    } else {
        NoMain(file).report();
        None
    }
}
fn convert_circuit_as_toplevel<'file, 'circuit>(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    connections: &mut simulation::connections::Connections,
    circuits: &'circuit arena::Arena<ir::CircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    mut expansion_stack: ExpansionStack<'file, 'circuit>,
    circuit: &'circuit ir::Circuit<'file>,
) -> Result<(ExpansionStack<'file, 'circuit>, simulation::hierarchy::GateChildren), InfiniteRecursion<'file, 'circuit>> {
    if expansion_stack.iter().any(|c| std::ptr::eq(*c, circuit)) {
        return Err(InfiniteRecursion(expansion_stack, circuit));
    }

    expansion_stack.push(circuit);

    let mut main_gates = simulation::hierarchy::GateChildren::new();
    let mut gate_index_map = HashMap::new();

    for (old_gate_i, gate) in circuit.gates.iter_with_ids() {
        let (expansion_stack_2, new_gate_i) = lower_gate(circuit_map, gate_map, node_map, connections, circuits, type_context, expansion_stack, *gate)?;
        main_gates.add_gate(new_gate_i);
        expansion_stack = expansion_stack_2;
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (start, end) in &circuit.connections {
        connect(circuit_map, gate_map, node_map, connections, type_context, None, &mut gate_index_map, start, end);
    }

    assert!(std::ptr::eq(*expansion_stack.last().unwrap(), circuit), "expansion stack should be in the same state as at the start of the function");
    expansion_stack.pop();

    Ok((expansion_stack, main_gates))
}
fn convert_circuit<'file, 'circuit>(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    connections: &mut simulation::connections::Connections,
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
        let (expansion_stack_2, new_gate_i) = lower_gate(circuit_map, gate_map, node_map, connections, circuits, type_context, expansion_stack, *gate)?;
        circuit_map[new_circuit_idx].gates.add_gate(new_gate_i);
        expansion_stack = expansion_stack_2;
        gate_index_map.insert(old_gate_i, new_gate_i);
    }

    for (start, end) in &circuit.connections {
        connect(circuit_map, gate_map, node_map, connections, type_context, Some(new_circuit_idx), &mut gate_index_map, start, end);
    }

    assert!(std::ptr::eq(*expansion_stack.last().unwrap(), circuit), "expansion stack should be in the same state as at the start of the function");
    expansion_stack.pop();

    Ok((expansion_stack, new_circuit_idx))
}

fn lower_gate<'file, 'circuit>(
    circuit_map: &mut simulation::CircuitMap,
    gate_map: &mut simulation::GateMap,
    node_map: &mut simulation::NodeMap,
    connections: &mut simulation::connections::Connections,
    circuits: &'circuit arena::Arena<ir::CircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    expansion_stack: ExpansionStack<'file, 'circuit>,
    (circuit_id, _): (ast::CircuitOrIntrinsicId, ir::Inline),
) -> Result<(ExpansionStack<'file, 'circuit>, simulation::GateKey), InfiniteRecursion<'file, 'circuit>> {
    let (expansion_stack, gate_key) = match circuits.get(circuit_id) {

        ir::CircuitOrIntrinsic::Custom(subcircuit) => {
            let (expansion_stack, subcircuit_idx) = convert_circuit(circuit_map, gate_map, node_map, connections, circuits, type_context, expansion_stack, subcircuit)?;
            // TODO: implement inlining
            (expansion_stack, gate_map.insert(simulation::Gate::Custom(subcircuit_idx)))
        }
        ir::CircuitOrIntrinsic::Nand => {
            (expansion_stack, gate_map.insert_with_key(|gk| simulation::Gate::Nand { logic: logic::NandLogic::new(node_map, gk), widget: simulation::ui::GateWidget::new(), location: simulation::location::GateLocation::new() }))
        }
        ir::CircuitOrIntrinsic::Const(value) => {
            (expansion_stack, gate_map.insert_with_key(|gk| simulation::Gate::Const { logic: logic::ConstLogic::new(node_map, gk, *value), widget: simulation::ui::GateWidget::new(), location: simulation::location::GateLocation::new() }))
        }
        ir::CircuitOrIntrinsic::Unerror => {
            (expansion_stack, gate_map.insert_with_key(|gk| simulation::Gate::Unerror { logic: logic::UnerrorLogic::new(node_map, gk), widget: simulation::ui::GateWidget::new(), location: simulation::location::GateLocation::new() }))
        }
    };

    Ok((expansion_stack, gate_key))
}

fn connect(
    circuits: &mut simulation::CircuitMap,
    gates: &mut simulation::GateMap,
    nodes: &mut simulation::NodeMap,
    connections: &mut simulation::connections::Connections,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: Option<simulation::CircuitKey>,
    gate_index_map: &mut HashMap<ir::GateIdx, simulation::GateKey>,
    start: &ir::bundle::Bundle,
    end: &ir::bundle::Bundle,
) {
    let start_nodes: Vec<simulation::NodeKey> = convert_bundle(circuits, gates, type_context, new_circuit, gate_index_map, start);
    let end_nodes: Vec<simulation::NodeKey> = convert_bundle(circuits, gates, type_context, new_circuit, gate_index_map, end);

    assert_eq!(start_nodes.len(), end_nodes.len(), "connecting bundles that have different size");

    for (p, r) in start_nodes.into_iter().zip(end_nodes) {
        simulation::connections::connect(connections, nodes, p, r);
    }
}

fn convert_bundle(
    circuits: &simulation::CircuitMap,
    gates: &simulation::GateMap,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    new_circuit: Option<simulation::CircuitKey>,
    gate_index_map: &HashMap<ir::GateIdx, simulation::GateKey>,
    bundle: &ir::bundle::Bundle,
) -> Vec<simulation::NodeKey> {
    match bundle {
        ir::bundle::Bundle::CurCircuitInput(_) => {
            if let Some(new_circuit) = new_circuit {
                circuits[new_circuit].nodes.inputs().to_vec()
            } else {
                UseInputsInMain(todo!()).report();
                vec![] // correct anyways because the inputs always have type [] in main
            }
        }
        ir::bundle::Bundle::CurCircuitOutput(_) => {
            if let Some(new_circuit) = new_circuit {
                circuits[new_circuit].nodes.outputs().to_vec()
            } else {
                UseOutputsInMain(todo!()).report();
                vec![] // correct anyways because the outputs always have type [] in main
            }
        }
        ir::bundle::Bundle::GateInput(_, old_gate_index) => simulation::Gate::inputs(circuits, gates, gate_index_map[old_gate_index]).to_vec(),
        ir::bundle::Bundle::GateOutput(_, old_gate_index) => simulation::Gate::outputs(circuits, gates, gate_index_map[old_gate_index]).to_owned(),
        ir::bundle::Bundle::Get(b, field) => {
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
            let b_nodes = convert_bundle(circuits, gates, type_context, new_circuit, gate_index_map, b);

            let b_type = b.type_(type_context);
            let field_indexes = field_indexes(type_context, &type_context.get(b_type).fields(type_context), field).expect("get bundle should not get nonexistent field after type checking");

            b_nodes[field_indexes].to_vec()
        }
        ir::bundle::Bundle::Product(subbundles) => subbundles.iter().flat_map(|(_, sb)| convert_bundle(circuits, gates, type_context, new_circuit, gate_index_map, sb)).collect(),
    }
}
