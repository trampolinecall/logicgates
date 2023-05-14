mod location;
mod render;

pub(crate) use render::render;

use crate::simulation::{self, CircuitMap, Gate, GateKey, GateMap, NodeMap};

#[derive(Copy, Clone)]
enum Shape {
    Line { radius: f32, start: nannou::prelude::Vec2, end: nannou::prelude::Vec2 },
    Rect { rect: nannou::prelude::Rect },
    Circle { pos: nannou::prelude::Vec2, rad: f32 },
}
#[derive(Copy, Clone)]
struct DrawShape {
    shape: Shape,
    color: nannou::prelude::Rgb,
}

pub(crate) struct NodeWidget {}

pub(crate) struct GateWidget {
    location: location::GateLocation,
}

impl NodeWidget {
    pub(crate) fn new() -> NodeWidget {
        NodeWidget {}
    }
}

impl GateWidget {
    pub(crate) fn new() -> GateWidget {
        GateWidget { location: location::GateLocation::new() }
    }
}

pub(crate) fn update() {
    // TODO
}

impl DrawShape {
    fn new_for_connection(cur_pos: nannou::prelude::Vec2, adj_pos: nannou::prelude::Vec2, color: nannou::prelude::Rgb) -> DrawShape {
        DrawShape { shape: Shape::Line { radius: render::CONNECTION_RAD, start: cur_pos, end: adj_pos }, color }
    }

    fn new_for_gate(window_rect: nannou::prelude::Rect, circuits: &CircuitMap, gates: &GateMap, gate: GateKey) -> DrawShape {
        let location = &simulation::Gate::widget(circuits, gates, gate).location;
        let num_inputs = simulation::Gate::num_inputs(circuits, gates, gate);
        let num_outputs = simulation::Gate::num_outputs(circuits, gates, gate);
        let rect = render::gate_rect(window_rect, location, num_inputs, num_outputs);
        DrawShape { shape: Shape::Rect { rect }, color: render::GATE_COLOR }
        // TODO: add text
        // draw.text(gates[gate_k].name(&circuits)).xy(rect.xy()).wh(rect.wh()).center_justify().align_text_middle_y();
    }

    fn new_for_node(window_rect: nannou::prelude::Rect, circuits: &CircuitMap, gates: &GateMap, nodes: &NodeMap, node_key: simulation::NodeKey) -> DrawShape {
        let pos = render::node_pos(window_rect, circuits, gates, nodes, node_key);
        let color = render::node_color(nodes, node_key, true);

        // draw.ellipse().color(color).x_y(pos[0], pos[1]).radius(CIRCLE_RAD);
        DrawShape { shape: Shape::Circle { pos, rad: render::CIRCLE_RAD }, color }
    }

    fn render(&self, draw: &nannou::Draw) {
        match self.shape {
            Shape::Line { radius, start, end } => {
                draw.line().start(start).end(end).weight(radius).color(self.color);
            }
            Shape::Rect { rect } => {
                draw.rect().xy(rect.xy()).wh(rect.wh()).color(self.color);
            }
            Shape::Circle { pos, rad } => {
                draw.ellipse().x_y(pos[0], pos[1]).radius(rad).color(self.color);
            }
        }
    }
}
