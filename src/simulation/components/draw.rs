use generational_arena::Arena;

use crate::simulation::{Gate, GateIndex};

use super::connection;

const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;
const GATE_WIDTH: f64 = 50.0;

const VERTICAL_VALUE_SPACING: f64 = 20.0;
const NODE_RAD: f64 = 5.0;
const CONNECTION_RAD: f64 = NODE_RAD / 2.0;
const HORIZONTAL_GATE_SPACING: f64 = 100.0;

const BG: [f32; 4] = [0.172, 0.243, 0.313, 1.0];
const GATE_COLOR: [f32; 4] = [0.584, 0.647, 0.65, 1.0];
const ON_COLOR: [f32; 4] = [0.18, 0.8, 0.521, 1.0];
const OFF_COLOR: [f32; 4] = [0.498, 0.549, 0.552, 1.0];

// TODO: double check that this can handle gates being deleted without crashing

pub(crate) struct DrawComponent {
    // TODO: separate location into locationcomponent, draw is something else
    pub(crate) position: (i32, f64),
}

pub(crate) fn render(gates: &Arena<Gate>, main_circuit: GateIndex, graphics: &mut opengl_graphics::GlGraphics, render_args: &piston::RenderArgs) {
    let main_circuit = match gates.get(main_circuit) {
        Some(Gate { calculation, .. }) => calculation.as_custom().expect("main circuit should be custom gate"),
        None => panic!("main circuit is deleted"),
    };

    use graphics::*;
    graphics.draw(render_args.viewport(), |c, gl| {
        clear(BG, gl);

        // draw circuit inputs and outputs
        for (input_i, input_producer) in main_circuit.inputs.iter().enumerate() {
            let pos = circuit_input_pos(render_args, input_i, main_circuit.inputs.len());
            ellipse(bool_color(input_producer.value), ellipse::circle(pos[0], pos[1], NODE_RAD), c.transform, gl);
        }
        for (output_i, output) in main_circuit.outputs.iter().enumerate() {
            let output_pos = circuit_output_pos(render_args, output_i, main_circuit.outputs.len());
            let color = node_color(gates, output.into());
            ellipse(color, ellipse::circle(output_pos[0], output_pos[1], NODE_RAD), c.transform, gl);

            // draw lines connecting outputs with their values
            if let Some(producer) = output.producer() {
                let connection_start_pos = node_pos(render_args, gates, producer);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }
        }

        // draw each gate
        for gate in &main_circuit.gates {
            let gate = &gates[*gate];
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(render_args, gate);

            rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

            // draw gate input dots and connections to their values
            for input_receiver in gate.calculation.inputs() {
                let color = node_color(gates, connection::get_node(gates, input_receiver).unwrap());
                let input_pos @ [x, y] = node_pos(render_args, gates, input_receiver);
                ellipse(color, ellipse::circle(x, y, NODE_RAD), c.transform, gl);

                /* TODO
                if let Some(producer) = input_receiver.producer() {
                    let connection_start_pos = node_pos(render_args, gates, producer);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
                */
            }

            // draw gate output dots
            for output in gate.calculation.outputs() {
                let color = node_color(gates, connection::get_node(gates, output).unwrap());
                let [x, y] = node_pos(render_args, gates, output);
                ellipse(color, ellipse::circle(x, y, NODE_RAD), c.transform, gl);
            }
        }
    });
}

pub(crate) fn centered_arg_y(center_y: f64, num_args: usize, i: usize) -> f64 {
    let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
    let args_start_y = center_y - (args_height / 2.0);
    args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
}

fn circuit_input_pos(args: &piston::RenderArgs, index: usize, num: usize) -> [f64; 2] {
    [0.0, centered_arg_y(args.window_size[1] / 2.0, num, index)]
}
fn circuit_output_pos(args: &piston::RenderArgs, index: usize, num: usize) -> [f64; 2] {
    [args.window_size[0], centered_arg_y(args.window_size[1] / 2.0, num, index)]
}

fn gate_box(args: &piston::RenderArgs, gate: &Gate) -> [f64; 4] {
    let (gate_x, gate_y) = gate.draw.position;
    let [gate_width, gate_height] = display_size(gate);
    [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
}
fn node_pos(render_args: &piston::RenderArgs, gates: &Arena<Gate>, idx: connection::NodeIdx) -> [f64; 2] {
    let gate = &gates[idx.gate];
    let [gate_x, gate_y, gate_width, gate_height] = gate_box(render_args, gate);
    if idx.outputs {
        [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, gate.calculation.num_outputs(), idx.index)]
    } else {
        [gate_x, centered_arg_y(gate_y + gate_height / 2.0, gate.calculation.num_inputs(), idx.index)]
    }
}

pub(crate) fn display_size(gate: &Gate) -> [f64; 2] {
    // TODO: display component
    let gate_height = (std::cmp::max(gate.calculation.num_inputs(), gate.calculation.num_outputs()) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
    [GATE_WIDTH, gate_height]
}

/*
fn producer_pos(node: ProducerIdx) {
    match node {
        ProducerIdx::CI(ci) => circuit_input_pos(ci.0),
        ProducerIdx::GO(go) => gate_output_pos(go),
    }
}
fn receiver_node_pos(node: ReceiverIdx) { match node {
    ReceiverIdx::CO(co) => circuit_output_pos(co.0),
    ReceiverIdx::GI(gi) => gate_input_pos(gi),
}
*/

fn bool_color(value: bool) -> [f32; 4] {
    if value {
        ON_COLOR
    } else {
        OFF_COLOR
    }
}
fn node_color(gates: &Arena<Gate>, node: connection::Node) -> [f32; 4] {
    match node {
        connection::Node::Producer(p) => bool_color(p.value),
        connection::Node::Receiver(r) => {
            if let Some(producer) = r.producer() {
                if let Some(producer) = connection::get_node(gates, producer) {
                    node_color(gates, producer)
                } else {
                    OFF_COLOR
                }
            } else {
                OFF_COLOR
            }
        } // TODO: move this logic into connection
    }
}
/*
fn producer_color(producer: ProducerIdx) -> [f32; 4] {
    bool_color(self.get_producer(producer).value)
}
fn receiver_color(receiver: ReceiverIdx) -> [f32; 4] {
    bool_color(if let Some(producer) = self.get_receiver(receiver).producer { self.get_producer(producer).value } else { false })
}
*/
