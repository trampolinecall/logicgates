use std::collections::HashSet;

use crate::simulation::{CircuitKey, CircuitMap, Gate, GateKey, GateMap, Node, NodeKey, NodeMap, NodeParent};
pub(crate) use connections::{connect, disconnect};

mod connections;

const SUBTICKS_PER_UPDATE: usize = 1; // TODO: make this adjustable at runtime

pub(crate) struct NodeLogic {
    production: Option<Value>,
    value: Value,
    connections: connections::NodeConnections,
}

pub(crate) struct NandLogic {
    pub(crate) inputs: [NodeKey; 2],
    pub(crate) outputs: [NodeKey; 1],
    _dont_construct: ()
}
pub(crate) struct ConstLogic {
    pub(crate) inputs: [NodeKey; 0],
    pub(crate) outputs: [NodeKey; 1],
    name: &'static str,
}
pub(crate) struct UnerrorLogic {
    pub(crate) inputs: [NodeKey; 1],
    pub(crate) outputs: [NodeKey; 1],
    _dont_construct: ()
}

#[derive(Clone, Copy)]
pub(crate) enum Value {
    H,
    L,
    Z,
    X,
}

impl Value {
    fn join(a: Value, b: Value) -> Value {
        match (a, b) {
            (Value::X, _) | (_, Value::X) => Value::X,
            (Value::Z, b) => b,
            (Value::H, Value::Z) => Value::H,
            (Value::H, _) => Value::X,
            (Value::L, Value::Z) => Value::L,
            (Value::L, _) => Value::X,
        }
    }
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

impl NodeLogic {
    pub(crate) fn new() -> Self {
        NodeLogic { production: None, value: Value::Z, connections: connections::NodeConnections::new() }
        // TODO: reconsider whether this default is actually correct
    }

    pub(crate) fn value(&self) -> Value {
        self.value
    }

    pub(crate) fn adjacent(&self) -> &HashSet<NodeKey> {
        self.connections.adjacent()
    }
}

impl NandLogic {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey) -> NandLogic {
        NandLogic {
            inputs: [
                nodes.insert(Node { logic: { NodeLogic { production: None, value: Value::Z, connections: connections::NodeConnections::new() } }, parent: NodeParent::GateIn(gate_key, 0) }),
                nodes.insert(Node { logic: { NodeLogic { production: None, value: Value::Z, connections: connections::NodeConnections::new() } }, parent: NodeParent::GateIn(gate_key, 1) }),
            ],
            outputs: [nodes.insert(Node { logic: { NodeLogic { production: None, value: Value::Z, connections: connections::NodeConnections::new() } }, parent: NodeParent::GateOut(gate_key, 0) })],
            _dont_construct: (),
        }
    }
    pub(crate) fn name(&self) -> &str {
        "nand"
    }
}

impl ConstLogic {
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey, value: bool) -> ConstLogic {
        ConstLogic {
            inputs: [],
            outputs: [nodes.insert(Node {
                logic: { NodeLogic { production: Some(if value { Value::H } else { Value::L }), value: Value::Z, connections: connections::NodeConnections::new() } },
                parent: NodeParent::GateOut(gate_key, 0),
            })],
            name: if value { "true" } else { "false" },
        }
    }
    pub(crate) fn name(&self) -> &str {
        self.name
    }
}

impl UnerrorLogic {
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey) -> UnerrorLogic {
        UnerrorLogic {
            inputs: [nodes.insert(Node {
                logic: { NodeLogic { production: None, value: Value::Z, connections: connections::NodeConnections::new() } },
                parent: NodeParent::GateIn(gate_key, 0),
            })],
            outputs: [nodes.insert(Node {
                logic: { NodeLogic { production: Some(Value::Z), value: Value::Z, connections: connections::NodeConnections::new() } },
                parent: NodeParent::GateOut(gate_key, 0),
            })],
            _dont_construct: (),
        }
    }
    pub(crate) fn name(&self) -> &str {
        "unerror"
    }
}

