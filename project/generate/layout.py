INDIVIDUAL_GATE_RECT = (200, 200) # TODO: sync this with constants from rust?

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

class Flow:
    def __init__(self, direction, *children):
        self.direction = direction
        self.children = children

    def size(self):
        children_sizes = [child.size() for child in self.children]
        children_widths = [size[0] for size in children_sizes]
        children_heights = [size[1] for size in children_sizes]

        match self.direction:
            case 'ltr' | 'rtl':
                width = sum(children_widths)
                height = max(children_heights)
            case 'ttb' | 'btt':
                width = max(children_widths)
                height = sum(children_heights)

            case _:
                raise Exception(f'invalid flow direction \'{self.direction}\'')

        return (width, height)

    def apply(self, center=(0, 0)):
        center_x, center_y = center
        size_x, size_y = self.size()

        match self.direction:
            case 'ltr':
                cur_pos = center_x - size_x / 2
            case 'rtl':
                cur_pos = center_x + size_x / 2
            case 'ttb':
                cur_pos = center_y - size_y / 2
            case 'btt':
                cur_pos = center_y + size_y / 2

        for child in self.children:
            match self.direction:
                case 'ltr':
                    child.apply((cur_pos + child.size()[0] / 2, center_y))
                    cur_pos += child.size()[0]
                case 'rtl':
                    child.apply((cur_pos - child.size()[0] / 2, center_y))
                    cur_pos -= child.size()[0]
                case 'ttb':
                    child.apply((center_x, cur_pos + child.size()[1] / 2))
                    cur_pos += child.size()[1]
                case 'btt':
                    child.apply((center_x, cur_pos - child.size()[1] / 2))
                    cur_pos -= child.size()[1]

def snake(first_direction, second_direction, row_len, *things):
    chunked = [things[n:n + row_len] for n in range(0, len(things), row_len)]

    cur_direction = first_direction
    chunk_layouts = []
    for chunk in chunked:
        chunk_layouts.append(Flow(cur_direction, *map(lambda chunk: chunk(cur_direction), chunk)))
        cur_direction = cur_direction[::-1]

    return Flow(second_direction, *chunk_layouts)

def ltr_flow(*children):
    return Flow('ltr', *children)
def rtl_flow(*children):
    return Flow('rtl', *children)
def ttb_flow(*children):
    return Flow('ttb', *children)
def btt_flow(*children):
    return Flow('btt', *children)

def ltr_gate(gate):
    return Gate(gate, 'ltr')
def rtl_gate(gate):
    return Gate(gate, 'rtl')
def ttb_gate(gate):
    return Gate(gate, 'ttb')
def btt_gate(gate):
    return Gate(gate, 'btt')
