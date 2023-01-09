use std::collections::HashMap;

use crate::{
    compiler::{
        data::{circuit1, circuit2, nominal_type, ty},
        error::{CompileError, Report, Span},
        phases::type_exprs,
    },
    utils::arena,
};

struct TypeMismatch<'file> {
    // got_span: Span<'file>, TODO
    pub(super) expected_span: Span<'file>,
    pub(super) got_type: ty::TypeSym,
    pub(super) expected_type: ty::TypeSym,
}

struct LoopInLocalsError<'file>(Vec<Value<'file>>);

impl<'file> From<(&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, TypeMismatch<'file>)> for CompileError<'file> {
    fn from((types, TypeMismatch { expected_span, got_type, expected_type }): (&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, TypeMismatch<'file>)) -> Self {
        let expected_type = types.get(expected_type).fmt(types);
        let got_type = types.get(got_type).fmt(types);
        CompileError::new_with_note(expected_span, format!("type mismatch: expected {}, got {}", expected_type, got_type), format!("this has type {}", expected_type))
        // .note(got_span, format!("this has type {}", got_type)) TODO
    }
}

impl<'file> From<LoopInLocalsError<'file>> for CompileError<'file> {
    fn from(LoopInLocalsError(loop_): LoopInLocalsError<'file>) -> Self {
        let (first, more) = loop_.split_first().expect("loop cannot be empty");

        let mut error = CompileError::new_with_note(first.span, "infinite loop in evaluation of locals".into(), "evaluating this expression...".into());

        for e in more {
            error = error.note_and(e.span, "requires evaluating this one:".to_string(), "which...".to_string());
        }

        error = error.note(first.span, "leads to the first expression".to_string());

        error
    }
}

pub(crate) struct IR<'file> {
    pub(crate) circuits: arena::Arena<circuit2::CircuitOrIntrinsic<'file>, circuit1::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, circuit1::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn convert(type_exprs::IR { mut circuits, circuit_table, mut type_context }: type_exprs::IR) -> Option<IR> {
    let const_0 = circuits.add(circuit1::TypedCircuitOrIntrinsic::Const(false));
    let const_1 = circuits.add(circuit1::TypedCircuitOrIntrinsic::Const(true));

    let circuits = circuits.transform(|circuit| {
        Some(match circuit {
            circuit1::TypedCircuitOrIntrinsic::Circuit(circuit) => circuit2::CircuitOrIntrinsic::Custom(convert_circuit((const_0, const_1), &circuit_table, &mut type_context, circuit)?),
            circuit1::TypedCircuitOrIntrinsic::Nand => circuit2::CircuitOrIntrinsic::Nand,
            circuit1::TypedCircuitOrIntrinsic::Const(value) => circuit2::CircuitOrIntrinsic::Const(value),
        })
    })?;

    Some(IR { circuits, circuit_table, type_context })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ValueId(usize);
