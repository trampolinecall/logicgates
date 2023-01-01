use std::collections::HashMap;

use generational_arena::Arena;

use crate::circuit;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum CircuitGenError<'file> {
    Duplicate(&'file str),
    OutOfRange { expr_size: usize, index: usize },
    NoSuchLocal(&'file str),
    NoSuchCircuit(&'file str),
    TypeMismatchInAssignment { value_type: ast::Type, pattern_type: ast::Type },
    NoMain,
    ArgNumMismatchInCall { actual_arity: usize, expected_arity: usize },
    TypeMismatchInCall { actual_type: ast::Type, expected_type: ast::Type },
    // AndWith0 { actual_size: usize },
}

type CircuitTable<'file> = HashMap<&'file str, CircuitEntity>;
type Gates<'gates> = &'gates mut Arena<circuit::Gate>;
struct GlobalGenState<'file> {
    circuit_table: CircuitTable<'file>,
}

impl<'file, 'gates> GlobalGenState<'file> {
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
    fn to_gate(&self, gates: Gates, circuit_state: &mut CircuitGenState) -> (circuit::GateIndex, Vec<ReceiverBundle>, ProducerBundle) {
        // TODO: refactor this and probably refactor the rest of the module too
        match self {
            CircuitEntity::Circuit(c) => {
                let gate_i = circuit_state.circuit.new_subcircuit_gate(gates, c.clone());
                (gate_i, gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(), todo!()) // TODO: make this actually respond to types
            }
            CircuitEntity::AndBuiltin => {
                let gate_i = circuit_state.circuit.new_and_gate(gates);
                (gate_i, gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(), ProducerBundle::Single(gates[gate_i].outputs().nth(0).expect("and gate should have exactly one output").into()))
            }
            CircuitEntity::NotBuiltin => {
                let gate_i = circuit_state.circuit.new_not_gate(gates);
                (gate_i, gates[gate_i].inputs().map(|input| ReceiverBundle::Single(input.into())).collect(), ProducerBundle::Single(gates[gate_i].outputs().nth(0).expect("and gate should have exactly one output").into()))
            }
        }
    }

    fn add_gate(&self, gates: Gates, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
        let (gate_i, input_bundles, output_bundles) = self.to_gate(gates, circuit_state);
        let gate = &gates[gate_i];

        // connect the inputs
        let input_types: Vec<_> = inputs.iter().map(|bundle| bundle.type_()).collect();
        let expected_input_types: Vec<_> = input_bundles.iter().map(|bundle| bundle.type_()).collect();
        if input_types.len() != expected_input_types.len() {
            CircuitGenError::ArgNumMismatchInCall { actual_arity: input_types.len(), expected_arity: expected_input_types.len() }.report();
            None?
        }
        for (input_type, expected_type) in input_types.iter().zip(expected_input_types) {
            if *input_type != expected_type {}
        }

        for (producer_bundle, receiver_bundle) in inputs.into_iter().zip(input_bundles) {
            connect_bundle(gates, &mut circuit_state.circuit, producer_bundle, receiver_bundle)?;
            // circuit_state.circuit.connect(input_value, gate_input_node.into());
        }

        Some(output_bundles)
    }

    fn inline_gate(&self, circuit_state: &mut CircuitGenState, inputs: Vec<ProducerBundle>) -> Option<ProducerBundle> {
        todo!("inlining gates")
        /*
        if let CircuitEntity::Circuit(subcircuit) = self {
            use crate::circuit::GateIndex;

            let mut gate_number_mapping: HashMap<GateIndex, GateIndex> = HashMap::new();
            let convert_producer_idx = |p, circuit: &circuit::Circuit, gate_number_mapping: &HashMap<GateIndex, GateIndex>| match p {
                circuit::ProducerIdx::CI(ci) => inputs[ci.0],
                circuit::ProducerIdx::GO(go) => circuit::ProducerIdx::GO(
                    circuit
                        .get_gate(gate_number_mapping[&go.0])
                        .outputs()
                        .nth(go.1)
                        .expect("gate index should be in range for the same gate type when converting producer index for inlining subcircuit"),
                ),
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
                        circuit_state.circuit.connect(convert_producer_idx(inner_producer_idx, &circuit_state.circuit, &gate_number_mapping), new_gate_input.into())
                    }
                }

                gate_number_mapping.insert(subcircuit_gate_i, gate_added_to_main_circuit);
            }

            Some(
                subcircuit
                    .output_indexes()
                    .flat_map(|o| subcircuit.get_receiver(o.into()).producer.map(|producer| convert_producer_idx(producer, &circuit_state.circuit, &gate_number_mapping)))
                    .collect(),
            ) // TODO: allow unconnected nodes
        } else {
            self.add_gate(circuit_state, inputs)
        }
        */
    }

    fn expected_input_types(&self) -> Vec<ast::Type> {
        match self {
            CircuitEntity::AndBuiltin => vec![ast::Type::Bit, ast::Type::Bit],
            CircuitEntity::NotBuiltin => vec![ast::Type::Bit],
            CircuitEntity::Circuit(_) => todo!(),
        }
    }
}

