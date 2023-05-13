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
    expected_span: Span<'file>,
    got_type: ty::TypeSym,
    expected_type: ty::TypeSym,
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
    let const_0 = circuits.add(ast::CircuitOrIntrinsic::Const(false));
    let const_1 = circuits.add(ast::CircuitOrIntrinsic::Const(true));

    let circuits = circuits.transform(|circuit| {
        Some(match circuit {
            ast::CircuitOrIntrinsic::Circuit(circuit) => ir::CircuitOrIntrinsic::Custom(convert_circuit((const_0, const_1), &circuit_table, &mut type_context, circuit)?),
            ast::CircuitOrIntrinsic::Nand => ir::CircuitOrIntrinsic::Nand,
            ast::CircuitOrIntrinsic::Const(value) => ir::CircuitOrIntrinsic::Const(value),
            ast::CircuitOrIntrinsic::Unerror => ir::CircuitOrIntrinsic::Unerror,
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
// TODO: remove spans from this?
#[derive(Debug)]
enum ExprInArenaKind<'file> {
    Ref(token::PlainIdentifier<'file>),
    Const(Span<'file>, bool, ir::GateIdx),
    Get(ExprId, (Span<'file>, &'file str)),
    MadeUpGet(ExprId, String), // used for gets in destructuring
    Product { values: Vec<(String, ExprId)> },
    CircuitInput,
    CircuitOutput,
    GateInput(ty::TypeSym, ir::GateIdx),
    GateOutput(ty::TypeSym, ir::GateIdx),
    Poison,
}

fn convert_circuit<'file>(
    const_values: (ast::CircuitOrIntrinsicId, ast::CircuitOrIntrinsicId),
    circuit_table: &HashMap<&'file str, (ty::TypeSym, ty::TypeSym, ast::CircuitOrIntrinsicId)>,
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct<'file>>,
    circuit_ast: ast::Circuit<'file, ast::Typed>,
) -> Option<ir::Circuit<'file>> {
    if circuit_ast.name.name == "main" {
        if circuit_ast.input.type_info != type_context.intern(ty::Type::Product(vec![])) {
            let err = TypeMismatch {
                got_span: circuit_ast.input.span,
                expected_span: circuit_ast.input.span,
                got_type: circuit_ast.input.type_info,
                expected_type: type_context.intern(ty::Type::Product(vec![])),
            };
            (&*type_context, err).report();
            return None;
        }
        if circuit_ast.output.type_info != type_context.intern(ty::Type::Product(vec![])) {
            let err = TypeMismatch {
                got_span: circuit_ast.output.span,
                expected_span: circuit_ast.output.span,
                got_type: circuit_ast.output.type_info,
                expected_type: type_context.intern(ty::Type::Product(vec![])),
            };
            (&*type_context, err).report();
            return None;
        }
    }

    let mut circuit = ir::Circuit::new(circuit_ast.name.name, circuit_ast.input.type_info, circuit_ast.output.type_info);

    let mut values = arena::Arena::new();
    let mut locals = HashMap::new();

    {
        let circuit_input = values.add(ExprInArena { kind: ExprInArenaKind::CircuitInput, span: circuit_ast.input.span, type_info: circuit.input_type });
        let circuit_output = values.add(ExprInArena { kind: ExprInArenaKind::CircuitOutput, span: circuit_ast.output.span, type_info: circuit.output_type });
        assign_pattern(type_context, &mut values, &mut locals, &circuit_ast.input, circuit_input).ok()?;
        assign_pattern(type_context, &mut values, &mut locals, &circuit_ast.output, circuit_output).ok()?;
    }

    for let_ in circuit_ast.lets {
        let (subc_input_type, subc_output_type, subc) = circuit_table[let_.gate.name]; // TODO: report error if the circuit does not exist
        let gate_idx = circuit.gates.add((subc, ir::Inline::NoInline)); // TODO: syntax for inlining
        let input = values.add(ExprInArena { kind: ExprInArenaKind::GateInput(subc_input_type, gate_idx), span: let_.gate.span, type_info: subc_input_type });
        let output = values.add(ExprInArena { kind: ExprInArenaKind::GateOutput(subc_output_type, gate_idx), span: let_.gate.span, type_info: subc_output_type });
        assign_pattern(type_context, &mut values, &mut locals, &let_.inputs, input).ok()?;
        assign_pattern(type_context, &mut values, &mut locals, &let_.outputs, output).ok()?;
    }

    for alias in circuit_ast.aliases {
        let expr = convert_expr_to_value(const_values, &mut circuit, &mut values, alias.expr);
        assign_pattern(type_context, &mut values, &mut locals, &alias.pat, expr).ok()?;
    }

    let mut connections = Vec::new();
    for connection in circuit_ast.connects {
        let start_span = connection.start.span;
        let end_span = connection.end.span;
        let start = convert_expr_to_value(const_values, &mut circuit, &mut values, connection.start);
        let end = convert_expr_to_value(const_values, &mut circuit, &mut values, connection.end);

        connections.push((start_span, start, end_span, end));
    }

    let values = match values
        .transform_dependent(|value, get_other_value_as_bundle| convert_value(type_context, get_other_value_as_bundle, &locals, &circuit, value), |original_value, bundle| (original_value, bundle))
    {
        Ok(r) => r,
        Err((loop_errors, others)) => {
            for () in others {} // TODO: change this to Infallible or something
                                // never makes other errors
            for loop_ in loop_errors {
                LoopInLocalsError(loop_).report();
            }
            return None;
        }
    };

    for (start_sp, start, end_sp, end) in connections {
        connect_bundle(type_context, &mut circuit, start_sp, end_sp, values.get(start).1.clone(), values.get(end).1.clone())?;
    }

    Some(circuit)
}

fn assign_pattern<'file>(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    values: &mut arena::Arena<ExprInArena<'file>, ExprId>,
    locals: &mut HashMap<&'file str, ExprId>,
    pat: &ast::Pattern<'file, ast::Typed>,
    value: ExprId,
) -> Result<(), ()> {
    if values.get(value).type_info != pat.type_info {
        (&*type_context, TypeMismatch { expected_span: pat.span, got_type: values.get(value).type_info, expected_type: pat.type_info, got_span: values.get(value).span }).report();
        assign_pattern_poison(values, locals, pat, values.get(value).span);
        return Err(());
    }

    match &pat.kind {
        ast::PatternKind::Identifier(name, _) => {
            locals.insert(name.name, value);
        }
        ast::PatternKind::Product(subpats) => {
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

fn assign_pattern_poison<'file>(values: &mut arena::Arena<ExprInArena<'file>, ExprId>, locals: &mut HashMap<&'file str, ExprId>, pat: &ast::Pattern<'file, ast::Typed>, span: Span<'file>) {
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

fn convert_expr_to_value<'file>(
    const_values @ (const_0, const_1): (ast::CircuitOrIntrinsicId, ast::CircuitOrIntrinsicId),
    circuit: &mut ir::Circuit,
    values: &mut arena::Arena<ExprInArena<'file>, ExprId>,
    expr: ast::Expr<'file, ast::Typed>,
) -> ExprId {
    let value = ExprInArena {
        kind: match expr.kind {
            ast::ExprKind::Ref(name) => ExprInArenaKind::Ref(name),
            ast::ExprKind::Const(sp, value) => {
                let gate_idx = circuit.gates.add((if value { const_1 } else { const_0 }, ir::Inline::NoInline));
                ExprInArenaKind::Const(sp, value, gate_idx)
            }
            ast::ExprKind::Get(base, field) => ExprInArenaKind::Get(convert_expr_to_value(const_values, circuit, values, *base), field),
            ast::ExprKind::Product(exprs) => {
                ExprInArenaKind::Product { values: exprs.into_iter().map(|(field_name, e)| (field_name, convert_expr_to_value(const_values, circuit, values, e))).collect() }
            }
        },
        span: expr.span,
        type_info: expr.type_info,
    };
    values.add(value)
}

fn convert_value(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    get_other_value_as_bundle: arena::DependancyGetter<ir::bundle::Bundle, ExprInArena, (), ExprId>,
    locals: &HashMap<&str, ExprId>,
    circuit: &ir::Circuit,
    value: &ExprInArena,
) -> arena::SingleTransformResult<ir::bundle::Bundle, ExprId, ()> {
    let mut do_get = |expr, field_name| -> arena::SingleTransformResult<ir::bundle::Bundle, ExprId, ()> {
        let expr = try_transform_result!(get_other_value_as_bundle.get(expr)).1;
        let expr_type = expr.type_(type_context);
        assert!(
            ty::Type::get_field_type(&type_context.get(expr_type).fields(type_context), field_name).is_some(),
            "get field that does not exist after already checking that all gets are valid in previous phase"
        );
        arena::SingleTransformResult::Ok(ir::bundle::Bundle::Get(Box::new(expr.clone()), field_name.to_string()))
    };

    match &value.kind {
        ExprInArenaKind::Ref(name) => arena::SingleTransformResult::Ok((try_transform_result!(get_other_value_as_bundle.get(locals[name.name]))).1.clone()),

        ExprInArenaKind::Const(_, _, gate_idx) => arena::SingleTransformResult::Ok(ir::bundle::Bundle::GateOutput(type_context.intern(ty::Type::Bit), *gate_idx)),

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
                arena::SingleTransformResult::Ok(ir::bundle::Bundle::Product(results))
            } else {
                arena::SingleTransformResult::Err(())
            }
        }

        ExprInArenaKind::CircuitInput => arena::SingleTransformResult::Ok(ir::bundle::Bundle::CurCircuitInput(circuit.input_type)),
        ExprInArenaKind::CircuitOutput => arena::SingleTransformResult::Ok(ir::bundle::Bundle::CurCircuitOutput(circuit.output_type)),

        ExprInArenaKind::GateInput(ty, gate_idx) => arena::SingleTransformResult::Ok(ir::bundle::Bundle::GateInput(*ty, *gate_idx)),
        ExprInArenaKind::GateOutput(ty, gate_idx) => arena::SingleTransformResult::Ok(ir::bundle::Bundle::GateOutput(*ty, *gate_idx)),

        ExprInArenaKind::Poison => arena::SingleTransformResult::Err(()),
    }
}

fn connect_bundle(
    type_context: &mut ty::TypeContext<nominal_type::FullyDefinedStruct>,
    circuit: &mut ir::Circuit,
    start_span: Span,
    end_span: Span,
    start_bundle: ir::bundle::Bundle,
    end_bundle: ir::bundle::Bundle,
) -> Option<()> {
    let start_type = start_bundle.type_(type_context);
    let end_type = end_bundle.type_(type_context);
    if start_type != end_type {
        (&*type_context, TypeMismatch { got_type: start_type, expected_type: end_type, expected_span: end_span, got_span: start_span }).report(); // TODO: redo type mismatch error
        None?;
    }

    circuit.connections.push((start_bundle, end_bundle));

    Some(())
}
