use super::connections;

pub(crate) type CircuitIndex = generational_arena::Index;
pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Circuit {
    pub(crate) index: CircuitIndex,
    pub(crate) name: String,
    pub(crate) gates: Vec<GateIndex>,
    pub(crate) inputs: Vec<connections::Producer>,
    pub(crate) outputs: Vec<connections::Receiver>,
}

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) kind: GateKind,
    pub(crate) location: (u32, f64),
    _dont_construct: (),
}

pub(crate) enum GateKind {
    Nand([connections::Receiver; 2], [connections::Producer; 1]), // TODO: figure out a better way of doing this
    Const([connections::Receiver; 0], [connections::Producer; 1]),
    Subcircuit(Vec<connections::Receiver>, Vec<connections::Producer>, CircuitIndex),
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
            inputs: std::iter::repeat_with(|| connections::Producer::new(None, false)).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| connections::Receiver::new(None)).take(num_outputs).collect(),
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
        self.inputs.resize(num, connections::Producer::new(None, false));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, connections::Receiver::new(None));
    }
}

impl Gate {
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand_gate(index: GateIndex) -> Gate {
        Gate {
            index,
            kind: GateKind::Nand([connections::Receiver::new(Some(index)), connections::Receiver::new(Some(index))], [connections::Producer::new(Some(index), true)]),
            location: (0, 0.0),
            _dont_construct: (),
        }
    }
    pub(crate) fn new_const_gate(index: GateIndex, value: bool) -> Gate {
        Gate { index, kind: GateKind::Const([], [connections::Producer::new(Some(index), value)]), location: (0, 0.0), _dont_construct: () }
    }
    pub(crate) fn new_subcircuit_gate(index: GateIndex, subcircuit: CircuitIndex) -> Gate {
        let num_inputs = todo!(); // subcircuit.inputs.len();
        let output_values: Vec<_> = todo!(); // subcircuit.output_values().collect();
        Gate {
            index,
            kind: GateKind::Subcircuit(
                (0..num_inputs).map(|_| connections::Receiver::new(Some(index))).collect(),
                output_values.into_iter().map(|value| connections::Producer::new(Some(index), value)).collect(),
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
            GateKind::Const(_, [connections::Producer { value: true, .. }]) => "true".to_string(),
            GateKind::Const(_, [connections::Producer { value: false, .. }]) => "false".to_string(),
            GateKind::Subcircuit(_, _, subcircuit) =>
            /* subcircuit.borrow().name.clone() */
            {
                todo!()
            }
        }
    }

    pub(crate) fn inputs(&self) -> &[connections::Receiver] {
        match &self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn outputs(&self) -> &[connections::Producer] {
        match &self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn inputs_mut(&mut self) -> &mut [connections::Receiver] {
        match &mut self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn outputs_mut(&mut self) -> &mut [connections::Producer] {
        match &mut self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
}
