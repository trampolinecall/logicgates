use std::collections::HashSet;

use super::{Circuit, CircuitIndex, CircuitMap, GateIndex, GateMap};

// TODO: reorganize this module

pub(crate) struct Calculation {
    kind: CalculationKind,
}
#[allow(clippy::large_enum_variant)] // TODO: reconsider whether this is correct
enum CalculationKind {
    Nand([Node; 2], [Node; 1]),
    Const([Node; 0], [Node; 1]),
    Custom(CircuitIndex), // the circuit already contains the input and output nodes
}

#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) gate: Option<GateIndex>,
    value: Value,
    dependants: HashSet<NodeIdx>,
}
#[derive(Clone, Copy)]
enum Value {
    Manual(bool),
    Passthrough(NodeIdx),
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateInputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ()); // unit at the end so that it cannot be constructed outside of this module
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateOutputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitInputNodeIdx(pub(crate) CircuitIndex, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOutputNodeIdx(pub(crate) CircuitIndex, pub(crate) usize, ());

impl Calculation {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(index: GateIndex) -> Calculation {
        Calculation { kind: CalculationKind::Nand([Node::new(Some(index), false), Node::new(Some(index), false)], [Node::new(Some(index), true)]) }
    }
    pub(crate) fn new_const(index: GateIndex, value: bool) -> Calculation {
        Calculation { kind: CalculationKind::Const([], [Node::new(Some(index), value)]) }
    }
    pub(crate) fn new_subcircuit(subcircuit: CircuitIndex) -> Calculation {
        Calculation { kind: CalculationKind::Custom(subcircuit) }
    }

    pub(crate) fn name<'a>(&self, circuits: &'a CircuitMap) -> &'a str {
        // TODO: hopefully somehow turn this into &str
        match self.kind {
            CalculationKind::Nand(_, _) => "nand",
            CalculationKind::Const(_, [Node { value: Value::Manual(false), .. }]) => "false",
            CalculationKind::Const(_, [Node { value: Value::Manual(true), .. }]) => "true",
            CalculationKind::Const(_, [Node { value: Value::Passthrough(_), .. }]) => unreachable!("const node with passthrough value"),
            CalculationKind::Custom(subcircuit) => &circuits[subcircuit].name,
        }
    }
}

impl Node {
    pub(crate) fn new(gate: Option<GateIndex>, value: bool) -> Self {
        Self { gate, value: Value::Manual(value), dependants: HashSet::new() }
    }

