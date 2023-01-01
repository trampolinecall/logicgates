mod bundle;
mod circuit_def;
mod ty;

use std::collections::HashMap;

use crate::circuit;

use bundle::ProducerBundle;
use circuit_def::CircuitDef;

use super::error::CompileError;
use super::error::File;
use super::error::Report;
use super::error::Span;
use super::parser::ast;

enum Error<'file> {
    Duplicate(Span<'file>, &'file str),
    NoField { ty: ty::Type, field_name_sp: Span<'file>, field_name: &'file str }, // TODO: list names of fields that do exist
    NoSuchLocal(Span<'file>, &'file str),
    NoSuchCircuit(Span<'file>, &'file str),
    TypeMismatchInAssignment { pat_sp: Span<'file>, actual_type: ty::Type, pattern_type: ty::Type },
    TypeMismatchInCall { expr_span: Span<'file>, actual_type: ty::Type, expected_type: ty::Type },
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

impl<'file> From<Error<'file>> for CompileError<'file> {
    fn from(val: Error<'file>) -> Self {
        match val {
            Error::Duplicate(name_sp, name) => CompileError::new(name_sp, format!("circuit '{}' defined more than once", name)),
            Error::NoField { ty, field_name_sp, field_name } => CompileError::new(field_name_sp, format!("no field called '{}' on type '{ty}'", field_name)),
            Error::NoSuchLocal(name_sp, name) => CompileError::new(name_sp, format!("no local called '{}'", name)),
            Error::NoSuchCircuit(name_sp, name) => CompileError::new(name_sp, format!("no circuit called '`{}'", name)),
            Error::TypeMismatchInAssignment { pat_sp, actual_type: value_type, pattern_type } => {
                CompileError::new(pat_sp, format!("type mismatch in assignment: value has type {value_type} but pattern has type {pattern_type}"))
            }
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no '`main' circuit".into()),
            Error::TypeMismatchInCall { actual_type, expected_type, expr_span } => {
                CompileError::new(expr_span, format!("type mismatch in subcircuit: arguments have type {actual_type} but subcircuit expects type {expected_type}"))
            }
        }
    }
}

pub(crate) fn generate(file: &File, ast: Vec<ast::Circuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new();

    for circuit in ast {
        let ((name_sp, name), circuit, input_type, result_type) = convert_circuit(&global_state, circuit)?; // TODO: report multiple errors from this
        if global_state.circuit_table.contains_key(name) {
            Error::Duplicate(name_sp, name).report();
            None?
        } else {
            global_state.circuit_table.insert(name, CircuitDef::Circuit { circuit, input_type, result_type });
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

fn convert_circuit<'file>(global_state: &GlobalGenState, circuit_ast: ast::Circuit<'file>) -> Option<((Span<'file>, &'file str), circuit::Circuit, ty::Type, ty::Type)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.1.to_string());

    circuit_state.circuit.set_num_inputs(ty::Type::pat_type(&circuit_ast.input).size());
    assert_eq!(ty::Type::pat_type(&circuit_ast.input).size(), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits");

    let input_bundle = bundle::make_producer_bundle(&ty::Type::pat_type(&circuit_ast.input), &mut circuit_state.circuit.input_indexes().map(|circuit_input_idx| circuit_input_idx.into()));
    if let Err(e) = assign_pattern(&mut circuit_state, &circuit_ast.input, input_bundle) {
        e.report();
    }

    for ast::Let { pat, val } in circuit_ast.lets {
        let result = convert_expr(global_state, &mut circuit_state, val)?;
        if let Err(e) = assign_pattern(&mut circuit_state, &pat, result) {
            e.report();
        }
    }

    let output_span = circuit_ast.output.span();
    let output_value = convert_expr(global_state, &mut circuit_state, circuit_ast.output)?;
    circuit_state.circuit.set_num_outputs(output_value.type_().size());
    assert_eq!(circuit_state.circuit.num_outputs(), output_value.type_().size(), "number of circuit outputs should be equal to the number of output producers");

    let output_bundle = bundle::make_receiver_bundle(&output_value.type_(), &mut circuit_state.circuit.output_indexes().map(|output_idx| output_idx.into()));
    bundle::connect_bundle(&mut circuit_state.circuit, output_span, &output_value, &output_bundle);

    circuit_state.circuit.calculate_locations();

    Some((name, circuit_state.circuit, ty::Type::pat_type(&circuit_ast.input), output_value.type_()))
}

fn assign_pattern<'cgs, 'file>(circuit_state: &'cgs mut CircuitGenState<'file>, pat: &ast::Pattern<'file>, bundle: ProducerBundle) -> Result<(), Error<'file>> {
    if bundle.type_() != ty::Type::pat_type(pat) {
        Err(Error::TypeMismatchInAssignment { pat_sp: pat.span(), actual_type: bundle.type_(), pattern_type: ty::Type::pat_type(pat) })?
    }

    match (pat, bundle) {
        (ast::Pattern::Identifier(iden, _), bundle) => {
            circuit_state.locals.insert(iden.1, bundle);
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

fn convert_expr<'file>(global_state: &GlobalGenState, circuit_state: &mut CircuitGenState, expr: ast::Expr<'file>) -> Option<ProducerBundle> {
    let span = expr.span();
    match expr {
        ast::Expr::Ref(name_sp, name) => {
            let name_resolved = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    Error::NoSuchLocal(name_sp, name).report();
                    None?
                }
            };

            Some(name_resolved.clone())
        }

        ast::Expr::Call(circuit_name, inline, input) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name.1) {
                Some(n) => n,
                None => {
                    Error::NoSuchCircuit(circuit_name.0, circuit_name.1).report();
                    None?
                }
            };

            let input = convert_expr(global_state, circuit_state, *input)?;
            if inline {
                name_resolved.inline_gate(circuit_state, span, input)
            } else {
                name_resolved.add_gate(circuit_state, span, input)
            }
        }

        ast::Expr::Const(_, val) => CircuitDef::Const(val).add_gate(circuit_state, expr.span(), ProducerBundle::Product(Vec::new())),

        ast::Expr::Get(expr, field_name) => {
            let expr = convert_expr(global_state, circuit_state, *expr)?;
            let field = match &expr {
                ProducerBundle::Single(_) => None,
                ProducerBundle::Product(items) => match field_name.1.parse::<usize>() {
                    Ok(i) if i < items.len() => Some(items[i].clone()),
                    _ => None,
                },
            };
            if let Some(r) = field {
                Some(r)
            } else {
                Error::NoField { ty: expr.type_(), field_name: field_name.1, field_name_sp: field_name.0 }.report();
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
