use std::collections::HashSet;

use generational_arena::Arena;

use crate::simulation::{Gate, GateIndex};

use super::calculator::{self, ProducerIdx, ReceiverIdx};

pub(crate) enum Node {
    Producer(Producer),
    Passthrough(Receiver), // TODO: rename to Passthrough
                           // TODO: computation node?
}
pub(crate) struct Producer {
    dependants: HashSet<NodeIdx>,
    pub(crate) value: bool,
}

pub(crate) struct Receiver {
    producer: Option<NodeIdx>,
}

pub(crate) struct ProducersComponent(pub(crate) Vec<Producer>);
pub(crate) struct ReceiversComponent(pub(crate) Vec<Receiver>);

pub(crate) struct NodeIdx {
    gate: GateIndex,
    outputs: bool,
    index: usize,
}

impl Producer {
    pub(crate) fn new(value: bool) -> Self {
        Self { dependants: HashSet::new(), value }
    }

    pub(crate) fn dependants(&self) -> &'static HashSet<ReceiverIdx> {
        &self.dependants
    }
}
impl Receiver {
    pub(crate) fn new() -> Self {
        Self { producer: None }
    }

    pub(crate) fn producer(&self) -> Option<ProducerIdx> {
        self.producer
    }
}

// TODO: test connection, replacing old connection
pub(crate) fn connect(gates: &Arena<Gate>, producer_idx: ProducerIdx, receiver_idx: ReceiverIdx) {
    if let Some(old_producer) = get_receiver(receiver_idx).producer {
        get_receiver_mut(receiver_idx).producer = None;
        get_producer_mut(old_producer).dependants.remove(&receiver_idx);
    }

    get_receiver_mut(receiver_idx).producer = Some(producer_idx);
    get_producer_mut(producer_idx).dependants.insert(receiver_idx);
}

pub(crate) fn get_receiver(gates: &Arena<Gate>, index: ReceiverIdx) -> &Receiver {
    match index {
        ReceiverIdx::CO(co) => &self.outputs[co.0],
        ReceiverIdx::GI(gi) => self.get_gate(gi.0).get_input(gi),
    }
}
pub(crate) fn get_receiver_mut(gates: &mut Arena<Gate>, index: ReceiverIdx) -> &mut Receiver {
    match index {
        ReceiverIdx::CO(co) => &mut self.outputs[co.0],
        ReceiverIdx::GI(gi) => self.get_gate_mut(gi.0).get_input_mut(gi),
    }
}
pub(crate) fn get_producer(gates: &Arena<Gate>, index: ProducerIdx) -> &Producer {
    match index {
        ProducerIdx::CI(ci) => &self.inputs[ci.0],
        ProducerIdx::GO(go) => self.get_gate(go.0).get_output(go),
    }
}
pub(crate) fn get_producer_mut(gates: &mut Arena<Gate>, index: ProducerIdx) -> &mut Producer {
    match index {
        ProducerIdx::CI(ci) => &mut self.inputs[ci.0],
        ProducerIdx::GO(go) => self.get_gate_mut(go.0).get_output_mut(go),
    }
}

/* TODO this in the producers and receivers compoennts
    pub(crate) fn get_input(&self, input: GateInputNodeIdx) -> &Receiver {
        assert_eq!(self.index, input.0, "get input node with index that is not this node");
        let inputs = self._inputs();
        inputs.get(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, self.name(), inputs.len()))
    }
    pub(crate) fn get_input_mut(&mut self, input: GateInputNodeIdx) -> &mut Receiver {
        assert_eq!(self.index, input.0, "get input node with index that is not this node");
        let name = self.name();
        let inputs = self._inputs_mut();
        let len = inputs.len();
        inputs.get_mut(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, name, len))
        // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
        // TODO: there is also probably a better way of doing this that doesnt need
    }
    pub(crate) fn get_output(&self, index: GateOutputNodeIdx) -> &Producer {
        assert_eq!(self.index, index.0, "get output node with index that is not this node");
        let outputs = self._outputs();
        outputs.get(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, self.name(), outputs.len()))
    }
    pub(crate) fn get_output_mut(&mut self, index: GateOutputNodeIdx) -> &mut Producer {
        assert_eq!(self.index, index.0, "get output node with index that is not this node");
        let name = self.name();
        let outputs = self._outputs_mut();
        let len = outputs.len();
        outputs.get_mut(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
    }


    pub(crate) fn toggle_input(&mut self, i: usize) {
        assert!(i < self.inputs.len(), "toggle input out of range of number of inputs");
        self.set_input(CircuitInputNodeIdx(i), !self.get_producer(CircuitInputNodeIdx(i).into()).value);
    }

    pub(crate) fn set_num_inputs(&mut self, num: usize) {
        self.inputs.resize(num, Producer { gate: None, dependants: HashSet::new(), value: false });
    }
    pub(crate) fn set_num_outputs(&mut self, num: usize) {
        self.outputs.resize(num, Receiver { gate: None, producer: None });
    }


    // TODO: test removing, make sure it removes from both to keep in sync
    pub(crate) fn disconnect(&mut self, producer: ProducerIdx, receiver: ReceiverIdx) {
        todo!()
    }

    fn output_values(&self) -> impl Iterator<Item = bool> + '_ {
        // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
        self.outputs.iter().map(|output| if let Some(producer) = output.producer { self.get_producer(producer).value } else { false })
    }
*/
