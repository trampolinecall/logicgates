def connect_chain(context, *things):
    for (a, b) in zip(things, things[1:]):
        context.connect(a.outputs, b.inputs)
