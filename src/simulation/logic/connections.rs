use std::collections::HashSet;

use crate::simulation::{NodeKey, NodeMap};

pub(super) struct NodeConnections {
    adjacent: HashSet<NodeKey>,
}

impl NodeConnections {
    pub(super) fn new() -> Self {
        Self { adjacent: HashSet::new() }
    }

    pub(super) fn adjacent(&self) -> &HashSet<NodeKey> {
        &self.adjacent
    }
}

pub(crate) fn connect(nodes: &mut NodeMap, a: NodeKey, b: NodeKey) {
    nodes[a].logic.connections.adjacent.insert(b);
    nodes[b].logic.connections.adjacent.insert(a);
}
pub(crate) fn disconnect(nodes: &mut NodeMap, a: NodeKey, b: NodeKey) {
    nodes[a].logic.connections.adjacent.remove(&b);
    nodes[b].logic.connections.adjacent.remove(&a);
}
