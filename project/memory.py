from generate import gates, ty, layout
import basic
import utils

@gates.make_circuit('sr', ty.DictProduct(set=ty.Bit(), reset=ty.Bit()), ty.Bit())
def sr_latch(context, circuit):
    not_gate = basic.not_(context, circuit)
    unerror_gate = basic.unerror(context, circuit)
    or_gate = basic.or_(context, circuit)
    and_gate = basic.and_(context, circuit)

    context.connect(circuit.inputs['set'], or_gate.inputs[1])
    context.connect(or_gate.outputs, and_gate.inputs[0])
    context.connect(and_gate.outputs, unerror_gate.inputs)
    context.connect(unerror_gate.outputs, or_gate.inputs[0])
    context.connect(circuit.inputs['reset'], not_gate.inputs)
    context.connect(not_gate.outputs, and_gate.inputs[1])
    context.connect(and_gate.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ttb_flow(
            layout.ltr_gate(not_gate),
            layout.ltr_flow(layout.ltr_gate(unerror_gate), layout.ltr_gate(or_gate)),
        ),
        layout.ltr_gate(and_gate),
    ).apply()

@gates.make_circuit('d latch', ty.DictProduct(data=ty.Bit(), store=ty.Bit()), ty.Bit())
def d_latch(context, circuit):
    sr = sr_latch(context, circuit)

    set_and = basic.and_(context, circuit)
    reset_and = basic.and_(context, circuit)

    data_inverse = basic.not_(context, circuit)

    context.connect(circuit.inputs['store'], set_and.inputs[0])
    context.connect(circuit.inputs['data'], set_and.inputs[1])

    context.connect(circuit.inputs['data'], data_inverse.inputs)
    context.connect(circuit.inputs['store'], reset_and.inputs[0])
    context.connect(data_inverse.outputs, reset_and.inputs[1])

    context.connect(set_and.outputs, sr.inputs['set'])
    context.connect(reset_and.outputs, sr.inputs['reset'])

    context.connect(sr.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ttb_flow(
            layout.ltr_gate(set_and),
            layout.ltr_flow(layout.ltr_gate(data_inverse), layout.ltr_gate(reset_and)),
        ),
        layout.ltr_gate(sr),
    ).apply()

@gates.make_circuit('d flip flop', ty.DictProduct(data=ty.Bit(), clock=ty.Bit()), ty.Bit())
def d_flip_flop(context, circuit):
    clock_not = basic.not_(context, circuit)
    context.connect(circuit.inputs['clock'], clock_not.inputs)

    d_before = d_latch(context, circuit)
    context.connect(clock_not.outputs, d_before.inputs['store'])
    context.connect(circuit.inputs['data'], d_before.inputs['data'])

    d_main = d_latch(context, circuit)
    context.connect(circuit.inputs['clock'], d_main.inputs['store'])
    context.connect(d_before.outputs, d_main.inputs['data'])

    context.connect(d_main.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ltr_gate(clock_not),
        layout.ltr_gate(d_before),
        layout.ltr_gate(d_main),
    ).apply()

@gates.make_circuit('1 bit register', ty.DictProduct(data=ty.Bit(), store=ty.Bit(), clock=ty.Bit()), ty.Bit())
def register1(context, circuit):
    multi = utils.multiplexer(context, circuit)
    d = d_flip_flop(context, circuit)

    loop_unerror = basic.unerror(context, circuit)
    context.connect(d.outputs, loop_unerror.inputs)

    store = circuit.inputs['store']
    data = circuit.inputs['data']
    current_output = loop_unerror.outputs
    # if store, choose data, but if not store, choose current output
    context.connect(store, multi.inputs['select'])
    context.connect(current_output, multi.inputs['a'])
    context.connect(data, multi.inputs['b'])

    context.connect(multi.outputs, d.inputs['data'])
    context.connect(circuit.inputs['clock'], d.inputs['clock'])

    final_unerror = basic.unerror(context, circuit) # initially registers have an error value saved so this turns it into a 0
    context.connect(d.outputs, final_unerror.inputs)
    context.connect(final_unerror.outputs, circuit.outputs)

    layout.ltr_flow(layout.ltr_gate(loop_unerror), layout.ltr_gate(multi), layout.ltr_gate(d), layout.ltr_gate(final_unerror)).apply()

def register(width):
    @gates.make_circuit(f'{width} bit register', ty.DictProduct(data=ty.ListProduct(*[ty.Bit() for _ in range(width)]), store=ty.Bit(), clock=ty.Bit()), ty.ListProduct(*[ty.Bit() for _ in range(width)]))
    def make(context, circuit):
        bits = [register1(context, circuit) for _ in range(width)]

        for i, bit in enumerate(bits):
            context.connect(circuit.inputs['data'][i], bit.inputs['data'])
            context.connect(circuit.inputs['clock'], bit.inputs['clock'])
            context.connect(circuit.inputs['store'], bit.inputs['store'])

            context.connect(bit.outputs, circuit.outputs[i])

        layout.ttb_flow(*map(layout.ltr_gate, bits)).apply()

    return make
