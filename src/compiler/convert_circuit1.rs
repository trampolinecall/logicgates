use std::collections::HashMap;

use crate::compiler::ir::circuit2::Gate;
use crate::utils::CollectAll;

use super::error::File;
use super::error::Report;
use super::error::Span;
use super::ir;
use super::ir::circuit1::TypedPattern;
use super::ir::circuit2;
use super::ir::circuit2::bundle::ProducerBundle;
use super::ir::circuit2::Circuit;
use super::ir::ty;
use circuit2::bundle::ReceiverBundle;

// TODO: replace all String with &'file str?

mod error;

struct GlobalGenState<'file> {
    circuit_table: HashMap<&'file str, Gate>,
    const_0: Gate,
    const_1: Gate,
}

impl<'file> GlobalGenState<'file> {
    fn new() -> Self {
        let mut circuit_table = HashMap::new();
        circuit_table.insert("nand", Gate::Nand);

        let const_0 = Gate::Const(false);
        let const_1 = Gate::Const(true);
        Self { circuit_table, const_0, const_1 }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
    circuit: circuit2::Circuit,
}
impl CircuitGenState<'_> {
    fn new(name: String, input_type: ty::TypeSym, output_type: ty::TypeSym) -> Self {
        Self { locals: HashMap::default(), circuit: (Circuit::new(name, input_type, output_type)) }
    }
}

pub(crate) fn convert(file: &File, types: &mut ty::Types, circuits: HashMap<String, ir::circuit1::TypedCircuitOrIntrinsic>) -> Option<circuit2::Circuit> {
    // TODO: remove symbol table from global_state, replace with the actual symbol table, also prevent recursion
    let mut global_state = GlobalGenState::new();

    let mut circuits: HashMap<_, _> = circuits
        .into_iter()
        .map(|(name, circuit)| {
            Some((
                name,
                match circuit {
                    ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => circuit2::Gate::Custom(convert_circuit(&global_state, types, circuit)?),
                    ir::circuit1::CircuitOrIntrinsic::Nand => circuit2::Gate::Nand,
                },
            ))
        })
        .collect_all()?;

    match circuits.remove("main") {
        Some(Gate::Custom(c)) => Some(c),
        Some(_) => unreachable!("builtin circuit called main"),
        None => {
            (&*types, error::Error::NoMain(file)).report();
            None?
        }
    }
}

fn convert_circuit<'ggs, 'types, 'file>(global_state: &'ggs GlobalGenState<'file>, types: &'types mut ty::Types, circuit1: ir::circuit1::TypedCircuit<'file>) -> Option<circuit2::Circuit> {
    let mut circuit_state = CircuitGenState::new(circuit1.name.1.to_string(), circuit1.input.type_info, circuit1.output_type);

    if let Err(e) = assign_pattern(types, &mut circuit_state, &circuit1.input, circuit2::bundle::ProducerBundle::CurCircuitInput) {
        (&*types, e).report();
    }

    // TODO: allowing recursive lets
    for ir::circuit1::Let { pat, val } in &circuit1.lets {
        let result = convert_expr(global_state, types, &mut circuit_state, &circuit1, *val)?;
        if let Err(e) = assign_pattern(types, &mut circuit_state, &pat, result) {
            (&*types, e).report();
        }
    }

    let output_value_span = circuit1.expressions[circuit1.output].kind.span(&circuit1.expressions);
    let output_value = convert_expr(global_state, types, &mut circuit_state, &circuit1, circuit1.output)?;

    connect_bundle(types, &mut circuit_state, output_value_span, output_value, circuit2::bundle::ReceiverBundle::CurCircuitOutput);

    Some(circuit_state.circuit)
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

    match &pat.kind {
        ir::circuit1::PatternKind::Identifier(_, iden, _) => {
            circuit_state.locals.insert(iden, bundle);
        }
        ir::circuit1::PatternKind::Product(_, subpats) => {
            for (subpat_i, subpat) in subpats.iter().enumerate() {
                // when named product expressions are implemented, this should not be enumerate
                assign_pattern(types, circuit_state, subpat, ProducerBundle::Get(Box::new(bundle.clone()), subpat_i.to_string()))?;
            }
        }
    }

    Ok(())
}

fn convert_expr<'file, 'types>(
    global_state: &GlobalGenState<'file>,
    types: &'types mut ty::Types,
    circuit_state: &mut CircuitGenState,
    circuit1: &ir::circuit1::TypedCircuit,
    expr: id_arena::Id<ir::circuit1::TypedExpr>,
) -> Option<ProducerBundle> {
    let span = circuit1.expressions[expr].kind.span(&circuit1.expressions);
    match &circuit1.expressions[expr].kind {
        ir::circuit1::ExprKind::Ref(name_sp, name) => {
            let name_resolved = if let Some(resolved) = circuit_state.locals.get(name) {
                resolved
            } else {
                (&*types, error::Error::NoSuchLocal(*name_sp, name)).report();
                None?
            };

            Some(name_resolved.clone())
        }

        ir::circuit1::ExprKind::Call(circuit_name, inline, arg) => {
            let name_resolved = if let Some(n) = global_state.circuit_table.get(circuit_name.1) {
                n
            } else {
                (&*types, error::Error::NoSuchCircuit(circuit_name.0, circuit_name.1)).report();
                None?
            };

            let arg = convert_expr(global_state, types, circuit_state, circuit1, *arg)?;
            let gate_i = circuit_state.circuit.add_gate(name_resolved.clone()); // TODO: also put this into an arena so clone isnt needed
                                                                                // TODO: implement inlining
            connect_bundle(types, circuit_state, span, arg, circuit2::bundle::ReceiverBundle::GateInput(gate_i))?;
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::ExprKind::Const(_, value) => {
            let gate_i = circuit_state.circuit.add_gate(if *value { &global_state.const_1 } else { &global_state.const_0 }.clone());
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::ExprKind::Get(expr, (field_name_sp, field_name)) => {
            let expr = convert_expr(global_state, types, circuit_state, circuit1, *expr)?;
            let expr_type = expr.type_(types, &circuit_state.circuit);
            if types.get(expr_type).field_type(types, field_name).is_some() {
                // TODO: make .fields.contains() instead of has_field
                Some(ProducerBundle::Get(Box::new(expr), field_name.to_string()))
            } else {
                (&*types, error::Error::NoField { ty: expr_type, field_name, field_name_sp: *field_name_sp }).report();
                None
            }
        }

        ir::circuit1::ExprKind::Multiple { exprs, .. } => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.into_iter().enumerate() {
                if let Some(expr) = convert_expr(global_state, types, circuit_state, circuit1, *expr) {
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

    Some(())
}