impl arena::ArenaId for ValueId {
    fn make(i: usize) -> Self {
        Self(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}
impl<'file> arena::IsArenaIdFor<Value<'file>> for ValueId {}
impl<'file> arena::IsArenaIdFor<(Value<'file>, circuit2::bundle::ProducerBundle)> for ValueId {}
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

fn convert_circuit<'file>(
    (const_0, const_1): (circuit1::CircuitOrIntrinsicId, circuit1::CircuitOrIntrinsicId),
    circuit_table: &HashMap<&'file str, (ty::TypeSym, ty::TypeSym, circuit1::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    circuit1: circuit1::TypedCircuit<'file>,
) -> Option<circuit2::Circuit<'file>> {
    // TODO: move all typechecking into a separate phase

    let mut circuit = circuit2::Circuit::new(circuit1.name.1, circuit1.input.type_info, circuit1.output_type.1);

    let mut values = arena::Arena::new();

    let circuit_input_value = values.add(Value { kind: ValueKind::Input, type_info: circuit1.input.type_info, span: circuit1.input.span });
    let lets: Vec<_> = circuit1.lets.into_iter().map(|circuit1::TypedLet { pat, val }| circuit1::Let { pat, val: convert_expr_to_value(&mut values, val) }).collect();
    let circuit_output_value = convert_expr_to_value(&mut values, circuit1.output);

    // steps for resolving locals

    // step 1: add all gates
    let mut gates = HashMap::new(); // only calls and consts are included in this map
    for (value_id, value) in values.iter_with_ids() {
        match value.kind {
            ValueKind::Call((_, name), _, _) => {
                gates.insert(value_id, circuit.gates.add(circuit_table[name].2));
            }
            ValueKind::Const(_, value) => {
                gates.insert(value_id, circuit.gates.add(if value { const_1 } else { const_0 }));
            }

            _ => {}
        }
    }

    // step 2: assign all patterns to values
    let mut locals = HashMap::new();
    let mut errored = false;
    if let Err(e) = assign_pattern(type_context, &mut values, &mut locals, &circuit1.input, circuit_input_value) {
        (&*type_context, e).report();
        errored = true;
    }

    for circuit1::Let { pat, val } in lets {
        if let Err(e) = assign_pattern(type_context, &mut values, &mut locals, &pat, val) {
            (&*type_context, e).report();
            errored = true;
        }
    }

    // step 3: convert all values to producer bundles
    let values = match values.transform_dependant_with_id(
        |value_id, value, get_other_value_as_bundle| convert_value(type_context, get_other_value_as_bundle, &locals, &gates, &circuit, value_id, value),
        |original_value, producer_bundle| (original_value, producer_bundle),
    ) {
        Ok(r) => r,
        Err((loop_errors, _)) => {
            // never makes other errors
            loop_errors.into_iter().for_each(|loop_| LoopInLocalsError(loop_).report());
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
    let output_value_span = values.get(circuit_output_value).0.span;
    let output_value = values.get(circuit_output_value);
    connect_bundle(type_context, &mut circuit, output_value_span, output_value.1.clone(), circuit2::bundle::ReceiverBundle::CurCircuitOutput(circuit1.output_type.1));

    if errored {
        None
    } else {
        Some(circuit)
    }
}

fn assign_pattern<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    values: &mut arena::Arena<Value<'file>, ValueId>,
    locals: &mut HashMap<&'file str, ValueId>,
    pat: &circuit1::TypedPattern<'file>,
    value: ValueId,
) -> Result<(), TypeMismatch<'file>> {
    // TODO: if this has a type error, any names in the pattern will go unassigned, meaning that getting that local later will panic
    if values.get(value).type_info != pat.type_info {
        Err(TypeMismatch { expected_span: pat.span, got_type: values.get(value).type_info, expected_type: pat.type_info })?;
    }

    match &pat.kind {
        circuit1::TypedPatternKind::Identifier(_, iden, _) => {
            locals.insert(iden, value);
        }
        circuit1::TypedPatternKind::Product(_, subpats) => {
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

fn convert_expr_to_value<'file>(values: &mut arena::Arena<Value<'file>, ValueId>, expr: circuit1::TypedExpr<'file>) -> ValueId {
    let value = Value {
        kind: match expr.kind {
            circuit1::TypedExprKind::Ref(sp, name) => ValueKind::Ref(sp, name),
            circuit1::TypedExprKind::Call(name, inline, arg) => ValueKind::Call(name, inline, convert_expr_to_value(values, *arg)),
            circuit1::TypedExprKind::Const(sp, value) => ValueKind::Const(sp, value),
            circuit1::TypedExprKind::Get(base, field) => ValueKind::Get(convert_expr_to_value(values, *base), field),
            circuit1::TypedExprKind::Multiple(exprs) => ValueKind::Multiple { values: exprs.into_iter().map(|e| convert_expr_to_value(values, e)).collect() },
        },
        span: expr.span,
        type_info: expr.type_info,
    };
    values.add(value)
}

enum NeverErrors {}
fn convert_value(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    get_other_value_as_bundle: arena::DependancyGetter<circuit2::bundle::ProducerBundle, Value, NeverErrors, ValueId>,
    locals: &HashMap<&str, ValueId>,
    gates: &HashMap<ValueId, circuit2::GateIdx>,
    circuit: &circuit2::Circuit,
    value_id: ValueId,
    value: &Value,
) -> arena::SingleTransformResult<circuit2::bundle::ProducerBundle, ValueId, NeverErrors> {
    let mut do_get = |expr, field_name| -> arena::SingleTransformResult<circuit2::bundle::ProducerBundle, ValueId, NeverErrors> {
        let expr = try_transform_result!(get_other_value_as_bundle.get(expr)).1;
        let expr_type = expr.type_(type_context);
        assert!(type_context.get(expr_type).field_type(type_context, field_name).is_some(), "get field that does not exist after already checking that all gets are valid in previous phase");
        arena::SingleTransformResult::Ok(circuit2::bundle::ProducerBundle::Get(Box::new(expr.clone()), field_name.to_string()))
    };

    match &value.kind {
        ValueKind::Ref(_, name) => arena::SingleTransformResult::Ok((try_transform_result!(get_other_value_as_bundle.get(locals[name]))).1.clone()),

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
            let mut results = Vec::new();

            for (ind, subvalue) in subvalues.iter().enumerate() {
                match get_other_value_as_bundle.get(*subvalue) {
                    arena::SingleTransformResult::Ok(result) => results.push((ind.to_string(), result.1.clone())),
                    arena::SingleTransformResult::Dep(d_error) => return arena::SingleTransformResult::Dep(d_error),
                    arena::SingleTransformResult::Err(never) => match never {},
                }
            }

            arena::SingleTransformResult::Ok(circuit2::bundle::ProducerBundle::Product(results))
        }
        ValueKind::Input => arena::SingleTransformResult::Ok(circuit2::bundle::ProducerBundle::CurCircuitInput(circuit.input_type)),
    }
}

fn connect_bundle(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    circuit: &mut circuit2::Circuit,
    // got_span: Span,
    expected_span: Span,
    producer_bundle: circuit2::bundle::ProducerBundle,
    receiver_bundle: circuit2::bundle::ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(type_context);
    let receiver_type = receiver_bundle.type_(type_context);
    if producer_type != receiver_type {
        (&*type_context, TypeMismatch { got_type: producer_type, expected_type: receiver_type, /* got_span, */ expected_span }).report();
        None?;
    }

    circuit.connections.push((producer_bundle, receiver_bundle));

    Some(())
}
