INDIVIDUAL_GATE_RECT = (100, 100) # TODO: sync this with constants from rust?

class GateLayout:
    def __init__(self, position, direction):
        self.position = position
        self.direction = direction

# TODO:
# class Gate:
#     def __init__(self, gate):
#         self.gate = gate
#
#     def size(self):
#         return INDIVIDUAL_GATE_RECT
#
#     def apply(self, center=(0, 0)):
#         self.gate.layout = GateLayout(center, )

class LTRChain:
    def __init__(self, *gates):
        self.gates = gates

    def size(self):
        return (INDIVIDUAL_GATE_RECT[0] * len(self.gates), INDIVIDUAL_GATE_RECT[1])

    def apply(self, center=(0, 0)):
        center_x, center_y = center
        size_x, size_y = self.size()

        cur_x = center_x - size_x / 2
        for child in self.gates:
            child.layout = GateLayout((cur_x, center_y), 'ltr')
            cur_x += INDIVIDUAL_GATE_RECT[0]
