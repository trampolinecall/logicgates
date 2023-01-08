use std::collections::HashSet;
// TODO: rename to calculation and further split into submodules

use super::circuit::{Circuit, Gate, GateIndex, GateKind};

#[derive(Clone)]
pub(crate) struct Receiver {
    pub(crate) gate: Option<GateIndex>,
    pub(crate) producer: Option<ProducerIdx>,
    _dont_construct: (),
}

#[derive(Clone)]
pub(crate) struct Producer {
    pub(crate) gate: Option<GateIndex>,
    dependants: HashSet<ReceiverIdx>,
    pub(crate) value: bool,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateInputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ()); // unit at the end so that it cannot be constructed outside of this module
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateOutputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitInputNodeIdx(pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOutputNodeIdx(pub(crate) usize, ());

impl Receiver {
    pub(crate) fn new(gate: Option<GateIndex>) -> Self {
        Self { gate, producer: None, _dont_construct: () }
    }
}
impl Producer {
    pub(crate) fn new(gate: Option<GateIndex>, value: bool) -> Self {
        Self { gate, dependants: HashSet::new(), value }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum ProducerIdx {
    CI(CircuitInputNodeIdx),
    GO(GateOutputNodeIdx),
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) enum ReceiverIdx {
    CO(CircuitOutputNodeIdx),
    GI(GateInputNodeIdx),
}

impl From<GateOutputNodeIdx> for ProducerIdx {
    fn from(v: GateOutputNodeIdx) -> Self {
        Self::GO(v)
    }
}
impl From<CircuitInputNodeIdx> for ProducerIdx {
    fn from(v: CircuitInputNodeIdx) -> Self {
        Self::CI(v)
    }
}
impl From<GateInputNodeIdx> for ReceiverIdx {
    fn from(v: GateInputNodeIdx) -> Self {
        Self::GI(v)
    }
}
impl From<CircuitOutputNodeIdx> for ReceiverIdx {
    fn from(v: CircuitOutputNodeIdx) -> Self {
        Self::CO(v)
    }
}

pub(crate) fn input_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitInputNodeIdx> {
    (0..circuit.inputs.len()).map(|i| CircuitInputNodeIdx(i, ()))
}
pub(crate) fn output_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitOutputNodeIdx> {
    (0..circuit.outputs.len()).map(|i| CircuitOutputNodeIdx(i, ()))
}

// TODO: test connection, replacing old connection
pub(crate) fn connect(circuit: &mut Circuit, producer_idx: ProducerIdx, receiver_idx: ReceiverIdx) {
    if let Some(old_producer) = get_receiver(circuit, receiver_idx).producer {
        get_receiver_mut(circuit, receiver_idx).producer = None;
        get_producer_mut(circuit, old_producer).dependants.remove(&receiver_idx);
    }

    get_receiver_mut(circuit, receiver_idx).producer = Some(producer_idx);
    get_producer_mut(circuit, producer_idx).dependants.insert(receiver_idx);
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(circuit: &mut Circuit, producer: ProducerIdx, receiver: ReceiverIdx) {
    todo!()
}

pub(crate) fn get_receiver(circuit: &Circuit, index: ReceiverIdx) -> &Receiver {
    match index {
        ReceiverIdx::CO(co) => &circuit.outputs[co.0],
        ReceiverIdx::GI(gi) => gate_get_input(circuit.get_gate(gi.0), gi),
    }
}
pub(crate) fn get_receiver_mut(circuit: &mut Circuit, index: ReceiverIdx) -> &mut Receiver {
    match index {
        ReceiverIdx::CO(co) => &mut circuit.outputs[co.0],
        ReceiverIdx::GI(gi) => gate_get_input_mut(circuit.get_gate_mut(gi.0), gi),
    }
}
pub(crate) fn get_producer(circuit: &Circuit, index: ProducerIdx) -> &Producer {
    match index {
        ProducerIdx::CI(ci) => &circuit.inputs[ci.0],
        ProducerIdx::GO(go) => gate_get_output(circuit.get_gate(go.0), go),
    }
}
pub(crate) fn get_producer_mut(circuit: &mut Circuit, index: ProducerIdx) -> &mut Producer {
    match index {
        ProducerIdx::CI(ci) => &mut circuit.inputs[ci.0],
        ProducerIdx::GO(go) => gate_get_output_mut(circuit.get_gate_mut(go.0), go),
    }
}

pub(crate) fn toggle_input(circuit: &mut Circuit, i: usize) {
    assert!(i < circuit.inputs.len(), "toggle input out of range of number of inputs");
    set_input(circuit, CircuitInputNodeIdx(i, ()), !get_producer(circuit, CircuitInputNodeIdx(i, ()).into()).value);
}
pub(crate) fn set_input(circuit: &mut Circuit, ci: CircuitInputNodeIdx, value: bool) {
    set_producer_value(circuit, ci.into(), value);
    // TODO: consider whether or not this is supposed to call update()
}

pub(crate) fn update(circuit: &mut Circuit) {
    let mut stack: Vec<_> = circuit.gates.iter().map(|(i, _)| i).collect();
    let mut changed = HashSet::new();
    while let Some(gate) = stack.pop() {
        if changed.contains(&gate) {
            continue;
        }

        let gate_changed = update_gate(circuit, &mut stack, gate);
        if gate_changed {
            changed.insert(gate);
        }
    }
}
fn set_producer_value(circuit: &mut Circuit, index: ProducerIdx, value: bool) {
    let producer = get_producer_mut(circuit, index);
    producer.value = value;
    // caller should call update next
}

fn update_gate(circuit: &mut Circuit, update_stack: &mut Vec<GateIndex>, gate: GateIndex) -> bool {
    let gate = circuit.get_gate(gate);
    let outputs = compute(&gate.kind, circuit);
    assert_eq!(outputs.len(), gate.num_outputs());

    let mut changed = false;

    for (new_value, output_node) in outputs.into_iter().zip(gate_outputs(gate).collect::<Vec<_>>().into_iter()) {
        let as_producer_index = output_node.into();
        let old_value = get_producer(circuit, as_producer_index).value;
        if old_value != new_value {
            changed = true;
        }
        set_producer_value(circuit, as_producer_index, new_value);

        for dependant in get_producer(circuit, as_producer_index).dependants.clone() {
            // clone so that the borrow checker is happy, TODO: find better solution to this
            if let Some(gate) = get_receiver(circuit, dependant).gate {
                update_stack.push(gate);
            }
        }
    }

    changed
}

pub(crate) fn gate_get_input(gate: &Gate, input: GateInputNodeIdx) -> &Receiver {
    assert_eq!(gate.index, input.0, "get input node with index that is not this node");
    let inputs = gate.inputs();
    inputs.get(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, gate.name(), inputs.len()))
}
pub(crate) fn gate_get_input_mut(gate: &mut Gate, input: GateInputNodeIdx) -> &mut Receiver {
    assert_eq!(gate.index, input.0, "get input node with index that is not this node");
    let name = gate.name();
    let inputs = gate.inputs_mut();
    let len = inputs.len();
    inputs.get_mut(input.1).unwrap_or_else(|| panic!("gate input node index invalid: index has index {} but '{}' gate has only {} inputs", input.1, name, len))
    // TODO: there is probably a better way of doing this that doesnt need this code to be copy pasted
    // TODO: there is also probably a better way of doing this that doesnt need
}
pub(crate) fn gate_get_output(gate: &Gate, index: GateOutputNodeIdx) -> &Producer {
    assert_eq!(gate.index, index.0, "get output node with index that is not this node");
    let outputs = gate.outputs();
    outputs.get(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, gate.name(), outputs.len()))
}
pub(crate) fn gate_get_output_mut(gate: &mut Gate, index: GateOutputNodeIdx) -> &mut Producer {
    assert_eq!(gate.index, index.0, "get output node with index that is not this node");
    let name = gate.name();
    let outputs = gate.outputs_mut();
    let len = outputs.len();
    outputs.get_mut(index.1).unwrap_or_else(|| panic!("gate output node index invalid: index has index {} but '{}' gate has only {} outputs", index.1, name, len))
}

pub(crate) fn gate_inputs(gate: &Gate) -> impl ExactSizeIterator<Item = GateInputNodeIdx> + '_ {
    (0..gate.inputs().len()).map(|i| GateInputNodeIdx(gate.index, i, ()))
}

pub(crate) fn gate_outputs(gate: &Gate) -> impl ExactSizeIterator<Item = GateOutputNodeIdx> + '_ {
    (0..gate.outputs().len()).map(|i| GateOutputNodeIdx(gate.index, i, ()))
}

pub(crate) fn compute(gate: &GateKind, circuit: &Circuit) -> Vec<bool> {
    let get_producer_value = |producer_idx| if let Some(producer_idx) = producer_idx { get_producer(circuit, producer_idx).value } else { false };
    // TODO: figure out a way for this to set its outputs
    match gate {
        GateKind::Nand([a, b], _) => vec![!(get_producer_value(a.producer) && get_producer_value(b.producer))],
        GateKind::Const(_, [o]) => vec![o.value],
        GateKind::Subcircuit(inputs, _, subcircuit) => {
            let mut subcircuit = subcircuit.borrow_mut();
            for (input_node, subcircuit_input_node) in inputs.iter().zip(input_indexes(&mut subcircuit)) {
                set_producer_value(&mut subcircuit, subcircuit_input_node.into(), get_producer_value(input_node.producer));
            }

            // TODO: move everything into one global Gate arena which means figuring out lifetimes and things
            update(&mut subcircuit);
            output_indexes(&subcircuit)
                .into_iter()
                .map(|output_idx| if let Some(producer) = get_receiver(&mut subcircuit, output_idx.into()).producer { get_producer(&mut subcircuit, producer).value } else { false })
                .collect()
        }
    }
}
