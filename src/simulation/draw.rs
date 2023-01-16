use std::marker::PhantomData;

use nannou::prelude::*;

use crate::simulation::{logic, CircuitKey, CircuitMap, GateKey, GateMap};

// TODO: consider whether to merge draw and location components

const CIRCLE_RAD: f32 = 5.0;
const CONNECTION_RAD: f32 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f32 = 20.0;
const HORIZONTAL_GATE_SPACING: f32 = 100.0;

const BG: Rgb = Rgb { red: 0.172, green: 0.243, blue: 0.313, standard: PhantomData };
const GATE_COLOR: Rgb = Rgb { red: 0.584, green: 0.647, blue: 0.65, standard: PhantomData };
const ON_COLOR: Rgb = Rgb { red: 0.18, green: 0.8, blue: 0.521, standard: PhantomData };
const OFF_COLOR: Rgb = Rgb { red: 0.498, green: 0.549, blue: 0.552, standard: PhantomData };

pub(crate) fn render(app: &App, draw: &Draw, circuits: &CircuitMap, gates: &GateMap, main_circuit: CircuitKey) {
    let main_circuit = &circuits[main_circuit];
    draw.background().color(BG);

    let window_rect = app.window_rect();

    // draw connections first
    // dont go through circuit inputs (the ones on the left edge of the screen) because those should not be drawn connected to anything
    // dont go through gate output indexes (the ones on the right edge of gates) because those are usually conteccted to some internal gates not part of the main circuit
    let circuit_outputs = logic::circuit_output_indexes(main_circuit).map(Into::into);
    let gate_inputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_input_indexes(circuits, gates, *gi).map(Into::into));
    for node_idx in circuit_outputs.chain(gate_inputs) {
        if let Some(producer) = logic::get_node(circuits, gates, node_idx).producer() {
            let color = node_color(circuits, gates, node_idx);
            let cur_pos = node_pos(window_rect, circuits, gates, node_idx);
            let producer_pos = node_pos(window_rect, circuits, gates, producer);
            draw.line().start(producer_pos).end(cur_pos).color(color).weight(CONNECTION_RAD);
        }
    }

    // draw gate rectangles
    for gate_i in &main_circuit.gates {
        let rect = gate_rect(window_rect, circuits, gates, *gate_i);
        draw.rect().color(GATE_COLOR).xy(rect.xy()).wh(rect.wh());
        draw.text(gates[*gate_i].calculation.name(circuits)).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y();
    }

    // draw nodes
    let circuit_inputs = logic::circuit_input_indexes(main_circuit).map(Into::into);
    let circuit_outputs = logic::circuit_output_indexes(main_circuit).map(Into::into);
    let gate_inputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_input_indexes(circuits, gates, *gi).map(Into::into));
    let gate_outputs = main_circuit.gates.iter().flat_map(|gi| logic::gate_output_indexes(circuits, gates, *gi).map(Into::into));
    for node_idx in circuit_inputs.chain(circuit_outputs).chain(gate_inputs).chain(gate_outputs) {
        let pos = node_pos(window_rect, circuits, gates, node_idx);
        let color = node_color(circuits, gates, node_idx);
        draw.ellipse().color(color).x_y(pos[0], pos[1]).radius(CIRCLE_RAD);
    }
}

fn gate_rect(window_rect: Rect, circuits: &CircuitMap, gates: &GateMap, gate_index: GateKey) -> Rect {
    let (x, y) = gates[gate_index].location.location; // TODO: this should eventually be the center
    let wh = gate_display_size(circuits, gates, gate_index);
    Rect::from_x_y_w_h(x as f32 * HORIZONTAL_GATE_SPACING - window_rect.x.len() / 2.0 + wh.x / 2.0, y + wh.y / 2.0, wh.x, wh.y)
}

pub(crate) fn gate_display_size(circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> Vec2 {
    const EXTRA_VERTICAL_HEIGHT: f32 = 40.0;
    const GATE_WIDTH: f32 = 50.0;

    let gate_height = (std::cmp::max(logic::gate_num_inputs(circuits, gates, gate), logic::gate_num_outputs(circuits, gates, gate)) - 1) as f32 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    pt2(GATE_WIDTH, gate_height)
}

fn y_centered_around(center_y: f32, total: usize, index: usize) -> f32 {
    let window_rect_height: f32 = ((total - 1) as f32) * VERTICAL_VALUE_SPACING;
    let window_rect_start_y = center_y + (window_rect_height / 2.0);
    window_rect_start_y - (index as f32) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(window_rect: Rect, circuits: &CircuitMap, index: logic::CircuitInputNodeIdx) -> Vec2 {
    let circuit = &circuits[index.0];
    pt2(window_rect.x.start, y_centered_around(0.0, circuit.inputs.len(), index.1))
}
fn circuit_output_pos(window_rect: Rect, circuits: &CircuitMap, index: logic::CircuitOutputNodeIdx) -> Vec2 {
    let circuit = &circuits[index.0];
    pt2(window_rect.x.end, y_centered_around(0.0, circuit.outputs.len(), index.1))
}

fn gate_input_pos(window_rect: Rect, circuits: &CircuitMap, gates: &GateMap, input_idx: logic::GateInputNodeIdx) -> Vec2 {
    let gate_index = input_idx.0;
    let rect = gate_rect(window_rect, circuits, gates, gate_index);
    pt2(rect.left(), y_centered_around(rect.y(), logic::gate_num_inputs(circuits, gates, gate_index), input_idx.1))
}
fn gate_output_pos(window_rect: Rect, circuits: &CircuitMap, gates: &GateMap, output_idx: logic::GateOutputNodeIdx) -> Vec2 {
    let gate_index = output_idx.0;
    let rect = gate_rect(window_rect, circuits, gates, gate_index);
    pt2(rect.right(), y_centered_around(rect.y(), logic::gate_num_outputs(circuits, gates, gate_index), output_idx.1))
}

fn node_pos(window_rect: Rect, circuits: &CircuitMap, gates: &GateMap, node: logic::NodeIdx) -> Vec2 {
    match node {
        logic::NodeIdx::CI(ci) => circuit_input_pos(window_rect, circuits, ci),
        logic::NodeIdx::CO(co) => circuit_output_pos(window_rect, circuits, co),
        logic::NodeIdx::GI(gi) => gate_input_pos(window_rect, circuits, gates, gi),
        logic::NodeIdx::GO(go) => gate_output_pos(window_rect, circuits, gates, go),
    }
}

fn node_color(circuits: &CircuitMap, gates: &GateMap, node: logic::NodeIdx) -> Rgb {
    if logic::get_node_value(circuits, gates, node) {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
