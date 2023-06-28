import json
import functools

from . import bundle, serialize, ty, layout

class Context:
    def __init__(self):
        self.connections = []
        self.toplevel_gates = None

    def set_main_circuit(self, make_main):
        class MainCircuitHolder:
            def __init__(self):
                self.main = None

            def new_circuit(self, name, input_type, output_type):
                assert self.main is None, "main circuit can only create 1 circuit"
                self.main = Circuit(name, input_type, output_type)
                return self.main

        holder = MainCircuitHolder()
        make_main(self, holder)
        circuit = holder.main

        assert circuit.inputs.type() == ty.ListProduct(), f'main circuit must have no inputs (has type {circuit.inputs.type()})'
        assert circuit.outputs.type() == ty.ListProduct(), f'main circuit must have no outputs (has type {circuit.outputs.type()})'
        self.toplevel_gates = circuit.gates

    def connect(self, a, b):
        if a.type() != b.type():
            raise Exception(f'connecting two bundles of different types: {a.type()} and {b.type()}')

        self.connections.append((a, b))

    def export(self, output):
        result = serialize.serialize_context(self)
        with open(output, 'w') as f:
            json.dump(result, f)

class Circuit:
    def __init__(self, name, input_type, output_type):
        self.name = name
        self.inputs = input_type.make_bundle()
        self.outputs = output_type.make_bundle()
        self.layout = layout.GateLayout((0, 0), 'ltr')
        self.gates = []

    def new_circuit(self, name, input_type, output_type):
        circuit = Circuit(name, input_type, output_type)
        self.gates.append(circuit)
        return circuit

    def add_gate(self, gate):
        self.gates.append(gate)

class _NandGate:
    def __init__(self):
        self.inputs = bundle.ListProduct(bundle.Bit(), bundle.Bit())
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

class _FalseGate:
    def __init__(self):
        self.inputs = bundle.ListProduct()
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

class _TrueGate:
    def __init__(self):
        self.inputs = bundle.ListProduct()
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

class _UnerrorGate:
    def __init__(self):
        self.inputs = bundle.Bit()
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

class _Button:
    def __init__(self):
        self.inputs = bundle.ListProduct()
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

class _TristateBuffer:
    def __init__(self):
        self.inputs = bundle.DictProduct(data=bundle.Bit(), enable=bundle.Bit())
        self.outputs = bundle.Bit()
        self.layout = layout.GateLayout((0, 0), 'ltr')

def make_circuit(name, input_type, output_type):
    def decorator(func):
        @functools.wraps(func)
        def circuit_wrapper(context, parent):
            c = parent.new_circuit(name, input_type, output_type)
            func(context, c)
            return c

        return circuit_wrapper

    return decorator

def nand(context, parent):
    gate = _NandGate()
    parent.add_gate(gate)
    return gate

def false(context, parent):
    gate = _FalseGate()
    parent.add_gate(gate)
    return gate

def true(context, parent):
    gate = _TrueGate()
    parent.add_gate(gate)
    return gate

def unerror(context, parent):
    gate = _UnerrorGate()
    parent.add_gate(gate)
    return gate

def button(context, parent):
    gate = _Button()
    parent.add_gate(gate)
    return gate

def tristate_buffer(context, parent):
    gate = _TristateBuffer()
    parent.add_gate(gate)
    return gate

def export(main, output):
    context = Context()
    context.set_main_circuit(main)
    context.export(output)
