use std::marker::PhantomData;

use nannou::prelude::*;

use crate::simulation::{logic, CircuitKey, GateKey, NodeKey, Simulation};

const CIRCLE_RAD: f32 = 5.0;
const CONNECTION_RAD: f32 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;

const BG: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };
const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };
const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };

enum NodePos {
    CI(CircuitKey, usize),
    CO(CircuitKey, usize),
    GI(GateKey, usize),
    GO(GateKey, usize),
}

pub(crate) fn render(app: &App, draw: &Draw, simulation: &Simulation, main_circuit: CircuitKey) {
    let main_circuit = &simulation.circuits[main_circuit];
    draw.background().color(BG);

    let window_rect = app.window_rect();

    // draw connections first
    // dont go through circuit inputs (the ones on the left edge of the screen) because those should not be drawn connected to anything
    // dont go through gate output indexes (the ones on the right edge of gates) because those are usually conteccted to some internal gates not part of the main circuit
    let circuit_outputs = main_circuit.outputs.iter().enumerate().map(|(i, k)| (NodePos::CO(simulation.main_circuit, i), *k));
    let gate_inputs = main_circuit.gates.iter().flat_map(|gk| logic::gate_inputs(&simulation.circuits, &simulation.gates, *gk).iter().enumerate().map(|(i, k)| (NodePos::GI(*gk, i), *k)));
    for (pos, node_idx) in circuit_outputs.chain(gate_inputs) {
        if let Some(producer) = simulation.nodes[node_idx].value.producer() {
            let color = node_color(simulation, node_idx);
            let cur_pos = node_pos(window_rect, simulation, pos);
            let producer_pos = Vec2::new(0.0, 0.0); // TODO: node_pos(window_rect, simulation, producer);
            draw.line().start(producer_pos).end(cur_pos).color(color).weight(CONNECTION_RAD);
        }
    }

    // draw gate rectangles
    for gate_i in &main_circuit.gates {
        let rect = gate_rect(window_rect, simulation, *gate_i);
        draw.rect().color(GATE_COLOR).xy(rect.xy()).wh(rect.wh());
        draw.text(simulation.gates[*gate_i].logic.name(&simulation.circuits)).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y();
    }

    // draw nodes
    let circuit_inputs = main_circuit.inputs.iter().enumerate().map(|(i, k)| (NodePos::CI(simulation.main_circuit, i), *k));
    let circuit_outputs = main_circuit.outputs.iter().enumerate().map(|(i, k)| (NodePos::CO(simulation.main_circuit, i), *k));
    let gate_inputs = main_circuit.gates.iter().flat_map(|gk| logic::gate_inputs(&simulation.circuits, &simulation.gates, *gk).iter().enumerate().map(|(i, k)| (NodePos::GI(*gk, i), *k)));
    let gate_outputs = main_circuit.gates.iter().flat_map(|gk| logic::gate_outputs(&simulation.circuits, &simulation.gates, *gk).iter().enumerate().map(|(i, k)| (NodePos::GO(*gk, i), *k)));
    for (pos, node_idx) in circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs) {
        let pos = node_pos(window_rect, simulation, pos);
        let color = node_color(simulation, node_idx);
        draw.ellipse().color(color).x_y(pos[0], pos[1]).radius(CIRCLE_RAD);
    }
}

fn gate_rect(window_rect: Rect, simulation: &Simulation, gate_index: GateKey) -> Rect {
    let location = &simulation.gates[gate_index].location; // TODO: this should eventually be the center
    let (x, y) = (location.x, location.y);
    let wh = gate_display_size(simulation, gate_index);
    Rect::from_x_y_w_h(x as f32 * HORIZONTAL_GATE_SPACING - window_rect.x.len() / 2.0 + wh.x / 2.0, y + wh.y / 2.0, wh.x, wh.y)
}

pub(crate) fn gate_display_size(simulation: &Simulation, gate: GateKey) -> Vec2 {
    const EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
    const GATE_WIDTH: f32 = 50.0;

    let gate_height = (std::cmp::max(logic::gate_num_inputs(&simulation.circuits, &simulation.gates, gate), logic::gate_num_outputs(&simulation.circuits, &simulation.gates, gate)) - 1) as f32
        * VERTICAL_VALUE_SPACING
        + EXTRA_VERTICAL_HEIGHT;
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

fn gate_input_pos(window_rect: Rect, simulation: &Simulation, gate: GateKey, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, simulation, gate);
    pt2(rect.left(), y_centered_around(rect.y(), logic::gate_num_inputs(&simulation.circuits, &simulation.gates, gate), idx))
}
fn gate_output_pos(window_rect: Rect, simulation: &Simulation, gate: GateKey, idx: usize) -> Vec2 {
    let rect = gate_rect(window_rect, simulation, gate);
    pt2(rect.right(), y_centered_around(rect.y(), logic::gate_num_outputs(&simulation.circuits, &simulation.gates, gate), idx))
}

fn node_pos(window_rect: Rect, simulation: &Simulation, node: NodePos) -> Vec2 {
    match node {
        NodePos::CI(c, i) => circuit_input_pos(window_rect, simulation, c, i),
        NodePos::CO(c, i) => circuit_output_pos(window_rect, simulation, c, i),
        NodePos::GI(g, i) => gate_input_pos(window_rect, simulation, g, i),
        NodePos::GO(g, i) => gate_output_pos(window_rect, simulation, g, i),
    }
}

fn node_color(simulation: &Simulation, node: NodeKey) -> Rgb {
    if logic::get_node_value(&simulation.nodes, node) {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
