use nannou::prelude::*;

use crate::{
    simulation::{logic, NodeKey, NodeMap},
    theme::Theme,
    view::Drawing,
    LogicGates,
};

pub(crate) struct NodeDrawing {
    pub(crate) key: NodeKey,
    pub(crate) location: nannou::geom::Vec2,
}

impl Drawing for NodeDrawing {
    fn draw(&self, simulation: &LogicGates, draw: &nannou::Draw, hovered: Option<&dyn Drawing>) {
        if let Some(hovered) = hovered {
            if std::ptr::eq(hovered, self) {
                // TODO: fix clippy lint about this
                draw.ellipse().xy(self.location).radius(Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist).color(Theme::DEFAULT.node_hover_color);
            }
        }

        let color = node_color(&simulation.simulation.nodes, self.key, true);
        draw.ellipse().xy(self.location).radius(Theme::DEFAULT.node_rad).color(color);
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if self.location.distance(mouse_pos) < Theme::DEFAULT.node_rad + Theme::DEFAULT.node_hover_dist {
            return Some(self);
        }
        None
    }
}

// TODO: also refactor so that this does not need to be pub(crate)
pub(crate) fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb<u8> {
    fn value_to_color(v: logic::Value) -> Rgb<u8> {
        match v {
            logic::Value::H => Theme::DEFAULT.on_color,
            logic::Value::L => Theme::DEFAULT.off_color,
            logic::Value::Z => Theme::DEFAULT.high_impedance_color,
            logic::Value::X => Theme::DEFAULT.err_color,
        }
    }
    if use_production {
        if let Some(v) = logic::get_node_production(nodes, node) {
            value_to_color(v)
        } else {
            value_to_color(logic::get_node_value(nodes, node))
        }
    } else {
        value_to_color(logic::get_node_value(nodes, node))
    }
}
