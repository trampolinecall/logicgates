import math

from generate import gates, ty, bundle, layout, utils

@gates.make_circuit('and', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def and_(context, circuit):
    nand_gate = gates.nand(context, circuit)
    not_gate = not_(context, circuit)

    context.connect(circuit.inputs, nand_gate.inputs)
    context.connect(nand_gate.outputs, not_gate.inputs)
    context.connect(not_gate.outputs, circuit.outputs)

    layout.ltr_flow(layout.ltr_gate(nand_gate), layout.ltr_gate(not_gate)).apply()

@gates.make_circuit('or', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def or_(context, circuit):
    nand_gate = gates.nand(context, circuit)
    not0 = not_(context, circuit)
    not1 = not_(context, circuit)

    context.connect(circuit.inputs[0], not0.inputs)
    context.connect(circuit.inputs[1], not1.inputs)

    context.connect(bundle.ListProduct(not0.outputs, not1.outputs), nand_gate.inputs)
    context.connect(nand_gate.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ttb_flow(layout.ltr_gate(not0), layout.ltr_gate(not1)),
        layout.ltr_gate(nand_gate),
    ).apply()

@gates.make_circuit('not', ty.Bit(), ty.Bit())
def not_(context, circuit):
    nand = gates.nand(context, circuit)

    context.connect(bundle.ListProduct(circuit.inputs, circuit.inputs), nand.inputs)
    context.connect(nand.outputs, circuit.outputs)

@gates.make_circuit('nor', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def nor(context, circuit):
    or_gate = or_(context, circuit)
    not_gate = not_(context, circuit)

    context.connect(circuit.inputs, or_gate.inputs)
    context.connect(or_gate.outputs, not_gate.inputs)
    context.connect(not_gate.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ltr_gate(or_gate),
        layout.ltr_gate(not_gate),
    ).apply()

@gates.make_circuit('xor', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def xor(context, circuit):
    first_nand = gates.nand(context, circuit)

    context.connect(circuit.inputs, first_nand.inputs)

    nand0 = gates.nand(context, circuit)
    nand1 = gates.nand(context, circuit)
    context.connect(bundle.ListProduct(circuit.inputs[0], first_nand.outputs), nand0.inputs)
    context.connect(bundle.ListProduct(circuit.inputs[1], first_nand.outputs), nand1.inputs)

    final_nand = gates.nand(context, circuit)
    context.connect(bundle.ListProduct(nand0.outputs, nand1.outputs), final_nand.inputs)
    context.connect(final_nand.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ltr_gate(first_nand),
        layout.ttb_flow(layout.ltr_gate(nand0), layout.ltr_gate(nand1)),
        layout.ltr_gate(final_nand),
    ).apply()

@gates.make_circuit('adder1', ty.DictProduct(a=ty.Bit(), b=ty.Bit(), carry=ty.Bit()), ty.DictProduct(carry=ty.Bit(), result=ty.Bit()))
def adder1(context, circuit):
    a_b_xor = xor(context, circuit)
    a_b_and = and_(context, circuit)
    ab_carry_xor = xor(context, circuit)
    ab_carry_and = and_(context, circuit)
    carry_or = or_(context, circuit)

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

def clock(length):
    @gates.make_circuit('clock', ty.DictProduct(enable=ty.Bit(), manual=ty.Bit()), ty.Bit())
    def make(context, circuit):
        unerror = gates.unerror(context, circuit)
        nots = [not_(context, circuit) for _ in range(length)]

        utils.connect_chain(context, unerror, *nots, unerror)

        enable_and = and_(context, circuit)
        context.connect(bundle.ListProduct(nots[-1].outputs, circuit.inputs['enable']), enable_and.inputs)

        manual_or = or_(context, circuit)
        context.connect(bundle.ListProduct(enable_and.outputs, circuit.inputs['manual']), manual_or.inputs)

        context.connect(manual_or.outputs, circuit.outputs)

        layout.ltr_flow(
            layout.snake('ltr', 'ttb', math.floor(math.sqrt(length)), lambda direction: layout.Gate(unerror, direction), *map(lambda g: lambda direction: layout.Gate(g, direction), nots)),
            layout.ltr_gate(enable_and),
            layout.ltr_gate(manual_or),
        ).apply()

    return make

@gates.make_circuit('main', ty.ListProduct(), ty.ListProduct())
def main(context, circuit):
    enable_button = gates.button(context, circuit)
    manual_button = gates.button(context, circuit)
    c = clock(51)(context, circuit)

    context.connect(bundle.DictProduct(enable=enable_button.outputs, manual=manual_button.outputs), c.inputs)

    buttons = [gates.button(context, circuit) for _ in range(11)]
    adder_gate = adder_many(5)(context, circuit)
    context.connect(
        bundle.DictProduct(
            a=bundle.ListProduct(buttons[0].outputs, buttons[1].outputs, buttons[2].outputs, buttons[3].outputs, buttons[4].outputs),
            b=bundle.ListProduct(buttons[5].outputs, buttons[6].outputs, buttons[7].outputs, buttons[8].outputs, buttons[9].outputs),
            carry=buttons[10].outputs,
        ),
        adder_gate.inputs,
    )

    layout.ltr_flow(
        layout.ttb_flow(layout.ltr_gate(enable_button), layout.ltr_gate(manual_button)),
        layout.ltr_gate(c),
        layout.ttb_flow(*map(layout.ltr_gate, buttons)),
        layout.ltr_gate(adder_gate),
    ).apply()

if __name__ == '__main__':
    gates.export(main, 'project.json')
