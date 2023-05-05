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
    pub(crate) inputs: Vec<NodeKey>,
    pub(crate) outputs: Vec<NodeKey>,
}

pub(crate) struct Node {
    pub(crate) value: logic::NodeValue,
}

pub(crate) struct Gate {
    kind: GateKind,
    pub(crate) location: location::GateLocation,
}

enum GateKind {
    Nand([NodeKey; 2], [NodeKey; 1]),
    Const([NodeKey; 0], [NodeKey; 1]),
    Custom(CircuitKey),
}

impl Circuit {
    pub(crate) fn new(nodes: &mut NodeMap, name: String, num_inputs: usize, num_outputs: usize) -> Circuit {
        // even if this circuit is part of a subcircuit, these nodes dont have to update anything
        // instead, because the gates inside this circuit are connected to these nodes, updates to these nodes will propagate to the gates' nodes, properly updating those gates
        Circuit {
            name,
            gates: Vec::new(),
            inputs: std::iter::repeat_with(|| nodes.insert(Node { value: logic::NodeValue::new(false) })).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| nodes.insert(Node { value: logic::NodeValue::new(false) })).take(num_outputs).collect(),
        }
    }
}

impl Circuit {
    // TODO: where is this used?
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
}

impl Gate {
    fn name<'c>(&self, circuits: &'c CircuitMap) -> &'c str {
        match self.kind {
            GateKind::Nand(_, _) => "nand",
            GateKind::Const(_, _) => "const", // TODO: change name to "true" or "false"
            GateKind::Custom(ck) => &circuits[ck].name,
        }
    }

    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand(nodes: &mut NodeMap) -> Gate {
        Gate {
            kind: GateKind::Nand(
                [nodes.insert(Node { value: logic::NodeValue::new(false) }), nodes.insert(Node { value: logic::NodeValue::new(false) })],
                [nodes.insert(Node { value: logic::NodeValue::new(true) })],
            ),
            location: location::GateLocation::new(),
        }
    }
    pub(crate) fn new_const(nodes: &mut NodeMap, value: bool) -> Gate {
        Gate { kind: GateKind::Const([], [nodes.insert(Node { value: logic::NodeValue::new(value) })]), location: location::GateLocation::new() }
    }
    pub(crate) fn new_subcircuit(_: &mut NodeMap, subcircuit: CircuitKey) -> Gate {
        Gate { kind: GateKind::Custom(subcircuit), location: location::GateLocation::new() }
    }
}

pub(crate) fn gate_inputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate].kind {
        GateKind::Nand(i, _) => i,
        GateKind::Const(i, _) => i,
        GateKind::Custom(circuit_idx) => &circuits[*circuit_idx].inputs,
    }
}
pub(crate) fn gate_outputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
    match &gates[gate].kind {
        GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
        GateKind::Custom(circuit_idx) => &circuits[*circuit_idx].outputs,
    }
}

pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_inputs(circuits, gates, gate).len()
}
pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
    gate_outputs(circuits, gates, gate).len()
}
