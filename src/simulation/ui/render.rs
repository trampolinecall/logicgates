use std::{collections::BTreeSet, marker::PhantomData};

use nannou::prelude::*;

use crate::simulation::{
    self, hierarchy, logic,
    ui::{location, DrawShape},
    CircuitKey, CircuitMap, Gate, GateMap, NodeKey, NodeMap, Simulation,
};

// TODO: figure out a better place to put all of these constants
pub(super) const CIRCLE_RAD: f32 = 5.0;
pub(super) const CONNECTION_RAD: f32 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;

const BG: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };
pub(super) const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };
const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };
const HIGH_IMPEDANCE_COLOR: Rgb = Rgb { red: 52.0 / 255.0, green: 152.0 / 255.0, blue: 219.0 / 255.0, standard: PhantomData };
const ERR_COLOR: Rgb = Rgb { red: 231.0 / 255.0, green: 76.0 / 255.0, blue: 60.0 / 255.0, standard: PhantomData };

// TODO: alternate coordinate system

pub(crate) fn render(app: &App, draw: &Draw, simulation: &Simulation) {
    draw.background().color(BG);
    all_widgets(app, simulation).for_each(|wid| wid.render(draw))
}

fn all_widgets<'a>(app: &'a App, simulation: &'a Simulation) -> impl Iterator<Item = DrawShape> + 'a {
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
}

pub(super) fn gate_rect(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize) -> Rect {
    // TODO: gate_location should eventually be the center
    let (x, y) = (gate_location.x, gate_location.y);
    let wh = gate_display_size(num_inputs, num_outputs);
    Rect::from_x_y_w_h(x as f32 * HORIZONTAL_GATE_SPACING - window_rect.x.len() / 2.0 + wh.x / 2.0, y + wh.y / 2.0, wh.x, wh.y)
}

pub(crate) fn gate_display_size(num_inputs: usize, num_outputs: usize) -> Vec2 {
    const EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
    const GATE_WIDTH: f32 = 50.0;

    let gate_height = (std::cmp::max(num_inputs, num_outputs) - 1) as f32 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    pt2(GATE_WIDTH, gate_height)
}

fn y_centered_around(center_y: f32, total: usize, index: usize) -> f32 {
    let box_height: f32 = ((total - 1) as f32) * VERTICAL_VALUE_SPACING;
    let box_start_y = center_y + (box_height / 2.0);
    box_start_y - (index as f32) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(window_rect: Rect, simulation: &Simulation, circuit: CircuitKey, index: usize) -> Vec2 {
    let circuit = &simulation.circuits[circuit];
    pt2(window_rect.x.start, y_centered_around(0.0, circuit.nodes.inputs().len(), index))
}
fn circuit_output_pos(window_rect: Rect, simulation: &Simulation, circuit: CircuitKey, index: usize) -> Vec2 {
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

pub(super) fn node_pos(window_rect: Rect, circuits: &CircuitMap, gates: &GateMap, nodes: &NodeMap, node: NodeKey) -> Vec2 {
    match nodes[node].parent.kind() {
        // hierarchy::NodeParentKind::CircuitIn(c, i) if c == simulation.main_circuit => circuit_input_pos(window_rect, simulation, c, i), TODO: switching between different views
        // hierarchy::NodeParentKind::CircuitOut(c, i) if c == simulation.main_circuit => circuit_output_pos(window_rect, simulation, c, i), TODO: switching between different views
        hierarchy::NodeParentKind::CircuitIn(c, i) => {
            let circuit = &circuits[c];
            let location = &circuit.widget.location;
            let num_inputs = circuit.nodes.inputs().len();
            let num_outputs = circuit.nodes.outputs().len();
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::CircuitOut(c, i) => {
            let circuit = &circuits[c];
            let location = &circuit.widget.location;
            let num_inputs = circuit.nodes.inputs().len();
            let num_outputs = circuit.nodes.outputs().len();
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::GateIn(g, i) => {
            let location = &simulation::Gate::widget(circuits, gates, g).location;
            let num_inputs = simulation::Gate::num_inputs(circuits, gates, g);
            let num_outputs = simulation::Gate::num_outputs(circuits, gates, g);
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        hierarchy::NodeParentKind::GateOut(g, i) => {
            let location = &simulation::Gate::widget(circuits, gates, g).location;
            let num_inputs = simulation::Gate::num_inputs(circuits, gates, g);
            let num_outputs = simulation::Gate::num_outputs(circuits, gates, g);
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
    }
}

pub(super) fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb {
    fn value_to_color(v: logic::Value) -> Rgb {
        match v {
            logic::Value::H => ON_COLOR,
            logic::Value::L => OFF_COLOR,
            logic::Value::Z => HIGH_IMPEDANCE_COLOR,
            logic::Value::X => ERR_COLOR,
        }
    }
    if use_production {
        if let Some(v) = logic::get_node_production(nodes, node) {
            value_to_color(v)
        } else {
            value_to_color(logic::get_node_value(nodes, node))
        }
    } else {
        value_to_color(logic::get_node_value(nodes, node))
    }
}
