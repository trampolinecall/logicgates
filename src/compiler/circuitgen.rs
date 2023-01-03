use std::collections::HashMap;

use crate::circuit;

use crate::compiler::ir::bundle::ProducerBundle;
use crate::compiler::ir::circuit2::CircuitDef;

use super::error::CompileError;
use super::error::File;
use super::error::Report;
use super::error::Span;
use super::ir;
use super::ir::bundle::ReceiverBundle;
use super::ir::circuit1::TypedPattern;
use super::ir::ty;

// TODO: calculate types first to prevent repeated calls to intern

enum Error<'file> {
    Duplicate(Span<'file>, &'file str),
    NoField { ty: ty::TypeSym, field_name_sp: Span<'file>, field_name: &'file str }, // TODO: list names of fields that do exist
    NoSuchLocal(Span<'file>, &'file str),
    NoSuchCircuit(Span<'file>, &'file str),
    TypeMismatch { /* got_span: Span<'file>, TODO */ expected_span: Span<'file>, got_type: ty::TypeSym, expected_type: ty::TypeSym },
    NoMain(&'file File),
}

struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, CircuitDef>,
    const_0: CircuitDef,
    const_1: CircuitDef,
}

impl<'file> GlobalGenState<'file> {
    fn new(types: &mut ty::Types) -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert(
            "nand",
            CircuitDef::Nand {
                input_type: {
                    let b = types.intern(ty::Type::Bit);
                    types.intern(ty::Type::Product(vec![("0".into(), b), ("1".into(), b)]))
                },
                result_type: types.intern(ty::Type::Bit),
            },
        );

        let const_0 = CircuitDef::Const { value: false, input_type: types.intern(ty::Type::Product(vec![])), result_type: types.intern(ty::Type::Bit) };
        let const_1 = CircuitDef::Const { value: true, input_type: types.intern(ty::Type::Product(vec![])), result_type: types.intern(ty::Type::Bit) };
        Self { circuit_table, const_0, const_1 }
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
            Error::NoSuchCircuit(name_sp, name) => CompileError::new(name_sp, format!("no circuit called '{}'", name)),
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no 'main' circuit".into()),
            Error::TypeMismatch { expected_span, got_type, expected_type } => CompileError::new(
                // TODO: show on the producer and receiver spans which has which type
                expected_span,
                format!("type mismatch: expected {}, got {}", types.get(expected_type).fmt(types), types.get(got_type).fmt(types)),
            ),
        }
    }
}

pub(crate) fn generate(file: &File, types: &mut ty::Types, ast: Vec<ir::circuit1::TypedCircuit>) -> Option<circuit::Circuit> {
    let mut global_state = GlobalGenState::new(types);

    let mut errored = false;

    for circuit in ast {
        let ((name_sp, name), circuit, input_type, result_type) = convert_circuit(&global_state, types, circuit)?;
        if global_state.circuit_table.contains_key(name) {
            (&*types, Error::Duplicate(name_sp, name)).report();
            errored = true;
        } else {
            global_state.circuit_table.insert(name, CircuitDef::Circuit { circuit, input_type, result_type });
        }
    }

    if errored {
        None?;
    }
    match global_state.circuit_table.remove("main") {
        Some(CircuitDef::Circuit { circuit: r, .. }) => Some(r),
        Some(_) => unreachable!("non user-defined circuit called main"),
        None => {
            (&*types, Error::NoMain(file)).report();
            None?
        }
    }
}

