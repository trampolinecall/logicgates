use std::collections::{hash_set, HashSet};

use crate::simulation::{NodeKey, NodeMap};

pub(crate) struct Connections {
    connections: HashSet<(NodeKey, NodeKey)>,
}

pub(crate) struct NodeConnections {
    adjacent: HashSet<NodeKey>,
}

impl Connections {
    pub(crate) fn new() -> Self {
        Self { connections: HashSet::new() }
    }

    pub(crate) fn iter(&self) -> hash_set::Iter<(NodeKey, NodeKey)> {
        self.connections.iter()
    }
}

impl NodeConnections {
    pub(super) fn new() -> Self {
        Self { adjacent: HashSet::new() }
    }

    pub(super) fn adjacent(&self) -> &HashSet<NodeKey> {
        &self.adjacent
    }
}

pub(crate) fn connect(connections: &mut Connections, nodes: &mut NodeMap, a: NodeKey, b: NodeKey) {
    let (lower, higher) = if a < b { (a, b) } else { (b, a) };
    connections.connections.insert((lower, higher));
    nodes[lower].connections.adjacent.insert(higher);
    nodes[higher].connections.adjacent.insert(lower);
}
pub(crate) fn disconnect(connections: &mut Connections, nodes: &mut NodeMap, a: NodeKey, b: NodeKey) {
    let (lower, higher) = if a < b { (a, b) } else { (b, a) };
    connections.connections.remove(&(lower, higher));
    nodes[lower].connections.adjacent.remove(&higher);
    nodes[higher].connections.adjacent.remove(&lower);
}
