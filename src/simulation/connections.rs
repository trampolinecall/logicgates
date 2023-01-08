use std::collections::HashSet;
// TODO: rename to calculation and further split into submodules

use generational_arena::Arena;

use super::circuit::{Circuit, CircuitIndex, Gate, GateIndex, GateKind};

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
/*
pub(crate) fn circuit_output_values<'c>(circuits: &'c Arena<Circuit>, gates: &'c Arena<Gate>, circuit: CircuitIndex) -> impl Iterator<Item = bool> + 'c {
    // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
    circuits[circuit].outputs.iter().map(|output| if let Some(producer) = output.producer() { get_node(circuits, gates, producer).value } else { false })
}
*/

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
        NodeIdx::GI(gi) => gate_get_input(&gates[gi.0], gi),
        NodeIdx::GO(go) => gate_get_output(&gates[go.0], go),
    }
}
pub(crate) fn get_node_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: NodeIdx) -> &'c mut Node {
    match index {
        NodeIdx::CI(ci) => &mut circuits[ci.0].inputs[ci.1],
        NodeIdx::CO(co) => &mut circuits[co.0].outputs[co.1],
        NodeIdx::GI(gi) => gate_get_input_mut(&mut gates[gi.0], gi),
        NodeIdx::GO(go) => gate_get_output_mut(&mut gates[go.0], go),
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

fn update_gate(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, update_stack: &mut Vec<GateIndex>, gate: GateIndex) -> bool {
    let gate = &gates[gate];
    let outputs = compute(circuits, gates, &gate.kind);
    assert_eq!(outputs.len(), gate.num_outputs());

    let mut changed = false;

    // TODO: merge this with update and compute and clean up
    for (new_value, output_node) in outputs.into_iter().zip(gate_outputs(gate).collect::<Vec<_>>().into_iter()) {
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

pub(crate) fn gate_get_input(gate: &Gate, input: GateInputNodeIdx) -> &Node {
    assert_eq!(gate.index, input.0, "get input node with index that is not this node");
    let inputs = gate.inputs();
    inputs.get(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, gate.name(), inputs.len()))
}
pub(crate) fn gate_get_input_mut(gate: &mut Gate, input: GateInputNodeIdx) -> &mut Node {
    assert_eq!(gate.index, input.0, "get input node with index that is not this node");
    let name = gate.name();
    let inputs = gate.inputs_mut();
    let len = inputs.len();
    inputs.get_mut(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, name, len))
    // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
    // TODO: there is also probably a better way of doing this that doesnt need
}
pub(crate) fn gate_get_output(gate: &Gate, index: GateOutputNodeIdx) -> &Node {
    assert_eq!(gate.index, index.0, "get output node with index that is not this node");
    let outputs = gate.outputs();
    outputs.get(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, gate.name(), outputs.len()))
}
pub(crate) fn gate_get_output_mut(gate: &mut Gate, index: GateOutputNodeIdx) -> &mut Node {
    assert_eq!(gate.index, index.0, "get output node with index that is not this node");
    let name = gate.name();
    let outputs = gate.outputs_mut();
    let len = outputs.len();
    outputs.get_mut(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
}

pub(crate) fn gate_inputs(gate: &Gate) -> impl ExactSizeIterator<Item = GateInputNodeIdx> + '_ {
    (0..gate.inputs().len()).map(|i| GateInputNodeIdx(gate.index, i, ()))
}

pub(crate) fn gate_outputs(gate: &Gate) -> impl ExactSizeIterator<Item = GateOutputNodeIdx> + '_ {
    (0..gate.outputs().len()).map(|i| GateOutputNodeIdx(gate.index, i, ()))
}

pub(crate) fn compute(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: &GateKind) -> Vec<bool> {
    // TODO: merge this with update

    let get_node_value = |node| get_node_value_not_idx(circuits, gates, node);
    // TODO: figure out a way for this to set its outputs
    match gate {
        GateKind::Nand([a, b], _) => vec![!(get_node_value(a) && get_node_value(b))],
        GateKind::Const(_, [o]) => vec![get_node_value(o)],
        GateKind::Custom(inputs, _, subcircuit) => {
            /*
            // TODO: make passthrough nodes so this does not need to happen
            let mut subcircuit = &mut circuits[*subcircuit];
            for (input_node, subcircuit_input_node) in inputs.iter().zip(circuit_input_indexes(&mut subcircuit)) {
                set_producer_value(circuits, gates, subcircuit_input_node.into(), get_producer_value(input_node.producer));
            }

            circuit_output_indexes(&subcircuit)
                .into_iter()
                .map(|output_idx| if let Some(producer) = get_receiver(circuits, gates, output_idx.into()).producer { get_producer(circuits, gates, producer).value } else { false })
                .collect()
            */
            todo!()
        }
    }
}
