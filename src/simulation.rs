pub(crate) mod draw;
pub(crate) mod hierarchy;
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
    pub(crate) gates: hierarchy::GateChildren,
    pub(crate) nodes: hierarchy::NodeChildren<Vec<NodeKey>, Vec<NodeKey>>,
    pub(crate) location: location::GateLocation,
}

pub(crate) struct Node {
    pub(crate) logic: logic::NodeLogic,
    pub(crate) parent: hierarchy::NodeParent,
}

pub(crate) enum Gate {
    Nand { logic: logic::NandLogic, location: location::GateLocation },
    Const { logic: logic::ConstLogic, location: location::GateLocation },
    Unerror { logic: logic::UnerrorLogic, location: location::GateLocation },
    Custom(CircuitKey),
}

impl Circuit {
    pub(crate) fn new(circuit_key: CircuitKey, nodes: &mut NodeMap, name: String, num_inputs: usize, num_outputs: usize) -> Circuit {
        Circuit {
            name,
            gates: hierarchy::GateChildren::new(),
            nodes: hierarchy::NodeChildren::new(nodes, hierarchy::NodeParentType::Circuit(circuit_key), num_inputs, num_outputs),
            location: location::GateLocation::new(),
        }
    }
}
impl Gate {
    pub(crate) fn location<'s: 'r, 'c: 'r, 'r>(&'s self, circuits: &'c CircuitMap) -> &'r location::GateLocation {
        match self {
            Gate::Nand { logic: _, location } | Gate::Const { logic: _, location } => location,
            Gate::Custom(sck) => &circuits[*sck].location,
            Gate::Unerror { logic: _, location } => location,
        }
    }
    pub(crate) fn location_mut<'s: 'r, 'c: 'r, 'r>(&'s mut self, circuits: &'c mut CircuitMap) -> &'r mut location::GateLocation {
        match self {
            Gate::Nand { logic: _, location } | Gate::Const { logic: _, location } => location,
            Gate::Unerror { logic: _, location } => location,
            Gate::Custom(sck) => &mut circuits[*sck].location,
        }
    }

    fn name<'s: 'r, 'c: 'r, 'r>(&'s self, circuits: &'c slotmap::SlotMap<CircuitKey, Circuit>) -> &'r str {
        match self {
            Gate::Nand { logic, location: _ } => logic.name(),
            Gate::Const { logic, location: _ } => logic.name(),
            Gate::Unerror { logic, location: _ } => logic.name(),
            Gate::Custom(sck) => &circuits[*sck].name,
        }
    }
}

pub(crate) fn gate_inputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate] {
        Gate::Nand { logic, location: _ } => logic.nodes.inputs(),
        Gate::Const { logic, location: _ } => logic.nodes.inputs(),
        Gate::Custom(circuit_idx) => circuits[*circuit_idx].nodes.inputs(),
        Gate::Unerror { logic, location: _ } => logic.nodes.inputs(),
    }
}
pub(crate) fn gate_outputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate] {
        Gate::Nand { logic, location: _ } => logic.nodes.outputs(),
        Gate::Const { logic, location: _ } => logic.nodes.outputs(),
        Gate::Custom(circuit_idx) => circuits[*circuit_idx].nodes.outputs(),
        Gate::Unerror { logic, location: _ } => logic.nodes.outputs(),
    }
}

pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_outputs(circuits, gates, gate).len()
}
