use crate::circuit;

pub struct Simulation {
    pub circuit: circuit::Circuit,
    locations: Vec<[f64; 2]>,
}

const VERTICAL_VALUE_SPACING: f64 = 20.0;
const CIRCLE_RAD: f64 = 5.0;
const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
const GATE_WIDTH: f64 = 50.0;

const BG: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GATE_COLOR: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const ON_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const OFF_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

fn centered_y(center_y: f64, num_args: usize, i: usize) -> f64 {
    let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
    let args_start_y = center_y - (args_height / 2.0);
    args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
}

// TODO: refactor everything
impl Simulation {
    pub fn new(circuit: circuit::Circuit) -> Self {
        let mut s = Self { circuit, locations: Vec::new() };
        s.locations = s.calculate_locations();
        s
    }

    fn circuit_input_pos(&self, index: usize, window_size: [f64; 2]) -> [f64; 2] {
        [0.0, centered_y(window_size[1] / 2.0, self.circuit.num_inputs, index)]
    }
    fn circuit_output_pos(&self, index: usize, window_size: [f64; 2]) -> [f64; 2] {
        [window_size[0], centered_y(window_size[1] / 2.0, self.circuit.outputs.len(), index)]
    }

    fn gate_box(&self, gate_index: usize) -> [f64; 4] {
        let gate: &circuit::Gate = &self.circuit.gates[gate_index];
        let [gate_x, gate_y]: [f64; 2] = self.locations[gate_index]; // x is left but y in the locations represents center
        let gate_height = (std::cmp::max(gate.num_inputs(), gate.num_outputs()) - 1 + 2) as f64 * VERTICAL_VALUE_SPACING;
        [gate_x, gate_y - gate_height / 2.0, GATE_WIDTH, gate_height]
    }
    fn gate_input_pos(&self, gate_index: usize, input_index: usize) -> [f64; 2] {
        let gate: &circuit::Gate = &self.circuit.gates[gate_index];
        let [gate_x, gate_y, _, gate_height] = self.gate_box(gate_index);
        [gate_x, centered_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_index)]
    }
    fn gate_output_pos(&self, gate_index: usize, output_index: usize) -> [f64; 2] {
        let gate: &circuit::Gate = &self.circuit.gates[gate_index];
        let [gate_x, gate_y, _, gate_height] = self.gate_box(gate_index);
        [gate_x + GATE_WIDTH, centered_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_index)]
    }

    pub fn render(&self, graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs, inputs: &[bool]) {
        use graphics::*;
        let (_, evaluated_values) = self.circuit.eval_with_results(inputs);

        let value_pos = |value: circuit::Value| match value {
            circuit::Value::Arg(index) => self.circuit_input_pos(index, args.window_size),
            circuit::Value::GateValue(gate_index, arg_index) => self.gate_output_pos(gate_index, arg_index),
        };
        let on_off_color = |value| if value { ON_COLOR } else { OFF_COLOR };
        let get_value = |value| match value {
            circuit::Value::Arg(arg_index) => inputs[arg_index],
            circuit::Value::GateValue(gate_index, output_index) => evaluated_values[gate_index][output_index],
        };

        graphics.draw(args.viewport(), |c, gl| {
            clear(BG, gl);

            // draw circuit inputs and outputs
            for (i, input_value) in (0..self.circuit.num_inputs).zip(inputs.iter()) {
                let pos = self.circuit_input_pos(i, args.window_size);
                ellipse(on_off_color(*input_value), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
            }
            for (output_i, output) in self.circuit.outputs.iter().enumerate() {
                let output_pos = self.circuit_output_pos(output_i, args.window_size);
                let color = on_off_color(get_value(*output));
                ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

                // draw lines connecting outputs with their values
                let connection_start_pos = value_pos(*output);
                line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
            }

            // draw each gate
            for (gate_i, gate) in self.circuit.gates.iter().enumerate() {
                let [gate_x, gate_y, gate_width, gate_height] = self.gate_box(gate_i);

                rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
                // TODO: draw gate name
                // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

                // draw gate input dots and connections to their values
                for (input_i, input) in gate.inputs().into_iter().enumerate() {
                    let color = on_off_color(get_value(input));
                    let input_pos @ [x, y] = self.gate_input_pos(gate_i, input_i);
                    ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                    let connection_start_pos = value_pos(input);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                }
                // draw gate output dots
                for output_i in 0..gate.num_outputs() {
                    let color = on_off_color(evaluated_values[gate_i][output_i]);
                    let [x, y] = self.gate_output_pos(gate_i, output_i);
                    ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
                }
            }
        });
    }

    fn calculate_locations(&mut self) -> Vec<[f64; 2]> {
        /* old iterative position calculating algorithm based on a loss function and trying to find a minimum loss
        // gate position scoring; lower is better
        let score = |current_idx: usize, current_loc @ [x, y]: [f64; 2], gate: &circuit::Gate| -> f64 {
            let place_100_right_of_rightmost_input = {
                let desired_x = gate
                    .inputs()
                    .into_iter()
                    .map(|input| match input {
                        crate::circuit::Value::Arg(_) => 0.0,
                        crate::circuit::Value::GateValue(g, _) => self.locations[g][0],
                    })
                    .reduce(f64::max)
                    .unwrap_or(0.0)
                    + 100.0;

                ((x - desired_x) / 10.0).powf(2.0)
            };

            let place_y_at_middle_of_inputs: f64 = {
                let input_y = |input| match input {
                    circuit::Value::Arg(_) => 360.0, // TODO: dont hardcode input argument position
                    circuit::Value::GateValue(g, o) => self.gate_output_pos(g, o)[1],
                };
                let desired_y = (gate.inputs().into_iter().map(input_y).sum::<f64>()) / (gate.num_inputs() as f64);

                ((y - desired_y) / 10.0).powf(2.0)
            };

            let space_from_others: f64 = {
                let dist = |[x1, y1]: [f64; 2], [x2, y2]: [f64; 2]| ((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0)).sqrt();
                let min_dist = self
                    .locations
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(loc_idx, loc)| if loc_idx != current_idx && (loc[0] - current_loc[0]).abs() < 200.0 { dist(loc, current_loc) } else { f64::MAX })
                    .reduce(f64::min);

                match min_dist {
                    Some(min_dist) if min_dist < 100.0 => 10000.0 / min_dist,
                    _ => 0.0,
                }
            };

            place_100_right_of_rightmost_input + place_y_at_middle_of_inputs + space_from_others
        };

        let new_locations: Vec<[f64; 2]> = self
            .locations
            .iter()
            .zip(self.circuit.gates.iter())
            .enumerate()
            .map(|(idx, (location, gate))| {
                const DELTA: f64 = 0.0001;
                let x_deriv = (score(idx, [location[0] + DELTA, location[1]], gate) - score(idx, *location, gate)) / DELTA;
                let y_deriv = (score(idx, [location[0], location[1] + DELTA], gate) - score(idx, *location, gate)) / DELTA;

                [location[0] - x_deriv.clamp(-100.0, 100.0), location[1] - y_deriv.clamp(-100.0, 100.0)]
            })
            .collect();

        self.locations = new_locations;
        */

        // TODO: test this

        // group them into columns with each one going one column right of its rightmost dependency
        let mut xs: Vec<i32> = Vec::new();
        for gate in self.circuit.gates.iter() {
            let input_x = |value: &_| match value {
                circuit::Value::Arg(_) => 0,
                circuit::Value::GateValue(g, _) => xs[*g],
            };
            xs.push(gate.inputs().iter().map(input_x).max().unwrap_or(0) + 1)
        }

        // within each column sort them by the average of their input ys
        let mut ys: Vec<f64> = self.circuit.gates.iter().map(|_| 0.0).collect();
        for x in 1..=*xs.iter().max().unwrap() {
            let input_y = |input: &_| match input {
                circuit::Value::Arg(_) => 0,
                circuit::Value::GateValue(g, _) => ys[*g] as i32,
            };
            let mut on_col: Vec<_> = self.circuit.gates.iter().enumerate().filter(|(gate_i, _)| xs[*gate_i] == x).collect();
            on_col.sort_by_cached_key(|(_, gate)| gate.inputs().iter().map(input_y).sum::<i32>());

            const SPACING: f64 = 100.0;
            const CENTER_Y: f64 = 360.0;
            let all_height = SPACING * (on_col.len() - 1) as f64;
            let start_y = CENTER_Y - all_height / 2.0;
            for (i, (gate_i, _)) in on_col.iter().enumerate() {
                ys[*gate_i] = start_y + i as f64 * SPACING;
            }
        }

        xs.into_iter().zip(ys).map(|(x, y)| [x as f64 * 100.0, y]).collect()
    }
}
