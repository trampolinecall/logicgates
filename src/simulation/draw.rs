use generational_arena::Arena;

use crate::simulation::{
    circuit::{Circuit, CircuitIndex, Gate},
    connections::{self, GateInputNodeIdx, GateOutputNodeIdx, NodeIdx},
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
        for (input_i, input_node_index) in connections::circuit_input_indexes(circuit).enumerate() {
            let pos = circuit_input_pos(circuit, args, input_i);
            ellipse(node_color(circuits, gates, input_node_index.into()), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
        }
        for (output_i, output) in connections::circuit_output_indexes(circuit).enumerate() {
            let output_pos = circuit_output_pos(circuit, args, output_i);
            let color = node_color(circuits, gates, output.into());
            ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

            // draw lines connecting outputs with their values
            if let Some(producer) = connections::get_node(circuits, gates, output.into()).producer() {
                let connection_start_pos = node_pos(circuit, gates, args, producer);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }
        }

        // draw each gate
        for gate in circuit.gates.iter() {
            let gate = &gates[*gate];
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(args, gate);

            rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

            // draw gate input dots and connections to their values
            for input_receiver in connections::gate_inputs(gate) {
                let color = node_color(circuits, gates, input_receiver.into());
                let input_pos @ [x, y] = gate_input_pos(gates, args, input_receiver);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                if let Some(producer) = connections::get_node(circuits, gates, input_receiver.into()).producer() {
                    let connection_start_pos = node_pos(circuit, gates, args, producer);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
            }
            // draw gate output dots
            for output in connections::gate_outputs(gate) {
                let color = node_color(circuits, gates, output.into());
                let [x, y] = gate_output_pos(gates, args, output);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
            }
        }
    });
}

pub(crate) fn gate_display_size(gate: &Gate) -> [f64; 2] {
    const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;
    const GATE_WIDTH: f64 = 50.0;

    let gate_height = (std::cmp::max(gate.num_inputs(), gate.num_outputs()) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    [GATE_WIDTH, gate_height]
}

fn centered_arg_y(center_y: f64, num_args: usize, i: usize) -> f64 {
    let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
    let args_start_y = center_y - (args_height / 2.0);
    args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
}

fn gate_box(args: &piston::RenderArgs, gate: &Gate) -> [f64; 4] {
    let (gate_x, gate_y) = gate.location;
    let [gate_width, gate_height] = gate_display_size(gate);
    [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
}

fn circuit_input_pos(circuit: &Circuit, args: &piston::RenderArgs, index: usize) -> [f64; 2] {
    [0.0, centered_arg_y(args.window_size[1] / 2.0, circuit.inputs.len(), index)]
}
fn circuit_output_pos(circuit: &Circuit, args: &piston::RenderArgs, index: usize) -> [f64; 2] {
    [args.window_size[0], centered_arg_y(args.window_size[1] / 2.0, circuit.outputs.len(), index)]
}

fn gate_input_pos(gates: &Arena<Gate>, args: &piston::RenderArgs, input_idx: GateInputNodeIdx) -> [f64; 2] {
    let gate = &gates[input_idx.0];
    let [gate_x, gate_y, _, gate_height] = gate_box(args, gate);
    [gate_x, centered_arg_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_idx.1)]
}
fn gate_output_pos(gates: &Arena<Gate>, args: &piston::RenderArgs, output_idx: GateOutputNodeIdx) -> [f64; 2] {
    let gate = &gates[output_idx.0];
    let [gate_x, gate_y, gate_width, gate_height] = gate_box(args, gate);
    [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_idx.1)]
}

fn node_pos(circuit: &Circuit, gates: &Arena<Gate>, args: &piston::RenderArgs, node: NodeIdx) -> [f64; 2] {
    match node {
        NodeIdx::CI(ci) => circuit_input_pos(circuit, args, ci.1),
        NodeIdx::CO(co) => circuit_output_pos(circuit, args, co.1),
        NodeIdx::GI(gi) => gate_input_pos(gates, args, gi),
        NodeIdx::GO(go) => gate_output_pos(gates, args, go),
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
    bool_color(connections::get_node_value(circuits, gates, node))
}
