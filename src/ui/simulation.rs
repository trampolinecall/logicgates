use std::marker::PhantomData;

use nannou::prelude::*;

use crate::{
    simulation::{self, hierarchy, location, Gate, GateKey, NodeKey, Simulation},
    ui::Widget,
    ui::{gate::GateWidget, node::NodeWidget},
};

const CONNECTION_RAD: f32 = super::node::CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;
const GATE_EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
const GATE_WIDTH: f32 = 50.0;

const BG_COLOR: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };

pub(crate) struct SimulationWidget {
    pub(crate) gates: Vec<GateWidget>,
    pub(crate) nodes: Vec<NodeWidget>,
    pub(crate) rect: nannou::geom::Rect,
}
impl SimulationWidget {
    pub(crate) fn new(rect: nannou::geom::Rect, simulation: &Simulation) -> SimulationWidget {
        let toplevel_gates = &simulation.toplevel_gates; // TODO: ability to switch between viewing toplevel and circuit

        let gates = toplevel_gates.iter().copied();
        let nodes =
            toplevel_gates.iter().flat_map(|gate| Gate::inputs(&simulation.circuits, &simulation.gates, *gate).iter().chain(Gate::outputs(&simulation.circuits, &simulation.gates, *gate))).copied();

        let (/* connection_positions, TODO */ gate_widgets, node_widgets) = layout(&simulation.circuits, &simulation.gates, &simulation.nodes, gates, nodes, rect);

        SimulationWidget { gates: gate_widgets, nodes: node_widgets, rect }
    }
}

impl Widget for SimulationWidget {
    // TODO: figure out a more elegant way to draw the simulation because this relies on the simulation being passed to be the same simulation that this widget belongs to
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw) {
        /*
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

        draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(BG_COLOR);
        /* TODO
        for (connection, position) in connection_positions {
            todo!()
        }
        */
        for gate in &self.gates {
            gate.draw(simulation, draw);
        }
        for node in &self.nodes {
            node.draw(simulation, draw);
        }
    }
}

fn layout(
    circuit_map: &simulation::CircuitMap,
    gate_map: &simulation::GateMap,
    node_map: &simulation::NodeMap,
    gates: impl IntoIterator<Item = GateKey>,
    nodes: impl IntoIterator<Item = NodeKey>,
    rect: nannou::geom::Rect,
) -> (Vec<GateWidget>, Vec<NodeWidget>) {
    let gate_positions = gates
        .into_iter()
        .map(|gate| {
            let gate_location = Gate::location(circuit_map, gate_map, gate);
            let num_inputs = Gate::num_inputs(circuit_map, gate_map, gate);
            let num_outputs = Gate::num_outputs(circuit_map, gate_map, gate);

            GateWidget { key: gate, rect: gate_rect(rect, gate_location, num_inputs, num_outputs) }
        })
        .collect();
    let node_positions = nodes.into_iter().map(|node| NodeWidget { key: node, location: node_pos(rect, circuit_map, gate_map, node_map, node) }).collect();
    (gate_positions, node_positions)
}

// TODO: reorganize all of these functions
fn gate_rect(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize) -> Rect {
    // TODO: gate_location should eventually be the center
    let (x, y) = (gate_location.x, gate_location.y);
    let wh = gate_display_size(num_inputs, num_outputs);
    Rect::from_x_y_w_h(window_rect.left() + x as f32 * HORIZONTAL_GATE_SPACING + wh.x / 2.0, window_rect.y() + y + wh.y / 2.0, wh.x, wh.y)
}

pub(crate) fn gate_display_size(num_inputs: usize, num_outputs: usize) -> Vec2 {
    let gate_height = (std::cmp::max(num_inputs, num_outputs) - 1) as f32 * VERTICAL_VALUE_SPACING + GATE_EXTRA_VERTICAL_HEIGHT;
    pt2(GATE_WIDTH, gate_height)
}

fn y_centered_around(center_y: f32, total: usize, index: usize) -> f32 {
    let box_height: f32 = ((total - 1) as f32) * VERTICAL_VALUE_SPACING;
    let box_start_y = center_y + (box_height / 2.0);
    box_start_y - (index as f32) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(window_rect: Rect, simulation: &Simulation, circuit: simulation::CircuitKey, index: usize) -> Vec2 {
    let circuit = &simulation.circuits[circuit];
    pt2(window_rect.x.start, y_centered_around(0.0, circuit.nodes.inputs().len(), index))
}
fn circuit_output_pos(window_rect: Rect, simulation: &Simulation, circuit: simulation::CircuitKey, index: usize) -> Vec2 {
    let circuit = &simulation.circuits[circuit];
    pt2(window_rect.x.end, y_centered_around(0.0, circuit.nodes.outputs().len(), index))
}

fn gate_input_pos(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.left(), y_centered_around(rect.y(), num_inputs, idx))
}
fn gate_output_pos(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.right(), y_centered_around(rect.y(), num_outputs, idx))
}

fn node_pos(window_rect: Rect, circuits: &simulation::CircuitMap, gates: &simulation::GateMap, nodes: &simulation::NodeMap, node: NodeKey) -> Vec2 {
    match nodes[node].parent.kind() {
        // hierarchy::NodeParentKind::CircuitIn(c, i) if c == simulation.main_circuit => circuit_input_pos(window_rect, simulation, c, i), TODO: switching between different views
        // hierarchy::NodeParentKind::CircuitOut(c, i) if c == simulation.main_circuit => circuit_output_pos(window_rect, simulation, c, i), TODO: switching between different views
        hierarchy::NodeParentKind::CircuitIn(c, i) => {
            let circuit = &circuits[c];
            let location = &circuit.location;
            let num_inputs = circuit.nodes.inputs().len();
            let num_outputs = circuit.nodes.outputs().len();
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::CircuitOut(c, i) => {
            let circuit = &circuits[c];
            let location = &circuit.location;
            let num_inputs = circuit.nodes.inputs().len();
            let num_outputs = circuit.nodes.outputs().len();
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::GateIn(g, i) => {
            let location = &simulation::Gate::location(circuits, gates, g);
            let num_inputs = simulation::Gate::num_inputs(circuits, gates, g);
            let num_outputs = simulation::Gate::num_outputs(circuits, gates, g);
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::GateOut(g, i) => {
            let location = &simulation::Gate::location(circuits, gates, g);
            let num_inputs = simulation::Gate::num_inputs(circuits, gates, g);
            let num_outputs = simulation::Gate::num_outputs(circuits, gates, g);
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
    }
}
