from generate import gates, ty, bundle, layout

nand = gates.nand
false = gates.false
true = gates.true
unerror = gates.unerror
button = gates.button
tristate_buffer = gates.tristate_buffer

@gates.make_circuit('and', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def and_(context, circuit):
    nand_gate = nand(context, circuit)
    not_gate = not_(context, circuit)

    context.connect(circuit.inputs, nand_gate.inputs)
    context.connect(nand_gate.outputs, not_gate.inputs)
    context.connect(not_gate.outputs, circuit.outputs)

    layout.ltr_flow(layout.ltr_gate(nand_gate), layout.ltr_gate(not_gate)).apply()

@gates.make_circuit('or', ty.ListProduct(ty.Bit(), ty.Bit()), ty.Bit())
def or_(context, circuit):
    nand_gate = nand(context, circuit)
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
    n = nand(context, circuit)

    context.connect(bundle.ListProduct(circuit.inputs, circuit.inputs), n.inputs)
    context.connect(n.outputs, circuit.outputs)

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
    first_nand = nand(context, circuit)

    context.connect(circuit.inputs, first_nand.inputs)

    nand0 = nand(context, circuit)
    nand1 = nand(context, circuit)
    context.connect(bundle.ListProduct(circuit.inputs[0], first_nand.outputs), nand0.inputs)
    context.connect(bundle.ListProduct(circuit.inputs[1], first_nand.outputs), nand1.inputs)

    final_nand = nand(context, circuit)
    context.connect(bundle.ListProduct(nand0.outputs, nand1.outputs), final_nand.inputs)
    context.connect(final_nand.outputs, circuit.outputs)

    layout.ltr_flow(
        layout.ltr_gate(first_nand),
        layout.ttb_flow(layout.ltr_gate(nand0), layout.ltr_gate(nand1)),
        layout.ltr_gate(final_nand),
    ).apply()
