use std::{cell::RefCell, collections::HashSet};

use generational_arena::Arena;

#[derive(Clone)]
pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Arena<Gate>,
    inputs: Vec<ValueProducingNode>,
    outputs: Vec<ValueReceivingNode>,
}
pub(crate) type GateIndex = generational_arena::Index;

#[derive(Clone)]
pub(crate) struct Gate {
    index: GateIndex,
    pub(crate) kind: GateKind,
    pub(crate) location: (f64, f64),
}

#[derive(Clone)]
pub(crate) enum GateKind {
    And([ValueReceivingNode; 2], [ValueProducingNode; 1]), // TODO: figure out a better way of doing this
    Not([ValueReceivingNode; 1], [ValueProducingNode; 1]),
    Const([ValueReceivingNode; 0], [ValueProducingNode; 1]),
    Subcircuit(Vec<ValueReceivingNode>, Vec<ValueProducingNode>, RefCell<Circuit>),
}

#[derive(Clone)]
pub(crate) struct ValueReceivingNode {
    gate: Option<GateIndex>,
    producer: Option<ValueProducingNodeIdx>,
}
#[derive(Clone)]
pub(crate) struct ValueProducingNode {
    dependants: HashSet<ValueReceivingNodeIdx>,
    value: bool,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct GateInputNodeIdx(GateIndex, usize);
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct GateOutputNodeIdx(GateIndex, usize);
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct CircuitInputNodeIdx(usize);
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct CircuitOutputNodeIdx(usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum ValueProducingNodeIdx {
    CI(CircuitInputNodeIdx),
    GO(GateOutputNodeIdx),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) enum ValueReceivingNodeIdx {
    CO(CircuitOutputNodeIdx),
    GI(GateInputNodeIdx),
}

/*
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

impl From<GateOutputNodeIdx> for ValueProducingNodeIdx {
    fn from(v: GateOutputNodeIdx) -> Self {
        Self::GO(v)
    }
}
impl From<CircuitInputNodeIdx> for ValueProducingNodeIdx {
    fn from(v: CircuitInputNodeIdx) -> Self {
        Self::CI(v)
    }
}
impl From<GateInputNodeIdx> for ValueReceivingNodeIdx {
    fn from(v: GateInputNodeIdx) -> Self {
        Self::GI(v)
    }
}
impl From<CircuitOutputNodeIdx> for ValueReceivingNodeIdx {
    fn from(v: CircuitOutputNodeIdx) -> Self {
        Self::CO(v)
    }
}

const VERTICAL_VALUE_SPACING: f64 = 20.0;

// TODO: refactor everything
impl Circuit {
    pub(crate) fn new(name: String) -> Self {
        Self { name, gates: Arena::new(), inputs: Vec::new(), outputs: Vec::new() }
    }

    pub(crate) fn input_indexes(&self) -> impl Iterator<Item = CircuitInputNodeIdx> {
        (0..self.inputs.len()).map(|i| CircuitInputNodeIdx(i))
    }
    pub(crate) fn output_indexes(&self) -> impl Iterator<Item = CircuitOutputNodeIdx> {
        (0..self.outputs.len()).map(|i| CircuitOutputNodeIdx(i))
    }

    fn output_values(&self) -> impl Iterator<Item = bool> + '_ {
        // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
        self.outputs.iter().map(|output| if let Some(producer) = output.producer { self.get_value_producing_node(producer).value } else { false })
    }

    // TODO: tests
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_and_gate(&mut self) -> GateIndex {
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::And(
                [ValueReceivingNode { gate: Some(index), producer: None }, ValueReceivingNode { gate: Some(index), producer: None }],
                [ValueProducingNode { dependants: HashSet::new(), value: false }],
            ),
            location: (0.0, 0.0),
        })
    }
    pub(crate) fn new_not_gate(&mut self) -> GateIndex {
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::Not([ValueReceivingNode { gate: Some(index), producer: None }], [ValueProducingNode { dependants: HashSet::new(), value: false }]),
            location: (0.0, 0.0),
        })
    }
    pub(crate) fn new_const_gate(&mut self, value: bool) -> GateIndex {
        self.gates.insert_with(|index| Gate { index, kind: GateKind::Const([], [ValueProducingNode { dependants: HashSet::new(), value: false }]), location: (0.0, 0.0) })
    }
    pub(crate) fn new_subcircuit_gate(&mut self, subcircuit: Circuit) -> GateIndex {
        let num_inputs = subcircuit.inputs.len();
        let output_values: Vec<_> = subcircuit.output_values().collect();
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::Subcircuit(
                (0..num_inputs).map(|_| ValueReceivingNode { gate: Some(index), producer: None }).collect(),
                output_values.into_iter().map(|value| ValueProducingNode { dependants: HashSet::new(), value }).collect(),
                RefCell::new(subcircuit),
            ),
            location: (0.0, 0.0),
        })
    }
    // TODO: test that it removes all connections
    pub(crate) fn remove_gate(&mut self) {
        todo!()
    }

    // TODO: test connection, replacing old connection
    pub(crate) fn connect(&mut self, producer_idx: ValueProducingNodeIdx, receiver_idx: ValueReceivingNodeIdx) {
        if let Some(old_producer) = self.get_value_receiving_node(receiver_idx).producer {
            self.get_value_receiving_node_mut(receiver_idx).producer = None;
            self.get_value_producing_node_mut(old_producer).dependants.remove(&receiver_idx);
        }

        self.get_value_receiving_node_mut(receiver_idx).producer = Some(producer_idx);
        self.get_value_producing_node_mut(producer_idx).dependants.insert(receiver_idx);
    }
    // TODO: test removing, make sure it removes from both to keep in sync
    pub(crate) fn disconnect(&mut self, producer: ValueProducingNodeIdx, receiver: ValueReceivingNodeIdx) {
        todo!()
    }

    pub(crate) fn set_input(&mut self, ci: CircuitInputNodeIdx, value: bool) {
        self.set_producer_value(ci.into(), value);
    }
    pub(crate) fn set_producer_value(&mut self, index: ValueProducingNodeIdx, value: bool) {
        let producer = self.get_value_producing_node_mut(index);
        producer.value = value;
        for dependant in producer.dependants.clone().into_iter() {
            // clone so that the borrow checker is happy, TODO: find better solution to this
            self.update_receiver(dependant)
        }
    }

    pub(crate) fn update_receiver(&mut self, receiver: ValueReceivingNodeIdx) {
        if let Some(gate) = self.get_value_receiving_node(receiver).gate {
            let gate = self.get_gate(gate);
            let outputs = gate.kind.compute(self);
            assert_eq!(outputs.len(), gate.num_outputs());
            for (output, node) in outputs.into_iter().zip(gate.outputs().collect::<Vec<_>>().into_iter()) {
                self.set_producer_value(node.into(), output);
            }
        }
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, ValueProducingNode { dependants: HashSet::new(), value: false })
    }

    pub(crate) fn get_gate(&self, index: GateIndex) -> &Gate {
        self.gates.get(index).unwrap()
    }
    pub(crate) fn get_gate_mut(&mut self, index: GateIndex) -> &mut Gate {
        self.gates.get_mut(index).unwrap()
    }

    pub(crate) fn get_value_receiving_node(&self, index: ValueReceivingNodeIdx) -> &ValueReceivingNode {
        match index {
            ValueReceivingNodeIdx::CO(co) => &self.outputs[co.0],
            ValueReceivingNodeIdx::GI(gi) => self.get_gate(gi.0).get_input(gi),
        }
    }
    pub(crate) fn get_value_receiving_node_mut(&mut self, index: ValueReceivingNodeIdx) -> &mut ValueReceivingNode {
        match index {
            ValueReceivingNodeIdx::CO(co) => &mut self.outputs[co.0],
            ValueReceivingNodeIdx::GI(gi) => self.get_gate_mut(gi.0).get_input_mut(gi),
        }
    }
    pub(crate) fn get_value_producing_node(&self, index: ValueProducingNodeIdx) -> &ValueProducingNode {
        match index {
            ValueProducingNodeIdx::CI(ci) => &self.inputs[ci.0],
            ValueProducingNodeIdx::GO(go) => self.get_gate(go.0).get_output(go),
        }
    }
    pub(crate) fn get_value_producing_node_mut(&mut self, index: ValueProducingNodeIdx) -> &mut ValueProducingNode {
        match index {
            ValueProducingNodeIdx::CI(ci) => &mut self.inputs[ci.0],
            ValueProducingNodeIdx::GO(go) => self.get_gate_mut(go.0).get_output_mut(go),
        }
    }
}

impl Gate {
    pub(crate) fn inputs(&self) -> impl Iterator<Item = GateInputNodeIdx> + '_ {
        (0..self._inputs().len()).map(|i| GateInputNodeIdx(self.index, i))
    }

    pub(crate) fn outputs(&self) -> impl Iterator<Item = GateOutputNodeIdx> + '_ {
        (0..self._outputs().len()).map(|i| GateOutputNodeIdx(self.index, i))
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self._inputs().len()
    }
    pub(crate) fn num_outputs(&self) -> usize {
        self._outputs().len()
    }

    pub(crate) fn name(&self) -> String {
        // TODO: hopefully somehow turn this into &str
        match &self.kind {
            GateKind::And(_, _) => "and".to_string(),
            GateKind::Not(_, _) => "not".to_string(),
            GateKind::Const(_, [ValueProducingNode { value: true, .. }]) => "true".to_string(),
            GateKind::Const(_, [ValueProducingNode { value: false, .. }]) => "false".to_string(),
            GateKind::Subcircuit(_, _, subcircuit) => subcircuit.borrow().name.clone(),
        }
    }

    pub(crate) fn _inputs(&self) -> &[ValueReceivingNode] {
        match &self.kind {
            GateKind::And(i, _) => i,
            GateKind::Not(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs(&self) -> &[ValueProducingNode] {
        match &self.kind {
            GateKind::And(_, o) => o,
            GateKind::Not(_, o) => o,
            GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn _inputs_mut(&mut self) -> &mut [ValueReceivingNode] {
        match &mut self.kind {
            GateKind::And(i, _) => i,
            GateKind::Not(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs_mut(&mut self) -> &mut [ValueProducingNode] {
        match &mut self.kind {
            GateKind::And(_, o) => o,
            GateKind::Not(_, o) => o,
            GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn get_input(&self, input: GateInputNodeIdx) -> &ValueReceivingNode {
        assert_eq!(self.index, input.0, "get input node with index that is not this node");
        let inputs = self._inputs();
        inputs.get(input.1).expect(&format!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, self.name(), inputs.len()))
    }
    pub(crate) fn get_input_mut(&mut self, input: GateInputNodeIdx) -> &mut ValueReceivingNode {
        assert_eq!(self.index, input.0, "get input node with index that is not this node");
        let name = self.name();
        let inputs = self._inputs_mut();
        let len = inputs.len();
        inputs.get_mut(input.1).expect(&format!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, name, len))
        // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
        // TODO: there is also probably a better way of doing this that doesnt need
    }
    pub(crate) fn get_output(&self, index: GateOutputNodeIdx) -> &ValueProducingNode {
        assert_eq!(self.index, index.0, "get output node with index that is not this node");
        let outputs = self._outputs();
        outputs.get(index.1).expect(&format!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, self.name(), outputs.len()))
    }
    pub(crate) fn get_output_mut(&mut self, index: GateOutputNodeIdx) -> &mut ValueProducingNode {
        assert_eq!(self.index, index.0, "get output node with index that is not this node");
        let name = self.name();
        let outputs = self._outputs_mut();
        let len = outputs.len();
        outputs.get_mut(index.1).expect(&format!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
    }

    pub(crate) fn size(&self) -> [f64; 2] {
        const GATE_WIDTH: f64 = 50.0;
        const EXTRA_VERTICAL_HEIGHT: f64 = 40.0;

        let gate_height = (std::cmp::max(self.num_inputs(), self.num_outputs()) - 1) as f64 * VERTICAL_VALUE_SPACING + EXTRA_VERTICAL_HEIGHT;
        [GATE_WIDTH, gate_height]
    }
}

impl GateKind {
    pub(crate) fn compute(&self, circuit: &Circuit) -> Vec<bool> {
        let get_producer_value = |producer_idx| if let Some(producer_idx) = producer_idx { circuit.get_value_producing_node(producer_idx).value } else { false };
        // TODO: figure out a way for this to set its outputs
        match self {
            GateKind::And([a, b], _) => vec![get_producer_value(a.producer) && get_producer_value(b.producer)],
            GateKind::Not([i], _) => vec![!get_producer_value(i.producer)],
            GateKind::Const(_, [o]) => vec![o.value],
            GateKind::Subcircuit(inputs, _, subcircuit) => {
                let mut subcircuit = subcircuit.borrow_mut();
                for (input_node, subcircuit_input_node) in inputs.iter().zip(subcircuit.input_indexes()) {
                    subcircuit.set_producer_value(subcircuit_input_node.into(), get_producer_value(input_node.producer))
                }

                circuit
                    .output_indexes()
                    .into_iter()
                    .map(|output_idx| if let Some(producer) = subcircuit.get_value_receiving_node(output_idx.into()).producer { subcircuit.get_value_producing_node(producer).value } else { false })
                    .collect()
            }
        }
    }
}

/* TODO
impl Circuit {
        pub(crate) fn render(&self, graphics: &mut opengl_graphics::GlGraphics, args: &piston::RenderArgs) {
            use graphics::*;
            let circuit_input_pos = |index: usize| -> [f64; 2] { [0.0, centered_arg_y(args.window_size[1] / 2.0, self.inputs.len(), index)] };
            let circuit_output_pos = |index: usize| -> [f64; 2] { [args.window_size[0], centered_arg_y(args.window_size[1] / 2.0, self.outputs.len(), index)] };

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

            let gate_box = |gate: &Gate| -> [f64; 4] {
                let (gate_x, gate_y) = gate.location;
                let [gate_width, gate_height] = gate_size(gate);
                [gate_x as f64 * HORIZONTAL_GATE_SPACING, gate_y + args.window_size[1] / 2.0, gate_width, gate_height]
            };
            let gate_input_pos = |input_idx: GateInputNodeIdx| -> [f64; 2] {
                let gate = &self.gates[input_idx.0];
                let [gate_x, gate_y, _, gate_height] = gate_box(gate);
                [gate_x, centered_arg_y(gate_y + gate_height / 2.0, gate.inputs().len(), input_idx.1)]
            };
            let gate_output_pos = |output_idx: GateOutputNodeIdx| -> [f64; 2] {
                let gate = &self.gates[output_idx.0];
                let [gate_x, gate_y, gate_width, gate_height] = gate_box(gate);
                [gate_x + gate_width, centered_arg_y(gate_y + gate_height / 2.0, gate.outputs().len(), output_idx.1)]
            };

            let producer_node_pos = |node: ValueProducingNodeIdx| match node {
                ValueProducingNodeIdx::CI(_) => todo!(),
                ValueProducingNodeIdx::GO(_) => todo!(),
            };
            let receiver_node_pos = |node: ValueReceivingNodeIdx| match node {
                ValueReceivingNodeIdx::CO(_) => todo!(),
                ValueReceivingNodeIdx::GI(_) => todo!(),
            };
            let bool_color = |value| if value { ON_COLOR } else { OFF_COLOR };
            let producer_color = |producer: ValueProducingNodeIdx| bool_color(match producer {
                ValueProducingNodeIdx::CI(CircuitInputNodeIdx(idx)) => self.inputs[idx].value,
                ValueProducingNodeIdx::GO(go) => self.get_gate_output(go).value,
            });

            graphics.draw(args.viewport(), |c, gl| {
                clear(BG, gl);

                // draw circuit inputs and outputs
                for (input_i, input_value) in self.inputs.iter().enumerate() {
                    let pos = circuit_input_pos(input_i);
                    ellipse(bool_color(input_value.value), ellipse::circle(pos[0], pos[1], CIRCLE_RAD), c.transform, gl);
                }
                for (output_i, output) in self.outputs.iter().enumerate() {
                    let output_pos = circuit_output_pos(output_i);
                    let color = producer_color(*output);
                    ellipse(color, ellipse::circle(output_pos[0], output_pos[1], CIRCLE_RAD), c.transform, gl);

                    // draw lines connecting outputs with their values
                    let connection_start_pos = value_pos(*output);
                    line_from_to(color, CONNECTION_RAD, connection_start_pos, output_pos, c.transform, gl);
                }

                // draw each gate
                for (gate_i, gate) in self.gates.iter() {
                    let [gate_x, gate_y, gate_width, gate_height] = gate_box(gate);

                    rectangle(GATE_COLOR, [gate_x, gate_y, gate_width, gate_height], c.transform, gl);
                    // TODO: draw gate name
                    // text(BLACK, 10, gate.name(), /* character cache */, c.transform, gl);

                    // draw gate input dots and connections to their values
                    for (input_i, input) in gate.inputs().into_iter().enumerate() {
                        let color = value_color(input);
                        let input_pos @ [x, y] = gate_input_pos(input);
                        ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);

                        let connection_start_pos = value_pos(gate.get_input(input).producer);
                        line_from_to(color, CONNECTION_RAD, connection_start_pos, input_pos, c.transform, gl);
                    }
                    // draw gate output dots
                    for output in gate.outputs().into_iter() {
                        let color = bool_color(evaluated_values[gate_i][output_i]);
                        let [x, y] = gate_output_pos(output);
                        ellipse(color, ellipse::circle(x, y, CIRCLE_RAD), c.transform, gl);
                    }
                }
            });
        }
}
*/