fn convert_circuit<'ggs, 'types, 'file>(
    global_state: &'ggs GlobalGenState<'file>,
    types: &'types mut ty::Types,
    circuit_ast: ir::circuit1::TypedCircuit<'file>,
) -> Option<((Span<'file>, &'file str), circuit::Circuit, ty::TypeSym, ty::TypeSym)> {
    let mut circuit_state = CircuitGenState::new(circuit_ast.name.1.to_string());

    let (input_type_sym, input_type) = {
        let sym = circuit_ast.input.type_info;
        (sym, types.get(sym))
    };
    circuit_state.circuit.set_num_inputs(input_type.size(types));
    assert_eq!(input_type.size(types), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits"); // sanity check

    let input_bundle = ir::bundle::make_producer_bundle(types, input_type_sym, &mut circuit_state.circuit.input_indexes().map(Into::into));
    if let Err(e) = assign_pattern(types, &mut circuit_state, &circuit_ast.input, input_bundle) {
        (&*types, e).report();
    }

    // TODO: allowing recursive lets
    for ir::circuit1::Let { pat, val } in circuit_ast.lets {
        let result = convert_expr(global_state, types, &mut circuit_state, val)?;
        if let Err(e) = assign_pattern(types, &mut circuit_state, &pat, result) {
            (&*types, e).report();
        }
    }

    let output_value_span = circuit_ast.output.span();
    let output_value = convert_expr(global_state, types, &mut circuit_state, circuit_ast.output)?;
    let (output_type_sym, output_type) = {
        let sym = output_value.type_(types);
        (sym, types.get(sym))
    };
    circuit_state.circuit.set_num_outputs(output_type.size(types));
    assert_eq!(circuit_state.circuit.num_outputs(), output_type.size(types), "number of circuit outputs should be equal to the number of output producers");

    let output_bundle = ir::bundle::make_receiver_bundle(types, output_type_sym, &mut circuit_state.circuit.output_indexes().map(Into::into));
    connect_bundle(types, &mut circuit_state.circuit, output_value_span, &output_value, &output_bundle);

    circuit_state.circuit.calculate_locations();

    Some((circuit_ast.name, circuit_state.circuit, input_type_sym, output_value.type_(types)))
}

fn assign_pattern<'types, 'cgs, 'file>(types: &'types mut ty::Types, circuit_state: &'cgs mut CircuitGenState<'file>, pat: &TypedPattern<'file>, bundle: ProducerBundle) -> Result<(), Error<'file>> {
    if bundle.type_(types) != pat.type_info {
        Err(Error::TypeMismatch { expected_span: pat.kind.span(), got_type: bundle.type_(types), expected_type: pat.type_info })?;
        // TODO
    }

    match (&pat.kind, bundle) {
        (ir::circuit1::PatternKind::Identifier(_, iden, _), bundle) => {
            circuit_state.locals.insert(iden, bundle);
        }
        (ir::circuit1::PatternKind::Product(_, subpats), ProducerBundle::Product(subbundles)) => {
            assert_eq!(subpats.len(), subbundles.len(), "assign product pattern to procut bundle with different length"); // sanity check
            for (subpat, (_, subbundle)) in subpats.iter().zip(subbundles) {
                assign_pattern(types, circuit_state, subpat, subbundle)?;
            }
        }

        (pat, bundle) => unreachable!("assign pattern to bundle with different type after type checking: pattern = {pat:?}, bundle = {bundle:?}"),
    }

    Ok(())
}

fn convert_expr<'file, 'types>(global_state: &GlobalGenState<'file>, types: &'types mut ty::Types, circuit_state: &mut CircuitGenState, expr: ir::circuit1::Expr) -> Option<ProducerBundle> {
    let span = expr.span();
    match expr {
        ir::circuit1::Expr::Ref(name_sp, name) => {
            let name_resolved = if let Some(resolved) = circuit_state.locals.get(name) {
                resolved
            } else {
                (&*types, Error::NoSuchLocal(name_sp, name)).report();
                None?
            };

            Some(name_resolved.clone())
        }

        ir::circuit1::Expr::Call(circuit_name, inline, arg) => {
            let name_resolved = if let Some(n) = global_state.circuit_table.get(circuit_name.1) {
                n
            } else {
                (&*types, Error::NoSuchCircuit(circuit_name.0, circuit_name.1)).report();
                None?
            };

            let arg = convert_expr(global_state, types, circuit_state, *arg)?;
            let (receiver, producer) = if inline { name_resolved.inline_gate(types, &mut circuit_state.circuit) } else { name_resolved.add_gate(types, &mut circuit_state.circuit) };
            connect_bundle(types, &mut circuit_state.circuit, span, &arg, &receiver)?;
            Some(producer)
        }

        ir::circuit1::Expr::Const(_, value) => {
            let (_, p) = if value { &global_state.const_1 } else { &global_state.const_0 }.add_gate(types, &mut circuit_state.circuit);
            Some(p)
        }

        ir::circuit1::Expr::Get(expr, (field_name_sp, field_name)) => {
            fn get_field(expr: &ProducerBundle, field_name: &str) -> Option<ProducerBundle> {
                match expr {
                    ProducerBundle::Single(_) => None,
                    ProducerBundle::Product(items) => items.iter().find(|(name, _)| name == field_name).map(|(_, bundle)| bundle).cloned(),
                    ProducerBundle::InstanceOfNamed(_, sub) => get_field(sub, field_name),
                }
            }

            let expr = convert_expr(global_state, types, circuit_state, *expr)?;
            let field = get_field(&expr, field_name);
            if let Some(r) = field {
                Some(r)
            } else {
                let ty = expr.type_(types);
                (&*types, Error::NoField { ty, field_name, field_name_sp }).report();
                None
            }
        }

        ir::circuit1::Expr::Multiple { exprs, .. } => {
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

fn connect_bundle(
    types: &mut ty::Types,
    circuit: &mut circuit::Circuit,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: &ProducerBundle,
    receiver_bundle: &ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(types);
    let receiver_type = receiver_bundle.type_(types);
    if producer_type != receiver_type {
        (&*types, Error::TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    match (producer_bundle, receiver_bundle) {
        (ProducerBundle::Single(producer_index), ReceiverBundle::Single(receiver_index)) => circuit.connect(*producer_index, *receiver_index),
        (ProducerBundle::Product(producers), ReceiverBundle::Product(receivers)) => {
            assert_eq!(producers.len(), receivers.len(), "cannot connect different amount of producers and receivers"); // sanity check
            for ((_, p), (_, r)) in producers.iter().zip(receivers.iter()) {
                connect_bundle(types, circuit, /* got_span, */ expected_span, p, r);
                // not ideal that this rechecks the item types but
            }
        }

        _ => unreachable!("connect two bundles with different types"),
    }

    Some(())
}
