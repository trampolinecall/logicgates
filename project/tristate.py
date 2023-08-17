from generate import gates, ty, layout
import basic

def tristate_buffer(width):
    @gates.make_circuit(f'tristate buffer {width}', ty.DictProduct(enable=ty.Bit(), data=ty.ListProduct(*[ty.Bit() for _ in range(width)])), ty.ListProduct(*[ty.Bit() for _ in range(width)]))
    def make(context, circuit):
        bits = [basic.tristate_buffer(context, circuit) for _ in range(width)]

        for i, bit in enumerate(bits):
            context.connect(circuit.inputs['data'][i], bit.inputs['data'])
            context.connect(circuit.inputs['enable'], bit.inputs['enable'])

            context.connect(bit.outputs, circuit.outputs[i])

        layout.ttb_flow(*map(layout.ltr_gate, bits)).apply()

    return make
