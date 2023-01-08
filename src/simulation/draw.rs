use generational_arena::Arena;

use crate::simulation::{
    logic::{self, GateInputNodeIdx, GateOutputNodeIdx, NodeIdx},
    {Circuit, CircuitIndex, Gate},
};

use super::{
    logic::{CircuitInputNodeIdx, CircuitOutputNodeIdx},
    GateIndex,
};

const CIRCLE_RAD: f64 = 5.0;
const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f64 = 20.0;
const HORIZONTAL_GATE_SPACING: f64 = 100.0;

const BG: [f32; 4] = [0.172, 0.243, 0.313, 1.0];
const GATE_COLOR: [f32; 4] = [0.584, 0.647, 0.65, 1.0];
const ON_COLOR: [f32; 4] = [0.18, 0.8, 0.521, 1.0];
const OFF_COLOR: [f32; 4] = [0.498, 0.549, 0.552, 1.0];

pub(crate) fn render(circuits: &Arena<Circuit>, gates: &Arena<Gate>, circuit: CircuitIndex, graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs) {
    use graphics::*;

    let circuit = &circuits[circuit];
    graphics.draw(args.viewport(), |c, gl| {
        clear(BG, gl);

        // draw circuit inputs and outputs
        for input_node_index in logic::circuit_input_indexes(circuit) {
            let pos = circuit_input_pos(circuits, args, input_node_index);
            ellipse(node_color(circuits, gates, input_node_index.into()), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
        }
        for output_node_index in logic::circuit_output_indexes(circuit) {
            let output_pos = circuit_output_pos(circuits, args, output_node_index);
            let color = node_color(circuits, gates, output_node_index.into());
            ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

            // draw lines connecting outputs with their values
            if let Some(producer) = logic::get_node(circuits, gates, output_node_index.into()).producer() {
                let connection_start_pos = node_pos(circuits, gates, args, producer);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }
        }

        // draw each gate
        for gate_i in circuit.gates.iter() {
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(circuits, gates, args, *gate_i);

            rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

            // draw gate input dots and connections to their values
            for input_receiver_index in logic::gate_input_indexes(circuits, gates, *gate_i) {
                let color = node_color(circuits, gates, input_receiver_index.into());
                let input_pos @ [x, y] = gate_input_pos(circuits, gates, args, input_receiver_index);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                if let Some(producer) = logic::get_node(circuits, gates, input_receiver_index.into()).producer() {
                    let connection_start_pos = node_pos(circuits, gates, args, producer);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
            }
            // draw gate output dots
            for output in logic::gate_output_indexes(circuits, gates, *gate_i) {
                let color = node_color(circuits, gates, output.into());
                let [x, y] = gate_output_pos(circuits, gates, args, output);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
            }
        }
    });
}

pub(crate) fn gate_display_size(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: GateIndex) -> [f64; 2] {
    const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;
    const GATE_WIDTH: f64 = 50.0;

    let gate_height = (std::cmp::max(logic::gate_num_inputs(circuits, gates, gate), logic::gate_num_outputs(circuits, gates, gate)) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    [GATE_WIDTH, gate_height]
}

fn centered_arg_y(center_y: f64, num_args: usize, i: usize) -> f64 {
    let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
    let args_start_y = center_y - (args_height / 2.0);
    args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
}

fn gate_box(circuits: &Arena<Circuit>, gates: &Arena<Gate>, args: &piston::RenderArgs, gate_index: GateIndex) -> [f64; 4] {
    let gate = &gates[gate_index];
    let (gate_x, gate_y) = gate.location;
    let [gate_width, gate_height] = gate_display_size(circuits, gates, gate_index);
    [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
}

fn circuit_input_pos(circuits: &Arena<Circuit>, args: &piston::RenderArgs, index: CircuitInputNodeIdx) -> [f64; 2] {
    let circuit = &circuits[index.0];
    [0.0, centered_arg_y(args.window_size[1] / 2.0, circuit.inputs.len(), index.1)]
}
fn circuit_output_pos(circuits: &Arena<Circuit>, args: &piston::RenderArgs, index: CircuitOutputNodeIdx) -> [f64; 2] {
    let circuit = &circuits[index.0];
    [args.window_size[0], centered_arg_y(args.window_size[1] / 2.0, circuit.outputs.len(), index.1)]
}

// TODO: merge these into node_pos()?
fn gate_input_pos(circuits: &Arena<Circuit>, gates: &Arena<Gate>, args: &piston::RenderArgs, input_idx: GateInputNodeIdx) -> [f64; 2] {
    let gate_index = input_idx.0;
    let [gate_x, gate_y, _, gate_height] = gate_box(circuits, gates, args, gate_index);
    [gate_x, centered_arg_y(gate_y + gate_height / 2.0, logic::gate_num_inputs(circuits, gates, gate_index), input_idx.1)]
}
fn gate_output_pos(circuits: &Arena<Circuit>, gates: &Arena<Gate>, args: &piston::RenderArgs, output_idx: GateOutputNodeIdx) -> [f64; 2] {
    let gate_index = output_idx.0;
    let [gate_x, gate_y, gate_width, gate_height] = gate_box(circuits, gates, args, gate_index);
    [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, logic::gate_num_outputs(circuits, gates, gate_index), output_idx.1)]
}

fn node_pos(circuits: &Arena<Circuit>, gates: &Arena<Gate>, args: &piston::RenderArgs, node: NodeIdx) -> [f64; 2] {
    match node {
        NodeIdx::CI(ci) => circuit_input_pos(circuits, args, ci),
        NodeIdx::CO(co) => circuit_output_pos(circuits, args, co),
        NodeIdx::GI(gi) => gate_input_pos(circuits, gates, args, gi),
        NodeIdx::GO(go) => gate_output_pos(circuits, gates, args, go),
    }
}
/* (unused)
fn receiver_node_pos(circuit: &Circuit, args: &piston::RenderArgs, node: ReceiverIdx) -> [f64; 2] {
    match node {
        ReceiverIdx::CO(co) => circuit_output_pos(circuit, args, co.0),
        ReceiverIdx::GI(gi) => gate_input_pos(circuit, args, gi),
    }
}
*/
fn bool_color(value: bool) -> [f32; 4] {
    if value {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
fn node_color(circuits: &Arena<Circuit>, gates: &Arena<Gate>, node: NodeIdx) -> [f32; 4] {
    bool_color(logic::get_node_value(circuits, gates, node))
}
