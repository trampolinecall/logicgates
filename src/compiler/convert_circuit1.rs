use std::collections::HashMap;

use super::arena;
use super::error::Report;
use super::error::Span;
use super::ir::circuit1;
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

impl arena::IsArenaIdFor<circuit2::CircuitOrIntrinsic> for super::make_name_tables::CircuitOrIntrinsicId {}
struct CircuitGenState<'file> {
    locals: HashMap<&'file str, ProducerBundle>,
    circuit: circuit2::Circuit,
}
impl CircuitGenState<'_> {
    fn new(name: String, input_type: ty::TypeSym, output_type: ty::TypeSym) -> Self {
        Self { locals: HashMap::default(), circuit: (Circuit::new(name, input_type, output_type)) }
    }
}

pub(crate) struct IR {
    pub(crate) circuits: arena::Arena<circuit2::CircuitOrIntrinsic, make_name_tables::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<named_type::FullyDefinedNamedType>,
}

pub(crate) fn convert(type_exprs::IR { mut circuits, circuit_table, mut type_context }: type_exprs::IR) -> Option<IR> {
    let const_0 = circuits.add(circuit1::CircuitOrIntrinsic::Const(false));
    let const_1 = circuits.add(circuit1::CircuitOrIntrinsic::Const(true));

    let circuits = circuits.transform(|circuit| {
        Some(match circuit {
            circuit1::CircuitOrIntrinsic::Circuit(circuit) => circuit2::CircuitOrIntrinsic::Custom(convert_circuit((const_0, const_1), &circuit_table, &mut type_context, circuit)?),
            circuit1::CircuitOrIntrinsic::Nand => circuit2::CircuitOrIntrinsic::Nand,
            circuit1::CircuitOrIntrinsic::Const(value) => circuit2::CircuitOrIntrinsic::Const(value),
        })
    })?;

    Some(IR { circuits, circuit_table, type_context })
}

fn convert_circuit(
    consts: (make_name_tables::CircuitOrIntrinsicId, make_name_tables::CircuitOrIntrinsicId),
    circuit_table: &HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit1: circuit1::TypedCircuit,
) -> Option<circuit2::Circuit> {
    let mut circuit_state = CircuitGenState::new(circuit1.name.1.to_string(), circuit1.input.type_info, circuit1.output_type.1);

    if let Err(e) = assign_pattern(type_context, &mut circuit_state, &circuit1.input, circuit2::bundle::ProducerBundle::CurCircuitInput(circuit1.input.type_info)) {
        (&*type_context, e).report();
    }

    let mut errored = false;
    // TODO: allowing recursive lets
    for circuit1::Let { pat, val } in &circuit1.lets {
        let result = convert_expr(consts, circuit_table, type_context, &mut circuit_state, &circuit1, *val)?;
        if let Err(e) = assign_pattern(type_context, &mut circuit_state, pat, result) {
            (&*type_context, e).report();
            errored = true;
        }
    }

    let output_value_span = circuit1.expressions.get(circuit1.output).kind.span(&circuit1.expressions);
    let output_value = convert_expr(consts, circuit_table, type_context, &mut circuit_state, &circuit1, circuit1.output)?;

    connect_bundle(type_context, &mut circuit_state, output_value_span, output_value, circuit2::bundle::ReceiverBundle::CurCircuitOutput(circuit1.output_type.1));

    if errored {
        None
    } else {
        Some(circuit_state.circuit)
    }
}

fn assign_pattern<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_state: &mut CircuitGenState<'file>,
    pat: &TypedPattern<'file>,
    bundle: ProducerBundle,
) -> Result<(), error::Error<'file>> {
    if bundle.type_(type_context) != pat.type_info {
        Err(error::Error::TypeMismatch { expected_span: pat.kind.span(), got_type: bundle.type_(type_context), expected_type: pat.type_info })?;
    }

    match &pat.kind {
        circuit1::PatternKind::Identifier(_, iden, _) => {
            circuit_state.locals.insert(iden, bundle);
        }
        circuit1::PatternKind::Product(_, subpats) => {
            for (subpat_i, subpat) in subpats.iter().enumerate() {
                // when named product expressions are implemented, this should not be enumerate
                assign_pattern(type_context, circuit_state, subpat, ProducerBundle::Get(Box::new(bundle.clone()), subpat_i.to_string()))?;
            }
        }
    }

    Ok(())
}

fn convert_expr(
    consts @ (const_0, const_1): (make_name_tables::CircuitOrIntrinsicId, make_name_tables::CircuitOrIntrinsicId),
    circuit_table: &HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit_state: &mut CircuitGenState,
    circuit1: &circuit1::TypedCircuit,
    expr: circuit1::expr::ExprId,
) -> Option<ProducerBundle> {
    let span = circuit1.expressions.get(expr).kind.span(&circuit1.expressions);
    match &circuit1.expressions.get(expr).kind {
        circuit1::expr::ExprKind::Ref(name_sp, name) => {
            let name_resolved = if let Some(resolved) = circuit_state.locals.get(name) { resolved } else { unreachable!("reference to nonexistent local after checking in previous phase") };

            Some(name_resolved.clone())
        }

        circuit1::expr::ExprKind::Call(circuit_name, _, arg) => {
            // TODO: implement inlining
            let (input_type, output_type, name_resolved) =
                if let Some(n) = circuit_table.get(circuit_name.1) { n } else { unreachable!("call to nonexistent circuit after checking in previous phase") };

            let arg = convert_expr(consts, circuit_table, type_context, circuit_state, circuit1, *arg)?;
            let gate_i = circuit_state.circuit.add_gate(*name_resolved);
            // TODO: move all typechecking into a separate phase
            connect_bundle(type_context, circuit_state, span, arg, circuit2::bundle::ReceiverBundle::GateInput(*input_type, gate_i))?;
            Some(circuit2::bundle::ProducerBundle::GateOutput(*output_type, gate_i))
        }

        circuit1::expr::ExprKind::Const(_, value) => {
            let gate_i = circuit_state.circuit.add_gate(if *value { const_1 } else { const_0 });
            Some(circuit2::bundle::ProducerBundle::GateOutput(type_context.intern(ty::Type::Bit), gate_i))
        }

        circuit1::expr::ExprKind::Get(expr, (_, field_name)) => {
            let expr = convert_expr(consts, circuit_table, type_context, circuit_state, circuit1, *expr)?;
            let expr_type = expr.type_(type_context);
            if type_context.get(expr_type).field_type(type_context, field_name).is_some() {
                Some(ProducerBundle::Get(Box::new(expr), field_name.to_string()))
            } else {
                unreachable!("get invalid field after checking in previous phase")
            }
        }

        circuit1::expr::ExprKind::Multiple { exprs, .. } => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.iter().enumerate() {
                if let Some(expr) = convert_expr(consts, circuit_table, type_context, circuit_state, circuit1, *expr) {
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
    let producer_type = producer_bundle.type_(type_context);
    let receiver_type = receiver_bundle.type_(type_context);
    if producer_type != receiver_type {
        (&*type_context, error::Error::TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    circuit_state.circuit.add_connection(producer_bundle, receiver_bundle);

    Some(())
}
