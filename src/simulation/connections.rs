use std::collections::HashSet;
// TODO: rename to calculation and further split into submodules

use generational_arena::Arena;

use super::circuit::{Circuit, CircuitIndex, Gate, GateIndex, GateKind};

// TODO: reorganize this module

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
    Disconnected, // same as Manual(false)
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

impl Node {
    pub(crate) fn new_value(gate: Option<GateIndex>, value: bool) -> Self {
        Self { gate, value: Value::Manual(value), dependants: HashSet::new() }
    }

    pub(crate) fn new_passthrough(gate: Option<GateIndex>, other: NodeIdx) -> Self {
        Self { gate, value: Value::Passthrough(other), dependants: HashSet::new() }
    }

    pub(crate) fn new_disconnected(gate: Option<GateIndex>) -> Self {
        Self { gate, value: Value::Disconnected, dependants: HashSet::new() }
    }

    pub(crate) fn producer(&self) -> Option<NodeIdx> {
        match self.value {
            Value::Manual(_) | Value::Disconnected => None,
            Value::Passthrough(v) => Some(v),
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
pub(crate) fn connect(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, producer_idx: NodeIdx, receiver_idx: NodeIdx) {
    set_node_value(circuits, gates, receiver_idx, Value::Passthrough(producer_idx))
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, node: NodeIdx) {
    set_node_value(circuits, gates, node, Value::Disconnected)
}

pub(crate) fn get_node_value(circuits: &Arena<Circuit>, gates: &Arena<Gate>, node: NodeIdx) -> bool {
    let node = get_node(circuits, gates, node);
    get_node_value_not_idx(circuits, gates, node)
}
// TODO: this function is only used in compute(), figure out a better way
fn get_node_value_not_idx(circuits: &Arena<Circuit>, gates: &Arena<Gate>, node: &Node) -> bool {
    match node.value {
        Value::Manual(v) => v,
        Value::Passthrough(other) => get_node_value(circuits, gates, other),
        Value::Disconnected => false,
    }
}

// TODO: test every possibility
fn set_node_value(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, index: NodeIdx, new_value: Value) {
    // remove any existing connection:
    let node = get_node_mut(circuits, gates, index);
    if let Value::Passthrough(other_idx) = node.value {
        let other = get_node_mut(circuits, gates, other_idx);
        other.dependants.remove(&index);
    }
    get_node_mut(circuits, gates, index).value = Value::Disconnected;

    // set the new value:
    get_node_mut(circuits, gates, index).value = new_value;
    if let Value::Passthrough(new_dep) = new_value {
        get_node_mut(circuits, gates, new_dep).dependants.insert(index);
    }
}

pub(crate) fn toggle_input(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, circuit: CircuitIndex, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    set_input(circuits, gates, CircuitInputNodeIdx(circuit, i, ()), !get_node_value(circuits, gates, CircuitInputNodeIdx(circuit, i, ()).into()));
}

pub(crate) fn set_input(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, ci: CircuitInputNodeIdx, value: bool) {
    set_node_value(circuits, gates, ci.into(), Value::Manual(value));
    // TODO: consider whether or not this is supposed to call update()
}

pub(crate) fn get_node<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, index: NodeIdx) -> &'c Node {
    match index {
        NodeIdx::CI(ci) => &circuits[ci.0].inputs[ci.1],
        NodeIdx::CO(co) => &circuits[co.0].outputs[co.1],
        NodeIdx::GI(gi) => gate_get_input(circuits, gates, gi),
        NodeIdx::GO(go) => gate_get_output(circuits, gates, go),
    }
}
pub(crate) fn get_node_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: NodeIdx) -> &'c mut Node {
    match index {
        NodeIdx::CI(ci) => &mut circuits[ci.0].inputs[ci.1],
        NodeIdx::CO(co) => &mut circuits[co.0].outputs[co.1],
        NodeIdx::GI(gi) => gate_get_input_mut(circuits, gates, gi),
        NodeIdx::GO(go) => gate_get_output_mut(circuits, gates, go),
    }
}

pub(crate) fn update(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>) {
    // TODO: make this update stack of nodes instead of of gates
    let mut stack: Vec<_> = gates.iter().map(|(i, _)| i).collect();
    let mut changed = HashSet::new();
    while let Some(gate) = stack.pop() {
        if changed.contains(&gate) {
            continue;
        }

        let gate_changed = update_gate(circuits, gates, &mut stack, gate);
        if gate_changed {
            changed.insert(gate);
        }
    }
}