// node values {{{1
pub(crate) fn get_node_value(nodes: &NodeMap, node: NodeKey) -> Value {
    nodes[node].logic.value
}
pub(crate) fn get_node_production(nodes: &slotmap::SlotMap<NodeKey, Node>, node: NodeKey) -> Option<Value> {
    nodes[node].logic.production
}

fn set_node_production(nodes: &mut NodeMap, index: NodeKey, new_value: Value) {
    nodes[index].logic.production = Some(new_value);
}

pub(crate) fn toggle_input(circuits: &mut CircuitMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];
    let node = &nodes[node_key];
    let old_production = node.logic.production;

    set_node_production(
        nodes,
        node_key,
        match old_production {
            Some(Value::L) => Value::H,
            Some(Value::H) => Value::L,
            Some(Value::Z) | Some(Value::X) | None => Value::H,
        },
    );
}

pub(crate) fn set_input(circuits: &mut CircuitMap, nodes: &mut NodeMap, circuit: CircuitKey, i: usize, value: Value) {
    assert!(i < circuits[circuit].inputs.len(), "set input out of range of number of inputs");
    let node_key = circuits[circuit].inputs[i];

    set_node_production(nodes, node_key, value);
}
// update {{{1
pub(crate) fn update(gates: &mut GateMap, nodes: &mut NodeMap) {
    use std::collections::BTreeMap;
    for _ in 0..SUBTICKS_PER_UPDATE {
        // all gates calculate their values based on the values of the nodes in the previous subtick and then all updates get applied all at once
        let gate_outputs: Vec<(NodeKey, Value)> = gates
            .iter()
            .filter_map(|(_, gate)| -> Option<(NodeKey, Value)> {
                match &gate {
                    Gate::Nand { logic: NandLogic { inputs: [a, b], outputs: [o], _dont_construct: () }, location: _ } => {
                        let a_value = nodes[*a].logic.value;
                        let b_value = nodes[*b].logic.value;

                        Some((
                            *o,
                            match (a_value, b_value) {
                                (Value::H, Value::H) => Value::L,
                                (Value::H, Value::L) => Value::H,
                                (Value::L, Value::H) => Value::H,
                                (Value::L, Value::L) => Value::H,
                                (Value::Z, _) | (_, Value::Z) | (_, Value::X) | (Value::X, _) => Value::X,
                            },
                        ))
                    }
                    Gate::Const { logic: ConstLogic { inputs: _, outputs: _, name: _ }, location: _ } => None, // const nodes do not need to update becuase they always output the value they were created with
                    Gate::Unerror { logic: UnerrorLogic { inputs: [in_], outputs: [out], _dont_construct: () }, location: _ } => {
                        let in_value = nodes[*in_].logic.value;
                        Some((*out, if let Value::X = in_value { Value::L } else { in_value }))
                    }
                    Gate::Custom(_) => None, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
                }
            })
            .collect();

        for (node, value) in gate_outputs {
            set_node_production(nodes, node, value);
        }

        // TODO: find a more efficient way to do this (disjoint union set data structure?)
        let mut node_values: BTreeMap<_, _> = nodes.iter().map(|(nk, _)| (nk, Value::Z)).collect();
        for (cur_node, node) in &*nodes {
            if let Some(production) = node.logic.production {
                let mut already: HashSet<_> = HashSet::new();
                let mut queue: Vec<_> = node.logic.adjacent().iter().copied().chain(std::iter::once(cur_node)).collect();
                while let Some(adj) = queue.pop() {
                    if already.contains(&adj) {
                        continue;
                    }
                    already.insert(adj);

                    let result = Value::join(node_values[&adj], production);
                    *node_values.get_mut(&adj).unwrap() = result;

                    queue.extend(nodes[adj].logic.adjacent());
                }
            }
        }
        for (node, value) in node_values {
            nodes[node].logic.value = value;
        }
    }
}
