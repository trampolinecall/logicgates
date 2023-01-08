use generational_arena::Arena;
use std::cell::RefCell;
use std::collections::HashSet;

use crate::simulation::connections::{GateInputNodeIdx, GateOutputNodeIdx};

use super::connections::{self, Producer, ProducerIdx, Receiver, ReceiverIdx};

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Arena<Gate>,
    pub(crate) inputs: Vec<Producer>,
    pub(crate) outputs: Vec<Receiver>,
}

pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) kind: GateKind,
    pub(crate) location: (u32, f64),
}

pub(crate) enum GateKind {
    Nand([Receiver; 2], [Producer; 1]), // TODO: figure out a better way of doing this
    Const([Receiver; 0], [Producer; 1]),
    Subcircuit(Vec<Receiver>, Vec<Producer>, RefCell<Circuit>),
}

/* TODO: decide what to do with this
impl CustomGate {
    pub(crate) fn table(&self) -> HashMap<Vec<bool>, Vec<bool>> {
        utils::enumerate_inputs(self.num_inputs)
            .into_iter()
            .map(|input| {
                let res = self.eval(&input);
                (input, res)
            })
            .collect()
    }
}
*/

const VERTICAL_VALUE_SPACING: f64 = 20.0;

// TODO: refactor everything
impl Circuit {
    pub(crate) fn new(name: String, num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            name,
            gates: Arena::new(),
            inputs: std::iter::repeat_with(|| Producer::new(None, false)).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| Receiver::new(None)).take(num_outputs).collect(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self.inputs.len()
    }
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    fn output_values(&self) -> impl Iterator<Item = bool> + '_ {
        // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
        self.outputs.iter().map(|output| if let Some(producer) = output.producer { connections::get_producer(self, producer).value } else { false })
    }

    // TODO: tests
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand_gate(&mut self) -> GateIndex {
        self.gates.insert_with(|index| Gate { index, kind: GateKind::Nand([Receiver::new(Some(index)), Receiver::new(Some(index))], [Producer::new(Some(index), true)]), location: (0, 0.0) })
    }
    pub(crate) fn new_const_gate(&mut self, value: bool) -> GateIndex {
        self.gates.insert_with(|index| Gate { index, kind: GateKind::Const([], [Producer::new(Some(index), value)]), location: (0, 0.0) })
    }
    pub(crate) fn new_subcircuit_gate(&mut self, subcircuit: Circuit) -> GateIndex {
        let num_inputs = subcircuit.inputs.len();
        let output_values: Vec<_> = subcircuit.output_values().collect();
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::Subcircuit(
                (0..num_inputs).map(|_| Receiver::new(Some(index))).collect(),
                output_values.into_iter().map(|value| Producer::new(Some(index), value)).collect(),
                RefCell::new(subcircuit),
            ),
            location: (0, 0.0),
        })
    }
    // TODO: test that it removes all connections
    pub(crate) fn remove_gate(&mut self) {
        todo!()
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, Producer::new(None, false));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, Receiver::new(None));
    }

    pub(crate) fn get_gate(&self, index: GateIndex) -> &Gate {
        self.gates.get(index).unwrap()
    }
    pub(crate) fn get_gate_mut(&mut self, index: GateIndex) -> &mut Gate {
        self.gates.get_mut(index).unwrap()
    }

    pub(crate) fn calculate_locations(&mut self) {
        let positions = crate::simulation::position::calculate_locations(self);
        for (gate_i, position) in positions {
            self.get_gate_mut(gate_i).location = position;
        }
    }
}

impl Gate {
    pub(crate) fn num_inputs(&self) -> usize {
        self._inputs().len()
    }
    pub(crate) fn num_outputs(&self) -> usize {
        self._outputs().len()
    }

    pub(crate) fn name(&self) -> String {
        // TODO: hopefully somehow turn this into &str
        match &self.kind {
            GateKind::Nand(_, _) => "nand".to_string(),
            GateKind::Const(_, [Producer { value: true, .. }]) => "true".to_string(),
            GateKind::Const(_, [Producer { value: false, .. }]) => "false".to_string(),
            GateKind::Subcircuit(_, _, subcircuit) => subcircuit.borrow().name.clone(),
        }
    }

