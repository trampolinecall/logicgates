mod bundle;
mod circuit_def;

use std::collections::HashMap;

use crate::circuit;

use bundle::ProducerBundle;
use circuit_def::CircuitDef;

use super::error::CompileError;
use super::error::File;
use super::error::Report;
use super::error::Span;
use super::parser::ast;

enum Error<'file, 'ast> {
    Duplicate(Span<'file>),
    NoField { ty: ast::Type, field_name: Span<'file> }, // TODO: list names of fields that do exist
    NoSuchLocal(Span<'file>),
    NoSuchCircuit(Span<'file>),
    TypeMismatchInAssignment { pat: &'ast ast::Pattern<'file>, actual_type: ast::Type, pattern_type: ast::Type },
    TypeMismatchInCall { expr: &'ast ast::Expr<'file>, actual_type: ast::Type, expected_type: ast::Type },
    NoMain(&'file File),
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

impl<'file> From<Error<'file, '_>> for CompileError<'file> {
    fn from(val: Error<'file, '_>) -> Self {
        match val {
            Error::Duplicate(name) => CompileError::new(name, format!("circuit '{}' defined more than once", name.slice())),
            Error::NoField { ty, field_name } => CompileError::new(field_name, format!("no field called '{}' on type '{ty}'", field_name.slice())),
            Error::NoSuchLocal(name) => CompileError::new(name, format!("no local called '{}'", name.slice())),
            Error::NoSuchCircuit(name) => CompileError::new(name, format!("no circuit called '`{}'", name.slice())),
            Error::TypeMismatchInAssignment { pat, actual_type: value_type, pattern_type } => {
                let x: CompileError<'file> = CompileError::new(pat.span(), format!("type mismatch in assignment: value has type {value_type} but pattern has type {pattern_type}"));
                x
            }
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no '`main' circuit".into()),
            Error::TypeMismatchInCall { actual_type, expected_type, expr } => {
                let x: CompileError<'file> = CompileError::new(expr.span(), format!("type mismatch in subcircuit: arguments have type {actual_type} but subcircuit expects type {expected_type}"));
                x
            } // CircuitGenError::AndWith0 { actual_size } => CompileError::new(todo!(), format!("'`and' gate needs a size of at least 1 but got size {actual_size}")),
        }
    }
}

pub(crate) fn generate(file: &File, ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new();

    for circuit in ast {
        let (name, circuit, input_type, result_type) = convert_circuit(&global_state, circuit)?; // TODO: report multiple errors from this
        if global_state.circuit_table.contains_key(name.slice()) {
            Error::Duplicate(name).report();
            None?
        } else {
            global_state.circuit_table.insert(name.slice(), CircuitDef::Circuit { circuit, input_type, result_type });
        }
    }

    match global_state.circuit_table.remove("main") {
        Some(CircuitDef::Circuit { circuit: r, .. }) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            Error::NoMain(file).report();
            None?
        }
    }
}

fn convert_circuit<'file>(global_state: &GlobalGenState, circuit_ast: ast::Circuit<'file>) -> Option<(Span<'file>, circuit::Circuit, ast::Type, ast::Type)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.slice().to_string());

    circuit_state.circuit.set_num_inputs(circuit_ast.input.type_().size());
    assert_eq!(circuit_ast.input.type_().size(), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits");

    let input_bundle = bundle::make_producer_bundle(&circuit_ast.input.type_(), &mut circuit_state.circuit.input_indexes().map(|circuit_input_idx| circuit_input_idx.into()));
    if let Err(e) = assign_pattern(&mut circuit_state, &circuit_ast.input, input_bundle) {
        e.report();
    }

    for ast::Let { pat, val } in circuit_ast.lets.iter() {
        let result = convert_expr(global_state, &mut circuit_state, &val)?;
        if let Err(e) = assign_pattern(&mut circuit_state, &pat, result) {
            e.report();
        }
    }

    let output_value = convert_expr(global_state, &mut circuit_state, &circuit_ast.output)?;
    circuit_state.circuit.set_num_outputs(output_value.type_().size());
    assert_eq!(circuit_state.circuit.num_outputs(), output_value.type_().size(), "number of circuit outputs should be equal to the number of output producers");

    let output_bundle = bundle::make_receiver_bundle(&output_value.type_(), &mut circuit_state.circuit.output_indexes().map(|output_idx| output_idx.into()));
    bundle::connect_bundle(&mut circuit_state.circuit, &circuit_ast.output, &output_value, &output_bundle);

    circuit_state.circuit.calculate_locations();

    Some((name, circuit_state.circuit, circuit_ast.input.type_(), output_value.type_()))
}

fn assign_pattern<'ast, 'file>(circuit_state: &mut CircuitGenState<'file>, pat: &'ast ast::Pattern<'file>, bundle: ProducerBundle) -> Result<(), Error<'ast, 'file>> {
    if bundle.type_() != pat.type_() {
        Err(Error::TypeMismatchInAssignment { pat, actual_type: bundle.type_(), pattern_type: pat.type_() })?
    }

    match (pat, bundle) {
        (ast::Pattern::Identifier(iden, _), bundle) => {
            circuit_state.locals.insert(iden.slice(), bundle);
        }
        (ast::Pattern::Product(_, subpats), ProducerBundle::Product(subbundles)) => {
            assert_eq!(subpats.len(), subbundles.len(), "assign product pattern to procut bundle with different length"); // sanity check
            for (subpat, subbundle) in subpats.iter().zip(subbundles) {
                assign_pattern(circuit_state, subpat, subbundle)?;
            }
        }

        (pat, bundle) => unreachable!("assign pattern to bundle with different type after type checking: pattern = {pat:?}, bundle = {bundle:?}"),
    }

    Ok(())
}

fn convert_expr(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: &ast::Expr) -> Option<ProducerBundle> {
    match expr {
        ast::Expr::Ref(name) => {
            let name_resolved = match circuit_state.locals.get(name.slice()) {
                Some(resolved) => resolved,
                None => {
                    Error::NoSuchLocal(*name).report();
                    None?
                }
            };

            Some(name_resolved.clone())
        }

        ast::Expr::Call(circuit_name, inline, input) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name.slice()) {
                Some(n) => n,
                None => {
                    Error::NoSuchCircuit(*circuit_name).report();
                    None?
                }
            };

            let input = convert_expr(global_state, circuit_state, input)?;
            if *inline {
                name_resolved.inline_gate(circuit_state, &expr, input)
            } else {
                name_resolved.add_gate(circuit_state, &expr, input)
            }
        }

        ast::Expr::Const(_, val) => CircuitDef::Const(*val).add_gate(circuit_state, &expr, ProducerBundle::Product(Vec::new())),

        ast::Expr::Get(expr, field_name) => {
            let expr = convert_expr(global_state, circuit_state, expr)?;
            let field = match &expr {
                ProducerBundle::Single(_) => None,
                ProducerBundle::Product(items) => match field_name.slice().parse::<usize>() {
                    Ok(i) if i < items.len() => Some(items[i].clone()),
                    _ => None,
                },
            };
            if let Some(r) = field {
                Some(r)
            } else {
                Error::NoField { ty: expr.type_(), field_name: *field_name }.report();
                None
            }
        }

        ast::Expr::Multiple(_, exprs) => {
            let results = exprs.into_iter().map(|e| convert_expr(global_state, circuit_state, e));
            let results_no_none = results.collect::<Option<Vec<ProducerBundle>>>()?; // TODO: dont stop at the first one in order to report all the errors
            Some(ProducerBundle::Product(results_no_none))
        }
    }
}
