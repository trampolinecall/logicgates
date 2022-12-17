use piston::{self, RenderEvent, UpdateEvent};

pub struct App {
    gl: opengl_graphics::GlGraphics, // OpenGL drawing backend.
    circuit: logicgates::circuit::Circuit,
    locations: Vec<[f64; 2]>,
}

impl App {
    fn render(&mut self, args: &piston::RenderArgs) {
        use graphics::*;

        const VERTICAL_VALUE_SPACING: f64 = 20.0;
        const CIRCLE_RAD: f64 = 5.0;
        const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
        const GATE_WIDTH: f64 = 50.0;

        let gate_box = |gate_index: usize| {
            let gate: &logicgates::circuit::Gate = &self.circuit.gates[gate_index];
            let [gate_x, gate_y]: [f64; 2] = self.locations[gate_index];
            let gate_height = (std::cmp::max(gate.num_inputs(), gate.num_outputs()) - 1 + 2) as f64 * VERTICAL_VALUE_SPACING;
            [gate_x, gate_y, gate_height]
        };

        let centered_y = |center_y, num_args, i| {
            let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
            let args_start_y = center_y - (args_height / 2.0);
            args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
        };
        let circuit_input_pos = |index: usize| [0.0, centered_y(args.window_size[1] / 2.0, self.circuit.arity, index)];
        let gate_input_pos = |gate_index: usize, input_index: usize| {
            let gate: &logicgates::circuit::Gate = &self.circuit.gates[gate_index];
            let [gate_x, gate_y, gate_height] = gate_box(gate_index);
            [gate_x, centered_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_index)]
        };
        let circuit_output_pos = |index| [args.window_size[0], centered_y(args.window_size[1] / 2.0, self.circuit.output.len(), index)];
        let gate_output_pos = |gate_index: usize, output_index: usize| {
            let gate: &logicgates::circuit::Gate = &self.circuit.gates[gate_index];
            let [gate_x, gate_y, gate_height] = gate_box(gate_index);
            [gate_x + GATE_WIDTH, centered_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_index)]
        };

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const GREY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);

            for i in 0..self.circuit.arity {
                let pos = circuit_input_pos(i);
                ellipse(BLACK, ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
            }
            for (output_i, output) in self.circuit.output.iter().enumerate() {
                let output_pos = circuit_output_pos(output_i);
                ellipse(BLACK, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

                let connection_start_pos = match output {
                    logicgates::circuit::Value::Arg(index) => circuit_input_pos(*index),
                    logicgates::circuit::Value::GateValue(gate_index, arg_index) => gate_output_pos(*gate_index, *arg_index),
                };

                line_from_to(BLACK, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }

            for (gate_i, gate) in self.circuit.gates.iter().enumerate() {
                let [gate_x, gate_y, gate_height] = gate_box(gate_i);

                rectangle(GREY, [gate_x, gate_y, GATE_WIDTH, gate_height], c.transform, gl);

                for (input_i, input) in gate.inputs().into_iter().enumerate() {
                    let input_pos@[x, y] = gate_input_pos(gate_i, input_i);
                    ellipse(BLACK, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                    let connection_start_pos = match input {
                        logicgates::circuit::Value::Arg(index) => circuit_input_pos(index),
                        logicgates::circuit::Value::GateValue(gate_index, arg_index) => gate_output_pos(gate_index, arg_index),
                    };

                    line_from_to(BLACK, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
                for i in 0..gate.num_outputs() {
                    let [x, y] = gate_output_pos(gate_i, i);
                    ellipse(BLACK, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
                }
            }
        });
    }

    fn update(&mut self, _: &piston::UpdateArgs) {}
}

fn main() {
    let opengl = opengl_graphics::OpenGL::V3_2;

    let mut window: glutin_window::GlutinWindow = piston::WindowSettings::new("logic gates", [1280, 720]).graphics_api(opengl).resizable(true).samples(4).exit_on_esc(true).build().unwrap();

    let mut app = App {
        gl: opengl_graphics::GlGraphics::new(opengl),
        circuit: logicgates::circuit::Circuit {
            arity: 2,
            gates: vec![logicgates::circuit::Gate::And(logicgates::circuit::Value::Arg(0), logicgates::circuit::Value::Arg(1))],
            output: vec![logicgates::circuit::Value::GateValue(0, 0)],
        },
        locations: vec![[60.0, 60.0]],
    };

    let mut events = piston::Events::new(piston::EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}
