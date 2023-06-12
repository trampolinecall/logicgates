use crate::{simulation::{NodeKey, Simulation}, ui::Widget};

pub(crate) struct ConnectionWidget {
    pub(crate) node1: NodeKey,
    pub(crate) node2: NodeKey,
    pub(crate) pos1: nannou::geom::Vec2,
    pub(crate) pos2: nannou::geom::Vec2,
}

const CONNECTION_RAD: f32 = super::node::CIRCLE_RAD / 2.0;

impl Widget for ConnectionWidget {
    fn draw(&self, simulation: &Simulation, draw: &nannou::Draw) {
        let color = super::node::node_color(&simulation.nodes, self.node1, false);
        draw.line().start(self.pos1).end(self.pos2).weight(CONNECTION_RAD).color(color);
    }
}