fn connect_bundle(gates: Gates, circuit: &mut circuit::Circuit, producer_bundle: ProducerBundle, receiver_bundle: ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        CircuitGenError::TypeMismatchInCall { actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    for (producer_node, receiver_node) in producer_bundle.flatten().into_iter().zip(receiver_bundle.flatten().into_iter()) {
        circuit.connect(gates, producer_node, receiver_node)
    }

    Some(())
}

#[derive(Clone)]
enum ProducerBundle {
    Single(circuit::ProducerIdx),
    // List(Vec<ProducerBundle>),
}
enum ReceiverBundle {
    Single(circuit::ReceiverIdx),
}

impl ProducerBundle {
    fn size(&self) -> usize {
        match self {
            ProducerBundle::Single(_) => 1,
            // ProducerBundle::List(subbundles) => subbundles.iter().map(ProducerBundle::size).sum::<usize>(),
        }
    }
    fn type_(&self) -> ast::Type {
        match self {
            ProducerBundle::Single(_) => ast::Type::Bit,
            // ProducerBundle::List(_) => todo!(),
        }
    }

    fn flatten(&self) -> Vec<circuit::ProducerIdx> {
        match self {
            ProducerBundle::Single(i) => vec![*i],
            // ProducerBundle::List(subbundles) => subbundles.iter().flat_map(ProducerBundle::flatten).collect(),
        }
    }
}
impl ReceiverBundle {
    fn type_(&self) -> ast::Type {
        match self {
            ReceiverBundle::Single(_) => ast::Type::Bit,
        }
    }

    fn flatten(&self) -> Vec<circuit::ReceiverIdx> {
        match self {
            ReceiverBundle::Single(i) => vec![*i],
        }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
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
            CircuitGenError::TypeMismatchInAssignment { value_type: value_size, pattern_type: pattern_size } => {
                CompileError { message: format!("size mismatch in assignment: value has size {value_size} but pattern has size {pattern_size}") }
            }
            CircuitGenError::NoMain => CompileError { message: "no '`main' circuit".into() },
            CircuitGenError::TypeMismatchInCall { actual_type, expected_type } => {
                CompileError { message: format!("type mismatch in subcircuit: arguments have type {actual_type} but subcircuit expects type {expected_type}") }
            }
            CircuitGenError::ArgNumMismatchInCall { actual_arity, expected_arity } => {
                CompileError { message: format!("arity mismatch in subcircuit: passed {actual_arity} arguments but subcircuit expects {expected_arity}") }
            } // CircuitGenError::AndWith0 { actual_size } => CompileError { message: format!("'`and' gate needs a size of at least 1 but got size {actual_size}") },
        }
    }
}

pub(crate) fn generate(gates: &mut Arena<circuit::Gate>, ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new();

    for circuit in ast {
        let (name, circuit) = convert_circuit(&mut global_state, gates, circuit)?; // TODO: report multiple errors from this
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

fn convert_circuit<'file>(global_state: &GlobalGenState, gates: Gates, circuit_ast: ast::Circuit<'file>) -> Option<(&'file str, circuit::Circuit)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.to_string());
    circuit_state.circuit.set_num_inputs(circuit_ast.inputs.len());

    assert_eq!(circuit_ast.inputs.len(), circuit_state.circuit.input_indexes().collect::<Vec<_>>().len());
    let mut input_idxs = circuit_state.circuit.input_indexes();
    for arg_pat in circuit_ast.inputs.iter() {
        let producers = ProducerBundle::Single(input_idxs.next().expect("input_idxs should have the same length as the pattern length").into());
        circuit_state.locals.insert(arg_pat.0 .0, producers);
    }

    for ast::Let { pat, val, type_ } in circuit_ast.lets {
        let result = convert_expr(global_state, gates, &mut circuit_state, val)?;

        if result.size() != 1 {
            CircuitGenError::TypeMismatchInAssignment { value_type: result.type_(), pattern_type: type_ }.report();
            None?
        }

        let mut result = result.flatten().into_iter();
        for sub_pat in [pat] {
            circuit_state.locals.insert(sub_pat.0, ProducerBundle::Single(result.next().unwrap()));
        }
    }

    let output_values = convert_expr(global_state, gates, &mut circuit_state, circuit_ast.outputs)?;
    circuit_state.circuit.set_num_outputs(output_values.size());
    assert_eq!(circuit_state.circuit.num_outputs(), output_values.size(), "number of circuit outputs should be equal to the number of output producers");
    for (output_producer, output_receiver) in output_values.flatten().into_iter().zip(circuit_state.circuit.output_indexes()) {
        circuit_state.circuit.connect(gates, output_producer, output_receiver.into());
    }

    circuit_state.circuit.calculate_locations(gates);

    Some((name, circuit_state.circuit))
}

fn convert_expr(global_state: &GlobalGenState, gates: Gates, circuit_state: &mut CircuitGenState, expr: ast::Expr) -> Option<ProducerBundle> {
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
            let inputs = inputs.into_iter().map(|input| convert_expr(global_state, gates, circuit_state, input)).collect::<Option<Vec<_>>>()?;

            let name_resolved = match global_state.circuit_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    CircuitGenError::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            if inline {
                name_resolved.inline_gate(circuit_state, inputs)
            } else {
                name_resolved.add_gate(gates, circuit_state, inputs)
            }
        }

        ast::Expr::Const(val) => {
            let gate_i = circuit_state.circuit.new_const_gate(gates, val);
            todo!("const gate")
            // Some(circuit_state.circuit.get_gate(gate_i).outputs().map(|t| t.into()).collect())
        }

        ast::Expr::Get(expr, slots) => {
            let expr = convert_expr(global_state, gates, circuit_state, *expr)?;
            /*
            [slots]
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
                */
            todo!("get expr")
        }
        ast::Expr::Multiple(exprs) => {
            let results = exprs.into_iter().map(|e| convert_expr(global_state, gates, circuit_state, e));
            // let results_no_none: Option<_> = results.collect::<Option<Vec<Vec<circuit::ProducerIdx>>>>(); // TODO: dont stop at the first one in order to report all the errors
            // Some(results_no_none?.into_iter().flatten().collect::<Vec<circuit::ProducerIdx>>())
            todo!("multiple expr")
        }
    }
}
