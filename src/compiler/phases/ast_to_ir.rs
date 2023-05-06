use std::collections::HashMap;

use crate::{
    compiler::{
        data::{ast, ir, nominal_type, token, ty},
        error::{CompileError, Report, Span},
        phases::type_exprs,
    },
    utils::arena,
};

struct TypeMismatch<'file> {
    got_span: Span<'file>,
    pub(super) expected_span: Span<'file>,
    pub(super) got_type: ty::TypeSym,
    pub(super) expected_type: ty::TypeSym,
}

struct LoopInLocalsError<'file>(Vec<ExprInArena<'file>>);

impl<'file> From<(&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, TypeMismatch<'file>)> for CompileError<'file> {
    fn from((types, TypeMismatch { expected_span, got_type, expected_type, got_span }): (&ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>, TypeMismatch<'file>)) -> Self {
        let expected_type = types.get(expected_type).fmt(types);
        let got_type = types.get(got_type).fmt(types);
        CompileError::new_with_note(expected_span, format!("type mismatch: expected {}, got {}", expected_type, got_type), format!("expected {}", expected_type))
            .note(got_span, format!("got {}", got_type))
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
    pub(crate) circuits: arena::Arena<ir::CircuitOrIntrinsic<'file>, ast::CircuitOrIntrinsicId>,
    pub(crate) circuit_table: HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,

    pub(crate) type_context: ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
}

pub(crate) fn convert(type_exprs::IR { mut circuits, circuit_table, mut type_context }: type_exprs::IR) -> Option<IR> {
    let const_0 = circuits.add(ast::TypedCircuitOrIntrinsic::Const(false));
    let const_1 = circuits.add(ast::TypedCircuitOrIntrinsic::Const(true));

    let circuits = circuits.transform(|circuit| {
        Some(match circuit {
            ast::TypedCircuitOrIntrinsic::Circuit(circuit) => ir::CircuitOrIntrinsic::Custom(convert_circuit((const_0, const_1), &circuit_table, &mut type_context, circuit)?),
            ast::TypedCircuitOrIntrinsic::Nand => ir::CircuitOrIntrinsic::Nand,
            ast::TypedCircuitOrIntrinsic::Const(value) => ir::CircuitOrIntrinsic::Const(value),
        })
    })?;

    Some(IR { circuits, circuit_table, type_context })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct ExprId(usize);
impl arena::ArenaId for ExprId {
    fn make(i: usize) -> Self {
        Self(i)
    }

    fn get(&self) -> usize {
        self.0
    }
}
#[derive(Debug)]
struct ExprInArena<'file> {
    kind: ExprInArenaKind<'file>,
    span: Span<'file>,
    type_info: ty::TypeSym,
}
#[derive(Debug)]
enum ExprInArenaKind<'file> {
    Ref(token::PlainIdentifier<'file>),
    Call(token::CircuitIdentifier<'file>, bool, ExprId),
    Const(Span<'file>, bool),
    Get(ExprId, (Span<'file>, &'file str)),
    MadeUpGet(ExprId, String), // used for gets in destructuring
    Product { values: Vec<(String, ExprId)> },
    Input,
    Poison,
}

fn convert_circuit<'file>(
    (const_0, const_1): (ast::CircuitOrIntrinsicId, ast::CircuitOrIntrinsicId),
    circuit_table: &HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    circuit_ast: ast::TypedCircuit<'file>,
) -> Option<ir::Circuit<'file>> {
    let mut circuit = ir::Circuit::new(circuit_ast.name.name, circuit_ast.input.type_info, circuit_ast.output_type.1);

    let mut values = arena::Arena::new();

    let circuit_input_value = values.add(ExprInArena { kind: ExprInArenaKind::Input, type_info: circuit_ast.input.type_info, span: circuit_ast.input.span });
    let lets: Vec<_> = circuit_ast.lets.into_iter().map(|ast::TypedLet { pat, val }| ast::Let { pat, val: convert_expr_to_value(&mut values, val) }).collect();
    let circuit_output_value = convert_expr_to_value(&mut values, circuit_ast.output);

    // steps for resolving locals

    // step 1: add all gates
    let mut gates = HashMap::new(); // only calls and consts are included in this map
    for (value_id, value) in values.iter_with_ids() {
        match &value.kind {
            ExprInArenaKind::Call(name, inline, _) => {
                gates.insert(value_id, circuit.gates.add((circuit_table[name.name].2, if *inline { ir::Inline::Inline } else { ir::Inline::NoInline })));
            }
            ExprInArenaKind::Const(_, value) => {
                gates.insert(value_id, circuit.gates.add((if *value { const_1 } else { const_0 }, ir::Inline::NoInline)));
            }

            _ => {}
        }
    }

    // step 2: assign all patterns to values
    let mut locals = HashMap::new();
    let mut errored = false;
    if let Err(()) = assign_pattern(type_context, &mut values, &mut locals, &circuit_ast.input, circuit_input_value) {
        errored = true;
    }

    for ast::Let { pat, val } in lets {
        if let Err(()) = assign_pattern(type_context, &mut values, &mut locals, &pat, val) {
            errored = true;
        }
    }

    // step 3: convert all values to producer bundles
    let values = match values.transform_dependent_with_id(
        |value_id, value, get_other_value_as_bundle| convert_value(type_context, get_other_value_as_bundle, &locals, &gates, &circuit, value_id, value),
        |original_value, producer_bundle| (original_value, producer_bundle),
    ) {
        Ok(r) => r,
        Err((loop_errors, _)) => {
            // never makes other errors
            for loop_ in loop_errors {
                LoopInLocalsError(loop_).report()
            }
            return None;
        }
    };

    // step 4: connect all receiver bundles
    for (value_id, value) in values.iter_with_ids() {
        if let ExprInArenaKind::Call(name, _, arg) = &value.0.kind {
            let (input_type, _, _) = circuit_table[name.name];
            let arg_span = values.get(*arg).0.span;
            let gate_i = gates[&value_id];
            let arg = values.get(*arg).1.clone();
            // TODO: the second arg_span argument should really be the receiver span, but arg_span is the producer span
            connect_bundle(type_context, &mut circuit, arg_span, arg_span, arg, ir::bundle::ReceiverBundle::GateInput(input_type, gate_i))?;
        }
    }
    let output_value = values.get(circuit_output_value);
    connect_bundle(type_context, &mut circuit, output_value.0.span, circuit_ast.output_type.0, output_value.1.clone(), ir::bundle::ReceiverBundle::CurCircuitOutput(circuit_ast.output_type.1));

    if errored {
        None
    } else {
        Some(circuit)
    }
}

fn assign_pattern<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    values: &mut arena::Arena<ExprInArena<'file>, ExprId>,
    locals: &mut HashMap<&'file str, ExprId>,
    pat: &ast::TypedPattern<'file>,
    value: ExprId,
) -> Result<(), ()> {
    if values.get(value).type_info != pat.type_info {
        (&*type_context, TypeMismatch { expected_span: pat.span, got_type: values.get(value).type_info, expected_type: pat.type_info, got_span: values.get(value).span }).report();
        assign_pattern_poison(values, locals, pat, values.get(value).span);
        return Err(());
    }

    match &pat.kind {
        ast::TypedPatternKind::Identifier(name, _) => {
            locals.insert(name.name, value);
        }
        ast::TypedPatternKind::Product(subpats) => {
            for (field_name, subpat) in subpats.iter() {
                // destructuring happens by setting each subpattern to a made up get
                let field_name = field_name.to_string();
                let field_type =
                    ty::Type::get_field_type(&type_context.get(pat.type_info).fields(type_context), &field_name).expect("field name does not exist in made up get for destructuring pattern");
                let new_value = values.add(ExprInArena { kind: ExprInArenaKind::MadeUpGet(value, field_name), type_info: field_type, span: subpat.span });
                assign_pattern(type_context, values, locals, subpat, new_value)?;
            }
        }
    }

    Ok(())
}

