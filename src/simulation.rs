pub(crate) mod hierarchy;
pub(crate) mod logic;
pub(crate) mod ui;

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

    pub(crate) toplevel_gates: hierarchy::GateChildren,
}

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: hierarchy::GateChildren,
    pub(crate) nodes: hierarchy::NodeChildren<Vec<NodeKey>, Vec<NodeKey>>,
    pub(crate) widget: ui::GateWidget,
}

pub(crate) struct Node {
    pub(crate) logic: logic::NodeLogic,
    pub(crate) parent: hierarchy::NodeParent,
    pub(crate) widget: ui::NodeWidget,
}

pub(crate) enum Gate {
    Nand { logic: logic::NandLogic, widget: ui::GateWidget },
    Const { logic: logic::ConstLogic, widget: ui::GateWidget },
    Unerror { logic: logic::UnerrorLogic, widget: ui::GateWidget },
    Custom(CircuitKey),
}

impl Circuit {
    pub(crate) fn new(circuit_key: CircuitKey, nodes: &mut NodeMap, name: String, num_inputs: usize, num_outputs: usize) -> Circuit {
        Circuit {
            name,
            gates: hierarchy::GateChildren::new(),
            nodes: hierarchy::NodeChildren::new(nodes, hierarchy::NodeParentType::Circuit(circuit_key), num_inputs, num_outputs),
            widget: ui::GateWidget::new(),
        }
    }
}
impl Gate {
    fn name<'s: 'r, 'c: 'r, 'r>(&'s self, circuits: &'c CircuitMap) -> &'r str {
        match self {
            Gate::Nand { logic, widget: _ } => logic.name(),
            Gate::Const { logic, widget: _ } => logic.name(),
            Gate::Unerror { logic, widget: _ } => logic.name(),
            Gate::Custom(sck) => &circuits[*sck].name,
        }
    }

    pub(crate) fn inputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
        match &gates[gate] {
            Gate::Nand { logic, widget: _ } => logic.nodes.inputs(),
            Gate::Const { logic, widget: _ } => logic.nodes.inputs(),
            Gate::Custom(circuit_idx) => circuits[*circuit_idx].nodes.inputs(),
            Gate::Unerror { logic, widget: _ } => logic.nodes.inputs(),
        }
    }
    pub(crate) fn outputs<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r [NodeKey] {
        match &gates[gate] {
            Gate::Nand { logic, widget: _ } => logic.nodes.outputs(),
            Gate::Const { logic, widget: _ } => logic.nodes.outputs(),
            Gate::Custom(circuit_idx) => circuits[*circuit_idx].nodes.outputs(),
            Gate::Unerror { logic, widget: _ } => logic.nodes.outputs(),
        }
    }

    pub(crate) fn gate_num_inputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
        Gate::inputs(circuits, gates, gate).len()
    }
    pub(crate) fn gate_num_outputs(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> usize {
        Gate::outputs(circuits, gates, gate).len()
    }

    pub(crate) fn widget<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r ui::GateWidget {
        match &gates[gate] {
            Gate::Nand { logic: _, widget } | Gate::Const { logic: _, widget } | Gate::Unerror { logic: _, widget } => widget,
            Gate::Custom(sck) => &circuits[*sck].widget,
        }
    }
    /* TODO: remove?
    pub(crate) fn widget_mut<'c: 'r, 'g: 'r, 'r>(circuits: &'c CircuitMap, gates: &'g GateMap, gate: GateKey) -> &'r mut widget::GateLocation {
        match &gates[gate] {
            Gate::Nand { logic: _, widget } | Gate::Const { logic: _, widget } | Gate::Unerror { logic: _, widget } => widget,
            Gate::Custom(sck) => &mut circuits[*sck].widget,
        }
    }
    */
}
