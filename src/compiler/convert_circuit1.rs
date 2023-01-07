use std::collections::HashMap;

use super::arena;
use super::error::CompileError;
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

struct TypeMismatch<'file> {
    /* got_span: Span<'file>, TODO */ pub(super) expected_span: Span<'file>,
    pub(super) got_type: ty::TypeSym,
    pub(super) expected_type: ty::TypeSym,
}

impl<'file> From<(&ty::TypeContext<named_type::FullyDefinedNamedType>, TypeMismatch<'file>)> for CompileError<'file> {
    fn from((types, TypeMismatch { expected_span, got_type, expected_type }): (&ty::TypeContext<named_type::FullyDefinedNamedType>, TypeMismatch<'file>)) -> Self {
        // TODO: show on the producer and receiver spans which has which type
        CompileError::new(expected_span, format!("type mismatch: expected {}, got {}", types.get(expected_type).fmt(types), types.get(got_type).fmt(types)))
    }
}

impl arena::IsArenaIdFor<circuit2::CircuitOrIntrinsic> for super::make_name_tables::CircuitOrIntrinsicId {}

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

type ValueId = circuit1::expr::ExprId;
impl<'file> arena::IsArenaIdFor<Value<'file>> for ValueId {}
impl<'file> arena::IsArenaIdFor<(Value<'file>, ProducerBundle)> for ValueId {}
#[derive(Debug)]
struct Value<'file> {
    kind: ValueKind<'file>,
    span: Span<'file>,
    type_info: ty::TypeSym,
}
#[derive(Debug)]
enum ValueKind<'file> {
    Ref(Span<'file>, &'file str),
    Call((Span<'file>, &'file str), bool, ValueId),
    Const(Span<'file>, bool),
    Get(ValueId, (Span<'file>, &'file str)),
    MadeUpGet(ValueId, String), // used for gets in destructuring
    Multiple { values: Vec<ValueId> },
    Input,
}

fn convert_circuit(
    (const_0, const_1): (make_name_tables::CircuitOrIntrinsicId, make_name_tables::CircuitOrIntrinsicId),
    circuit_table: &HashMap<String, (ty::TypeSym, ty::TypeSym, make_name_tables::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit1: circuit1::TypedCircuit,
) -> Option<circuit2::Circuit> {
    // TODO: move all typechecking into a separate phase

    let mut circuit = Circuit::new(circuit1.name.1.to_string(), circuit1.input.type_info, circuit1.output_type.1);

    let mut values = circuit1.expressions.transform_infallible(|expr| Value {
        kind: match expr.kind {
            circuit1::expr::ExprKind::Ref(sp, name) => ValueKind::Ref(sp, name),
            circuit1::expr::ExprKind::Call(name, inline, arg) => ValueKind::Call(name, inline, arg),
            circuit1::expr::ExprKind::Const(sp, value) => ValueKind::Const(sp, value),
            circuit1::expr::ExprKind::Get(base, field) => ValueKind::Get(base, field),
            circuit1::expr::ExprKind::Multiple(exprs) => ValueKind::Multiple { values: exprs },
        },
        span: expr.span,
        type_info: expr.type_info,
    });

    let input_value = values.add(Value { kind: ValueKind::Input, type_info: circuit1.input.type_info, span: circuit1.input.span });
    let mut locals = HashMap::new();
    let mut gates = HashMap::new(); // only calls and consts are included in this map

    // steps for resolving locals

    // step 1: add all gates
    for (value_id, value) in values.iter_with_ids() {
        match value.kind {
            ValueKind::Call((_, name), _, _) => {
                gates.insert(value_id, circuit.add_gate(circuit_table[name].2));
            }
            ValueKind::Const(_, value) => {
                gates.insert(value_id, circuit.add_gate(if value { const_1 } else { const_0 }));
            }

            _ => {}
        }
    }

    let mut errored = false;
    // step 2: assign all patterns to values
    if let Err(e) = assign_pattern(type_context, &mut values, &mut locals, &circuit1.input, input_value) {
        (&*type_context, e).report();
        errored = true;
    }

    for circuit1::Let { pat, val } in &circuit1.lets {
        if let Err(e) = assign_pattern(type_context, &mut values, &mut locals, pat, *val) {
            (&*type_context, e).report();
            errored = true;
        }
    }

    // step 3: convert all values to producer bundles
    let values = match values.annotate_dependant_with_id(
        |value_id, value, get_other_value_as_bundle| convert_value(type_context, get_other_value_as_bundle, &locals, &gates, &circuit, value_id, value),
        |original_value, producer_bundle| (original_value, producer_bundle),
    ) {
        Ok(r) => r,
        Err((loop_errors, other_errors)) => {
            // other errors are all ()
            todo!("report errors: loop_errors = {}, other_errors = {other_errors:?}", loop_errors.len());
            return None;
        }
    };

    // step 4: connect all receiver bundles
    for (value_id, value) in values.iter_with_ids() {
        if let ValueKind::Call((_, name), _, arg) = value.0.kind {
            let (input_type, _, _) = circuit_table[name];
            let arg_span = values.get(arg).0.span;
            let gate_i = gates[&value_id];
            let arg = values.get(arg).1.clone();
            connect_bundle(type_context, &mut circuit, arg_span, arg, circuit2::bundle::ReceiverBundle::GateInput(input_type, gate_i))?;
        }
    }
    let output_value_span = values.get(circuit1.output).0.span;
    let output_value = values.get(circuit1.output);
    connect_bundle(type_context, &mut circuit, output_value_span, output_value.1.clone(), circuit2::bundle::ReceiverBundle::CurCircuitOutput(circuit1.output_type.1));

    if errored {
        None
    } else {
        Some(circuit)
    }
}

fn assign_pattern<'file>(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    values: &mut arena::Arena<Value<'file>, ValueId>,
    locals: &mut HashMap<&'file str, ValueId>,
    pat: &TypedPattern<'file>,
    value: ValueId,
) -> Result<(), TypeMismatch<'file>> {
    if values.get(value).type_info != pat.type_info {
        Err(TypeMismatch { expected_span: pat.span, got_type: values.get(value).type_info, expected_type: pat.type_info })?;
    }

    match &pat.kind {
        circuit1::PatternKind::Identifier(_, iden, _) => {
            locals.insert(iden, value);
        }
        circuit1::PatternKind::Product(_, subpats) => {
            for (subpat_i, subpat) in subpats.iter().enumerate() {
                // destructuring happens by setting each subpattern to a made up get
                // TODO: when named product literals are implemented, this should be the actual field name and not just the enumerate index
                let field_name = subpat_i.to_string();
                let field_type = type_context.get(pat.type_info).field_type(type_context, &field_name).expect("field name does not exist in made up get for destructuring pattern");
                let new_value = values.add(Value { kind: ValueKind::MadeUpGet(value, field_name), type_info: field_type, span: subpat.span });
                assign_pattern(type_context, values, locals, subpat, new_value)?;
            }
        }
    }

    Ok(())
}

