use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, ir, nominal_type, ty},
        phases::ast_to_ir,
    },
    utils::arena,
};

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<ir::CircuitOrIntrinsic<'file, ()>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn inline(ast_to_ir::IR { circuits, circuit_table, type_context }: ast_to_ir::IR) -> Option<IR> {
    let circuits = circuits.transform_dependent(
        |circuit, dep_getter| match circuit {
            ir::CircuitOrIntrinsic::Custom(subc) => arena::SingleTransformResult::Ok(ir::CircuitOrIntrinsic::Custom(try_transform_result!(inline_inline_calls(subc, dep_getter)))),
            ir::CircuitOrIntrinsic::Nand => arena::SingleTransformResult::Ok(ir::CircuitOrIntrinsic::Nand),
            ir::CircuitOrIntrinsic::Const(val) => arena::SingleTransformResult::Ok(ir::CircuitOrIntrinsic::Const(*val)),
        },
        |_, new| new,
    );
    let circuits = match circuits {
        Ok(circuits) => Some(circuits),
        Err((loops, errs)) => {
            for l in loops {
                todo!("report loop {l:?}")
            }
            for () in errs {}
            None
        }
    };
    Some(IR { circuits: circuits?, circuit_table, type_context })
}

fn inline_inline_calls<'file>(
    ir::Circuit { name, gates: old_gates, connections: old_connections, input_type, output_type }: &ir::Circuit<'file, ir::Inline>,
    dep_getter: arena::DependancyGetter<ir::CircuitOrIntrinsic<()>, ir::CircuitOrIntrinsic<ir::Inline>, (), ast::CircuitOrIntrinsicId>,
) -> arena::SingleTransformResult<ir::Circuit<'file, ()>, ast::CircuitOrIntrinsicId, ()> {
    let mut new_gate_inputs_and_outputs: HashMap<ir::GateIdx, (Vec<ir::bundle::ReceiverBundle>, Vec<ir::bundle::ProducerBundle>)> = HashMap::new();
    let mut circuit = ir::Circuit { name, gates: arena::Arena::new(), connections: Vec::new(), input_type: *input_type, output_type: *output_type };
    for (old_gate_id, (gate, inline)) in old_gates.iter_with_ids() {
        match inline {
            ir::Inline::Inline => match try_transform_result!(dep_getter.get(*gate)).1 {
                ir::CircuitOrIntrinsic::Custom(subc) => {
                    let mut inlining_gate_map = HashMap::new();
                    for (gate_id_in_subc, gate) in subc.gates.iter_with_ids() {
                        let gate_id_in_curc = circuit.gates.add(*gate);
                        inlining_gate_map.insert(gate_id_in_subc, gate_id_in_curc);
                    }

                    let connected_to_circuit_input: Vec<ir::bundle::ReceiverBundle> = Vec::new();
                    let connected_to_circuit_output: Vec<ir::bundle::ProducerBundle> = Vec::new();

                    fn convert_connection_start(gate_map: &HashMap<ir::GateIdx, ir::GateIdx>, producer: &ir::bundle::ProducerBundle) -> Option<ir::bundle::ProducerBundle> {
                        match producer {
                            ir::bundle::ProducerBundle::CurCircuitInput(_) => None, // things connected to the inputs and outputs of this will get connected after when other nodes in the parent circuit are connected to the inputs and outputs of this node
                            ir::bundle::ProducerBundle::GateOutput(ty, gate_idx_in_subcircuit) => {
                                Some(ir::bundle::ProducerBundle::GateOutput(*ty, *gate_map.get(gate_idx_in_subcircuit).expect("invalid connection when inlining gates")))
                            }
                            ir::bundle::ProducerBundle::Get(subbundle, field) => Some(ir::bundle::ProducerBundle::Get(convert_connection_start(subbundle), field.clone())),
                            ir::bundle::ProducerBundle::Product(_) => todo!(),
                        }
                    }
                    fn convert_connection_end(gate_map: &HashMap<ir::GateIdx, ir::GateIdx>, receiver: &ir::bundle::ReceiverBundle) -> Option<ir::bundle::ReceiverBundle> {
                        match receiver {
                            ir::bundle::ReceiverBundle::CurCircuitOutput(_) => None,
                            ir::bundle::ReceiverBundle::GateInput(ty, gate_idx_in_subcircuit) => {
                                Some(ir::bundle::ReceiverBundle::GateInput(*ty, *gate_map.get(gate_idx_in_subcircuit).expect("invalid connection when inlining gates")))
                            }
                        }
                    }

                    for (start, end) in &subc.connections {
                        let start = convert_connection_start(&inlining_gate_map, start);
                        let end = convert_connection_end(&inlining_gate_map, end);
                        match (start, end) {
                            (Some(start), Some(end)) => circuit.connections.push((start, end)),
                            (None, None) => {} // TODO: is this correct?
                            (None, Some(end)) => connected_to_circuit_input.push(end),
                            (Some(start), None) => connected_to_circuit_output.push(start),
                        }
                    }

                    new_gate_inputs_and_outputs.insert(old_gate_id, (connected_to_circuit_input, connected_to_circuit_output));
                }

                ir::CircuitOrIntrinsic::Nand | ir::CircuitOrIntrinsic::Const(_) => {
                    // these gates cannot be inlined
                    // TODO: figure out something better than to have these things copied and pasted
                    let new_gate_id = circuit.gates.add((*gate, ()));
                    new_gate_inputs_and_outputs
                        .insert(old_gate_id, (vec![ir::bundle::ReceiverBundle::GateInput(todo!(), new_gate_id)], vec![ir::bundle::ProducerBundle::GateOutput(todo!(), new_gate_id)]));
                }
            },
            ir::Inline::NoInline => {
                let new_gate_id = circuit.gates.add((*gate, ()));
                new_gate_inputs_and_outputs
                    .insert(old_gate_id, (vec![ir::bundle::ReceiverBundle::GateInput(todo!(), new_gate_id)], vec![ir::bundle::ProducerBundle::GateOutput(todo!(), new_gate_id)]));
            }
        }
    }

    for connection in old_connections {}

    arena::SingleTransformResult::Ok(circuit)
}
