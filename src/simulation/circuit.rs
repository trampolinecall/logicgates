use generational_arena::Arena;
use std::cell::RefCell;

use super::connections;

pub(crate) struct Circuit {
    pub(crate) name: String,
    pub(crate) gates: Arena<Gate>,
    pub(crate) inputs: Vec<connections::Producer>,
    pub(crate) outputs: Vec<connections::Receiver>,
}

pub(crate) type GateIndex = generational_arena::Index;

pub(crate) struct Gate {
    pub(crate) index: GateIndex,
    pub(crate) kind: GateKind,
    pub(crate) location: (u32, f64),
}

pub(crate) enum GateKind {
    Nand([connections::Receiver; 2], [connections::Producer; 1]), // TODO: figure out a better way of doing this
    Const([connections::Receiver; 0], [connections::Producer; 1]),
    Subcircuit(Vec<connections::Receiver>, Vec<connections::Producer>, RefCell<Circuit>),
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
    pub(crate) fn new(name: String, num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            name,
            gates: Arena::new(),
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

    fn output_values(&self) -> impl Iterator<Item = bool> + '_ {
        // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
        self.outputs.iter().map(|output| if let Some(producer) = output.producer { connections::get_producer(self, producer).value } else { false })
    }

    // TODO: tests
    // default value for the outputs is whatever value results from having all false inputs
    pub(crate) fn new_nand_gate(&mut self) -> GateIndex {
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::Nand([connections::Receiver::new(Some(index)), connections::Receiver::new(Some(index))], [connections::Producer::new(Some(index), true)]),
            location: (0, 0.0),
        })
    }
    pub(crate) fn new_const_gate(&mut self, value: bool) -> GateIndex {
        self.gates.insert_with(|index| Gate { index, kind: GateKind::Const([], [connections::Producer::new(Some(index), value)]), location: (0, 0.0) })
    }
    pub(crate) fn new_subcircuit_gate(&mut self, subcircuit: Circuit) -> GateIndex {
        let num_inputs = subcircuit.inputs.len();
        let output_values: Vec<_> = subcircuit.output_values().collect();
        self.gates.insert_with(|index| Gate {
            index,
            kind: GateKind::Subcircuit(
                (0..num_inputs).map(|_| connections::Receiver::new(Some(index))).collect(),
                output_values.into_iter().map(|value| connections::Producer::new(Some(index), value)).collect(),
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
        self.inputs.resize(num, connections::Producer::new(None, false));
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, connections::Receiver::new(None));
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
            GateKind::Const(_, [connections::Producer { value: true, .. }]) => "true".to_string(),
            GateKind::Const(_, [connections::Producer { value: false, .. }]) => "false".to_string(),
            GateKind::Subcircuit(_, _, subcircuit) => subcircuit.borrow().name.clone(),
        }
    }

    pub(crate) fn _inputs(&self) -> &[connections::Receiver] {
        match &self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs(&self) -> &[connections::Producer] {
        match &self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
    pub(crate) fn _inputs_mut(&mut self) -> &mut [connections::Receiver] {
        match &mut self.kind {
            GateKind::Nand(i, _) => i,
            GateKind::Const(i, _) => i,
            GateKind::Subcircuit(i, _, _) => i,
        }
    }
    pub(crate) fn _outputs_mut(&mut self) -> &mut [connections::Producer] {
        match &mut self.kind {
            GateKind::Nand(_, o) | GateKind::Const(_, o) => o,
            GateKind::Subcircuit(_, o, _) => o,
        }
    }
}