fn convert_value(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    get_other_value_as_bundle: arena::DependancyGetter<ProducerBundle, Value, (), ValueId>,
    locals: &HashMap<&str, ValueId>,
    gates: &HashMap<ValueId, circuit2::GateIdx>,
    circuit: &Circuit,
    value_id: ValueId,
    value: &Value,
) -> arena::SingleTransformResult<ProducerBundle, ValueId, ()> {
    let mut do_get = |expr, field_name| {
        let expr = try_annotation_result!(get_other_value_as_bundle.get(expr)).1;
        let expr_type = expr.type_(type_context);
        assert!(type_context.get(expr_type).field_type(type_context, field_name).is_some(), "get field that does not exist after already checking that all gets are valid in previous phase");
        arena::SingleTransformResult::Ok(ProducerBundle::Get(Box::new(expr.clone()), field_name.to_string()))
    };

    match &value.kind {
        ValueKind::Ref(_, name) => arena::SingleTransformResult::Ok((try_annotation_result!(get_other_value_as_bundle.get(locals[name]))).1.clone()),

        ValueKind::Call(_, _, _) => {
            // TODO: implement inlining

            let gate_i = gates[&value_id];
            // the gate stays unconnected to its input because gates can be truend into a producerb undle with needing to be connected, which allows for loops
            // for example 'let x = 'not x' will be allowed because x refers to the output of the 'not gate and the input to the 'not gate doesnt need to be connected for x to have a value
            arena::SingleTransformResult::Ok(circuit2::bundle::ProducerBundle::GateOutput(value.type_info, gate_i))
        }

        ValueKind::Const(_, _) => {
            let gate_i = gates[&value_id];
            arena::SingleTransformResult::Ok(circuit2::bundle::ProducerBundle::GateOutput(type_context.intern(ty::Type::Bit), gate_i))
        }

        ValueKind::Get(expr, (_, field_name)) => do_get(*expr, field_name),
        ValueKind::MadeUpGet(expr, field_name) => do_get(*expr, field_name),

        ValueKind::Multiple { values: subvalues, .. } => {
            let mut results = Some(Vec::new());

            for (ind, subvalue) in subvalues.iter().enumerate() {
                match get_other_value_as_bundle.get(*subvalue) {
                    arena::SingleTransformResult::Ok(result) => {
                        if let Some(ref mut results) = results {
                            results.push((ind.to_string(), result.1.clone()));
                        }
                    }
                    arena::SingleTransformResult::Dep(d_error) => return arena::SingleTransformResult::Dep(d_error),
                    arena::SingleTransformResult::Err(()) => results = None,
                }
            }

            if let Some(results) = results {
                arena::SingleTransformResult::Ok(ProducerBundle::Product(results))
            } else {
                arena::SingleTransformResult::Err(())
            }
        }
        ValueKind::Input => arena::SingleTransformResult::Ok(ProducerBundle::CurCircuitInput(circuit.input_type)),
    }
}

fn connect_bundle(
    type_context: &mut ty::TypeContext<named_type::FullyDefinedNamedType>,
    circuit: &mut Circuit,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: ProducerBundle,
    receiver_bundle: ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(type_context);
    let receiver_type = receiver_bundle.type_(type_context);
    if producer_type != receiver_type {
        (&*type_context, TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    circuit.add_connection(producer_bundle, receiver_bundle);

    Some(())
}
