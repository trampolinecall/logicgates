use std::collections::HashMap;

use super::arena;
use super::error::File;
use super::error::Report;
use super::error::Span;
use super::ir;
use super::ir::circuit1::TypedPattern;
use super::ir::circuit2;
use super::ir::circuit2::bundle::ProducerBundle;
use super::ir::circuit2::Circuit;
use super::ir::named_type;
use super::ir::ty;
use super::make_name_tables;
use super::type_exprs;
use circuit2::bundle::ReceiverBundle;

// TODO: replace all String with &'file str?

mod error;

impl arena::IsArenaIdFor<circuit2::Gate> for super::make_name_tables::CircuitOrIntrinsicId {}
struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
    circuit: circuit2::Circuit,
}
impl CircuitGenState<'_> {
    fn new(name: String, input_type: ty::TypeSym, output_type: ty::TypeSym) -> Self {
        Self { locals: HashMap::default(), circuit: (Circuit::new(name, input_type, output_type)) }
    }
}

pub(crate) fn convert(
    file: &File,
    type_exprs::IR { circuits, circuit_table, mut type_context, type_table }: type_exprs::IR,
) -> Option<(ty::TypeContext<named_type::FullyDefinedNamedType>, circuit2::Circuit)> {
    let circuits = circuits.transform(|circuit| {
        Some(match circuit {
            ir::circuit1::CircuitOrIntrinsic::Circuit(circuit) => circuit2::Gate::Custom(convert_circuit(&circuit_table, &mut type_context, circuit)?),
            ir::circuit1::CircuitOrIntrinsic::Nand => circuit2::Gate::Nand,
        })
    })?;

    match circuit_table.get("main") {
        Some(main_id /*  */) => match circuits.get(*main_id) {
            circuit2::Gate::Custom(c) => Some((type_context, c.clone())), // TODO: move out of arena instead of cloning
            _ => unreachable!("builtin circuit called main"),
        },
        None => {
            (&type_context, error::Error::NoMain(file)).report();
            None?
        }
    }
}

fn convert_circuit<'ggs, 'types, 'file>(
    circuit_table: &'ggs HashMap<String, make_name_tables::CircuitOrIntrinsicId>,
    type_context: &'types mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit1: ir::circuit1::TypedCircuit<'file>,
) -> Option<circuit2::Circuit> {
    let mut circuit_state = CircuitGenState::new(circuit1.name.1.to_string(), circuit1.input.type_info, circuit1.output_type.1);

    if let Err(e) = assign_pattern(type_context, &mut circuit_state, &circuit1.input, circuit2::bundle::ProducerBundle::CurCircuitInput) {
        (&*type_context, e).report();
    }

    // TODO: allowing recursive lets
    for ir::circuit1::Let { pat, val } in &circuit1.lets {
        let result = convert_expr(circuit_table, type_context, &mut circuit_state, &circuit1, *val)?;
        if let Err(e) = assign_pattern(type_context, &mut circuit_state, &pat, result) {
            (&*type_context, e).report();
        }
    }

    let output_value_span = circuit1.expressions.get(circuit1.output).kind.span(&circuit1.expressions);
    let output_value = convert_expr(circuit_table, type_context, &mut circuit_state, &circuit1, circuit1.output)?;

    connect_bundle(type_context, &mut circuit_state, output_value_span, output_value, circuit2::bundle::ReceiverBundle::CurCircuitOutput);

    Some(circuit_state.circuit)
}

fn assign_pattern<'types, 'cgs, 'file>(
    type_context: &'types mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_state: &'cgs mut CircuitGenState<'file>,
    pat: &TypedPattern<'file>,
    bundle: ProducerBundle,
) -> Result<(), error::Error<'file>> {
    if bundle.type_(type_context, &circuit_state.circuit) != pat.type_info {
        Err(error::Error::TypeMismatch { expected_span: pat.kind.span(), got_type: bundle.type_(type_context, &circuit_state.circuit), expected_type: pat.type_info })?;
    }

    match &pat.kind {
        ir::circuit1::PatternKind::Identifier(_, iden, _) => {
            circuit_state.locals.insert(iden, bundle);
        }
        ir::circuit1::PatternKind::Product(_, subpats) => {
            for (subpat_i, subpat) in subpats.iter().enumerate() {
                // when named product expressions are implemented, this should not be enumerate
                assign_pattern(type_context, circuit_state, subpat, ProducerBundle::Get(Box::new(bundle.clone()), subpat_i.to_string()))?;
            }
        }
    }

    Ok(())
}

fn convert_expr<'file, 'types>(
    circuit_table: &HashMap<String, make_name_tables::CircuitOrIntrinsicId>,
    type_context: &'types mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_state: &mut CircuitGenState,
    circuit1: &ir::circuit1::TypedCircuit,
    expr: ir::circuit1::expr::ExprId,
) -> Option<ProducerBundle> {
    let span = circuit1.expressions.get(expr).kind.span(&circuit1.expressions);
    match &circuit1.expressions.get(expr).kind {
        ir::circuit1::expr::ExprKind::Ref(name_sp, name) => {
            let name_resolved = if let Some(resolved) = circuit_state.locals.get(name) {
                resolved
            } else {
                (&*type_context, error::Error::NoSuchLocal(*name_sp, name)).report();
                None?
            };

            Some(name_resolved.clone())
        }

        ir::circuit1::expr::ExprKind::Call(circuit_name, inline, arg) => {
            let name_resolved = if let Some(n) = circuit_table.get(circuit_name.1) {
                n
            } else {
                (&*type_context, error::Error::NoSuchCircuit(circuit_name.0, circuit_name.1)).report();
                None?
            };

            let arg = convert_expr(circuit_table, type_context, circuit_state, circuit1, *arg)?;
            let gate_i = circuit_state.circuit.add_gate(todo!() /* name_resolved.clone() */);
            // TODO: implement inlining
            connect_bundle(type_context, circuit_state, span, arg, circuit2::bundle::ReceiverBundle::GateInput(gate_i))?;
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::expr::ExprKind::Const(_, value) => {
            let gate_i = circuit_state.circuit.add_gate(if *value { circuit2::CONST_1 } else { circuit2::CONST_0 }.clone());
            Some(circuit2::bundle::ProducerBundle::GateOutput(gate_i))
        }

        ir::circuit1::expr::ExprKind::Get(expr, (field_name_sp, field_name)) => {
            let expr = convert_expr(circuit_table, type_context, circuit_state, circuit1, *expr)?;
            let expr_type = expr.type_(type_context, &circuit_state.circuit);
            if type_context.get(expr_type).field_type(type_context, field_name).is_some() {
                // TODO: make .fields.contains() instead of has_field
                Some(ProducerBundle::Get(Box::new(expr), field_name.to_string()))
            } else {
                (&*type_context, error::Error::NoField { ty: expr_type, field_name, field_name_sp: *field_name_sp }).report();
                None
            }
        }

        ir::circuit1::expr::ExprKind::Multiple { exprs, .. } => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.into_iter().enumerate() {
                if let Some(expr) = convert_expr(circuit_table, type_context, circuit_state, circuit1, *expr) {
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
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_state: &mut CircuitGenState,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: ProducerBundle,
    receiver_bundle: ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(type_context, &circuit_state.circuit);
    let receiver_type = receiver_bundle.type_(type_context, &circuit_state.circuit);
    if producer_type != receiver_type {
        (&*type_context, error::Error::TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    circuit_state.circuit.add_connection(producer_bundle, receiver_bundle);

    Some(())
}