    pub(crate) fn producer(&self) -> Option<NodeIdx> {
        if let Value::Passthrough(v) = self.value {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum NodeIdx {
    CI(CircuitInputNodeIdx),
    CO(CircuitOutputNodeIdx),
    GI(GateInputNodeIdx),
    GO(GateOutputNodeIdx),
}

impl From<GateOutputNodeIdx> for NodeIdx {
    fn from(v: GateOutputNodeIdx) -> Self {
        Self::GO(v)
    }
}
impl From<CircuitInputNodeIdx> for NodeIdx {
    fn from(v: CircuitInputNodeIdx) -> Self {
        Self::CI(v)
    }
}
impl From<GateInputNodeIdx> for NodeIdx {
    fn from(v: GateInputNodeIdx) -> Self {
        Self::GI(v)
    }
}
impl From<CircuitOutputNodeIdx> for NodeIdx {
    fn from(v: CircuitOutputNodeIdx) -> Self {
        Self::CO(v)
    }
}

pub(crate) fn circuit_input_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitInputNodeIdx> + '_ {
    (0..circuit.inputs.len()).map(|i| CircuitInputNodeIdx(circuit.index, i, ()))
}
pub(crate) fn circuit_output_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitOutputNodeIdx> + '_ {
    (0..circuit.outputs.len()).map(|i| CircuitOutputNodeIdx(circuit.index, i, ()))
}

// TODO: test connection, replacing old connection, also rename to set_node_value_passthrough or something like that
pub(crate) fn connect(circuits: &mut CircuitMap, gates: &mut GateMap, producer_idx: NodeIdx, receiver_idx: NodeIdx) {
    set_node_value(circuits, gates, receiver_idx, Value::Passthrough(producer_idx))
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(circuits: &mut CircuitMap, gates: &mut GateMap, node: NodeIdx) {
    set_node_value(circuits, gates, node, Value::Manual(false))
}

pub(crate) fn get_node_value(circuits: &CircuitMap, gates: &GateMap, node: NodeIdx) -> bool {
    let node = get_node(circuits, gates, node);
    match node.value {
        Value::Manual(v) => v,
        Value::Passthrough(other) => get_node_value(circuits, gates, other),
    }
}

// TODO: test every possibility
fn set_node_value(circuits: &mut CircuitMap, gates: &mut GateMap, index: NodeIdx, new_value: Value) {
    // remove any existing connection:
    let node = get_node_mut(circuits, gates, index);
    if let Value::Passthrough(other_idx) = node.value {
        let other = get_node_mut(circuits, gates, other_idx);
        other.dependants.remove(&index);
    }
    get_node_mut(circuits, gates, index).value = Value::Manual(false);

    // set the new value:
    get_node_mut(circuits, gates, index).value = new_value;
    if let Value::Passthrough(new_dep) = new_value {
        get_node_mut(circuits, gates, new_dep).dependants.insert(index);
    }
}

pub(crate) fn toggle_input(circuits: &mut CircuitMap, gates: &mut GateMap, circuit: CircuitIndex, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    set_input(circuits, gates, CircuitInputNodeIdx(circuit, i, ()), !get_node_value(circuits, gates, CircuitInputNodeIdx(circuit, i, ()).into()));
}

pub(crate) fn set_input(circuits: &mut CircuitMap, gates: &mut GateMap, ci: CircuitInputNodeIdx, value: bool) {
    set_node_value(circuits, gates, ci.into(), Value::Manual(value));
}

pub(crate) fn get_node<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, index: NodeIdx) -> &'c Node {
    match index {
        NodeIdx::CI(ci) => &circuits[ci.0].inputs[ci.1],
        NodeIdx::CO(co) => &circuits[co.0].outputs[co.1],
        NodeIdx::GI(gi) => gate_get_input(circuits, gates, gi),
        NodeIdx::GO(go) => gate_get_output(circuits, gates, go),
    }
}
pub(crate) fn get_node_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, index: NodeIdx) -> &'c mut Node {
    match index {
        NodeIdx::CI(ci) => &mut circuits[ci.0].inputs[ci.1],
        NodeIdx::CO(co) => &mut circuits[co.0].outputs[co.1],
        NodeIdx::GI(gi) => gate_get_input_mut(circuits, gates, gi),
        NodeIdx::GO(go) => gate_get_output_mut(circuits, gates, go),
    }
}

