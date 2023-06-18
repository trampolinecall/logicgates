mod connection;
mod gate;
mod node;

use std::{collections::HashMap, marker::PhantomData};

use nannou::prelude::*;

use crate::{
    simulation::{self, hierarchy, location, Gate, GateKey, NodeKey, Simulation},
    LogicGates, view::Drawing,
};

const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;
const GATE_EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
const GATE_WIDTH: f32 = 50.0;

const BG_COLOR: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };

pub(crate) struct SimulationDrawing {
    pub(crate) gates: Vec<gate::GateDrawing>,
    pub(crate) nodes: Vec<node::NodeDrawing>,
    pub(crate) connections: Vec<connection::ConnectionDrawing>,
    pub(crate) rect: nannou::geom::Rect,
}
impl SimulationDrawing {
    pub(crate) fn new(rect: nannou::geom::Rect, simulation: &Simulation) -> SimulationDrawing {
        let toplevel_gates = &simulation.toplevel_gates; // TODO: ability to switch between viewing toplevel and circuit

        let gates = toplevel_gates.iter().copied();
        let nodes =
            toplevel_gates.iter().flat_map(|gate| Gate::inputs(&simulation.circuits, &simulation.gates, *gate).iter().chain(Gate::outputs(&simulation.circuits, &simulation.gates, *gate))).copied();

        let (gate_widgets, node_widgets, connection_widgets) = layout(&simulation.circuits, &simulation.gates, &simulation.nodes, &simulation.connections, gates, nodes, rect);

        SimulationDrawing { gates: gate_widgets, nodes: node_widgets, connections: connection_widgets, rect }
    }
}

impl Drawing for SimulationDrawing {
    // TODO: figure out a more elegant way to draw the simulation because this relies on the simulation being passed to be the same simulation that this widget belongs to
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        draw.rect().xy(self.rect.xy()).wh(self.rect.wh()).color(BG_COLOR);

        for connection in &self.connections {
            connection.draw(simulation, draw, hovered);
        }
        for gate in &self.gates {
            gate.draw(simulation, draw, hovered);
        }
        for node in &self.nodes {
            node.draw(simulation, draw, hovered);
        }
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        // reverse to go in z order from highest to lowest
        for node in self.nodes.iter().rev() {
            if let hover @ Some(_) = node.find_hover(mouse_pos) {
                return hover;
            }
        }
        for gate in self.gates.iter().rev() {
            if let hover @ Some(_) = gate.find_hover(mouse_pos) {
                return hover;
            }
        }
        for connection in self.connections.iter().rev() {
            if let hover @ Some(_) = connection.find_hover(mouse_pos) {
                return hover;
            }
        }
        if self.rect.contains(mouse_pos) {
            return Some(self);
        }

        None
    }

    fn left_mouse_down(&self) -> Option<crate::Message> {
        None
    }
}

fn layout(
    circuit_map: &simulation::CircuitMap,
    gate_map: &simulation::GateMap,
    node_map: &simulation::NodeMap,
    connections: &simulation::connections::Connections,
    gates: impl IntoIterator<Item = GateKey>,
    nodes: impl IntoIterator<Item = NodeKey>,
    rect: nannou::geom::Rect,
) -> (Vec<gate::GateDrawing>, Vec<node::NodeDrawing>, Vec<connection::ConnectionDrawing>) {
    let gate_widgets = gates
        .into_iter()
        .map(|gate| {
            let gate_location = Gate::location(circuit_map, gate_map, gate);
            let num_inputs = Gate::num_inputs(circuit_map, gate_map, gate);
            let num_outputs = Gate::num_outputs(circuit_map, gate_map, gate);

            gate::GateDrawing { key: gate, rect: gate_rect(rect, gate_location, num_inputs, num_outputs) }
        })
        .collect();
    let node_positions: HashMap<_, _> = nodes.into_iter().map(|node| (node, node_pos(rect, circuit_map, gate_map, node_map, node))).collect();
    let connections: Vec<_> = connections.iter().filter_map(|(a, b)| Some(connection::ConnectionDrawing { node1: *a, node2: *b, pos1: *node_positions.get(a)?, pos2: *node_positions.get(b)? })).collect();
    let node_widgets = node_positions.into_iter().map(|(node, location)| node::NodeDrawing { key: node, location }).collect();

    (gate_widgets, node_widgets, connections)
}

// TODO: reorganize all of these functions
fn gate_rect(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize) -> Rect {
    // TODO: gate_location should eventually be the center
    let (x, y) = (gate_location.x, gate_location.y);
    let wh = gate_display_size(num_inputs, num_outputs);
    Rect::from_x_y_w_h(window_rect.x() + x, window_rect.y() + y, wh.x, wh.y)
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
