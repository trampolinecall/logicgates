mod bundle;
mod circuit_def;

use std::collections::HashMap;

use generational_arena::Arena;

use crate::circuit;

use bundle::ProducerBundle;
use bundle::ReceiverBundle;
use circuit_def::CircuitDef;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum Error<'file> {
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

type CircuitTable<'file> = HashMap<&'file str, CircuitDef>;
type Gates<'gates> = &'gates mut Arena<circuit::Gate>;
struct GlobalGenState<'file> {
    circuit_table: CircuitTable<'file>,
}

impl<'file, 'gates> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("and", CircuitDef::AndBuiltin);
        circuit_table.insert("not", CircuitDef::NotBuiltin);
        Self { circuit_table }
    }
}


fn connect_bundle(gates: Gates, circuit: &mut circuit::Circuit, producer_bundle: ProducerBundle, receiver_bundle: ReceiverBundle) -> Option<()> {
    if producer_bundle.type_() != receiver_bundle.type_() {
        Error::TypeMismatchInCall { actual_type: producer_bundle.type_(), expected_type: receiver_bundle.type_() }.report();
        None?
    }

    for (producer_node, receiver_node) in producer_bundle.flatten().into_iter().zip(receiver_bundle.flatten().into_iter()) {
        circuit.connect(gates, producer_node, receiver_node)
    }

    Some(())
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

impl From<Error<'_>> for CompileError {
    fn from(val: Error) -> Self {
        match val {
            Error::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
            Error::OutOfRange { expr_size: local_size, index } => CompileError { message: format!("get out of range: expression has a size of {local_size} and the index is {index}") },
            Error::NoSuchLocal(name) => CompileError { message: format!("no local called '{name}'") },
            Error::NoSuchCircuit(name) => CompileError { message: format!("no circuit called '`{name}'") },
            Error::TypeMismatchInAssignment { value_type: value_size, pattern_type: pattern_size } => {
                CompileError { message: format!("size mismatch in assignment: value has size {value_size} but pattern has size {pattern_size}") }
            }
            Error::NoMain => CompileError { message: "no '`main' circuit".into() },
            Error::TypeMismatchInCall { actual_type, expected_type } => {
                CompileError { message: format!("type mismatch in subcircuit: arguments have type {actual_type} but subcircuit expects type {expected_type}") }
            }
            Error::ArgNumMismatchInCall { actual_arity, expected_arity } => {
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
            Error::Duplicate(name).report();
            None?
        } else {
            global_state.circuit_table.insert(name, CircuitDef::Circuit(circuit));
        }
    }

    match global_state.circuit_table.remove("main") {
        Some(CircuitDef::Circuit(r)) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            Error::NoMain.report();
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

        if result.type_() != type_ {
            Error::TypeMismatchInAssignment { value_type: result.type_(), pattern_type: type_ }.report();
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
                    Error::NoSuchLocal(name).report();
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
                    Error::NoSuchCircuit(circuit_name).report();
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