    pub(crate) fn _inputs(&self) -> &[Receiver] {
        match &self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs(&self) -> &[Producer] {
        match &self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn _inputs_mut(&mut self) -> &mut [Receiver] {
        match &mut self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs_mut(&mut self) -> &mut [Producer] {
        match &mut self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }

    pub(crate) fn display_size(&self) -> [f64; 2] {
        const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;
        const GATE_WIDTH: f64 = 50.0;

        let gate_height = (std::cmp::max(self.num_inputs(), self.num_outputs()) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
        [GATE_WIDTH, gate_height]
    }
}

impl GateKind {}

impl Circuit {
    pub(crate) fn render(&self, graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs) {
        use graphics::*;
        const CIRCLE_RAD: f64 = 5.0;
        const CONNECTION_RAD: f64 = CIRCLE_RAD / 2.0;
        const HORIZONTAL_GATE_SPACING: f64 = 100.0;

        const BG: [f32; 4] = [0.172, 0.243, 0.313, 1.0];
        const GATE_COLOR: [f32; 4] = [0.584, 0.647, 0.65, 1.0];
        const ON_COLOR: [f32; 4] = [0.18, 0.8, 0.521, 1.0];
        const OFF_COLOR: [f32; 4] = [0.498, 0.549, 0.552, 1.0];

        pub(crate) fn centered_arg_y(center_y: f64, num_args: usize, i: usize) -> f64 {
            let args_height: f64 = ((num_args - 1) as f64) * VERTICAL_VALUE_SPACING;
            let args_start_y = center_y - (args_height / 2.0);
            args_start_y + (i as f64) * VERTICAL_VALUE_SPACING
        }

        let circuit_input_pos = |index: usize| -> [f64; 2] { [0.0, centered_arg_y(args.window_size[1] / 2.0, self.inputs.len(), index)] };
        let circuit_output_pos = |index: usize| -> [f64; 2] { [args.window_size[0], centered_arg_y(args.window_size[1] / 2.0, self.outputs.len(), index)] };

        let gate_box = |gate: &Gate| -> [f64; 4] {
            let (gate_x, gate_y) = gate.location;
            let [gate_width, gate_height] = gate.display_size();
            [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
        };
        let gate_input_pos = |input_idx: GateInputNodeIdx| -> [f64; 2] {
            let gate = &self.gates[input_idx.0];
            let [gate_x, gate_y, _, gate_height] = gate_box(gate);
            [gate_x, centered_arg_y(gate_y + gate_height / 2.0, gate.num_inputs(), input_idx.1)]
        };
        let gate_output_pos = |output_idx: GateOutputNodeIdx| -> [f64; 2] {
            let gate = &self.gates[output_idx.0];
            let [gate_x, gate_y, gate_width, gate_height] = gate_box(gate);
            [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, gate.num_outputs(), output_idx.1)]
        };

        let producer_pos = |node: ProducerIdx| match node {
            ProducerIdx::CI(ci) => circuit_input_pos(ci.0),
            ProducerIdx::GO(go) => gate_output_pos(go),
        };
        /* (unused)
        let receiver_node_pos = |node: ReceiverIdx| match node {
            ReceiverIdx::CO(co) => circuit_output_pos(co.0),
            ReceiverIdx::GI(gi) => gate_input_pos(gi),
        };
        */
        let bool_color = |value| if value { ON_COLOR } else { OFF_COLOR };
        let producer_color = |producer: ProducerIdx| bool_color(connections::get_producer(self, producer).value);
        let receiver_color =
            |receiver: ReceiverIdx| bool_color(if let Some(producer) = connections::get_receiver(self, receiver).producer { connections::get_producer(self, producer).value } else { false });

        graphics.draw(args.viewport(), |c, gl| {
            clear(BG, gl);

            // draw circuit inputs and outputs
            for (input_i, input_producer) in self.inputs.iter().enumerate() {
                let pos = circuit_input_pos(input_i);
                ellipse(bool_color(input_producer.value), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
            }
            for (output_i, output) in connections::output_indexes(self).enumerate() {
                let output_pos = circuit_output_pos(output_i);
                let color = receiver_color(output.into());
                ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

                // draw lines connecting outputs with their values
                if let Some(producer) = connections::get_receiver(self, output.into()).producer {
                    let connection_start_pos = producer_pos(producer);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
                }
            }

            // draw each gate
            for (_, gate) in self.gates.iter() {
                let [gate_x, gate_y, gate_width, gate_height] = gate_box(gate);

                rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
                // TODO: draw gate name
                // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

                // draw gate input dots and connections to their values
                for input_receiver in connections::gate_inputs(gate) {
                    let color = receiver_color(input_receiver.into());
                    let input_pos @ [x, y] = gate_input_pos(input_receiver);
                    ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                    if let Some(producer) = connections::get_receiver(self, input_receiver.into()).producer {
                        let connection_start_pos = producer_pos(producer);
                        line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                    }
                }
                // draw gate output dots
                for output in connections::gate_outputs(gate) {
                    let color = producer_color(output.into());
                    let [x, y] = gate_output_pos(output);
                    ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
                }
            }
        });
    }
}