fn assign_pattern_poison<'file>(values: &mut arena::Arena<ExprInArena<'file>, ExprId>, locals: &mut HashMap<&'file str, ExprId>, pat: &ast::TypedPattern<'file>, span: Span<'file>) {
    match &pat.kind {
        ast::PatternKind::Identifier(name, _) => {
            let value = values.add(ExprInArena { kind: ExprInArenaKind::Poison, type_info: pat.type_info, span });
            locals.insert(name.name, value);
        }
        ast::PatternKind::Product(subpats) => {
            for (_, subpat) in subpats {
                assign_pattern_poison(values, locals, subpat, span);
            }
        }
    }
}

fn convert_expr_to_value<'file>(values: &mut arena::Arena<ExprInArena<'file>, ExprId>, expr: ast::TypedExpr<'file>) -> ExprId {
    let value = ExprInArena {
        kind: match expr.kind {
            ast::TypedExprKind::Ref(name) => ExprInArenaKind::Ref(name),
            ast::TypedExprKind::Call(name, inline, arg) => ExprInArenaKind::Call(name, inline, convert_expr_to_value(values, *arg)),
            ast::TypedExprKind::Const(sp, value) => ExprInArenaKind::Const(sp, value),
            ast::TypedExprKind::Get(base, field) => ExprInArenaKind::Get(convert_expr_to_value(values, *base), field),
            ast::TypedExprKind::Product(exprs) => ExprInArenaKind::Product { values: exprs.into_iter().map(|(field_name, e)| (field_name, convert_expr_to_value(values, e))).collect() },
        },
        span: expr.span,
        type_info: expr.type_info,
    };
    values.add(value)
}

