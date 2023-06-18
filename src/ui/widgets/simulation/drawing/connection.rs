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
        let mut line = draw.line().start(self.pos1).end(self.pos2).weight(CONNECTION_RAD).color(color);

        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                line = line.weight(CONNECTION_RAD + 5.0); // TODO: constants for weight of hovered connection
                                                          // TODO: sync weight of hovered connection with hover distance and do the same for all other things too
            }
        }

        line.finish()
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if min_dist_squared((self.pos1, self.pos2), mouse_pos) < 4.0_f32.powf(2.0) {
            // TODO: hover distance
            Some(self)
        } else {
            None
        }
    }
}

fn min_dist_squared(line_segment: (nannou::geom::Vec2, nannou::geom::Vec2), point: nannou::geom::Vec2) -> f32 {
    let (a, b) = line_segment;

    let len_squared = a.distance_squared(b);
    if len_squared == 0.0 {
        point.distance_squared(a)
    } else {
        // project point onto line segment and return distance to that projected point
        let t = (point - a).dot(b - a) / len_squared;
        let t_clamped = t.clamp(0.0, 1.0);
        let projected = a.lerp(b, t_clamped);
        point.distance_squared(projected)
    }
}
