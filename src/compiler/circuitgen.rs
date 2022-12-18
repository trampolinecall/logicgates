use std::collections::HashMap;

use crate::circuit;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum CircuitGenError<'file> {
    Duplicate(&'file str),
    OutOfRange { local_name: &'file str, local_size: usize, index: usize },
    NoSuchLocal(&'file str),
    NoSuchCircuit(&'file str),
    SizeMismatch { value_size: usize, pattern_size: usize },
    NoMain,
}

#[derive(Default)]
struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, CircuitDefinition>,
}

impl<'file> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("and", CircuitDefinition::AndBuiltin);
        circuit_table.insert("not", CircuitDefinition::NotBuiltin);
        Self { circuit_table }
    }
}

enum CircuitDefinition {
    Circuit(circuit::Circuit),
    AndBuiltin,
    NotBuiltin,
}
impl CircuitDefinition {
    fn to_gate(&self, inputs: Vec<circuit::Value>) -> circuit::Gate {
        match self {
            CircuitDefinition::Circuit(c) => circuit::Gate::Custom(c.clone(), inputs), // TODO: input size should match arity
            CircuitDefinition::AndBuiltin => circuit::Gate::And(inputs[0], inputs[1]), // TODO: input size should match arity
            CircuitDefinition::NotBuiltin => circuit::Gate::Not(inputs[0]),            // TODO: same todo
        }
    }
}

#[derive(Default)]
struct CircuitGenState<'file> {
    locals: HashMap<&'file str, Vec<circuit::Value>>,
    gates: Vec<circuit::Gate>,
}
impl CircuitGenState<'_> {
    fn add_gate(&mut self, val: circuit::Gate) -> usize {
        self.gates.push(val);
        self.gates.len() - 1
    }
}

impl From<CircuitGenError<'_>> for CompileError {
    fn from(val: CircuitGenError) -> Self {
        match val {
            CircuitGenError::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
            CircuitGenError::OutOfRange { local_name, local_size, index } => CompileError { message: format!("get out of range: '{local_name}' has a size of {local_size} and the index is {index}") },
            CircuitGenError::NoSuchLocal(name) => CompileError { message: format!("no local called '{name}'") },
            CircuitGenError::NoSuchCircuit(name) => CompileError { message: format!("no circuit called '`{name}'") },
            CircuitGenError::SizeMismatch { value_size, pattern_size } => {
                CompileError { message: format!("size mismatch in assignment: value has size {value_size} but pattern has size {pattern_size}") }
            }
            CircuitGenError::NoMain => CompileError { message: format!("no '`main' circuit") },
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
            global_state.circuit_table.insert(name, CircuitDefinition::Circuit(circuit));
        }
    }

    match global_state.circuit_table.remove("main") {
        Some(CircuitDefinition::Circuit(r)) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None =>  {
            CircuitGenError::NoMain.report();
            None?
        }
    }
}

fn convert_circuit<'file>(global_state: &GlobalGenState, circuit_ast: ast::Circuit<'file>) -> Option<(&'file str, circuit::Circuit)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::default();

    let mut arg_i = 0;
    for arg_pat in circuit_ast.inputs.iter() {
        let args = (0..arg_pat.1)
            .map(|_| {
                arg_i += 1;
                circuit::Value::Arg(arg_i - 1)
            })
            .collect();
        circuit_state.locals.insert(arg_pat.0, args);
    }

    for ast::Let { pat, val } in circuit_ast.lets {
        let result = convert_expr(global_state, &mut circuit_state, val)?;

        if result.len() != pattern_size(&pat) {
            CircuitGenError::SizeMismatch { value_size: result.len(), pattern_size: pattern_size(&pat) }.report();
            None?
        }

        let mut result = result.into_iter();
        for sub_pat in pat {
            circuit_state.locals.insert(sub_pat.0, (0..sub_pat.1).map(|_| result.next().unwrap()).collect());
        }
    }

    let outputs = convert_expr(global_state, &mut circuit_state, circuit_ast.outputs)?;

    Some((name, circuit::Circuit { name: name.into(), num_inputs: pattern_size(&circuit_ast.inputs), gates: circuit_state.gates, outputs }))
}

fn convert_expr<'file>(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, exprs: Vec<ast::Expr>) -> Option<Vec<circuit::Value>> {
    let results = exprs.into_iter().map(|e| convert_single_expr(global_state, circuit_state, e));
    let results_no_none: Option<_> = results.collect::<Option<Vec<Vec<circuit::Value>>>>(); // TODO: dont stop at the first one in order to report all the errors
    Some(results_no_none?.into_iter().flatten().collect::<Vec<circuit::Value>>())
}

fn convert_single_expr<'file>(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: ast::Expr) -> Option<Vec<circuit::Value>> {
    match expr {
        ast::Expr::Ref(name, slots) => {
            let name_resolved = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    CircuitGenError::NoSuchLocal(name).report();
                    None?
                }
            };

            slots
                .into_iter()
                .map(|slot| {
                    if slot < name_resolved.len() {
                        Some(name_resolved[slot])
                    } else {
                        CircuitGenError::OutOfRange { local_name: name, local_size: name_resolved.len(), index: slot }.report();
                        None
                    }
                })
                .collect::<Option<_>>()
        }

        ast::Expr::Call(circuit_name, inputs) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    CircuitGenError::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            let inputs = convert_expr(global_state, circuit_state, inputs)?;
            let gate = name_resolved.to_gate(inputs);
            let num_outputs = gate.num_outputs();
            let gate_i = circuit_state.add_gate(gate);
            Some((0..num_outputs).map(|i| circuit::Value::GateValue(gate_i, i)).collect())
        }

        ast::Expr::Const(val) => {
            let gate_i = circuit_state.add_gate(circuit::Gate::Const(val));
            Some(vec![circuit::Value::GateValue(gate_i, 0)])
        }
    }
}

fn pattern_size(arguments: &[ast::Pattern]) -> usize {
    arguments.iter().map(|ast::Pattern(_, size)| size).sum::<usize>()
}
