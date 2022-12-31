use std::collections::HashMap;

use crate::circuit;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum CircuitGenError<'file> {
    Duplicate(&'file str),
    OutOfRange { expr_size: usize, index: usize },
    NoSuchLocal(&'file str),
    NoSuchCircuit(&'file str),
    SizeMismatchInAssignment { value_size: usize, pattern_size: usize },
    NoMain,
    SizeMismatchInCall { actual_size: usize, expected_size: usize },
    // AndWith0 { actual_size: usize },
}

#[derive(Default)]
struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, CircuitEntity>,
}

impl<'file> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("and", CircuitEntity::AndBuiltin);
        circuit_table.insert("not", CircuitEntity::NotBuiltin);
        Self { circuit_table }
    }
}

enum CircuitEntity {
    Circuit(circuit::Circuit),
    AndBuiltin,
    NotBuiltin,
}
impl CircuitEntity {
    fn to_gate(&self, circuit_state: &mut CircuitGenState) -> circuit::GateIndex {
        // TODO: refactor this and probably refactor the rest of the module too
        match self {
            CircuitEntity::Circuit(c) => circuit_state.circuit.new_subcircuit_gate(c.clone()),
            CircuitEntity::AndBuiltin => circuit_state.circuit.new_and_gate(),
            CircuitEntity::NotBuiltin => circuit_state.circuit.new_not_gate(),
        }
    }

    fn add_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<circuit::ValueProducingNodeIdx>) -> Option<Vec<circuit::ValueProducingNodeIdx>> {
        let gate_i = self.to_gate(circuit_state);
        let gate = circuit_state.circuit.get_gate(gate_i);

        // connect the inputs
        let num_inputs = inputs.len();
        let expected_num_inputs = gate.num_inputs();
        if num_inputs != expected_num_inputs {
            CircuitGenError::SizeMismatchInCall { actual_size: num_inputs, expected_size: expected_num_inputs }.report();
            None?
        }

        for (input_value, gate_input_node) in inputs.into_iter().zip(gate.inputs().collect::<Vec<_>>().into_iter()) {
            // TODO: find a better way to do this than to basically clone the inputs Vec, needed because inputs() borrows the gate, which borrows the circuit
            circuit_state.circuit.connect(input_value, gate_input_node.into());
        }

        Some(circuit_state.circuit.get_gate(gate_i).outputs().map(|o| o.into()).collect())
    }

    fn inline_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<circuit::ValueProducingNodeIdx>) -> Option<Vec<circuit::ValueProducingNodeIdx>> {
        if let CircuitEntity::Circuit(subcircuit) = self {
            use crate::circuit::GateIndex;

            let mut gate_number_mapping: HashMap<GateIndex, GateIndex> = HashMap::new();
            let convert_producer_idx = |p, circuit: &circuit::Circuit, gate_number_mapping: &HashMap<GateIndex, GateIndex>| match p {
                circuit::ValueProducingNodeIdx::CI(ci) => inputs[ci.0],
                circuit::ValueProducingNodeIdx::GO(go) => circuit::ValueProducingNodeIdx::GO(circuit.get_gate(gate_number_mapping[&go.0]).outputs().nth(go.1).expect("gate index should be in range for the same gate type when converting producer index for inlining subcircuit")),
            };

            for (subcircuit_gate_i, gate) in subcircuit.gates.iter() {
                let (inner_inputs, gate_added_to_main_circuit) = match &gate.kind {
                    circuit::GateKind::And(inputs, _) => {
                        (&inputs[..], circuit_state.circuit.new_and_gate())
                    }
                    circuit::GateKind::Not(inputs, _) => {
                        (&inputs[..], circuit_state.circuit.new_not_gate())
                    }
                    circuit::GateKind::Const(inputs, [circuit::ValueProducingNode { value, .. }]) => {
                        (&inputs[..], circuit_state.circuit.new_const_gate(*value))
                    }
                    circuit::GateKind::Subcircuit(inputs, _, subcircuit) => {
                        (&inputs[..], circuit_state.circuit.new_subcircuit_gate(subcircuit.borrow().clone()))
                    }
                };

                for (input, new_gate_input) in inner_inputs.iter().zip(circuit_state.circuit.get_gate(gate_added_to_main_circuit).inputs().collect::<Vec<_>>().into_iter()) { // TODO: dont clone this
                    if let Some(inner_producer_idx) = input.producer {
                        circuit_state.circuit.connect(convert_producer_idx(inner_producer_idx, &circuit_state.circuit, &gate_number_mapping), new_gate_input.into())
                    }
                }

                gate_number_mapping.insert(subcircuit_gate_i, gate_added_to_main_circuit);
            }

            Some(subcircuit.output_indexes().flat_map(|o| subcircuit.get_value_receiving_node(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping))).collect()) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, inputs)
        }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, Vec<circuit::ValueProducingNodeIdx>>,
    circuit: circuit::Circuit,
}
impl CircuitGenState<'_> {
    fn new(name: String) -> Self {
        Self { locals: HashMap::<_, _>::default(), circuit: circuit::Circuit::new(name) }
    }
}

