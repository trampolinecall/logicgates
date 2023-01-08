use slotmap::SlotMap;

pub(crate) mod draw;
pub(crate) mod location;
pub(crate) mod logic;

// TODO: clean up everything in here, for example some places use indexes and some use direct references, things like that, ...

slotmap::new_key_type! {
    pub(crate) struct CircuitIndex;
}
slotmap::new_key_type! {
    pub(crate) struct GateIndex;
}
pub(crate) type CircuitMap = SlotMap<CircuitIndex, Circuit>;
pub(crate) type GateMap = SlotMap<GateIndex, Gate>;

pub(crate) struct Simulation {
    pub(crate) circuits: CircuitMap,
    pub(crate) gates: GateMap,

    pub(crate) main_circuit: CircuitIndex,
}

// circuit kind of blurs the boundary between the simulation and the logic component but
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
    pub(crate) location: location::Location,
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