fn convert_value(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    get_other_value_as_bundle: arena::DependancyGetter<ir::bundle::ProducerBundle, ExprInArena, (), ExprId>,
    locals: &HashMap<&str, ExprId>,
    gates: &HashMap<ExprId, ir::GateIdx>,
    circuit: &ir::Circuit,
    value_id: ExprId,
    value: &ExprInArena,
) -> arena::SingleTransformResult<ir::bundle::ProducerBundle, ExprId, ()> {
    let mut do_get = |expr, field_name| -> arena::SingleTransformResult<ir::bundle::ProducerBundle, ExprId, ()> {
        let expr = try_transform_result!(get_other_value_as_bundle.get(expr)).1;
        let expr_type = expr.type_(type_context);
        assert!(
            ty::Type::get_field_type(&type_context.get(expr_type).fields(type_context), field_name).is_some(),
            "get field that does not exist after already checking that all gets are valid in previous phase"
        );
        arena::SingleTransformResult::Ok(ir::bundle::ProducerBundle::Get(Box::new(expr.clone()), field_name.to_string()))
    };

    match &value.kind {
        ExprInArenaKind::Ref(name) => arena::SingleTransformResult::Ok((try_transform_result!(get_other_value_as_bundle.get(locals[name.name]))).1.clone()),

        ExprInArenaKind::Call(_, _, _) => {
            let gate_i = gates[&value_id];
            // the gate stays unconnected to its input because gates can be truend into a producerb undle with needing to be connected, which allows for loops
            // for example 'let x = 'not x' will be allowed because x refers to the output of the 'not gate and the input to the 'not gate doesnt need to be connected for x to have a value
            arena::SingleTransformResult::Ok(ir::bundle::ProducerBundle::GateOutput(value.type_info, gate_i))
        }

        ExprInArenaKind::Const(_, _) => {
            let gate_i = gates[&value_id];
            arena::SingleTransformResult::Ok(ir::bundle::ProducerBundle::GateOutput(type_context.intern(ty::Type::Bit), gate_i))
        }

        ExprInArenaKind::Get(expr, (_, field_name)) => do_get(*expr, field_name),
        ExprInArenaKind::MadeUpGet(expr, field_name) => do_get(*expr, field_name),

        ExprInArenaKind::Product { values: subvalues, .. } => {
            let mut results = Vec::new();

            let mut errored = false;
            for (field_name, subvalue) in subvalues.iter() {
                match get_other_value_as_bundle.get(*subvalue) {
                    arena::SingleTransformResult::Ok(result) => results.push((field_name.clone(), result.1.clone())),
                    arena::SingleTransformResult::Dep(d_error) => return arena::SingleTransformResult::Dep(d_error),
                    arena::SingleTransformResult::Err(()) => errored = true,
                }
            }

            if !errored {
                arena::SingleTransformResult::Ok(ir::bundle::ProducerBundle::Product(results))
            } else {
                arena::SingleTransformResult::Err(())
            }
        }
        ExprInArenaKind::Input => arena::SingleTransformResult::Ok(ir::bundle::ProducerBundle::CurCircuitInput(circuit.input_type)),
        ExprInArenaKind::Poison => arena::SingleTransformResult::Err(()),
    }
}

fn connect_bundle(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    circuit: &mut ir::Circuit,
    producer_span: Span,
    receiver_span: Span,
    producer_bundle: ir::bundle::ProducerBundle,
    receiver_bundle: ir::bundle::ReceiverBundle,
) -> Option<()> {
    let producer_type = producer_bundle.type_(type_context);
    let receiver_type = receiver_bundle.type_(type_context);
    if producer_type != receiver_type {
        (&*type_context, TypeMismatch { got_type: producer_type, expected_type: receiver_type, expected_span: receiver_span, got_span: producer_span }).report();
        None?;
    }

    circuit.connections.push((producer_bundle, receiver_bundle));

    Some(())
}
