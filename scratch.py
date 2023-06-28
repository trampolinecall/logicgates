from generate import gates, ty, bundle, layout, utils

# TODO: do layout for the ones that dont have it

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

@gates.make_circuit('adder', ty.DictProduct(a=ty.Bit(), b=ty.Bit(), carry=ty.Bit()), ty.DictProduct(carry=ty.Bit(), result=ty.Bit()))
def adder(context, circuit):
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

# @gates.make_circuit(, in; [; a8; ty.Bit(), a4; ty.Bit(), a2; ty.Bit(), a1; ty.Bit(), b8; ty.Bit(), b4; ty.Bit(), b2; ty.Bit(), b1; ty.Bit(), carry; ty.Bit(), ] out; [; carry; ty.Bit(), result; [4]ty.Bit(), ])
# def adder4(context, circuit):
#
#     adder_gate = adder(context, circuit) adder1ins; [; a; ty.Bit(), b; ty.Bit(), carry; ty.Bit()] adder1res; [; carry; ty.Bit(), result; ty.Bit()]
#     adder_gate = adder(context, circuit) adder2ins; [; a; ty.Bit(), b; ty.Bit(), carry; ty.Bit()] adder2res; [; carry; ty.Bit(), result; ty.Bit()]
#     adder_gate = adder(context, circuit) adder4ins; [; a; ty.Bit(), b; ty.Bit(), carry; ty.Bit()] adder4res; [; carry; ty.Bit(), result; ty.Bit()]
#     adder_gate = adder(context, circuit) adder8ins; [; a; ty.Bit(), b; ty.Bit(), carry; ty.Bit()] adder8res; [; carry; ty.Bit(), result; ty.Bit()]
#
#     context.connect([in.a8, in.a4, in.a2, in.a1] [adder8ins.a, adder4ins.a, adder2ins.a, adder1ins.a])
#     context.connect([in.b8, in.b4, in.b2, in.b1] [adder8ins.b, adder4ins.b, adder2ins.b, adder1ins.b])
#
#     context.connect(in.carry adder1ins.carry)
#     context.connect(adder1res.carry adder2ins.carry)
#     context.connect(adder2res.carry adder4ins.carry)
#     context.connect(adder4res.carry adder8ins.carry)
#
#     context.connect([adder8res.result, adder4res.result, adder2res.result, adder1res.result] circuit.outputs.result)
#     context.connect(adder8res.carry circuit.outputs.carry)
#
#     context.connect(0 circuit.outputs.carry)

@gates.make_circuit('clock', ty.DictProduct(enable=ty.Bit(), manual=ty.Bit()), ty.Bit())
# TODO: adjustable speed
def clock(context, circuit):
    unerror = gates.unerror(context, circuit)
    nots = [not_(context, circuit) for _ in range(19)]

    utils.connect_chain(context, unerror, *nots, unerror)

    enable_and = and_(context, circuit)
    context.connect(bundle.ListProduct(nots[-1].outputs, circuit.inputs['enable']), enable_and.inputs)

    manual_or = or_(context, circuit)
    context.connect(bundle.ListProduct(enable_and.outputs, circuit.inputs['manual']), manual_or.inputs)

    context.connect(manual_or.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.snake('ltr', 'ttb', 5, lambda direction: layout.Gate(unerror, direction), *map(lambda g: lambda direction: layout.Gate(g, direction), nots)),
        layout.ltr_gate(enable_and),
        layout.ltr_gate(manual_or),
    ).apply()

@gates.make_circuit('main', ty.ListProduct(), ty.ListProduct())
def main(context, circuit):
    enable_button = gates.button(context, circuit)
    manual_button = gates.button(context, circuit)
    c = clock(context, circuit)

    context.connect(bundle.DictProduct(enable=enable_button.outputs, manual=manual_button.outputs), c.inputs)

    buttons = [gates.button(context, circuit) for _ in range(3)]
    adder_gate = adder(context, circuit)
    context.connect(bundle.DictProduct(a=buttons[0].outputs, b=buttons[1].outputs, carry=buttons[2].outputs), adder_gate.inputs)

    layout.ltr_flow(
        layout.ttb_flow(layout.ltr_gate(enable_button), layout.ltr_gate(manual_button)),
        layout.ltr_gate(c),
        layout.ttb_flow(*map(layout.ltr_gate, buttons)),
        layout.ltr_gate(adder_gate),
    ).apply()

if __name__ == '__main__':
    gates.export(main, 'project.json')
