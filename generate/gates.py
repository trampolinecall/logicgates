import json

from . import bundle, serialize, ty
class Context:
    def __init__(self):
        self.connections = []
        self.toplevel_gates = None

    def set_main_circuit(self, c):
        assert c.inputs.type() == ty.ListProduct(), f'main circuit must have no inputs (has type {c.inputs.type()})'
        assert c.outputs.type() == ty.ListProduct(), 'main circuit must have no outputs (has type {c.outputs.type()})'
        self.toplevel_gates = c.gates

    def connect(self, a, b):
        if a.type() != b.type():
            raise Exception(f'connecting two bundles of different types: {a.type()} and {b.type()}')

        self.connections.append((a, b))

    def export(self, output):
        result = serialize.serialize_context(self)
        with open(output, 'w') as f:
            json.dump(result, f)

class Circuit:
    def __init__(self, input_type, output_type):
        self.gates = []
        self.inputs = input_type.make_bundle()
        self.outputs = output_type.make_bundle()

    def new_circuit(self, input_type, output_type):
        circuit = Circuit(input_type, output_type)
        self.gates.append(circuit)
        return circuit

    def add_gate(self, gate):
        self.gates.append(gate)

class _NandGate:
    def __init__(self):
        self.inputs = bundle.ListProduct(bundle.Bit(), bundle.Bit())
        self.outputs = bundle.Bit()

class _FalseGate:
    def __init__(self):
        self.inputs = bundle.ListProduct()
        self.outputs = bundle.Bit()

class _TrueGate:
    def __init__(self):
        self.inputs = bundle.ListProduct()
        self.outputs = bundle.Bit()

class _UnerrorGate:
    def __init__(self):
        self.inputs = bundle.Bit()
        self.outputs = bundle.Bit()

class GateNodes:
    def __init__(self, inputs, outputs):
        self.inputs = inputs
        self.outputs = outputs

def nand(context, parent):
    gate = _NandGate()
    parent.add_gate(gate)
    return GateNodes(gate.inputs, gate.outputs)

def false(context, parent):
    gate = _FalseGate()
    parent.add_gate(gate)
    return GateNodes(gate.inputs, gate.outputs)

def true(context, parent):
    gate = _TrueGate()
    parent.add_gate(gate)
    return GateNodes(gate.inputs, gate.outputs)

def unerror(context, parent):
    gate = _UnerrorGate()
    parent.add_gate(gate)
    return GateNodes(gate.inputs, gate.outputs)
