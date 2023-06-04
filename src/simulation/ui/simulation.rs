use crate::simulation::{self, ui::Widget, GateKey, Simulation};

pub(crate) struct SimulationWidget {}

impl SimulationWidget {
    pub(crate) fn new() -> SimulationWidget {
        SimulationWidget {}
    }
}

impl Widget for SimulationWidget {
    // TODO: figure out a more elegant way to draw the simulation because this relies on the simulation being passed to be the same simulation that this widget belongs to
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw, rect: nannou::geom::Rect) {
        /*
        let toplevel_gates = &simulation.toplevel_gates; // TODO: ability to switch between viewing toplevel and circuit

        let window_rect = app.window_rect(); // TODO: probably remove this when the different coordinate system is implemented

        let circuit_inputs = std::iter::empty(); // main_circuit.nodes.inputs().iter(); // TODO: put back when adding switching between different views
        let circuit_outputs = std::iter::empty(); // main_circuit.nodes.outputs().iter();
        let gate_inputs = toplevel_gates.iter().flat_map(|gk| Gate::inputs(&simulation.circuits, &simulation.gates, *gk));
        let gate_outputs = toplevel_gates.iter().flat_map(|gk| Gate::outputs(&simulation.circuits, &simulation.gates, *gk));

        let all_nodes = circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs);

        let (custom_gates_in_current, gates_in_current) = {
            let mut custom_gates_in_current = BTreeSet::new();
            let mut gates_in_current = BTreeSet::new();
            for gate in toplevel_gates {
                match &simulation.gates[*gate] {
                    simulation::Gate::Nand { logic: _, widget: _ } | simulation::Gate::Const { logic: _, widget: _ } | simulation::Gate::Unerror { logic: _, widget: _ } => gates_in_current.insert(gate),
                    simulation::Gate::Custom(subck) => custom_gates_in_current.insert(subck),
                };
            }
            (custom_gates_in_current, gates_in_current)
        };
        let connection_widgets = all_nodes
            .clone()
            .flat_map({
                |&cur_node| {
                    // cur node is guaranteed to be part of the current circuit and not a node in a subcircuit because it taken only from things known to be in this circuit
                    simulation.nodes[cur_node].logic.adjacent().iter().map(move |adj| (cur_node, adj))
                }
            })
            .filter_map(move |(cur_node, adj_node)| {
                let adjacent_in_current_circuit = match simulation.nodes[*adj_node].parent.get_node_parent_type() {
                    hierarchy::NodeParentType::Gate(gk) => gates_in_current.contains(&gk),
                    // hierarchy::NodeParentType::Circuit(ck) if ck == simulation.main_circuit => true, TODO: switching between different views
                    hierarchy::NodeParentType::Circuit(ck) => custom_gates_in_current.contains(&ck),
                };

                if adjacent_in_current_circuit {
                    let color = node_color(&simulation.nodes, cur_node, false);
                    let cur_pos = node_pos(window_rect, &simulation.circuits, &simulation.gates, &simulation.nodes, cur_node);
                    let adj_pos = node_pos(window_rect, &simulation.circuits, &simulation.gates, &simulation.nodes, *adj_node);
                    Some(DrawShape::new_for_connection(cur_pos, adj_pos, color))
                } else {
                    None
                }
            });

        let gate_widgets = toplevel_gates.into_iter().map(move |&gate_k| DrawShape::new_for_gate(window_rect, &simulation.circuits, &simulation.gates, gate_k));
        let node_widgets = all_nodes.map(move |&node_key| DrawShape::new_for_node(window_rect, &simulation.circuits, &simulation.gates, &simulation.nodes, node_key));

        connection_widgets.chain(gate_widgets).chain(node_widgets)
        */

        let gate_positions = layout(&simulation.toplevel_gates);
        for (gate, position) in gate_positions {
            todo!()
        }
    }
}

fn layout(gates: &simulation::hierarchy::GateChildren) -> Vec<(GateKey, nannou::geom::Rect)> {
    todo!()
}
