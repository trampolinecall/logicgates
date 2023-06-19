use nannou::prelude::*;

use crate::{
    simulation::{logic, NodeKey, NodeMap},
    view::Drawing,
    LogicGates, theme::THEME,
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
                draw.ellipse().xy(self.location).radius(THEME.node_rad + THEME.node_hover_dist).color(THEME.node_hover_color);
            }
        }

        let color = node_color(&simulation.simulation.nodes, self.key, true);
        draw.ellipse().xy(self.location).radius(THEME.node_rad).color(color);
    }

    fn find_hover(&self, mouse_pos: nannou::geom::Vec2) -> Option<&dyn Drawing> {
        if self.location.distance(mouse_pos) < THEME.node_rad + THEME.node_hover_dist {
            return Some(self);
        }
        None
    }
}

// TODO: also refactor so that this does not need to be pub(crate)
pub(crate) fn node_color(nodes: &NodeMap, node: NodeKey, use_production: bool) -> Rgb<u8> {
    fn value_to_color(v: logic::Value) -> Rgb<u8> {
        match v {
            logic::Value::H => THEME.on_color,
            logic::Value::L => THEME.off_color,
            logic::Value::Z => THEME.high_impedance_color,
            logic::Value::X => THEME.err_color,
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
