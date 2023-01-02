mod bundle;
mod circuit_def;

use std::collections::HashMap;

use crate::circuit;

use bundle::ProducerBundle;
use circuit_def::CircuitDef;

use self::bundle::ReceiverBundle;

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
    TypeMismatch { producer_span: Span<'file>, arrow_span: Span<'file>, receiver_span: Span<'file>, producer_type: ty::TypeSym, receiver_type: ty::TypeSym },
    NoMain(&'file File),
    NotAReceiver(Span<'file>),
    // NotAProducer(Span<'file>),
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

        let const_0 = CircuitDef::Const { value: false, input_type: types.intern(ty::Type::Product(vec![])), result_type: types.intern(ty::Type::Bit) };
        let const_1 = CircuitDef::Const { value: true, input_type: types.intern(ty::Type::Product(vec![])), result_type: types.intern(ty::Type::Bit) };
        Self { circuit_table, const_0, const_1 }
    }
}

struct CircuitGenState<'file> {
    locals: HashMap<&'file str, (ReceiverBundle, ProducerBundle)>,
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
            Error::NoMain(f) => CompileError::new(f.eof_span(), "no '`main' circuit".into()),
            Error::TypeMismatch { producer_span: _producer_span, arrow_span, receiver_span: _receiver_span, producer_type, receiver_type } => CompileError::new(
                // TODO: show on the producer and receiver spans which has which type
                arrow_span,
                format!("type mismatch: connecting {} to {}", types.get(producer_type).fmt(types), types.get(receiver_type).fmt(types)),
            ),
            Error::NotAReceiver(sp) => CompileError::new(sp, "not a receiver".into()),
            // Error::NotAProducer(sp) => CompileError::new(sp, "not a producer".into()),
        }
    }
}

pub(crate) fn generate(file: &File, types: &mut ty::Types, ast: Vec<ir::TypedCircuit>) -> Option<circuit::Circuit> {
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
        None?
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
    circuit_ast: ir::TypedCircuit<'file>,
) -> Option<((Span<'file>, &'file str), circuit::Circuit, ty::TypeSym, ty::TypeSym)> {
    let mut circuit_state = CircuitGenState::new(circuit_ast.name.1.to_string());

    {
        let input_type_resolved = types.get(circuit_ast.input_type);
        let output_type_resolved = types.get(circuit_ast.output_type);

        circuit_state.circuit.set_num_inputs(input_type_resolved.size(types));
        circuit_state.circuit.set_num_outputs(output_type_resolved.size(types));

        // sanity checks
        assert_eq!(input_type_resolved.size(types), circuit_state.circuit.num_inputs(), "number of circuit inputs should be equal to the number of input bits");
        assert_eq!(output_type_resolved.size(types), circuit_state.circuit.num_outputs(), "number of circuit outputs should be equal to the number of output bits");
    }

    // TODO: this should probably be 'inputs' and 'outputs' but 'self' fits more nicely into the local table
    circuit_state.locals.insert(
        "self",
        (
            bundle::make_receiver_bundle(types, circuit_ast.output_type, &mut (circuit_state.circuit.output_indexes().map(|output_idx| output_idx.into()))),
            bundle::make_producer_bundle(types, circuit_ast.input_type, &mut (circuit_state.circuit.input_indexes().map(|input_idx| input_idx.into()))),
        ),
    );

    for ir::GateInstance { local_name, gate_name } in circuit_ast.gates {
        let gate_def = match global_state.circuit_table.get(&gate_name.1) {
            Some(def) => def,
            None => {
                (&*types, Error::NoSuchCircuit(gate_name.0, gate_name.1)).report();
                None?
            }
        };
        let gate_added = gate_def.add_gate(types, &mut circuit_state)?;
        circuit_state.locals.insert(local_name.1, gate_added);
    }

    for ir::Connection { arrow_span, producer, receiver } in circuit_ast.connections {
        let producer_span = producer.span();
        let receiver_span = receiver.span();
        let producer = convert_producer(global_state, types, &mut circuit_state, producer)?;
        let receiver = convert_receiver(global_state, types, &mut circuit_state, receiver)?;
        bundle::connect_bundle(types, &mut circuit_state.circuit, arrow_span, producer_span, receiver_span, &producer, &receiver);
    }

    circuit_state.circuit.calculate_locations();

    Some((circuit_ast.name, circuit_state.circuit, circuit_ast.input_type, circuit_ast.output_type))
}

// TODO: there is probably a better way of doing this so that it doesnt need to be copied and pasted between the two functions
fn convert_receiver(global_state: &GlobalGenState, types: &mut ty::Types, circuit_state: &mut CircuitGenState, expr: ir::Expr) -> Option<ReceiverBundle> {
    let span = expr.span();
    match expr {
        ir::Expr::Ref(name_sp, name) => {
            let (as_receiver, _) = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    (&*types, Error::NoSuchLocal(name_sp, name)).report();
                    None?
                }
            };

            Some(as_receiver.clone())
        }

        ir::Expr::Const(_, value) => {
            if value { &global_state.const_1 } else { &global_state.const_0 }.add_gate(types, circuit_state)?;
            // const expr is not a receiver
            // even though it has a receiver of type [] (empty product type), you shouldnt be able to connect to it
            (&*types, Error::NotAReceiver(span)).report();
            None
        }

        ir::Expr::Get(expr, (field_name_sp, field_name)) => {
            let expr = convert_receiver(global_state, types, circuit_state, *expr)?;
            let field = match &expr {
                ReceiverBundle::Single(_) => None,
                ReceiverBundle::Product(items) => items.iter().find(|(name, _)| name == field_name).map(|(_, bundle)| bundle).cloned(),
            };
            if let Some(r) = field {
                Some(r)
            } else {
                let ty = expr.type_(types);
                (&*types, Error::NoField { ty, field_name, field_name_sp }).report();
                None
            }
        }

        ir::Expr::Multiple { exprs, .. } => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.into_iter().enumerate() {
                if let Some(expr) = convert_receiver(global_state, types, circuit_state, expr) {
                    if let Some(ref mut results) = results {
                        results.push((ind.to_string(), expr));
                    }
                } else {
                    results = None;
                }
            }

            Some(ReceiverBundle::Product(results?))
        }
    }
}

fn convert_producer(global_state: &GlobalGenState, types: &mut ty::Types, circuit_state: &mut CircuitGenState, expr: ir::Expr) -> Option<ProducerBundle> {
    match expr {
        ir::Expr::Ref(name_sp, name) => {
            let (_, as_producer) = match circuit_state.locals.get(name) {
                Some(resolved) => resolved,
                None => {
                    (&*types, Error::NoSuchLocal(name_sp, name)).report();
                    None?
                }
            };

            Some(as_producer.clone())
        }

        ir::Expr::Const(_, value) => {
            let (_, p) = if value { &global_state.const_1 } else { &global_state.const_0 }.add_gate(types, circuit_state)?;
            Some(p)
        }

        ir::Expr::Get(expr, (field_name_sp, field_name)) => {
            let expr = convert_producer(global_state, types, circuit_state, *expr)?;
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

        ir::Expr::Multiple { exprs, .. } => {
            let mut results = Some(Vec::new());

            for (ind, expr) in exprs.into_iter().enumerate() {
                if let Some(expr) = convert_producer(global_state, types, circuit_state, expr) {
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
