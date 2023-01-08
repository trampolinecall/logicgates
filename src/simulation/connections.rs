use std::collections::HashSet;
// TODO: rename to calculation and further split into submodules

use generational_arena::Arena;

use super::circuit::{Circuit, CircuitIndex, Gate, GateIndex, GateKind};

#[derive(Clone)]
pub(crate) struct Receiver {
    pub(crate) gate: Option<GateIndex>,
    producer: Option<ProducerIdx>,
}

#[derive(Clone)]
pub(crate) struct Producer {
    pub(crate) gate: Option<GateIndex>,
    dependants: HashSet<ReceiverIdx>,
    pub(crate) value: bool,
}

// TODO: properly deal with removing gates so that it doesnt panic when gates are removed

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateInputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ()); // unit at the end so that it cannot be constructed outside of this module
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct GateOutputNodeIdx(pub(crate) GateIndex, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitInputNodeIdx(pub(crate) CircuitIndex, pub(crate) usize, ());
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub(crate) struct CircuitOutputNodeIdx(pub(crate) CircuitIndex, pub(crate) usize, ());

impl Receiver {
    pub(crate) fn new(gate: Option<GateIndex>) -> Self {
        Self { gate, producer: None }
    }

    pub(crate) fn producer(&self) -> Option<ProducerIdx> {
        self.producer
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

pub(crate) fn circuit_input_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitInputNodeIdx> + '_ {
    (0..circuit.inputs.len()).map(|i| CircuitInputNodeIdx(circuit.index, i, ()))
}
pub(crate) fn circuit_output_indexes(circuit: &Circuit) -> impl Iterator<Item = CircuitOutputNodeIdx> + '_ {
    (0..circuit.outputs.len()).map(|i| CircuitOutputNodeIdx(circuit.index, i, ()))
}
pub(crate) fn circuit_output_values<'c>(circuits: &'c Arena<Circuit>, gates: &'c Arena<Gate>, circuit: CircuitIndex) -> impl Iterator<Item = bool> + 'c {
    // TODO: take this logic to check the producer of a receiver node out from everywhere it is used and put it into a method
    circuits[circuit].outputs.iter().map(|output| if let Some(producer) = output.producer() { get_producer(circuits, gates, producer).value } else { false })
}

// TODO: test connection, replacing old connection
pub(crate) fn connect(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, producer_idx: ProducerIdx, receiver_idx: ReceiverIdx) {
    if let Some(old_producer) = get_receiver(circuits, gates, receiver_idx).producer {
        get_receiver_mut(circuits, gates, receiver_idx).producer = None;
        get_producer_mut(circuits, gates, old_producer).dependants.remove(&receiver_idx);
    }

    get_receiver_mut(circuits, gates, receiver_idx).producer = Some(producer_idx);
    get_producer_mut(circuits, gates, producer_idx).dependants.insert(receiver_idx);
}
// TODO: test removing, make sure it removes from both to keep in sync
pub(crate) fn disconnect(gates: &mut Arena<Gate>, producer: ProducerIdx, receiver: ReceiverIdx) {
    todo!()
}

pub(crate) fn get_receiver<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, index: ReceiverIdx) -> &'c Receiver {
    match index {
        ReceiverIdx::CO(co) => &circuits[co.0].outputs[co.1],
        ReceiverIdx::GI(gi) => gate_get_input(&gates[gi.0], gi),
    }
}
pub(crate) fn get_receiver_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: ReceiverIdx) -> &'c mut Receiver {
    match index {
        ReceiverIdx::CO(co) => &mut circuits[co.0].outputs[co.1],
        ReceiverIdx::GI(gi) => gate_get_input_mut(&mut gates[gi.0], gi),
    }
}
pub(crate) fn get_producer<'a: 'c, 'b: 'c, 'c>(circuits: &'a Arena<Circuit>, gates: &'b Arena<Gate>, index: ProducerIdx) -> &'c Producer {
    match index {
        ProducerIdx::CI(ci) => &circuits[ci.0].inputs[ci.1],
        ProducerIdx::GO(go) => gate_get_output(&gates[go.0], go),
    }
}
pub(crate) fn get_producer_mut<'a: 'c, 'b: 'c, 'c>(circuits: &'a mut Arena<Circuit>, gates: &'b mut Arena<Gate>, index: ProducerIdx) -> &'c mut Producer {
    match index {
        ProducerIdx::CI(ci) => &mut circuits[ci.0].inputs[ci.1],
        ProducerIdx::GO(go) => gate_get_output_mut(&mut gates[go.0], go),
    }
}

