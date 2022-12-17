use crate::{circuit, eval};

// TODO: refactor everything
pub fn render(graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs, circuit: &circuit::Circuit, locations: &[[f64; 2]], inputs: &[bool]) {
    use graphics::*;
    let (_, evaluated_values) = eval::eval_with_results(circuit, inputs);

    const VERTICAL_VALUE_SPACING: f64 = 20.0;
    const CIRCLE_RAD: f64 = 5.0;
    const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
    const GATE_WIDTH: f64 = 50.0;

    const BG: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    const GATE_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
    const ON_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
    const OFF_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

    let gate_box = |gate_index: usize| {
        let gate: &circuit::Gate = &circuit.gates[gate_index];
        let [gate_x, gate_y]: [f64; 2] = locations[gate_index];
        let gate_height = (std::cmp::max(gate.num_inputs(), gate.num_outputs()) - 1 + 2) as f64 * VERTICAL_VALUE_SPACING;
        [gate_x, gate_y, gate_height]
    };

    let centered_y = |center_y, num_args, i| {
        let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
        let args_start_y = center_y - (args_height / 2.0);
        args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
    };
    let circuit_input_pos = |index: usize| [0.0, centered_y(args.window_size[1] / 2.0, circuit.arity, index)];
    let gate_input_pos = |gate_index: usize, input_index: usize| {
        let gate: &circuit::Gate = &circuit.gates[gate_index];
        let [gate_x, gate_y, gate_height] = gate_box(gate_index);
        [gate_x, centered_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_index)]
    };
    let circuit_output_pos = |index| [args.window_size[0], centered_y(args.window_size[1] / 2.0, circuit.output.len(), index)];
    let gate_output_pos = |gate_index: usize, output_index: usize| {
        let gate: &circuit::Gate = &circuit.gates[gate_index];
        let [gate_x, gate_y, gate_height] = gate_box(gate_index);
        [gate_x + GATE_WIDTH, centered_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_index)]
    };

    let value_pos = |value: circuit::Value| match value {
        circuit::Value::Arg(index) => circuit_input_pos(index),
        circuit::Value::GateValue(gate_index, arg_index) => gate_output_pos(gate_index, arg_index),
    };

    let on_off_color = |value| if value { ON_COLOR } else { OFF_COLOR };
    let get_value = |value| match value {
        circuit::Value::Arg(arg_index) => inputs[arg_index],
        circuit::Value::GateValue(gate_index, output_index) => evaluated_values[gate_index][output_index],
    };

    graphics.draw(args.viewport(), |c, gl| {
        clear(BG, gl);

        // draw circuit inputs and outputs
        for (i, input_value) in (0..circuit.arity).zip(inputs.iter()) {
            let pos = circuit_input_pos(i);
            ellipse(on_off_color(*input_value), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
        }
        for (output_i, output) in circuit.output.iter().enumerate() {
            let output_pos = circuit_output_pos(output_i);
            let color = on_off_color(get_value(*output));
            ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

            // draw lines connecting outputs with their values
            let connection_start_pos = value_pos(*output);
            line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
        }

        // draw each gate
        for (gate_i, gate) in circuit.gates.iter().enumerate() {
            let [gate_x, gate_y, gate_height] = gate_box(gate_i);

            rectangle(GATE_COLOR, [gate_x, gate_y, GATE_WIDTH, gate_height], c.transform, gl);
            // TODO: draw gate name
            // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

            // draw gate input dots and connections to their values
            for (input_i, input) in gate.inputs().into_iter().enumerate() {
                let color = on_off_color(get_value(input));
                let input_pos @ [x, y] = gate_input_pos(gate_i, input_i);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                let connection_start_pos = value_pos(input);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
            }
            // draw gate output dots
            for output_i in 0..gate.num_outputs() {
                let color = on_off_color(evaluated_values[gate_i][output_i]);
                let [x, y] = gate_output_pos(gate_i, output_i);
                ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
            }
        }
    });
}