pub(crate) fn update(circuits: &mut CircuitMap, gates: &mut GateMap) {
    // TODO: make this update stack of nodes instead of of gates
    let mut update_stack: Vec<_> = gates.iter().map(|(i, _)| i).collect();
    let mut changed = HashSet::new();
    while let Some(gate) = update_stack.pop() {
        if changed.contains(&gate) {
            continue;
        }

        let (old_value, new_value, output_node_idx) = match &gates[gate].calculation.kind {
            CalculationKind::Nand([_, _], [_]) => {
                let a_i = GateInputNodeIdx(gate, 0, ()).into();
                let b_i = GateInputNodeIdx(gate, 1, ()).into();

                let o_i = GateOutputNodeIdx(gate, 0, ()).into();

                (get_node_value(circuits, gates, o_i), !(get_node_value(circuits, gates, a_i) && get_node_value(circuits, gates, b_i)), o_i)
            }
            CalculationKind::Const(_, [_]) => {
                let o_i = GateOutputNodeIdx(gate, 0, ()).into();

                (get_node_value(circuits, gates, o_i), get_node_value(circuits, gates, o_i), o_i)
            }
            CalculationKind::Custom(_) => continue, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
        };
        let gate_changed = old_value != new_value;

        set_node_value(circuits, gates, output_node_idx, Value::Manual(new_value));

        if gate_changed {
            changed.insert(gate);

            for dependant in &get_node(circuits, gates, output_node_idx).dependants {
                // if there is no dependant gate to update, this node is part of a circuit and even if
                // that circuit is a subcircuit, updates to this node will (when this is properly
                // implemented) propagate to this node's dependants, which will update the
                // those other gates that should be updated
                if let Some(dependant_gate) = get_node(circuits, gates, *dependant).gate {
                    update_stack.push(dependant_gate);
                }
            }
        }
    }
}

pub(crate) fn gate_get_input<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, index: GateInputNodeIdx) -> &'c Node {
    // let name = gates[index.0].name();
    let inputs = gate_inputs(circuits, gates, index.0);
    // inputs.get(index.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", index.1, name, inputs.len()))
    inputs.get(index.1).unwrap()
}
pub(crate) fn gate_get_input_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, index: GateInputNodeIdx) -> &'c mut Node {
    // let name = gates[index.0].name();
    let inputs = gate_inputs_mut(circuits, gates, index.0);
    // let len = inputs.len();
    // inputs.get_mut(index.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", index.1, name, len))
    inputs.get_mut(index.1).unwrap()
    // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
    // TODO: there is also probably a better way of doing this that doesnt need
}
pub(crate) fn gate_get_output<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, index: GateOutputNodeIdx) -> &'c Node {
    // let name = gates[index.0].name();
    let outputs = gate_outputs(circuits, gates, index.0);
    // outputs.get(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, outputs.len()))
    outputs.get(index.1).unwrap()
}
pub(crate) fn gate_get_output_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, index: GateOutputNodeIdx) -> &'c mut Node {
    // let name = gates[index.0].name();
    let outputs = gate_outputs_mut(circuits, gates, index.0);
    // let len = outputs.len();
    // outputs.get_mut(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
    outputs.get_mut(index.1).unwrap()
}

pub(crate) fn gate_input_indexes(circuits: &CircuitMap, gates: &GateMap, gate: GateIndex) -> impl ExactSizeIterator<Item = GateInputNodeIdx> {
    (0..gate_num_inputs(circuits, gates, gate)).map(move |i| GateInputNodeIdx(gate, i, ()))
    // a bit strange because GateIndex is Copy so it really shouldnt have to be moved (?)
}

pub(crate) fn gate_output_indexes(circuits: &CircuitMap, gates: &GateMap, gate: GateIndex) -> impl ExactSizeIterator<Item = GateOutputNodeIdx> {
    (0..gate_num_outputs(circuits, gates, gate)).map(move |i| GateOutputNodeIdx(gate, i, ()))
}
pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateIndex) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateIndex) -> usize {
    gate_outputs(circuits, gates, gate).len()
}

pub(crate) fn gate_inputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, gate: GateIndex) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.calculation.kind {
        CalculationKind::Nand(i, _) => i,
        CalculationKind::Const(i, _) => i,
        CalculationKind::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, gate: GateIndex) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.calculation.kind {
        CalculationKind::Nand(_, o) | CalculationKind::Const(_, o) => o,
        CalculationKind::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_inputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, gate: GateIndex) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.calculation.kind {
        CalculationKind::Nand(i, _) => i,
        CalculationKind::Const(i, _) => i,
        CalculationKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, gate: GateIndex) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.calculation.kind {
        CalculationKind::Nand(_, o) | CalculationKind::Const(_, o) => o,
        CalculationKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].outputs,
    }
}
