from generate import gates, ty, bundle, layout
import timing
import arithmetic
import basic
import tristate
import memory

BIT_WIDTH = 8

@gates.make_circuit('bus', ty.ListProduct(), ty.ListProduct(*[ty.Bit() for _ in range(BIT_WIDTH)]))
def make_bus(context, circuit):
    pass

@gates.make_circuit('main', ty.ListProduct(), ty.ListProduct())
def main(context, circuit):
    enable_button = basic.button(context, circuit)
    manual_button = basic.button(context, circuit)
    clock = timing.clock(51)(context, circuit)
    context.connect(bundle.DictProduct(enable=enable_button.outputs, manual=manual_button.outputs), clock.inputs)

    bus = make_bus(context, circuit)

    a_register = memory.register(BIT_WIDTH)(context, circuit)
    a_register_tristate = tristate.tristate_buffer(BIT_WIDTH)(context, circuit)
    a_register_output_enable = basic.button(context, circuit)
    context.connect(a_register.outputs, a_register_tristate.inputs['data'])
    context.connect(a_register.inputs['data'], bus.outputs)
    context.connect(a_register_tristate.outputs, bus.outputs)
    context.connect(a_register_output_enable.outputs, a_register_tristate.inputs['enable'])

    b_register = memory.register(BIT_WIDTH)(context, circuit)
    b_register_tristate = tristate.tristate_buffer(BIT_WIDTH)(context, circuit)
    b_register_output_enable = basic.button(context, circuit)
    context.connect(b_register.outputs, b_register_tristate.inputs['data'])
    context.connect(b_register.inputs['data'], bus.outputs)
    context.connect(b_register_tristate.outputs, bus.outputs)
    context.connect(b_register_output_enable.outputs, b_register_tristate.inputs['enable'])

    adder = arithmetic.adder_many(BIT_WIDTH)(context, circuit)
    adder_tristate = tristate.tristate_buffer(BIT_WIDTH)(context, circuit)
    adder_output_enable = basic.button(context, circuit)
    false_carry = basic.false(context, circuit)
    context.connect(bundle.DictProduct(a=a_register.outputs, b=b_register.outputs, carry=false_carry.outputs), adder.inputs)
    context.connect(adder.outputs['result'], adder_tristate.inputs['data'])
    context.connect(adder_tristate.outputs, bus.outputs)
    context.connect(adder_output_enable.outputs, adder_tristate.inputs['enable'])

    layout.ttb_flow(
        layout.ltr_flow(
            layout.ttb_flow(
                layout.ltr_flow(layout.ttb_gate(enable_button), layout.ttb_gate(manual_button)),
                layout.ttb_gate(clock),
            ),
        ),
        layout.ltr_gate(bus),
        layout.ltr_flow(
            layout.btt_flow(layout.btt_gate(a_register), layout.btt_gate(a_register_output_enable), layout.btt_gate(a_register_tristate)),
            layout.btt_flow(layout.btt_gate(b_register), layout.btt_gate(b_register_output_enable), layout.btt_gate(b_register_tristate)),
            layout.btt_flow(layout.btt_gate(false_carry), layout.btt_gate(adder), layout.btt_gate(adder_output_enable), layout.btt_gate(adder_tristate)),
        ),
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
    gates.export(main, 'project.json')
