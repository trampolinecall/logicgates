use super::logic;

pub(crate) type CircuitIndex = generational_arena::Index;
pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Circuit {
    pub(crate) index: CircuitIndex,
    pub(crate) name: String,
    pub(crate) gates: Vec<GateIndex>,
    pub(crate) inputs: Vec<logic::Node>,
    pub(crate) outputs: Vec<logic::Node>,
}

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) calculation: logic::Calculation,
    pub(crate) location: (u32, f64),
}

impl Circuit {
    pub(crate) fn new(index: CircuitIndex, name: String, num_inputs: usize, num_outputs: usize) -> Self {
        // even if this circuit is part of a subcircuit, these nodes dont have to update anything
        // instead, because the gates inside this circuit are connected to these nodes, updates to these nodes will propagate to the gates' nodes, properly updating those gates
        Self {
            index,
            name,
            gates: Vec::new(),
            inputs: std::iter::repeat_with(|| logic::Node::new(None, false)).take(num_inputs).collect(),
            outputs: std::iter::repeat_with(|| logic::Node::new(None, false)).take(num_outputs).collect(),
        }
    }

    pub(crate) fn num_inputs(&self) -> usize {
        self.inputs.len()
    }
    /* (unused, but may be used in the future)
    pub(crate) fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, connections::Node::new_disconnected(None));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, connections::Node::new_disconnected(None));
    }
    */
}
