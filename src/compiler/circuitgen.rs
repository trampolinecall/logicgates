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
    AndWith0 { actual_size: usize },
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
    fn to_gate(&self, inputs: Vec<circuit::Value>) -> Option<circuit::Gate> {
        // TODO: refactor this and probably refactor the rest of the module too
        Some(match self {
            CircuitDefinition::Circuit(c) => {
                if inputs.len() != c.num_inputs {
                    CircuitGenError::SizeMismatchInCall { actual_size: inputs.len(), expected_size: c.num_inputs }.report();
                    None?
                } else {
                    circuit::Gate::Custom(c.clone(), inputs)
                }
            }
            CircuitDefinition::AndBuiltin => {
                if inputs.len() == 0 {
                    CircuitGenError::AndWith0 { actual_size: inputs.len() }.report();
                    None?
                } else {
                    circuit::Gate::And(inputs)
                }
            }
            CircuitDefinition::NotBuiltin => {
                if inputs.len() != 1 {
                    CircuitGenError::SizeMismatchInCall { actual_size: inputs.len(), expected_size: 1 }.report();
                    None?
                } else {
                    circuit::Gate::Not(inputs[0])
                }
            }
        })
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
            CircuitGenError::OutOfRange { expr_size: local_size, index } => CompileError { message: format!("get out of range: expression has a size of {local_size} and the index is {index}") },
            CircuitGenError::NoSuchLocal(name) => CompileError { message: format!("no local called '{name}'") },
            CircuitGenError::NoSuchCircuit(name) => CompileError { message: format!("no circuit called '`{name}'") },
            CircuitGenError::SizeMismatchInAssignment { value_size, pattern_size } => {
                CompileError { message: format!("size mismatch in assignment: value has size {value_size} but pattern has size {pattern_size}") }
            }
            CircuitGenError::NoMain => CompileError { message: format!("no '`main' circuit") },
            CircuitGenError::SizeMismatchInCall { actual_size, expected_size } => {
                CompileError { message: format!("size mismatch in subcircuit: arguments have size {actual_size} but subcircuit expects size {expected_size}") }
            }
            CircuitGenError::AndWith0 { actual_size } => {
                CompileError { message: format!("'`and' gate needs a size of at least 1 but got size {actual_size}") }

            }
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
        None => {
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
            CircuitGenError::SizeMismatchInAssignment { value_size: result.len(), pattern_size: pattern_size(&pat) }.report();
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

fn convert_expr<'file>(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: ast::Expr) -> Option<Vec<circuit::Value>> {
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

        ast::Expr::Call(circuit_name, inputs) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    CircuitGenError::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            let inputs = convert_expr(global_state, circuit_state, *inputs)?;
            let gate = name_resolved.to_gate(inputs)?;
            let num_outputs = gate.num_outputs();
            let gate_i = circuit_state.add_gate(gate);
            Some((0..num_outputs).map(|i| circuit::Value::GateValue(gate_i, i)).collect())
        }

        ast::Expr::Const(val) => {
            let gate_i = circuit_state.add_gate(circuit::Gate::Const(val));
            Some(vec![circuit::Value::GateValue(gate_i, 0)])
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
            let results_no_none: Option<_> = results.collect::<Option<Vec<Vec<circuit::Value>>>>(); // TODO: dont stop at the first one in order to report all the errors
            Some(results_no_none?.into_iter().flatten().collect::<Vec<circuit::Value>>())
        }
    }
}

fn pattern_size(arguments: &[ast::Pattern]) -> usize {
    arguments.iter().map(|ast::Pattern(_, size)| size).sum::<usize>()
}
