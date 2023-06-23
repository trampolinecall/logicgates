use std::collections::HashSet;

use crate::simulation::{hierarchy, Gate, GateKey, GateMap, Node, NodeKey, NodeMap};

pub(crate) struct NodeLogic {
    production: Option<Value>,
    value: Value,
}

pub(crate) struct NandLogic {
    pub(crate) nodes: hierarchy::NodeChildren<[NodeKey; 2], [NodeKey; 1]>,
    _dont_construct: (),
}
pub(crate) struct ConstLogic {
    pub(crate) nodes: hierarchy::NodeChildren<[NodeKey; 0], [NodeKey; 1]>,
    name: &'static str,
}
pub(crate) struct UnerrorLogic {
    pub(crate) nodes: hierarchy::NodeChildren<[NodeKey; 1], [NodeKey; 1]>,
    _dont_construct: (),
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
        NodeLogic { production: None, value: Value::Z }
    }
}

impl NandLogic {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey) -> NandLogic {
        NandLogic { nodes: hierarchy::NodeChildren::new(nodes, hierarchy::NodeParentType::Gate(gate_key), (), ()), _dont_construct: () }
    }
    pub(crate) fn name(&self) -> &str {
        "nand"
    }
}

impl ConstLogic {
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey, value: bool) -> ConstLogic {
        let gate_nodes: hierarchy::NodeChildren<[NodeKey; 0], [NodeKey; 1]> = hierarchy::NodeChildren::new(nodes, hierarchy::NodeParentType::Gate(gate_key), (), ());
        set_node_production(nodes, gate_nodes.outputs()[0], if value { Value::H } else { Value::L });
        ConstLogic { nodes: gate_nodes, name: if value { "true" } else { "false" } }
    }
    pub(crate) fn name(&self) -> &str {
        self.name
    }
}

impl UnerrorLogic {
    pub(crate) fn new(nodes: &mut NodeMap, gate_key: GateKey) -> UnerrorLogic {
        UnerrorLogic { nodes: hierarchy::NodeChildren::new(nodes, hierarchy::NodeParentType::Gate(gate_key), (), ()), _dont_construct: () }
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
// update {{{1
pub(crate) fn update(gates: &mut GateMap, node_map: &mut NodeMap, subticks: usize) {
    use std::collections::BTreeMap;
    for _ in 0..subticks {
        // all gates calculate their values based on the values of the nodes in the previous subtick and then all updates get applied all at once
        let gate_outputs: Vec<(NodeKey, Value)> = gates
            .iter()
            .filter_map(|(_, gate)| -> Option<(NodeKey, Value)> {
                match &gate {
                    Gate::Nand { logic: NandLogic { nodes: logic_nodes, _dont_construct: () }, location: _, direction } => {
                        let [a, b] = logic_nodes.inputs();
                        let [o] = logic_nodes.outputs();
                        let a_value = node_map[*a].logic.value;
                        let b_value = node_map[*b].logic.value;

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
                    Gate::Const { logic: ConstLogic { nodes: _, name: _ }, location: _, direction } => None, // const nodes do not need to update becuase they always output the value they were created with
                    Gate::Unerror { logic: UnerrorLogic { nodes: logic_nodes, _dont_construct: () }, location: _, direction } => {
                        let [in_] = logic_nodes.inputs();
                        let [out] = logic_nodes.outputs();
                        let in_value = node_map[*in_].logic.value;
                        Some((*out, if let Value::X = in_value { Value::L } else { in_value }))
                    }
                    Gate::Custom(_) => None, // custom gates do not have to compute values because their nodes are connected to their inputs or are passthrough nodes and should automatically have the right values
                }
            })
            .collect();

        for (node, value) in gate_outputs {
            set_node_production(node_map, node, value);
        }

        // TODO: find a more efficient way to do this (disjoint union set data structure?)
        let mut node_values: BTreeMap<_, _> = node_map.iter().map(|(nk, _)| (nk, Value::Z)).collect();
        for (cur_node, node) in &*node_map {
            if let Some(production) = node.logic.production {
                let mut already: HashSet<_> = HashSet::new();
                let mut queue: Vec<_> = node.connections.adjacent().iter().copied().chain(std::iter::once(cur_node)).collect();
                while let Some(adj) = queue.pop() {
                    if already.contains(&adj) {
                        continue;
                    }
                    already.insert(adj);

                    let result = Value::join(node_values[&adj], production);
                    *node_values.get_mut(&adj).unwrap() = result;

                    queue.extend(node_map[adj].connections.adjacent());
                }
            }
        }
        for (node, value) in node_values {
            node_map[node].logic.value = value;
        }
    }
}
