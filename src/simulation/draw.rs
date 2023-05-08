use std::{collections::BTreeSet, marker::PhantomData};

use nannou::prelude::*;

use crate::simulation::{self, location, logic, CircuitKey, NodeKey, NodeParent, Simulation};

const CIRCLE_RAD: f32 = 5.0;
const CONNECTION_RAD: f32 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;

const BG: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };
const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };
const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };
const HIGH_IMPEDANCE_COLOR: Rgb = Rgb { red: 52.0 / 255.0, green: 152.0 / 255.0, blue: 219.0 / 255.0, standard: PhantomData };
const ERR_COLOR: Rgb = Rgb { red: 231.0 / 255.0, green: 76.0 / 255.0, blue: 60.0 / 255.0, standard: PhantomData };

pub(crate) fn render(app: &App, draw: &Draw, simulation: &Simulation, main_circuit: CircuitKey) {
    let main_circuit = &simulation.circuits[main_circuit];
    draw.background().color(BG);

    let window_rect = app.window_rect();

    let (custom_gates_in_current, gates_in_current) = {
        let mut custom_gates_in_current = BTreeSet::new();
        let mut gates_in_current = BTreeSet::new();
        for gate in &main_circuit.gates {
            match &simulation.gates[*gate] {
                simulation::Gate::Nand { logic: _, location: _ } | simulation::Gate::Const { logic: _, location: _ } | simulation::Gate::Unerror { logic: _, location: _ } => {
                    gates_in_current.insert(gate)
                }
                simulation::Gate::Custom(subck) => custom_gates_in_current.insert(subck),
            };
        }
        (custom_gates_in_current, gates_in_current)
    };

    let circuit_inputs = main_circuit.inputs.iter();
    let circuit_outputs = main_circuit.outputs.iter();
    let gate_inputs = main_circuit.gates.iter().flat_map(|gk| simulation::gate_inputs(&simulation.circuits, &simulation.gates, *gk));
    let gate_outputs = main_circuit.gates.iter().flat_map(|gk| simulation::gate_outputs(&simulation.circuits, &simulation.gates, *gk));
    let all_nodes = circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs);

    // draw connections first
    // this draws every connection twice because this is an undirected graph
    for &cur_node in all_nodes.clone() {
        // cur node is guaranteed to be part of the current circuit and not a node in a subcircuit
        for adjacent in simulation.nodes[cur_node].logic.adjacent() {
            let adjacent_in_current_circuit = match simulation.nodes[*adjacent].parent {
                NodeParent::GateIn(gk, _) | NodeParent::GateOut(gk, _) => gates_in_current.contains(&gk),
                NodeParent::CircuitIn(ck, _) | NodeParent::CircuitOut(ck, _) if ck == simulation.main_circuit => true,
                NodeParent::CircuitIn(ck, _) | NodeParent::CircuitOut(ck, _) => custom_gates_in_current.contains(&ck),
            };

            if adjacent_in_current_circuit {
                let color = node_color(simulation, cur_node, false);
                let cur_pos = node_pos(window_rect, simulation, cur_node);
                let adj_pos = node_pos(window_rect, simulation, *adjacent);
                draw.line().start(adj_pos).end(cur_pos).color(color).weight(CONNECTION_RAD);
            }
        }
    }

    // draw gate rectangles
    for &gate_k in &main_circuit.gates {
        let gate = &simulation.gates[gate_k];
        let location = gate.location(&simulation.circuits);
        let num_inputs = simulation::gate_num_inputs(&simulation.circuits, &simulation.gates, gate_k);
        let num_outputs = simulation::gate_num_outputs(&simulation.circuits, &simulation.gates, gate_k);
        let rect = gate_rect(window_rect, location, num_inputs, num_outputs);
        draw.rect().color(GATE_COLOR).xy(rect.xy()).wh(rect.wh());
        draw.text(simulation.gates[gate_k].name(&simulation.circuits)).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y();
    }

    // draw nodes
    for &node_key in all_nodes {
        let pos = node_pos(window_rect, simulation, node_key);
        let color = node_color(simulation, node_key, true);
        draw.ellipse().color(color).x_y(pos[0], pos[1]).radius(CIRCLE_RAD);
    }
}

fn gate_rect(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize) -> Rect {
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
    pt2(window_rect.x.start, y_centered_around(0.0, circuit.inputs.len(), index))
}
fn circuit_output_pos(window_rect: Rect, simulation: &Simulation, circuit: CircuitKey, index: usize) -> Vec2 {
    let circuit = &simulation.circuits[circuit];
    pt2(window_rect.x.end, y_centered_around(0.0, circuit.outputs.len(), index))
}

fn gate_input_pos(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.left(), y_centered_around(rect.y(), num_inputs, idx))
}
fn gate_output_pos(window_rect: Rect, gate_location: &location::GateLocation, num_inputs: usize, num_outputs: usize, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, gate_location, num_inputs, num_outputs);
    pt2(rect.right(), y_centered_around(rect.y(), num_outputs, idx))
}

fn node_pos(window_rect: Rect, simulation: &Simulation, node: NodeKey) -> Vec2 {
    match simulation.nodes[node].parent {
        NodeParent::CircuitIn(c, i) if c == simulation.main_circuit => circuit_input_pos(window_rect, simulation, c, i),
        NodeParent::CircuitOut(c, i) if c == simulation.main_circuit => circuit_output_pos(window_rect, simulation, c, i),

        NodeParent::CircuitIn(c, i) => {
            let circuit = &simulation.circuits[c];
            let location = &circuit.location;
            let num_inputs = circuit.inputs.len();
            let num_outputs = circuit.outputs.len();
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        NodeParent::CircuitOut(c, i) => {
            let circuit = &simulation.circuits[c];
            let location = &circuit.location;
            let num_inputs = circuit.inputs.len();
            let num_outputs = circuit.outputs.len();
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        NodeParent::GateIn(g, i) => {
            let gate = &simulation.gates[g];
            let location = gate.location(&simulation.circuits);
            let num_inputs = simulation::gate_num_inputs(&simulation.circuits, &simulation.gates, g);
            let num_outputs = simulation::gate_num_outputs(&simulation.circuits, &simulation.gates, g);
            gate_input_pos(window_rect, location, num_inputs, num_outputs, i)
        }
        NodeParent::GateOut(g, i) => {
            let gate = &simulation.gates[g];
            let location = gate.location(&simulation.circuits);
            let num_inputs = simulation::gate_num_inputs(&simulation.circuits, &simulation.gates, g);
            let num_outputs = simulation::gate_num_outputs(&simulation.circuits, &simulation.gates, g);
            gate_output_pos(window_rect, location, num_inputs, num_outputs, i)
        }
    }
}

fn node_color(simulation: &Simulation, node: NodeKey, use_production: bool) -> Rgb {
    fn value_to_color(v: logic::Value) -> Rgb {
        match v {
            logic::Value::H => ON_COLOR,
            logic::Value::L => OFF_COLOR,
            logic::Value::Z => HIGH_IMPEDANCE_COLOR,
            logic::Value::X => ERR_COLOR,
        }
    }
    if use_production {
        if let Some(v) = logic::get_node_production(&simulation.nodes, node) {
            value_to_color(v)
        } else {
            value_to_color(logic::get_node_value(&simulation.nodes, node))
        }
    } else {
        value_to_color(logic::get_node_value(&simulation.nodes, node))
    }
}
