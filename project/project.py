from generate import gates, ty, bundle, layout
import timing
import arithmetic
import basic
import memory

@gates.make_circuit('main', ty.ListProduct(), ty.ListProduct())
def main(context, circuit):
    enable_button = basic.button(context, circuit)
    manual_button = basic.button(context, circuit)
    c = timing.clock(51)(context, circuit)

    context.connect(bundle.DictProduct(enable=enable_button.outputs, manual=manual_button.outputs), c.inputs)

    buttons = [basic.button(context, circuit) for _ in range(11)]
    adder_gate = arithmetic.adder_many(5)(context, circuit)
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

@gates.make_circuit('main', ty.ListProduct(), ty.ListProduct())
def main_d(context, circuit):
    WIDTH = 8
    d_buttons = [basic.button(context, circuit) for _ in range(WIDTH)]
    s_button = basic.button(context, circuit)
    c_button = basic.button(context, circuit)

    reg = memory.register(WIDTH)(context, circuit)

    for button, reg_in in zip(d_buttons, reg.inputs['data'].fields):
        context.connect(button.outputs, reg_in)

    context.connect(c_button.outputs, reg.inputs['clock'])
    context.connect(s_button.outputs, reg.inputs['store'])

if __name__ == '__main__':
    gates.export(main_d, 'project.json')
