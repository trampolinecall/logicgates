use std::marker::PhantomData;

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

pub(crate) fn render(app: &App, draw: &Draw, simulation: &Simulation, main_circuit: CircuitKey) {
    let main_circuit = &simulation.circuits[main_circuit];
    draw.background().color(BG);

    let window_rect = app.window_rect();

    // draw connections first
    // dont go through circuit inputs (the ones on the left edge of the screen) because those should not be drawn connected to anything
    // dont go through gate output indexes (the ones on the right edge of gates) because those are usually conteccted to some internal gates not part of the main circuit
    let circuit_outputs = main_circuit.outputs.iter();
    let gate_inputs = main_circuit.gates.iter().flat_map(|gk| simulation::gate_inputs(&simulation.circuits, &simulation.gates, *gk));
    for &cur_node in circuit_outputs.chain(gate_inputs) {
        if let Some(producer) = simulation.nodes[cur_node].value.producer() {
            let color = node_color(simulation, cur_node);
            let cur_pos = node_pos(window_rect, simulation, cur_node);
            let producer_pos = node_pos(window_rect, simulation, producer);
            draw.line().start(producer_pos).end(cur_pos).color(color).weight(CONNECTION_RAD);
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
    let circuit_inputs = main_circuit.inputs.iter();
    let circuit_outputs = main_circuit.outputs.iter();
    let gate_inputs = main_circuit.gates.iter().flat_map(|gk| simulation::gate_inputs(&simulation.circuits, &simulation.gates, *gk));
    let gate_outputs = main_circuit.gates.iter().flat_map(|gk| simulation::gate_outputs(&simulation.circuits, &simulation.gates, *gk));
    for &node_key in circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs) {
        let pos = node_pos(window_rect, simulation, node_key);
        let color = node_color(simulation, node_key);
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

fn node_color(simulation: &Simulation, node: NodeKey) -> Rgb {
    if logic::get_node_value(&simulation.nodes, node) {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
