use std::collections::HashSet;

use crate::simulation::{NodeKey, NodeMap};

pub(crate) struct Connections {

}

pub(crate) struct NodeConnections {
    adjacent: HashSet<NodeKey>,
}

impl Connections {
    pub(crate) fn new() -> Self { Self {  } }
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
    nodes[a].connections.adjacent.insert(b);
    nodes[b].connections.adjacent.insert(a);
}
pub(crate) fn disconnect(nodes: &mut NodeMap, a: NodeKey, b: NodeKey) {
    nodes[a].connections.adjacent.remove(&b);
    nodes[b].connections.adjacent.remove(&a);
}

