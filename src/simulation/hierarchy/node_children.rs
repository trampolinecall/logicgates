use crate::{simulation::{connections, logic, CircuitKey, GateKey, Node, NodeKey, NodeMap}, ui};

pub(crate) struct NodeChildren<I: private::NodeVec, O: private::NodeVec> {
    inputs: I,
    outputs: O,
}

#[derive(Copy, Clone)]
pub(crate) struct NodeParent {
    kind: NodeParentKind,
}

#[derive(Copy, Clone)]
pub(crate) enum NodeParentKind {
    GateIn(GateKey, usize),
    GateOut(GateKey, usize),
    CircuitIn(CircuitKey, usize),
    CircuitOut(CircuitKey, usize),
}

#[derive(Copy, Clone)]
pub(crate) enum NodeParentType {
    Gate(GateKey),
    Circuit(CircuitKey),
}

mod private {
    use crate::simulation::NodeKey;

    pub(crate) trait NodeVec {
        type ExtraData;
        fn from_node_generator(next: impl FnMut() -> NodeKey, extra: Self::ExtraData) -> Self;
    }
}

impl NodeParent {
    pub(crate) fn get_node_parent_type(&self) -> NodeParentType {
        match self.kind {
            NodeParentKind::GateIn(g, _) | NodeParentKind::GateOut(g, _) => NodeParentType::Gate(g),
            NodeParentKind::CircuitIn(c, _) | NodeParentKind::CircuitOut(c, _) => NodeParentType::Circuit(c),
        }
    }

    pub(crate) fn kind(&self) -> NodeParentKind {
        self.kind
    }
}

impl<I: private::NodeVec, O: private::NodeVec> NodeChildren<I, O> {
    pub(crate) fn new(nodes: &mut NodeMap, parent_type: NodeParentType, i_extra: I::ExtraData, o_extra: O::ExtraData) -> NodeChildren<I, O> {
        NodeChildren {
            inputs: I::from_node_generator(
                {
                    let mut i = 0;
                    let nodes = &mut *nodes;
                    move || {
                        let nk = nodes.insert_with_key(|nk| Node {
                            logic: logic::NodeLogic::new(),
                            parent: NodeParent {
                                kind: match parent_type {
                                    NodeParentType::Gate(gk) => NodeParentKind::GateIn(gk, i),
                                    NodeParentType::Circuit(ck) => NodeParentKind::CircuitIn(ck, i),
                                },
                            },
                            connections: connections::NodeConnections::new(),
                        });
                        i += 1;
                        nk
                    }
                },
                i_extra,
            ),
            outputs: O::from_node_generator(
                {
                    let mut i = 0;
                    let nodes = &mut *nodes;
                    move || {
                        let nk = nodes.insert_with_key(|nk| Node {
                            logic: logic::NodeLogic::new(),
                            parent: NodeParent {
                                kind: match parent_type {
                                    NodeParentType::Gate(gk) => NodeParentKind::GateOut(gk, i),
                                    NodeParentType::Circuit(ck) => NodeParentKind::CircuitOut(ck, i),
                                },
                            },
                            connections: connections::NodeConnections::new(),
                        });
                        i += 1;
                        nk
                    }
                },
                o_extra,
            ),
        }
    }

    pub(crate) fn inputs(&self) -> &I {
        &self.inputs
    }

    pub(crate) fn outputs(&self) -> &O {
        &self.outputs
    }
}

impl private::NodeVec for Vec<NodeKey> {
    type ExtraData = usize;

    fn from_node_generator(next: impl FnMut() -> NodeKey, size: usize) -> Self {
        std::iter::repeat_with(next).take(size).collect()
    }
}
impl<const N: usize> private::NodeVec for [NodeKey; N] {
    type ExtraData = ();

    fn from_node_generator(mut next: impl FnMut() -> NodeKey, (): ()) -> Self {
        std::array::from_fn(|_| next())
    }
}
