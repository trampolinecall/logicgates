use crate::simulation::{Circuit, CircuitKey, CircuitMap, GateKey, GateKind, GateMap, NodeKey, NodeMap, Simulation};

const SUBTICKS_PER_UPDATE: usize = 1; // TODO: make this adjustable at runtime

#[derive(Clone)]
pub(crate) struct NodeValue {
    // pub(crate) gate: Option<GateKey>,
    value: Value,
}
#[derive(Clone, Copy)]
enum Value {
    Manual(bool),
    Passthrough(NodeKey),
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

/*
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
*/

impl NodeValue {
    pub(crate) fn new(value: bool) -> Self {
        Self { /* gate, */ value: Value::Manual(value) }
    }

    pub(crate) fn producer(&self) -> Option<NodeKey> {
        if let Value::Passthrough(v) = self.value {
            Some(v)
        } else {
            None
        }
    }
}

// node values {{{1
// TODO: test connection, replacing old connection
pub(crate) fn connect(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap, producer_idx: NodeKey, receiver_idx: NodeKey) {
    set_node_value(circuits, gates, nodes, receiver_idx, Value::Passthrough(producer_idx));
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap, node: NodeKey) {
    set_node_value(circuits, gates, nodes, node, Value::Manual(false));
}

pub(crate) fn get_node_value(circuits: &CircuitMap, gates: &GateMap, nodes: &NodeMap, node: NodeKey) -> bool {
    match nodes[node].value.value {
        Value::Manual(v) => v,
        Value::Passthrough(other) => get_node_value(circuits, gates, nodes, other),
    }
}

// TODO: test every possibility
fn set_node_value(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap, index: NodeKey, new_value: Value) {
    nodes[index].value = NodeValue { value: new_value };
}

pub(crate) fn toggle_input(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];
    assert!(!matches!(nodes[node_key].value.value, Value::Passthrough(_)), "toggle input that is a passthrough");

    set_node_value(circuits, gates, nodes, node_key, Value::Manual(!get_node_value(circuits, gates, nodes, node_key)));
}

pub(crate) fn set_input(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize, value: bool) {
    assert!(i < circuits[circuit].inputs.len(), "set input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];
    assert!(!matches!(nodes[node_key].value.value, Value::Passthrough(_)), "set input that is a passthrough");

    set_node_value(circuits, gates, nodes, node_key, Value::Manual(value));
}
// update {{{1
pub(crate) fn update(circuits: &mut CircuitMap, gates: &mut GateMap, nodes: &mut NodeMap) {
    use std::collections::HashMap;
    for _ in 0..SUBTICKS_PER_UPDATE {
        // all gates calculate their values based on the values of the nodes in the previous subtick and then all updates get applied all at once
        let node_values: HashMap<NodeKey, Value> = gates
            .iter()
            .filter_map(|(gate_i, gate)| {
                match &gate.kind {
                    GateKind::Nand([a, b], [o]) => Some((*o, Value::Manual(!(get_node_value(circuits, gates, nodes, *a) && get_node_value(circuits, gates, nodes, *b))))),
                    GateKind::Const(_, _) => None, // const nodes do not need to update becuase they always output the value they were created with
                    GateKind::Custom(_) => None, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
                }
            })
            .collect();

        for (node, value) in node_values {
            set_node_value(circuits, gates, nodes, node, value);
        }
    }
}
