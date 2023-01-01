mod bundle;
mod circuit_def;

use std::collections::HashMap;

use crate::circuit;

use bundle::ProducerBundle;
use circuit_def::CircuitDef;

use super::error::CompileError;
use super::error::Report;
use super::parser::ast;

enum Error<'file> {
    Duplicate(&'file str),
    NoField { ty: ast::Type, field_name: &'file str }, // TODO: list names of fields that do exist
    NoSuchLocal(&'file str),
    NoSuchCircuit(&'file str),
    TypeMismatchInAssignment { value_type: ast::Type, pattern_type: ast::Type },
    NoMain,
    ArgNumMismatchInCall { actual_arity: usize, expected_arity: usize },
    TypeMismatchInCall { actual_type: ast::Type, expected_type: ast::Type },
    // AndWith0 { actual_size: usize },
}

#[derive(Default)]
struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, CircuitDef>,
}

impl<'file> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("and", CircuitDef::And);
        circuit_table.insert("not", CircuitDef::Not);
        Self { circuit_table }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
    circuit: circuit::Circuit,
}
impl CircuitGenState<'_> {
    fn new(name: String) -> Self {
        Self { locals: HashMap::default(), circuit: circuit::Circuit::new(name) }
    }
}

impl From<Error<'_>> for CompileError {
    fn from(val: Error) -> Self {
        match val {
            Error::Duplicate(name) => CompileError { message: format!("circuit '{name}' defined more than once") },
            Error::NoField { ty, field_name } => CompileError { message: format!("no field called '{field_name}' on type '{ty}'") },
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

pub(crate) fn generate(ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new();

    for circuit in ast {
        let (name, circuit, input_types, result_type) = convert_circuit(&global_state, circuit)?; // TODO: report multiple errors from this
        if global_state.circuit_table.contains_key(name) {
            Error::Duplicate(name).report();
            None?
        } else {
            global_state.circuit_table.insert(name, CircuitDef::Circuit { circuit, input_types, result_type });
        }
    }

    match global_state.circuit_table.remove("main") {
        Some(CircuitDef::Circuit { circuit: r, .. }) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            Error::NoMain.report();
            None?
        }
    }
}

fn convert_circuit<'file>(global_state: &GlobalGenState, circuit_ast: ast::Circuit<'file>) -> Option<(&'file str, circuit::Circuit, Vec<ast::Type>, ast::Type)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.to_string());

    circuit_state.circuit.set_num_inputs(circuit_ast.inputs.iter().map(|(_, ty)| ty.size()).sum());
    assert_eq!(circuit_ast.inputs.iter().map(|(_, ty)| ty.size()).sum::<usize>(), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits");

    let mut input_idxs = circuit_state.circuit.input_indexes().map(|circuit_input_idx| circuit_input_idx.into());
    for (arg_pat, arg_ty) in circuit_ast.inputs.iter() {
        circuit_state.locals.insert(arg_pat.0, bundle::make_producer_bundle(&arg_ty, &mut input_idxs));
    }

    for ast::Let { pat, val, type_ } in circuit_ast.lets {
        let result = convert_expr(global_state, &mut circuit_state, val)?;

        if result.type_() != type_ {
            Error::TypeMismatchInAssignment { value_type: result.type_(), pattern_type: type_ }.report();
            None?
        }

        circuit_state.locals.insert(pat.0, result);
    }

    let output_value = convert_expr(global_state, &mut circuit_state, circuit_ast.outputs)?;
    circuit_state.circuit.set_num_outputs(output_value.type_().size());
    assert_eq!(circuit_state.circuit.num_outputs(), output_value.type_().size(), "number of circuit outputs should be equal to the number of output producers");

    let output_bundle = bundle::make_receiver_bundle(&output_value.type_(), &mut circuit_state.circuit.output_indexes().map(|output_idx| output_idx.into()));
    bundle::connect_bundle(&mut circuit_state.circuit, &output_value, &output_bundle);

    circuit_state.circuit.calculate_locations();

    Some((name, circuit_state.circuit, circuit_ast.inputs.into_iter().map(|(_, ty)| ty).collect(), output_value.type_()))
}

fn convert_expr(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: ast::Expr) -> Option<ProducerBundle> {
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
            let name_resolved = match global_state.circuit_table.get(circuit_name) {
                Some(n) => n,
                None => {
                    Error::NoSuchCircuit(circuit_name).report();
                    None?
                }
            };

            let inputs = inputs.into_iter().map(|input| convert_expr(global_state, circuit_state, input)).collect::<Option<Vec<_>>>()?;
            if inline {
                name_resolved.inline_gate(circuit_state, &inputs)
            } else {
                name_resolved.add_gate(circuit_state, &inputs)
            }
        }

        ast::Expr::Const(val) => CircuitDef::Const(val).add_gate(circuit_state, &[]),

        ast::Expr::Get(expr, field_name) => {
            let expr = convert_expr(global_state, circuit_state, *expr)?;
            let field = match &expr {
                ProducerBundle::Single(_) => None,
                ProducerBundle::Product(items) => match field_name.parse::<usize>() {
                    Ok(i) if i < items.len() => Some(items[i].clone()),
                    _ => None,
                },
            };
            if let Some(r) = field {
                Some(r)
            } else {
                Error::NoField { ty: expr.type_(), field_name }.report();
                None
            }
        }

        ast::Expr::Multiple(exprs) => {
            let results = exprs.into_iter().map(|e| convert_expr(global_state, circuit_state, e));
            let results_no_none = results.collect::<Option<Vec<ProducerBundle>>>()?; // TODO: dont stop at the first one in order to report all the errors
            Some(ProducerBundle::Product(results_no_none))
        }
    }
}
