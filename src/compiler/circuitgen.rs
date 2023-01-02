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
use super::ir;
use super::ty;

// TODO: calculate types first to prevent repeated calls to intern

enum Error<'file> {
    Duplicate(Span<'file>, &'file str),
    NoField { ty: ty::TypeSym, field_name_sp: Span<'file>, field_name: &'file str }, // TODO: list names of fields that do exist
    NoSuchLocal(Span<'file>, &'file str),
    NoSuchCircuit(Span<'file>, &'file str),
    TypeMismatchInAssignment { pat_sp: Span<'file>, actual_type: ty::TypeSym, pattern_type: ty::TypeSym },
    TypeMismatchInCall { expr_span: Span<'file>, actual_type: ty::TypeSym, expected_type: ty::TypeSym },
    NoMain(&'file File),
}

#[derive(Default)]
struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, CircuitDef>,
}

impl<'file> GlobalGenState<'file> {
    fn new(types: &mut ty::Types) -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert(
            "and",
            CircuitDef::And {
                input_type: {
                    let b = types.intern(ty::Type::Bit);
                    types.intern(ty::Type::Product(vec![("0".into(), b), ("1".into(), b)]))
                },
                result_type: types.intern(ty::Type::Bit),
            },
        );
        circuit_table.insert("not", CircuitDef::Not { input_type: types.intern(ty::Type::Bit), result_type: types.intern(ty::Type::Bit) });
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

impl<'file> From<(&ty::Types, Error<'file>)> for CompileError<'file> {
    fn from((types, val): (&ty::Types, Error<'file>)) -> Self {
        match val {
            Error::Duplicate(name_sp, name) => CompileError::new(name_sp, format!("circuit '{}' defined more than once", name)),
            Error::NoField { ty, field_name_sp, field_name } => CompileError::new(field_name_sp, format!("no field called '{}' on type '{}'", field_name, types.get(ty).fmt(types))),
            Error::NoSuchLocal(name_sp, name) => CompileError::new(name_sp, format!("no local called '{}'", name)),
            Error::NoSuchCircuit(name_sp, name) => CompileError::new(name_sp, format!("no circuit called '`{}'", name)),
            Error::TypeMismatchInAssignment { pat_sp, actual_type: value_type, pattern_type } => {
                CompileError::new(pat_sp, format!("type mismatch in assignment: value has type {} but pattern has type {}", types.get(value_type).fmt(types), types.get(pattern_type).fmt(types)))
            }
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no '`main' circuit".into()),
            Error::TypeMismatchInCall { actual_type, expected_type, expr_span } => CompileError::new(
                expr_span,
                format!("type mismatch in subcircuit: arguments have type {} but subcircuit expects type {}", types.get(actual_type).fmt(types), types.get(expected_type).fmt(types)),
            ),
        }
    }
}

pub(crate) fn generate(file: &File, ast: Vec<ir::Circuit<ir::Pattern<ty::TypeSym>, ir::Expr<ty::TypeSym>>>) -> Option<circuit::Circuit> {
    let mut types = ty::Types::new();
    let mut global_state = GlobalGenState::new(&mut types);

    let mut errored = false;

    for circuit in ast {
        let ((name_sp, name), circuit, input_type, result_type) = convert_circuit(&global_state, &mut types, circuit)?;
        if global_state.circuit_table.contains_key(name) {
            (&types, Error::Duplicate(name_sp, name)).report();
            errored = true;
        } else {
            global_state.circuit_table.insert(name, CircuitDef::Circuit { circuit, input_type, result_type });
        }
    }

    if errored {
        None?
    }
    match global_state.circuit_table.remove("main") {
        Some(CircuitDef::Circuit { circuit: r, .. }) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            (&types, Error::NoMain(file)).report();
            None?
        }
    }
}

