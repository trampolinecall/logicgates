use crate::{simulation::NodeKey, view::Drawing, LogicGates};

pub(crate) struct ConnectionDrawing {
    pub(crate) node1: NodeKey,
    pub(crate) node2: NodeKey,
    pub(crate) pos1: nannou::geom::Vec2,
    pub(crate) pos2: nannou::geom::Vec2,
}

const CONNECTION_RAD: f32 = super::node::CIRCLE_RAD / 2.0;

impl Drawing for ConnectionDrawing {
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        let color = super::node::node_color(&simulation.simulation.nodes, self.node1, false);
        draw.line().start(self.pos1).end(self.pos2).weight(CONNECTION_RAD).color(color);
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        None // TODO
    }
}
