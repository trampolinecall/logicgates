use crate::simulation::{CircuitKey, CircuitMap, GateKey, GateMap, Node, NodeKey, NodeMap, NodeParent};

const SUBTICKS_PER_UPDATE: usize = 1; // TODO: make this adjustable at runtime

#[derive(Clone)]
pub(crate) struct NodeLogic {
    // pub(crate) gate: Option<GateKey>,
    value: Value,
}

pub(crate) struct GateLogic(GateLogicKind);
enum GateLogicKind {
    Nand([NodeKey; 2], [NodeKey; 1]),
    Const([NodeKey; 0], [NodeKey; 1], &'static str),
    Custom(CircuitKey),
}

#[derive(Clone, Copy)]
enum Value {
    Manual(bool),
    Passthrough(NodeKey),
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

impl GateLogic {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(nodes: &mut NodeMap, gate_key: GateKey) -> GateLogic {
        GateLogic(GateLogicKind::Nand(
            [
                nodes.insert(Node { value: { NodeLogic { value: Value::Manual(false) } }, parent: NodeParent::Gate(gate_key) }),
                nodes.insert(Node { value: { NodeLogic { value: Value::Manual(false) } }, parent: NodeParent::Gate(gate_key) }),
            ],
            [nodes.insert(Node { value: { NodeLogic { value: Value::Manual(true) } }, parent: NodeParent::Gate(gate_key) })],
        ))
    }
    pub(crate) fn new_const(nodes: &mut NodeMap, gate_key: GateKey, value: bool) -> GateLogic {
        GateLogic(GateLogicKind::Const([], [nodes.insert(Node { value: { NodeLogic { value: Value::Manual(value) } }, parent: NodeParent::Gate(gate_key) })], if value { "true" } else { "false" }))
    }
    pub(crate) fn new_subcircuit(_: &mut NodeMap, _: GateKey, subcircuit: CircuitKey) -> GateLogic {
        GateLogic(GateLogicKind::Custom(subcircuit))
    }

    pub(crate) fn name<'c>(&self, circuits: &'c CircuitMap) -> &'c str {
        match self.0 {
            GateLogicKind::Nand(_, _) => "nand",
            GateLogicKind::Const(_, _, name) => name,
            GateLogicKind::Custom(ck) => &circuits[ck].name,
        }
    }

    pub(crate) fn as_subcircuit(&self) -> Option<CircuitKey> {
        match self.0 {
            GateLogicKind::Nand(_, _) | GateLogicKind::Const(_, _, _) => None,
            GateLogicKind::Custom(ck) => Some(ck),
        }
    }
}

impl NodeLogic {
    pub(crate) fn new() -> Self {
        Self { value: Value::Manual(false) } // TODO: reconsider whether this default is actually correct
    }

    pub(crate) fn producer(&self) -> Option<NodeKey> {
        if let Value::Passthrough(v) = self.value {
            Some(v)
        } else {
            None
        }
    }
}

// inputs and outputs {{{1
pub(crate) fn gate_inputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate].logic.0 {
        GateLogicKind::Nand(i, _) => i,
        GateLogicKind::Const(i, _, _) => i,
        GateLogicKind::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate].logic.0 {
        GateLogicKind::Nand(_, o) | GateLogicKind::Const(_, o, _) => o,
        GateLogicKind::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_outputs(circuits, gates, gate).len()
}
// node values {{{1
// TODO: test connection, replacing old connection
pub(crate) fn connect(nodes: &mut NodeMap, producer_idx: NodeKey, receiver_idx: NodeKey) {
    set_node_value(nodes, receiver_idx, Value::Passthrough(producer_idx));
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(nodes: &mut NodeMap, node: NodeKey) {
    set_node_value(nodes, node, Value::Manual(false));
}

pub(crate) fn get_node_value(nodes: &NodeMap, node: NodeKey) -> bool {
    match nodes[node].value.value {
        Value::Manual(v) => v,
        Value::Passthrough(other) => get_node_value(nodes, other),
    }
}

// TODO: test every possibility
fn set_node_value(nodes: &mut NodeMap, index: NodeKey, new_value: Value) {
    nodes[index].value = NodeLogic { value: new_value };
}

pub(crate) fn toggle_input(circuits: &mut CircuitMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];
    assert!(!matches!(nodes[node_key].value.value, Value::Passthrough(_)), "toggle input that is a passthrough");

    set_node_value(nodes, node_key, Value::Manual(!get_node_value(nodes, node_key)));
}

pub(crate) fn set_input(circuits: &mut CircuitMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize, value: bool) {
    assert!(i < circuits[circuit].inputs.len(), "set input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];
    assert!(!matches!(nodes[node_key].value.value, Value::Passthrough(_)), "set input that is a passthrough");

    set_node_value(nodes, node_key, Value::Manual(value));
}
// update {{{1
pub(crate) fn update(gates: &mut GateMap, nodes: &mut NodeMap) {
    for _ in 0..SUBTICKS_PER_UPDATE {
        // all gates calculate their values based on the values of the nodes in the previous subtick and then all updates get applied all at once
        let node_values: Vec<(NodeKey, Value)> = gates
            .iter()
            .filter_map(|(_, gate)| {
                match &gate.logic.0 {
                    GateLogicKind::Nand([a, b], [o]) => Some((*o, Value::Manual(!(get_node_value(nodes, *a) && get_node_value(nodes, *b))))),
                    GateLogicKind::Const(_, _, _) => None, // const nodes do not need to update becuase they always output the value they were created with
                    GateLogicKind::Custom(_) => None, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
                }
            })
            .collect();

        for (node, value) in node_values {
            set_node_value(nodes, node, value);
        }
    }
}
