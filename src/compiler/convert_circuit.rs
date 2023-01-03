use std::collections::HashMap;

use crate::compiler::ir::circuit2::Circuit;

use super::error::File;
use super::error::Report;
use super::error::Span;
use super::ir;
use super::ir::circuit1::TypedPattern;
use super::ir::circuit2;
use super::ir::circuit2::bundle::ProducerBundle;
use super::ir::circuit2::CustomCircuit;
use super::ir::ty;
use circuit2::bundle::ReceiverBundle;

// TODO: replace all String with &'file str?

mod error;

struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, Circuit>,
    const_0: Circuit,
    const_1: Circuit,
}

impl<'file> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("nand", Circuit::Nand);

        let const_0 = Circuit::Const(false);
        let const_1 = Circuit::Const(true);
        Self { circuit_table, const_0, const_1 }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
    circuit: circuit2::CustomCircuit,
}
impl CircuitGenState<'_> {
    fn new(name: String, input_type: ty::TypeSym, result_type: ty::TypeSym) -> Self {
        Self { locals: HashMap::default(), circuit: (CustomCircuit { gates: Vec::new(), connections: Vec::new(), input_type, result_type, name }) }
    }
}

pub(crate) fn convert(file: &File, types: &mut ty::Types, ast: Vec<ir::circuit1::TypedCircuit>) -> Option<circuit2::Circuit> {
    let mut global_state = GlobalGenState::new();

    let mut errored = false;

    for circuit in ast {
        let ((name_sp, name), circuit) = convert_circuit(&global_state, types, circuit)?;
        if global_state.circuit_table.contains_key(name) {
            (&*types, error::Error::Duplicate(name_sp, name)).report();
            errored = true;
        } else {
            global_state.circuit_table.insert(name, circuit2::Circuit::CustomCircuit(circuit));
        }
    }

    if errored {
        None?;
    }
    match global_state.circuit_table.remove("main") {
        Some(r) => Some(r),
        None => {
            (&*types, error::Error::NoMain(file)).report();
            None?
        }
    }
}

fn convert_circuit<'ggs, 'types, 'file>(
    global_state: &'ggs GlobalGenState<'file>,
    types: &'types mut ty::Types,
    circuit_ast: ir::circuit1::TypedCircuit<'file>,
) -> Option<((Span<'file>, &'file str), circuit2::CustomCircuit)> {
    let mut circuit_state = CircuitGenState::new(circuit_ast.name.1.to_string(), circuit_ast.input.type_info, circuit_ast.output_type);

    if let Err(e) = assign_pattern(types, &mut circuit_state, &circuit_ast.input, circuit2::bundle::ProducerBundle::CurCircuitInput) {
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

    connect_bundle(types, &mut circuit_state, output_value_span, output_value, circuit2::bundle::ReceiverBundle::CurCircuitOutput);

    // circuit_state.circuit.calculate_locations(); TODO

    Some((circuit_ast.name, circuit_state.circuit))
}

fn assign_pattern<'types, 'cgs, 'file>(
    types: &'types mut ty::Types,
    circuit_state: &'cgs mut CircuitGenState<'file>,
    pat: &TypedPattern<'file>,
    bundle: ProducerBundle,
) -> Result<(), error::Error<'file>> {
    if bundle.type_(types, &circuit_state.circuit) != pat.type_info {
        Err(error::Error::TypeMismatch { expected_span: pat.kind.span(), got_type: bundle.type_(types, &circuit_state.circuit), expected_type: pat.type_info })?;
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
                (&*types, error::Error::NoSuchLocal(name_sp, name)).report();
                None?
            };

            Some(name_resolved.clone()) // TODO: probably put these into an arena so that clone isnt needed and they will be cloned when the circuit2::Circuit is converted into a circuit::Circuit
        }

        ir::circuit1::Expr::Call(circuit_name, inline, arg) => {
            let name_resolved = if let Some(n) = global_state.circuit_table.get(circuit_name.1) {
                n
            } else {
                (&*types, error::Error::NoSuchCircuit(circuit_name.0, circuit_name.1)).report();
                None?
            };

            let arg = convert_expr(global_state, types, circuit_state, *arg)?;
            let gate_i = circuit_state.circuit.add_gate(name_resolved.clone()); // TODO: also put this into an arena so clone isnt needed
                                                                                // TODO: implement inlining
            connect_bundle(types, circuit_state, span, arg, circuit2::bundle::ReceiverBundle::GateInput(gate_i))?;
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::Expr::Const(_, value) => {
            let gate_i = circuit_state.circuit.add_gate(if value { &global_state.const_1 } else { &global_state.const_0 }.clone());
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::Expr::Get(expr, (field_name_sp, field_name)) => {
            /*
            fn get_field(expr: &ProducerBundle, field_name: &str) -> Option<ProducerBundle> {
                match expr {
                    ProducerBundle::Single(_) => None,
                    ProducerBundle::Product(items) => items.iter().find(|(name, _)| name == field_name).map(|(_, bundle)| bundle).cloned(),
                    ProducerBundle::InstanceOfNamed(_, sub) => get_field(sub, field_name),
                }
            }
            */

            let expr = convert_expr(global_state, types, circuit_state, *expr)?;
            let expr_type = expr.type_(types, &circuit_state.circuit);
            if types.get(expr_type).field_type(types, field_name).is_some() {
                // TODO: make .fields.contains() instead of has_field
                Some(ProducerBundle::Get(Box::new(expr), field_name.into()))
            } else {
                (&*types, error::Error::NoField { ty: expr_type, field_name, field_name_sp }).report();
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
    circuit_state: &mut CircuitGenState,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: ProducerBundle,
    receiver_bundle: ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(types, &circuit_state.circuit);
    let receiver_type = receiver_bundle.type_(types, &circuit_state.circuit);
    if producer_type != receiver_type {
        (&*types, error::Error::TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    circuit_state.circuit.add_connection(producer_bundle, receiver_bundle);
    /*
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
    */

    Some(())
}
