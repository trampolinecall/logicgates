from generate import gates, ty, layout
import basic

# if select is off, the result is a, and if select is on, the result is b
@gates.make_circuit('multiplexer', ty.DictProduct(select=ty.Bit(), a=ty.Bit(), b=ty.Bit()), ty.Bit())
def multiplexer(context, circuit):
    select_not = basic.not_(context, circuit)
    context.connect(circuit.inputs['select'], select_not.inputs)

    a_and = basic.and_(context, circuit)
    context.connect(circuit.inputs['a'], a_and.inputs[0])
    context.connect(select_not.outputs, a_and.inputs[1])

    b_and = basic.and_(context, circuit)
    context.connect(circuit.inputs['b'], b_and.inputs[0])
    context.connect(circuit.inputs['select'], b_and.inputs[1])

    final_or = basic.or_(context, circuit)
    context.connect(a_and.outputs, final_or.inputs[0])
    context.connect(b_and.outputs, final_or.inputs[1])
    context.connect(final_or.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ttb_flow(
            layout.ltr_gate(a_and),
            layout.ltr_flow(layout.ltr_gate(select_not), layout.ltr_gate(b_and)),
        ),
        layout.ltr_gate(final_or),
    ).apply()
