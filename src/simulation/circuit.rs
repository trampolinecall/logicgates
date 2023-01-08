use super::connections;

pub(crate) type CircuitIndex = generational_arena::Index;
pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Circuit {
    pub(crate) index: CircuitIndex,
    pub(crate) name: String,
    pub(crate) gates: Vec<GateIndex>,
    pub(crate) inputs: Vec<connections::Node>,
    pub(crate) outputs: Vec<connections::Node>,
}

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) kind: GateKind,
    pub(crate) location: (u32, f64),
    _dont_construct: (),
}

pub(crate) enum GateKind {
    Nand([connections::Node; 2], [connections::Node; 1]), // TODO: figure out a better way of doing this
    Const([connections::Node; 0], [connections::Node; 1]),
    Subcircuit(Vec<connections::Node>, Vec<connections::Node>, CircuitIndex),
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

// TODO: refactor everything
impl Circuit {
    pub(crate) fn new(index: CircuitIndex, name: String, num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            index,
            name,
            gates: Vec::new(),
            inputs: std::iter::repeat_with(|| connections::Node::new_value(todo!(), false)).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| connections::Node::new_disconnected(todo!())).take(num_outputs).collect(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self.inputs.len()
    }
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    // TODO: tests
    // TODO: test that it removes all connections
    pub(crate) fn remove_gate(&mut self) {
        todo!()
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, connections::Node::new_value(todo!(), false));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, connections::Node::new_disconnected(todo!()));
    }
}

impl Gate {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand_gate(index: GateIndex) -> Gate {
        Gate {
            index,
            kind: GateKind::Nand([connections::Node::new_disconnected(index), connections::Node::new_disconnected(index)], [connections::Node::new_value(index, true)]),
            location: (0, 0.0),
            _dont_construct: (),
        }
    }
    pub(crate) fn new_const_gate(index: GateIndex, value: bool) -> Gate {
        Gate { index, kind: GateKind::Const([], [connections::Node::new_value(index, value)]), location: (0, 0.0), _dont_construct: () }
    }
    pub(crate) fn new_subcircuit_gate(index: GateIndex, subcircuit: CircuitIndex) -> Gate {
        let num_inputs = todo!(); // subcircuit.inputs.len();
        let output_values: Vec<_> = todo!(); // subcircuit.output_values().collect();
        Gate {
            index,
            kind: GateKind::Subcircuit(
                (0..num_inputs).map(|_| connections::Node::new_disconnected(index)).collect(),
                output_values.into_iter().map(|value| connections::Node::new_value(index, value)).collect(),
                subcircuit,
            ),
            location: (0, 0.0),
            _dont_construct: (),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self.inputs().len()
    }
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs().len()
    }

    pub(crate) fn name(&self) -> String {
        // TODO: hopefully somehow turn this into &str
        match &self.kind {
            GateKind::Nand(_, _) => "nand".to_string(),
            GateKind::Const(_, [_]) => todo!(),
            // GateKind::Const(_, [connections::Node { value: true, .. }]) => "true".to_string(),
            // GateKind::Const(_, [connections::Node { value: false, .. }]) => "false".to_string(),
            GateKind::Subcircuit(_, _, subcircuit) =>
            /* subcircuit.borrow().name.clone() */
            {
                todo!()
            }
        }
    }

    pub(crate) fn inputs(&self) -> &[connections::Node] {
        match &self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn outputs(&self) -> &[connections::Node] {
        match &self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn inputs_mut(&mut self) -> &mut [connections::Node] {
        match &mut self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn outputs_mut(&mut self) -> &mut [connections::Node] {
        match &mut self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
}