fn update_gate(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, update_stack: &mut Vec<GateIndex>, gate_index: GateIndex) -> bool {
    let gate = &gates[gate_index];
    let Some(outputs) = compute(circuits, gates, &gate.kind) else { return false };
    assert_eq!(outputs.len(), gate_num_outputs(circuits, gates, gate_index));

    let mut changed = false;

    // TODO: merge this with update and compute and clean up
    for (new_value, output_node) in outputs.into_iter().zip(gate_output_indexes(circuits, gates, gate_index).collect::<Vec<_>>().into_iter()) {
        // TODO: dont "clone" the iterator
        let as_producer_index = output_node.into();
        let old_value = get_node_value(circuits, gates, as_producer_index);
        if old_value != new_value {
            changed = true;
        }
        set_node_value(circuits, gates, as_producer_index, Value::Manual(new_value));

        for dependant in &get_node(circuits, gates, as_producer_index).dependants {
            // if there is no dependant gate to update, this node is part of a circuit and even if
            // that circuit is a subcircuit, updates to this node will (when this is properly
            // implemented) propagate to this node's dependants, which will update the
            // those other gates that should be updated
            if let Some(dependant_gate) = get_node(circuits, gates, *dependant).gate {
                update_stack.push(dependant_gate);
            }
        }
    }

    changed
}

pub(crate) fn gate_get_input<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, index: GateInputNodeIdx) -> &'c Node {
    // let name = gates[index.0].name();
    let inputs = gate_inputs(circuits, gates, index.0);
    // inputs.get(index.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", index.1, name, inputs.len()))
    inputs.get(index.1).unwrap()
}
pub(crate) fn gate_get_input_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: GateInputNodeIdx) -> &'c mut Node {
    // let name = gates[index.0].name();
    let inputs = gate_inputs_mut(circuits, gates, index.0);
    // let len = inputs.len();
    // inputs.get_mut(index.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", index.1, name, len))
    inputs.get_mut(index.1).unwrap()
    // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
    // TODO: there is also probably a better way of doing this that doesnt need
}
pub(crate) fn gate_get_output<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, index: GateOutputNodeIdx) -> &'c Node {
    // let name = gates[index.0].name();
    let outputs = gate_outputs(circuits, gates, index.0);
    // outputs.get(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, outputs.len()))
    outputs.get(index.1).unwrap()
}
pub(crate) fn gate_get_output_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: GateOutputNodeIdx) -> &'c mut Node {
    // let name = gates[index.0].name();
    let outputs = gate_outputs_mut(circuits, gates, index.0);
    // let len = outputs.len();
    // outputs.get_mut(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
    outputs.get_mut(index.1).unwrap()
}

pub(crate) fn gate_input_indexes(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: GateIndex) -> impl ExactSizeIterator<Item = GateInputNodeIdx> {
    (0..gate_num_inputs(circuits, gates, gate)).map(move |i| GateInputNodeIdx(gate, i, ()))
    // a bit strange because GateIndex is Copy so it really shouldnt have to be moved (?)
}

pub(crate) fn gate_output_indexes(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: GateIndex) -> impl ExactSizeIterator<Item = GateOutputNodeIdx> {
    (0..gate_num_outputs(circuits, gates, gate)).map(move |i| GateOutputNodeIdx(gate, i, ()))
}
pub(crate) fn gate_num_inputs(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: GateIndex) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: GateIndex) -> usize {
    gate_outputs(circuits, gates, gate).len()
}

pub(crate) fn gate_inputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, gate: GateIndex) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.kind {
        GateKind::Nand(i, _) => i,
        GateKind::Const(i, _) => i,
        GateKind::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, gate: GateIndex) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.kind {
        GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
        GateKind::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_inputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, gate: GateIndex) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.kind {
        GateKind::Nand(i, _) => i,
        GateKind::Const(i, _) => i,
        GateKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, gate: GateIndex) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.kind {
        GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
        GateKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].outputs,
    }
}

fn compute(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: &GateKind) -> Option<Vec<bool>> {
    // TODO: merge this function with update

    let get_node_value = |node| get_node_value_not_idx(circuits, gates, node);
    // TODO: figure out a way for this to set its outputs
    match gate {
        GateKind::Nand([a, b], _) => Some(vec![!(get_node_value(a) && get_node_value(b))]),
        GateKind::Const(_, [o]) => Some(vec![get_node_value(o)]),
        GateKind::Custom(_) => None, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
    }
}