impl From<CircuitGenError<'_>> for CompileError {
    fn from(val: CircuitGenError) -> Self {
        match val {
            CircuitGenError::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
            CircuitGenError::OutOfRange { expr_size: local_size, index } => CompileError { message: format!("get out of range: expression has a size of {local_size} and the index is {index}") },
            CircuitGenError::NoSuchLocal(name) => CompileError { message: format!("no local called '{name}'") },
            CircuitGenError::NoSuchCircuit(name) => CompileError { message: format!("no circuit called '`{name}'") },
            CircuitGenError::SizeMismatchInAssignment { value_size, pattern_size } => {
                CompileError { message: format!("size mismatch in assignment: value has size {value_size} but pattern has size {pattern_size}") }
            }
            CircuitGenError::NoMain => CompileError { message: "no '`main' circuit".into() },
            CircuitGenError::SizeMismatchInCall { actual_size, expected_size } => {
                CompileError { message: format!("size mismatch in subcircuit: arguments have size {actual_size} but subcircuit expects size {expected_size}") }
            } // CircuitGenError::AndWith0 { actual_size } => CompileError { message: format!("'`and' gate needs a size of at least 1 but got size {actual_size}") },
        }
    }
}

pub(crate) fn generate(ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new();

    for circuit in ast {
        let (name, circuit) = convert_circuit(&global_state, circuit)?; // TODO: report multiple errors from this
        if global_state.circuit_table.contains_key(name) {
            CircuitGenError::Duplicate(name).report();
            None?
        } else {
            global_state.circuit_table.insert(name, CircuitEntity::Circuit(circuit));
        }
    }

    match global_state.circuit_table.remove("main") {
        Some(CircuitEntity::Circuit(r)) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            CircuitGenError::NoMain.report();
            None?
        }
    }
}

fn convert_circuit<'file>(global_state: &GlobalGenState, circuit_ast: ast::Circuit<'file>) -> Option<(&'file str, circuit::Circuit)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.to_string());
    circuit_state.circuit.set_num_inputs(circuit_ast.inputs.iter().map(|arg_pat| arg_pat.1).sum());

    assert_eq!(circuit_ast.inputs.iter().map(|arg_pat| arg_pat.1).sum::<usize>(), circuit_state.circuit.input_indexes().collect::<Vec<_>>().len());
    let mut input_idxs = circuit_state.circuit.input_indexes();
    for arg_pat in circuit_ast.inputs.iter() {
        let args = (0..arg_pat.1).map(|_| circuit::ValueProducingNodeIdx::CI(input_idxs.next().expect("input_idxs should have the same length as the pattern length"))).collect();
        circuit_state.locals.insert(arg_pat.0, args);
    }

    for ast::Let { pat, val } in circuit_ast.lets {
        let result = convert_expr(global_state, &mut circuit_state, val)?;

        if result.len() != pattern_size(&pat) {
            CircuitGenError::SizeMismatchInAssignment { value_size: result.len(), pattern_size: pattern_size(&pat) }.report();
            None?
        }

        let mut result = result.into_iter();
        for sub_pat in pat {
            circuit_state.locals.insert(sub_pat.0, (0..sub_pat.1).map(|_| result.next().unwrap()).collect());
        }
    }

    let output_values = convert_expr(global_state, &mut circuit_state, circuit_ast.outputs)?;
    circuit_state.circuit.set_num_outputs(output_values.len());
    assert_eq!(circuit_state.circuit.num_outputs(), output_values.len(), "number of circuit outputs should be equal to the number of output producers");
    for (output_producer, output_receiver) in output_values.into_iter().zip(circuit_state.circuit.output_indexes()) {
        circuit_state.circuit.connect(output_producer, output_receiver.into());
    }

    circuit_state.circuit.calculate_locations();

    Some((name, circuit_state.circuit))
}

fn convert_expr(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: ast::Expr) -> Option<Vec<circuit::ValueProducingNodeIdx>> {
    match expr {
        ast::Expr::Ref(name) => {
            let name_resolved = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    CircuitGenError::NoSuchLocal(name).report();
                    None?
                }
            };

            Some(name_resolved.clone())
        }

        ast::Expr::Call(circuit_name, inline, inputs) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    CircuitGenError::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            let inputs = convert_expr(global_state, circuit_state, *inputs)?;
            if inline {
                name_resolved.inline_gate(circuit_state, inputs)
            } else {
                name_resolved.add_gate(circuit_state, inputs)
            }
        }

        ast::Expr::Const(val) => {
            let gate_i = circuit_state.circuit.new_const_gate(val);
            Some(circuit_state.circuit.get_gate(gate_i).outputs().map(|t| t.into()).collect())
        }

        ast::Expr::Get(expr, slots) => {
            let expr = convert_expr(global_state, circuit_state, *expr)?;
            slots
                .into_iter()
                .map(|slot| {
                    if slot < expr.len() {
                        Some(expr[slot])
                    } else {
                        CircuitGenError::OutOfRange { expr_size: expr.len(), index: slot }.report();
                        None
                    }
                })
                .collect::<Option<_>>()
        }
        ast::Expr::Multiple(exprs) => {
            let results = exprs.into_iter().map(|e| convert_expr(global_state, circuit_state, e));
            let results_no_none: Option<_> = results.collect::<Option<Vec<Vec<circuit::ValueProducingNodeIdx>>>>(); // TODO: dont stop at the first one in order to report all the errors
            Some(results_no_none?.into_iter().flatten().collect::<Vec<circuit::ValueProducingNodeIdx>>())
        }
    }
}

fn pattern_size(arguments: &[ast::Pattern]) -> usize {
    arguments.iter().map(|ast::Pattern(_, size)| size).sum::<usize>()
}
