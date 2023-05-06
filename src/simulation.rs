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
    pub(crate) location: location::GateLocation,
    inputs: Vec<NodeKey>,
    outputs: Vec<NodeKey>,
}

#[derive(Copy, Clone)]
pub(crate) enum NodeParent {
    GateIn(GateKey, usize),
    GateOut(GateKey, usize),
    CircuitIn(CircuitKey, usize),
    CircuitOut(CircuitKey, usize),
}
pub(crate) struct Node {
    pub(crate) logic: logic::NodeLogic,
    pub(crate) parent: NodeParent, // TODO: move into NodeLocation component?
}

pub(crate) enum Gate {
    Nand { logic: logic::NandLogic, location: location::GateLocation },
    Const { logic: logic::ConstLogic, location: location::GateLocation },
    Custom(CircuitKey),
}

impl Circuit {
    pub(crate) fn new(circuit_key: CircuitKey, nodes: &mut NodeMap, name: String, num_inputs: usize, num_outputs: usize) -> Circuit {
        // even if this circuit is part of a subcircuit, these nodes dont have to update anything
        // instead, because the gates inside this circuit are connected to these nodes, updates to these nodes will propagate to the gates' nodes, properly updating those gates
        Circuit {
            name,
            gates: Vec::new(),
            inputs: (0..num_inputs).map(|i| nodes.insert(Node { logic: logic::NodeLogic::new(), parent: NodeParent::CircuitIn(circuit_key, i) })).collect(),
            outputs: (0..num_outputs).map(|i| nodes.insert(Node { logic: logic::NodeLogic::new(), parent: NodeParent::CircuitOut(circuit_key, i) })).collect(),
            location: location::GateLocation::new(),
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
impl Gate {
    pub(crate) fn location<'s: 'r, 'c: 'r, 'r>(&'s self, circuits: &'c CircuitMap) -> &'r location::GateLocation {
        match self {
            Gate::Nand { logic: _, location } | Gate::Const { logic: _, location } => location,
            Gate::Custom(sck) => &circuits[*sck].location,
        }
    }
    pub(crate) fn location_mut<'s: 'r, 'c: 'r, 'r>(&'s mut self, circuits: &'c mut CircuitMap) -> &'r mut location::GateLocation {
        match self {
            Gate::Nand { logic: _, location } | Gate::Const { logic: _, location } => location,
            Gate::Custom(sck) => &mut circuits[*sck].location,
        }
    }

    fn name<'s: 'r, 'c: 'r, 'r>(&'s self, circuits: &'c slotmap::SlotMap<CircuitKey, Circuit>) -> &'r str {
        match self {
            Gate::Nand { logic, location: _ } => logic.name(),
            Gate::Const { logic, location: _ } => logic.name(),
            Gate::Custom(sck) => &circuits[*sck].name,
        }
    }
}

pub(crate) fn gate_inputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate] {
        Gate::Nand { logic, location: _ } => &logic.inputs,
        Gate::Const { logic, location: _ } => &logic.inputs,
        Gate::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate] {
        Gate::Nand { logic, location: _ } => &logic.outputs,
        Gate::Const { logic, location: _ } => &logic.outputs,
        Gate::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_outputs(circuits, gates, gate).len()
}
