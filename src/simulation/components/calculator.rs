use crate::simulation::{components::connection, Circuit, Gate, GateIndex};

use generational_arena::Arena;
use std::collections::HashSet;

pub(crate) struct CalculationComponent {
    index: GateIndex,
    pub(crate) calculation: Calculation,
}

pub(crate) enum Calculation {
    Nand { inputs: [connection::Receiver; 2], outputs: [connection::Producer; 1] },
    Const { value: bool, inputs: [connection::Receiver; 0], outputs: [connection::Producer; 1] },
    Custom(Box<Circuit>),
}

impl CalculationComponent {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(index: GateIndex) -> Self {
        Self { index, calculation: Calculation::Nand { inputs: ([(connection::Receiver::new()), (connection::Receiver::new())]), outputs: ([(connection::Producer::new(true))]) } }
    }
    pub(crate) fn new_const(index: GateIndex, value: bool) -> Self {
        Self { index, calculation: Calculation::Const { value, outputs: ([(connection::Producer::new(value))]), inputs: [] } }
    }
    pub(crate) fn new_custom(index: GateIndex, subcircuit: Circuit) -> Self {
        Self { index, calculation: Calculation::Custom(Box::new(subcircuit)) }
    }

    pub(crate) fn inputs(&self) -> impl ExactSizeIterator<Item = connection::NodeIdx> + '_ {
        (0..self.num_inputs()).map(|i| connection::NodeIdx::new(self.index, false, i))
    }
    pub(crate) fn outputs(&self) -> impl ExactSizeIterator<Item = connection::NodeIdx> + '_ {
        (0..self.num_outputs()).map(|i| connection::NodeIdx::new(self.index, true, i))
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

    pub(crate) fn num_outputs(&self) -> usize {
        match &self.calculation {
            Calculation::Nand { outputs, .. } => outputs.len(),
            Calculation::Const { outputs, .. } => outputs.len(),
            Calculation::Custom(subc) => subc.outputs.len(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        match &self.calculation {
            Calculation::Nand { inputs, .. } => inputs.len(),
            Calculation::Const { inputs, .. } => inputs.len(),
            Calculation::Custom(subc) => subc.inputs.len(),
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

        fn get_node_value(gates: &Arena<Gate>, node: &connection::Node) -> bool {
            match node {
                connection::Node::Producer(producer) => producer.value,
                connection::Node::Receiver(receiver) => {
                    if let Some(producer) = receiver.producer() {
                        if let Some(producer) = connection::get_node(gates, producer) {
                            get_node_value(gates, &producer)
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            }
        }
        let new_value = match &gate.calculation.calculation {
            Calculation::Nand { outputs: _, inputs: [a, b] } => !(get_node_value(gates, &connection::Node::Receiver(a)) & get_node_value(gates, &connection::Node::Receiver(b))),
            Calculation::Const { value, outputs: _, inputs: _ } => *value,
            Calculation::Custom(_) => continue, // custom gates dont need to update their receivers (TODO) because their outputs just pass through the values of their subgates
        };
        let output_node = match &mut gates.get_mut(gate_i).unwrap().calculation.calculation {
            Calculation::Nand { outputs: [o], inputs: _ } => o,
            Calculation::Const { value: _, outputs: [o], inputs: _ } => o,
            Calculation::Custom(_) => continue,
        };

        let old_value = output_node.value;
        let gate_changed = old_value != new_value;

        output_node.value = new_value;

        for dependant in output_node.dependants() {
            update_stack.push(dependant.gate_index());
        }

        if gate_changed {
            changed.insert(gate_i);
        }
    }
}