fn convert_circuit<'ggs, 'types, 'file>(
    global_state: &'ggs GlobalGenState<'file>,
    types: &'types mut ty::Types,
    circuit_ast: ir::Circuit<'file, ir::Pattern<ty::TypeSym>, ir::Expr<ty::TypeSym>>,
) -> Option<((Span<'file>, &'file str), circuit::Circuit, ty::TypeSym, ty::TypeSym)> {
    let name = circuit_ast.name;

    let mut circuit_state = CircuitGenState::new(name.1.to_string());

    let (input_type_sym, input_type) = {
        let sym = circuit_ast.input.type_info;
        (sym, types.get(sym))
    };
    circuit_state.circuit.set_num_inputs(input_type.size(types));
    assert_eq!(input_type.size(types), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits"); // sanity check

    let input_bundle = bundle::make_producer_bundle(types, input_type_sym, &mut circuit_state.circuit.input_indexes().map(|circuit_input_idx| circuit_input_idx.into()));
    if let Err(e) = assign_pattern(types, &mut circuit_state, &circuit_ast.input, input_bundle) {
        (&*types, e).report();
    }

    for ir::Let { pat, val } in circuit_ast.lets {
        let result = convert_expr(global_state, types, &mut circuit_state, val)?;
        if let Err(e) = assign_pattern(types, &mut circuit_state, &pat, result) {
            (&*types, e).report();
        }
    }

    let output_span = circuit_ast.output.kind.span();
    let output_value = convert_expr(global_state, types, &mut circuit_state, circuit_ast.output)?;
    let (output_type_sym, output_type) = {
        let sym = output_value.type_(types);
        (sym, types.get(sym))
    };
    circuit_state.circuit.set_num_outputs(output_type.size(types));
    assert_eq!(circuit_state.circuit.num_outputs(), output_type.size(types), "number of circuit outputs should be equal to the number of output producers");

    let output_bundle = bundle::make_receiver_bundle(types, output_type_sym, &mut circuit_state.circuit.output_indexes().map(|output_idx| output_idx.into()));
    bundle::connect_bundle(types, &mut circuit_state.circuit, output_span, &output_value, &output_bundle);

    circuit_state.circuit.calculate_locations();

    Some((name, circuit_state.circuit, input_type_sym, output_value.type_(types)))
}

fn assign_pattern<'types, 'cgs, 'file>(types: &'types mut ty::Types, circuit_state: &'cgs mut CircuitGenState<'file>, pat: &ir::Pattern<'file, ty::TypeSym>, bundle: ProducerBundle) -> Result<(), Error<'file>> {
    if bundle.type_(types) != pat.type_info {
        Err(Error::TypeMismatchInAssignment { pat_sp: pat.kind.span(), actual_type: bundle.type_(types), pattern_type: pat.type_info })?
    }

    match (&pat.kind, bundle) {
        (ir::PatternKind::Identifier(_, iden, _), bundle) => {
            circuit_state.locals.insert(iden, bundle);
        }
        (ir::PatternKind::Product(_, subpats), ProducerBundle::Product(subbundles)) => {
            assert_eq!(subpats.len(), subbundles.len(), "assign product pattern to procut bundle with different length"); // sanity check
            for (subpat, (_, subbundle)) in subpats.iter().zip(subbundles) {
                assign_pattern(types, circuit_state, subpat, subbundle)?;
            }
        }

        (pat, bundle) => unreachable!("assign pattern to bundle with different type after type checking: pattern = {pat:?}, bundle = {bundle:?}"),
    }

    Ok(())
}

fn convert_expr<'file, 'types>(global_state: &GlobalGenState<'file>, types: &'types mut ty::Types, circuit_state: &mut CircuitGenState, expr: ir::Expr<'file, ty::TypeSym>) -> Option<ProducerBundle> {
    let span = expr.kind.span();
    match expr.kind {
        ir::ExprKind::Ref(name_sp, name) => {
            let name_resolved = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    (&*types, Error::NoSuchLocal(name_sp, name)).report();
                    None?
                }
            };

            Some(name_resolved.clone())
        }

        ir::ExprKind::Call(circuit_name, inline, arg) => {
            let name_resolved = match global_state.circuit_table.get(circuit_name.1) {
                Some(n) => n,
                None => {
                    (&*types, Error::NoSuchCircuit(circuit_name.0, circuit_name.1)).report();
                    None?
                }
            };

            let arg = convert_expr(global_state, types, circuit_state, *arg)?;
            let (receiver, producer) = if inline { name_resolved.inline_gate(types, circuit_state) } else { name_resolved.add_gate(types, circuit_state) }?;
            bundle::connect_bundle(types, &mut circuit_state.circuit, span, &arg, &receiver)?;
            Some(producer)
        }

        ir::ExprKind::Const(_, value) => {
            let (_, p) = CircuitDef::Const { value, input_type: todo!(), result_type: todo!() }.add_gate(types, circuit_state)?;
            Some(p)
        }

        ir::ExprKind::Get(expr, (field_name_sp, field_name)) => {
            let expr = convert_expr(global_state, types, circuit_state, *expr)?;
            let field = match &expr {
                ProducerBundle::Single(_) => None,
                ProducerBundle::Product(items) => items.iter().find(|(name, _)| name == field_name).map(|(_, bundle)| bundle).cloned(),
            };
            if let Some(r) = field {
                Some(r)
            } else {
                let ty = expr.type_(types);
                (&*types, Error::NoField { ty, field_name, field_name_sp }).report();
                None
            }
        }

        ir::ExprKind::Multiple(_, exprs) => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.into_iter().enumerate() {
                if let Some(expr) = convert_expr(global_state, types, circuit_state, expr) {
                    if let Some(ref mut results) = results {
                        results.push((ind.to_string(), expr));
                    }
                } else {
                    results = None;
                }
            }

            Some(ProducerBundle::Product(results?))
        }
    }
}
