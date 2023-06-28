INDIVIDUAL_GATE_RECT = (100, 100) # TODO: sync this with constants from rust?

class GateLayout:
    def __init__(self, position, direction):
        self.position = position
        self.direction = direction

class Gate:
    def __init__(self, gate, direction):
        self.gate = gate
        self.direction = direction

    def size(self):
        return INDIVIDUAL_GATE_RECT

    def apply(self, center=(0, 0)):
        self.gate.layout = GateLayout(center, self.direction)

class HorizontalFlow:
    def __init__(self, *children):
        self.children = children

    def size(self):
        children_sizes = [child.size() for child in self.children]
        width = sum([size[0] for size in children_sizes])
        height = max(children_sizes, key=lambda size: size[1])
        return (width, height)

    def apply(self, center=(0, 0)):
        center_x, center_y = center
        size_x, size_y = self.size()

        cur_x = center_x - size_x / 2
        for child in self.children:
            child.apply((cur_x, center_y))
            cur_x += child.size()[0]

def ltr_gate(gate):
    return Gate(gate, 'ltr')
def rtl_gate(gate):
    return Gate(gate, 'rtl')
def ttb_gate(gate):
    return Gate(gate, 'ttb')
def btt_gate(gate):
    return Gate(gate, 'btt')
