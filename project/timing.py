from generate import gates, ty, bundle, layout, utils
import basic

import math

def clock(length):
    @gates.make_circuit('clock', ty.DictProduct(enable=ty.Bit(), manual=ty.Bit()), ty.Bit())
    def make(context, circuit):
        unerror = basic.unerror(context, circuit)
        nots = [basic.not_(context, circuit) for _ in range(length)]

        utils.connect_chain(context, unerror, *nots, unerror)

        enable_and = basic.and_(context, circuit)
        context.connect(bundle.ListProduct(nots[-1].outputs, circuit.inputs['enable']), enable_and.inputs)

        manual_or = basic.or_(context, circuit)
        context.connect(bundle.ListProduct(enable_and.outputs, circuit.inputs['manual']), manual_or.inputs)

        context.connect(manual_or.outputs, circuit.outputs)

        layout.ltr_flow(
            layout.snake('ltr', 'ttb', math.floor(math.sqrt(length)), lambda direction: layout.Gate(unerror, direction), *map(lambda g: lambda direction: layout.Gate(g, direction), nots)),
            layout.ltr_gate(enable_and),
            layout.ltr_gate(manual_or),
        ).apply()

    return make
