use crate::simulation::{Circuit, CircuitKey, CircuitMap, GateKey, GateMap};

pub(crate) struct Calculation {
    kind: CalculationKind,
}
#[allow(clippy::large_enum_variant)] // TODO: reconsider whether this is correct
enum CalculationKind {
    Nand([Node; 2], [Node; 1]),
    Const([Node; 0], [Node; 1]),
    Custom(CircuitKey), // the circuit already contains the input and output nodes
}

const SUBTICKS_PER_UPDATE: usize = 1; // TODO: make this adjustable at runtime

#[derive(Clone)]
pub(crate) struct Node {
    pub(crate) gate: Option<GateKey>,
    value: Value,
}
#[derive(Clone, Copy)]
enum Value {
    Manual(bool),
    Passthrough(NodeIdx),
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateInputNodeIdx(pub(crate) GateKey, pub(crate) usize, ()); // unit at the end so that it cannot be constructed outside of this module
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateOutputNodeIdx(pub(crate) GateKey, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitInputNodeIdx(pub(crate) CircuitKey, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOutputNodeIdx(pub(crate) CircuitKey, pub(crate) usize, ());

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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum NodeIdx {
    CI(CircuitInputNodeIdx),
    CO(CircuitOutputNodeIdx),
    GI(GateInputNodeIdx),
    GO(GateOutputNodeIdx),
}

impl Calculation {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(index: GateKey) -> Calculation {
        Calculation { kind: CalculationKind::Nand([Node::new(Some(index), false), Node::new(Some(index), false)], [Node::new(Some(index), true)]) }
    }
    pub(crate) fn new_const(index: GateKey, value: bool) -> Calculation {
        Calculation { kind: CalculationKind::Const([], [Node::new(Some(index), value)]) }
    }
    pub(crate) fn new_subcircuit(subcircuit: CircuitKey) -> Calculation {
        Calculation { kind: CalculationKind::Custom(subcircuit) }
    }

    pub(crate) fn name<'a>(&self, circuits: &'a CircuitMap) -> &'a str {
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
    pub(crate) fn new(gate: Option<GateKey>, value: bool) -> Self {
        Self { gate, value: Value::Manual(value) }
    }

    pub(crate) fn producer(&self) -> Option<NodeIdx> {
        if let Value::Passthrough(v) = self.value {
            Some(v)
        } else {
            None
        }
    }
}

// inputs and outputs {{{1
fn gate_inputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, gate: GateKey) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.calculation.kind {
        CalculationKind::Nand(i, _) => i,
        CalculationKind::Const(i, _) => i,
        CalculationKind::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
fn gate_outputs<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, gate: GateKey) -> &'c [Node] {
    let gate = &gates[gate];
    match &gate.calculation.kind {
        CalculationKind::Nand(_, o) | CalculationKind::Const(_, o) => o,
        CalculationKind::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

fn gate_inputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, gate: GateKey) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.calculation.kind {
        CalculationKind::Nand(i, _) => i,
        CalculationKind::Const(i, _) => i,
        CalculationKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].inputs,
    }
}
fn gate_outputs_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, gate: GateKey) -> &'c mut [Node] {
    let gate = &mut gates[gate];
    match &mut gate.calculation.kind {
        CalculationKind::Nand(_, o) | CalculationKind::Const(_, o) => o,
        CalculationKind::Custom(circuit_idx) => &mut circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_outputs(circuits, gates, gate).len()
}

// enumerating indexes {{{2
pub(crate) fn circuit_input_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitInputNodeIdx> + '_ {
    (0..circuit.inputs.len()).map(|i| CircuitInputNodeIdx(circuit.index, i, ()))
}
pub(crate) fn circuit_output_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitOutputNodeIdx> + '_ {
    (0..circuit.outputs.len()).map(|i| CircuitOutputNodeIdx(circuit.index, i, ()))
}
pub(crate) fn gate_input_indexes(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> impl ExactSizeIterator<Item = GateInputNodeIdx> {
    (0..gate_num_inputs(circuits, gates, gate)).map(move |i| GateInputNodeIdx(gate, i, ()))
}
pub(crate) fn gate_output_indexes(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> impl ExactSizeIterator<Item = GateOutputNodeIdx> {
    (0..gate_num_outputs(circuits, gates, gate)).map(move |i| GateOutputNodeIdx(gate, i, ()))
}
// getting nodes {{{1
fn gate_get_input<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, index: GateInputNodeIdx) -> &'c Node {
    let inputs = gate_inputs(circuits, gates, index.0);
    inputs.get(index.1).unwrap()
}
fn gate_get_input_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, index: GateInputNodeIdx) -> &'c mut Node {
    let inputs = gate_inputs_mut(circuits, gates, index.0);
    inputs.get_mut(index.1).unwrap()
    // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
    // TODO: there is also probably a better way of doing this that doesnt need
}
fn gate_get_output<'a: 'c, 'b: 'c, 'c>(circuits: &'a CircuitMap, gates: &'b GateMap, index: GateOutputNodeIdx) -> &'c Node {
    let outputs = gate_outputs(circuits, gates, index.0);
    outputs.get(index.1).unwrap()
}
fn gate_get_output_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut CircuitMap, gates: &'b mut GateMap, index: GateOutputNodeIdx) -> &'c mut Node {
    let outputs = gate_outputs_mut(circuits, gates, index.0);
    outputs.get_mut(index.1).unwrap()
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
// node values {{{1
// TODO: test connection, replacing old connection
pub(crate) fn connect(circuits: &mut CircuitMap, gates: &mut GateMap, producer_idx: NodeIdx, receiver_idx: NodeIdx) {
    set_node_value(circuits, gates, receiver_idx, Value::Passthrough(producer_idx));
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(circuits: &mut CircuitMap, gates: &mut GateMap, node: NodeIdx) {
    set_node_value(circuits, gates, node, Value::Manual(false));
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
    get_node_mut(circuits, gates, index).value = new_value;
}

pub(crate) fn toggle_input(circuits: &mut CircuitMap, gates: &mut GateMap, circuit: CircuitKey, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    set_input(circuits, gates, CircuitInputNodeIdx(circuit, i, ()), !get_node_value(circuits, gates, CircuitInputNodeIdx(circuit, i, ()).into()));
}

pub(crate) fn set_input(circuits: &mut CircuitMap, gates: &mut GateMap, ci: CircuitInputNodeIdx, value: bool) {
    set_node_value(circuits, gates, ci.into(), Value::Manual(value));
}
// update {{{1
pub(crate) fn update(circuits: &mut CircuitMap, gates: &mut GateMap) {
    use std::collections::HashMap;
    for _ in 0..SUBTICKS_PER_UPDATE {
        // all gates calculate their values based on the values of the nodes in the previous subtick and then all updates get applied all at once
        let node_values: HashMap<NodeIdx, Value> = gates
            .iter()
            .filter_map(|(gate_i, gate)| {
                if let CalculationKind::Nand([_, _], [_]) = &gate.calculation.kind {
                    let a_i = GateInputNodeIdx(gate_i, 0, ()).into();
                    let b_i = GateInputNodeIdx(gate_i, 1, ()).into();

                    let o_i = GateOutputNodeIdx(gate_i, 0, ()).into();

                    Some((o_i, Value::Manual(!(get_node_value(circuits, gates, a_i) && get_node_value(circuits, gates, b_i)))))
                } else {
                    // const nodes do not need to update becuase they always output the value they were created with
                    // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
                    None
                }
            })
            .collect();

        for (node, value) in node_values {
            set_node_value(circuits, gates, node, value);
        }
    }
}
