from . import gates, bundle

class Nodes:
    def __init__(self):
        self.nodes = {}
        self.cur_num = 0

    def node_for_bit(self, b):
        assert isinstance(b, bundle.Bit)
        if b in self.nodes:
            return self.nodes[b]
        else:
            b_num = self.cur_num
            self.cur_num += 1
            self.nodes[b] = b_num
            return b_num

def serialize_context(context):
    nodes = Nodes()
    return {
        'connections': sum([serialize_connection(nodes, connection) for connection in context.connections], []),
        'toplevel_gates': [serialize_gate(nodes, gate) for gate in context.toplevel_gates],
    }

def serialize_gate(nodes, gate):
    if isinstance(gate, gates.Circuit):
        return {'type': 'subcircuit', 'inputs': convert_bundle(nodes, gate.inputs), 'outputs': convert_bundle(nodes, gate.outputs), 'gates': [serialize_gate(nodes, gate) for gate in gate.gates]}
    elif isinstance(gate, gates._NandGate):
        return {'type': 'nand', 'inputs': convert_bundle(nodes, gate.inputs), 'outputs': convert_bundle(nodes, gate.outputs)}
    elif isinstance(gate, gates._FalseGate):
        return {'type': 'false', 'inputs': convert_bundle(nodes, gate.inputs), 'outputs': convert_bundle(nodes, gate.outputs)}
    elif isinstance(gate, gates._TrueGate):
        return {'type': 'true', 'inputs': convert_bundle(nodes, gate.inputs), 'outputs': convert_bundle(nodes, gate.outputs)}
    elif isinstance(gate, gates._UnerrorGate):
        return {'type': 'unerror', 'inputs': convert_bundle(nodes, gate.inputs), 'outputs': convert_bundle(nodes, gate.outputs)}
    else:
        raise Exception(f'invalid gate {gate}')

def serialize_connection(nodes, connection):
    start = connection[0]
    end = connection[1]

    start_bundle = convert_bundle(nodes, start)
    end_bundle = convert_bundle(nodes, end)

    assert len(start_bundle) == len(end_bundle)

    return [[a, b] for (a, b) in zip(start_bundle, end_bundle)]

def convert_bundle(nodes, b):
    if isinstance(b, bundle.Bit):
        return [nodes.node_for_bit(b)]
    elif isinstance(b, bundle.ListProduct):
        return sum([convert_bundle(nodes, subb) for subb in b.fields], [])
    elif isinstance(b, bundle.DictProduct):
        return sum([convert_bundle(nodes, subb) for (_, subb) in sorted(b.fields.items())], [])
    else:
        raise Exception(f'invalid bundle {b}')
