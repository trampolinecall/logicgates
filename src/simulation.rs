pub(crate) mod draw;
pub(crate) mod location;
pub(crate) mod logic;

slotmap::new_key_type! {
    pub(crate) struct CircuitKey;
    pub(crate) struct GateKey;
    pub(crate) struct NodeKey;
}
pub(crate) type CircuitMap = slotmap::SlotMap<CircuitKey, Circuit>;
pub(crate) type GateMap = slotmap::SlotMap<GateKey, Gate>;
pub(crate) type NodeMap = slotmap::SlotMap<NodeKey, Node>;

pub(crate) struct Simulation {
    pub(crate) circuits: CircuitMap,
    pub(crate) gates: GateMap,
    pub(crate) nodes: NodeMap,

    pub(crate) main_circuit: CircuitKey,
}

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Vec<GateKey>,
    inputs: Vec<NodeKey>,
    outputs: Vec<NodeKey>,
}

#[derive(Copy, Clone)]
pub(crate) enum NodeParent {
    Gate(GateKey),
    Circuit(CircuitKey),
}
pub(crate) struct Node {
    pub(crate) value: logic::NodeLogic,
    pub(crate) parent: NodeParent, // TODO: move into NodeLocation component?
}

pub(crate) struct Gate {
    pub(crate) logic: logic::GateLogic,
    pub(crate) location: location::GateLocation,
}

impl Circuit {
    pub(crate) fn new(circuit_key: CircuitKey, nodes: &mut NodeMap, name: String, num_inputs: usize, num_outputs: usize) -> Circuit {
        // even if this circuit is part of a subcircuit, these nodes dont have to update anything
        // instead, because the gates inside this circuit are connected to these nodes, updates to these nodes will propagate to the gates' nodes, properly updating those gates
        Circuit {
            name,
            gates: Vec::new(),
            inputs: std::iter::repeat_with(|| nodes.insert(Node { value: logic::NodeLogic::new(), parent: NodeParent::Circuit(circuit_key) })).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| nodes.insert(Node { value: logic::NodeLogic::new(), parent: NodeParent::Circuit(circuit_key) })).take(num_outputs).collect(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self.inputs.len()
    }
    /* (unused, but may be used in the future)
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, connections::Node::new_disconnected(None));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, connections::Node::new_disconnected(None));
    }
    */

    pub(crate) fn inputs(&self) -> &[NodeKey] {
        self.inputs.as_ref()
    }

    pub(crate) fn outputs(&self) -> &[NodeKey] {
        self.outputs.as_ref()
    }
}
