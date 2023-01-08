use crate::simulation::{components::connection, Circuit, Gate, GateIndex};

use generational_arena::Arena;
use std::collections::HashSet;

pub(crate) struct CalculationComponent {
    index: GateIndex,
    calculation: Calculation,
}

pub(crate) enum Either<L, R> { // TODO: probably move this into utils
    Left(L),
    Right(R),
}
use Either::*;

enum Calculation {
    Nand { inputs: connection::ReceiversComponent, outputs: connection::ProducersComponent },
    Const { value: bool, inputs: connection::ReceiversComponent, outputs: connection::ProducersComponent },
    Custom(Box<Circuit>),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct InputNodeIdx(GateIndex, usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct OutputNodeIdx(GateIndex, usize);

impl InputNodeIdx {
    pub fn gate_index(self) -> GateIndex {
        self.0
    }
}
impl OutputNodeIdx {
    pub fn gate_index(self) -> GateIndex {
        self.0
    }
}
impl CalculationComponent {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(index: GateIndex) -> Self {
        Self {
            index,
            calculation: Calculation::Nand {
                outputs: connection::ProducersComponent(vec![connection::Producer::new(true)]),
                inputs: connection::ReceiversComponent(vec![connection::Receiver::new(), connection::Receiver::new()]),
            },
        }
    }
    pub(crate) fn new_const(index: GateIndex, value: bool) -> Self {
        Self { index, calculation: Calculation::Const { value, outputs: connection::ProducersComponent(vec![connection::Producer::new(value)]), inputs: connection::ReceiversComponent(vec![]) } }
    }
    pub(crate) fn new_custom(index: GateIndex, subcircuit: Circuit) -> Self {
        Self { index, calculation: Calculation::Custom(Box::new(subcircuit)) }
    }

    pub(crate) fn inputs(&self) -> Either<&connection::ReceiversComponent, &connection::ProducersComponent> {
        match &self.calculation {
            Calculation::Nand { inputs, outputs } => Left(&inputs),
            Calculation::Const { value, inputs, outputs } => Left(&inputs),
            Calculation::Custom(subc) => Right(&subc.inputs),
        }
    }
    pub(crate) fn outputs(&self) -> Either<&connection::ProducersComponent, &connection::ReceiversComponent> {
        match &self.calculation {
            Calculation::Nand { inputs, outputs } => Left(&outputs),
            Calculation::Const { value, inputs, outputs } => Left(&outputs),
            Calculation::Custom(subc) => Right(&subc.outputs),
        }
    }

    pub(crate) fn name(&self) -> &str {
        match &self.calculation {
            Calculation::Nand { outputs: _, inputs: _ } => "nand",
            Calculation::Const { value: true, outputs: _, inputs: _ } => "true",
            Calculation::Const { value: false, outputs: _, inputs: _ } => "false",
            Calculation::Custom(subcircuit) => &subcircuit.name,
        }
    }

    pub(crate) fn as_custom(&self) -> Option<&Box<Circuit>> {
        // TODO: figure out better solution to this
        if let Calculation::Custom(v) = &self.calculation {
            Some(v)
        } else {
            None
        }
    }
}

pub(crate) fn update(gates: &mut Arena<Gate>) {
    // TODO: rename to propogate_value_changes or something like that?
    // TODO: save updates from last tick

    let mut update_stack: Vec<_> = gates.iter().map(|(i, _)| i).collect();
    let mut changed = HashSet::new();

    while let Some(gate_i) = update_stack.pop() {
        if changed.contains(&gate_i) {
            continue;
        }

        let gate = if let Some(gate) = gates.get(gate_i) {
            gate
        } else {
            continue;
        };

        let get_receiver_value = |receiver: &connection::Receiver| {
            if let Some(producer_idx) = receiver.producer() {
                if let Some(gate_with_producer) = gates.get(producer_idx.gate_index()) {
                    // gate_with_producer.calculation.calculation.
                    todo!()
                } else {
                    false
                }
            } else {
                false
            }
        };
        let output_values = match &gate.calculation.calculation {
            Calculation::Nand { outputs: _, inputs } => vec![!(get_receiver_value(&inputs.0[0]) & get_receiver_value(&inputs.0[1]))],
            Calculation::Const { value, outputs: _, inputs: _ } => vec![*value],
            Calculation::Custom(_) => continue, // custom gates dont need to update their receivers (TODO) because their outputs just pass through the values of their subgates
        };
        let output_nodes = match &mut gates.get_mut(gate_i).unwrap().calculation.calculation {
            Calculation::Nand { outputs, inputs: _ } => outputs,
            Calculation::Const { value: _, outputs, inputs: _ } => outputs,
            Calculation::Custom(_) => continue,
        };
        assert_eq!(output_values.len(), output_nodes.0.len());

        let mut gate_changed = false;

        for (new_value, producer) in output_values.into_iter().zip(output_nodes.0.iter_mut()) {
            let old_value = producer.value;
            if old_value != new_value {
                gate_changed = true;
            }

            producer.value = new_value;

            for dependant in producer.dependants() {
                update_stack.push(dependant.gate_index()); // the ReceiverIdx stores the GateIdx that the receiver is attached to
            }
        }

        if gate_changed {
            changed.insert(gate_i);
        }
    }
}
