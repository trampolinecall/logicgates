use crate::simulation::{
    circuit::{Circuit, Gate},
    connections::{self, GateInputNodeIdx, GateOutputNodeIdx, ProducerIdx, ReceiverIdx},
};

const CIRCLE_RAD: f64 = 5.0;
const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
const VERTICAL_VALUE_SPACING: f64 = 20.0;
const HORIZONTAL_GATE_SPACING: f64 = 100.0;

const BG: [f32; 4] = [0.172, 0.243, 0.313, 1.0];
const GATE_COLOR: [f32; 4] = [0.584, 0.647, 0.65, 1.0];
const ON_COLOR: [f32; 4] = [0.18, 0.8, 0.521, 1.0];
const OFF_COLOR: [f32; 4] = [0.498, 0.549, 0.552, 1.0];

pub(crate) fn render(circuit: &Circuit, graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs) {
    use graphics::*;

    graphics.draw(args.viewport(), |c, gl| {
        clear(BG, gl);

        // draw circuit inputs and outputs
        for (input_i, input_producer) in circuit.inputs.iter().enumerate() {
            let pos = circuit_input_pos(circuit, args, input_i);
            ellipse(bool_color(input_producer.value), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
        }
        for (output_i, output) in connections::circuit_output_indexes(circuit).enumerate() {
            let output_pos = circuit_output_pos(circuit, args, output_i);
            let color = receiver_color(circuit, output.into());
            ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

            // draw lines connecting outputs with their values
            if let Some(producer) = connections::get_receiver(circuit, output.into()).producer(){
                let connection_start_pos = producer_pos(circuit, args, producer);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }
        }

        // draw each gate
        for (_, gate) in circuit.gates.iter() {
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(args, gate);

            rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

            // draw gate input dots and connections to their values
            for input_receiver in connections::gate_inputs(gate) {
                let color = receiver_color(circuit, input_receiver.into());
                let input_pos @ [x, y] = gate_input_pos(circuit, args, input_receiver);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                if let Some(producer) = connections::get_receiver(circuit, input_receiver.into()).producer(){
                    let connection_start_pos = producer_pos(circuit, args, producer);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
            }
            // draw gate output dots
            for output in connections::gate_outputs(gate) {
                let color = producer_color(circuit, output.into());
                let [x, y] = gate_output_pos(circuit, args, output);
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

fn gate_input_pos(circuit: &Circuit, args: &piston::RenderArgs, input_idx: GateInputNodeIdx) -> [f64; 2] {
    let gate = &circuit.gates[input_idx.0];
    let [gate_x, gate_y, _, gate_height] = gate_box(args, gate);
    [gate_x, centered_arg_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_idx.1)]
}
fn gate_output_pos(circuit: &Circuit, args: &piston::RenderArgs, output_idx: GateOutputNodeIdx) -> [f64; 2] {
    let gate = &circuit.gates[output_idx.0];
    let [gate_x, gate_y, gate_width, gate_height] = gate_box(args, gate);
    [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_idx.1)]
}

fn producer_pos(circuit: &Circuit, args: &piston::RenderArgs, node: ProducerIdx) -> [f64; 2] {
    match node {
        ProducerIdx::CI(ci) => circuit_input_pos(circuit, args, ci.0),
        ProducerIdx::GO(go) => gate_output_pos(circuit, args, go),
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
fn producer_color(circuit: &Circuit, producer: ProducerIdx) -> [f32; 4] {
    bool_color(connections::get_producer(circuit, producer).value)
}
fn receiver_color(circuit: &Circuit, receiver: ReceiverIdx) -> [f32; 4] {
    bool_color(if let Some(producer) = connections::get_receiver(circuit, receiver).producer(){ connections::get_producer(circuit, producer).value } else { false })
}
