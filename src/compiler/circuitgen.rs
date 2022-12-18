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
}

impl From<CircuitGenError<'_>> for CompileError {
    fn from(val: CircuitGenError) -> Self {
        match val {
            CircuitGenError::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
            CircuitGenError::OutOfRange { local_name, local_size, index } => CompileError { message: format!("get out of range: '{local_name}' has a size of {local_size} and the index is {index}") },
            CircuitGenError::NoSuchLocal(name) => CompileError { message: format!("no local called '{name}'") },
            CircuitGenError::NoSuchCircuit(name) => CompileError { message: format!("no circuit called '`{name}'") },
        }
    }
}

pub(crate) fn generate(ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut circuit_name_table = HashMap::new();

    for circuit in ast {
        let (name, circuit) = convert_circuit(&circuit_name_table, circuit)?; // TODO: report multiple errors from this
        if circuit_name_table.contains_key(name) {
            CircuitGenError::Duplicate(name).report();
            None?
        } else {
            circuit_name_table.insert(name, circuit);
        }
    }
    todo!()
}

fn convert_circuit<'file>(circuit_name_table: &HashMap<&'file str, circuit::Circuit>, circuit_ast: ast::Circuit<'file>) -> Option<(&'file str, circuit::Circuit)> {
    let name = circuit_ast.name;

    let mut locals: HashMap<&str, Vec<circuit::Value>> = HashMap::new();
    let mut gates: Vec<circuit::Gate> = Vec::new();
    for r#let in circuit_ast.lets {
        let result = convert_expr(circuit_name_table, &mut gates, &mut locals, r#let.val);

        // TODO: assign results to locals
    }

    let outputs = convert_expr(circuit_name_table, &mut gates, &mut locals, circuit_ast.outputs)?;

    Some((name, circuit::Circuit { name: name.into(), num_inputs: pattern_size(&circuit_ast.inputs), gates, outputs }))
}

fn convert_expr<'file>(
    circuit_name_table: &HashMap<&'file str, circuit::Circuit>,
    gates: &mut Vec<circuit::Gate>,
    locals: &mut HashMap<&str, Vec<circuit::Value>>,
    exprs: Vec<ast::Expr>,
) -> Option<Vec<circuit::Value>> {
    let results = exprs.into_iter().map(|e| convert_single_expr(circuit_name_table, gates, locals, e));
    let results_no_none: Option<_> = results.collect::<Option<Vec<Vec<circuit::Value>>>>(); // TODO: dont stop at the first one in order to report all the errors
    Some(results_no_none?.into_iter().flatten().collect::<Vec<circuit::Value>>())
}

fn convert_single_expr<'file>(
    circuit_name_table: &HashMap<&'file str, circuit::Circuit>,
    gates: &mut Vec<circuit::Gate>,
    locals: &mut HashMap<&str, Vec<circuit::Value>>,
    expr: ast::Expr,
) -> Option<Vec<circuit::Value>> {
    match expr {
        ast::Expr::Ref(name, slots) => {
            let name_resolved = match locals.get(name) {
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
            let name_resolved = match circuit_name_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    CircuitGenError::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            let inputs = convert_expr(circuit_name_table, gates, locals, inputs)?;
            let gate = add_gate(gates, circuit::Gate::Custom(name_resolved.clone(), inputs));
            Some((0..name_resolved.outputs.len()).map(|i| circuit::Value::GateValue(gate, i)).collect())
        }

        ast::Expr::Const(val) => {
            let gate_i = add_gate(gates, circuit::Gate::Const(val));
            Some(vec![circuit::Value::GateValue(gate_i, 0)])
        }
    }
}

fn add_gate(gates: &mut Vec<circuit::Gate>, val: circuit::Gate) -> usize {
    gates.push(val);
    gates.len() - 1
}

fn pattern_size(arguments: &[ast::Pattern]) -> usize {
    arguments.iter().map(|ast::Pattern(_, size)| size).sum::<usize>()
}
