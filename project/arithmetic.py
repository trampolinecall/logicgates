from generate import gates, ty, layout, bundle
import basic

@gates.make_circuit('adder1', ty.DictProduct(a=ty.Bit(), b=ty.Bit(), carry=ty.Bit()), ty.DictProduct(carry=ty.Bit(), result=ty.Bit()))
def adder1(context, circuit):
    a_b_xor = basic.xor(context, circuit)
    a_b_and = basic.and_(context, circuit)
    ab_carry_xor = basic.xor(context, circuit)
    ab_carry_and = basic.and_(context, circuit)
    carry_or = basic.or_(context, circuit)

    context.connect(bundle.ListProduct(circuit.inputs['a'], circuit.inputs['b']), a_b_xor.inputs)
    context.connect(bundle.ListProduct(circuit.inputs['a'], circuit.inputs['b']), a_b_and.inputs)

    context.connect(bundle.ListProduct(a_b_xor.outputs, circuit.inputs['carry']), ab_carry_xor.inputs)
    context.connect(bundle.ListProduct(a_b_xor.outputs, circuit.inputs['carry']), ab_carry_and.inputs)

    context.connect(ab_carry_xor.outputs, circuit.outputs['result'])

    context.connect(bundle.ListProduct(ab_carry_and.outputs, a_b_and.outputs), carry_or.inputs)
    context.connect(carry_or.outputs, circuit.outputs['carry'])

    layout.ltr_flow(
        layout.ttb_flow(layout.ltr_gate(a_b_xor), layout.ltr_gate(a_b_and)),
        layout.ttb_flow(layout.ltr_gate(ab_carry_xor), layout.ltr_gate(ab_carry_and)),
        layout.ltr_gate(carry_or),
    ).apply()

def adder_many(width):
    @gates.make_circuit(
        f'adder{width}',
        ty.DictProduct(a=ty.ListProduct(*[ty.Bit() for _ in range(width)]), b=ty.ListProduct(*[ty.Bit() for _ in range(width)]), carry=ty.Bit()),
        ty.DictProduct(result=ty.ListProduct(*[ty.Bit() for _ in range(width)]), carry=ty.Bit()),
    )
    def make(context, circuit):
        # ones place is at the end of each of the lists
        # TODO: make bits bundle indexed 1, 2, 4, 8, ...
        adders = [adder1(context, circuit) for _ in range(width)]

        for i in range(width):
            context.connect(circuit.inputs['a'][i], adders[i].inputs['a'])
            context.connect(circuit.inputs['b'][i], adders[i].inputs['b'])
            context.connect(adders[i].outputs['result'], circuit.outputs['result'][i])

        context.connect(circuit.inputs['carry'], adders[0].inputs['carry'])
        for i in range(width - 1):
            context.connect(adders[i].outputs['carry'], adders[i + 1].inputs['carry'])
        context.connect(adders[-1].outputs['carry'], circuit.outputs['carry'])

        layout.ltr_flow(*map(layout.ltr_gate, adders)).apply() # TODO: grid layout to make this diagonal, also figure out how to use layout without .apply()

    return make