pub(crate) fn toggle_input(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, circuit: CircuitIndex, i: usize) {
    assert!(i < circuits[circuit].inputs.len(), "toggle input out of range of number of inputs");
    set_input(circuits, gates, CircuitInputNodeIdx(circuit, i, ()), !get_producer(circuits, gates, CircuitInputNodeIdx(circuit, i, ()).into()).value);
}
pub(crate) fn set_input(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, ci: CircuitInputNodeIdx, value: bool) {
    set_producer_value(circuits, gates, ci.into(), value);
    // TODO: consider whether or not this is supposed to call update()
}

pub(crate) fn update(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>) {
    let mut stack: Vec<_> = gates.iter().map(|(i, _)| i).collect();
    let mut changed = HashSet::new();
    while let Some(gate) = stack.pop() {
        if changed.contains(&gate) {
            continue;
        }

        let gate_changed = update_gate(circuits, gates, &mut stack, gate);
        if gate_changed {
            changed.insert(gate);
        }
    }
}
fn set_producer_value(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, index: ProducerIdx, value: bool) {
    let producer = get_producer_mut(circuits, gates, index);
    producer.value = value;
    // caller should call update next
}

fn update_gate(circuits: &mut Arena<Circuit>, gates: &mut Arena<Gate>, update_stack: &mut Vec<GateIndex>, gate: GateIndex) -> bool {
    let gate = &gates[gate];
    let outputs = compute(circuits, gates, &gate.kind);
    assert_eq!(outputs.len(), gate.num_outputs());

    let mut changed = false;

    for (new_value, output_node) in outputs.into_iter().zip(gate_outputs(&gate).collect::<Vec<_>>().into_iter()) {
        let as_producer_index = output_node.into();
        let old_value = get_producer(circuits, gates, as_producer_index).value;
        if old_value != new_value {
            changed = true;
        }
        set_producer_value(circuits, gates, as_producer_index, new_value);

        for dependant in get_producer(circuits, gates, as_producer_index).dependants.clone() {
            // clone so that the borrow checker is happy, TODO: find better solution to this
            if let Some(gate) = get_receiver(circuits, gates, dependant).gate {
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

pub(crate) fn compute(circuits: &Arena<Circuit>, gates: &Arena<Gate>, gate: &GateKind) -> Vec<bool> {
    // TODO: merge this with update

    let get_producer_value = |producer_idx| if let Some(producer_idx) = producer_idx { get_producer(circuits, gates, producer_idx).value } else { false };
    // TODO: figure out a way for this to set its outputs
    match gate {
        GateKind::Nand([a, b], _) => vec![!(get_producer_value(a.producer) && get_producer_value(b.producer))],
        GateKind::Const(_, [o]) => vec![o.value],
        GateKind::Subcircuit(inputs, _, subcircuit) => {
            /*
            // TODO: make passthrough nodes so this does not need to happen
            let mut subcircuit = &mut circuits[*subcircuit];
            for (input_node, subcircuit_input_node) in inputs.iter().zip(circuit_input_indexes(&mut subcircuit)) {
                set_producer_value(circuits, gates, subcircuit_input_node.into(), get_producer_value(input_node.producer));
            }

            circuit_output_indexes(&subcircuit)
                .into_iter()
                .map(|output_idx| if let Some(producer) = get_receiver(circuits, gates, output_idx.into()).producer { get_producer(circuits, gates, producer).value } else { false })
                .collect()
            */
            todo!()
        }
    }
}
